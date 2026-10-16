//! W146: the four-slot item bar and one useful consumable (Small Health Potion).
//!
//! # Recovered original facts
//!
//! * 42/6 (`00c14720`, built by `00c23ed0`): `i32 slot` (zero-based, four slots), `i32 owned
//!   instance id`; 0 clears the slot. 12 bytes.
//! * 42/9 (ctor `00a122e0`, sender `00a1dd30`, serializer `00a1c080` read from the bytes in W146):
//!   header, `i32 slot` (packet `+0xc`), then the embedded ItemRecord through `00a1aee0`, which
//!   writes only its first two dwords: `i32 definition`, `i32 tint`. 16 bytes. The ctor copies a
//!   third dword into `+0x1c`, but no writer reads it.
//! * 38/25 (`0089e7f0`, slot reader `008dae30`): bar id (2 = item bar), slot, 51-byte slot record.
//! * 36/10 (`00a35150`, writer `00a342e0`): bar, slot, target mode, then a `u64` guid (mode 0) or a
//!   `Vector4`. The dock sends bar 2 for the item bar (`Ui.ClickedAbility(slot, 2)`). The slot's
//!   self-target selector pair 1/2 makes the client target itself (`00a35600`).
//! * 42/16 (`00c109f0`) to 38/30 (`0089ea30`): the client shows `titleStringId`/`bodyStringId` in a
//!   confirm window on success and sends 42/15 (`00c10940`) on Okay (Lua `OnIsItemUsable`).
//! * The wire codecs and the recovered consumable catalogue are the reviewed restoration pieces
//!   046 and 048; this module adds state, validation and the use rule on top of them.
//!
//! # Reconstructed policy (R)
//!
//! * Only the seven recovered consumable rows may sit on the bar; only the Small Health Potion rows
//!   (2173, 11002) are usable. Other bar items show disabled and a use is refused.
//! * Heal: the cached wiki gives 2% instantly plus 11% per second for 3 seconds; the effect is
//!   applied at once as 35% of maximum health. Cooldown 10 s per item definition, shared by every
//!   slot and by 42/15. Both are tunable in `consumables.json` (see [`Effects`]).
//! * A use consumes exactly one item. A refusal changes nothing and sends 36/1 (bar use) or a chat
//!   line. Using at full health is allowed. A bound stack that empties unbinds its slot.
//! * Cooldowns and the pending 42/16 confirmation live in memory only; a relog resets them.
//! * 42/15 only works for the instance the last 42/16 confirmed, once.

use crate::{abilities, chat, inventory::Inventory};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

#[path = "restoration/piece_048.rs"]
pub mod bar_wire;
#[path = "restoration/piece_046.rs"]
pub mod use_wire;

use bar_wire::{ActionBarBinding, ActionBarItems, ActionBarSlot, AssignRejection, OwnedAction};

const EFFECTS_FILE: &str = "consumables.json";
const MAX_EFFECTS_FILE: u64 = 64 * 1024;
const MAX_EFFECT_ROWS: usize = 16;
const MAX_COOLDOWN_MS: u32 = 600_000;
const SUB_ASSIGN_BY_RECORD: &[u8] = &[42, 0, 9, 0];

/// One usable consumable. `heal_percent` is of maximum health, 1 to 100.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    pub definition: i32,
    pub heal_percent: u32,
    pub cooldown_ms: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EffectFile {
    schema_version: u32,
    consumables: Vec<Effect>,
}

/// The usable consumables. The built-in table is the Small Health Potion rows (R); an optional
/// `consumables.json` in the content directory replaces it, so values can be tuned without code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effects(Vec<Effect>);

impl Default for Effects {
    fn default() -> Self {
        Self(
            [2173, 11002]
                .map(|definition| Effect {
                    definition,
                    heal_percent: 35,
                    cooldown_ms: 10_000,
                })
                .to_vec(),
        )
    }
}

impl Effects {
    /// Sanity bounds, not tuned values: rows must be recovered catalogue consumables.
    pub fn new(rows: Vec<Effect>) -> io::Result<Self> {
        let mut seen = Vec::new();
        for row in &rows {
            if !(1..=100).contains(&row.heal_percent)
                || row.cooldown_ms > MAX_COOLDOWN_MS
                || use_wire::consumable(row.definition).is_none()
                || seen.contains(&row.definition)
            {
                return Err(invalid("invalid consumable effect row"));
            }
            seen.push(row.definition);
        }
        if rows.len() > MAX_EFFECT_ROWS {
            return Err(invalid("too many consumable effect rows"));
        }
        Ok(Self(rows))
    }

