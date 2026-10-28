//! A deliberately small, original sandbox encounter using verified NPC framing.
//!
//! Positions/rules come from private local content; they are not restored AI.
//! AddNpc's unproven fields are retained from a hash-checked supplied-server
//! template. The public API consumes/returns INNER gateway packets. A defeat
//! event is emitted once; the caller owns quest progress, inventory, and saving.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs;
use std::path::{Component, Path};
use std::sync::Arc;

pub const NPC_GUID: u64 = 0x5255_5354_0000_0001;
pub const ENEMY_GUID: u64 = 0x5255_5354_0000_0002;
pub const INTERACTION_QUEST: i32 = 1;
pub const INTERACTION_ATTACK: i32 = 2;
pub const INTERACTION_RESPAWN: i32 = 3;

#[derive(Debug)]
pub struct CombatError(String);
impl fmt::Display for CombatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for CombatError {}
impl From<std::io::Error> for CombatError {
    fn from(value: std::io::Error) -> Self {
        Self(value.to_string())
    }
}
fn invalid(message: &str) -> CombatError {
    CombatError(message.to_owned())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CombatState {
    pub enemy_health: u32,
    pub defeats: u64,
}

#[derive(Debug, Default)]
pub struct CombatOutcome {
    pub replies: Vec<Vec<u8>>,
    pub changed: bool,
    pub defeated: Option<i32>,
    pub interacted: Option<u64>,
    pub feedback: Option<String>,
}
impl CombatOutcome {
    fn feedback(message: impl Into<String>) -> Self {
        Self {
            feedback: Some(message.into()),
            ..Self::default()
        }
    }
}

#[derive(Deserialize)]
struct Actor {
    definition_id: i32,
    model_id: i32,
    name: String,
    texture_alias: String,
    position: [f32; 3],
}
#[derive(Deserialize)]
struct Content {
    schema_version: u32,
    /// T853: a local filename used by existing content; the release overlay leaves this empty.
    #[serde(default)]
    template_file: String,
    /// T853: the AddNpc skeleton as hex; when non-empty, `template_file` is not read.
    /// `template_sha256` pins the bytes either way.
    #[serde(default)]
    template_hex: String,
    template_sha256: String,
    npc: Actor,
    enemy: Actor,
    max_health: u32,
    hit_damage: u32,
    range: f32,
    cooldown_ms: u64,
    #[serde(skip)]
    template: Vec<u8>,
}

#[derive(Clone)]
pub struct Combat {
    content: Arc<Content>,
    state: CombatState,
    last_attack: Option<u64>,
}

impl Combat {
    pub fn load(
        directory: impl AsRef<Path>,
        state: Option<CombatState>,
    ) -> Result<Self, CombatError> {
        let directory = directory.as_ref();
        let config = fs::read(directory.join("combat.json"))?;
        if config.len() > 16_384 {
            return Err(invalid("combat config too large"));
        }
        let mut content: Content =
            serde_json::from_slice(&config).map_err(|e| invalid(&e.to_string()))?;
        if content.template_hex.is_empty() {
            let path = Path::new(&content.template_file);
            if path.components().count() != 1
                || !matches!(path.components().next(), Some(Component::Normal(_)))
            {
                return Err(invalid("combat template must be a local filename"));
            }
            content.template = fs::read(directory.join(path))?;
        } else {
            // T853: the release ships typed JSON with no binary template. The bytes are
            // the same AddNpc skeleton `validate_template` pins; the hash still verifies.
            if content.template_hex.len() > 4096 {
                return Err(invalid("combat template hex too large"));
            }
            content.template = crate::startup_snapshot::decode_hex(&content.template_hex)
                .map_err(|_| invalid("invalid combat template hex"))?;
        }
        if format!("{:x}", Sha256::digest(&content.template)) != content.template_sha256 {
            return Err(invalid("combat template hash mismatch"));
        }
        validate_template(&content.template)?;
        if content.schema_version != 1
            || content.max_health == 0
            || content.max_health > 1_000_000
            || content.hit_damage == 0
            || content.hit_damage > content.max_health
            || !content.range.is_finite()
            || !(0.5..=100.0).contains(&content.range)
            || !(100..=60_000).contains(&content.cooldown_ms)
            || content.enemy.definition_id <= 0
        {
            return Err(invalid("invalid combat rules"));
        }
        for actor in [&content.npc, &content.enemy] {
            if actor.model_id <= 0
                || actor.name.is_empty()
                || actor.name.len() > 120
                || actor.name.chars().any(char::is_control)
                || actor.texture_alias.len() > 120
                || !valid_position(actor.position)
            {
                return Err(invalid("invalid combat actor"));
            }
        }
        let state = state.unwrap_or(CombatState {
            enemy_health: content.max_health,
            defeats: 0,
        });
        if state.enemy_health > content.max_health {
            return Err(invalid("saved enemy health exceeds maximum"));
        }
        Ok(Self {
            content: Arc::new(content),
            state,
            last_attack: None,
        })
    }

    pub fn state(&self) -> CombatState {
        self.state.clone()
    }

    /// Absolute content positions survive restarts; player position never moves actors.
    pub fn initialization(&self, _position: [f32; 3]) -> Vec<Vec<u8>> {
        let mut replies = vec![self.spawn(&self.content.npc, NPC_GUID, 2)];
        if self.state.enemy_health != 0 {
            replies.push(self.spawn(&self.content.enemy, ENEMY_GUID, 0));
        }
        replies.push(relevance(self.state.enemy_health != 0));
        replies
    }

    pub fn handle(
        &mut self,
        inner: &[u8],
        position: [f32; 3],
        now_ms: u64,
    ) -> Result<CombatOutcome, CombatError> {
        if inner.len() < 4 {
            return Ok(CombatOutcome::default());
        }
        let opcode = u16::from_le_bytes(inner[..2].try_into().unwrap());
        let sub = u16::from_le_bytes(inner[2..4].try_into().unwrap());
        match (opcode, sub) {
            (26, 8) => {
                if inner.len() != 13 || inner[12] > 1 {
                    return Err(invalid("malformed NPC interact request"));
                }
                let guid = u64::from_le_bytes(inner[4..12].try_into().unwrap());
                Ok(self.interact(guid, position))
            }
            (26, 10) => {
                if inner.len() != 16 {
                    return Err(invalid("malformed NPC interaction selection"));
                }
                let guid = u64::from_le_bytes(inner[4..12].try_into().unwrap());
                let id = i32::from_le_bytes(inner[12..16].try_into().unwrap());
                Ok(match (guid, id) {
                    (NPC_GUID, INTERACTION_QUEST)
                        if self.near(position, self.content.npc.position) =>
                    {
                        CombatOutcome {
                            interacted: Some(NPC_GUID),
                            ..CombatOutcome::default()
                        }
                    }
                    (NPC_GUID, INTERACTION_RESPAWN) => self.respawn(position),
                    (ENEMY_GUID, INTERACTION_ATTACK) => self.attack(position, now_ms),
                    _ => CombatOutcome::default(),
                })
            }
            (36, 10) => {
                if inner.len() < 16 {
                    return Err(invalid("truncated ability request"));
                }
                let id = i32::from_le_bytes(inner[4..8].try_into().unwrap());
                let slot = i32::from_le_bytes(inner[8..12].try_into().unwrap());
                let target = i32::from_le_bytes(inner[12..16].try_into().unwrap());
                let expected = if target == 1 { 32 } else { 24 };
                if inner.len() != expected || id < 0 || !(0..=11).contains(&slot) {
                    return Err(invalid("malformed ability request"));
                }
                if target == 1 {
                    return Ok(CombatOutcome::default());
                }
                let guid = u64::from_le_bytes(inner[16..24].try_into().unwrap());
                // Sandbox basic attack only; this does not implement historical ability definitions.
                if slot == 0 && guid == ENEMY_GUID {
                    Ok(self.attack(position, now_ms))
                } else {
                    Ok(CombatOutcome::default())
                }
            }
            _ => Ok(CombatOutcome::default()),
        }
    }

    pub fn attack(&mut self, position: [f32; 3], now_ms: u64) -> CombatOutcome {
        if !self.near(position, self.content.enemy.position) {
            return CombatOutcome::feedback("Move within 8 units of the training enemy.");
        }
        if self.state.enemy_health == 0 {
            return CombatOutcome::feedback(
                "Enemy defeated. Talk to the Realm Guide or use realm respawn.",
            );
        }
        if self.last_attack.is_some_and(|last| {
            now_ms
                .checked_sub(last)
                .is_none_or(|elapsed| elapsed < self.content.cooldown_ms)
        }) {
            return CombatOutcome::feedback("Your training attack is cooling down.");
        }
        if self.state.defeats == u64::MAX {
            return CombatOutcome::feedback("Encounter defeat counter is full.");
        }
        self.last_attack = Some(now_ms);
        self.state.enemy_health = self
            .state
            .enemy_health
            .saturating_sub(self.content.hit_damage);
        let mut outcome = CombatOutcome {
            changed: true,
            feedback: Some(format!(
                "Training hit: {} / {} health remaining.",
                self.state.enemy_health, self.content.max_health
            )),
            ..CombatOutcome::default()
        };
        if self.state.enemy_health == 0 {
            self.state.defeats += 1;
            outcome.defeated = Some(self.content.enemy.definition_id);
            outcome.replies.push(remove_enemy());
            outcome.replies.push(relevance(false));
            outcome.feedback = Some("Training enemy defeated! Quest progress recorded; realm respawn starts the next encounter.".to_owned());
        }
        outcome
    }

    pub fn respawn(&mut self, position: [f32; 3]) -> CombatOutcome {
        if !self.near(position, self.content.npc.position)
            && !self.near(position, self.content.enemy.position)
        {
            return CombatOutcome::feedback("Return to the Realm Guide to reset the encounter.");
        }
        if self.state.enemy_health != 0 {
            return CombatOutcome::feedback("The training enemy is still alive.");
        }
        self.state.enemy_health = self.content.max_health;
        self.last_attack = None;
        CombatOutcome {
            changed: true,
            replies: vec![
                self.spawn(&self.content.enemy, ENEMY_GUID, 0),
                relevance(true),
            ],
            feedback: Some("Training enemy respawned.".to_owned()),
            ..CombatOutcome::default()
        }
    }

    fn interact(&self, guid: u64, position: [f32; 3]) -> CombatOutcome {
        if guid == NPC_GUID && self.near(position, self.content.npc.position) {
            CombatOutcome { replies: vec![menu(NPC_GUID, "Realm Guide - reconstructed local encounter", &[
                (INTERACTION_QUEST, 79), (INTERACTION_RESPAWN, 3300)], false)],
                interacted: Some(NPC_GUID), feedback: Some("Realm Guide: Quest opens your progress. Talk resets a defeated enemy. Commands: realm quest, realm attack, realm respawn.".to_owned()),
                ..CombatOutcome::default() }
        } else if guid == ENEMY_GUID
            && self.state.enemy_health != 0
            && self.near(position, self.content.enemy.position)
        {
            CombatOutcome {
                replies: vec![menu(
                    ENEMY_GUID,
                    "Training attack - reconstructed rules",
                    &[(INTERACTION_ATTACK, 22498)],
                    true,
                )],
                ..CombatOutcome::default()
            }
        } else {
            CombatOutcome::default()
        }
    }
    fn near(&self, from: [f32; 3], to: [f32; 3]) -> bool {
        valid_position(from)
            && from
                .iter()
                .zip(to)
                .map(|(a, b)| (*a - b).powi(2))
                .sum::<f32>()
                <= self.content.range.powi(2)
    }
    fn spawn(&self, actor: &Actor, guid: u64, disposition: i32) -> Vec<u8> {
        let mut bytes = self.content.template.clone();
        bytes[4..12].copy_from_slice(&guid.to_le_bytes());
        bytes[12..16].copy_from_slice(&0_i32.to_le_bytes());
        bytes[16..20].copy_from_slice(&actor.model_id.to_le_bytes());
        for (i, value) in actor.position.into_iter().enumerate() {
            bytes[37 + i * 4..41 + i * 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes[69..73].copy_from_slice(&0_i32.to_le_bytes());
        bytes[77..81].copy_from_slice(&disposition.to_le_bytes());
        bytes[133..137].copy_from_slice(&(self.content.range.ceil() as i32).to_le_bytes());
        // Replace in reverse order so the known empty-field offsets remain valid.
        replace_empty_string(&mut bytes, 106, &actor.name);
        replace_empty_string(&mut bytes, 81, &actor.texture_alias);
        bytes
    }
}

fn valid_position(position: [f32; 3]) -> bool {
    position
        .iter()
        .all(|v| v.is_finite() && v.abs() <= 1_000_000.0)
}
fn validate_template(bytes: &[u8]) -> Result<(), CombatError> {
    if bytes.len() != 334
        || bytes[..4] != [35, 0, 2, 0]
        || [73, 81, 85, 106, 163]
            .iter()
            .any(|&offset| bytes[offset..offset + 4] != [0; 4])
        || bytes[224] != 1
        || bytes[133..137] != 100_i32.to_le_bytes()
    {
        return Err(invalid("unsupported AddNpc template shape"));
    }
    Ok(())
}
fn replace_empty_string(bytes: &mut Vec<u8>, offset: usize, value: &str) {
    let field = (value.len() as u32)
        .to_le_bytes()
        .into_iter()
        .chain(value.bytes());
    bytes.splice(offset..offset + 4, field);
}
fn header(opcode: u16, sub: u16) -> Vec<u8> {
    [opcode.to_le_bytes(), sub.to_le_bytes()].concat()
}
fn menu(guid: u64, name: &str, entries: &[(i32, i32)], auto_select: bool) -> Vec<u8> {
    let mut bytes = header(26, 9);
    bytes.extend(guid.to_le_bytes());
    bytes.push(u8::from(auto_select));
    bytes.extend((entries.len() as i32).to_le_bytes());
    for (id, label) in entries {
        for value in [*id, 0, *label, 0, 0, 0, 0] {
            bytes.extend(value.to_le_bytes());
        }
    }
    bytes.extend((name.len() as u32).to_le_bytes());
    bytes.extend(name.as_bytes());
    bytes.extend([1, 0]);
    bytes
}
fn relevance(enemy_alive: bool) -> Vec<u8> {
    let mut bytes = header(35, 12);
    bytes.extend((if enemy_alive { 2_i32 } else { 1 }).to_le_bytes());
    bytes.extend(NPC_GUID.to_le_bytes());
    bytes.extend([1, 5, 0]);
    if enemy_alive {
        bytes.extend(ENEMY_GUID.to_le_bytes());
        bytes.extend([1, 1, 0]);
    }
    bytes
}
fn remove_enemy() -> Vec<u8> {
    let mut bytes = header(35, 3);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(ENEMY_GUID.to_le_bytes());
    bytes.push(0);
    for value in [0_i32, 0, 0, 0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}
