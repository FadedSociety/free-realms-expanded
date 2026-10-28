//! W146/T510: the four-slot item bar and the data-driven consumable rows (potions, food auras,
//! transformation treats and size effects).
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
//! * A slot holds any owned item whose definition has a [`Effect`] row (schema 2, T510) or a
//!   boombox in [`boombox::Catalogue`] (owned outright, not rented). Recovery's seven-row
//!   `use_wire` table is no longer the gate; only the row decides.
//! * A row is one of four effects: a heal applied here, or an aura, transform or scale handed to
//!   the effect stack in [`Reply::effect`] for the caller to apply. A row names its own buff icon
//!   and name; the slot's icon and name still come from [`Inventory::presentation`].
//! * A boombox press is never spent and saves nothing: it stamps the row's `recast_ms` and hands
//!   the definition back in [`Reply::boombox`] for the caller to place the box. The caller says in
//!   [`Player::can_place_box`] whether the player may place one now.
//! * Heal: the cached wiki gives 2% instantly plus 11% per second for 3 seconds; the effect is
//!   applied at once as the row's percent of maximum health. Cooldown 10 s per item definition,
//!   shared by every slot and by 42/15. Both are tunable in `consumables.json` (see [`Effects`]).
//! * (R) OSFR MinCooldownMs: an accepted use always stamps at least 3 s, for every row and every
//!   38/25 refresh.
//! * A use consumes exactly one item when the row is single-use. A refusal changes nothing and
//!   sends 36/1 (bar use) or a chat line. Using at full health is allowed. A bound stack that
//!   empties unbinds its slot. A transform row is refused while a transform is live.
//! * Cooldowns and the pending 42/16 confirmation live in memory only; a relog resets them.
//! * 42/15 only works for the instance the last 42/16 confirmed, once.

use crate::{abilities, boombox, chat, consumable_effects, inventory::Inventory};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
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
const MAX_EFFECTS_FILE: u64 = 256 * 1024;
const MAX_EFFECT_ROWS: usize = 512;
const MAX_COOLDOWN_MS: u32 = 86_400_000;
/// (R) OSFR `MinCooldownMs`: the client never shows a shorter recast than this.
const MIN_COOLDOWN_MS: u32 = 3_000;
const SUB_ASSIGN_BY_RECORD: &[u8] = &[42, 0, 9, 0];

/// One usable consumable row. `name_id`/`description_id` are the 42/16 confirm text and the buff
/// row name; `source` is content provenance that the server never reads.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    pub definition: i32,
    pub name_id: i32,
    pub description_id: i32,
    pub cooldown_ms: u32,
    pub single_use: bool,
    pub effect: EffectKind,
    pub source: String,
}

/// The four row effects. A heal is applied by the caller to health; the other three become a
/// [`consumable_effects::Spec`] that the caller applies through the shared effect stack.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum EffectKind {
    Heal {
        percent: u32,
    },
    Aura {
        composite_effect_id: i32,
        duration_ms: u64,
        delay_ms: u64,
        icon_id: i32,
    },
    Transform {
        model_id: i32,
        poof_effect_id: i32,
        duration_ms: u64,
        icon_id: i32,
    },
    Scale {
        scale: f32,
        duration_ms: u64,
        icon_id: i32,
    },
}

impl Effect {
    /// Sanity bounds, not tuning values.
    fn valid(&self) -> bool {
        let duration = |ms: u64| (1..=consumable_effects::MAX_DURATION_MS).contains(&ms);
        self.definition > 0
            && self.name_id > 0
            && self.description_id > 0
            && self.cooldown_ms <= MAX_COOLDOWN_MS
            && match self.effect {
                EffectKind::Heal { percent } => (1..=100).contains(&percent),
                EffectKind::Aura {
                    composite_effect_id,
                    duration_ms,
                    delay_ms,
                    icon_id,
                } => {
                    composite_effect_id > 0
                        && icon_id > 0
                        && duration(duration_ms)
                        && delay_ms <= consumable_effects::MAX_DELAY_MS
                }
                EffectKind::Transform {
                    model_id,
                    poof_effect_id,
                    duration_ms,
                    icon_id,
                } => model_id > 0 && poof_effect_id >= 0 && icon_id > 0 && duration(duration_ms),
                EffectKind::Scale {
                    scale,
                    duration_ms,
                    icon_id,
                } => {
                    icon_id > 0
                        && duration(duration_ms)
                        && (consumable_effects::MIN_SCALE..=consumable_effects::MAX_SCALE)
                            .contains(&scale)
                }
            }
    }