    /// `consumables.json` from the fixed content directory, or the built-in table when absent.
    pub fn load(directory: &Path) -> io::Result<Self> {
        let file = match fs::File::open(directory.join(EFFECTS_FILE)) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };
        let mut bytes = Vec::new();
        file.take(MAX_EFFECTS_FILE + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_EFFECTS_FILE {
            return Err(invalid("consumables.json exceeds 64 KiB"));
        }
        let parsed: EffectFile = serde_json::from_slice(&bytes).map_err(invalid)?;
        if parsed.schema_version != 1 {
            return Err(invalid("unsupported consumables.json version"));
        }
        Self::new(parsed.consumables)
    }

    pub fn get(&self, definition: i32) -> Option<Effect> {
        self.0.iter().copied().find(|e| e.definition == definition)
    }
}

/// The facts about the player this module needs. The guid is the authenticated character's.
#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub guid: u64,
    pub alive: bool,
}

/// What a handled request produced. `heal_percent` is for the caller to apply to the player's
/// health (the item is already consumed when it is non-zero).
#[derive(Debug, Default)]
pub struct Reply {
    pub replies: Vec<Vec<u8>>,
    pub changed: bool,
    pub heal_percent: u32,
}

/// The persisted item bar: four slots of owned consumable instances.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemBar {
    bar: bar_wire::ActionBarState,
    #[serde(skip)]
    effects: Arc<Effects>,
    /// Last accepted use per item definition, in the caller's millisecond clock.
    #[serde(skip)]
    last_used: BTreeMap<i32, u64>,
    #[serde(skip)]
    pending: Option<i32>,
}

/// Read-only view of the inventory for the bar. Only recovered consumables can back a slot.
struct View<'a>(&'a Inventory);

impl ActionBarItems for View<'_> {
    fn owned_action(&self, item_instance_id: i32) -> Option<OwnedAction> {
        let item = self
            .0
            .state()
            .items
            .iter()
            .find(|i| i.id == item_instance_id)?;
        use_wire::consumable(item.definition_id)?;
        let shown = self.0.presentation(item.definition_id).ok()?;
        Some(OwnedAction {
            item_instance_id,
            definition_id: item.definition_id,
            count: item.count.saturating_sub(item.consumed_count),
            icon_id: shown.icon_id,
            icon_tint_id: bar_wire::resolve_tint(item.tint, shown.icon_tint),
            name_id: shown.name_id,
        })
    }
}

enum Refusal {
    NotUsable,
    Empty,
    KnockedOut,
    Cooldown(u64),
}

impl Refusal {
    fn text(&self) -> String {
        match self {
            Self::NotUsable => "You can't use that item.".into(),
            Self::Empty => "You have none of that item left.".into(),
            Self::KnockedOut => "You can't use items while knocked out.".into(),
            Self::Cooldown(ms) => format!("That item is not ready yet ({} s).", ms.div_ceil(1000)),
        }
    }
}

impl ItemBar {
    /// Restore a saved bar (validated) with the effect table from the content directory.
    pub fn load(directory: &Path, saved: Option<Self>) -> io::Result<Self> {
        let mut bar = saved.unwrap_or_default();
        bar.bar.validate()?;
        bar.effects = Arc::new(Effects::load(directory)?);
        Ok(bar)
    }

    /// True when nothing is bound, so an unused bar adds nothing to a save.
    pub fn is_empty(&self) -> bool {
        self.bar.bindings().is_empty()
    }

    pub fn bindings(&self) -> &[ActionBarBinding] {
        self.bar.bindings()
    }

