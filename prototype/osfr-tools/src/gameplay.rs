//! Local gameplay integration. Content and wire templates remain private.
//! Domain transitions stage on a clone so rewards and quest claims commit together.
use crate::{
    character_store::CharacterState,
    chat,
    chess_service::{ChessActivityOutcome, ChessService},
    combat::{Combat, CombatOutcome, CombatState},
    cooking::{Cooking, CookingState},
    dev_teleport,
    encounter::{EncounterEngine, EncounterOutcome, EncounterState, ZoneTransition},
    forging::{Forging, ForgingState},
    game_rules::GameRuleSet,
    harvest_nodes::Gather,
    harvesting,
    identity::CharacterIdentity,
    inventory::{Inventory, InventoryState},
    item_categories::ItemCategories,
    kart_lobby_native,
    mining::{self, Type8Engine, Type8Outcome, Type8State},
    mystery_chest,
    mystery_chest_loot::MysteryChests,
    npc_interaction::{self, DialogueContext, MerchantWindow, NpcInteraction, NpcSelection},
    opening::{OpeningEngine, OpeningEvent, OpeningOutcome, OpeningState, OpeningTrigger},
    opening_abilities::OpeningAbilities,
    opening_journal::OpeningJournal,
    opening_world::OpeningWorld,
    profiles::{ProfileState, Profiles},
    quests::{QuestEngine, QuestOutcome, QuestState},
    recipe_book::CraftOutcome,
    recipes::CraftingJob,
    rewards::RewardState,
    smelting::{Smelting, SmeltingState},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    io::{self, Read},
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

mod content_reload;
mod derby_session;
mod field_battle;
mod jobs;
mod npc;
mod npc_quests;
mod occupation;
/// Warpstone definitions, attunement and validated travel (W142).
#[path = "restoration/piece_077.rs"]
#[allow(dead_code)]
mod piece_077;
/// Progress-driven achievement observation over authoritative gameplay state.
mod progression;
use self::npc::{dialogue_packet, quest_offer_packet_with};

#[derive(Clone)]
struct DisappearingActor {
    guid: u64,
    stage_id: u32,
    index: usize,
    required: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DisappearBinding {
    actor_guid: u64,
    stage_id: u32,
    objective_id: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameplayState {
    pub inventory: InventoryState,
    pub quests: QuestState,
    pub combat: CombatState,
    /// Regional quest progress is separate from the byte-parity opening quest state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub npc_quests: Option<crate::quest_engine::QuestState>,
    /// Numeric client recipe ids accepted/learned by this character.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub learned_recipes: BTreeSet<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profiles: Option<ProfileState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opening: Option<OpeningState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encounter: Option<EncounterState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smelting: Option<SmeltingState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forging: Option<ForgingState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooking: Option<CookingState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mining: Option<Type8State>,
    /// Opt-in Harvesting111 candidate; absent unless `harvesting.json` is
    /// loaded. Older saves without the field stay byte-identical.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harvesting: Option<Type8State>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kart_entry: Option<crate::kart_gateway::KartGatewayState>,
    /// Predecessor derby save shapes only (see `DerbyGatewayState`); never written now.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derby: Option<crate::derby_gateway::DerbyGatewayState>,
    /// Chess's daily coin ledger; absent until the first paid win.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chess: Option<crate::chess_service::ChessRewardLedger>,
    /// Attuned warpstones (W142). Absent until the first attunement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warpstones: Option<piece_077::WarpState>,
    /// The four-slot item bar (W146). Absent until something is bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_bar: Option<crate::item_bar::ItemBar>,
    /// W178 title ownership and durable claim history. Pending claims and
    /// in-flight definition trackers are session-only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progression_extras: Option<crate::progression_extras::ProgressionExtrasState>,
}

/// Opt-in Harvesting candidate bound by this gateway integration: Farnum's
/// Farm (MiniGameData 111) under the recovered Chef job (45).
const HARVESTING_GAME_ID: i32 = 111;
/// (R) Minimum gap between answered activity-portal refreshes (167/1/4).
const PORTAL_REFRESH_MS: u64 = 250;

#[derive(Clone)]
pub struct Gameplay {
    inventory: Inventory,
    categories: ItemCategories,
    quests: QuestEngine,
    combat: Combat,
    profiles: Option<Profiles>,
    identity_fingerprint: Option<String>,
    /// Character gender (1 boy, 2 girl) from `apply_identity`; 0 until then (gender not checked).
    gender: i32,
    opening: Option<OpeningEngine>,
    encounter: Option<EncounterEngine>,
    world: Option<OpeningWorld>,
    journal: Option<OpeningJournal>,
    abilities: Option<OpeningAbilities>,
    smelting: Option<Smelting>,
    forging: Option<Forging>,
    cooking: Option<Cooking>,
    mining: Option<Type8Engine>,
    harvesting: Option<Type8Engine>,
    forging_tick_ms: Option<u64>,
    type8_tick_ms: Option<u64>,
    disappearing_actors: Vec<DisappearingActor>,
    /// Developer-only chat commands stay off unless the launcher opts in.
    dev_tools: bool,
    terrain: Option<std::sync::Arc<crate::terrain::Terrain>>,
    /// 98/1 path-request budget for this session (W164).
    pathing: crate::pathing::Pathing,
    // World Builder live layer (`--world`); None keeps building in content_dir.
    world_dir: Option<PathBuf>,
    // Who may run developer `realm` commands (Build Plan 1.5). Player until
    // the gateway sets it from the account store (or admin for local solo).
    role: crate::account_store::AccountRole,
    // Security Model decision 1: with an account store the role is
    // server-owned, so role-checked commands run without --dev-tools.
    role_commands: bool,
    // Stock locale row keys, so authored text ids never collide with them.
    stock_text: Option<Arc<HashSet<u32>>>,
    /// Optional native five-table test catalogue. Not character progression.
    kart_lobby_reply: Option<Arc<[u8]>>,
    /// W150 (R): battles published in the activity portal (167/1/1). Not persisted.
    activities: crate::activity_portal::Catalogue,
    /// Soccer content and one live session-only match. Neither is persisted;
    /// a reconnect drops the match without a grant.
    soccer_content: Option<Arc<crate::soccer_entry::Content>>,
    soccer: Option<crate::soccer_entry::Session>,
    /// T022 (R): single-player chess rows and this session's table. Not persisted;
    /// a reconnect starts with no run, so no stale result can be replayed.
    chess: Option<ChessService>,
    /// Last answered 167/1/4 refresh: a 4-byte request returns up to ~6.7 KB,
    /// so refreshes are answered at most once per `PORTAL_REFRESH_MS` (R).
    portal_refreshed_ms: Option<u64>,
    kart_entry: Option<crate::kart_gateway::KartGateway>,
    /// Opt-in demolition derby package. A match is connection-local; a reconnect drops it.
    derby: Option<crate::derby_gateway::DerbyGateway>,
    /// W160 (R): the guide NPCs that offer fishing (`fishing_spots.json`). Not persisted.
    fishing_spots: Option<Arc<crate::fishing_service::FishingSpots>>,
    /// The player's fishing round, if any. Ephemeral by design: a reconnect releases it.
    fishing: crate::fishing_service::FishingService,
    /// W171 (R): this player's ore and mushroom nodes. Not persisted: a login sees every node.
    gathering_nodes: Option<crate::harvest_nodes::GatheringNodes>,
    /// (R) World-model station rules. Opening actors and current placements are
    /// resolved at use so a placement reload cannot leave stale coordinates.
    world_station_rules: Vec<crate::recipe_book::WorldStationRule>,
    /// The authored content directory (data/content), for developer reloads.
    content_dir: PathBuf,
    mystery_chests: Option<Arc<MysteryChests>>,
    pending_chest: Option<i32>,
    npc_visibility: Option<crate::world::NpcVisibility>,
    npc_interaction: Option<NpcInteraction>,
    /// Last NPC Talk packet, retained only for the opt-in offer experiment.
    last_npc_dialogue: Option<(u32, u64, u64, Vec<u8>)>,
    /// Quest offered as a dialog choice: (runtime guid, content key, quest id).
    pending_quest_choice: Option<(u64, u64, u32)>,
    /// Packets sent when the pending quest row is chosen. Empty means "send the
    /// 49/1 offer"; a turn-in holds its 49/13 end screen here.
    pending_choice_packets: Vec<Vec<u8>>,
    /// Inline rather than shared: purchases follow the gateway's staged Gameplay commit.
    npc_coin_shop: Option<crate::coin_shop::CoinShop>,
    npc_quests: Option<crate::quest_engine::QuestEngine>,
    saved_npc_quests: Option<crate::quest_engine::QuestState>,
    /// Quests a player may start remotely with 49/15 (R), each with its giver NPC.
    breadcrumb_quests: std::collections::BTreeMap<u32, u64>,
    /// Original breadcrumb quest id -> regional quest (`data/content/breadcrumb_quests.json`).
    breadcrumb_crosswalk: crate::quest_journal::Crosswalk,
    /// Client-reported goals the client has been told to watch; not saved.
    client_objectives: crate::quest_engine::ClientObjectives,
    /// The "Take Me There" pin (47/14) the client currently shows; not saved.
    tracked_pin: crate::quest_journal::Tracker,
    /// Regional quest state is sent once per network session, not after every zone load.
    npc_quests_synced: bool,
    game_rules: GameRuleSet,
    profile_content_directory: Option<PathBuf>,
    recipe_catalog: crate::recipes::RecipeCatalog,
    learned_recipes: BTreeSet<u32>,
    /// Warpstone attunement and travel cooldowns (W142).
    warp: piece_077::WarpRegistry,
    item_bar: crate::item_bar::ItemBar,
    /// W162: the summoned ride and mount state. Session-only (the ride actor does not survive a
    /// disconnect); absent when the content set has no `mounts.json`.
    mounts: Option<crate::mounts::MountService>,
    /// W162: a field enemy is chasing this player (from the last `receive_field_damage`). This is
    /// the "combat start" signal for rides; set by the shared field tick and never saved.
    field_engaged: bool,
    /// W177: the saved pet roster and session-only active actor. The roster stays in the
    /// canonical save's top-level `pets` payload; only the actor is reset on reconnect.
    pets: crate::pets::PetService,
    /// W178 titles and claim history, staged with the rest of this character.
    progression_extras: crate::progression_extras::ProgressionExtras,
    /// Data-driven achievement counters and exact title-name bindings.
    achievement_bindings: crate::progression_extras::AchievementBindings,
}

#[derive(Default)]
pub struct Outcome {
    pub dev_command_denied: bool,
    pub shared_field: Option<crate::encounter::SharedFieldAction>,
    pub replies: Vec<Vec<u8>>,
    pub changed: bool,
    pub zoning: Option<ZoneTransition>,
    /// Complete native scene transition. Vehicle scenes need the captured
    /// quaternion's fourth component, which the legacy encounter writer omits.
    pub scene_zoning: Option<crate::zone_transition::ZoneTransition>,
    /// A developer relocation to apply to the staged character state. The
    /// gateway owns the packet so bytes and saved position cannot diverge.
    pub relocation: Option<dev_teleport::Relocation>,
    /// W158: a big heart was picked up; the gateway heals the rest of the group.
    pub group_heal: Option<crate::encounter::GroupHeal>,
    /// Inner packets that nearby players must also receive: W162's ride actor, the rider's
    /// attachment and their removal, and T027's live 35/6 equip change / 35/7 job switch. The
    /// gateway fans them out through the player registry's interest set.
    pub nearby: Vec<Vec<u8>>,
    /// W177: the saved top-level pet roster changed and must be written through its own
    /// `game_store` seam after the normal character snapshot commits.
    pub pets_changed: bool,
}

impl Outcome {
    /// Publish a staged outcome into this one. Every field is named in the
    /// destructuring pattern, so adding an `Outcome` field fails to compile here
    /// until its merge rule is decided.
    fn absorb(&mut self, other: Outcome) {
        let Outcome {
            dev_command_denied,
            shared_field,
            replies,
            changed,
            zoning,
            scene_zoning,
            relocation,
            group_heal,
            nearby,
            pets_changed,
        } = other;
        self.dev_command_denied |= dev_command_denied;
        self.shared_field = shared_field.or(self.shared_field.take());
        self.replies.extend(replies);
        self.changed |= changed;
        self.zoning = zoning.or(self.zoning.take());
        self.scene_zoning = scene_zoning.or(self.scene_zoning.take());
        self.relocation = relocation.or(self.relocation.take());
        self.group_heal = group_heal.or(self.group_heal.take());
        self.nearby.extend(nearby);
        self.pets_changed |= pets_changed;
    }
}

/// Horizontal distance in metres.
fn flat_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// A named developer waypoint (`realm mark <name>`).
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Mark {
    name: String,
    zone: u32,
    position: [f32; 3],
    /// Hand-added fields (notes) survive rewrites.
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MarkFile {
    schema_version: u32,
    marks: Vec<Mark>,
}

fn invalid(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// (R) 35/6 EquipItemChange a neighbour needs after an active-job equip or
/// unequip. Inventory content names its own player guid in the packet, so the
/// broadcast names the authenticated character instead. An empty model is the
/// client's removal path (`0092f460` case 6).
fn item_change_packet(inner: &[u8], guid: u64, replies: &[Vec<u8>]) -> Option<Vec<u8>> {
    if inner.len() == 12 && inner.starts_with(&[42, 0, 2, 0]) {
        let slot = i32::from_le_bytes(inner[4..8].try_into().ok()?);
        let profile = i32::from_le_bytes(inner[8..12].try_into().ok()?);
        let mut packet = vec![35, 0, 6, 0];
        packet.extend(guid.to_le_bytes());
        packet.extend(0i32.to_le_bytes()); // no item; the empty model removes the slot
        for _ in 0..3 {
            packet.extend(0i32.to_le_bytes()); // empty model, texture and tint alias
        }
        for value in [0, 0, slot, profile, 0, 0] {
            packet.extend(value.to_le_bytes()); // tint, effect, slot, profile, unused, wield
        }
        return Some(packet);
    }
    let packet = replies
        .iter()
        .find(|packet| packet.starts_with(&[35, 0, 6, 0]))?;
    let mut packet = packet.clone();
    packet.get_mut(4..12)?.copy_from_slice(&guid.to_le_bytes());
    Some(packet)
}

/// W171: draw and remove node actors. The caller sends `visibility.refresh()` to the client.
fn apply_node_changes(
    visibility: &mut crate::world::NpcVisibility,
    changes: Vec<crate::harvest_nodes::NodeChange>,
) {
    use crate::harvest_nodes::NodeChange;
    for change in changes {
        match change {
            NodeChange::Show(node) => visibility.add_static_actor(
                node.guid,
                &node.name,
                0,
                node.model_id as i32,
                node.position,
                0,
            ),
            NodeChange::Hide(guid) => {
                visibility.remove(guid);
            }
        }
    }
}

fn npc_event(value: serde_json::Value) {
    println!("{value}");
}

impl Gameplay {
    pub fn take_shared_fields(&mut self) -> Option<crate::encounter::SharedFieldState> {
        let engine = self.encounter.as_mut()?;
        let runtime = engine.take_shared_fields()?;
        if let Some(visibility) = self.npc_visibility.as_mut() {
            visibility.exclude_actor_guids(engine.shared_field_guids().collect::<Vec<_>>());
            visibility.enable_shared_interest();
        }
        Some(runtime)
    }

    /// (R) Common NPC budget, computed without mutating staged Gameplay clones.
    pub fn npc_visibility_revision(&self) -> u64 {
        self.npc_visibility.as_ref().map_or(0, |v| v.revision())
    }
    pub fn shared_npc_view(
        &self,
        character: &CharacterState,
        fields: &[crate::world::NpcInterest],
        known: &BTreeSet<u64>,
    ) -> io::Result<(BTreeSet<u64>, Vec<Vec<u8>>)> {
        let (visible, mut packets) = self
            .npc_visibility
            .as_ref()
            .unwrap_or(&crate::world::NpcVisibility::default())
            .shared_view(character.zone_id, character.position, fields, known)?;
        self.append_shared_pending_end_screen(&mut packets);
        Ok((visible, packets))
    }
    pub fn bind_run_instance(&mut self, instance: i32) -> io::Result<bool> {
        self.encounter
            .as_mut()
            .map_or(Ok(false), |e| e.bind_run_instance(instance))
    }
    pub fn field_participant(
        &self,
        character: &CharacterState,
    ) -> Option<crate::encounter::FieldParticipant> {
        let profile = self.profiles.as_ref()?.active();
        Some(self.encounter.as_ref()?.field_participant(
            character.guid,
            character.position,
            self.abilities.as_ref()?.supports_basic(profile),
        ))
    }
    pub fn active_field_quests(&self) -> BTreeSet<u32> {
        self.npc_quests
            .as_ref()
            .map(|q| {
                q.state()
                    .quests
                    .iter()
                    .filter(|(_, p)| p.status == crate::quest_engine::QuestStatus::Active)
                    .map(|(id, _)| *id)
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn handle_shared_field(
        &mut self,
        action: &crate::encounter::SharedFieldAction,
        field: &mut crate::encounter::SharedFieldState,
        known: &BTreeSet<u64>,
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        use crate::encounter::SharedFieldAction;
        let mut out = Outcome::default();
        let Some(profiles) = &self.profiles else {
            return Ok(out);
        };
        let profile = profiles.active();
        let Some(abilities) = &self.abilities else {
            return Ok(out);
        };
        let slot = match *action {
            SharedFieldAction::Ability { slot, .. } => Some(slot),
            SharedFieldAction::Select { choice: 103, .. } => Some(0),
            _ => None,
        };
        if let Some(slot) = slot {
            if !abilities.supports_basic(profile) {
                return Ok(out);
            }
            if !abilities.weapon_matches(&self.inventory, profile, slot, self.job_level(profile)) {
                self.encounter
                    .as_mut()
                    .unwrap()
                    .refuse_weapon(profile, slot, now_ms);
                return Ok(out);
            }
        }
        let skill = match *action {
            SharedFieldAction::Ability { slot, .. } if slot > 0 => abilities
                .job_slot(profile, slot, self.job_level(profile))
                .cloned(),
            _ => None,
        };
        let engine = self
            .encounter
            .as_mut()
            .ok_or_else(|| invalid("shared combat requires an encounter engine"))?;
        if engine.active() || character.zone_id != engine.world().zone_id {
            return Ok(out);
        }
        let presence = field.restrict_to(known);
        let result: io::Result<EncounterOutcome> = (|| {
            Ok(match *action {
                SharedFieldAction::Ability { slot, target } if slot > 0 => match skill {
                    Some(ref ability) => engine.shared_job_ability(
                        field,
                        character.guid,
                        ability,
                        target,
                        character.position,
                        profile,
                        now_ms,
                    )?,
                    None => return Ok(EncounterOutcome::default()),
                },
                _ => engine.with_shared_fields(field, character.guid, |engine| {
                    engine.handle(
                        &action.request(),
                        character.guid,
                        character.position,
                        profile,
                        now_ms,
                    )
                })?,
            })
        })();
        field.restore_presence(presence);
        self.apply_encounter(result?, &mut out)?;
        Ok(out)
    }
    pub fn tick_shared_effects(
        &mut self,
        field: &mut crate::encounter::SharedFieldState,
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        if let Some(engine) = &mut self.encounter {
            let result = engine.shared_effects(field, character.guid, now_ms)?;
            self.apply_encounter(result, &mut out)?;
        }
        Ok(out)
    }
    pub fn tick_shared_motion(
        &mut self,
        field: &mut crate::encounter::SharedFieldState,
        players: &[crate::encounter::FieldParticipant],
        now_ms: u64,
        elapsed: u64,
    ) -> io::Result<(crate::encounter::EncounterOutcome, Vec<(u64, u32)>)> {
        self.encounter
            .as_mut()
            .ok_or_else(|| invalid("shared motion requires encounter context"))?
            .shared_motion(field, players, now_ms, elapsed)
    }
    pub fn receive_field_damage(
        &mut self,
        damage: u32,
        now_ms: u64,
        engaged: bool,
    ) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        self.field_engaged = engaged;
        if let Some(engine) = &mut self.encounter {
            let result = engine.receive_field_damage(damage, now_ms, engaged);
            self.apply_encounter(result, &mut out)?;
        }
        Ok(out)
    }
    /// (R) An instance transfer replaces this session's scene even when the
    /// zone id is unchanged; old ground pickups are no longer in the client.
    pub fn discard_ground_pickups(&mut self) {
        if let Some(encounter) = &mut self.encounter {
            encounter.discard_ground_pickups();
        }
    }
    /// How many ground pickups this session still holds (tests and diagnostics).
    pub fn pickup_count(&self) -> usize {
        self.encounter
            .as_ref()
            .map_or(0, crate::encounter::EncounterEngine::pickup_count)
    }

    /// W158: another group member picked up a big heart; heal this player too.
    pub fn receive_group_heal(&mut self, percent: u32) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        if let Some(engine) = &mut self.encounter {
            let result = engine.receive_group_heal(percent);
            self.apply_encounter(result, &mut out)?;
        }
        Ok(out)
    }

    pub fn load(directory: &Path, state: Option<GameplayState>) -> io::Result<Self> {
        let mut warp = piece_077::WarpRegistry::curated();
        let saved_derby = state.as_ref().and_then(|s| s.derby.clone());
        let saved_chess = state.as_ref().and_then(|s| s.chess);
        if let Some(saved) = state.as_ref().and_then(|s| s.warpstones.as_ref()) {
            warp.restore(saved);
        }
        let item_bar = crate::item_bar::ItemBar::load(
            directory,
            state.as_ref().and_then(|s| s.item_bar.clone()),
        )?;
        let progression_extras = crate::progression_extras::ProgressionExtras::load(
            state.as_ref().and_then(|s| s.progression_extras.clone()),
        )?;
        let achievement_bindings = crate::progression_extras::AchievementBindings::load(directory)?;
        let (
            inventory,
            quests,
            combat,
            profiles,
            identity_fingerprint,
            opening,
            encounter,
            smelting,
            forging,
            cooking,
            mining,
            harvesting,
            kart_entry,
            saved_npc_quests,
            mut learned_recipes,
        ) = match state {
            Some(state) => (
                Some(state.inventory),
                Some(state.quests),
                Some(state.combat),
                state.profiles,
                state.identity_fingerprint,
                state.opening,
                state.encounter,
                state.smelting,
                state.forging,
                state.cooking,
                state.mining,
                state.harvesting,
                state.kart_entry,
                state.npc_quests,
                state.learned_recipes,
            ),
            None => (
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                BTreeSet::new(),
            ),
        };
        let mut profiles = if directory.join("profiles.json").is_file() {
            Some(Profiles::load(directory, profiles)?)
        } else if profiles.is_some() {
            return Err(invalid("class snapshot requires its opening content"));
        } else {
            None
        };
        let (opening, encounter, world, journal, mut abilities) =
            if directory.join("opening.json").is_file() {
                if profiles.is_none() {
                    return Err(invalid("opening content requires class definitions"));
                }
                (
                    Some(OpeningEngine::load(directory, opening)?),
                    Some(EncounterEngine::load(directory, encounter)?),
                    Some(OpeningWorld::load(directory)?),
                    Some(OpeningJournal::load(directory)?),
                    Some(OpeningAbilities::load(directory)?),
                )
            } else if opening.is_some() || encounter.is_some() {
                return Err(invalid("opening snapshot requires its content"));
            } else {
                (None, None, None, None, None)
            };
        let smelting = if directory.join("smelting.json").is_file() {
            if opening.is_none() {
                return Err(invalid("smelting requires opening content"));
            }
            Some(Smelting::load(directory, smelting)?)
        } else if smelting.is_some() {
            return Err(invalid("smelting snapshot requires its content"));
        } else {
            None
        };
        // The isolated preparer does not silently enable139 in older campaigns.
        // Only an explicitly versioned, hash-bound campaign enables the new
        // original hammer objective and once-only tutorial material grants.
        let enabled = directory.join("forging-campaign.json");
        let (forging, disappearing_actors) = if enabled.is_file() {
            if std::fs::metadata(&enabled)?.len() > 4096 {
                return Err(invalid("forging campaign marker too large"));
            }
            let bytes = std::fs::read(enabled)?;
            if bytes.len() > 4096 {
                return Err(invalid("forging campaign marker too large"));
            }
            let marker: serde_json::Value = serde_json::from_slice(&bytes).map_err(invalid)?;
            if marker["schema_version"] != 1
                || marker["opening_sha256"].as_str()
                    != opening.as_ref().map(|o| o.state().content_sha256.as_str())
            {
                return Err(invalid(
                    "forging campaign marker does not match opening content",
                ));
            }
            let bindings: Vec<DisappearBinding> = serde_json::from_value(
                marker
                    .get("disappear_after_interaction")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!([])),
            )
            .map_err(invalid)?;
            if bindings.len() > 16 {
                return Err(invalid("too many disappearing actor bindings"));
            }
            let engine = opening.as_ref().unwrap();
            let journal = engine.journal_entries();
            let mut proof = Vec::new();
            std::fs::File::open(directory.join("opening.json"))?
                .take(524_289)
                .read_to_end(&mut proof)?;
            if proof.len() > 524_288
                || format!("{:x}", Sha256::digest(&proof)) != engine.state().content_sha256
            {
                return Err(invalid(
                    "opening content changed while loading visibility bindings",
                ));
            }
            let proof: serde_json::Value = serde_json::from_slice(&proof).map_err(invalid)?;
            let mut seen = BTreeSet::new();
            let mut disappearing = Vec::new();
            for binding in bindings {
                if !seen.insert(binding.actor_guid) || !engine.actors().iter().any(|a|
                    a.guid == binding.actor_guid && a.kind == "resource"
                    && matches!(a.action, Some(crate::opening::OpeningTrigger::Interact {actor_guid}) if actor_guid == a.guid)) {
                    return Err(invalid("invalid disappearing resource binding"));
                }
                let entry = journal
                    .iter()
                    .find(|e| e.quest_id == binding.stage_id)
                    .ok_or_else(|| invalid("disappearing actor stage missing"))?;
                let (index, goal) = entry
                    .objectives
                    .iter()
                    .enumerate()
                    .find(|(_, g)| g.id == binding.objective_id)
                    .ok_or_else(|| invalid("disappearing actor objective missing"))?;
                let matched = proof["stages"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|s| s["id"].as_u64() == Some(u64::from(binding.stage_id)))
                    .and_then(|s| s["objectives"].as_array())
                    .into_iter()
                    .flatten()
                    .find(|g| g["id"].as_u64() == Some(u64::from(binding.objective_id)))
                    .is_some_and(|g| {
                        g["trigger"]["kind"] == "interact"
                            && g["trigger"]["actor_guid"].as_u64() == Some(binding.actor_guid)
                    });
                if !matched {
                    return Err(invalid(
                        "disappearing actor does not match interaction objective",
                    ));
                }
                disappearing.push(DisappearingActor {
                    guid: binding.actor_guid,
                    stage_id: binding.stage_id,
                    index,
                    required: goal.required,
                });
            }
            (Some(Forging::load(directory, forging)?), disappearing)
        } else if forging.is_some() {
            return Err(invalid("forging snapshot requires its versioned campaign"));
        } else {
            (None, Vec::new())
        };
        // Cooking changes future gathering requirements. An explicit campaign
        // marker binds that compatibility transition to the new opening hash.
        let cooking_marker = directory.join("cooking-campaign.json");
        let cooking = if cooking_marker.is_file() {
            if std::fs::metadata(&cooking_marker)?.len() > 4096 {
                return Err(invalid("cooking campaign marker too large"));
            }
            let bytes = std::fs::read(&cooking_marker)?;
            if bytes.len() > 4096 {
                return Err(invalid("cooking campaign marker too large"));
            }
            let marker: serde_json::Value = serde_json::from_slice(&bytes).map_err(invalid)?;
            let recipe = directory.join("cooking.json");
            if std::fs::metadata(&recipe)?.len() > 32_768 {
                return Err(invalid("cooking content too large"));
            }
            if marker["schema_version"] != 1
                || marker["opening_sha256"].as_str()
                    != opening.as_ref().map(|o| o.state().content_sha256.as_str())
                || marker["cooking_sha256"].as_str()
                    != Some(format!("{:x}", Sha256::digest(std::fs::read(recipe)?)).as_str())
            {
                return Err(invalid("cooking campaign marker does not match content"));
            }
            Some(Cooking::load(directory, cooking)?)
        } else if cooking.is_some() {
            return Err(invalid("cooking snapshot requires its versioned campaign"));
        } else {
            None
        };
        // Mining18 is opt-in through `mining.json` next to the campaign content.
        // The reconstructed board only binds the accepted Miner job, and a saved
        // run without its content file fails closed.
        let mining = if directory.join(mining::CONTENT_FILE).is_file() {
            if opening.is_none() {
                return Err(invalid("mining requires opening content"));
            }
            let engine = Type8Engine::load(
                directory,
                mining::CONTENT_FILE,
                mining::MINING_GAME_ID,
                mining::FIRST_MINING_INSTANCE,
                mining,
            )?;
            if engine.profile_id() != mining::MINER_PROFILE_ID {
                return Err(invalid("mining content does not bind the Miner job"));
            }
            Some(engine)
        } else if mining.is_some() {
            return Err(invalid("mining snapshot requires its content"));
        } else {
            None
        };
        // Load the item catalogue once here so content bindings can be checked
        // against the definitions this save actually has.
        let inventory = Inventory::load(directory, inventory)?;
        if let Some(saved) = &saved_derby {
            derby_session::restore_saved_derby_job(saved, &mut profiles, &inventory)?;
        }
        let recipe_catalog = crate::recipes::RecipeCatalog::load_runtime(directory)?;
        // (R) The opening campaign historically has no recovered recipe-learn
        // packet sequence. Seed its two supplied starter patterns, the client
        // Copper Bar recipe (row 4) and the opening station's synthetic smelting
        // id so W61's cooking/smelting/forging flow remains usable.
        if opening.is_some() {
            learned_recipes.extend([11, 70, 4, 2002]);
        }
        let bundled_npc_content = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let npc_content = if directory.join("npc_spawns_extra.json").is_file()
            && directory.join("npcs").is_dir()
        {
            directory
        } else {
            bundled_npc_content.as_path()
        };
        // W162: the ride catalogue ships with the bundled content; a set without it has no rides.
        let mounts = if npc_content.join("mounts.json").is_file() {
            Some(crate::mounts::MountService::new(
                crate::mounts::MountCatalog::load(npc_content.join("mounts.json"))?,
            ))
        } else {
            None
        };
        let mut npc_visibility = if npc_content.join("npc_spawns_extra.json").is_file()
            && npc_content.join("npcs").is_dir()
        {
            Some(crate::world::NpcVisibility::load(npc_content)?)
        } else {
            None
        };
        // Overworld enemies replace their passive Legacy copies (same GUIDs).
        let mut encounter = encounter;
        if let Some(engine) = encounter.as_mut() {
            let behaviours = npc_content.join("behaviours.json");
            if behaviours.is_file() {
                engine.load_behaviours(&behaviours)?;
            }
            let path = npc_content.join("field_enemies.json");
            if path.is_file() {
                engine.load_field_enemies(&path)?;
                if let Some(visibility) = npc_visibility.as_mut() {
                    visibility.exclude_actor_guids(engine.field_guids());
                }
            }
            // W158: combat power-up drop tables (slice 1 data, slice 2 drops).
            let power_ups = npc_content.join("power_ups.json");
            if power_ups.is_file() {
                engine.load_power_ups(&power_ups)?;
            }
            let job_abilities = npc_content.join("job_abilities.json");
            if job_abilities.is_file() {
                if let Some(abilities) = abilities.as_mut() {
                    abilities.load_job_abilities(&job_abilities)?;
                }
            }
            let battles = npc_content.join("field_battles.json");
            if battles.is_file() {
                engine.load_field_battles(&battles)?;
                if let Some(visibility) = npc_visibility.as_mut() {
                    for (guid, name, name_id, model_id, position, icon) in engine.battle_spawners()
                    {
                        visibility.add_static_actor(guid, &name, name_id, model_id, position, icon);
                    }
                }
            }
        }
        let battle_file = npc_content.join("field_battles.json");
        // T022: optional chess rows; an absent file publishes none.
        let chess_path = npc_content.join(crate::chess_service::CONTENT_FILE);
        let chess = if chess_path.is_file() {
            Some(ChessService::with_state(
                crate::chess_service::ChessContent::load(&chess_path)?,
                saved_chess,
            )?)
        } else {
            None
        };
        let mut activities = if battle_file.is_file() {
            crate::activity_portal::Catalogue::load(&battle_file)?
        } else {
            Default::default()
        };
        if let Some(chess) = &chess {
            activities = activities.with_chess_rows(chess.activities())?;
        }
        let soccer_path = [directory, bundled_npc_content.as_path()]
            .into_iter()
            .map(|base| base.join(crate::soccer_entry::CONTENT_FILE))
            .find(|path| path.is_file());
        let soccer_content = match soccer_path {
            Some(path) => Some(Arc::new(crate::soccer_entry::Content::load(&path)?)),
            None => None,
        };
        if let Some(content) = &soccer_content {
            activities.add_soccer_rows(content.portal_rows())?;
        }
        let mut npc_interaction = if npc_content.join("npcs").is_dir()
            && npc_content.join("stores.json").is_file()
            && npc_content.join("vendors.json").is_file()
        {
            Some(NpcInteraction::load(
                &npc_content.join("npcs"),
                &npc_content.join("stores.json"),
            )?)
        } else {
            None
        };
        let npc_coin_shop = if npc_interaction.is_some() {
            let catalogue =
                crate::coin_shop::StoreCatalogue::load(&npc_content.join("stores.json"))?;
            // (R) Local bag capacity is 100 occupied stacks until an account capacity is wired.
            Some(crate::coin_shop::CoinShop::new(
                directory,
                inventory.clone(),
                catalogue,
                100,
            )?)
        } else {
            None
        };
        // Harvesting111 is a separate opt-in candidate bound to Chef45 and the
        // recovered farming assets. A saved run without its content fails closed.
        let harvesting = if directory.join(harvesting::CONTENT_FILE).is_file() {
            if opening.is_none() {
                return Err(invalid("harvesting requires opening content"));
            }
            let engine = Type8Engine::load(
                directory,
                harvesting::CONTENT_FILE,
                HARVESTING_GAME_ID,
                harvesting::FIRST_INSTANCE_ID,
                harvesting,
            )?;
            if engine.profile_id() != harvesting::CHEF_PROFILE_ID {
                return Err(invalid("harvesting content does not bind the Chef job"));
            }
            if engine.preload_file() != harvesting::FARMING_PRELOAD_FILE
                || engine.launch_movie() != harvesting::FARMING_LAUNCH_MOVIE
            {
                return Err(invalid(
                    "harvesting content does not bind the farming assets",
                ));
            }
            // Runtime registration gate: the shared engine stays permissive so the Mining
            // contract is unchanged, but an optional Harvest campaign must register every value
            // it can spawn before any offer. See notes/Research/Harvesting Runtime Tile Guard.md.
            let spec = engine.spec();
            let tiles: Vec<(u16, u16)> = spec
                .tiles
                .iter()
                .map(|kind| (kind.value, kind.category))
                .collect();
            harvesting::validate_tile_kinds_parts(&spec.selectable, &spec.collectable, &tiles)
                .map_err(|error| invalid(format!("harvesting tile registration: {error}")))?;
            // Border gate: a playable cell on shape row 0 or column 0 could never fall under the
            // original loops, so an unpadded persisted document is refused before any offer.
            harvesting::validate_border(&spec.shape, spec.width, spec.height)
                .map_err(|error| invalid(format!("harvesting border gate: {error}")))?;
            for output in &engine.spec().outputs {
                if !inventory.has_definition(output.definition_id) {
                    return Err(invalid(format!(
                        "harvesting reward definition {} is not in the loaded inventory",
                        output.definition_id
                    )));
                }
            }
            Some(engine)
        } else if harvesting.is_some() {
            return Err(invalid("harvesting snapshot requires its content"));
        } else {
            None
        };
        if [
            forging.as_ref().is_some_and(Forging::active),
            smelting.as_ref().is_some_and(Smelting::active),
            cooking.as_ref().is_some_and(Cooking::active),
            mining.as_ref().is_some_and(Type8Engine::active),
            harvesting.as_ref().is_some_and(Type8Engine::active),
        ]
        .into_iter()
        .filter(|active| *active)
        .count()
            > 1
        {
            return Err(invalid("multiple active minigames in snapshot"));
        }
        let lobby_path = directory.join("kart_lobby.json");
        let kart_lobby_reply = match std::fs::symlink_metadata(&lobby_path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
            Ok(metadata) => {
                const MAX_LOBBY_CONTENT: usize = 256 * 1024;
                if !metadata.is_file() || metadata.file_type().is_symlink() {
                    return Err(invalid("kart lobby content must be a regular file"));
                }
                let mut bytes = Vec::new();
                std::fs::File::open(&lobby_path)?
                    .take((MAX_LOBBY_CONTENT + 1) as u64)
                    .read_to_end(&mut bytes)?;
                if bytes.len() > MAX_LOBBY_CONTENT {
                    return Err(invalid("kart lobby content exceeds 256 KiB"));
                }
                let body = serde_json::from_slice(&bytes).map_err(invalid)?;
                Some(Arc::from(
                    kart_lobby_native::encode_inner(&body).map_err(invalid)?,
                ))
            }
        };
        let mystery_chests = MysteryChests::load(directory, &inventory)?.map(Arc::new);
        let kart_entry = crate::kart_gateway::KartGateway::load(directory, kart_entry)?;
        if let (Some(kart), Some(interaction)) = (&kart_entry, &mut npc_interaction) {
            let unknown = interaction.set_race_gates(kart.gate_npcs(), kart.menu_string_id());
            for npc in unknown {
                npc_event(
                    serde_json::json!({"event": "kart_gate_unknown_npc", "content_key": npc}),
                );
            }
        }
        let derby = crate::derby_gateway::DerbyGateway::load(directory)?;
        if let (Some(derby), Some(interaction)) = (&derby, &mut npc_interaction) {
            let unknown = interaction.set_derby_gates(derby.gate_npcs(), derby.menu_string_id());
            for npc in unknown {
                npc_event(
                    serde_json::json!({"event": "derby_gate_unknown_npc", "content_key": npc}),
                );
            }
        }
        let fishing_spots = crate::fishing_service::FishingSpots::load(directory)?.map(Arc::new);
        if let (Some(spots), Some(interaction)) = (&fishing_spots, &mut npc_interaction) {
            let unknown = interaction.set_fish_gates(&spots.gate_npcs(), spots.menu_string_id());
            for npc in unknown {
                npc_event(
                    serde_json::json!({"event": "fishing_gate_unknown_npc", "content_key": npc}),
                );
            }
        }
        let mut gathering_nodes = crate::harvest_nodes::GatheringNodes::load(npc_content)?;
        if let (Some(nodes), Some(visibility)) = (gathering_nodes.as_mut(), npc_visibility.as_mut())
        {
            apply_node_changes(visibility, nodes.sync(0, false));
        }
        let station_content = if npc_content.join("crafting_stations.json").is_file() {
            npc_content
        } else {
            bundled_npc_content.as_path()
        };
        let world_station_rules = crate::recipe_book::load_world_station_rules(station_content)?;
        if let (Some(profiles), Some(interaction)) = (&profiles, &mut npc_interaction) {
            let trainers = profiles.load_trainers(directory)?;
            let unknown = interaction.set_job_trainers(trainers);
            for npc in unknown {
                npc_event(
                    serde_json::json!({"event": "job_trainer_unknown_npc", "content_key": npc}),
                );
            }
        }
        Ok(Self {
            mystery_chests,
            pending_chest: None,
            npc_visibility,
            npc_interaction,
            last_npc_dialogue: None,
            pending_quest_choice: None,
            pending_choice_packets: Vec::new(),
            npc_coin_shop,
            npc_quests: None,
            saved_npc_quests,
            breadcrumb_quests: std::collections::BTreeMap::new(),
            breadcrumb_crosswalk: Default::default(),
            client_objectives: Default::default(),
            tracked_pin: crate::quest_journal::Tracker::default(),
            npc_quests_synced: false,
            game_rules: GameRuleSet::default(),
            profile_content_directory: directory
                .join("profiles.json")
                .is_file()
                .then(|| directory.to_path_buf()),
            recipe_catalog,
            learned_recipes,
            warp,
            item_bar,
            mounts,
            field_engaged: false,
            pets: crate::pets::PetService::new(crate::pets::PetService::bundled_content()?)?,
            progression_extras,
            achievement_bindings,
            inventory,
            categories: ItemCategories::load(directory)?,
            quests: QuestEngine::load(directory, quests)?,
            combat: Combat::load(directory, combat).map_err(invalid)?,
            profiles,
            identity_fingerprint,
            gender: 0,
            opening,
            encounter,
            world,
            journal,
            abilities,
            smelting,
            forging,
            cooking,
            mining,
            harvesting,
            forging_tick_ms: None,
            type8_tick_ms: None,
            disappearing_actors,
            dev_tools: false,
            terrain: None,
            pathing: crate::pathing::Pathing::default(),
            world_dir: None,
            stock_text: None,
            role: crate::account_store::AccountRole::Player,
            role_commands: false,
            kart_lobby_reply,
            activities,
            soccer_content,
            soccer: None,
            chess,
            portal_refreshed_ms: None,
            kart_entry,
            derby,
            fishing_spots,
            fishing: Default::default(),
            gathering_nodes,
            world_station_rules,
            content_dir: npc_content.to_path_buf(),
        })
    }

    /// Enable the developer-only chat commands. Off by default; the local
    /// launcher exposes it only through its explicit `--dev-tools` flag.
    pub fn set_dev_tools(&mut self, enabled: bool) {
        self.dev_tools = enabled;
    }

    /// Trust the account-store role for `realm` commands (decision 1).
    /// Diagnostics and wire experiments still need `--dev-tools`.
    pub fn set_role_commands(&mut self, enabled: bool) {
        self.role_commands = enabled;
    }

    pub fn set_terrain(&mut self, terrain: Option<std::sync::Arc<crate::terrain::Terrain>>) {
        if let Some(encounter) = self.encounter.as_mut() {
            encounter.set_terrain(terrain.clone());
        }
        self.terrain = terrain;
    }

    /// Ground height under (x, z) in a zone, when terrain data is loaded.
    pub fn ground_height(&self, zone: u32, x: f32, z: f32) -> Option<f32> {
        self.terrain.as_ref()?.height_at(zone, x, z)
    }

    /// The active job profile, when profiles are loaded (other players' AddPc).
    pub fn active_profile(&self) -> Option<i32> {
        self.profiles.as_ref().map(|p| p.state().active)
    }

    /// What other players may see on Inspect (W143): unlocked jobs with their
    /// ranks and the active job's worn items. Coins, quests, stats and titles
    /// are never included (R).
    pub fn public_look(&self) -> Option<crate::player_radial::PublicLook> {
        let profiles = self.profiles.as_ref()?;
        let active = profiles.state().active;
        let state = self.inventory.state();
        let items = state
            .equipment
            .iter()
            .filter(|worn| worn.profile_id == active && worn.item_id != 0)
            .filter_map(|worn| {
                let item = state.items.iter().find(|item| item.id == worn.item_id)?;
                Some(crate::player_radial::PublicItem {
                    slot: worn.slot,
                    definition_id: item.definition_id,
                    tint: item.tint,
                    definition_record: self
                        .inventory
                        .definition_record(item.definition_id)?
                        .to_vec(),
                })
            })
            .collect();
        Some(crate::player_radial::PublicLook {
            profiles: profiles
                .state()
                .unlocked
                .iter()
                .map(|&id| (id, self.job_level(id) as i32))
                .collect(),
            active_profile: active,
            items,
        })
    }

    /// A battle or dungeon run is under way, so the player is busy elsewhere (W163 trade).
    pub fn in_encounter(&self) -> bool {
        self.encounter
            .as_ref()
            .is_some_and(|encounter| encounter.active())
    }

    /// The inventory a trade validates offers against (W163): this character's items and
    /// coins plus the trade flags of the definitions it owns. `None` when the inventory does
    /// not validate, so such a character cannot trade. (R) The bag holds 100 stacks like the
    /// reward bag (see `RewardState` below); nothing is equipped on a pet yet.
    /// The live inventory a trade commit compares with the settlement's `before` (lead).
    pub fn inventory_state(&self) -> &crate::inventory::InventoryState {
        self.inventory.state()
    }

    /// Adopt a settled trade's validated inventory; returns the client updates (lead).
    pub fn adopt_trade_inventory(
        &mut self,
        after: crate::inventory::InventoryState,
    ) -> io::Result<Vec<Vec<u8>>> {
        self.inventory.adopt_state(after)
    }

    pub fn trade_inventory(&self) -> Option<crate::trade::TradeInventory> {
        let state = self.inventory.state();
        let definitions = state
            .items
            .iter()
            .filter_map(|item| {
                let (no_trade, max_stack) = self.inventory.trade_flags(item.definition_id)?;
                let definition = crate::trade::TradeItemDefinition {
                    no_trade,
                    max_stack,
                };
                Some((item.definition_id, definition))
            })
            .collect();
        crate::trade::TradeInventory::new(
            state.clone(),
            definitions,
            Default::default(),
            state.items.len().max(100),
        )
        .ok()
    }

    /// One profile's outfit as the native attachment list (count + rows).
    pub fn profile_attachments(&self, profile_id: i32) -> io::Result<Vec<u8>> {
        self.inventory.profile_attachments(profile_id)
    }

    /// (R) 35/7 EquippedItemsChange for a job switch: the new active outfit,
    /// which the client applies as a profile-swap attachment group. Client
    /// reader 0x0091af50 (ctor 0x0091d3b0).
    fn profile_switch_packet(&self, guid: u64, attachments: &[u8]) -> Vec<u8> {
        let mut packet = vec![35, 0, 7, 0];
        packet.extend(guid.to_le_bytes());
        packet.extend_from_slice(attachments);
        packet.extend(0i32.to_le_bytes()); // (R) trailing value; the client reads one i32.
        packet
    }

    /// Announce the active outfit after the complete profile operation, not
    /// just a change of profile id: activating the same job can restore a
    /// missing starter outfit.
    fn announce_profile_outfit(
        &self,
        guid: u64,
        before: &[u8],
        switched: bool,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let Some(profiles) = &self.profiles else {
            return Ok(());
        };
        let after = self.inventory.profile_attachments(profiles.active())?;
        if !switched && after == before {
            return Ok(());
        }
        out.nearby.push(self.profile_switch_packet(guid, &after));
        Ok(())
    }

    /// T027: whether an equip names an inactive job (whose 35/6 must not reach
    /// the owner or neighbours), and the 35/6 a neighbour needs for an active
    /// change.
    fn inventory_visibility(
        &self,
        inner: &[u8],
        guid: u64,
        changed: bool,
        replies: &[Vec<u8>],
    ) -> (bool, Option<Vec<u8>>) {
        let target_profile = if inner.starts_with(&[42, 0, 2, 0]) && inner.len() == 12 {
            Some(i32::from_le_bytes(inner[8..12].try_into().unwrap()))
        } else if inner.starts_with(&[42, 0, 3, 0]) && inner.len() == 16 {
            Some(i32::from_le_bytes(inner[8..12].try_into().unwrap()))
        } else if inner.starts_with(&[42, 0, 8, 0]) && inner.len() == 20 {
            Some(i32::from_le_bytes(inner[12..16].try_into().unwrap()))
        } else {
            None
        };
        let active = self.profiles.as_ref().map(|profiles| profiles.active());
        let inactive = active
            .zip(target_profile)
            .is_some_and(|(active, target)| active != target);
        let packet = (changed && target_profile.is_some() && !inactive)
            .then(|| item_change_packet(inner, guid, replies))
            .flatten();
        (inactive, packet)
    }

    pub fn state(&self) -> GameplayState {
        let progression_extras =
            (!self.progression_extras.is_empty()).then(|| self.progression_extras.state());
        GameplayState {
            inventory: self.inventory.state().clone(),
            quests: self.quests.state().clone(),
            combat: self.combat.state(),
            npc_quests: self
                .npc_quests
                .as_ref()
                .map(|engine| engine.state().clone())
                .or_else(|| self.saved_npc_quests.clone()),
            learned_recipes: self.learned_recipes.clone(),
            profiles: self.saved_profile_state(),
            identity_fingerprint: self.identity_fingerprint.clone(),
            opening: self.opening.as_ref().map(|o| o.state().clone()),
            encounter: self.encounter.as_ref().map(|e| e.state().clone()),
            smelting: self.smelting.as_ref().map(|s| s.state().clone()),
            forging: self.forging.as_ref().map(|s| s.state().clone()),
            cooking: self.cooking.as_ref().map(|s| s.state().clone()),
            mining: self.mining.as_ref().map(|m| m.state().clone()),
            harvesting: self.harvesting.as_ref().map(|h| h.state().clone()),
            kart_entry: self.kart_entry.as_ref().map(|k| k.state().clone()),
            derby: None,
            chess: self.chess.as_ref().and_then(ChessService::ledger),
            warpstones: Some(self.warp.state()).filter(|s| !s.is_empty()),
            item_bar: Some(self.item_bar.clone()).filter(|b| !b.is_empty()),
            progression_extras,
        }
    }

    /// Restore the character's rolling-hour fishing cap ledger after a reconnect. This is
    /// host-owned state, deliberately outside `GameplayState` until the save schema is extended.
    pub fn restore_fishing_hours(&mut self, guid: u64, hours: Vec<u64>) -> io::Result<()> {
        self.fishing
            .restore_hour_history(guid, hours)
            .map_err(|error| invalid(error.to_string()))
    }

    pub fn fishing_hour_history(&self, guid: u64) -> Vec<u64> {
        self.fishing.hour_history(guid)
    }

    /// W178: trusted store/event code queues a server-owned claim. No client
    /// request reaches this method or supplies its reward payload.
    pub fn issue_claim(
        &mut self,
        claim_id: u64,
        grant: crate::rewards::RewardGrant,
    ) -> io::Result<()> {
        self.progression_extras.issue_server_grant(claim_id, grant)
    }

    /// W178: award titles only for achievements completed by the existing
    /// achievement engine during this transition.
    pub fn award_achievement_titles(
        &mut self,
        newly_completed: &[String],
        bindings: &[crate::progression_extras::AchievementTitleBinding],
    ) -> io::Result<Vec<Vec<u8>>> {
        self.progression_extras
            .award_completed_achievement_titles(newly_completed, bindings)
    }

    fn recipe_replace_all_packet(&self) -> io::Result<Vec<u8>> {
        let learned: HashSet<u32> = self.learned_recipes.iter().copied().collect();
        let rows = self
            .recipe_catalog
            .wire_rows(&learned, &self.inventory)
            .map_err(invalid)?;
        crate::recipes::encode_recipe_replace_all(&rows).map_err(invalid)
    }

    /// Persist one client recipe id and return its native 0x3f/op1 packet once.
    pub fn learn_recipe(&mut self, recipe_id: u32) -> io::Result<Option<Vec<u8>>> {
        let Some(recipe) = self
            .recipe_catalog
            .records
            .iter()
            .find(|recipe| recipe.recipe_id == Some(recipe_id))
        else {
            return Err(invalid("unknown client recipe id"));
        };
        if recipe.recipe_item_definition_id.is_none() {
            return Err(invalid("learned recipe has no client pattern item"));
        }
        if self.learned_recipes.contains(&recipe_id) {
            return Ok(None);
        }
        let learned = HashSet::from([recipe_id]);
        let row = self
            .recipe_catalog
            .wire_rows(&learned, &self.inventory)
            .map_err(invalid)?
            .into_iter()
            .next()
            .ok_or_else(|| invalid("learned recipe has no encodable wire row"))?;
        let packet = crate::recipes::encode_recipe_add(&row.1).map_err(invalid)?;
        self.learned_recipes.insert(recipe_id);
        Ok(Some(packet))
    }

    fn learned_recipe_for_key(&self, key: &str) -> bool {
        self.recipe_catalog
            .by_key(key)
            .and_then(|recipe| recipe.recipe_id)
            .is_some_and(|id| self.learned_recipes.contains(&id))
            // (R) The opening tutorial's synthetic smelting id also unlocks the Copper Bar sample.
            || (key == "miner.copper_bar.sample" && self.learned_recipes.contains(&2002))
    }

    fn recipe_rank(&self, profile_id: i32) -> u8 {
        self.profiles
            .as_ref()
            .and_then(|profiles| profiles.state().experience.get(&profile_id))
            .map_or(1, |xp| (1 + xp / 100).min(20) as u8)
    }

    fn available_ingredients(&self) -> Vec<(i32, u32)> {
        let mut available = std::collections::BTreeMap::<i32, u32>::new();
        for item in &self.inventory.state().items {
            if item.rental.is_none() {
                let count = available.entry(item.definition_id).or_default();
                *count = count.saturating_add(item.count);
            }
        }
        available.into_iter().collect()
    }

    fn learn_consumed_patterns(&mut self, before: &InventoryState) -> io::Result<Vec<Vec<u8>>> {
        let count = |items: &[crate::inventory::InventoryItem], definition_id: i32| {
            items
                .iter()
                .filter(|item| item.definition_id == definition_id)
                .map(|item| item.count)
                .fold(0u32, u32::saturating_add)
        };
        let mut packets = Vec::new();
        let candidates: Vec<u32> = self
            .recipe_catalog
            .records
            .iter()
            .filter_map(|recipe| {
                let id = recipe.recipe_id?;
                let pattern = recipe.recipe_item_definition_id?;
                (count(&before.items, pattern) > count(&self.inventory.state().items, pattern))
                    .then_some(id)
            })
            .collect();
        for id in candidates {
            if let Some(packet) = self.learn_recipe(id)? {
                packets.push(packet);
            }
        }
        Ok(packets)
    }

    pub fn reset_session(&mut self) {
        self.pending_chest = None;
        self.forging_tick_ms = None;
        self.type8_tick_ms = None;
        self.quests.reset_session();
        self.field_engaged = false;
        if let Some(mounts) = &mut self.mounts {
            mounts.reset();
        }
        self.pets.reset_session();
        if let Some(opening) = &mut self.opening {
            opening.reset_session();
        }
        if let Some(encounter) = &mut self.encounter {
            encounter.reset_session();
        }
        // A derby match belongs to one connection (R): drop it and restore the job it replaced.
        self.retire_derby_for_reset();
        // W160: a fishing round belongs to one connection. Paid catches are already saved;
        // only the cast in flight is dropped.
        self.fishing.reset_all();
    }

    /// Race clocks belong to one connection. A reconnect returns to the saved
    /// world pose rather than resuming a race against a restarted clock.
    pub fn abandon_kart_for_reconnect(&mut self) -> Option<CharacterState> {
        let kart = self.kart_entry.as_mut()?;
        let previous = kart.previous_job();
        let location = kart.exit();
        if let (Some(job), Some(profiles)) = (previous, self.profiles.as_mut()) {
            // Self data at login carries the job, so the activation reply is not needed.
            let _ = profiles.activate(job);
        }
        location
    }

    /// A save taken inside a battle arena loads with the battle dropped (the
    /// reward was already granted on the win); this is where it started.
    pub fn abandon_battle_for_reconnect(&mut self) -> Option<CharacterState> {
        self.encounter
            .as_mut()
            .and_then(EncounterEngine::take_dropped_return)
    }

    pub fn summary(&self) -> serde_json::Value {
        serde_json::json!({"coins": self.inventory.state().coins, "inventory_instances": self.inventory.state().items.len(), "quests":self.quests.state(), "combat":self.combat.state(), "profiles":self.profiles.as_ref().map(Profiles::state),"identity":self.identity_fingerprint,"opening":self.opening.as_ref().map(OpeningEngine::state),"encounter":self.encounter.as_ref().map(EncounterEngine::state),"smelting":self.smelting.as_ref().map(Smelting::state),"forging":self.forging.as_ref().map(Forging::state),"cooking":self.cooking.as_ref().map(Cooking::state),"mining":self.mining.as_ref().map(Type8Engine::state),"harvesting":self.harvesting.as_ref().map(Type8Engine::state)})
    }

    pub fn patch_self(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        // The quest span follows inventory in the original self payload. Apply
        // the later splice first so inventory's original offsets stay valid.
        let bytes = match &self.profiles {
            Some(profiles) => profiles.patch_self(bytes)?,
            None => bytes.to_vec(),
        };
        let bytes = match &self.journal {
            Some(journal) => {
                journal.patch_self(&bytes, &self.opening.as_ref().unwrap().journal_entries())?
            }
            None => self.quests.patch_self_template(&bytes)?,
        };
        self.inventory.initialization_patch(&bytes)
    }

    pub fn definitions_packet(&self) -> Option<Vec<u8>> {
        self.profiles
            .as_ref()
            .map(|_| self.inventory.definitions_packet())
    }

    pub fn is_opening(&self) -> bool {
        self.opening.is_some()
    }

    /// True while a session-local soccer match owns the character.
    pub fn is_soccer_active(&self) -> bool {
        self.soccer.is_some()
    }

    pub fn soccer_score(&self) -> Option<[u32; 2]> {
        self.soccer
            .as_ref()
            .map(crate::soccer_entry::Session::score)
    }

    pub fn soccer_phase(&self) -> Option<crate::soccer_match::MatchPhase> {
        self.soccer
            .as_ref()
            .map(crate::soccer_entry::Session::phase)
    }

    /// True while a demolition derby match or its results screen belongs to this session.
    pub fn derby_active(&self) -> bool {
        self.derby
            .as_ref()
            .is_some_and(crate::derby_gateway::DerbyGateway::active)
    }

    pub fn initial_character(
        directory: &Path,
        mut reference: CharacterState,
    ) -> io::Result<CharacterState> {
        if directory.join("opening.json").is_file() {
            let encounter = EncounterEngine::load(directory, None)?;
            reference.zone_id = encounter.world().zone_id;
            reference.position = encounter.world().position;
            reference.rotation = encounter.world().rotation;
        }
        Ok(reference)
    }

    pub fn supports_location(&self, character: &CharacterState) -> bool {
        if self.soccer.is_some() {
            return true;
        }
        if let Some(kart) = self.kart_entry.as_ref().filter(|k| k.active()) {
            return kart.supports_location(character);
        }
        self.encounter
            .as_ref()
            .map(|e| e.supports_location(character))
            .unwrap_or(character.zone_id == 0)
    }

    pub fn current_zone(&self) -> Option<ZoneTransition> {
        if let Some(scene) = self.kart_entry.as_ref().and_then(|k| k.scene()) {
            return Some(ZoneTransition {
                zone_id: scene.id as u32,
                name: scene.scene.clone(),
                sky: scene.sky.clone(),
                geometry_id: scene.geometry_id,
                position: scene.position[..3].try_into().unwrap(),
                rotation: scene.rotation[..3].try_into().unwrap(),
                tutorial: scene.tutorial,
            });
        }
        self.encounter
            .as_ref()
            .map(|e| e.zone().unwrap_or_else(|| e.world().clone()))
    }

    pub fn patch_gender(&self, template: &[u8], gender: i32) -> io::Result<Vec<u8>> {
        self.inventory.patch_gender(template, gender)
    }

    pub fn initialize_world(
        &mut self,
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        if let Some(soccer) = self.soccer.as_mut() {
            return Ok(Outcome {
                replies: soccer.loaded_replies(),
                ..Outcome::default()
            });
        }
        if let Some(kart) = self.kart_entry.as_ref().filter(|k| k.active()) {
            return Ok(Outcome {
                replies: kart.loaded()?,
                ..Outcome::default()
            });
        }
        if self.opening.is_none() {
            let mut out = self.initialization(character.position)?;
            self.append_npc_quest_login_sync(&mut out)?;
            self.npc_location_event(character, &mut out)?;
            self.sync_tracked_target(character, &mut out);
            self.append_warp_definitions(character, &mut out);
            self.append_activity_list(&mut out);
            self.append_item_bar(now_ms, &mut out)?;
            out.replies.push(self.pets.list_packet());
            self.sync_client_objectives(&mut out);
            self.sync_achievement_progress(&mut out, None)?;
            return Ok(out);
        }
        let before = self.opening.as_ref().unwrap().journal_entries();
        let mut out = Outcome::default();
        // A fresh movie cannot reuse data_sent from a disconnected movie.
        // Keep attempt IDs/completions, cancel without consuming or awarding.
        if let Some(smelting) = &mut self.smelting {
            let cancelled = smelting.cancel();
            out.changed |= cancelled.changed;
            out.replies.extend(cancelled.replies);
        }
        if let Some(cooking) = &mut self.cooking {
            let cancelled = cooking.cancel();
            out.changed |= cancelled.changed;
            out.replies.extend(cancelled.replies);
        }
        if let Some(forging) = &mut self.forging {
            let cancelled = forging.cancel();
            out.changed |= cancelled.changed;
            out.replies.extend(cancelled.replies);
        }
        // (R) Same rule for the Type8 movies: a reconnecting client has no board or tile queue,
        // so an unfinished Mining/Harvesting run (even a paid Won run waiting for results Close)
        // is released here instead of blocking jobs, equipment and interaction forever.
        for engine in [&mut self.mining, &mut self.harvesting]
            .into_iter()
            .flatten()
        {
            let cancelled = engine.cancel();
            out.changed |= cancelled.changed;
            out.replies.extend(cancelled.replies);
        }
        self.type8_tick_ms = None;
        self.forging_tick_ms = None;
        out.replies.push(self.categories.initialization());
        let profile = self.profiles.as_ref().unwrap().active();
        out.replies.push(
            self.profiles
                .as_ref()
                .unwrap()
                .activation(profile, &self.inventory)?,
        );
        out.replies.extend(
            self.abilities
                .as_ref()
                .unwrap()
                .activation(profile, self.job_level(profile))?,
        );
        let result = self
            .opening
            .as_mut()
            .unwrap()
            .on_position(character.zone_id, character.position)?;
        self.apply_opening(result, &mut out)?;
        self.opening_equipment_events(&mut out)?;
        let result = self.opening.as_ref().unwrap().initialization();
        self.apply_opening(result, &mut out)?;
        if !self.encounter.as_ref().unwrap().active() {
            let visible: Vec<_> = self
                .opening
                .as_ref()
                .unwrap()
                .actors()
                .iter()
                .filter(|actor| !self.actor_hidden(actor.guid))
                .cloned()
                .collect();
            out.replies.extend(
                self.world
                    .as_ref()
                    .unwrap()
                    .initialization(&visible, character.zone_id),
            );
        }
        self.sync_world_stage(&mut out)?;
        self.sync_field_quests(&mut out)?;
        self.sync_player_level(&mut out);
        let result = self
            .encounter
            .as_mut()
            .unwrap()
            .on_zone_ready(profile, now_ms)?;
        self.apply_encounter(result, &mut out)?;
        self.append_journal_updates(&before, &mut out)?;
        self.append_npc_quest_login_sync(&mut out)?;
        self.npc_location_event(character, &mut out)?;
        self.sync_tracked_target(character, &mut out);
        self.append_warp_definitions(character, &mut out);
        self.append_activity_list(&mut out);
        self.append_item_bar(now_ms, &mut out)?;
        out.replies.push(self.recipe_replace_all_packet()?);
        out.replies.push(self.pets.list_packet());
        self.sync_client_objectives(&mut out);
        self.sync_achievement_progress(&mut out, None)?;
        Ok(out)
    }

    pub fn on_location(&mut self, character: &CharacterState, now_ms: u64) -> io::Result<Outcome> {
        let owned_movement = self.blocks(|gates| gates.location);
        if owned_movement {
            return Ok(Outcome::default());
        }
        let mut out = Outcome::default();
        let visited = self.warp.visit(&self.warp_context(character));
        out.changed |= visited.changed;
        out.replies.extend(visited.replies);
        out.replies
            .extend(visited.notices.iter().map(|text| chat::system(text)));
        // gateway.rs calls this only after movement validation accepted/clamped
        // the pose. Engine checks active objectives indexed by current zone.
        self.npc_quest_event(
            crate::quest_engine::QuestEvent::VisitPosition {
                zone_id: character.zone_id,
                position: character.position,
            },
            &mut out,
        )?;
        if self.opening.is_none() {
            let result = self.on_position(character.position, now_ms)?;
            out.changed |= result.changed;
            out.replies.extend(result.replies);
            return Ok(out);
        }
        let before = self.opening.as_ref().unwrap().journal_entries();
        let result = self
            .opening
            .as_mut()
            .unwrap()
            .on_position(character.zone_id, character.position)?;
        self.apply_opening(result, &mut out)?;
        self.append_journal_updates(&before, &mut out)?;
        Ok(out)
    }

    /// The native racing capture registers the kart between Ready13 and
    /// Loaded10. Sending entry after Loaded creates a circular wait.
    pub fn kart_ready_stream(&self) -> io::Result<Option<Vec<Vec<u8>>>> {
        self.kart_entry
            .as_ref()
            .filter(|k| k.active())
            .map(|k| {
                let mut replies = k.initialize()?;
                // Same recovered preload-complete message as the normal Ready
                // stream; the client resets this latch on every scene transition.
                replies.push(vec![38, 0, 26, 0, 0]);
                replies.push(vec![14, 0]);
                Ok(replies)
            })
            .transpose()
    }

    pub fn accepts_kart_loading_application(&self, inner: &[u8]) -> bool {
        self.kart_entry.as_ref().is_some_and(|k| k.active())
            && (inner.starts_with(&[33, 0]) || inner.starts_with(&[102, 0]))
    }

    pub fn tick(&mut self, character: &CharacterState, now_ms: u64) -> io::Result<Outcome> {
        let counters_before = self.achievement_counters();
        // W160: bite timers and the leash run beside the world tick; nothing here is saved.
        let fishing = match self.fishing.supervise(
            character.guid,
            character.zone_id,
            character.position,
            now_ms,
        ) {
            Ok(Some(change)) => change.replies,
            _ => Vec::new(),
        };
        let mut out = self.tick_world(character, now_ms)?;
        self.tick_gathering_nodes(now_ms, &mut out)?;
        out.replies.splice(0..0, fishing);
        self.settle_achievement_counters(&counters_before, &mut out)?;
        Ok(out)
    }

    fn tick_world(&mut self, character: &CharacterState, now_ms: u64) -> io::Result<Outcome> {
        if self.soccer.is_some() {
            let mut staged = self.clone();
            let result = staged
                .soccer
                .as_mut()
                .unwrap()
                .tick(character.guid, now_ms)?;
            // The match is session-local; periodic clock ticks must not force
            // a character save. Only an adopted reward marks a persisted change.
            let mut out = Outcome {
                replies: result.replies,
                ..Outcome::default()
            };
            if let Some(rewards) = result.reward_state {
                staged.adopt_reward_state(rewards, &mut out)?;
            }
            *self = staged;
            return Ok(out);
        }
        if self
            .derby
            .as_ref()
            .is_some_and(crate::derby_gateway::DerbyGateway::active)
        {
            return self.tick_derby(character, now_ms);
        }
        if self.kart_entry.as_ref().is_some_and(|k| k.active()) {
            let mut staged = self.clone();
            let kart = staged.kart_entry.as_mut().unwrap();
            let mut before = kart.state().clone();
            let mut replies = kart.tick(now_ms)?;
            if kart
                .state()
                .race
                .as_ref()
                .is_some_and(|race| race.phase == crate::kart_service::RacePhase::Aborted)
            {
                // Reuse ordinary race return so a timed-out kart cannot strand the player.
                let mut returned =
                    staged.handle_kart_entry(&[], Some("realm exit"), character, now_ms)?;
                replies.append(&mut returned.replies);
                returned.replies = replies;
                *self = staged;
                return Ok(returned);
            }
            let claim = if let Some(profiles) = &mut staged.profiles {
                kart.claim_reward_with_event(&mut staged.inventory, profiles)?
            } else {
                crate::kart_gateway::KartRewardClaim::default()
            };
            replies.extend(claim.replies);
            // A racing tick re-stamps the session-local weapon clock every 50 ms. Copy the new
            // clock into the old state before comparing, so only persisted changes request a
            // save; every other field (including one added later) is compared structurally.
            if let (Some(before), Some(after)) = (before.race.as_mut(), kart.state().race.as_ref())
            {
                before.weapons.last_tick_ms = after.weapons.last_tick_ms;
            }
            let changed = before != *kart.state();
            staged.refresh_npc_quest_eligibility(current_unix_seconds());
            let mut out = Outcome {
                replies,
                changed,
                ..Default::default()
            };
            if let Some(minigame_id) = claim.minigame_id {
                staged.npc_quest_event(
                    crate::quest_engine::QuestEvent::FinishMinigame { minigame_id },
                    &mut out,
                )?;
            }
            *self = staged;
            return Ok(out);
        }
        let mut out = Outcome::default();
        if self.forging.as_ref().is_some_and(Forging::active) {
            let previous = self.forging_tick_ms.unwrap_or(now_ms);
            let elapsed = now_ms.saturating_sub(previous);
            // Bound snapshot writes while refreshing the server timer. The
            // gateway stages/persists this state before replies just like input.
            if elapsed >= 250 {
                let result = self
                    .forging
                    .as_mut()
                    .unwrap()
                    .tick(elapsed.min(u32::MAX as u64) as u32)?;
                self.apply_craft(CraftingJob::Blacksmith, result, character, &mut out)?;
                self.forging_tick_ms = Some(now_ms);
            }
            return Ok(out);
        }
        if self.type8(Gather::Mining).is_some_and(Type8Engine::active) {
            let previous = self.type8_tick_ms.unwrap_or(now_ms);
            let elapsed = now_ms.saturating_sub(previous);
            // Bound snapshot writes while refreshing the authoritative timer.
            // The gateway persists this state before replies just like input.
            if elapsed >= 250 {
                let result = self
                    .type8_mut(Gather::Mining)
                    .unwrap()
                    .tick(elapsed.min(u32::MAX as u64) as u32)?;
                self.apply_type8(Gather::Mining, result, &mut out)?;
                self.type8_tick_ms = Some(now_ms);
            }
            return Ok(out);
        }
        if self
            .type8(Gather::Harvesting)
            .is_some_and(Type8Engine::active)
        {
            let previous = self.type8_tick_ms.unwrap_or(now_ms);
            let elapsed = now_ms.saturating_sub(previous);
            // Bound snapshot writes while refreshing the authoritative timer.
            // The gateway persists this state before replies just like input.
            if elapsed >= 250 {
                let result = self
                    .type8_mut(Gather::Harvesting)
                    .unwrap()
                    .tick(elapsed.min(u32::MAX as u64) as u32)?;
                self.apply_type8(Gather::Harvesting, result, &mut out)?;
                self.type8_tick_ms = Some(now_ms);
            }
            return Ok(out);
        }
        if self.smelting.as_ref().is_some_and(Smelting::active)
            || self.cooking.as_ref().is_some_and(Cooking::active)
        {
            return Ok(out);
        }
        self.sync_field_quests(&mut out)?;
        if let Some(encounter) = &mut self.encounter {
            // W158: pickups left in another zone are already gone from the client.
            encounter.enter_zone(character.zone_id);
            let result = encounter.tick(
                character.position,
                self.profiles.as_ref().unwrap().active(),
                now_ms,
            )?;
            self.apply_encounter(result, &mut out)?;
        }
        Ok(out)
    }

    /// W162: true while the rider sits on a ride (the emote rules refuse emotes then).
    pub fn is_mounted(&self) -> bool {
        self.mounts
            .as_ref()
            .is_some_and(crate::mounts::MountService::is_mounted)
    }

    /// W162: the summoned ride as other players must see it; `None` without one.
    pub fn ride_view(&self, guid: u64) -> Option<crate::mounts::RideView> {
        self.mounts.as_ref()?.view(guid)
    }

    /// (R) No per-zone riding data was recovered, so every zone allows rides; fights (an instance
    /// battle or a field enemy chasing the player), kart races and a knockout do not.
    fn ride_restrictions(&self) -> crate::mounts::RideRestrictions {
        let in_minigame = self.blocks(|gates| gates.ride);
        crate::mounts::RideRestrictions {
            zone_allows_riding: true,
            in_minigame,
            ability_active: false,
            knocked_out: self
                .encounter
                .as_ref()
                .is_some_and(|e| e.state().player_health == 0),
        }
    }

    fn ride_outcome(wire: crate::mounts::RideWire) -> Outcome {
        let mut out = Outcome::default();
        if let Some(notice) = wire.notice {
            out.replies.push(chat::system(notice));
        }
        out.replies.extend(wire.private);
        out.replies.extend(wire.shared.iter().cloned());
        out.nearby = wire.shared;
        out
    }

    /// W162: summon, board and put away a ride (family 168). Ownership is read from the
    /// inventory on every request.
    fn handle_mounts(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let restrictions = self.ride_restrictions();
        let Some(mounts) = self.mounts.as_mut() else {
            return Ok(None);
        };
        let context = crate::mounts::RideContext {
            guid: character.guid,
            inventory: self.inventory.state(),
            restrictions,
            pose: crate::mounts::RidePose {
                position: character.position,
                heading: character.rotation[0].atan2(character.rotation[2]),
            },
            now_ms,
        };
        Ok(mounts.handle(inner, &context)?.map(Self::ride_outcome))
    }

    /// W162 (R): a ride ends when the player zones, a fight or kart race
    /// starts, or the player is knocked out. The gateway calls this after every handled message
    /// and tick, once the new state is known.
    pub fn settle_mounts(&mut self, character: &CharacterState, zoned: bool) -> Outcome {
        let restrictions = self.ride_restrictions();
        let Some(mounts) = self.mounts.as_mut() else {
            return Outcome::default();
        };
        if !(zoned || restrictions.in_minigame || restrictions.knocked_out) {
            return Outcome::default();
        }
        let ended = mounts.dismount_request(character.guid, self.inventory.state(), &[]);
        let pose = crate::mounts::RidePose {
            position: character.position,
            heading: 0.0,
        };
        Self::ride_outcome(mounts.wire(&ended, pose))
    }

    /// W177: the persisted pet roster is loaded from the character save before world entry.
    pub fn load_pet_roster(&mut self, path: &Path) -> io::Result<()> {
        self.pets =
            crate::pets::PetService::load_from(crate::pets::PetService::bundled_content()?, path)?;
        Ok(())
    }

    /// Persist the pet roster through the existing top-level envelope seam. This
    /// must run after the ordinary gameplay snapshot so a committed request
    /// cannot silently lose its name or need changes.
    pub fn save_pet_roster(&self, path: &Path) -> io::Result<()> {
        self.pets.save_to(path)
    }

    /// Persist the character snapshot and the pet roster in one atomic write (adoption consumes
    /// an item and adds a pet; both must land or neither).
    pub fn save_with_pet_roster(
        &self,
        path: &Path,
        snapshot: &crate::game_store::Snapshot,
    ) -> io::Result<()> {
        self.pets.save_with(path, snapshot)
    }

    /// W177: the active pet as other players need it for AddNpc replay.
    pub fn pet_view(&self, owner_guid: u64) -> Option<crate::pets::PetPresence> {
        self.pets.view(owner_guid)
    }

    fn pet_outcome(wire: crate::pets::PetWire) -> Outcome {
        let mut out = Outcome::default();
        out.changed = wire.changed;
        out.pets_changed = wire.pets_changed;
        if let Some(notice) = wire.notice {
            out.replies.push(chat::system(notice));
        }
        out.replies.extend(wire.private);
        out.nearby = wire.shared;
        out
    }

    /// W177: summon, dismiss and UI mode (family 53). Ownership comes only from the saved
    /// roster loaded by `load_pet_roster`; the actor GUID is derived from the authenticated
    /// character, never from request bytes.
    fn handle_pets(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let minigame_active = self.blocks(|gates| gates.pets);
        let busy = crate::pets::PetBusy { minigame_active };
        let pets = &mut self.pets;
        let inventory = &mut self.inventory;
        let wire = pets.handle_with_inventory(
            inner,
            character.guid,
            character.position,
            inventory,
            busy,
            // Pet need clocks are stored in seconds; the gateway passes milliseconds.
            now_ms / 1_000,
        )?;
        Ok(wire.map(Self::pet_outcome))
    }

    /// W177 (R): a pet does not zone with its owner; a fresh summon is required.
    pub fn settle_pets(&mut self, zoned: bool, now_ms: u64) -> io::Result<Outcome> {
        if !zoned || self.pets.active_pet_id().is_none() {
            return Ok(Outcome::default());
        }
        let mut pets = self.pets.clone();
        pets.rebase_clock(now_ms / 1_000);
        let transition = pets.dismiss(now_ms / 1_000)?;
        self.pets = pets;
        Ok(Self::pet_outcome(self.pets.wire(transition)))
    }

    /// Quest-gated overworld enemies follow the player's active NPC quests.
    fn sync_field_quests(&mut self, out: &mut Outcome) -> io::Result<()> {
        let Some(quests) = &self.npc_quests else {
            return Ok(());
        };
        let active: BTreeSet<u32> = quests
            .state()
            .quests
            .iter()
            .filter(|(_, progress)| progress.status == crate::quest_engine::QuestStatus::Active)
            .map(|(id, _)| *id)
            .collect();
        if let Some(encounter) = &mut self.encounter {
            let result = encounter.set_active_quests(&active);
            self.apply_encounter(result, out)?;
        }
        Ok(())
    }

    pub fn handle_in_world(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let counters_before = self.achievement_counters();
        let mut out = self.handle_in_world_inner(inner, character, now_ms)?;
        self.sync_tracked_target(character, &mut out);
        self.sync_client_objectives(&mut out);
        self.settle_achievement_counters(&counters_before, &mut out)?;
        Ok(out)
    }

    /// Tell the client to watch the client-reported goals that just opened: 45/6, 45/10 or
    /// 45/11 once per session, 45/9 when a goal opens again (W175, R; rules in
    /// `quest_engine::ClientObjectives`).
    fn sync_client_objectives(&mut self, out: &mut Outcome) {
        if let Some(engine) = &self.npc_quests {
            out.replies.extend(self.client_objectives.update(engine));
        }
    }

    /// Send the "Take Me There" pin (47/14) when the target of the character's own
    /// selected quest changed: after selecting, progress, a zone change or login (R trigger
    /// rules in `quest_journal::Tracker`). No place known means nothing is sent.
    fn sync_tracked_target(&mut self, character: &CharacterState, out: &mut Outcome) {
        let Some(engine) = &self.npc_quests else {
            return;
        };
        let npc = |id: u64| {
            let npcs = self.npc_interaction.as_ref()?;
            npcs.location(id).map(|(_, position)| position)
        };
        let enemy = |kind: u32| self.encounter.as_ref()?.field_enemy_site(kind);
        let poi = |id: u32| engine.poi_location(id);
        let places = crate::quest_journal::Places {
            npc: &npc,
            enemy: &enemy,
            poi: &poi,
        };
        let selected = engine.state().selected_quest_id;
        let open = selected.map_or_else(Vec::new, |id| engine.current_objectives(id));
        let wanted = crate::quest_journal::tracked_pin(selected, &open, character.zone_id, &places);
        out.replies.extend(self.tracked_pin.update(wanted));
    }

    fn handle_in_world_inner(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        if let Some(out) = self.command_refusal(inner, character.guid)? {
            return Ok(out);
        }
        if let Some(out) = self.handle_progression_extras(inner)? {
            return Ok(out);
        }
        if let Some(out) = self.handle_chess(inner, character, now_ms)? {
            return Ok(out);
        }
        if self.chess.as_ref().is_some_and(ChessService::is_active)
            && (inner.starts_with(&[26, 0, 8, 0]) || inner.starts_with(&[26, 0, 10, 0]))
        {
            return Ok(Outcome {
                replies: vec![chat::system("Finish the chess game before interacting.")],
                ..Outcome::default()
            });
        }
        if let Some(out) = self.handle_mounts(inner, character, now_ms)? {
            return Ok(out);
        }
        if let Some(out) = self.handle_soccer(inner, character, now_ms)? {
            return Ok(out);
        }
        if let Some(out) = self.handle_pets(inner, character, now_ms)? {
            return Ok(out);
        }
        // 98/1 "Take Me There" path request (W164). A refusal sends nothing.
        if let Some(answer) = self.pathing.answer(
            inner,
            character.zone_id,
            character.position,
            self.terrain.as_deref(),
            now_ms,
        ) {
            return Ok(Outcome {
                replies: answer.into_iter().collect(),
                ..Outcome::default()
            });
        }
        if self.derby_active() {
            let mut staged = self.clone();
            let out = staged.handle_derby(inner, character, now_ms)?;
            *self = staged;
            return Ok(out);
        }
        if self.kart_entry.is_some() {
            let active = self.kart_entry.as_ref().unwrap().active();
            let command = chat::request(inner, character.guid)?;
            let start = command
                .as_deref()
                .is_some_and(|c| c.trim().eq_ignore_ascii_case("realm race"));
            if active || (start && self.dev_tools) {
                let mut staged = self.clone();
                let out = staged.handle_kart_entry(inner, command.as_deref(), character, now_ms)?;
                *self = staged;
                return Ok(out);
            }
        }
        if !self.fishing.is_empty() {
            if let Some(out) = self.handle_fishing(inner, character, now_ms)? {
                return Ok(out);
            }
        }
        if let Some(encounter) = &self.encounter {
            if let Some(action) = encounter.shared_action(inner, character)? {
                return Ok(Outcome {
                    shared_field: Some(action),
                    ..Default::default()
                });
            }
        }
        // ORIGINAL request: opcode102/sub1 has no payload. This opt-in reply
        // serves a caller-authored catalogue; it does not launch a race or
        // mutate quests, classes, inventory, encounters or character state.
        if self.dev_tools && inner.starts_with(&[102, 0]) {
            if let Some(reply) = &self.kart_lobby_reply {
                if inner.len() < 4 {
                    return Err(invalid("truncated native lobby request"));
                }
                if inner[2..4] == [1, 0] {
                    if inner.len() != 4 {
                        return Err(invalid("native lobby request has trailing bytes"));
                    }
                    return Ok(Outcome {
                        replies: vec![reply.to_vec()],
                        ..Outcome::default()
                    });
                }
            }
        }
        if let Some(outcome) = self.handle_activity_portal(inner, character, now_ms)? {
            return Ok(outcome);
        }
        if let Some(outcome) = self.handle_warp(inner, character, now_ms)? {
            return Ok(outcome);
        }
        if let Some(outcome) = self.handle_gathering_node(inner, character, now_ms)? {
            return Ok(outcome);
        }
        let inventory_before = self.inventory.state().clone();
        if let Some(mut outcome) = self.handle_npc_packet(inner, character, now_ms)? {
            self.npc_inventory_events(&inventory_before, &mut outcome)?;
            return Ok(outcome);
        }
        if self.opening.is_none() {
            return self.handle(inner, character.guid, character.position, now_ms);
        }
        let mut staged = self.clone();
        let before = staged.opening.as_ref().unwrap().journal_entries();
        let hidden_before = staged.hidden_actor_guids();
        let accepted_before: Vec<_> = [910004, 910006, 910013, 910015]
            .into_iter()
            .filter(|id| {
                staged
                    .opening
                    .as_ref()
                    .unwrap()
                    .state()
                    .stages
                    .get(id)
                    .is_some_and(|s| s.accepted)
            })
            .collect();
        let mut out = staged.handle_opening_inner(inner, character, now_ms)?;
        if staged.forging.is_some() {
            for id in [910006, 910015] {
                if !accepted_before.contains(&id)
                    && staged
                        .opening
                        .as_ref()
                        .unwrap()
                        .state()
                        .stages
                        .get(&id)
                        .is_some_and(|s| s.accepted)
                {
                    let mut supplies = staged.forging.as_ref().unwrap().requirements().to_vec();
                    if id == 910015 {
                        // The reinforcement step first requires smelting.
                        // Give its inputs, rather than free bars that would
                        // bypass that native movie. This repeat grant is local.
                        supplies.retain(|(definition, _)| *definition != 21785);
                        supplies.extend(
                            staged
                                .smelting
                                .as_ref()
                                .ok_or_else(|| invalid("reinforcement requires smelting content"))?
                                .requirements()
                                .iter()
                                .copied(),
                        );
                    }
                    staged.grant_reward(0, &supplies, &mut out)?;
                    if id == 910015 {
                        out.replies.push(chat::system("Local reinforcement supplies are reconstructed; the original repeat-quest material grant was not recovered."));
                    }
                }
            }
        }
        if staged.cooking.is_some() {
            for id in [910004, 910013] {
                if !accepted_before.contains(&id)
                    && staged
                        .opening
                        .as_ref()
                        .unwrap()
                        .state()
                        .stages
                        .get(&id)
                        .is_some_and(|s| s.accepted)
                {
                    // Explicit local bounded supplies for each untouched Chef
                    // step. Harvested leaf/fruit are earned through counted goals.
                    staged.grant_reward(0, &[(2862, 4), (2900, 4)], &mut out)?;
                    out.replies.push(chat::system(
                        "Mae supplies Bat Meat and Spice for this recipe.",
                    ));
                }
            }
        }
        for guid in staged.hidden_actor_guids().difference(&hidden_before) {
            // Native PlayerUpdateRemovePlayer: subtype3, one entity, GUID,
            // then the same zero-valued removal fields as the combat oracle.
            let mut remove = vec![35, 0, 3, 0, 1, 0];
            remove.extend(guid.to_le_bytes());
            remove.extend([0; 17]);
            out.replies.push(remove);
        }
        staged.append_journal_updates(&before, &mut out)?;
        *self = staged;
        Ok(out)
    }

    /// W178: title list/select and claim list/apply/definition requests. Claim
    /// rewards are server-owned; request ids only choose queued rows.
    fn handle_progression_extras(&mut self, inner: &[u8]) -> io::Result<Option<Outcome>> {
        if !inner.starts_with(&[108, 0]) && !inner.starts_with(&[152, 0]) {
            return Ok(None);
        }
        let rewards = RewardState::new(
            self.inventory.clone(),
            self.reward_profiles(),
            self.inventory.state().items.len().max(100),
        );
        let mut staged = self.progression_extras.clone();
        let change = staged.handle(inner, &rewards)?;
        if !change.handled {
            return Ok(None);
        }
        let mut out = Outcome {
            replies: change.replies,
            changed: change.changed,
            ..Outcome::default()
        };
        if let Some(next) = change.reward_state {
            let before = self.inventory.state().clone();
            let next_profiles = if self.profiles.is_some()
                && next.profiles != *self.profiles.as_ref().unwrap().state()
            {
                let directory = self
                    .profile_content_directory
                    .as_deref()
                    .ok_or_else(|| invalid("claim XP needs profile content"))?;
                Some(Profiles::load(directory, Some(next.profiles))?)
            } else {
                None
            };
            // Commit the claim ledger before adopting the bag/wallet snapshot.
            self.progression_extras = staged;
            self.inventory = next.inventory;
            if let Some(profiles) = next_profiles {
                self.profiles = Some(profiles);
            }
            self.npc_inventory_events(&before, &mut out)?;
        } else {
            self.progression_extras = staged;
        }
        Ok(Some(out))
    }

    /// W171: hide paid nodes once their run closed, bring back grown-back ones. A change marks
    /// the state changed so the staged clone (which holds the node timers) is kept.
    fn tick_gathering_nodes(&mut self, now_ms: u64, out: &mut Outcome) -> io::Result<()> {
        let busy = self.activity_busy();
        let Some(nodes) = self.gathering_nodes.as_mut() else {
            return Ok(());
        };
        let changes = nodes.sync(now_ms, busy);
        let Some(visibility) = self.npc_visibility.as_mut().filter(|_| !changes.is_empty()) else {
            return Ok(());
        };
        apply_node_changes(visibility, changes);
        out.replies.extend(visibility.refresh()?);
        out.changed = true;
        Ok(())
    }

    /// W171 (R): the ordinary way into mining and harvesting. A click (26/8) on one of this
    /// player's node actors starts the node's Type8 game on a staged copy, so a refusal changes
    /// nothing. Anything that is not a node falls through to the NPC menu path.
    fn handle_gathering_node(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        use crate::harvest_nodes::{Visitor, payout_fits};
        if !inner.starts_with(&[26, 0, 8, 0]) || inner.len() < 12 {
            return Ok(None);
        }
        let target = u64::from_le_bytes(inner[4..12].try_into().unwrap());
        let Some(nodes) = self.gathering_nodes.as_ref().filter(|n| n.is_node(target)) else {
            return Ok(None);
        };
        npc_interaction::decode_npc_interact_26_8(inner)
            .map_err(|_| invalid("invalid node interaction request"))?;
        let mut out = Outcome::default();
        let Some(profiles) = self.profiles.as_ref() else {
            return Ok(None);
        };
        let visitor = Visitor {
            zone: character.zone_id,
            position: character.position,
            busy: self.activity_busy(),
            active_job: profiles.active(),
            unlocked_jobs: &profiles.state().unlocked,
        };
        let gather = match nodes.admit(target, &visitor) {
            Ok(gather) => gather,
            Err(denied) => {
                out.replies.push(chat::system(&denied.message()));
                return Ok(Some(out));
            }
        };
        let Some(engine) = self.type8(gather) else {
            out.replies
                .push(chat::system("Gathering is not available in this campaign."));
            return Ok(Some(out));
        };
        if !payout_fits(&self.inventory, engine.spec()) {
            out.replies.push(chat::system(
                "Your bag is full. Make room before gathering.",
            ));
            return Ok(Some(out));
        }
        let mut staged = self.clone();
        staged.type8_tick_ms = Some(now_ms);
        staged.gathering_nodes.as_mut().unwrap().begin(target);
        let result = staged.type8_mut(gather).unwrap().begin(gather.job())?;
        staged.apply_type8(gather, result, &mut out)?;
        *self = staged;
        Ok(Some(out))
    }

    /// True while a crafting, gathering or battle activity is running.
    fn activity_busy(&self) -> bool {
        self.blocks(|gates| gates.start)
    }

    /// Switches to Kart Driver and enters the track. Shared by the developer `realm race` command
    /// and the ordinary NPC menu choice; the caller has already checked admission and `busy`.
    fn begin_kart_race(&mut self, character: &CharacterState) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        let profiles = self
            .profiles
            .as_mut()
            .ok_or_else(|| invalid("kart entry requires class content"))?;
        let previous_job = profiles.active();
        let outfit_before = self.inventory.profile_attachments(previous_job)?;
        // (R) First use still unlocks Kart Driver for saves that predate trainers; the
        // option is idempotent. W176 also offers the job from a trainer. The prior job is
        // restored on exit.
        let first_time = profiles.unlock(48)?;
        profiles.activate(48)?;
        if first_time {
            self.give_starter_outfit(48, false, &mut out)?;
        }
        self.ensure_outfit(48, &mut out)?;
        // Last, after the outfit repair: the 38/21 attachment list comes from what
        // the player wears now (client reader `0x0091b080`).
        out.replies.insert(
            0,
            self.profiles
                .as_ref()
                .unwrap()
                .activation(48, &self.inventory)?,
        );
        self.announce_profile_outfit(character.guid, &outfit_before, previous_job != 48, &mut out)?;
        let kart = self.kart_entry.as_mut().unwrap();
        let (replies, scene) = kart.begin(character)?;
        kart.set_previous_job(previous_job);
        out.replies.extend(replies);
        out.scene_zoning = Some(scene);
        out.changed = true;
        Ok(out)
    }

    /// Profile state for a reward transaction; a default one without class content.
    fn reward_profiles(&self) -> ProfileState {
        self.profiles
            .as_ref()
            .map(|profiles| profiles.state().clone())
            .unwrap_or_else(|| ProfileState {
                version: 1,
                active: 1,
                unlocked: BTreeSet::from([1]),
                experience: Default::default(),
            })
    }

    /// Snapshot the inventory and profiles for a staged reward transaction.
    fn reward_snapshot(&self) -> RewardState {
        RewardState::new(
            self.inventory.clone(),
            self.reward_profiles(),
            self.inventory.state().items.len().max(100),
        )
    }

    /// Adopt the inventory and profiles from a staged minigame transaction.
    ///
    /// Soccer ticks, fishing catches, and chess results use this path. Derby Close,
    /// progression-extras claims, and quest turn-in deliberately keep their
    /// existing payout implementations because their rollback and commit
    /// ordering differ.
    fn adopt_reward_state(&mut self, next: RewardState, out: &mut Outcome) -> io::Result<()> {
        let before = self.inventory.state().clone();
        if self
            .profiles
            .as_ref()
            .is_some_and(|profiles| *profiles.state() != next.profiles)
        {
            let directory = self
                .profile_content_directory
                .as_deref()
                .ok_or_else(|| invalid("minigame XP needs profile content"))?;
            self.profiles = Some(Profiles::load(directory, Some(next.profiles))?);
        }
        self.inventory = next.inventory;
        out.changed = true;
        self.npc_inventory_events(&before, out)?;
        Ok(())
    }

    /// W160 (R): start the fishing round the chosen guide offers. The caller stages `self`, so a
    /// refusal (a chat line, `changed` false) leaves the player exactly as they were.
    fn begin_fishing(&mut self, gate: u64, character: &CharacterState) -> io::Result<Outcome> {
        use crate::fishing_service::{EntryDenied, FISHERMAN_PROFILE};
        let mut out = Outcome::default();
        let busy = self.activity_busy();
        let Some(spots) = self.fishing_spots.clone() else {
            out.replies
                .push(chat::system(EntryDenied::Unavailable.message()));
            return Ok(out);
        };
        let Some(profiles) = self.profiles.as_mut() else {
            out.replies
                .push(chat::system(EntryDenied::Unavailable.message()));
            return Ok(out);
        };
        // (R) The Fisherman quest line is not built, so the first trip unlocks the job (no
        // outfit change: fishing needs no active job, a quest video shows an Archer fishing).
        let _ = profiles.unlock(FISHERMAN_PROFILE);
        let level = self.job_level(FISHERMAN_PROFILE);
        // (R) Same bag floor as quest turn-ins: never below what the player already holds.
        let rewards = self.reward_snapshot();
        match self.fishing.enter(
            &spots,
            gate,
            character.guid,
            character.zone_id,
            character.position,
            busy,
            level,
            rewards,
        ) {
            Ok(change) => {
                out.replies = change.replies;
                out.changed = true;
            }
            Err(denied) => out.replies.push(chat::system(denied.message())),
        }
        Ok(out)
    }

    /// W160: a packet while a fishing round exists. Hostile or out-of-order frames change
    /// nothing and never end the connection; only a paid catch changes saved state.
    fn handle_fishing(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let profiles = self.reward_profiles();
        let change =
            match self
                .fishing
                .route(character.guid, inner, now_ms, &self.inventory, &profiles)
            {
                Ok(Some(change)) => change,
                Ok(None) => return Ok(None),
                Err(error) => {
                    npc_event(serde_json::json!({
                        "event": "fishing_refused",
                        "reason": error.to_string()
                    }));
                    return Ok(Some(Outcome::default()));
                }
            };
        // Collect is derived from the adopted inventory diff below. The
        // service's own CollectItem is intentionally skipped to avoid counting
        // the catch twice; its minigame completion has no other source.
        let minigame_events: Vec<_> = if change.paid {
            change
                .quest_events
                .iter()
                .filter(|event| {
                    matches!(
                        event,
                        crate::quest_engine::QuestEvent::FinishMinigame { .. }
                    )
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        let mut out = Outcome {
            replies: change.replies,
            ..Outcome::default()
        };
        out.replies
            .extend(change.notices.iter().map(|line| chat::system(line)));
        if change.paid {
            // The service paid on its staged copy of the player's inventory and profiles;
            // adopt it, then let the inventory diff raise the collect event.
            if let Some(next) = self.fishing.reward_state(character.guid).cloned() {
                self.adopt_reward_state(next, &mut out)?;
                // A level-up unlocks fish and refreshes the client's Fish Finder.
                let level = self.job_level(crate::fishing_service::FISHERMAN_PROFILE);
                if let Ok(update) = self.fishing.update_fisherman_level(character.guid, level) {
                    out.replies.extend(update.replies);
                }
            }
        }
        for event in minigame_events {
            self.npc_quest_event(event, &mut out)?;
        }
        Ok(Some(out))
    }

    fn handle_kart_entry(
        &mut self,
        inner: &[u8],
        command: Option<&str>,
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        let active = self.kart_entry.as_ref().unwrap().active();
        if !active {
            if self.activity_busy() {
                out.replies.push(chat::system(
                    crate::kart_gateway::EntryDenied::Busy.message(),
                ));
                return Ok(out);
            }
            return self.begin_kart_race(character);
        }
        let exit = command.is_some_and(|c| c.trim().eq_ignore_ascii_case("realm exit"))
            || inner == [26, 0, 42, 0]
            || self.kart_entry.as_ref().unwrap().accepts_close(inner)?;
        if exit {
            let previous_job = self.kart_entry.as_ref().unwrap().previous_job();
            let return_location = self.kart_entry.as_mut().unwrap().exit().unwrap();
            if let Some(job) = previous_job {
                self.restore_job(job, character.guid, &mut out)?;
            }
            // Captured return frame44446 clears racing bit10 (0x401 ->1).
            out.replies
                .push(crate::kart_gateway::player_status(return_location.guid, 1));
            let world = self
                .encounter
                .as_ref()
                .ok_or_else(|| invalid("kart return requires world content"))?
                .world();
            out.scene_zoning = Some(crate::zone_transition::ZoneTransition {
                envelope_flag: 1,
                scene: world.name.clone(),
                type_id: 2,
                position: [
                    return_location.position[0],
                    return_location.position[1],
                    return_location.position[2],
                    1.,
                ],
                rotation: [
                    return_location.rotation[0],
                    return_location.rotation[1],
                    return_location.rotation[2],
                    1.,
                ],
                sky: world.sky.clone(),
                id: return_location.zone_id as i32,
                geometry_id: world.geometry_id,
                override_update_radius: true,
                unknown: 1,
                ..Default::default()
            });
            out.changed = true;
            return Ok(out);
        }
        if inner.starts_with(&[33, 0]) || inner.starts_with(&[41, 0, 108, 0]) {
            let kart = self.kart_entry.as_mut().unwrap();
            let before = kart.state().clone();
            out.replies = kart.handle(inner, now_ms)?;
            let mut minigame_id = None;
            if let Some(profiles) = &mut self.profiles {
                let claim = kart.claim_reward_with_event(&mut self.inventory, profiles)?;
                minigame_id = claim.minigame_id;
                out.replies.extend(claim.replies);
            }
            out.changed = before != *kart.state();
            self.refresh_npc_quest_eligibility(current_unix_seconds());
            if let Some(minigame_id) = minigame_id {
                self.npc_quest_event(
                    crate::quest_engine::QuestEvent::FinishMinigame { minigame_id },
                    &mut out,
                )?;
            }
            return Ok(out);
        }
        if inner == [102, 0, 1, 0] {
            if let Some(reply) = &self.kart_lobby_reply {
                out.replies.push(reply.to_vec());
            }
            return Ok(out);
        }
        if inner.starts_with(&[26, 0, 13, 0]) || inner.starts_with(&[42, 0]) || command.is_some() {
            out.replies.push(chat::system(
                "Exit the race before changing jobs or starting another activity.",
            ));
        }
        Ok(out)
    }

    /// Developer-only `realm goto`. Returns whether the command was a `goto`.
    /// The relocation itself moves the position and refused states leave every
    /// gameplay counter untouched.
    /// Every actor the developer tools know in the character's zone (opening
    /// actors, visible NPCs, field enemies, battle markers, arena enemies).
    pub fn known_actors(&self, character: &CharacterState) -> Vec<dev_teleport::Target> {
        let mut targets: Vec<_> = self
            .opening
            .as_ref()
            .map(|o| o.actors().iter().map(dev_teleport::Target::from).collect())
            .unwrap_or_default();
        if let Some(visibility) = &self.npc_visibility {
            targets.extend(visibility.teleport_targets(character.zone_id));
        }
        if let Some(encounter) = &self.encounter {
            targets.extend(encounter.field_teleport_targets());
        }
        // (R) Runtime GUID owns identity; keep the opening descriptor first if
        // both indexes include the same actor.
        let mut seen = HashSet::new();
        targets.retain(|target| seen.insert(target.guid) && target.zone_id == character.zone_id);
        targets
    }

    /// Developer cheat console (Studio toolkit). Returns whether `command`
    /// was one of these: place, unplace, placements, mark, unmark, marks,
    /// reload, level, heal, god, give, coins, battle, win, sim, fightlog.
    fn developer_cheat(
        &mut self,
        command: &str,
        raw: &str,
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) -> io::Result<bool> {
        let words: Vec<&str> = command.split_whitespace().collect();
        let raw_words: Vec<&str> = raw.split_whitespace().collect();
        let say = |out: &mut Outcome, text: String| out.replies.push(chat::system(&text));
        match words.as_slice() {
            ["mark", name @ ..] if !name.is_empty() => {
                let name = name.join(" ");
                let mut marks = self.load_marks();
                marks.retain(|m| m.name != name);
                let p = character.position;
                marks.push(Mark {
                    name: name.clone(),
                    zone: character.zone_id,
                    position: p,
                    extra: serde_json::Map::new(),
                });
                marks.sort_by(|a, b| a.name.cmp(&b.name));
                let text = match self.save_marks(marks) {
                    Ok(()) => format!(
                        "Marked {name}: zone {}, x {:.1}, y {:.2}, z {:.1}. realm goto mark {name} returns here.",
                        character.zone_id, p[0], p[1], p[2]
                    ),
                    Err(error) => format!("Could not save the mark: {error}."),
                };
                say(out, text);
            }
            ["unmark", name @ ..] if !name.is_empty() => {
                let name = name.join(" ");
                let mut marks = self.load_marks();
                let before = marks.len();
                marks.retain(|m| m.name != name);
                let text = if marks.len() == before {
                    format!("No mark named {name}.")
                } else {
                    match self.save_marks(marks) {
                        Ok(()) => format!("Removed mark {name}."),
                        Err(error) => format!("Could not save the marks: {error}."),
                    }
                };
                say(out, text);
            }
            ["place", model, rest @ ..] => {
                let raw_rest = raw_words.get(2..).unwrap_or_default();
                let text = self.place_npc(model, rest, raw_rest, character, out)?;
                say(out, text);
            }
            ["unplace", which] => {
                let text = self.unplace_npc(which, character, out)?;
                say(out, text);
            }
            ["placements"] => {
                let p = character.position;
                let mut rows = self.load_placements();
                rows.sort_by(|a, b| {
                    flat_distance(a.position, p).total_cmp(&flat_distance(b.position, p))
                });
                let list: Vec<String> = rows
                    .iter()
                    .take(10)
                    .map(|r| {
                        format!(
                            "{} model {} ({:.0} m)",
                            r.guid,
                            r.model_id,
                            flat_distance(r.position, p)
                        )
                    })
                    .collect();
                let text = if rows.is_empty() {
                    "No placed NPCs; realm place <model id> [name text id] [at x z].".to_owned()
                } else {
                    format!("{} placed NPCs, nearest: {}.", rows.len(), list.join("; "))
                };
                say(out, text);
            }
            ["text", ..] if raw_words.len() > 1 => {
                let literal = raw_words[1..].join(" ");
                let mut pool = crate::world_text::TextPool::load(&self.live_dir())?;
                let text = match pool.add(&literal, None, self.stock_text.as_deref()) {
                    Ok(id) => format!(
                        "Text {id}: \"{literal}\". Use the id in quests and NPC names; players see it after their client restarts with the rebuilt locale."
                    ),
                    Err(error) => format!("Text not saved: {error}."),
                };
                say(out, text);
            }
            ["texts"] => {
                let pool = crate::world_text::TextPool::load(&self.live_dir())?;
                let recent: Vec<String> = pool
                    .entries
                    .iter()
                    .rev()
                    .take(8)
                    .map(|e| format!("{} \"{}\"", e.id, e.text))
                    .collect();
                let text = if recent.is_empty() {
                    "No authored text yet; realm text <words> adds some.".to_owned()
                } else {
                    format!(
                        "{} texts, newest: {}.",
                        pool.entries.len(),
                        recent.join("; ")
                    )
                };
                say(out, text);
            }
            ["marks"] => {
                let marks = self.load_marks();
                let text = if marks.is_empty() {
                    "No marks yet; realm mark <name> saves where you stand.".to_owned()
                } else {
                    let list: Vec<String> = marks
                        .iter()
                        .map(|m| {
                            format!(
                                "{} (zone {}, {:.0}, {:.0})",
                                m.name, m.zone, m.position[0], m.position[2]
                            )
                        })
                        .collect();
                    format!("Marks: {}.", list.join("; "))
                };
                say(out, text);
            }
            ["reload", rest @ ..] => {
                let what = rest.first().copied().unwrap_or("all");
                let text = match self.reload_content(what, out) {
                    Ok(done) => format!("Reloaded: {done}."),
                    Err(error) => format!("Reload failed: {error}. Nothing changed."),
                };
                say(out, text);
            }
            ["level", n] => {
                let Ok(level) = n.parse::<u32>() else {
                    say(out, "Usage: realm level <1-20>".into());
                    return Ok(true);
                };
                let level = level.clamp(1, 20);
                let profile = self.profiles.as_ref().unwrap().active();
                let before = self.job_level(profile);
                let profiles = self.profiles.as_mut().unwrap();
                let previous = profiles
                    .state()
                    .experience
                    .get(&profile)
                    .copied()
                    .unwrap_or(0);
                out.changed |= profiles.set_experience((level - 1) * 100);
                out.replies.extend(profiles.experience_packets(previous));
                if let Some(abilities) = self.abilities.as_ref() {
                    if abilities.unlocks_between(profile, before, level) || level < before {
                        out.replies.extend(abilities.activation(profile, level)?);
                    }
                }
                self.sync_player_level(out);
                self.refresh_npc_quest_eligibility(current_unix_seconds());
                say(out, format!("Job {profile} is now level {level}."));
            }
            ["heal"] => {
                if let Some(encounter) = self.encounter.as_mut() {
                    out.replies.extend(encounter.heal_full());
                }
                say(out, "Health and energy restored.".into());
            }
            ["god"] => {
                let on = self.encounter.as_mut().is_some_and(|e| e.toggle_god());
                say(out, format!("God mode {}.", if on { "on" } else { "off" }));
            }
            ["give", item, rest @ ..] => {
                let (Ok(item), count) = (
                    item.parse::<i32>(),
                    rest.first()
                        .and_then(|n| n.parse::<u32>().ok())
                        .unwrap_or(1),
                ) else {
                    say(out, "Usage: realm give <item id> [count]".into());
                    return Ok(true);
                };
                match self.grant_reward(0, &[(item, count.clamp(1, 999))], out) {
                    Ok(()) => say(out, format!("Gave {count} x item {item}.")),
                    Err(error) => say(out, format!("Could not give item {item}: {error}")),
                }
            }
            ["coins", n] => {
                let Ok(coins) = n.parse::<u32>() else {
                    say(out, "Usage: realm coins <amount>".into());
                    return Ok(true);
                };
                self.grant_reward(coins.min(1_000_000), &[], out)?;
                say(out, format!("Gave {coins} coins."));
            }
            ["battle", id] => {
                let Ok(battle) = id.parse::<u32>() else {
                    say(out, "Usage: realm battle <battle id>".into());
                    return Ok(true);
                };
                match self.begin_field_battle(battle, character, out)? {
                    field_battle::BattleStart::Started => {}
                    field_battle::BattleStart::Unavailable => {
                        say(out, field_battle::UNAVAILABLE_BATTLE_PRIZE.into())
                    }
                    field_battle::BattleStart::Refused(error) => {
                        say(out, format!("Cannot start battle {battle}: {error}"));
                    }
                }
            }
            ["win"] => {
                let result = self.encounter.as_mut().unwrap().debug_win(now_ms)?;
                self.apply_encounter(result, out)?;
            }
            ["sim", rest @ ..] => {
                let battle = rest
                    .first()
                    .and_then(|n| n.parse::<u32>().ok())
                    .unwrap_or(96);
                for line in self.simulate_battle(battle, character)? {
                    say(out, line);
                }
            }
            ["fightlog"] => {
                let log = self
                    .encounter
                    .as_ref()
                    .map(|e| e.fight_log().to_vec())
                    .unwrap_or_default();
                let dealt: i64 = log
                    .iter()
                    .filter(|e| e.source == "you" && matches!(e.kind, "hit" | "crit"))
                    .map(|e| e.amount)
                    .sum();
                let taken: i64 = log
                    .iter()
                    .filter(|e| e.kind == "taken")
                    .map(|e| e.amount)
                    .sum();
                let healed: i64 = log
                    .iter()
                    .filter(|e| e.kind == "heal")
                    .map(|e| e.amount)
                    .sum();
                let crits = log.iter().filter(|e| e.kind == "crit").count();
                say(
                    out,
                    format!(
                        "Fight log: {} events; dealt {dealt} ({crits} crits), taken {taken}, healed {healed}.",
                        log.len()
                    ),
                );
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// Balance simulator: a scripted player of the active job fights `battle`
    /// at several job levels on a copy of the engine (nothing is changed).
    /// The bot stands next to its target, swings on cooldown, uses First Aid
    /// below 60% health and Triage-like strikes when energy allows.
    fn simulate_battle(&self, battle: u32, character: &CharacterState) -> io::Result<Vec<String>> {
        let profile = self.profiles.as_ref().unwrap().active();
        let Some(base) = self.encounter.as_ref() else {
            return Ok(vec!["No encounter engine.".into()]);
        };
        if base.active() {
            return Ok(vec!["Finish or exit the current encounter first.".into()]);
        }
        if !self.battle_prizes_available(battle) {
            return Ok(vec![field_battle::UNAVAILABLE_BATTLE_PRIZE.into()]);
        }
        let mut lines = vec![format!("Battle {battle}, job {profile} (scripted bot):")];
        for level in [1_u32, 5, 10, 15, 20] {
            let mut engine = base.clone();
            // The scripted bot must not be helped by power-up hearts.
            engine.unload_power_ups();
            engine.set_player_job(profile, level);
            // Eligibility is per level; the eligible levels still run through
            // the same production `begin` guard.
            if let Some(required) = engine.battle_level_refusal(battle, level) {
                lines.push(format!(
                    "  level {level:>2}: needs level {required}; skipped"
                ));
                continue;
            }
            if let Err(error) = engine.begin(battle, character.clone(), profile) {
                return Ok(vec![format!("Cannot simulate battle {battle}: {error}")]);
            }
            engine.on_zone_ready(profile, 0)?;
            let abilities = self.abilities.as_ref();
            let slots: Vec<_> = (1..=7)
                .filter_map(|slot| {
                    abilities
                        .and_then(|a| a.job_slot(profile, slot, level))
                        .cloned()
                })
                .collect();
            let mut result = "timeout";
            let mut t = 0_u64;
            let mut lowest = engine.player_health().0;
            while t < 600_000 {
                t += 100;
                let standing = engine.standing_enemies();
                let Some(&(target, at, _)) =
                    standing.iter().min_by_key(|(_, _, shielded)| *shielded)
                else {
                    break;
                };
                let me = [at[0] + 1.5, at[1], at[2]];
                engine.tick(me, profile, t)?;
                let (hp, max) = engine.player_health();
                lowest = lowest.min(hp);
                if hp == 0 {
                    result = "lost";
                    break;
                }
                let heal = slots
                    .iter()
                    .find(|a| a.heal > 0 && a.damage_multiplier == 0.0);
                let strike = slots.iter().find(|a| a.damage_multiplier > 0.0);
                if hp * 10 < max * 6 {
                    if let Some(heal) = heal {
                        engine.job_ability(heal, 1, me, profile, t)?;
                    }
                }
                if let Some(strike) = strike {
                    engine.job_ability(strike, target, me, profile, t)?;
                }
                engine.attack(target, me, profile, 0, t)?;
                if engine
                    .state()
                    .active
                    .as_ref()
                    .is_some_and(|a| a.phase == crate::encounter::EncounterPhase::Won)
                {
                    result = "won";
                    break;
                }
            }
            let log = engine.fight_log();
            let dealt: i64 = log
                .iter()
                .filter(|e| matches!(e.kind, "hit" | "crit"))
                .map(|e| e.amount)
                .sum();
            let crits = log.iter().filter(|e| e.kind == "crit").count();
            let max = engine.player_health().1;
            lines.push(format!(
                "  level {level:>2}: {result} in {:.0}s, lowest health {lowest}/{max}, dealt {dealt} ({crits} crits)",
                t as f64 / 1000.0
            ));
        }
        Ok(lines)
    }

    /// The authored content directory (Studio content editor).
    pub fn content_dir(&self) -> &Path {
        &self.content_dir
    }

    /// Developer fight log of the current or last fight (Studio).
    pub fn fight_log(&self) -> Vec<crate::encounter::FightEvent> {
        self.encounter
            .as_ref()
            .map(|e| e.fight_log().to_vec())
            .unwrap_or_default()
    }

    /// `realm here`: where the character stands and what is near.
    fn developer_here(&self, character: &CharacterState) -> String {
        let p = character.position;
        let mut near: Vec<_> = self
            .known_actors(character)
            .into_iter()
            .map(|t| {
                let d = ((t.position[0] - p[0]).powi(2) + (t.position[2] - p[2]).powi(2)).sqrt();
                (d, t)
            })
            .collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut text = format!(
            "Zone {} at x {:.1}, y {:.2}, z {:.1}.",
            character.zone_id, p[0], p[1], p[2]
        );
        if let Some(ground) = self.ground_height(character.zone_id, p[0], p[2]) {
            text.push_str(&format!(" Ground {ground:.2} ({:+.2} m).", p[1] - ground));
        }
        for (d, t) in near.iter().take(5) {
            text.push_str(&format!(" {} ({:.0} m, {});", t.name, d, t.kind));
        }
        text
    }

    fn developer_teleport(
        &mut self,
        command: &str,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<bool> {
        let argument = command
            .strip_prefix("goto")
            .filter(|rest| rest.starts_with(char::is_whitespace))
            .map(str::trim);
        if let Some(target) = argument.and_then(|a| self.goto_place(a, character)) {
            match target {
                Ok((position, reply)) => {
                    let relocation = dev_teleport::Relocation {
                        position,
                        rotation: character.rotation,
                    };
                    self.relocate(relocation, reply, character, out)?;
                }
                Err(text) => out.replies.push(chat::system(&text)),
            }
            return Ok(true);
        }
        let targets = self.known_actors(character);
        let Some(result) = dev_teleport::handle(command, &targets, character) else {
            return Ok(false);
        };
        match result {
            dev_teleport::Command::Reply(text) => out.replies.push(chat::system(&text)),
            dev_teleport::Command::Relocate { reply, relocation } => {
                self.relocate(relocation, reply, character, out)?;
            }
        }
        Ok(true)
    }

    fn relocate(
        &mut self,
        relocation: dev_teleport::Relocation,
        reply: String,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(reason) = self.relocation_block() {
            out.replies.push(chat::system(&reason));
            return Ok(());
        }
        self.apply_relocation(relocation, character, out)?;
        out.replies.push(chat::system(&reply));
        Ok(())
    }

    /// Moves the staged character inside the zone. The gateway owns the 38/12
    /// packet and the saved position, so bytes and save cannot diverge.
    fn apply_relocation(
        &mut self,
        relocation: dev_teleport::Relocation,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(opening) = self.opening.as_mut() {
            opening.set_location_without_events(character.zone_id, relocation.position)?;
        }
        if let Some(visibility) = &mut self.npc_visibility {
            out.replies
                .extend(visibility.update(character.zone_id, relocation.position)?);
        }
        out.relocation = Some(relocation);
        Ok(())
    }

    fn warp_context(&self, character: &CharacterState) -> piece_077::TravelContext {
        piece_077::TravelContext {
            guid: character.guid,
            zone_id: character.zone_id,
            position: character.position,
            rotation: character.rotation,
            blocked: self.warp_blocker(),
        }
    }

    /// (R) Warpstones never interrupt a fight, a knockout or a running
    /// crafting/gathering movie; those states own the character.
    fn warp_blocker(&self) -> Option<piece_077::Blocker> {
        if self
            .encounter
            .as_ref()
            .is_some_and(|e| e.state().player_health == 0)
        {
            return Some(piece_077::Blocker::KnockedOut);
        }
        if self.encounter.as_ref().is_some_and(EncounterEngine::active) {
            return Some(piece_077::Blocker::Encounter);
        }
        let busy = self.blocks(|gates| gates.warp);
        busy.then_some(piece_077::Blocker::Busy)
    }

    /// W150: the portal list for this player (167/1/1).
    fn append_activity_list(&self, out: &mut Outcome) {
        out.replies
            .push(self.activities.list_packet(self.activity_viewer()));
    }

    fn activity_viewer(&self) -> crate::activity_portal::Viewer {
        let profile = self.profiles.as_ref().map_or(0, Profiles::active);
        crate::activity_portal::Viewer {
            combat_job: self
                .abilities
                .as_ref()
                .is_some_and(|a| a.supports_basic(profile)),
            level: self.job_level(profile),
        }
    }

    /// (R) Begin a portal soccer row: join the authenticated player, fill the
    /// solo roster with passive bots, and offer the shared type-12 run. The
    /// client sees the captured scene entry first; roster and the 39/16 offer
    /// follow after Loaded. Any failure leaves the player untouched.
    fn begin_soccer(
        &mut self,
        activity: u32,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let Some(content) = self.soccer_content.clone() else {
            out.replies
                .push(chat::system("Soccer is not available right now."));
            return Ok(());
        };
        let scene = match content.entry_scene() {
            Ok(scene) => scene,
            Err(_) => {
                out.replies
                    .push(chat::system("Soccer is not available right now."));
                return Ok(());
            }
        };
        let rewards = self.reward_snapshot();
        match crate::soccer_entry::Session::begin(&content, activity, character, rewards) {
            Ok(session) => {
                self.soccer = Some(session);
                out.scene_zoning = Some(scene);
                out.changed = true;
            }
            Err(_) => out
                .replies
                .push(chat::system("You cannot start that activity here.")),
        }
        Ok(())
    }

    /// While a soccer match owns the character, family-76 input and the shared
    /// lifecycle controls are consumed here. Unknown families still pass
    /// through; job/equipment changes are refused with a chat line.
    fn handle_soccer(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if self.soccer.is_none() {
            return Ok(None);
        }
        if inner.starts_with(&[76, 0]) {
            let replies =
                self.soccer
                    .as_mut()
                    .unwrap()
                    .handle_client(character.guid, inner, now_ms)?;
            return Ok(Some(Outcome {
                replies,
                ..Outcome::default()
            }));
        }
        if inner.starts_with(&[39, 0])
            || inner.starts_with(&[41, 0, 109, 0])
            || inner.starts_with(&[26, 0, 42, 0])
        {
            let result =
                self.soccer
                    .as_mut()
                    .unwrap()
                    .handle_lifecycle(character.guid, inner, now_ms)?;
            let mut out = Outcome {
                replies: result.replies,
                ..Outcome::default()
            };
            if matches!(
                result.state,
                Some(crate::minigame::RunState::Cancelled)
                    | Some(crate::minigame::RunState::Closed)
            ) {
                self.exit_soccer(&mut out)?;
            }
            return Ok(Some(out));
        }
        if inner.starts_with(&[26, 0, 13, 0])
            || inner.starts_with(&[42, 0])
            || inner.starts_with(&[36, 0, 10, 0])
        {
            return Ok(Some(Outcome {
                replies: vec![chat::system(
                    "Finish the soccer match before changing jobs or using equipment.",
                )],
                ..Outcome::default()
            }));
        }
        Ok(None)
    }

    /// Return to the pre-match world after a leave or results Close. The world
    /// scene comes from the loaded encounter, so the saved position and the
    /// op31 frame always agree.
    fn exit_soccer(&mut self, out: &mut Outcome) -> io::Result<()> {
        let Some(session) = self.soccer.take() else {
            return Ok(());
        };
        let Some(encounter) = &self.encounter else {
            return Ok(());
        };
        let (zone, position, rotation) = session.return_location();
        let world = encounter.world();
        out.scene_zoning = Some(crate::zone_transition::ZoneTransition {
            envelope_flag: 1,
            scene: world.name.clone(),
            type_id: 2,
            position: [position[0], position[1], position[2], 1.0],
            rotation: [rotation[0], rotation[1], rotation[2], 1.0],
            sky: world.sky.clone(),
            id: i32::try_from(zone).unwrap_or(0),
            geometry_id: world.geometry_id,
            override_update_radius: true,
            unknown: 1,
            ..Default::default()
        });
        out.changed = true;
        Ok(())
    }

    /// W150 (R): 167/1/4 re-sends the list; 167/1/2 Play starts the published battle through the
    /// same `begin()` as the battle start screen's Go. Unknown ids, a busy player, a non-combat
    /// job or a low level get a chat line and change nothing; only the battle itself awards.
    fn handle_activity_portal(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        use crate::activity_portal::Request;
        let Some(request) = crate::activity_portal::parse(inner)? else {
            return Ok(None);
        };
        let mut out = Outcome::default();
        let viewer = self.activity_viewer();
        match request {
            Request::Refresh => {
                // (R) Amplification bound: the client asks on job-level and
                // membership changes, never in a burst.
                if self
                    .portal_refreshed_ms
                    .is_none_or(|last| now_ms >= last.saturating_add(PORTAL_REFRESH_MS))
                {
                    self.portal_refreshed_ms = Some(now_ms);
                    self.append_activity_list(&mut out);
                }
            }
            Request::Play { activity } => {
                // A missing battle engine keeps the old Busy answer for battle
                // rows; chess rows never need it.
                let missing_battle = matches!(
                    self.activities.kind(activity),
                    Some(crate::activity_portal::Kind::Battle)
                ) && self.encounter.is_none();
                let busy = self.warp_blocker().is_some() || missing_battle;
                match self
                    .activities
                    .admit(activity, viewer, busy)
                    .map(|a| (a.id, a.kind))
                {
                    Err(refusal) => out.replies.push(chat::system(&refusal.message())),
                    Ok((_, crate::activity_portal::Kind::Chess { row_id })) => {
                        self.begin_chess(row_id, character.guid, &mut out)?;
                    }
                    Ok((soccer, crate::activity_portal::Kind::Soccer)) => {
                        self.begin_soccer(soccer, character, &mut out)?;
                    }
                    Ok((battle, crate::activity_portal::Kind::Battle)) => {
                        match self.begin_field_battle(battle, character, &mut out)? {
                            field_battle::BattleStart::Started => {}
                            field_battle::BattleStart::Unavailable => out
                                .replies
                                .push(chat::system(field_battle::UNAVAILABLE_BATTLE_PRIZE)),
                            field_battle::BattleStart::Refused(_) => out
                                .replies
                                .push(chat::system("You cannot start that activity here.")),
                        }
                    }
                }
            }
        }
        Ok(Some(out))
    }

    /// (R) Deterministic chess replies for tests and offline fixtures. The
    /// gateway loads tables from OS entropy and never calls this.
    pub fn seed_chess(&mut self, seed: u64) {
        if let Some(chess) = &mut self.chess {
            chess.reseed(seed);
        }
    }

    /// T022: chess movie and lifecycle packets while this character's chess run exists.
    /// The packet is staged on a clone; a refusal or a structural error leaves the table
    /// (and the wallet) unchanged. `None` leaves the packet to the other handlers.
    fn handle_chess(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if !self.chess.as_ref().is_some_and(ChessService::is_active) {
            return Ok(None);
        }
        let mut staged = self.clone();
        let rewards = staged.reward_snapshot();
        let Some(change) =
            staged
                .chess
                .as_mut()
                .unwrap()
                .route(character.guid, inner, now_ms, &rewards)?
        else {
            return Ok(None);
        };
        let mut out = Outcome::default();
        staged.apply_chess(change, &mut out)?;
        *self = staged;
        Ok(Some(out))
    }

    /// T022: a portal Play of a published chess row publishes the 39/16 offer.
    fn begin_chess(&mut self, row_id: u32, player_id: u64, out: &mut Outcome) -> io::Result<()> {
        let Some(chess) = self.chess.as_mut() else {
            out.replies.push(chat::system(
                crate::chess_service::EntryDenied::Unavailable.message(),
            ));
            return Ok(());
        };
        match chess.begin(row_id, player_id) {
            Ok(change) => self.apply_chess(change, out),
            Err(denied) => {
                out.replies.push(chat::system(denied.message()));
                Ok(())
            }
        }
    }

    /// T022: stage a routed chess change. Payout packets follow the result packets,
    /// and the wallet/profile snapshot is adopted before anything is sent.
    fn apply_chess(&mut self, change: ChessActivityOutcome, out: &mut Outcome) -> io::Result<()> {
        let lifecycle = change.replies.clone();
        out.changed |= change.changed;
        out.replies.extend(change.replies);
        if let Some(next) = change.reward_state {
            self.adopt_reward_state(next, out)?;
        }
        out.replies.extend(change.reward_packets);
        // Only the lifecycle result (39/18) advances quests. A checkmate movie
        // message alone is not a payout: the client still has to acknowledge it.
        if lifecycle
            .iter()
            .any(|packet| packet.starts_with(&[39, 0, 18]))
        {
            self.npc_minigame_win(&lifecycle, out)?;
        }
        Ok(())
    }

    /// The client replaces its whole destination table with each 57.
    fn append_warp_definitions(&self, character: &CharacterState, out: &mut Outcome) {
        if character.zone_id == piece_077::OPEN_WORLD_ZONE {
            out.replies.push(self.warp.definitions_message());
        }
    }

    /// 56 destination list, 90 warpstone, 122 safe teleport, 58 friend teleport.
    /// Every request is validated against server state; no position or path
    /// comes from the client. `friend` stays empty until the player registry
    /// supplies an online-friend view, so 58 is refused (R).
    fn handle_warp(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let context = self.warp_context(character);
        let terrain = self.terrain.clone();
        let zone = character.zone_id;
        let ground = move |x: f32, z: f32| terrain.as_ref()?.height_at(zone, x, z);
        let world = piece_077::WarpWorld {
            ground: &ground,
            friend: &|_| None,
        };
        let Some(reply) = self.warp.handle(inner, &context, now_ms, &world) else {
            return Ok(None);
        };
        if matches!(
            reply.rejection,
            Some(piece_077::WarpRejection::MalformedRequest(_))
        ) {
            return Err(invalid("invalid warpstone request"));
        }
        let mut out = Outcome {
            changed: reply.changed,
            replies: reply.replies,
            ..Outcome::default()
        };
        if let Some(target) = reply.relocation {
            self.apply_relocation(
                dev_teleport::Relocation {
                    position: target.position,
                    rotation: target.rotation,
                },
                character,
                &mut out,
            )?;
        }
        out.replies
            .extend(reply.notices.iter().map(|text| chat::system(text)));
        Ok(Some(out))
    }

    /// `goto x z` (onto the ground), `goto x y z` or `goto mark <name>`;
    /// `None` leaves the argument to the actor-name teleport.
    fn goto_place(
        &self,
        argument: &str,
        character: &CharacterState,
    ) -> Option<Result<([f32; 3], String), String>> {
        let zone = character.zone_id;
        if let Some(name) = argument.strip_prefix("mark ").map(str::trim) {
            let marks = self.load_marks();
            return Some(match marks.iter().find(|m| m.name == name) {
                Some(m) if m.zone == zone => {
                    Ok((m.position, format!("Developer teleport: mark {name}.")))
                }
                Some(m) => Err(format!(
                    "Mark {name} is in zone {}; you are in zone {zone}. Nothing changed.",
                    m.zone
                )),
                None => Err(format!("No mark named {name}; realm marks lists them.")),
            });
        }
        let numbers: Vec<f32> = argument
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|w| !w.is_empty())
            .map(|w| w.parse::<f32>().ok().filter(|v| v.is_finite()))
            .collect::<Option<_>>()?;
        Some(match numbers.as_slice() {
            [x, z] => match self.ground_height(zone, *x, *z) {
                // (R) Half a metre up: the client grounds from Y+2 downwards.
                Some(ground) => Ok((
                    [*x, ground + 0.5, *z],
                    format!("Developer teleport: x {x:.1}, z {z:.1}, ground {ground:.2}."),
                )),
                None => Err(format!(
                    "No terrain heights for zone {zone} at x {x:.0}, z {z:.0}; give a height: realm goto x y z."
                )),
            },
            [x, y, z] => Ok((
                [*x, *y, *z],
                format!("Developer teleport: x {x:.1}, y {y:.2}, z {z:.1}."),
            )),
            _ => return None,
        })
    }

    /// Where in-game building is saved: the live world (`--world`), or the
    /// content folder when none is set.
    pub fn live_dir(&self) -> PathBuf {
        self.world_dir
            .clone()
            .unwrap_or_else(|| self.content_dir.clone())
    }

    pub fn set_role(&mut self, role: crate::account_store::AccountRole) {
        self.role = role;
    }

    /// Developer commands need `--dev-tools` or a server-owned role
    /// (account store), plus a role that may run this command (`reload` is keyed with its target, e.g. `reload battles`).
    pub fn may_run(&self, command: &str) -> bool {
        let mut words = command.split_whitespace();
        let first = words.next().unwrap_or_default();
        let key = if first == "reload" {
            format!("reload {}", words.next().unwrap_or("all"))
        } else {
            first.to_owned()
        };
        (self.dev_tools || self.role_commands)
            && crate::account_store::can_run_realm_command(self.role, &key)
    }

    /// (R) Ordinary play remains available to every role. All other realm routes
    /// fail closed, including diagnostics and future developer commands.
    pub(crate) fn developer_command_denied(&self, inner: &[u8], guid: u64) -> io::Result<bool> {
        let Some(text) = chat::request(inner, guid)? else {
            return Ok(false);
        };
        let lower = text.trim().to_ascii_lowercase();
        let Some(command) = lower.strip_prefix("realm ") else {
            return Ok(false);
        };
        let first = command.split_whitespace().next().unwrap_or_default();
        let ordinary = matches!(
            first,
            "help"
                | "status"
                | "quest"
                | "quests"
                | "attack"
                | "respawn"
                | "inventory"
                | "exit"
                | "retry"
                | "accept"
                | "turnin"
        ) || (first == "race" && self.kart_entry.is_none());
        Ok(!ordinary && !self.may_run(command))
    }

    fn command_refusal(&self, inner: &[u8], guid: u64) -> io::Result<Option<Outcome>> {
        if !self.developer_command_denied(inner, guid)? {
            return Ok(None);
        }
        let mut message = if self.role_commands {
            "Cannot do that: your account role does not allow this command."
        } else {
            "Cannot do that: developer command requires --dev-tools and an authorized account role."
        };
        if !self.dev_tools && !self.role_commands {
            // Preserve the existing feature-disabled feedback, while recording
            // the same refusal as other authorization failures (R).
            let text = chat::request(inner, guid)?.unwrap_or_default();
            message = match text.trim().to_ascii_lowercase().as_str() {
                "realm mine" => {
                    "Developer mining is disabled; start the gateway with --dev-tools to test it."
                }
                "realm harvest" => {
                    "Developer harvesting is disabled; start the gateway with --dev-tools to test it."
                }
                _ => message,
            };
        }
        Ok(Some(Outcome {
            dev_command_denied: true,
            replies: vec![chat::system(message)],
            ..Default::default()
        }))
    }

    /// Stock locale row keys (`--client-locale`) for authored text ids.
    pub fn set_stock_text(&mut self, hashes: Option<Arc<HashSet<u32>>>) {
        self.stock_text = hashes;
    }

    /// Use a live-world folder (World Builder layer 2) and load its placed NPCs.
    pub fn set_world_dir(&mut self, dir: Option<PathBuf>) -> io::Result<()> {
        if let Some(dir) = &dir {
            std::fs::create_dir_all(dir)?;
        }
        self.world_dir = dir;
        let records = self.load_placements();
        if let Some(visibility) = self.npc_visibility.as_mut() {
            visibility.replace_placements(&records)?;
        }
        Ok(())
    }

    fn placements_path(&self) -> PathBuf {
        self.live_dir().join("npc_placements.json")
    }

    fn load_placements(&self) -> Vec<crate::world::Placement> {
        std::fs::read(self.placements_path())
            .ok()
            .and_then(|bytes| serde_json::from_slice::<crate::world::PlacementFile>(&bytes).ok())
            .map(|file| file.records)
            .unwrap_or_default()
    }

    fn save_placements(&self, mut records: Vec<crate::world::Placement>) -> io::Result<()> {
        records.sort_by_key(|r| r.guid);
        let file = crate::world::PlacementFile {
            schema_version: 1,
            records,
        };
        let text = serde_json::to_string_pretty(&file).map_err(invalid)?;
        // Atomic (temp + fsync + rename): a crash mid-write must not leave a corrupt file
        // that every later login fails to load.
        crate::character_store::replace_bytes(&self.placements_path(), (text + "\n").as_bytes())
    }

    /// `realm place <model id> [name text id] [at <x> <z>]`: an NPC where you
    /// stand (facing your way), or on the ground at x, z (facing you). It is
    /// saved to `npc_placements.json` and appears at once.
    fn place_npc(
        &mut self,
        model: &str,
        rest: &[&str],
        raw_rest: &[&str],
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<String> {
        const USAGE: &str =
            "Usage: realm place <model id> [name text id] [at <x> <z>] [named <literal name>]";
        let Some(model_id) = model.parse::<u32>().ok().filter(|m| *m > 0) else {
            return Ok(USAGE.into());
        };
        if character.zone_id != crate::world::OPEN_WORLD_ZONE {
            return Ok(
                "Placing NPCs works in the main world (zone 0) for now. Nothing changed.".into(),
            );
        }
        let (mut name_string_id, mut at, mut literal) = (0_u32, None, None);
        let mut words = rest.iter().enumerate();
        while let Some((index, word)) = words.next() {
            if *word == "named" {
                let name = raw_rest.get(index + 1..).unwrap_or_default().join(" ");
                if name.is_empty() || name.len() > 64 {
                    return Ok(USAGE.into());
                }
                literal = Some(name);
                break;
            }
            if *word == "at" {
                let mut number = || {
                    words
                        .next()
                        .and_then(|(_, w)| w.trim_end_matches(',').parse::<f32>().ok())
                };
                match (number(), number()) {
                    (Some(x), Some(z)) if x.is_finite() && z.is_finite() => at = Some((x, z)),
                    _ => return Ok(USAGE.into()),
                }
            } else if let Ok(id) = word.parse::<u32>() {
                name_string_id = id;
            } else {
                return Ok(USAGE.into());
            }
        }
        let me = character.position;
        let (position, heading) = match at {
            Some((x, z)) => {
                let Some(ground) = self.ground_height(character.zone_id, x, z) else {
                    return Ok(format!(
                        "No terrain heights at x {x:.0}, z {z:.0}; stand there and use realm place without 'at'."
                    ));
                };
                ([x, ground, z], (me[0] - x).atan2(me[2] - z))
            }
            // Where you stand, on the ground under you: teleports leave you
            // 0.5 m up. Ground far below means a bridge or roof; keep yours.
            None => {
                let mut at = me;
                if let Some(ground) = self
                    .ground_height(character.zone_id, me[0], me[2])
                    .filter(|g| (g - me[1]).abs() <= 6.0)
                {
                    at[1] = ground;
                }
                (at, character.rotation[0].atan2(character.rotation[2]))
            }
        };
        let mut records = self.load_placements();
        let guid = records
            .iter()
            .map(|r| r.guid)
            .max()
            .unwrap_or(crate::world::PLACEMENT_GUID_BASE)
            .max(crate::world::PLACEMENT_GUID_BASE)
            + 1;
        let placement = crate::world::Placement {
            guid,
            model_id,
            name_string_id,
            name: literal,
            position,
            heading,
            scale: 1.0,
            extra: serde_json::Map::from_iter([(
                "source".to_owned(),
                serde_json::json!([{"kind": "reconstructed", "evidence": "(R) placed with the Studio placer"}]),
            )]),
        };
        records.push(placement.clone());
        if let Err(error) = self.save_placements(records) {
            return Ok(format!(
                "Could not save the placement: {error}. Nothing changed."
            ));
        }
        if let Some(visibility) = &mut self.npc_visibility {
            visibility.place(&placement);
            out.replies
                .extend(visibility.update(character.zone_id, character.position)?);
        }
        Ok(format!(
            "Placed NPC {guid}: model {model_id}, name {name_string_id}, at x {:.1}, y {:.2}, z {:.1}. realm unplace {guid} removes it.",
            position[0], position[1], position[2]
        ))
    }

    /// `realm unplace <guid|nearest>`: delete a placed NPC.
    fn unplace_npc(
        &mut self,
        which: &str,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<String> {
        let mut records = self.load_placements();
        let me = character.position;
        let guid = if which == "nearest" {
            records
                .iter()
                .min_by(|a, b| {
                    flat_distance(a.position, me).total_cmp(&flat_distance(b.position, me))
                })
                .map(|r| r.guid)
        } else {
            which.parse::<u64>().ok()
        };
        let Some(guid) = guid.filter(|g| records.iter().any(|r| r.guid == *g)) else {
            return Ok(format!(
                "No placed NPC {which}; realm placements lists them."
            ));
        };
        records.retain(|r| r.guid != guid);
        if let Err(error) = self.save_placements(records) {
            return Ok(format!(
                "Could not save the placements: {error}. Nothing changed."
            ));
        }
        if let Some(visibility) = &mut self.npc_visibility {
            visibility.remove(guid);
            out.replies
                .extend(visibility.update(character.zone_id, character.position)?);
        }
        Ok(format!("Removed placed NPC {guid}."))
    }

    fn marks_path(&self) -> PathBuf {
        self.live_dir().join("marks.json")
    }

    /// Named developer waypoints (`marks.json` beside the NPC content).
    fn load_marks(&self) -> Vec<Mark> {
        std::fs::read(self.marks_path())
            .ok()
            .and_then(|bytes| serde_json::from_slice::<MarkFile>(&bytes).ok())
            .map(|file| file.marks)
            .unwrap_or_default()
    }

    fn save_marks(&self, marks: Vec<Mark>) -> io::Result<()> {
        let file = MarkFile {
            schema_version: 1,
            marks,
        };
        let text = serde_json::to_string_pretty(&file).map_err(invalid)?;
        crate::character_store::replace_bytes(&self.marks_path(), (text + "\n").as_bytes())
    }

    /// (R) Diagnostic only: use the native 41/2 clear byte to test the client
    /// tutorial guard. The client also starts its current minigame state and
    /// disposes the tutorial manager, so refuse this while an encounter runs.
    fn developer_npx_off(&self, command: &str, out: &mut Outcome) -> bool {
        if command != "npx off" {
            return false;
        }
        if !self.dev_tools {
            out.replies.push(chat::system(
                "The NPX diagnostic is disabled; start the gateway with --dev-tools.",
            ));
            return true;
        }
        if self.encounter.as_ref().is_some_and(EncounterEngine::active) {
            out.replies.push(chat::system(
                "Run realm npx off outside an encounter; the client starts encounter state for 41/2.",
            ));
            return true;
        }

        // ORIGINAL wire shape: u16 family 41, u16 subtype 2, two i32 header
        // values, u64 player value, trailing byte. (R) Zero identity fields
        // keep this diagnostic detached from a live encounter roster.
        let mut packet = Vec::with_capacity(21);
        packet.extend_from_slice(&[41, 0, 2, 0]);
        packet.extend_from_slice(&0_i32.to_le_bytes());
        packet.extend_from_slice(&0_i32.to_le_bytes());
        packet.extend_from_slice(&0_u64.to_le_bytes());
        packet.push(0);
        out.replies.push(packet);
        true
    }

    /// Developer-only wire experiment. It sends 49/1 packets without changing quest state.
    fn developer_quest_offer(
        &mut self,
        command: &str,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<bool> {
        if command != "offer" && !command.starts_with("offer ") {
            return Ok(false);
        }
        if !self.dev_tools {
            out.replies.push(chat::system(
                "Quest offer experiments are disabled; start the gateway with --dev-tools.",
            ));
            return Ok(true);
        }
        let mut args = command.split_whitespace();
        let _ = args.next();
        let Some(quest_id) = args.next().and_then(|value| value.parse::<u32>().ok()) else {
            out.replies.push(chat::system(
                "Usage: realm offer <quest_id> v0|v1|v2|v3|v4|v5|v6.",
            ));
            return Ok(true);
        };
        let Some(variant) = args.next() else {
            out.replies.push(chat::system(
                "Usage: realm offer <quest_id> v0|v1|v2|v3|v4|v5|v6.",
            ));
            return Ok(true);
        };
        if quest_id == 0
            || args.next().is_some()
            || !matches!(variant, "v0" | "v1" | "v2" | "v3" | "v4" | "v5" | "v6")
        {
            out.replies.push(chat::system(
                "Usage: realm offer <quest_id> v0|v1|v2|v3|v4|v5|v6.",
            ));
            return Ok(true);
        }

        let targets = self
            .npc_visibility
            .as_ref()
            .map(|visibility| visibility.teleport_targets(character.zone_id))
            .unwrap_or_default();
        let distance_squared = |target: &dev_teleport::Target| {
            target
                .position
                .iter()
                .zip(character.position)
                .map(|(target, player)| (target - player).powi(2))
                .sum::<f32>()
        };
        // (R) Keep the diagnostic target inside the existing reconstructed 12 m NPC range.
        let current =
            self.last_npc_dialogue
                .as_ref()
                .and_then(|(zone_id, runtime_guid, content_key, _)| {
                    (*zone_id == character.zone_id)
                        .then(|| {
                            targets.iter().find(|target| {
                                target.guid == *runtime_guid
                                    && self.content_npc_key(target.guid) == Some(*content_key)
                                    && distance_squared(target) <= 144.0
                            })
                        })
                        .flatten()
                        .map(|target| (target, *content_key))
                });
        let nearest = || {
            targets
                .iter()
                .filter(|target| target.kind == "npc")
                .filter_map(|target| {
                    let content_key = self.content_npc_key(target.guid)?;
                    let distance = distance_squared(target);
                    (distance.is_finite() && distance <= 144.0).then_some((
                        target,
                        content_key,
                        distance,
                    ))
                })
                .min_by(|left, right| left.2.total_cmp(&right.2))
                .map(|(target, content_key, _)| (target, content_key))
        };
        let Some((target, content_key)) = current.or_else(nearest) else {
            out.replies.push(chat::system(
                "No nearby content NPC is available for the quest offer experiment.",
            ));
            return Ok(true);
        };
        let Some(engine) = self.npc_quests.as_ref() else {
            out.replies.push(chat::system(
                "Quest data is unavailable for the offer experiment.",
            ));
            return Ok(true);
        };

        if !engine.is_enabled(quest_id) {
            out.replies
                .push(chat::system("This quest is disabled or has no objectives."));
            return Ok(true);
        }
        let normal_intro = engine.intro_text_string_id(quest_id).unwrap_or(0);
        let title_id = engine.title_string_id(quest_id).unwrap_or(0);
        let (send_dialogue, blank_dialogue, offers): (bool, bool, Vec<(u16, Option<u32>)>) =
            match variant {
                "v0" => (true, false, vec![(0, None)]),
                "v1" => (false, false, vec![(0, None)]),
                "v2" => (false, false, vec![(0, Some(0))]),
                "v3" => (false, false, vec![(0, Some(title_id))]),
                "v4" => (true, true, vec![(0, None)]),
                "v5" => (false, false, vec![(1, None)]),
                // Prime UIProcessor+0x1c with intro 0, then send the original intro.
                "v6" => (false, false, vec![(0, Some(0)), (0, None)]),
                _ => unreachable!("variant was validated above"),
            };

        let dialogue = if send_dialogue {
            let cached_dialogue = if blank_dialogue {
                None
            } else {
                self.last_npc_dialogue
                    .as_ref()
                    .filter(|(zone, guid, key, _)| {
                        *zone == character.zone_id && *guid == target.guid && *key == content_key
                    })
                    .map(|(_, _, _, packet)| packet.clone())
            };
            Some(match cached_dialogue {
                Some(packet) => packet,
                None => dialogue_packet(&DialogueContext {
                    body_string_id: None,
                    quest_title_string_ids: Vec::new(),
                })
                .map_err(|_| invalid("NPC dialogue could not be encoded"))?,
            })
        } else {
            None
        };
        if let Some(dialogue) = dialogue {
            out.replies.push(dialogue);
        }

        let prefixes: Vec<u16> = offers.iter().map(|(prefix, _)| *prefix).collect();
        let intros: Vec<u32> = offers
            .iter()
            .map(|(_, intro)| intro.unwrap_or(normal_intro))
            .collect();
        for (_prefix, intro) in &offers {
            out.replies.push(
                // The u16 "prefix" variants (W72 v5) are gone: the client header
                // has no such field (native run 142216).
                quest_offer_packet_with(engine, quest_id, *intro)
                    .map_err(|_| invalid("NPC quest offer could not be encoded"))?,
            );
        }
        let mut packet_types = Vec::with_capacity(offers.len() + usize::from(send_dialogue));
        if send_dialogue {
            packet_types.push("26/3");
        }
        for _ in &offers {
            packet_types.push("49/1");
        }
        // Accepting a developer offer goes through the normal 49/2 path.
        if let Some(engine) = self.npc_quests.as_mut() {
            engine.dev_mark_offered(quest_id);
        }
        npc_event(serde_json::json!({
            "event": "offer_variant",
            "variant": variant,
            "quest_id": quest_id,
            "npc_guid": target.guid,
            "npc_content_id": content_key,
            "packet_types": packet_types,
            "prefix_u16": prefixes,
            "intro_string_ids": intros
        }));
        out.replies.push(chat::system(&format!(
            "Sent quest {quest_id} offer variant {variant} to {}.",
            target.name
        )));
        Ok(true)
    }

    /// Developer-only `realm mine`: starts the reconstructed Mining18 board for
    /// the accepted Miner job. The launcher must have enabled dev tools, the
    /// Miner job must be unlocked and active, and no other live activity may
    /// compete. Reward authority stays server-side in `apply_type8`.
    fn developer_mining(&mut self, now_ms: u64, out: &mut Outcome) -> io::Result<()> {
        let Some(engine) = self.type8(Gather::Mining) else {
            out.replies.push(chat::system(
                "Mining content is not loaded in this campaign.",
            ));
            return Ok(());
        };
        if engine.active() {
            out.replies.push(chat::system(
                "A mining test is already running. Type realm exit to cancel it.",
            ));
            return Ok(());
        }
        if self.forging.as_ref().is_some_and(Forging::active)
            || self.smelting.as_ref().is_some_and(Smelting::active)
            || self.cooking.as_ref().is_some_and(Cooking::active)
            || self.encounter.as_ref().is_some_and(EncounterEngine::active)
        {
            out.replies.push(chat::system(
                "Finish or exit the active activity before starting the mining test.",
            ));
            return Ok(());
        }
        let (active, unlocked) = {
            let profiles = self.profiles.as_ref().unwrap();
            (
                profiles.active(),
                profiles
                    .state()
                    .unlocked
                    .contains(&mining::MINER_PROFILE_ID),
            )
        };
        if !unlocked {
            out.replies
                .push(chat::system("The Miner job is not unlocked in this save."));
            return Ok(());
        }
        if active != mining::MINER_PROFILE_ID {
            out.replies.push(chat::system(
                "Switch to the Miner job before starting the mining test.",
            ));
            return Ok(());
        }
        let result = self
            .type8_mut(Gather::Mining)
            .unwrap()
            .begin(mining::MINER_PROFILE_ID)?;
        self.type8_tick_ms = Some(now_ms);
        self.apply_type8(Gather::Mining, result, out)?;
        out.replies.push(chat::system(
            "Mining test started; type realm exit to cancel.",
        ));
        Ok(())
    }

    /// Developer-only `realm harvest`: starts the reconstructed Harvesting111
    /// candidate for the accepted Chef job. Dev tools must be enabled, Chef
    /// must be unlocked and active, and no other live activity may compete.
    /// Reward authority stays server-side in `apply_type8`.
    fn developer_harvesting(&mut self, now_ms: u64, out: &mut Outcome) -> io::Result<()> {
        let Some(engine) = self.type8(Gather::Harvesting) else {
            out.replies.push(chat::system(
                "Harvesting content is not loaded in this campaign.",
            ));
            return Ok(());
        };
        if engine.active() {
            out.replies.push(chat::system(
                "A harvesting test is already running. Type realm exit to cancel it.",
            ));
            return Ok(());
        }
        if self.forging.as_ref().is_some_and(Forging::active)
            || self.smelting.as_ref().is_some_and(Smelting::active)
            || self.cooking.as_ref().is_some_and(Cooking::active)
            || self.type8(Gather::Mining).is_some_and(Type8Engine::active)
            || self.encounter.as_ref().is_some_and(EncounterEngine::active)
        {
            out.replies.push(chat::system(
                "Finish or exit the active activity before starting the harvesting test.",
            ));
            return Ok(());
        }
        let (active, unlocked) = {
            let profiles = self.profiles.as_ref().unwrap();
            (
                profiles.active(),
                profiles
                    .state()
                    .unlocked
                    .contains(&harvesting::CHEF_PROFILE_ID),
            )
        };
        if !unlocked {
            out.replies
                .push(chat::system("The Chef job is not unlocked in this save."));
            return Ok(());
        }
        if active != harvesting::CHEF_PROFILE_ID {
            out.replies.push(chat::system(
                "Switch to the Chef job before starting the harvesting test.",
            ));
            return Ok(());
        }
        let result = self
            .type8_mut(Gather::Harvesting)
            .unwrap()
            .begin(harvesting::CHEF_PROFILE_ID)?;
        self.type8_tick_ms = Some(now_ms);
        self.apply_type8(Gather::Harvesting, result, out)?;
        out.replies.push(chat::system(
            "Harvesting test started; type realm exit to cancel.",
        ));
        Ok(())
    }

    /// States that must not be relocated out of. Crafting movies are refused
    /// by their own earlier gates; this covers live combat and knockout.
    fn relocation_block(&self) -> Option<String> {
        let encounter = self.encounter.as_ref()?;
        if encounter.state().player_health == 0 {
            return Some(
                "Developer teleport is refused while the character is knocked out.".into(),
            );
        }
        // Inside a battle arena the only targets are that arena's enemies, so
        // a relocation stays in the instance.
        if encounter.active() && !encounter.in_battle_arena() {
            return Some(
                "Developer teleport is refused during an encounter; finish or exit first.".into(),
            );
        }
        None
    }

    fn append_journal_updates(
        &self,
        before: &[crate::opening::OpeningJournalEntry],
        out: &mut Outcome,
    ) -> io::Result<()> {
        out.replies.extend(
            self.journal
                .as_ref()
                .unwrap()
                .updates(before, &self.opening.as_ref().unwrap().journal_entries())?,
        );
        Ok(())
    }

    fn handle_opening_inner(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        if self.forging.as_ref().is_some_and(Forging::active) {
            if inner.starts_with(&[26, 0, 42, 0]) {
                if inner.len() != 4 {
                    return Err(invalid("invalid forging leave request"));
                }
                let result = self.forging.as_mut().unwrap().cancel();
                self.apply_craft(CraftingJob::Blacksmith, result, character, &mut out)?;
                self.forging_tick_ms = None;
                return Ok(out);
            }
            if inner.starts_with(&[39, 0]) {
                let result = self.forging.as_mut().unwrap().handle(inner)?;
                self.apply_craft(CraftingJob::Blacksmith, result, character, &mut out)?;
                return Ok(out);
            }
            if inner.starts_with(&[26, 0, 13, 0])
                || inner.starts_with(&[26, 0, 8, 0])
                || inner.starts_with(&[26, 0, 10, 0])
                || inner.starts_with(&[36, 0, 10, 0])
                || inner.starts_with(&[42, 0])
            {
                out.replies.push(chat::system(
                    "Close the forging game before changing jobs, equipment or interacting.",
                ));
                return Ok(out);
            }
            if let Some(text) = chat::request(inner, character.guid)? {
                if text.trim().eq_ignore_ascii_case("realm exit") {
                    let result = self.forging.as_mut().unwrap().cancel();
                    self.apply_craft(CraftingJob::Blacksmith, result, character, &mut out)?;
                    self.forging_tick_ms = None;
                } else if text.trim().to_ascii_lowercase().starts_with("realm ") {
                    out.replies.push(chat::system(
                        "Close the forging game before other opening actions.",
                    ));
                }
                return Ok(out);
            }
        }
        if self.cooking.as_ref().is_some_and(Cooking::active) {
            if inner.starts_with(&[26, 0, 42, 0]) {
                if inner.len() != 4 {
                    return Err(invalid("invalid cooking leave request"));
                }
                let result = self.cooking.as_mut().unwrap().cancel();
                self.apply_craft(CraftingJob::Chef, result, character, &mut out)?;
                return Ok(out);
            }
            if inner.starts_with(&[39, 0]) {
                let result = self.cooking.as_mut().unwrap().handle(inner)?;
                self.apply_craft(CraftingJob::Chef, result, character, &mut out)?;
                return Ok(out);
            }
            if inner.starts_with(&[26, 0, 13, 0])
                || inner.starts_with(&[26, 0, 8, 0])
                || inner.starts_with(&[26, 0, 10, 0])
                || inner.starts_with(&[36, 0, 10, 0])
                || inner.starts_with(&[42, 0])
            {
                out.replies.push(chat::system(
                    "Close the cooking game before changing jobs, equipment or interacting.",
                ));
                return Ok(out);
            }
            if let Some(text) = chat::request(inner, character.guid)? {
                if text.trim().eq_ignore_ascii_case("realm exit") {
                    let result = self.cooking.as_mut().unwrap().cancel();
                    self.apply_craft(CraftingJob::Chef, result, character, &mut out)?;
                } else if text.trim().to_ascii_lowercase().starts_with("realm ") {
                    out.replies.push(chat::system(
                        "Close the cooking game before other opening actions.",
                    ));
                }
                return Ok(out);
            }
        }
        if self.smelting.as_ref().is_some_and(Smelting::active) {
            if inner.starts_with(&[26, 0, 42, 0]) {
                if inner.len() != 4 {
                    return Err(invalid("invalid smelting leave request"));
                }
                let result = self.smelting.as_mut().unwrap().cancel();
                self.apply_craft(CraftingJob::Miner, result, character, &mut out)?;
                return Ok(out);
            }
            if inner.starts_with(&[39, 0]) {
                let result = self.smelting.as_mut().unwrap().handle(inner)?;
                self.apply_craft(CraftingJob::Miner, result, character, &mut out)?;
                return Ok(out);
            }
            if inner.starts_with(&[26, 0, 13, 0])
                || inner.starts_with(&[26, 0, 8, 0])
                || inner.starts_with(&[26, 0, 10, 0])
                || inner.starts_with(&[36, 0, 10, 0])
                || inner.starts_with(&[42, 0])
            {
                out.replies.push(chat::system(
                    "Close the smelting game before changing jobs, equipment or interacting.",
                ));
                return Ok(out);
            }
            if let Some(text) = chat::request(inner, character.guid)? {
                if text.trim().eq_ignore_ascii_case("realm exit") {
                    let result = self.smelting.as_mut().unwrap().cancel();
                    self.apply_craft(CraftingJob::Miner, result, character, &mut out)?;
                } else if text.trim().to_ascii_lowercase().starts_with("realm ") {
                    out.replies.push(chat::system(
                        "Close the smelting game before other opening actions.",
                    ));
                }
                return Ok(out);
            }
        }
        if self.type8(Gather::Mining).is_some_and(Type8Engine::active) {
            if inner.starts_with(&[26, 0, 42, 0]) {
                if inner.len() != 4 {
                    return Err(invalid("invalid mining leave request"));
                }
                let result = self.type8_mut(Gather::Mining).unwrap().cancel();
                self.apply_type8(Gather::Mining, result, &mut out)?;
                self.type8_tick_ms = None;
                return Ok(out);
            }
            if inner.starts_with(&[39, 0]) {
                let result = self.type8_mut(Gather::Mining).unwrap().handle(inner)?;
                self.apply_type8(Gather::Mining, result, &mut out)?;
                return Ok(out);
            }
            if inner.starts_with(&[26, 0, 13, 0])
                || inner.starts_with(&[26, 0, 8, 0])
                || inner.starts_with(&[26, 0, 10, 0])
                || inner.starts_with(&[36, 0, 10, 0])
                || inner.starts_with(&[42, 0])
            {
                out.replies.push(chat::system(
                    "Close the mining game before changing jobs, equipment or interacting.",
                ));
                return Ok(out);
            }
            if let Some(text) = chat::request(inner, character.guid)? {
                if text.trim().eq_ignore_ascii_case("realm exit") {
                    let result = self.type8_mut(Gather::Mining).unwrap().cancel();
                    self.apply_type8(Gather::Mining, result, &mut out)?;
                    self.type8_tick_ms = None;
                } else if text.trim().to_ascii_lowercase().starts_with("realm ") {
                    out.replies.push(chat::system(
                        "Close the mining game before other opening actions.",
                    ));
                }
                return Ok(out);
            }
        }
        if self
            .type8(Gather::Harvesting)
            .is_some_and(Type8Engine::active)
        {
            if inner.starts_with(&[26, 0, 42, 0]) {
                if inner.len() != 4 {
                    return Err(invalid("invalid harvesting leave request"));
                }
                let result = self.type8_mut(Gather::Harvesting).unwrap().cancel();
                self.apply_type8(Gather::Harvesting, result, &mut out)?;
                self.type8_tick_ms = None;
                return Ok(out);
            }
            if inner.starts_with(&[39, 0]) {
                let result = self.type8_mut(Gather::Harvesting).unwrap().handle(inner)?;
                self.apply_type8(Gather::Harvesting, result, &mut out)?;
                return Ok(out);
            }
            if inner.starts_with(&[26, 0, 13, 0])
                || inner.starts_with(&[26, 0, 8, 0])
                || inner.starts_with(&[26, 0, 10, 0])
                || inner.starts_with(&[36, 0, 10, 0])
                || inner.starts_with(&[42, 0])
            {
                out.replies.push(chat::system(
                    "Close the harvesting game before changing jobs, equipment or interacting.",
                ));
                return Ok(out);
            }
            if let Some(text) = chat::request(inner, character.guid)? {
                if text.trim().eq_ignore_ascii_case("realm exit") {
                    let result = self.type8_mut(Gather::Harvesting).unwrap().cancel();
                    self.apply_type8(Gather::Harvesting, result, &mut out)?;
                    self.type8_tick_ms = None;
                } else if text.trim().to_ascii_lowercase().starts_with("realm ") {
                    out.replies.push(chat::system(
                        "Close the harvesting game before other opening actions.",
                    ));
                }
                return Ok(out);
            }
        }
        if let Some(text) = chat::request(inner, character.guid)? {
            if let Some(command) = text.trim().to_ascii_lowercase().strip_prefix("realm ") {
                let tool = self.may_run(command);
                if tool && self.developer_npx_off(command, &mut out) {
                    return Ok(out);
                }
                if tool && self.developer_quest_offer(command, character, &mut out)? {
                    return Ok(out);
                }
                if tool && self.developer_teleport(command, character, &mut out)? {
                    return Ok(out);
                }
                if tool && command == "here" {
                    out.replies
                        .push(chat::system(&self.developer_here(character)));
                    return Ok(out);
                }
                // Same byte offsets as `command` (ASCII lowercasing keeps lengths):
                // literal names keep their case.
                let raw = &text.trim()["realm ".len()..];
                if tool && self.developer_cheat(command, raw, character, now_ms, &mut out)? {
                    return Ok(out);
                }
                match command {
                    "exit" => {
                        let result = self.encounter.as_mut().unwrap().exit()?;
                        self.apply_encounter(result, &mut out)?;
                    }
                    "retry" => {
                        let result = self
                            .encounter
                            .as_mut()
                            .unwrap()
                            .retry(self.profiles.as_ref().unwrap().active(), now_ms)?;
                        self.apply_encounter(result, &mut out)?;
                    }
                    "mine" => {
                        if !self.dev_tools {
                            out.replies.push(chat::system(
                                "Developer mining is disabled; start the gateway with --dev-tools to test it.",
                            ));
                        } else {
                            self.developer_mining(now_ms, &mut out)?;
                        }
                    }
                    "harvest" => {
                        if !self.dev_tools {
                            out.replies.push(chat::system(
                                "Developer harvesting is disabled; start the gateway with --dev-tools to test it.",
                            ));
                        } else {
                            self.developer_harvesting(now_ms, &mut out)?;
                        }
                    }
                    _ => {
                        let result = self.opening.as_mut().unwrap().handle_command(command);
                        self.apply_opening(result, &mut out)?;
                    }
                }
            }
            return Ok(out);
        }
        if inner.starts_with(&[26, 0, 13, 0]) {
            if inner.len() != 8 {
                return Err(invalid("invalid class selection request"));
            }
            let id = i32::from_le_bytes(inner[4..8].try_into().unwrap());
            let outfit_before = self
                .inventory
                .profile_attachments(self.profiles.as_ref().unwrap().active())?;
            if let Some(changed) = self.stage_job_selection(id, &mut out)? {
                out.replies.extend(
                    self.abilities
                        .as_ref()
                        .unwrap()
                        .activation(id, self.job_level(id))?,
                );
                self.sync_player_level(&mut out);
                self.opening_equipment_events(&mut out)?;
                self.sync_world_stage(&mut out)?;
                self.announce_profile_outfit(character.guid, &outfit_before, changed, &mut out)?;
            }
            return Ok(out);
        }
        // Recipe-book start (client 26/24): recipe_book.rs decodes it and checks
        // learned recipe, active job, idle services, a server-known station in
        // range, rank and exact inputs before the service's own `begin_recipe`.
        if let Some(recipe_id) = crate::recipe_book::decode(inner)? {
            // `recipe_book::start` owns the busy refusal, so there is no second
            // check here that a canary could bypass or duplicate. A pack with no
            // class content (supported by `load`) has no recipe-book player at
            // all, so refuse before building one.
            let Some(profile) = self.profiles.as_ref().map(Profiles::active) else {
                out.replies
                    .push(crate::recipe_book::Refusal::Unavailable.feedback());
                return Ok(out);
            };
            let available = self.available_ingredients();
            let player = crate::recipe_book::Player {
                zone_id: character.zone_id,
                position: character.position,
                active_profile: profile,
                job_rank: self.recipe_rank(profile),
                learned: &self.learned_recipes,
                available: &available,
                in_activity: self.activity_busy(),
            };
            let crafting_stations = self.crafting_stations();
            let started = crate::recipe_book::start(
                recipe_id,
                &self.recipe_catalog,
                &player,
                &crafting_stations,
                crate::recipe_book::Services {
                    cooking: self.cooking.as_mut(),
                    smelting: self.smelting.as_mut(),
                    forging: self.forging.as_mut(),
                },
            );
            match started {
                Ok(crate::recipe_book::Started::Cooking(result)) => {
                    self.apply_craft(CraftingJob::Chef, result, character, &mut out)?
                }
                Ok(crate::recipe_book::Started::Smelting(result)) => {
                    self.apply_craft(CraftingJob::Miner, result, character, &mut out)?
                }
                Ok(crate::recipe_book::Started::Forging(result)) => {
                    self.forging_tick_ms = Some(now_ms);
                    self.apply_craft(CraftingJob::Blacksmith, result, character, &mut out)?
                }
                Err(refusal) => out.replies.push(refusal.feedback()),
            }
            return Ok(out);
        }
        // Item bar (42/6, 42/9, 42/15 and 42/16 for bar items, 36/10 on bar 2) before the chest
        // route and the shared 42 branch; a 36/10 on any other bar still reaches unsupported_cast.
        if let Some(result) = self.handle_item_bar(inner, character, now_ms)? {
            return Ok(result);
        }
        if let Some(result) = self.handle_mystery_chest(inner, character.guid)? {
            return Ok(result);
        }
        if inner.len() >= 4 && (inner[..2] == [42, 0] || inner[..4] == [35, 0, 34, 0]) {
            let inventory_before = self.inventory.state().clone();
            let result = self.handle_inventory(inner)?;
            out.changed |= result.changed;
            // T027: only a change a stranger can see goes to neighbours; an alt
            // job's outfit and a no-op replay stay local.
            let (inactive, broadcast) =
                self.inventory_visibility(&inner, character.guid, result.changed, &result.replies);
            out.replies.extend(
                result
                    .replies
                    .into_iter()
                    .filter(|r| !(inactive && r.starts_with(&[35, 0, 6, 0]))),
            );
            if let Some(packet) = broadcast {
                out.nearby.push(packet);
            }
            if result.changed {
                self.npc_inventory_events(&inventory_before, &mut out)?;
                self.opening_equipment_events(&mut out)?;
                self.sync_world_stage(&mut out)?;
            }
            return Ok(out);
        }
        let interaction = if inner.starts_with(&[26, 0, 8, 0]) {
            if inner.len() != 13 || inner[12] > 1 {
                return Err(invalid("malformed opening interaction"));
            }
            Some((u64::from_le_bytes(inner[4..12].try_into().unwrap()), false))
        } else if inner.starts_with(&[26, 0, 10, 0]) {
            if inner.len() != 16 {
                return Err(invalid("malformed opening interaction selection"));
            }
            Some((u64::from_le_bytes(inner[4..12].try_into().unwrap()), true))
        } else {
            None
        };
        if let Some((guid, selection)) = interaction {
            if self.actor_hidden(guid) {
                return Ok(out);
            }
            if !self.encounter.as_ref().unwrap().active() {
                let actor = self
                    .opening
                    .as_ref()
                    .unwrap()
                    .actors()
                    .iter()
                    .find(|a| a.guid == guid && a.kind != "enemy")
                    .cloned();
                if let Some(actor) = actor {
                    let distance_squared: f32 = actor
                        .position
                        .iter()
                        .zip(character.position)
                        .map(|(a, b)| (a - b).powi(2))
                        .sum();
                    if actor.zone_id != character.zone_id
                        || distance_squared > actor.interaction_radius.powi(2)
                    {
                        out.replies.push(chat::system("Move closer to interact."));
                        return Ok(out);
                    }
                    if selection && inner[12..16] != 1i32.to_le_bytes() {
                        return Ok(out);
                    }
                    // One actual menu choice performs one talk/gather/craft
                    // action. OpeningEngine independently checks job and range.
                    if !selection {
                        out.replies.push(OpeningWorld::menu(&actor));
                        return Ok(out);
                    }
                    // Old checkpoints returned to the cave's original world
                    // location. The recovered story sends the hero onward to
                    // Farnum through Darkthorne's magic door instead.
                    if let Some(zone) = self.opening_dungeon_destination(Some(guid)) {
                        out.zoning = Some(zone);
                        out.replies.push(chat::system(
                            "Darkthorne's magic door takes you to Farnum's Farmstead.",
                        ));
                        return Ok(out);
                    }
                    if matches!(
                        actor.action,
                        Some(crate::opening::OpeningTrigger::Craft { recipe_id: 2001 })
                    ) && self.cooking.is_some()
                    {
                        if self.profiles.as_ref().unwrap().active() != 45 {
                            out.replies
                                .push(chat::system("Switch to Chef before cooking."));
                        } else if !self.opening_craft_pending(2001) {
                            out.replies.push(chat::system(
                                "Accept the cooking objective and finish its earlier steps first.",
                            ));
                        } else if self
                            .cooking
                            .as_ref()
                            .unwrap()
                            .requirements()
                            .iter()
                            .any(|(id, n)| self.material_count(*id) < u64::from(*n))
                        {
                            out.replies.push(chat::system("StrongArm Stew requires 4 Bat Meat, 4 Gummyfruit, 4 Rainleaf and 4 Spice."));
                        } else {
                            let key = self.cooking.as_ref().unwrap().recipe_key().to_owned();
                            let rank = self.recipe_rank(45);
                            let learned = self.learned_recipe_for_key(&key);
                            let available = self.available_ingredients();
                            let result = self
                                .cooking
                                .as_mut()
                                .unwrap()
                                .begin_recipe(45, &key, rank, learned, &available)?;
                            self.apply_craft(CraftingJob::Chef, result, character, &mut out)?;
                        }
                        return Ok(out);
                    }
                    if matches!(
                        actor.action,
                        Some(crate::opening::OpeningTrigger::Craft { recipe_id: 2003 })
                    ) && self.forging.is_some()
                    {
                        if self.profiles.as_ref().unwrap().active() != 16 {
                            out.replies
                                .push(chat::system("Switch to Blacksmith before forging."));
                        } else if !self.opening_craft_pending(2003) {
                            out.replies.push(chat::system(
                                "Accept the forging objective and finish its earlier steps first.",
                            ));
                        } else if self
                            .forging
                            .as_ref()
                            .unwrap()
                            .requirements()
                            .iter()
                            .any(|(id, n)| self.material_count(*id) < u64::from(*n))
                        {
                            out.replies.push(chat::system("Forging requires 10 Copper Bars, 5 Turquoise, 5 Wooden Planks and 5 Thorny Fasteners."));
                        } else {
                            let key = self.forging.as_ref().unwrap().recipe_key().to_owned();
                            let rank = self.recipe_rank(16);
                            let learned = self.learned_recipe_for_key(&key);
                            let available = self.available_ingredients();
                            let result = self
                                .forging
                                .as_mut()
                                .unwrap()
                                .begin_recipe(16, &key, rank, learned, &available)?;
                            self.forging_tick_ms = Some(now_ms);
                            self.apply_craft(CraftingJob::Blacksmith, result, character, &mut out)?;
                            out.replies.push(chat::system("Original Spunky Scrapper Hammer controls; board tuning, consumption and tint choice are reconstructed."));
                        }
                        return Ok(out);
                    }
                    if matches!(
                        actor.action,
                        Some(crate::opening::OpeningTrigger::Craft { recipe_id: 2002 })
                    ) && self.smelting.is_some()
                    {
                        if self.profiles.as_ref().unwrap().active() != 14 {
                            out.replies
                                .push(chat::system("Switch to Miner before smelting."));
                        } else if !self.opening_craft_pending(2002) {
                            out.replies.push(chat::system(
                                "Accept the smelting objective and finish its earlier steps first.",
                            ));
                        } else if !self
                            .smelting
                            .as_ref()
                            .unwrap()
                            .requirements()
                            .iter()
                            .all(|(id, n)| self.material_count(*id) >= u64::from(*n))
                        {
                            out.replies.push(chat::system("Smelting requires 5 Copper Ore and 3 Singing Coal. Mine the accepted nearby ore objective first."));
                        } else {
                            let key = self.smelting.as_ref().unwrap().recipe_key().to_owned();
                            let rank = self.recipe_rank(14);
                            let learned = self.learned_recipe_for_key(&key);
                            let available = self.available_ingredients();
                            let result = self
                                .smelting
                                .as_mut()
                                .unwrap()
                                .begin_recipe(14, &key, rank, learned, &available)?;
                            self.apply_craft(CraftingJob::Miner, result, character, &mut out)?;
                            out.replies.push(chat::system("Original four-stage smelting controls; phase tuning and this local recipe output are reconstructed."));
                        }
                        return Ok(out);
                    }
                    let gather_before = self.gather_count(1003);
                    let resource_before = match actor.action {
                        Some(crate::opening::OpeningTrigger::Gather { resource_id }) => {
                            Some((resource_id, self.gather_count(resource_id)))
                        }
                        _ => None,
                    };
                    let chef_resource = match actor.action {
                        Some(crate::opening::OpeningTrigger::Gather {
                            resource_id: id @ (1001 | 1002),
                        }) => Some(id),
                        _ => None,
                    };
                    let chef_before = chef_resource.map(|id| self.gather_count(id));
                    let result = self.opening.as_mut().unwrap().on_interaction(guid)?;
                    self.apply_opening(result, &mut out)?;
                    if matches!(
                        &actor.action,
                        Some(crate::opening::OpeningTrigger::Interact { actor_guid })
                            if *actor_guid == guid
                    ) {
                        self.npc_quest_event(
                            crate::quest_engine::QuestEvent::InteractNpc { npc_id: guid },
                            &mut out,
                        )?;
                    }
                    if let Some((resource_id, before)) = resource_before {
                        let quantity = self.gather_count(resource_id).saturating_sub(before);
                        if quantity > 0 {
                            self.npc_quest_event(
                                crate::quest_engine::QuestEvent::GatherResource {
                                    resource_id,
                                    quantity,
                                },
                                &mut out,
                            )?;
                        }
                    }
                    if self.smelting.is_some()
                        && self.profiles.as_ref().unwrap().active() == 14
                        && matches!(
                            actor.action,
                            Some(crate::opening::OpeningTrigger::Gather { resource_id: 1003 })
                        )
                        && self.gather_count(1003) > gather_before
                    {
                        // Temporary local drop, only on newly counted accepted
                        // mining progress. This is not an original loot table.
                        self.grant_reward(0, &[(2739, 1), (879, 1)], &mut out)?;
                    }
                    if self.cooking.is_some() && self.profiles.as_ref().unwrap().active() == 45 {
                        if let (Some(resource), Some(before)) = (chef_resource, chef_before) {
                            if self.gather_count(resource) > before {
                                let definition = if resource == 1001 { 2884 } else { 2875 };
                                self.grant_reward(0, &[(definition, 1)], &mut out)?;
                            }
                        }
                    }
                    self.opening_equipment_events(&mut out)?;
                    self.sync_world_stage(&mut out)?;
                    let pending = self
                        .opening
                        .as_ref()
                        .unwrap()
                        .active_stage()
                        .and_then(|stage| {
                            if stage.giver_guid != guid {
                                return None;
                            }
                            let progress =
                                &self.opening.as_ref().unwrap().state().stages[&stage.id];
                            if !progress.accepted {
                                return None;
                            }
                            stage.objectives.iter().zip(&progress.counts).find_map(
                                |(goal, count)| {
                                    if *count >= goal.required {
                                        return None;
                                    }
                                    match goal.trigger {
                                        crate::opening::OpeningTrigger::DungeonCompleted {
                                            encounter_id,
                                        } => Some(encounter_id),
                                        _ => None,
                                    }
                                },
                            )
                        });
                    if let Some(id) = pending {
                        self.enter_opening_cave(id, character, &mut out)?;
                    } else if selection {
                        if let Some(id) = self.completed_dungeon_replay(guid) {
                            self.enter_opening_cave(id, character, &mut out)?;
                        }
                    }
                    return Ok(out);
                }
            }
        }
        let active_profile = self.profiles.as_ref().unwrap().active();
        // 36/12: tooltip/detail lookup. (R) Only the definitions already sent for this
        // job at this level resolve; any other id is silently dropped (no state change).
        if inner.starts_with(&[36, 0, 12, 0]) {
            let id = crate::abilities::parse_definition_request(inner)?;
            let loaded = self
                .abilities
                .as_ref()
                .unwrap()
                .activation(active_profile, self.job_level(active_profile))?;
            out.replies
                .extend(crate::abilities::definition_reply(&loaded, id));
            return Ok(out);
        }
        // Ground/pulse modes and other bars get a client failure, never silence. (R)
        if let Some(refusal) = crate::abilities::unsupported_cast(inner)? {
            out.replies.push(refusal);
            return Ok(out);
        }
        if inner.starts_with(&[36, 0, 10, 0]) && inner.len() >= 12 {
            // Native ActionBarData.Id is the bar category (1), not the job.
            // The supplied client's captured basic requests confirm bar1/slot0.
            // Only the actual displayed basic slot is implemented in this slice.
            let bar = i32::from_le_bytes(inner[4..8].try_into().unwrap());
            let slot = i32::from_le_bytes(inner[8..12].try_into().unwrap());
            // Job abilities on slots 1.. (keys 2..): targeted (type 0) requests.
            if bar == 1 && slot >= 1 {
                let ability = self
                    .abilities
                    .as_ref()
                    .unwrap()
                    .job_slot(active_profile, slot, self.job_level(active_profile))
                    .cloned();
                if let Some(ability) = ability {
                    if inner.len() == 24 && inner[12..16] == 0_i32.to_le_bytes() {
                        if !self.abilities.as_ref().unwrap().weapon_matches(
                            &self.inventory,
                            active_profile,
                            slot,
                            self.job_level(active_profile),
                        ) {
                            self.encounter.as_mut().unwrap().refuse_weapon(
                                active_profile,
                                slot,
                                now_ms,
                            );
                            return Ok(out);
                        }
                        let target = u64::from_le_bytes(inner[16..24].try_into().unwrap());
                        let result = self.encounter.as_mut().unwrap().job_ability(
                            &ability,
                            target,
                            character.position,
                            active_profile,
                            now_ms,
                        )?;
                        self.apply_encounter(result, &mut out)?;
                    }
                } else {
                    // Empty or level-locked slot: explicit failure, no state change. (R)
                    out.replies
                        .push(crate::abilities::unsupported_cast_failure());
                }
                return Ok(out);
            }
            if bar != 1
                || slot != 0
                || !self
                    .abilities
                    .as_ref()
                    .unwrap()
                    .supports_basic(active_profile)
            {
                // Slot 0 on a non-combat job: explicit failure. (R)
                out.replies
                    .push(crate::abilities::unsupported_cast_failure());
                return Ok(out);
            }
        }
        if inner.starts_with(&[26, 0, 10, 0])
            && inner.len() == 16
            && inner[12..16] == 103i32.to_le_bytes()
            && !self
                .abilities
                .as_ref()
                .unwrap()
                .supports_basic(active_profile)
        {
            out.replies
                .push(chat::system("Switch to a combat job before attacking."));
            return Ok(out);
        }
        // Both native basic paths share the live equipment guard. (R) Job
        // weapon classes gate casts, including self-centred basic requests;
        // no damage, energy, effects or cooldown mutation precedes this check.
        let basic_cast = (inner.len() == 24
            && inner.starts_with(&[36, 0, 10, 0])
            && inner[4..8] == 1_i32.to_le_bytes()
            && inner[8..16] == [0; 8])
            || (inner.len() == 16
                && inner.starts_with(&[26, 0, 10, 0])
                && inner[12..16] == 103_i32.to_le_bytes());
        if basic_cast
            && !self.abilities.as_ref().unwrap().weapon_matches(
                &self.inventory,
                active_profile,
                0,
                self.job_level(active_profile),
            )
        {
            self.encounter
                .as_mut()
                .unwrap()
                .refuse_weapon(active_profile, 0, now_ms);
            return Ok(out);
        }
        // Go on a battle start screen (41/108): the owning module runs the
        // prize gate and keeps the screen open on a refusal.
        if inner.starts_with(&[41, 0, 108, 0]) && self.handle_battle_go(character, &mut out)? {
            return Ok(out);
        }
        let result = self.encounter.as_mut().unwrap().handle(
            inner,
            character.guid,
            character.position,
            self.profiles.as_ref().unwrap().active(),
            now_ms,
        )?;
        self.apply_encounter(result, &mut out)?;
        Ok(out)
    }

    fn opening_craft_pending(&self, recipe: u32) -> bool {
        self.opening.as_ref().is_some_and(|opening| {
            opening.active_stage().is_some_and(|stage| {
                let progress = &opening.state().stages[&stage.id];
                progress.accepted
                    && stage
                        .objectives
                        .iter()
                        .zip(&progress.counts)
                        .find(|(goal, count)| **count < goal.required)
                        .is_some_and(|(goal, _)| {
                            matches!(goal.trigger,
                        crate::opening::OpeningTrigger::Craft { recipe_id } if recipe_id == recipe)
                        })
            })
        })
    }

    fn near_craft_station(&self, character: &CharacterState, job: CraftingJob) -> bool {
        crate::recipe_book::near_any(
            &self.crafting_stations(),
            job,
            character.zone_id,
            character.position,
        )
    }

    /// Resolve the opening stations and every currently placed station model.
    /// Deriving this view at use keeps placement reloads and removals authoritative.
    fn crafting_stations(&self) -> Vec<crate::recipe_book::Station> {
        let mut stations = self
            .opening
            .as_ref()
            .map(|engine| crate::recipe_book::stations(engine.actors()))
            .unwrap_or_default();
        if let Some(visibility) = &self.npc_visibility {
            crate::recipe_book::merge_stations(
                &mut stations,
                crate::recipe_book::world_stations(
                    &self.world_station_rules,
                    visibility.model_actors(),
                ),
            );
        }
        stations
    }

    fn actor_hidden(&self, guid: u64) -> bool {
        self.disappearing_actors.iter().any(|a| {
            a.guid == guid
                && self.opening.as_ref().is_some_and(|o| {
                    o.state().stages.get(&a.stage_id).is_some_and(|p| {
                        p.accepted && p.counts.get(a.index).is_some_and(|n| *n >= a.required)
                    })
                })
        })
    }

    fn hidden_actor_guids(&self) -> BTreeSet<u64> {
        self.disappearing_actors
            .iter()
            .filter(|a| self.actor_hidden(a.guid))
            .map(|a| a.guid)
            .collect()
    }

    fn gather_count(&self, resource: u32) -> u32 {
        let opening = self.opening.as_ref().unwrap();
        opening.active_stage().map(|stage| {
            stage.objectives.iter().zip(&opening.state().stages[&stage.id].counts)
                .filter(|(goal, _)| matches!(goal.trigger,
                    crate::opening::OpeningTrigger::Gather { resource_id } if resource_id == resource))
                .map(|(_, count)| *count).sum()
        }).unwrap_or(0)
    }

    fn material_count(&self, definition: i32) -> u64 {
        self.inventory
            .state()
            .items
            .iter()
            .filter(|item| item.definition_id == definition && item.rental.is_none())
            .map(|item| u64::from(item.count))
            .sum()
    }

    /// One completion for the three crafting movies. The recipe key is read
    /// only after a transaction arrives, and only the Miner tutorial counts the
    /// bars it grants.
    fn apply_craft(
        &mut self,
        job: CraftingJob,
        result: CraftOutcome,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(transaction) = result.transaction {
            // (R) A recipe-book run has no tutorial objective; the profile and
            // station checks still bind the completion.
            if self.profiles.as_ref().unwrap().active() != job.profile_id()
                || !self.near_craft_station(character, job)
            {
                return Err(invalid(format!(
                    "{} completion is outside a {} station",
                    job.noun(),
                    job.name()
                )));
            }
            if transaction
                .consumes
                .iter()
                .any(|(id, n)| self.material_count(*id) < u64::from(*n))
            {
                return Err(invalid(format!(
                    "{} ingredients changed before completion",
                    job.noun()
                )));
            }
            self.consume_materials(transaction.consumes, out)?;
            self.grant_reward(0, &transaction.grants, out)?;
            // (R) Only the Miner tutorial counts its Copper Bar (21785) grants;
            // cooking and forging advance their objective by one completion.
            let quantity: u32 = if job == CraftingJob::Miner {
                transaction
                    .grants
                    .iter()
                    .filter(|(id, _)| *id == 21785)
                    .map(|(_, n)| *n)
                    .sum()
            } else {
                1
            };
            let opening_recipe = job.opening_recipe_id();
            if self.opening_craft_pending(opening_recipe) {
                let progress = self
                    .opening
                    .as_mut()
                    .unwrap()
                    .on_event(OpeningEvent::Craft {
                        recipe_id: opening_recipe,
                        quantity,
                    })?;
                self.apply_opening(progress, out)?;
            }
            // (R) The opening binding is the fallback for the unnumbered sample.
            let recipe_id = self
                .recipe_catalog
                .by_key(match job {
                    CraftingJob::Chef => self.cooking.as_ref().unwrap().recipe_key(),
                    CraftingJob::Miner => self.smelting.as_ref().unwrap().recipe_key(),
                    CraftingJob::Blacksmith => self.forging.as_ref().unwrap().recipe_key(),
                })
                .and_then(|r| r.recipe_id)
                .unwrap_or(opening_recipe);
            self.npc_quest_event(
                crate::quest_engine::QuestEvent::CraftRecipe {
                    recipe_id,
                    quantity: 1,
                },
                out,
            )?;
            self.npc_minigame_win(&result.replies, out)?;
        }
        out.changed |= result.changed;
        out.replies.extend(result.replies);
        Ok(())
    }

    /// The Type8 engine for one gather game. Mining and Harvesting keep
    /// separate runs; `Gather` is the only key their shared paths need.
    fn type8(&self, gather: Gather) -> Option<&Type8Engine> {
        match gather {
            Gather::Mining => self.mining.as_ref(),
            Gather::Harvesting => self.harvesting.as_ref(),
        }
    }

    fn type8_mut(&mut self, gather: Gather) -> Option<&mut Type8Engine> {
        match gather {
            Gather::Mining => self.mining.as_mut(),
            Gather::Harvesting => self.harvesting.as_mut(),
        }
    }

    /// Stage a TYPE8 outcome (Mining18 or Harvesting111). The only reward path is
    /// the engine's server-side transaction derived from the configured
    /// `outputs` and the collected quota; client score or win reports never
    /// reach it. Grants commit inside the staged gameplay clone, which the
    /// gateway saves before sending.
    fn apply_type8(
        &mut self,
        gather: Gather,
        result: Type8Outcome,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(transaction) = result.transaction {
            let grant = match gather {
                Gather::Mining => crate::mining::lifecycle::reward_grant(Some(&transaction), 0)?,
                Gather::Harvesting => crate::harvesting::lifecycle::reward_grant(
                    Some(&transaction),
                    HARVESTING_GAME_ID,
                    0,
                )?,
            };
            if self.profiles.as_ref().map(Profiles::active) != Some(gather.job()) {
                return Err(invalid(match gather {
                    Gather::Mining => "mining completion is outside the accepted Miner job",
                    Gather::Harvesting => "harvesting completion is outside the accepted Chef job",
                }));
            }
            self.consume_materials(transaction.consumes, out)?;
            let grants: Vec<_> = grant
                .items
                .iter()
                .map(|item| (item.definition_id, item.count))
                .collect();
            self.grant_reward(0, &grants, out)?;
            // (R) TYPE8 resource keys use the configured output item definition;
            // no retail regional resource-to-quest table has been recovered.
            for item in &grant.items {
                self.npc_quest_event(
                    crate::quest_engine::QuestEvent::GatherResource {
                        resource_id: item.definition_id as u32,
                        quantity: item.count,
                    },
                    out,
                )?;
            }
            self.npc_minigame_win(&result.replies, out)?;
            for amount in grant.job_xp.values() {
                self.award_job_xp(*amount, out)?;
            }
            if let Some(nodes) = self.gathering_nodes.as_mut() {
                nodes.paid();
            }
        }
        out.changed |= result.changed;
        out.replies.extend(result.replies);
        Ok(())
    }

    fn consume_materials(
        &mut self,
        consumes: Vec<(i32, u32)>,
        out: &mut Outcome,
    ) -> io::Result<()> {
        for (definition, mut remaining) in consumes {
            let stacks: Vec<_> = self
                .inventory
                .state()
                .items
                .iter()
                .filter(|item| item.definition_id == definition && item.rental.is_none())
                .map(|item| (item.id, item.count))
                .collect();
            for (id, available) in stacks {
                if remaining == 0 {
                    break;
                }
                let quantity = available.min(remaining);
                let consumed = self.inventory.consume(id, quantity)?;
                out.changed |= consumed.changed;
                out.replies.extend(consumed.replies);
                remaining -= quantity;
            }
            if remaining != 0 {
                return Err(invalid("crafting material consumption incomplete"));
            }
        }
        Ok(())
    }

    fn opening_equipment_events(&mut self, out: &mut Outcome) -> io::Result<()> {
        let profile = self.profiles.as_ref().unwrap().active();
        let result = self
            .opening
            .as_mut()
            .unwrap()
            .on_event(OpeningEvent::ClassSelected {
                profile_id: profile,
            })?;
        self.apply_opening(result, out)?;
        let equipped: Vec<_> = self
            .inventory
            .state()
            .equipment
            .iter()
            .filter(|e| e.profile_id == profile && e.item_id != 0)
            .filter_map(|e| {
                self.inventory
                    .state()
                    .items
                    .iter()
                    .find(|i| i.id == e.item_id)
                    .map(|i| i.definition_id)
            })
            .collect();
        for definition in equipped {
            let result =
                self.opening
                    .as_mut()
                    .unwrap()
                    .on_event(OpeningEvent::CosmeticEquipped {
                        definition_id: definition,
                    })?;
            self.apply_opening(result, out)?;
        }
        Ok(())
    }

    fn apply_opening(&mut self, result: OpeningOutcome, out: &mut Outcome) -> io::Result<()> {
        out.changed |= result.changed;
        out.replies
            .extend(result.messages.iter().map(|m| chat::system(m)));
        for id in result.unlock_profiles {
            if self.profiles.as_mut().unwrap().unlock(id)? {
                out.changed = true;
                self.give_starter_outfit(id, false, out)?;
            }
        }
        for reward in result.rewards {
            self.grant_reward(reward.coins, &reward.items, out)?;
            let profiles = self.profiles.as_mut().unwrap();
            let previous_xp = profiles
                .state()
                .experience
                .get(&profiles.active())
                .copied()
                .unwrap_or(0);
            out.changed |= profiles.add_experience(25)?;
            out.replies.extend(profiles.experience_packets(previous_xp));
        }
        self.refresh_npc_quest_eligibility(current_unix_seconds());
        Ok(())
    }

    fn sync_world_stage(&mut self, out: &mut Outcome) -> io::Result<()> {
        let opening = self.opening.as_ref().unwrap();
        // Do not expose finite quest enemies until their ordered defeat goal
        // can count them. Otherwise early kills permanently exhaust the pool.
        let stage = opening
            .active_stage()
            .filter(|s| {
                let progress = &opening.state().stages[&s.id];
                progress.accepted
                    && s.objectives
                        .iter()
                        .zip(&progress.counts)
                        .find(|(goal, count)| **count < goal.required)
                        .is_some_and(|(goal, _)| {
                            matches!(goal.trigger, crate::opening::OpeningTrigger::Defeat { .. })
                        })
            })
            .map(|s| s.id)
            .unwrap_or(0);
        let result = self.encounter.as_mut().unwrap().set_world_stage(stage);
        self.apply_encounter(result, out)
    }

    fn apply_encounter(&mut self, result: EncounterOutcome, out: &mut Outcome) -> io::Result<()> {
        out.changed |= result.changed;
        out.group_heal = result.group_heal.or(out.group_heal);
        out.replies.extend(result.replies);
        if let Some(feedback) = result.feedback {
            out.replies.push(chat::system(&feedback));
        }
        // Only job XP, a kill or a completed run can move the achievement
        // counters; a plain hit or a coin-only reward skips the scan.
        let counters_before = (result.rewards.iter().any(|reward| reward.job_xp > 0)
            || !result.defeated.is_empty()
            || result.completed.is_some())
        .then(|| self.achievement_counters());
        for reward in result.rewards {
            self.grant_reward(reward.coins, &reward.items, out)?;
            if reward.job_xp > 0 {
                self.award_job_xp(reward.job_xp, out)?;
            }
        }
        for definition in result.defeated {
            let result = self
                .opening
                .as_mut()
                .unwrap()
                .on_event(OpeningEvent::Defeat {
                    definition_id: definition,
                })?;
            self.apply_opening(result, out)?;
            // NPC quests count kills by enemy definition id ("defeat" goals author
            // that id as `model_id`; quest_engine maps it to KillActorKind).
            if let Ok(actor_kind_id) = u32::try_from(definition) {
                if actor_kind_id != 0 {
                    // Ticks and requests both report kills here; register any goal
                    // this defeat opened before the outcome is returned.
                    self.npc_quest_event(
                        crate::quest_engine::QuestEvent::KillActorKind {
                            actor_kind_id,
                            count: 1,
                        },
                        out,
                    )?;
                }
            }
        }
        if let Some(id) = result.completed {
            let result = self
                .opening
                .as_mut()
                .unwrap()
                .on_event(OpeningEvent::DungeonCompleted { encounter_id: id })?;
            self.apply_opening(result, out)?;
            // `completed` is set by EncounterEngine only when a won run exits. Entry,
            // losses, retries and duplicate result/close messages leave it empty.
            self.npc_quest_event(
                crate::quest_engine::QuestEvent::CompleteDungeon { encounter_id: id },
                out,
            )?;
        }
        if let Some(zone) = result.zoning {
            out.zoning = Some(if result.completed.is_some() {
                self.opening_dungeon_destination(None).unwrap_or(zone)
            } else {
                zone
            });
        }
        if let Some(counters_before) = counters_before {
            self.settle_achievement_counters(&counters_before, out)?;
        }
        Ok(())
    }

    /// Follow the completed cave objective to the next story guide. Locale
    /// 4420033/4420043 prove the magic-door journey; the nearby offset is local.
    fn opening_dungeon_destination(&self, giver: Option<u64>) -> Option<ZoneTransition> {
        let opening = self.opening.as_ref()?;
        let stage = opening.active_stage()?;
        if giver.is_some_and(|id| id != stage.giver_guid) {
            return None;
        }
        let progress = opening.state().stages.get(&stage.id)?;
        if !progress.accepted {
            return None;
        }
        let mut cleared_cave = false;
        for (goal, count) in stage.objectives.iter().zip(&progress.counts) {
            if *count >= goal.required {
                if let crate::opening::OpeningTrigger::DungeonCompleted { encounter_id } =
                    goal.trigger
                {
                    cleared_cave |= self
                        .encounter
                        .as_ref()?
                        .state()
                        .rewarded
                        .contains(&encounter_id);
                }
                continue;
            }
            if !cleared_cave {
                return None;
            }
            let crate::opening::OpeningTrigger::Talk { actor_guid } = goal.trigger else {
                return None;
            };
            let actor = opening.actors().iter().find(|a| a.guid == actor_guid)?;
            let mut zone = self.encounter.as_ref()?.world().clone();
            if actor.zone_id != zone.zone_id || actor.guid == stage.giver_guid {
                return None;
            }
            zone.position = actor.position;
            zone.position[0] += 3.0;
            zone.rotation = [-1.0, 0.0, 0.0];
            return Some(zone);
        }
        None
    }

    /// Switch to a basic-combat job when needed, then begin the bound cave run.
    fn enter_opening_cave(
        &mut self,
        encounter_id: u32,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if !self
            .abilities
            .as_ref()
            .unwrap()
            .supports_basic(self.profiles.as_ref().unwrap().active())
        {
            let outfit_before = self
                .inventory
                .profile_attachments(self.profiles.as_ref().unwrap().active())?;
            let reply = self
                .profiles
                .as_mut()
                .unwrap()
                .select(43, &self.inventory)?;
            out.changed = true;
            out.replies.push(reply);
            out.replies.extend(
                self.abilities
                    .as_ref()
                    .unwrap()
                    .activation(43, self.job_level(43))?,
            );
            out.replies
                .push(chat::system("Switched to Brawler for the opening cave."));
            self.opening_equipment_events(out)?;
            self.announce_profile_outfit(character.guid, &outfit_before, true, out)?;
        }
        let result = self.encounter.as_mut().unwrap().begin(
            encounter_id,
            character.clone(),
            self.profiles.as_ref().unwrap().active(),
        )?;
        self.apply_encounter(result, out)
    }

    /// Replay gate for the finished opening cave. The giver, stage and
    /// encounter are derived from the opening content already loaded in
    /// `OpeningEngine`; no id is hardcoded and no save schema is involved.
    /// Fails closed: no start while any story stage is unfinished or
    /// unrewarded, for a different actor, for an ambiguous binding, or without
    /// a rewarded-dungeon proof.
    fn completed_dungeon_replay(&self, guid: u64) -> Option<u32> {
        let opening = self.opening.as_ref()?;
        if opening.active_stage().is_some() {
            return None;
        }
        let state = opening.state();
        if state.stages.values().any(|progress| !progress.rewarded) {
            return None;
        }
        let mut bound = None;
        for stage in opening
            .stages()
            .iter()
            .filter(|stage| stage.giver_guid == guid)
        {
            let progress = state.stages.get(&stage.id)?;
            if !progress.completed || !progress.rewarded {
                return None;
            }
            for goal in &stage.objectives {
                if let OpeningTrigger::DungeonCompleted { encounter_id } = goal.trigger {
                    if bound.is_some_and(|id| id != encounter_id) {
                        return None;
                    }
                    bound = Some(encounter_id);
                }
            }
        }
        let encounter_id = bound?;
        let encounter = self.encounter.as_ref()?;
        if encounter.active() || !encounter.state().rewarded.contains(&encounter_id) {
            return None;
        }
        Some(encounter_id)
    }

    /// W146: the four item-bar slots (38/25 at world entry).
    fn append_item_bar(&mut self, now_ms: u64, out: &mut Outcome) -> io::Result<()> {
        out.replies
            .extend(self.item_bar.refresh(&self.inventory, now_ms)?);
        Ok(())
    }

    /// W146: item-bar requests. The potion is consumed and the heal applied in this one call, so
    /// the saved state never holds one without the other. Without an encounter engine the player
    /// has no tracked health, so nothing is claimed and nothing is spent.
    fn handle_item_bar(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let Some(encounter) = self.encounter.as_ref() else {
            return Ok(None);
        };
        let player = crate::item_bar::Player {
            guid: character.guid,
            alive: encounter.state().player_health > 0,
        };
        let Some(reply) = self
            .item_bar
            .handle(inner, &mut self.inventory, &player, now_ms)?
        else {
            return Ok(None);
        };
        let mut out = Outcome {
            replies: reply.replies,
            changed: reply.changed,
            ..Outcome::default()
        };
        if reply.heal_percent > 0 {
            if let Some(encounter) = self.encounter.as_mut() {
                out.replies
                    .extend(encounter.heal_player_percent(reply.heal_percent));
            }
        }
        Ok(Some(out))
    }

    fn handle_mystery_chest(
        &mut self,
        inner: &[u8],
        player_guid: u64,
    ) -> io::Result<Option<Outcome>> {
        let Some(content) = &self.mystery_chests else {
            return Ok(None);
        };
        let Some(request) = mystery_chest::parse_request(inner)? else {
            return Ok(None);
        };
        let before = self.inventory.state().clone();
        let result = content.handle(
            &mut self.inventory,
            &mut self.pending_chest,
            request,
            player_guid,
        )?;
        let mut out = Outcome {
            replies: result.replies,
            changed: result.changed,
            ..Outcome::default()
        };
        self.npc_inventory_events(&before, &mut out)?;
        Ok(Some(out))
    }

    /// Job level on the local rank curve (R): one base rank, one per 100 XP,
    /// capped at 20 (as `character_eligibility` uses).
    fn job_level(&self, profile: i32) -> u32 {
        let xp = self
            .profiles
            .as_ref()
            .and_then(|p| p.state().experience.get(&profile).copied())
            .unwrap_or(0);
        (1 + xp / 100).min(20)
    }

    /// Max health and weapon damage follow the active combat job's level
    /// (final-era scaling in the encounter engine); other jobs stay at the
    /// level-1 baseline.
    fn sync_player_level(&mut self, out: &mut Outcome) {
        let Some(profile) = self.profiles.as_ref().map(Profiles::active) else {
            return;
        };
        let combat = self
            .abilities
            .as_ref()
            .is_some_and(|a| a.supports_basic(profile));
        let level = if combat { self.job_level(profile) } else { 1 };
        if let Some(encounter) = self.encounter.as_mut() {
            out.replies.extend(encounter.set_player_job(profile, level));
        }
    }

    /// Experience for the active job; a level that unlocks a job ability
    /// refreshes the ability bar at once.
    fn award_job_xp(&mut self, amount: u32, out: &mut Outcome) -> io::Result<()> {
        let profile = self.profiles.as_ref().unwrap().active();
        let before = self.job_level(profile);
        let profiles = self.profiles.as_mut().unwrap();
        let previous_xp = profiles
            .state()
            .experience
            .get(&profile)
            .copied()
            .unwrap_or(0);
        out.changed |= profiles.add_experience(amount)?;
        out.replies.extend(profiles.experience_packets(previous_xp));
        let after = self.job_level(profile);
        if self
            .abilities
            .as_ref()
            .is_some_and(|a| a.unlocks_between(profile, before, after))
        {
            out.replies.extend(
                self.abilities
                    .as_ref()
                    .unwrap()
                    .activation(profile, after)?,
            );
        }
        if after != before {
            self.sync_player_level(out);
        }
        Ok(())
    }

    fn grant_reward(
        &mut self,
        coins: u32,
        items: &[(i32, u32)],
        out: &mut Outcome,
    ) -> io::Result<()> {
        let before = self.inventory.state().clone();
        let result = self.inventory.grant_bundle(coins, items)?;
        out.changed |= result.changed;
        out.replies.extend(result.replies);
        self.npc_inventory_events(&before, out)?;
        Ok(())
    }

    /// (R) Player facts for equip eligibility; ranks use `job_level`'s curve.
    /// `None` without class progression, where the permissive route applies.
    fn equip_context(&self) -> Option<crate::inventory::EquipContext> {
        let profiles = self.profiles.as_ref()?;
        Some(crate::inventory::EquipContext {
            gender: self.gender,
            member: self
                .game_rules
                .membership
                .is_active_at(current_unix_seconds()),
            profile_ranks: profiles
                .state()
                .unlocked
                .iter()
                .map(|&id| (id, self.job_level(id) as i32))
                .collect(),
            rental_now: None, // (R) rental expiry unit unproven
        })
    }

    fn handle_inventory(&mut self, inner: &[u8]) -> io::Result<crate::inventory::InventoryOutcome> {
        match self.equip_context() {
            Some(player) => self.inventory.handle_with(inner, &player),
            None => self.inventory.handle(inner),
        }
    }

    pub fn apply_identity(&mut self, identity: &CharacterIdentity) -> io::Result<bool> {
        if self.profiles.is_none() {
            return Ok(false);
        }
        identity.validate()?;
        self.gender = identity.gender;
        let fingerprint = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(identity).map_err(invalid)?)
        );
        if let Some(saved) = &self.identity_fingerprint {
            if saved != &fingerprint {
                return Err(invalid(
                    "character identity differs from this progression save",
                ));
            }
            return Ok(false);
        }
        for attachment in &identity.attachments {
            self.inventory
                .grant_with_tint(0, attachment.definition, 1, attachment.tint_id)?;
            self.inventory
                .equip_owned_definition(attachment.definition, 1)?;
        }
        self.identity_fingerprint = Some(fingerprint);
        Ok(true)
    }

    pub fn initialization(&mut self, position: [f32; 3]) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        out.replies.push(self.categories.initialization());
        if let Some(profiles) = &self.profiles {
            out.replies
                .push(profiles.activation(profiles.active(), &self.inventory)?);
        }
        out.replies.extend(self.combat.initialization(position));
        let quest = self.quests.initialization()?;
        self.apply_quest(quest, &mut out)?;
        out.replies.push(self.recipe_replace_all_packet()?);
        out.replies.push(chat::system(
            "Rust restoration is live. Talk to the nearby guide, or type realm help in chat.",
        ));
        Ok(out)
    }

    pub fn on_position(&mut self, position: [f32; 3], now_ms: u64) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        let quest = self.quests.on_position(position, now_ms)?;
        self.apply_quest(quest, &mut out)?;
        Ok(out)
    }

    pub fn handle(
        &mut self,
        inner: &[u8],
        guid: u64,
        position: [f32; 3],
        now_ms: u64,
    ) -> io::Result<Outcome> {
        if let Some(out) = self.command_refusal(inner, guid)? {
            return Ok(out);
        }
        let mut out = Outcome::default();
        let text = match chat::request(inner, guid) {
            Ok(text) => text,
            Err(_) => return Ok(out),
        };
        if let Some(text) = text {
            let lower = text.trim().to_ascii_lowercase();
            if let Some(command) = lower.strip_prefix("realm ") {
                match command {
                    "help" => out.replies.push(chat::system("Local play: realm quest / realm quests / realm attack / realm respawn / realm race / realm status / realm inventory. Native inventory equip/unequip also works. Combat and checkpoint rules are reconstructed for this local build.")),
                    "attack" => {
                        let combat=self.combat.attack(position,now_ms);
                        self.apply_combat(combat,&mut out)?;
                    }
                    "respawn" => {
                        let combat=self.combat.respawn(position);
                        self.apply_combat(combat,&mut out)?;
                    }
                    "inventory" => out.replies.push(chat::system(&format!("Inventory: {} coins; {} item instances. Open your backpack to equip items and see reward quantities.",self.inventory.state().coins,self.inventory.state().items.len()))),
                    "status" => {
                        out.replies.push(chat::system(&format!("Saved wallet: {} coins. Quest and checkpoint status follows.",self.inventory.state().coins)));
                        self.quest_command("status",now_ms,&mut out)?;
                    }
                    other => {
                        self.quest_command(other,now_ms,&mut out)?;
                    }
                }
            }
            return Ok(out);
        }
        if let Some(result) = self.handle_mystery_chest(inner, guid)? {
            return Ok(result);
        }
        if inner.len() >= 4 && (inner[..2] == [42, 0] || inner[..4] == [35, 0, 34, 0]) {
            let before = self.inventory.state().clone();
            let inventory = self.handle_inventory(inner)?;
            out.changed |= inventory.changed;
            let (inactive, broadcast) =
                self.inventory_visibility(inner, guid, inventory.changed, &inventory.replies);
            out.replies.extend(
                inventory
                    .replies
                    .into_iter()
                    .filter(|r| !(inactive && r.starts_with(&[35, 0, 6, 0]))),
            );
            if let Some(packet) = broadcast {
                out.nearby.push(packet);
            }
            self.npc_inventory_events(&before, &mut out)?;
            out.replies.extend(self.learn_consumed_patterns(&before)?);
        }
        if inner.starts_with(&[26, 0, 13, 0]) {
            if inner.len() != 8 {
                return Err(invalid("invalid class selection request"));
            }
            let id = i32::from_le_bytes(inner[4..8].try_into().unwrap());
            let outfit_before = self
                .inventory
                .profile_attachments(self.profiles.as_ref().unwrap().active())?;
            if let Some(changed) = self.stage_job_selection(id, &mut out)? {
                self.announce_profile_outfit(guid, &outfit_before, changed, &mut out)?;
            }
        }
        let combat = self
            .combat
            .handle(inner, position, now_ms)
            .map_err(invalid)?;
        self.apply_combat(combat, &mut out)?;
        Ok(out)
    }

    fn apply_combat(&mut self, combat: CombatOutcome, out: &mut Outcome) -> io::Result<()> {
        out.changed |= combat.changed;
        out.replies.extend(combat.replies);
        if let Some(feedback) = combat.feedback {
            out.replies.push(chat::system(&feedback));
        }
        if let Some(guid) = combat.interacted {
            let quest = self.quests.interaction(guid)?;
            self.apply_quest(quest, out)?;
        }
        if let Some(definition) = combat.defeated {
            let quest = self.quests.enemy_defeated(definition)?;
            self.apply_quest(quest, out)?;
        }
        Ok(())
    }

    fn quest_command(&mut self, command: &str, now_ms: u64, out: &mut Outcome) -> io::Result<()> {
        let mut staged = self.quests.clone();
        match staged.handle_command(command, now_ms) {
            Ok(quest) => {
                self.quests = staged;
                self.apply_quest(quest, out)?;
            }
            Err(error) => out
                .replies
                .push(chat::system(&format!("Cannot do that yet: {error}"))),
        }
        Ok(())
    }

    fn apply_quest(&mut self, quest: QuestOutcome, out: &mut Outcome) -> io::Result<()> {
        out.changed |= quest.changed;
        out.replies.extend(quest.replies);
        out.replies
            .extend(quest.messages.iter().map(|message| chat::system(message)));
        for reward in quest.rewards {
            let mut inventory = self.inventory.clone();
            let mut quests = self.quests.clone();
            match inventory.grant_bundle(reward.coins, &reward.items) {
                Ok(result) => {
                    if reward.quest_id != 0 {
                        quests.acknowledge_reward(reward.quest_id)?;
                    }
                    self.inventory = inventory;
                    self.quests = quests;
                    out.changed |= result.changed;
                    out.replies.extend(result.replies);
                    for (item_id, quantity) in reward.items.iter().copied().filter(|(_, n)| *n > 0)
                    {
                        self.npc_quest_event(
                            crate::quest_engine::QuestEvent::CollectItem {
                                item_id,
                                quantity: quantity.min(1_000_000),
                            },
                            out,
                        )?;
                    }
                }
                Err(error) => out.replies.push(chat::system(&format!(
                    "Cannot collect this reward yet: {error}. Quest reward remains pending."
                ))),
            }
        }
        Ok(())
    }
}

/// CommandPacketEndDialog: family 26 subtype 4, header only. Client handler
/// `CommandPacketEndDialog_Handle` (`00aa0870`) requires exactly the 4-byte
/// header; constructor `009f5760` (BaseCommandPacket sub-opcode 4).
const END_DIALOG_26_4: [u8; 4] = [26, 0, 4, 0];

#[cfg(test)]
mod developer_teleport;

#[cfg(test)]
mod ordinary_quit_routing;

#[cfg(test)]
mod w64_regional_quest_tests;

#[cfg(test)]
mod w95_vendor_coin_store_regressions;

#[cfg(test)]
mod w104_quest_tests;

#[cfg(test)]
mod t013_objective_events;

#[cfg(test)]
mod t018_station_refresh;

#[cfg(test)]
mod reward_adoption;