    /// (R) OSFR `MinCooldownMs`: every consumer, including the 38/25 sweep, uses this value.
    fn effective_cooldown_ms(&self) -> u32 {
        self.cooldown_ms.max(MIN_COOLDOWN_MS)
    }

    fn heal_percent(&self) -> u32 {
        match self.effect {
            EffectKind::Heal { percent } => percent,
            _ => 0,
        }
    }

    /// The effect-stack spec for a non-heal row; `None` for a heal.
    pub fn spec(&self) -> Option<consumable_effects::Spec> {
        let (kind, duration_ms, icon_id) = match self.effect {
            EffectKind::Heal { .. } => return None,
            EffectKind::Aura {
                composite_effect_id,
                duration_ms,
                delay_ms,
                icon_id,
            } => (
                consumable_effects::Kind::Aura {
                    composite_effect_id,
                    delay_ms,
                },
                duration_ms,
                icon_id,
            ),
            EffectKind::Transform {
                model_id,
                poof_effect_id,
                duration_ms,
                icon_id,
            } => (
                consumable_effects::Kind::Transform {
                    model_id,
                    poof_effect_id,
                },
                duration_ms,
                icon_id,
            ),
            EffectKind::Scale {
                scale,
                duration_ms,
                icon_id,
            } => (
                consumable_effects::Kind::Scale { scale },
                duration_ms,
                icon_id,
            ),
        };
        Some(consumable_effects::Spec {
            kind,
            duration_ms,
            icon_id,
            name_id: self.name_id,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EffectFile {
    schema_version: u32,
    consumables: Vec<Effect>,
}

/// Schema 1's potion-only row. Its name and description come from the recovered catalogue.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyEffect {
    definition: i32,
    heal_percent: u32,
    cooldown_ms: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyFile {
    schema_version: u32,
    consumables: Vec<LegacyEffect>,
}

#[derive(Deserialize)]
struct FileVersion {
    schema_version: u32,
}

/// The usable consumables. The built-in table is the Small Health Potion rows (R); an optional
/// `consumables.json` in the content directory replaces it, so values can be tuned without code.
#[derive(Clone, Debug, PartialEq)]
pub struct Effects(Vec<Effect>);

impl Default for Effects {
    fn default() -> Self {
        Self(
            [2173, 11002]
                .map(|definition| {
                    let item = use_wire::consumable(definition).expect("recovered potion row");
                    Effect {
                        definition,
                        name_id: item.name_id,
                        description_id: item.description_id,
                        cooldown_ms: 10_000,
                        single_use: true,
                        effect: EffectKind::Heal { percent: 35 },
                        source: String::new(),
                    }
                })
                .to_vec(),
        )
    }
}

impl Effects {
    /// Sanity bounds, not tuned values.
    pub fn new(rows: Vec<Effect>) -> io::Result<Self> {
        if rows.len() > MAX_EFFECT_ROWS {
            return Err(invalid("too many consumable effect rows"));
        }
        let mut seen = BTreeSet::new();
        for row in &rows {
            if !row.valid() || !seen.insert(row.definition) {
                return Err(invalid("invalid consumable effect row"));
            }
        }
        Ok(Self(rows))
    }

    /// `consumables.json` from the fixed content directory, or the built-in table when absent.
    /// Schema 1 (the potion-only rows) still loads; schema 2 is the row shape above.
    pub fn load(directory: &Path) -> io::Result<Self> {
        let file = match fs::File::open(directory.join(EFFECTS_FILE)) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };
        let mut bytes = Vec::new();
        file.take(MAX_EFFECTS_FILE + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_EFFECTS_FILE {
            return Err(invalid("consumables.json exceeds 256 KiB"));
        }
        let version: FileVersion = serde_json::from_slice(&bytes).map_err(invalid)?;
        match version.schema_version {
            1 => {
                let parsed: LegacyFile = serde_json::from_slice(&bytes).map_err(invalid)?;
                if parsed.schema_version != 1 {
                    return Err(invalid("unsupported consumables.json version"));
                }
                let rows = parsed
                    .consumables
                    .into_iter()
                    .map(Effect::from_legacy)
                    .collect::<io::Result<Vec<_>>>()?;
                Self::new(rows)
            }
            2 => {
                let parsed: EffectFile = serde_json::from_slice(&bytes).map_err(invalid)?;
                if parsed.schema_version != 2 {
                    return Err(invalid("unsupported consumables.json version"));
                }
                Self::new(parsed.consumables)
            }
            _ => Err(invalid("unsupported consumables.json version")),
        }
    }

    pub fn get(&self, definition: i32) -> Option<&Effect> {
        self.0.iter().find(|e| e.definition == definition)
    }

    /// Every loaded row, for content gates that re-check the effect-stack bounds.
    pub fn rows(&self) -> &[Effect] {
        &self.0
    }
}

impl Effect {
    /// Schema 1 accepted only the recovered catalogue rows; that stays true.
    fn from_legacy(row: LegacyEffect) -> io::Result<Self> {
        let item = use_wire::consumable(row.definition)
            .ok_or_else(|| invalid("not a recovered consumable"))?;
        Ok(Self {
            definition: row.definition,
            name_id: item.name_id,
            description_id: item.description_id,
            cooldown_ms: row.cooldown_ms,
            single_use: true,
            effect: EffectKind::Heal {
                percent: row.heal_percent,
            },
            source: String::new(),
        })
    }
}

/// The facts about the player this module needs. The guid is the authenticated character's.
#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub guid: u64,
    pub alive: bool,
    /// Not mounted, not busy and in the open world: a boombox press is refused otherwise.
    pub can_place_box: bool,
    /// An encounter engine tracks this player's health, so a heal row may be used. Without one the
    /// player counts as alive but a heal row is refused (R).
    pub can_heal: bool,
    /// A temporary appearance is live, so a transform row is refused (R).
    pub transformed: bool,
}

/// What a handled request produced. `heal_percent` is for the caller to apply to the player's
/// health (the item is already consumed when it is non-zero). `effect` is the non-heal row's spec
/// for the caller to apply through the effect stack. `boombox` is the definition of a box the
/// caller must now place (nothing was consumed or changed for it).
#[derive(Debug, Default)]
pub struct Reply {
    pub replies: Vec<Vec<u8>>,
    /// True when the bar or the inventory changed and must be saved. A non-single-use row
    /// changes neither (its effect and cooldown are session-only).
    pub changed: bool,
    pub heal_percent: u32,
    pub boombox: Option<i32>,
    pub effect: Option<consumable_effects::Spec>,
    /// T789: the definition of a single-use item actually spent, for a `use_item` objective.
    /// It is the same `Use.spent` flag that sets `changed`; a boombox press and every refusal
    /// leave it `None`.
    pub used_definition: Option<i32>,
}

/// One accepted use. The item was spent when `spent` is set and the cooldown stamped; the caller
/// applies `heal_percent` to health or `effect` through the stack.
struct Use {
    replies: Vec<Vec<u8>>,
    heal_percent: u32,
    effect: Option<consumable_effects::Spec>,
    spent: bool,
    definition: i32,
}

/// The persisted item bar: four slots of owned consumable instances.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemBar {
    bar: bar_wire::ActionBarState,
    #[serde(skip)]
    effects: Arc<Effects>,
    #[serde(skip)]
    boomboxes: Arc<boombox::Catalogue>,
    /// Last accepted use per item definition, in the caller's millisecond clock.
    #[serde(skip)]
    last_used: BTreeMap<i32, u64>,
    /// The instance the last accepted 42/16 check confirmed. Cleared only by an accepted
    /// 42/15; a refused confirm is a no-op, so it keeps this confirmation.
    #[serde(skip)]
    pending: Option<i32>,
}

/// Read-only view of the inventory for the bar. Only items with an effect row and catalogued
/// boomboxes can back a slot.
struct View<'a>(&'a Inventory, &'a boombox::Catalogue, &'a Effects);

impl ActionBarItems for View<'_> {
    fn owned_action(&self, item_instance_id: i32) -> Option<OwnedAction> {
        let item = self
            .0
            .state()
            .items
            .iter()
            .find(|i| i.id == item_instance_id)?;
        // A rented boombox cannot take a slot (R); a rented row item may bind, but `usable`
        // refuses every rental at use time.
        let listed = self.2.get(item.definition_id).is_some()
            || (item.rental.is_none() && self.1.get(item.definition_id).is_some());
        if !listed {
            return None;
        }
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
    Transformed,
}

impl Refusal {
    fn text(&self) -> String {
        match self {
            Self::NotUsable => "You can't use that item.".into(),
            Self::Empty => "You have none of that item left.".into(),
            Self::KnockedOut => "You can't use items while knocked out.".into(),
            Self::Cooldown(ms) => format!("That item is not ready yet ({} s).", ms.div_ceil(1000)),
            // (R) OSFR's transform-food refusal text.
            Self::Transformed => "You are already transformed.".into(),
        }
    }
}

impl ItemBar {
    /// Restore a saved bar (validated) with the effect table from the content directory and the
    /// shared boombox catalogue.
    pub fn load(
        directory: &Path,
        boomboxes: Arc<boombox::Catalogue>,
        saved: Option<Self>,
    ) -> io::Result<Self> {
        let mut bar = saved.unwrap_or_default();
        bar.bar.validate()?;
        bar.effects = Arc::new(Effects::load(directory)?);
        bar.boomboxes = boomboxes;
        Ok(bar)
    }