    /// Route one client message. `None` means it is not an item-bar request, or (42/15, 42/16) the
    /// instance is not a recovered consumable, so the caller keeps its existing handling.
    pub fn handle(
        &mut self,
        inner: &[u8],
        inventory: &mut Inventory,
        player: &Player,
        now_ms: u64,
    ) -> io::Result<Option<Reply>> {
        if inner.starts_with(&[42, 0, 6, 0]) {
            let bar_wire::AssignParse::Request(request) = bar_wire::parse_assign(inner)? else {
                return Ok(None);
            };
            return self
                .assign(request.slot, request.item_instance_id, inventory, now_ms)
                .map(Some);
        }
        if inner.starts_with(SUB_ASSIGN_BY_RECORD) {
            if inner.len() != 16 {
                return Err(invalid("invalid item-bar record assignment"));
            }
            let (slot, definition, tint) = (word(inner, 4), word(inner, 8), word(inner, 12));
            // Same record match as 42/8 equip: the first owned stack with this definition and tint.
            let owned = inventory.state().items.iter().find(|i| {
                i.definition_id == definition
                    && i.tint == tint
                    && i.count.saturating_sub(i.consumed_count) > 0
            });
            let instance = owned.map_or(-1, |i| i.id);
            return self.assign(slot, instance, inventory, now_ms).map(Some);
        }
        if let Some((kind, id)) = use_wire::decode_immediate_activation(inner) {
            let consumable = inventory
                .state()
                .items
                .iter()
                .any(|i| i.id == id && use_wire::consumable(i.definition_id).is_some());
            if !consumable {
                return Ok(None);
            }
            let reply = match kind {
                use_wire::ImmediateActivation::Check => self.check(id, inventory, player, now_ms),
                use_wire::ImmediateActivation::Use => {
                    self.confirm(id, inventory, player, now_ms)?
                }
            };
            return Ok(Some(reply));
        }
        if inner.starts_with(&[36, 0, 10, 0])
            && inner.get(4..8) == Some(&use_wire::ITEM_ACTION_BAR_ID.to_le_bytes()[..])
        {
            let request = use_wire::decode_client_request_start_ability(inner)
                .ok_or_else(|| invalid("malformed item-bar activation"))?;
            return self.activate(request, inventory, player, now_ms).map(Some);
        }
        Ok(None)
    }

    /// 38/25 for all four slots: bound slots with quantity and recast, the rest empty. Stale
    /// bindings (stack gone) are dropped first.
    pub fn refresh(&mut self, inventory: &Inventory, now_ms: u64) -> io::Result<Vec<Vec<u8>>> {
        self.bar.reconcile(&View(inventory));
        (0..bar_wire::ITEM_BAR_SLOTS)
            .map(|slot| {
                let update = self
                    .bar
                    .bound(slot)
                    .and_then(|binding| self.slot_update(binding, inventory, now_ms));
                update.unwrap_or_else(|| bar_wire::empty_item_slot_update(slot))
            })
            .collect()
    }

    fn assign(
        &mut self,
        slot: i32,
        instance: i32,
        inventory: &Inventory,
        now_ms: u64,
    ) -> io::Result<Reply> {
        let before = self.bar.clone();
        let outcome = self.bar.assign(
            bar_wire::AssignRequest {
                slot,
                item_instance_id: instance,
            },
            &View(inventory),
        )?;
        let mut replies = Vec::new();
        if let Some(why) = outcome.rejection {
            replies.push(chat::system(match why {
                AssignRejection::UnknownItem => "That item can't go on the item bar.",
                AssignRejection::Unavailable => "You have none of that item left.",
                AssignRejection::SlotOutOfRange => "There is no such item bar slot.",
                AssignRejection::ByRecordUnrecovered => "That item can't go on the item bar.",
            }));
        }
        // The answer is always the authoritative bar, so a refused drag snaps back.
        replies.extend(self.refresh(inventory, now_ms)?);
        Ok(Reply {
            replies,
            changed: before != self.bar,
            heal_percent: 0,
        })
    }

    fn check(&mut self, id: i32, inventory: &Inventory, player: &Player, now_ms: u64) -> Reply {
        self.pending = None;
        let row = inventory
            .state()
            .items
            .iter()
            .find(|i| i.id == id)
            .and_then(|i| use_wire::consumable(i.definition_id));
        let refusal = match (self.usable(id, inventory, player, now_ms), row) {
            (Ok(()), Some(row)) => {
                self.pending = Some(id);
                // Success text is the item's own name and description (R): the chest strings
                // would read "Use a Mystery Key". Category and bundle only matter on failure.
                let mut bytes = vec![38, 0, 30, 0];
                for value in [id, 0, 0, row.name_id, row.description_id] {
                    bytes.extend(value.to_le_bytes());
                }
                bytes.push(1);
                return Reply {
                    replies: vec![bytes],
                    ..Reply::default()
                };
            }
            (Err(refusal), _) => refusal,
            (Ok(()), None) => Refusal::NotUsable,
        };
        Reply {
            replies: vec![chat::system(&refusal.text())],
            ..Reply::default()
        }
    }