    /// True when nothing is bound, so an unused bar adds nothing to a save.
    pub fn is_empty(&self) -> bool {
        self.bar.bindings().is_empty()
    }

    pub fn bindings(&self) -> &[ActionBarBinding] {
        self.bar.bindings()
    }

    /// The catalogue row of the boombox `definition`, for the host that places the box. The
    /// catalogue's shared pointer, not a copy: a press must not copy the clip tables.
    pub fn boombox(&self, definition: i32) -> Option<Arc<boombox::Boombox>> {
        self.boomboxes.get(definition).cloned()
    }

    /// Route one client message. `None` means it is not an item-bar request, or (42/15, 42/16) the
    /// instance's definition has no effect row, so the caller keeps its existing handling.
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
            let listed = inventory
                .state()
                .items
                .iter()
                .find(|i| i.id == id)
                .is_some_and(|item| self.effects.get(item.definition_id).is_some());
            if !listed {
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
        self.bar
            .reconcile(&View(inventory, &self.boomboxes, &self.effects));
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
            &View(inventory, &self.boomboxes, &self.effects),
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
            ..Reply::default()
        })
    }

    fn check(&mut self, id: i32, inventory: &Inventory, player: &Player, now_ms: u64) -> Reply {
        let row = inventory
            .state()
            .items
            .iter()
            .find(|i| i.id == id)
            .and_then(|item| self.effects.get(item.definition_id))
            .map(|row| (row.name_id, row.description_id));
        let refusal = match (self.usable(id, inventory, player, now_ms), row) {
            (Ok(()), Some((name_id, description_id))) => {
                self.pending = Some(id);
                // Success text is the item's own name and description (R): the chest strings
                // would read "Use a Mystery Key". Category and bundle only matter on failure.
                let bytes = use_wire::encode_immediate_activation_check(
                    id,
                    0,
                    0,
                    name_id,
                    description_id,
                    true,
                );
                return Reply {
                    replies: vec![bytes],
                    ..Reply::default()
                };
            }
            (Err(refusal), _) => refusal,
            (Ok(()), None) => Refusal::NotUsable,
        };
        // A refused check leaves any earlier confirmation pending (a refusal consumes nothing).
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
        if self.pending != Some(id) {
            return Ok(Reply {
                replies: vec![chat::system("Confirm the item before using it.")],
                ..Reply::default()
            });
        }
        match self.consume(id, inventory, player, now_ms) {
            Ok(used) => {
                // T789 follow-up: a refused 42/15 changed nothing, so it also keeps the
                // confirmation it was checked with; only a successful use consumes it.
                self.pending = None;
                let mut replies = used.replies;
                replies.extend(self.refresh(inventory, now_ms)?);
                Ok(Reply {
                    replies,
                    changed: used.spent,
                    heal_percent: used.heal_percent,
                    effect: used.effect,
                    used_definition: used.spent.then_some(used.definition),
                    ..Reply::default()
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
        if let Some(definition) = self.boombox_of(binding.item_instance_id, inventory) {
            // Every check first; the box is never spent, so only the recast changes.
            let usable = self.usable(binding.item_instance_id, inventory, player, now_ms);
            if !player.can_place_box || usable.is_err() {
                return Ok(refused());
            }
            self.last_used.insert(definition, now_ms);
            return Ok(Reply {
                replies: self.refresh(inventory, now_ms)?,
                boombox: Some(definition),
                ..Reply::default()
            });
        }
        match self.consume(binding.item_instance_id, inventory, player, now_ms) {
            Ok(used) => {
                let mut replies = used.replies;
                replies.extend(self.refresh(inventory, now_ms)?);
                Ok(Reply {
                    replies,
                    changed: used.spent,
                    heal_percent: used.heal_percent,
                    effect: used.effect,
                    used_definition: used.spent.then_some(used.definition),
                    ..Reply::default()
                })
            }
            Err(_) => Ok(refused()),
        }
    }

    /// The one place an item is spent: every check first (including `EffectStack::admits`, which
    /// the caller's `apply` repeats on the same spec), then one consume and the recast stamp.
    fn consume(
        &mut self,
        id: i32,
        inventory: &mut Inventory,
        player: &Player,
        now_ms: u64,
    ) -> Result<Use, Refusal> {
        self.usable(id, inventory, player, now_ms)?;
        let definition = inventory
            .state()
            .items
            .iter()
            .find(|i| i.id == id)
            .map(|i| i.definition_id)
            .ok_or(Refusal::NotUsable)?;
        let effect = self.effects.get(definition).ok_or(Refusal::NotUsable)?;
        let spent = effect.single_use;
        let heal_percent = effect.heal_percent();
        let spec = effect.spec();
        let replies = if spent {
            inventory
                .consume(id, 1)
                .map_err(|_| Refusal::NotUsable)?
                .replies
        } else {
            Vec::new()
        };
        self.last_used.insert(definition, now_ms);
        Ok(Use {
            replies,
            heal_percent,
            effect: spec,
            spent,
            definition,
        })
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
        let recast = self
            .recast_ms(item.definition_id)
            .filter(|_| item.rental.is_none())
            .ok_or(Refusal::NotUsable)?;
        if item.count.saturating_sub(item.consumed_count) == 0 {
            return Err(Refusal::Empty);
        }
        if !player.alive {
            return Err(Refusal::KnockedOut);
        }
        if let Some(left) = self.cooldown_left(item.definition_id, recast, now_ms) {
            return Err(Refusal::Cooldown(left));
        }
        // A boombox has no effect row; every row below is checked after the cooldown, as 42/16
        // stages it, so a refusal never spends or stamps.
        let Some(effect) = self.effects.get(item.definition_id) else {
            return Ok(());
        };
        if matches!(effect.effect, EffectKind::Heal { .. }) && !player.can_heal {
            return Err(Refusal::NotUsable);
        }
        if let Some(spec) = effect.spec() {
            consumable_effects::admits(&spec).map_err(|_| Refusal::NotUsable)?;
            if matches!(spec.kind, consumable_effects::Kind::Transform { .. }) && player.transformed
            {
                return Err(Refusal::Transformed);
            }
        }
        Ok(())
    }

    /// The recast of a usable item: an effect row's floored cooldown or a catalogued boombox's
    /// `recast_ms`.
    fn recast_ms(&self, definition: i32) -> Option<u32> {
        self.effects
            .get(definition)
            .map(Effect::effective_cooldown_ms)
            .or_else(|| self.boomboxes.get(definition).map(|row| row.recast_ms))
    }

    /// The definition of the catalogued boombox that instance `id` is, if it is one.
    fn boombox_of(&self, id: i32, inventory: &Inventory) -> Option<i32> {
        let item = inventory.state().items.iter().find(|i| i.id == id)?;
        self.boomboxes
            .get(item.definition_id)
            .map(|_| item.definition_id)
    }

    /// Milliseconds of recast left; a clock that went backwards counts as ready.
    fn cooldown_left(&self, definition: i32, recast_ms: u32, now_ms: u64) -> Option<u64> {
        let elapsed = now_ms.checked_sub(*self.last_used.get(&definition)?)?;
        u64::from(recast_ms)
            .checked_sub(elapsed)
            .filter(|left| *left > 0)
    }

    fn slot_update(
        &self,
        binding: ActionBarBinding,
        inventory: &Inventory,
        now_ms: u64,
    ) -> Option<io::Result<Vec<u8>>> {
        let action = View(inventory, &self.boomboxes, &self.effects)
            .owned_action(binding.item_instance_id)?;
        let mut slot = ActionBarSlot::for_owned_action(&action);
        match self.recast_ms(action.definition_id) {
            Some(recast) => {
                slot.total_refresh_time = recast as i32; // bounded by MAX_COOLDOWN_MS and MAX_MS
                if self
                    .cooldown_left(action.definition_id, recast, now_ms)
                    .is_some()
                {
                    let used = self.last_used[&action.definition_id];
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