    fn confirm(
        &mut self,
        id: i32,
        inventory: &mut Inventory,
        player: &Player,
        now_ms: u64,
    ) -> io::Result<Reply> {
        if self.pending.take() != Some(id) {
            return Ok(Reply {
                replies: vec![chat::system("Confirm the item before using it.")],
                ..Reply::default()
            });
        }
        match self.consume(id, inventory, player, now_ms) {
            Ok((mut replies, heal_percent)) => {
                replies.extend(self.refresh(inventory, now_ms)?);
                Ok(Reply {
                    replies,
                    changed: true,
                    heal_percent,
                })
            }
            Err(refusal) => Ok(Reply {
                replies: vec![chat::system(&refusal.text())],
                ..Reply::default()
            }),
        }
    }

    fn activate(
        &mut self,
        request: use_wire::ActivateRequest,
        inventory: &mut Inventory,
        player: &Player,
        now_ms: u64,
    ) -> io::Result<Reply> {
        let refused = || Reply {
            replies: vec![abilities::unsupported_cast_failure()],
            ..Reply::default()
        };
        // Self-target only: a forged guid can't redirect the heal (R). 0 is "no target".
        let on_self =
            matches!(request.target, use_wire::Target::Guid(g) if g == player.guid || g == 0);
        let binding = self.bar.bound(request.slot);
        let (true, Some(binding)) = (on_self, binding) else {
            return Ok(refused());
        };
        match self.consume(binding.item_instance_id, inventory, player, now_ms) {
            Ok((mut replies, heal_percent)) => {
                replies.extend(self.refresh(inventory, now_ms)?);
                Ok(Reply {
                    replies,
                    changed: true,
                    heal_percent,
                })
            }
            Err(_) => Ok(refused()),
        }
    }

    /// The one place an item is spent: every check first, then one consume and the recast stamp.
    fn consume(
        &mut self,
        id: i32,
        inventory: &mut Inventory,
        player: &Player,
        now_ms: u64,
    ) -> Result<(Vec<Vec<u8>>, u32), Refusal> {
        self.usable(id, inventory, player, now_ms)?;
        let definition = inventory
            .state()
            .items
            .iter()
            .find(|i| i.id == id)
            .map(|i| i.definition_id)
            .ok_or(Refusal::NotUsable)?;
        let effect = self.effects.get(definition).ok_or(Refusal::NotUsable)?;
        let outcome = inventory.consume(id, 1).map_err(|_| Refusal::NotUsable)?;
        self.last_used.insert(definition, now_ms);
        Ok((outcome.replies, effect.heal_percent))
    }

    fn usable(
        &self,
        id: i32,
        inventory: &Inventory,
        player: &Player,
        now_ms: u64,
    ) -> Result<(), Refusal> {
        let item = inventory
            .state()
            .items
            .iter()
            .find(|i| i.id == id)
            .ok_or(Refusal::NotUsable)?;
        let effect = self
            .effects
            .get(item.definition_id)
            .filter(|_| item.rental.is_none())
            .ok_or(Refusal::NotUsable)?;
        if item.count.saturating_sub(item.consumed_count) == 0 {
            return Err(Refusal::Empty);
        }
        if !player.alive {
            return Err(Refusal::KnockedOut);
        }
        match self.cooldown_left(effect, now_ms) {
            Some(left) => Err(Refusal::Cooldown(left)),
            None => Ok(()),
        }
    }

    /// Milliseconds of recast left; a clock that went backwards counts as ready.
    fn cooldown_left(&self, effect: Effect, now_ms: u64) -> Option<u64> {
        let elapsed = now_ms.checked_sub(*self.last_used.get(&effect.definition)?)?;
        u64::from(effect.cooldown_ms)
            .checked_sub(elapsed)
            .filter(|left| *left > 0)
    }

    fn slot_update(
        &self,
        binding: ActionBarBinding,
        inventory: &Inventory,
        now_ms: u64,
    ) -> Option<io::Result<Vec<u8>>> {
        let action = View(inventory).owned_action(binding.item_instance_id)?;
        let mut slot = ActionBarSlot::for_owned_action(&action);
        match self.effects.get(action.definition_id) {
            Some(effect) => {
                slot.total_refresh_time = effect.cooldown_ms as i32; // bounded by MAX_COOLDOWN_MS
                if self.cooldown_left(effect, now_ms).is_some() {
                    let used = self.last_used[&effect.definition];
                    slot.cast_age = now_ms.saturating_sub(used).min(i32::MAX as u64) as i32;
                }
            }
            // On the bar but not usable yet: show it greyed out.
            None => slot.enabled = false,
        }
        Some(bar_wire::item_slot_update(binding.slot, &slot))
    }
}

fn word(bytes: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(
        bytes[at..at + 4]
            .try_into()
            .expect("length checked by caller"),
    )
}

fn invalid(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
