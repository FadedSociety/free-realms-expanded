//! Local gameplay integration. Content and wire templates remain private.
//! Domain transitions stage on a clone so rewards and quest claims commit together.
use crate::{
    character_store::CharacterState,
    chat,
    chess_service::{ChessActivityOutcome, ChessService},
    combat::{Combat, CombatOutcome, CombatState},
    cooking::{Cooking, CookingOutcome, CookingState},
    dev_teleport,
    encounter::{EncounterEngine, EncounterOutcome, EncounterState, ZoneTransition},
    forging::{Forging, ForgingOutcome, ForgingState},
    game_rules::GameRuleSet,
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
    rewards::RewardState,
    smelting::{Smelting, SmeltingOutcome, SmeltingState},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, HashSet},
    io::{self, Read},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

/// Warpstone definitions, attunement and validated travel (W142).
#[path = "restoration/piece_077.rs"]
#[allow(dead_code)]
mod piece_077;

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
    /// A connection-local derby keeps only the job to restore after a reconnect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derby: Option<crate::derby_gateway::DerbyGatewayState>,
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
    npc_interaction: Option<Arc<Mutex<NpcInteraction>>>,
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
        self.npc_visibility
            .as_ref()
            .unwrap_or(&crate::world::NpcVisibility::default())
            .shared_view(character.zone_id, character.position, fields, known)
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
        if let (Some(saved), Some(profiles)) = (saved_derby, profiles.as_mut()) {
            if saved.previous_job <= 0 {
                return Err(invalid("invalid saved derby previous job"));
            }
            // The match itself is connection-local; restore only the job it replaced. Self data at
            // login already carries the resulting profile, so no activation packet is needed here.
            profiles.select(saved.previous_job, &inventory)?;
        }
        let recipe_catalog = crate::recipes::RecipeCatalog::load_runtime(directory)?;
        // (R) The opening campaign historically has no recovered recipe-learn
        // packet sequence. Seed its two supplied starter patterns and the local
        // Copper Bar recipe so W61's cooking/smelting/forging flow remains usable.
        if opening.is_some() {
            learned_recipes.extend([11, 70, 990001, 2002]);
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
            Some(ChessService::new(
                crate::chess_service::ChessContent::load(&chess_path)?,
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
        let npc_interaction = if npc_content.join("npcs").is_dir()
            && npc_content.join("stores.json").is_file()
            && npc_content.join("vendors.json").is_file()
        {
            Some(Arc::new(Mutex::new(NpcInteraction::load(
                &npc_content.join("npcs"),
                &npc_content.join("stores.json"),
            )?)))
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
        if let (Some(kart), Some(interaction)) = (&kart_entry, &npc_interaction) {
            let unknown = interaction
                .lock()
                .map_err(|_| invalid("NPC interaction state is unavailable"))?
                .set_race_gates(kart.gate_npcs(), kart.menu_string_id());
            for npc in unknown {
                npc_event(
                    serde_json::json!({"event": "kart_gate_unknown_npc", "content_key": npc}),
                );
            }
        }
        let derby = crate::derby_gateway::DerbyGateway::load(directory)?;
        if let (Some(derby), Some(interaction)) = (&derby, &npc_interaction) {
            let unknown = interaction
                .lock()
                .map_err(|_| invalid("NPC interaction state is unavailable"))?
                .set_derby_gates(derby.gate_npcs(), derby.menu_string_id());
            for npc in unknown {
                npc_event(
                    serde_json::json!({"event": "derby_gate_unknown_npc", "content_key": npc}),
                );
            }
        }
        let fishing_spots = crate::fishing_service::FishingSpots::load(directory)?.map(Arc::new);
        if let (Some(spots), Some(interaction)) = (&fishing_spots, &npc_interaction) {
            let unknown = interaction
                .lock()
                .map_err(|_| invalid("NPC interaction state is unavailable"))?
                .set_fish_gates(&spots.gate_npcs(), spots.menu_string_id());
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
        if let (Some(profiles), Some(interaction)) = (&profiles, &npc_interaction) {
            let trainers = profiles.load_trainers(directory)?;
            let unknown = interaction
                .lock()
                .map_err(|_| invalid("NPC interaction state is unavailable"))?
                .set_job_trainers(trainers);
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

    /// Bind the split quest catalogue and its character-owned eligibility.
    pub fn bind_npc_quest_catalogue(
        &mut self,
        content_dir: &Path,
        character_id: u64,
    ) -> io::Result<()> {
        let bundled = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let source = if content_dir.join("quests").is_dir()
            && content_dir.join("jobs.json").is_file()
            && content_dir.join("npcs").is_dir()
        {
            content_dir
        } else if bundled.join("quests").is_dir()
            && bundled.join("jobs.json").is_file()
            && bundled.join("npcs").is_dir()
        {
            bundled.as_path()
        } else {
            if self.saved_npc_quests.is_some() {
                return Err(invalid("saved NPC quests require split quest content"));
            }
            return Ok(());
        };
        self.npc_quests = Some(crate::quest_engine::QuestEngine::from_split_content(
            source,
            character_id,
            crate::quest_engine::CharacterEligibility::default(),
            self.saved_npc_quests.take(),
        )?);
        self.register_breadcrumb_crosswalk(crate::quest_journal::load_crosswalk(source)?);
        self.refresh_npc_quest_eligibility(current_unix_seconds());
        Ok(())
    }

    /// Allow `quest_ids` to be started by a 49/15 breadcrumb request. Ids without a quest
    /// definition are dropped (R).
    pub fn register_breadcrumb_quests(&mut self, quest_ids: impl IntoIterator<Item = u32>) {
        let Some(engine) = &self.npc_quests else {
            return;
        };
        self.breadcrumb_quests = quest_ids
            .into_iter()
            .filter_map(|id| Some((id, engine.giver_npc_id(id)?)))
            .collect();
    }

    /// Bind the original-id crosswalk (W175): a 49/15 start may only name an original id it
    /// lists, and only quests with a definition can be started (R).
    pub fn register_breadcrumb_crosswalk(&mut self, crosswalk: crate::quest_journal::Crosswalk) {
        self.register_breadcrumb_quests(crosswalk.values().copied());
        self.breadcrumb_crosswalk = crosswalk;
    }

    /// Replace account rules after account configuration or membership changes.
    pub fn set_game_rules(&mut self, rules: GameRuleSet) {
        self.game_rules = rules;
        self.refresh_npc_quest_eligibility(current_unix_seconds());
    }

    fn character_eligibility(
        &self,
        now_unix_seconds: u64,
    ) -> crate::quest_engine::CharacterEligibility {
        let profile_state = self.profiles.as_ref().map(Profiles::state);
        let job_levels = self
            .npc_quests
            .as_ref()
            .into_iter()
            .flat_map(|engine| engine.job_profile_ids())
            .filter_map(|(job_id, profile_id)| {
                let state = profile_state?;
                if profile_id <= 0
                    || profile_id != state.active
                    || !state.unlocked.contains(&profile_id)
                {
                    return None;
                }
                // (R) Reuse Profiles' existing local rank curve: one base rank,
                // one rank per 100 XP, capped at rank 20. Native quest gates do
                // not establish this XP-to-rank mapping.
                let xp = state.experience.get(&profile_id).copied().unwrap_or(0);
                Some((job_id.to_owned(), (1 + xp / 100).min(20) as u8))
            })
            .collect();
        let limits = &self.game_rules.policy.quest_limits;
        crate::quest_engine::CharacterEligibility {
            // (R) No separate avatar level is saved; use the active job's local rank curve.
            player_level: profile_state.map_or(1, |state| self.job_level(state.active) as u8),
            job_levels,
            member: self.game_rules.membership.is_active_at(now_unix_seconds),
            // (R) The client reads both quest-limit values from self data, but the historical
            // numbers are unknown; retain the existing permissive fallback until configured.
            nonmember_quest_limit: limits
                .nonmember
                .map_or(100, |limit| usize::try_from(limit).unwrap_or(usize::MAX)),
            member_quest_limit: limits
                .member
                .map_or(100, |limit| usize::try_from(limit).unwrap_or(usize::MAX)),
        }
    }

    fn refresh_npc_quest_eligibility(&mut self, now_unix_seconds: u64) {
        let eligibility = self.character_eligibility(now_unix_seconds);
        if let Some(engine) = &mut self.npc_quests {
            engine.set_eligibility(eligibility);
        }
    }

    pub fn exclude_startup_npcs(&mut self, actor_guids: impl IntoIterator<Item = u64>) {
        if let Some(visibility) = &mut self.npc_visibility {
            visibility.exclude_actor_guids(actor_guids);
        }
    }

    pub fn update_npc_visibility(
        &mut self,
        character: &CharacterState,
    ) -> io::Result<Vec<Vec<u8>>> {
        let packets = self
            .npc_visibility
            .as_mut()
            .map(|visibility| visibility.update(character.zone_id, character.position))
            .unwrap_or_else(|| Ok(Vec::new()))?;
        if !packets.is_empty() {
            npc_event(serde_json::json!({
                "event": "npc_visibility",
                "zone_id": character.zone_id,
                "position": character.position,
                "added": packets.iter().filter(|p| p.starts_with(&[35, 0, 2, 0])).count(),
                "removed": packets.iter().filter(|p| p.starts_with(&[35, 0, 3, 0])).count(),
            }));
        }
        Ok(packets)
    }

    fn content_npc_key(&self, runtime_guid: u64) -> Option<u64> {
        if self.opening.as_ref().is_some_and(|opening| {
            opening
                .actors()
                .iter()
                .any(|actor| actor.guid == runtime_guid)
        }) {
            return None;
        }
        self.npc_visibility
            .as_ref()
            .and_then(|visibility| visibility.content_key(runtime_guid))
    }

    fn handle_npc_packet(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
    ) -> io::Result<Option<Outcome>> {
        self.refresh_npc_quest_eligibility(current_unix_seconds());
        // Overworld enemies answer interact/select with the encounter's Attack
        // menu; Legacy also lists their GUIDs as ordinary NPCs (run 002115).
        if (inner.starts_with(&[26, 0, 8, 0]) || inner.starts_with(&[26, 0, 10, 0]))
            && inner.len() >= 12
            && self.encounter.as_ref().is_some_and(|encounter| {
                encounter.is_field_actor(u64::from_le_bytes(inner[4..12].try_into().unwrap()))
            })
        {
            return Ok(None);
        }
        if inner.starts_with(&[26, 0, 6, 0]) {
            let selected = npc_interaction::decode_dialogue_response_26_6(inner)
                .map_err(|_| invalid("invalid dialogue response"))?;
            npc_event(serde_json::json!({"event": "dialogue_choice", "value": selected}));
            // (R) The choice reply carries the row's response id (or its index);
            // the quest row is first, so accept either form.
            if selected == npc_interaction::QUEST_CHOICE_RESPONSE || selected == 0 {
                if let Some((runtime_guid, key, quest_id)) = self.pending_quest_choice.take() {
                    let held = std::mem::take(&mut self.pending_choice_packets);
                    if !held.is_empty() {
                        npc_event(
                            serde_json::json!({"event": "quest_end_from_choice", "quest_id": quest_id}),
                        );
                        return Ok(Some(Outcome {
                            replies: held,
                            ..Outcome::default()
                        }));
                    }
                    let mut out = Outcome::default();
                    if let Some(engine) = self.npc_quests.as_mut() {
                        let result = engine.offer(quest_id, key);
                        self.append_npc_quest_outcome(result, &mut out)?;
                    }
                    for packet in &mut out.replies {
                        set_dialog_camera_target(packet, runtime_guid);
                    }
                    npc_event(
                        serde_json::json!({"event": "quest_offer_from_choice", "quest_id": quest_id}),
                    );
                    return Ok(Some(out));
                }
            }
            self.pending_quest_choice = None;
            self.pending_choice_packets.clear();
            if let Some(interaction) = &self.npc_interaction {
                interaction
                    .lock()
                    .map_err(|_| invalid("NPC interaction state is unavailable"))?
                    .close_dialogue();
            }
            return Ok(Some(Outcome {
                replies: vec![npc_interaction::encode_dialogue_close_26_4()],
                ..Outcome::default()
            }));
        }
        if inner.starts_with(&[26, 0, 4, 0]) {
            if inner.len() != 4 {
                return Err(invalid("invalid dialogue close request"));
            }
            // (R) Also accept an inbound empty 26/4 if the client mirrors EndDialog.
            if let Some(interaction) = &self.npc_interaction {
                interaction
                    .lock()
                    .map_err(|_| invalid("NPC interaction state is unavailable"))?
                    .close_dialogue();
            }
            return Ok(Some(Outcome::default()));
        }
        // 45/7: the client says a goal it judges itself is met (W175). Only an open goal of
        // the character's own active quest can move; anything else changes nothing (R).
        if let Some(client_id) = crate::quest_engine::decode_client_complete(inner)
            .map_err(|_| invalid("invalid client objective completion"))?
        {
            let mut out = Outcome::default();
            self.npc_quest_event(
                crate::quest_engine::QuestEvent::ClientReported { client_id },
                &mut out,
            )?;
            npc_event(serde_json::json!({
                "event": "client_objective",
                "outcome": if out.changed { "advanced" } else { "ignored" }
            }));
            return Ok(Some(out));
        }
        if let Some(request) = crate::quest_journal::decode(inner)
            .map_err(|_| invalid("invalid quest journal request"))?
        {
            // 49/15 names an ORIGINAL quest id; an id the crosswalk does not list gets one
            // chat line and changes nothing (R).
            let Some(request) =
                crate::quest_journal::translate(request, &self.breadcrumb_crosswalk)
            else {
                npc_event(serde_json::json!({"event": "quest_journal", "outcome": "unlisted"}));
                return Ok(Some(Outcome {
                    replies: vec![chat::system(crate::quest_journal::BREADCRUMB_REFUSED)],
                    ..Outcome::default()
                }));
            };
            let Some(engine) = self.npc_quests.as_mut() else {
                return Ok(Some(Outcome::default()));
            };
            let breadcrumb = matches!(
                request,
                crate::quest_journal::JournalRequest::Breadcrumb { .. }
            );
            let result = crate::quest_journal::handle(engine, request, &self.breadcrumb_quests);
            let refused = breadcrumb && result.denial.is_some() && result.outbound.is_empty();
            npc_event(serde_json::json!({
                "event": "quest_journal",
                "request": format!("{request:?}"),
                "outcome": if result.changed {
                    "changed"
                } else if result.denial.is_some() {
                    "rejected"
                } else {
                    "unchanged"
                }
            }));
            let mut out = Outcome::default();
            self.append_npc_quest_outcome(result, &mut out)?;
            if refused {
                out.replies
                    .push(chat::system(crate::quest_journal::BREADCRUMB_REFUSED));
            }
            return Ok(Some(out));
        }
        if inner.starts_with(&[49, 0, 2, 0]) {
            let (quest_id, accept) = npc_interaction::decode_quest_reply_49_2(inner)
                .map_err(|_| invalid("invalid quest accept/decline reply"))?;
            let Some(engine) = self.npc_quests.as_mut() else {
                npc_event(serde_json::json!({
                    "event": "quest_reply",
                    "quest_id": quest_id,
                    "accepted": accept,
                    "outcome": "unavailable"
                }));
                return Ok(Some(Outcome::default()));
            };
            let result = if accept {
                engine.accept(quest_id)
            } else {
                engine.decline(quest_id)
            };
            npc_event(serde_json::json!({
                "event": "quest_reply",
                "quest_id": quest_id,
                "accepted": accept,
                "outcome": if result.changed {
                    if accept { "accepted" } else { "declined" }
                } else {
                    "rejected"
                }
            }));
            let mut out = Outcome::default();
            self.append_npc_quest_outcome(result, &mut out)?;
            // End the NPC dialog so the client leaves dialog camera and restores
            // the HUD (native run 200411 stayed in dialog view after accepting).
            out.replies.push(END_DIALOG_26_4.to_vec());
            return Ok(Some(out));
        }
        if inner.starts_with(&[49, 0, 14, 0]) {
            let quest_id = npc_interaction::decode_quest_end_reply_49_14(inner)
                .map_err(|_| invalid("invalid quest end reply"))?;
            let Some(engine) = &self.npc_quests else {
                return Ok(Some(Outcome::default()));
            };
            let profile_state = self
                .profiles
                .as_ref()
                .map(|profiles| profiles.state().clone())
                .unwrap_or_else(|| ProfileState {
                    version: 1,
                    active: 1,
                    unlocked: BTreeSet::from([1]),
                    experience: Default::default(),
                });
            // (R) Preserve pre-existing stack counts even if the reconstructed 100-slot
            // limit is lower; any new slot still has to fit within that existing floor.
            let bag_capacity = self.inventory.state().items.len().max(100);
            let reward_state =
                RewardState::new(self.inventory.clone(), profile_state.clone(), bag_capacity);
            let mut staged_engine = engine.clone();
            let (quest, next_state) = staged_engine.finish_turnin(quest_id, &reward_state)?;
            let next_profiles = if self.profiles.is_some() {
                if next_state.profiles == profile_state {
                    None
                } else {
                    let directory = self
                        .profile_content_directory
                        .as_deref()
                        .ok_or_else(|| invalid("quest XP needs profile content"))?;
                    Some(Profiles::load(
                        directory,
                        Some(next_state.profiles.clone()),
                    )?)
                }
            } else if next_state.profiles != profile_state {
                return Err(invalid("quest XP needs character profile state"));
            } else {
                None
            };

            self.npc_quests = Some(staged_engine);
            self.inventory = next_state.inventory;
            if let Some(profiles) = next_profiles {
                self.profiles = Some(profiles);
            }
            self.refresh_npc_quest_eligibility(current_unix_seconds());
            let mut out = Outcome::default();
            self.append_npc_quest_outcome(quest, &mut out)?;
            out.replies.push(END_DIALOG_26_4.to_vec());
            return Ok(Some(out));
        }
        if inner.starts_with(&[26, 0, 23, 0, 0, 0]) {
            if inner.len() != 10 {
                return Err(invalid("invalid quest abandon request"));
            }
            let quest_id = u32::from_le_bytes(inner[6..10].try_into().unwrap());
            let mut out = Outcome::default();
            let result = self
                .npc_quests
                .as_mut()
                .map(|engine| engine.abandon(quest_id));
            if let Some(result) = result {
                self.append_npc_quest_outcome(result, &mut out)?;
            }
            return Ok(Some(out));
        }
        if inner.starts_with(&[26, 0, 8, 0]) {
            let request = npc_interaction::decode_npc_interact_26_8(inner)
                .map_err(|_| invalid("invalid NPC interaction request"))?;
            let Some(key) = self.content_npc_key(request.target) else {
                npc_event(serde_json::json!({
                    "event": "npc_interact",
                    "guid": request.target,
                    "content_key": serde_json::Value::Null,
                    "outcome": "none",
                    "reason": "unknown_runtime_guid"
                }));
                return Ok(None);
            };
            npc_event(serde_json::json!({
                "event": "npc_interact",
                "guid": request.target,
                "content_key": key
            }));
            let Some(interaction) = &self.npc_interaction else {
                npc_event(serde_json::json!({
                    "event": "npc_menu",
                    "guid": request.target,
                    "content_key": key,
                    "options": [],
                    "outcome": "none",
                    "reason": "npc_content_unavailable"
                }));
                return Ok(None);
            };
            let mut interaction = interaction
                .lock()
                .map_err(|_| invalid("NPC interaction state is unavailable"))?;
            let menu = interaction
                .interact(
                    key,
                    character.zone_id,
                    character.position,
                    self.npc_quests.as_ref(),
                )
                .map_err(|_| invalid("NPC interaction is unavailable"))?;
            let options: Vec<_> = menu
                .options
                .iter()
                .map(|option| {
                    let action = match option.action {
                        npc_interaction::NpcAction::Talk => "talk",
                        npc_interaction::NpcAction::TurnInQuest { .. } => "turn_in_quest",
                        npc_interaction::NpcAction::Shop => "shop",
                        npc_interaction::NpcAction::Race => "race",
                        npc_interaction::NpcAction::Derby => "derby",
                        npc_interaction::NpcAction::Fish => "fish",
                        npc_interaction::NpcAction::Learn { .. } => "learn",
                    };
                    serde_json::json!({
                        "id": option.id,
                        "action": action,
                        "string_id": option.string_id
                    })
                })
                .collect();
            npc_event(serde_json::json!({
                "event": "npc_menu",
                "guid": request.target,
                "content_key": key,
                "options": options
            }));
            let mut packet = menu.packet;
            // (R) The client request target is treated as the runtime actor GUID; the content
            // service uses the stable regional NPC key internally.
            packet[4..12].copy_from_slice(&request.target.to_le_bytes());
            return Ok(Some(Outcome {
                replies: vec![packet],
                ..Outcome::default()
            }));
        }

        if inner.starts_with(&[26, 0, 10, 0]) {
            if inner.len() != 16 {
                return Err(invalid("invalid NPC menu selection"));
            }
            let runtime_guid = u64::from_le_bytes(inner[4..12].try_into().unwrap());
            let option_id = i32::from_le_bytes(inner[12..16].try_into().unwrap());
            let Some(key) = self.content_npc_key(runtime_guid) else {
                npc_event(serde_json::json!({
                    "event": "npc_select",
                    "guid": runtime_guid,
                    "content_key": serde_json::Value::Null,
                    "option": option_id,
                    "outcome": "none",
                    "reason": "unknown_runtime_guid"
                }));
                return Ok(None);
            };
            let Some(interaction) = &self.npc_interaction else {
                npc_event(serde_json::json!({
                    "event": "npc_select",
                    "guid": runtime_guid,
                    "content_key": key,
                    "option": option_id,
                    "outcome": "none",
                    "reason": "npc_content_unavailable"
                }));
                return Ok(None);
            };
            let mut selection = inner.to_vec();
            selection[4..12].copy_from_slice(&key.to_le_bytes());
            let mut interaction = interaction
                .lock()
                .map_err(|_| invalid("NPC interaction state is unavailable"))?;
            let selected = interaction.select(
                &selection,
                character.zone_id,
                character.position,
                self.npc_quests.as_mut(),
            );
            let mut out = Outcome::default();
            let mut race_gate = None;
            let mut derby_gate = None;
            let mut fish_gate = None;
            let mut learn_trainer = None;
            match selected {
                Ok(NpcSelection::Talk(mut context)) => {
                    if let Some(engine) = &mut self.npc_quests {
                        let quest = engine
                            .on_event(crate::quest_engine::QuestEvent::TalkToNpc { npc_id: key })?;
                        // A completed talk step speaks its own line (Shakey's bubble was
                        // blank in native run 215840).
                        if let Some(line) = engine.step_dialogue_for(&quest) {
                            context.body_string_id = Some(line);
                        }
                        self.append_npc_quest_outcome(quest, &mut out)?;
                    }
                    let target_quest = self
                        .npc_quests
                        .as_ref()
                        .and_then(|engine| interaction.named_target_quest_for_talk(key, engine));
                    if let Some(quest_id) = target_quest {
                        let result = self
                            .npc_quests
                            .as_mut()
                            .unwrap()
                            .begin_named_target_turnin(quest_id);
                        self.append_npc_quest_outcome(result, &mut out)?;
                    }
                    let already_offering = out
                        .replies
                        .iter()
                        .any(|packet| packet.starts_with(&[49, 0, 1, 0]));
                    let turning_in = out
                        .replies
                        .iter()
                        .any(|packet| packet.starts_with(&[49, 0, 13, 0]));
                    self.pending_choice_packets.clear();
                    let offer_outcome = if already_offering {
                        "offer"
                    } else if turning_in {
                        // Same shape as the offer: the 49/13 end screen opens from a
                        // dialog choice (run 202324 drew it over the open dialog).
                        let (held, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut out.replies)
                            .into_iter()
                            .partition(|packet| packet.starts_with(&[49, 0, 13, 0]));
                        out.replies = rest;
                        let quest_id = held
                            .first()
                            .and_then(|packet| packet.get(14..18))
                            .map(|id| u32::from_le_bytes(id.try_into().unwrap()));
                        // Offset 6 is an actor GUID, not the quest id: ShowQuestEndScreen
                        // looks it up in the actor table (00984960) to expand
                        // [*player*] in the end text and to play the NPC animation.
                        // With the quest id there, run 224000 showed "[*player*]" literally.
                        let mut held = held;
                        for packet in &mut held {
                            if packet.len() >= 14 {
                                packet[6..14].copy_from_slice(&runtime_guid.to_le_bytes());
                            }
                        }
                        match quest_id {
                            Some(quest_id) => {
                                self.pending_quest_choice = Some((runtime_guid, key, quest_id));
                                self.pending_choice_packets = held;
                                "quest_choice"
                            }
                            None => {
                                out.replies.extend(held);
                                "dialogue"
                            }
                        }
                    } else {
                        // (R) The client leaves the 26/10-to-quest action unresolved; W67 chooses
                        // Talk as the server action that starts the separate 49/1 quest window.
                        let quest_id = self
                            .npc_quests
                            .as_ref()
                            .and_then(|engine| interaction.offerable_quest_for_talk(key, engine));
                        // Native runs 200411/201127: the offer window must sit on an
                        // open NPC dialog, and the server's 26/4 EndDialog after the
                        // reply restores camera and HUD. The quest is therefore a
                        // dialog choice; choosing it (26/6) sends the 49/1 offer.
                        if let Some(quest_id) = quest_id {
                            self.pending_quest_choice = Some((runtime_guid, key, quest_id));
                            "quest_choice"
                        } else {
                            "dialogue"
                        }
                    };
                    let quest_choice = match (offer_outcome, self.pending_quest_choice) {
                        ("quest_choice", Some((_, _, quest_id))) => {
                            let title = self
                                .npc_quests
                                .as_ref()
                                .and_then(|engine| engine.title_string_id(quest_id))
                                .unwrap_or(0);
                            Some(npc_interaction::quest_dialogue_choice(title))
                        }
                        _ => None,
                    };
                    let mut dialogue = dialogue_packet_with(&context, quest_choice)
                        .map_err(|_| invalid("NPC dialogue could not be encoded"))?;
                    set_dialog_camera_target(&mut dialogue, runtime_guid);
                    self.last_npc_dialogue =
                        Some((character.zone_id, runtime_guid, key, dialogue.clone()));
                    // A quest offer is shown alone, like the original: the 49/1 window
                    // draws its own speech bubble. (Earlier runs were silently ignored
                    // because 49/1 was 120 bytes, not for lack of a 26/3; a leading 26/3
                    // left a blank bubble and "Goodbye" behind the quest window.)
                    if offer_outcome != "offer" {
                        out.replies.push(dialogue);
                    }
                    let reason = if offer_outcome == "offer" {
                        None
                    } else if turning_in {
                        Some("quest_turnin")
                    } else if self.npc_quests.is_none() {
                        Some("quest_engine_unavailable")
                    } else {
                        Some("no_eligible_quest")
                    };
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": offer_outcome,
                        "reason": reason
                    }));
                }
                Ok(NpcSelection::Shop(window)) => {
                    out.replies.push(
                        merchant_packet(&window)
                            .map_err(|_| invalid("NPC merchant list could not be encoded"))?,
                    );
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": "shop"
                    }));
                    npc_event(serde_json::json!({
                        "event": "shop_open",
                        "guid": runtime_guid,
                        "content_key": key,
                        "item_count": window.items.len()
                    }));
                }
                Ok(NpcSelection::Race) => {
                    race_gate = Some(key);
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": "race"
                    }));
                }
                Ok(NpcSelection::Derby) => {
                    derby_gate = Some(key);
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": "derby"
                    }));
                }
                Ok(NpcSelection::Fish) => {
                    fish_gate = Some(key);
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": "fish"
                    }));
                }
                Ok(NpcSelection::Learn(trainer)) => {
                    learn_trainer = Some(trainer);
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": "learn"
                    }));
                }
                Ok(NpcSelection::Quest(quest)) => {
                    let has_offer = quest.outbound.iter().any(|message| {
                        matches!(
                            message,
                            crate::quest_engine::OutboundMessage::Quest(
                                crate::quest_engine::QuestMessage::QuestInfo { .. }
                            )
                        )
                    });
                    self.append_npc_quest_outcome(quest, &mut out)?;
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": if has_offer { "offer" } else { "none" },
                        "reason": if has_offer { serde_json::Value::Null } else { serde_json::json!("quest_turnin") }
                    }));
                }
                Err(crate::npc_interaction::NpcInteractionError::NeedsQuestEngine) => {
                    out.replies
                        .push(chat::system("Quest data is unavailable in this area."));
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": "none",
                        "reason": "quest_engine_unavailable"
                    }));
                }
                Err(_) => {
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": "none",
                        "reason": "selection_rejected"
                    }));
                    return Err(invalid("NPC menu choice is unavailable"));
                }
            }
            drop(interaction);
            if let Some(gate) = race_gate {
                // (R) Ordinary entry: the NPC service proved range and that this exact option
                // was offered; the kart service decides admission. Staged, so a failure after
                // the job switch leaves the player exactly as they were.
                let busy = self.activity_busy();
                match self.kart_entry.as_ref().map(|kart| kart.admit(gate, busy)) {
                    Some(Ok(())) => {
                        let mut staged = self.clone();
                        out = staged.begin_kart_race(character)?;
                        *self = staged;
                    }
                    Some(Err(denied)) => out.replies.push(chat::system(denied.message())),
                    None => out.replies.push(chat::system(
                        crate::kart_gateway::EntryDenied::NoRace.message(),
                    )),
                }
            }
            if let Some(gate) = derby_gate {
                // (R) The NPC service proved range and that this exact option was offered;
                // the derby service decides admission. Staging keeps the job switch atomic.
                let busy = self.activity_busy();
                match self.derby.as_ref().map(|derby| derby.admit(gate, busy)) {
                    Some(Ok(())) => {
                        let mut staged = self.clone();
                        out = staged.begin_derby(character)?;
                        *self = staged;
                    }
                    Some(Err(denied)) => out.replies.push(chat::system(denied.message())),
                    None => out.replies.push(chat::system(
                        crate::derby_gateway::DerbyEntryDenied::Unavailable.message(),
                    )),
                }
            }
            if let Some(trainer) = learn_trainer {
                // (R) Ordinary unlock: the NPC service proved range and that this exact option
                // was offered; `learn_job` decides the rest.
                let mut staged = self.clone();
                out = staged.learn_job(&trainer)?;
                // Leave the NPC dialog camera first (see the fishing choice).
                out.replies.insert(0, END_DIALOG_26_4.to_vec());
                if out.changed {
                    *self = staged;
                }
            }
            if let Some(gate) = fish_gate {
                // (R) Ordinary entry: the NPC service proved range and that this exact option
                // was offered; the fishing service decides admission. Staged, so a refusal
                // leaves the player exactly as they were.
                let mut staged = self.clone();
                out = staged.begin_fishing(gate, character)?;
                // Leave the NPC dialog camera first (it otherwise outlives a menu choice, as
                // the quest accept showed in native run 200411).
                out.replies.insert(0, END_DIALOG_26_4.to_vec());
                if out.changed {
                    *self = staged;
                }
            }
            return Ok(Some(out));
        }

        // 165/4 buy and 165/5 sell, plus the siblings (W172): 2 definitions, 8 list update,
        // 12 buy-back, 14 gift (always refused).
        if matches!(inner, [0xa5, 0, 2 | 4 | 5 | 8 | 12 | 14, 0, ..]) {
            if let Some(shop) = &mut self.npc_coin_shop {
                let Some(interaction) = &self.npc_interaction else {
                    return Ok(None);
                };
                let mut interaction = interaction
                    .lock()
                    .map_err(|_| invalid("NPC interaction state is unavailable"))?;
                let Some(visibility) = &self.npc_visibility else {
                    return Ok(None);
                };
                // Only buys and sells name a merchant; 165/8 is 4 bytes and 165/2 and 165/12
                // carry other fields at 4..12, so those are never rewritten (W172).
                let merchant = inner
                    .get(4..12)
                    .map_or(0, |bytes| u64::from_le_bytes(bytes.try_into().unwrap()));
                let mut request = inner.to_vec();
                if matches!(inner[2], 4 | 5) {
                    if let Some(key) = visibility.content_key(merchant) {
                        request[4..12].copy_from_slice(&key.to_le_bytes());
                    }
                }
                let content_key = request
                    .get(4..12)
                    .map_or(0, |bytes| u64::from_le_bytes(bytes.try_into().unwrap()));
                // The shop was built with the login-time inventory; without this
                // a purchase overwrote everything earned since (10-15 audit).
                shop.sync_inventory(self.inventory.clone());
                let outcome = interaction.handle_coin_store(
                    shop,
                    &request,
                    character.zone_id,
                    character.position,
                );
                match outcome {
                    Ok(outcome) => {
                        npc_event(serde_json::json!({
                            "event": "shop_txn",
                            "guid": merchant,
                            "content_key": content_key,
                            "subtype": u16::from_le_bytes(inner[2..4].try_into().unwrap()),
                            "outcome": "accepted"
                        }));
                        self.inventory = shop.inventory().clone();
                        return Ok(Some(Outcome {
                            replies: outcome.replies,
                            changed: outcome.changed,
                            ..Outcome::default()
                        }));
                    }
                    Err(error) => {
                        npc_event(serde_json::json!({
                            "event": "shop_txn",
                            "guid": merchant,
                            "content_key": content_key,
                            "subtype": u16::from_le_bytes(inner[2..4].try_into().unwrap()),
                            "outcome": "rejected",
                            "reason": format!("{error:?}")
                        }));
                        return Err(invalid("NPC Coin Store transaction was rejected"));
                    }
                }
            }
        }

        // Marketplace siblings (W172): wallet, status, SKU and code lists, account info, and the
        // refused order rows. Stateless and payment-free; `handle_marketplace` replies or errors.
        if matches!(
            inner,
            [
                66,
                0,
                1 | 3 | 10 | 12 | 14 | 16 | 18 | 20 | 22 | 25 | 27 | 38,
                0,
                ..
            ]
        ) {
            let Some(shop) = &mut self.npc_coin_shop else {
                return Ok(None);
            };
            let outcome = shop.handle_marketplace(inner)?;
            return Ok(Some(Outcome {
                replies: outcome.replies,
                ..Outcome::default()
            }));
        }

        if inner.starts_with(&[66, 0, 43, 0]) {
            let Ok(request) = npc_interaction::decode_merchant_buy_66_43(inner) else {
                return Ok(Some(Outcome {
                    replies: vec![crate::coin_shop::merchant_buy_result_packet(false)],
                    ..Outcome::default()
                }));
            };
            // (R) The Lua callsite uses its first argument as the item key, its fifth as
            // quantity, and the generated request id as the transaction key. Client price is
            // ignored; stores.json supplies the authoritative local Coin price.
            let item_id = i32::try_from(request.initial_dwords[0]).unwrap_or(-1);
            let quantity = request.trailing_dwords[0];
            let transaction_id = request.trailing_dwords[2];
            let Some(interaction) = &self.npc_interaction else {
                return Ok(Some(Outcome {
                    replies: vec![crate::coin_shop::merchant_buy_result_packet(false)],
                    ..Outcome::default()
                }));
            };
            let Ok(interaction) = interaction.lock() else {
                return Ok(Some(Outcome {
                    replies: vec![crate::coin_shop::merchant_buy_result_packet(false)],
                    ..Outcome::default()
                }));
            };
            let refusal_code = interaction
                .validate_merchant_item_66_43(item_id, character.zone_id, character.position)
                .err()
                .map(|error| {
                    if error == npc_interaction::NpcInteractionError::InvalidOption {
                        crate::coin_shop::RESULT_INVALID_ITEM
                    } else {
                        crate::coin_shop::RESULT_INVALID_MERCHANT
                    }
                });
            let Some(shop) = &mut self.npc_coin_shop else {
                return Ok(Some(Outcome {
                    replies: vec![crate::coin_shop::merchant_buy_result_packet(false)],
                    ..Outcome::default()
                }));
            };
            shop.sync_inventory(self.inventory.clone());
            let outcome = if let Some(result_code) = refusal_code {
                shop.refuse_vendor_buy(transaction_id, result_code)
            } else {
                match shop.buy_vendor(transaction_id, item_id, quantity) {
                    Ok(outcome) => outcome,
                    Err(_) => shop.refuse_vendor_buy(
                        transaction_id,
                        crate::coin_shop::RESULT_SERVICE_UNAVAILABLE,
                    ),
                }
            };
            self.inventory = shop.inventory().clone();
            npc_event(serde_json::json!({
                "event": "shop_txn",
                "subtype": 43,
                "item_id": item_id,
                "quantity": quantity,
                "outcome": if outcome.result_code == crate::coin_shop::RESULT_SUCCESS {
                    "accepted"
                } else {
                    "refused"
                }
            }));
            return Ok(Some(Outcome {
                replies: outcome.replies,
                changed: outcome.changed,
                ..Outcome::default()
            }));
        }

        Ok(None)
    }

    fn append_npc_quest_outcome(
        &self,
        quest: crate::quest_engine::QuestOutcome,
        out: &mut Outcome,
    ) -> io::Result<()> {
        out.changed |= quest.changed;
        for message in quest.outbound {
            match message {
                crate::quest_engine::OutboundMessage::Quest(
                    crate::quest_engine::QuestMessage::QuestInfo { quest_id },
                ) => {
                    let engine = self
                        .npc_quests
                        .as_ref()
                        .ok_or_else(|| invalid("NPC quest engine is unavailable"))?;
                    out.replies.push(
                        quest_offer_packet(engine, quest_id)
                            .map_err(|_| invalid("NPC quest offer could not be encoded"))?,
                    );
                    npc_event(serde_json::json!({
                        "event": "quest_offer",
                        "quest_id": quest_id
                    }));
                }
                crate::quest_engine::OutboundMessage::Quest(
                    crate::quest_engine::QuestMessage::QuestAdd { quest_id },
                ) => {
                    let packet = self
                        .npc_quests
                        .as_ref()
                        .ok_or_else(|| invalid("NPC quest engine is unavailable"))?
                        .quest_add_packet(quest_id)?;
                    out.replies.push(packet);
                }
                crate::quest_engine::OutboundMessage::Quest(
                    crate::quest_engine::QuestMessage::QuestEnd { quest_id },
                ) => {
                    let packet = self
                        .npc_quests
                        .as_ref()
                        .ok_or_else(|| invalid("NPC quest engine is unavailable"))?
                        .quest_end_packet(quest_id)?;
                    out.replies.push(packet);
                }
                crate::quest_engine::OutboundMessage::Quest(message) => {
                    if let Some(packet) = message.encode_exact() {
                        out.replies.push(packet);
                    }
                }
                crate::quest_engine::OutboundMessage::Reward(packet) => out.replies.push(packet),
            }
        }
        Ok(())
    }

    fn npc_quest_event(
        &mut self,
        event: crate::quest_engine::QuestEvent,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(engine) = &mut self.npc_quests {
            let result = engine.on_event(event)?;
            self.append_npc_quest_outcome(result, out)?;
        }
        Ok(())
    }

    fn npc_location_event(
        &mut self,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        // (R) Scene entry is an authoritative location event. Area and reach
        // objectives must advance without waiting for the first movement packet.
        self.npc_quest_event(
            crate::quest_engine::QuestEvent::VisitPosition {
                zone_id: character.zone_id,
                position: character.position,
            },
            out,
        )
    }

    // (R) Collect counts newly granted, non-rental items, not login snapshots
    // or existing stock. Equip counts changed owned slots, not packet claims.
    fn npc_inventory_events(
        &mut self,
        before: &InventoryState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if self.npc_quests.is_none() {
            return Ok(());
        }
        let totals = |state: &InventoryState| {
            let mut counts = std::collections::BTreeMap::<i32, u64>::new();
            for item in state.items.iter().filter(|item| item.rental.is_none()) {
                *counts.entry(item.definition_id).or_default() += u64::from(item.count);
            }
            counts
        };
        let previous = totals(before);
        for (item_id, total) in totals(self.inventory.state()) {
            let quantity = total
                .saturating_sub(*previous.get(&item_id).unwrap_or(&0))
                .min(1_000_000) as u32;
            if quantity > 0 {
                self.npc_quest_event(
                    crate::quest_engine::QuestEvent::CollectItem { item_id, quantity },
                    out,
                )?;
            }
        }
        let equipped: BTreeSet<_> = self
            .inventory
            .state()
            .equipment
            .iter()
            .filter(|slot| slot.item_id != 0 && !before.equipment.contains(slot))
            .filter_map(|slot| {
                self.inventory
                    .state()
                    .items
                    .iter()
                    .find(|item| item.id == slot.item_id && item.rental.is_none())
                    .map(|item| item.definition_id)
            })
            .collect();
        for definition_id in equipped {
            self.npc_quest_event(
                crate::quest_engine::QuestEvent::EquipItem { definition_id },
                out,
            )?;
        }
        Ok(())
    }

    // Only inspect server-generated results after a successful payout. Never
    // interpret an incoming win/score packet as quest authority (R policy).
    fn npc_minigame_win(&mut self, replies: &[Vec<u8>], out: &mut Outcome) -> io::Result<()> {
        if let Some(packet) = replies.iter().find(|p| {
            (p.len() >= 16 && p.starts_with(&[39, 0, 18]) && p[15] == 1)
                || (p.len() >= 19 && p.starts_with(&[39, 0, 14]) && &p[19..] == b"OnGameWonMsg\0")
        }) {
            let id = i32::from_le_bytes(packet[11..15].try_into().unwrap());
            if id > 0 {
                self.npc_quest_event(
                    crate::quest_engine::QuestEvent::FinishMinigame {
                        minigame_id: id as u32,
                    },
                    out,
                )?;
            }
        }
        Ok(())
    }

    fn append_npc_quest_login_sync(&mut self, out: &mut Outcome) -> io::Result<()> {
        if self.npc_quests_synced {
            return Ok(());
        }
        let Some(engine) = self.npc_quests.as_ref() else {
            return Ok(());
        };
        let opening_ids = self
            .opening
            .as_ref()
            .map(|opening| {
                opening
                    .journal_entries()
                    .into_iter()
                    .map(|entry| entry.quest_id)
                    .collect()
            })
            .unwrap_or_default();
        out.replies
            .extend(engine.active_login_packets(&opening_ids)?);
        self.npc_quests_synced = true;
        Ok(())
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
    /// which the client applies as a profile-swap attachment group.
    fn profile_switch_packet(&self, guid: u64) -> io::Result<Option<Vec<u8>>> {
        let Some(profiles) = &self.profiles else {
            return Ok(None);
        };
        let mut packet = vec![35, 0, 7, 0];
        packet.extend(guid.to_le_bytes());
        packet.extend(self.inventory.profile_attachments(profiles.active())?);
        packet.extend(0i32.to_le_bytes()); // (R) trailing value; the client reads one i32.
        Ok(Some(packet))
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
            profiles: self.profiles.as_ref().map(|p| p.state().clone()),
            identity_fingerprint: self.identity_fingerprint.clone(),
            opening: self.opening.as_ref().map(|o| o.state().clone()),
            encounter: self.encounter.as_ref().map(|e| e.state().clone()),
            smelting: self.smelting.as_ref().map(|s| s.state().clone()),
            forging: self.forging.as_ref().map(|s| s.state().clone()),
            cooking: self.cooking.as_ref().map(|s| s.state().clone()),
            mining: self.mining.as_ref().map(|m| m.state().clone()),
            harvesting: self.harvesting.as_ref().map(|h| h.state().clone()),
            kart_entry: self.kart_entry.as_ref().map(|k| k.state().clone()),
            derby: self
                .derby
                .as_ref()
                .and_then(crate::derby_gateway::DerbyGateway::snapshot),
            warpstones: Some(self.warp.state()).filter(|s| !s.is_empty()),
            item_bar: Some(self.item_bar.clone()).filter(|b| !b.is_empty()),
            progression_extras,
        }
    }

    /// W178: trusted store/event code queues a server-owned claim. No client
    /// request reaches this method or supplies its reward payload.
    pub fn issue_claim(
        &mut self,
        claim_id: u64,
        grant: crate::rewards::RewardGrant,
        client_record: Vec<u8>,
    ) -> io::Result<()> {
        self.progression_extras
            .issue_server_grant(claim_id, grant, client_record)
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
            // (R) The tutorial Copper Bar sample has no recovered recipe row/id.
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
        // Derby state is connection-local (R): an unfinished match is dropped and its job switch
        // is restored from the prior job. Paid closes have already committed their inventory.
        if let Some(job) = self.abandon_derby_for_reconnect() {
            if let Some(profiles) = &mut self.profiles {
                let _ = profiles.select(job, &self.inventory);
            }
        }
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
            let _ = profiles.select(job, &self.inventory);
        }
        location
    }

    /// A derby's wire processor cannot resume after a reconnect. Drop the connection-local match
    /// and return the job that was active before entry.
    pub fn abandon_derby_for_reconnect(&mut self) -> Option<i32> {
        self.derby.as_mut()?.exit()
    }

    /// Grant and equip a job's starter outfit from the content's starter table.
    /// With `reuse_owned`, a piece the player already owns is equipped instead
    /// of granting a duplicate. (R) Pieces the catalogue lacks are skipped.
    fn give_starter_outfit(
        &mut self,
        job: i32,
        reuse_owned: bool,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let before = self.inventory.state().clone();
        let items = self.profiles.as_ref().unwrap().starter_items(job).to_vec();
        let inactive = job != self.profiles.as_ref().unwrap().active();
        for definition in items {
            // The inherited Chef defaults include the same hat that
            // this local story awards at Cooking101 turn-in. Defer only
            // that cosmetic in the versioned Cooking campaign; older
            // campaigns and original profile records stay compatible.
            if job == 45 && definition == 14794 && self.cooking.is_some() {
                continue;
            }
            if !self.inventory.has_definition(definition) {
                continue;
            }
            let owned = if reuse_owned {
                self.inventory.equip_owned_definition(definition, job).ok()
            } else {
                None
            };
            let result = match owned {
                Some(result) => result,
                None => {
                    out.replies
                        .extend(self.inventory.grant(0, definition, 1)?.replies);
                    self.inventory.equip_owned_definition(definition, job)?
                }
            };
            out.changed = true;
            out.replies.extend(
                result
                    .replies
                    .into_iter()
                    .filter(|r| !(inactive && r.starts_with(&[35, 0, 6, 0]))),
            );
        }
        self.npc_inventory_events(&before, out)?;
        Ok(())
    }

    /// (R) A job never stays bare: when a job with a completely empty outfit
    /// becomes active, it gets its starter outfit (the owner, 2026-10-15). Fixes
    /// saves whose job was unlocked by a path that granted no clothes.
    fn ensure_outfit(&mut self, job: i32, out: &mut Outcome) -> io::Result<()> {
        if self
            .inventory
            .profile_attachments(job)?
            .starts_with(&0i32.to_le_bytes())
        {
            self.give_starter_outfit(job, true, out)?;
        }
        let has_weapon = self
            .inventory
            .state()
            .equipment
            .iter()
            .any(|entry| entry.profile_id == job && entry.slot == 7 && entry.item_id != 0);
        let starter = self
            .abilities
            .as_ref()
            .and_then(|abilities| abilities.starter_weapon(job));
        if !has_weapon
            && let Some(definition) = starter
            && self.inventory.has_definition(definition)
        {
            if !self
                .inventory
                .state()
                .items
                .iter()
                .any(|item| item.definition_id == definition)
            {
                out.replies
                    .extend(self.inventory.grant(0, definition, 1)?.replies);
            }
            let equipped = self.inventory.equip_owned_definition(definition, job)?;
            out.changed |= equipped.changed;
            out.replies.extend(equipped.replies);
        }
        Ok(())
    }

    /// (R) Leaving a race puts the player back in the job they had before it;
    /// Kart Driver stays only when chosen or required (the owner, 2026-10-15).
    fn restore_job(&mut self, job: i32, out: &mut Outcome) -> io::Result<()> {
        let profiles = self.profiles.as_mut().unwrap();
        if profiles.active() == job {
            return Ok(());
        }
        out.replies.push(profiles.select(job, &self.inventory)?);
        if let Some(abilities) = &self.abilities {
            out.replies
                .extend(abilities.activation(job, self.job_level(job))?);
        }
        self.sync_player_level(out);
        self.ensure_outfit(job, out)
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
        Ok(out)
    }

    pub fn on_location(&mut self, character: &CharacterState, now_ms: u64) -> io::Result<Outcome> {
        if self.soccer.is_some() {
            return Ok(Outcome::default());
        }
        if self.kart_entry.as_ref().is_some_and(|k| k.active()) {
            return Ok(Outcome::default());
        }
        if self
            .derby
            .as_ref()
            .is_some_and(crate::derby_gateway::DerbyGateway::active)
        {
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
            let mut staged = self.clone();
            let replies = staged.derby.as_mut().unwrap().tick(now_ms)?;
            *self = staged;
            return Ok(Outcome {
                changed: !replies.is_empty(),
                replies,
                ..Default::default()
            });
        }
        if self.kart_entry.as_ref().is_some_and(|k| k.active()) {
            let mut staged = self.clone();
            let kart = staged.kart_entry.as_mut().unwrap();
            let before = kart.state().clone();
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
                self.apply_forging(result, character, &mut out)?;
                self.forging_tick_ms = Some(now_ms);
            }
            return Ok(out);
        }
        if self.mining.as_ref().is_some_and(Type8Engine::active) {
            let previous = self.type8_tick_ms.unwrap_or(now_ms);
            let elapsed = now_ms.saturating_sub(previous);
            // Bound snapshot writes while refreshing the authoritative timer.
            // The gateway persists this state before replies just like input.
            if elapsed >= 250 {
                let result = self
                    .mining
                    .as_mut()
                    .unwrap()
                    .tick(elapsed.min(u32::MAX as u64) as u32)?;
                self.apply_mining(result, &mut out)?;
                self.type8_tick_ms = Some(now_ms);
            }
            return Ok(out);
        }
        if self.harvesting.as_ref().is_some_and(Type8Engine::active) {
            let previous = self.type8_tick_ms.unwrap_or(now_ms);
            let elapsed = now_ms.saturating_sub(previous);
            // Bound snapshot writes while refreshing the authoritative timer.
            // The gateway persists this state before replies just like input.
            if elapsed >= 250 {
                let result = self
                    .harvesting
                    .as_mut()
                    .unwrap()
                    .tick(elapsed.min(u32::MAX as u64) as u32)?;
                self.apply_harvesting(result, &mut out)?;
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
        crate::mounts::RideRestrictions {
            zone_allows_riding: true,
            in_minigame: self.encounter.as_ref().is_some_and(EncounterEngine::active)
                || self.field_engaged
                || self.kart_entry.as_ref().is_some_and(|k| k.active())
                || self.soccer.is_some()
                || self
                    .derby
                    .as_ref()
                    .is_some_and(crate::derby_gateway::DerbyGateway::active),
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
        let busy = crate::pets::PetBusy {
            minigame_active: self.activity_busy()
                || self.kart_entry.as_ref().is_some_and(|k| k.active()),
        };
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
        let mut out = self.handle_in_world_inner(inner, character, now_ms)?;
        self.sync_tracked_target(character, &mut out);
        self.sync_client_objectives(&mut out);
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
            let npcs = self.npc_interaction.as_ref()?.lock().ok()?;
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
        if let Some(mut outcome) = self.handle_npc_packet(inner, character)? {
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
        use crate::harvest_nodes::{Gather, Visitor, payout_fits};
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
        let engine = match gather {
            Gather::Mining => self.mining.as_ref(),
            Gather::Harvesting => self.harvesting.as_ref(),
        };
        let Some(engine) = engine else {
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
        match gather {
            Gather::Mining => {
                let result = staged.mining.as_mut().unwrap().begin(gather.job())?;
                staged.apply_mining(result, &mut out)?;
            }
            Gather::Harvesting => {
                let result = staged.harvesting.as_mut().unwrap().begin(gather.job())?;
                staged.apply_harvesting(result, &mut out)?;
            }
        }
        *self = staged;
        Ok(Some(out))
    }

    /// True while a crafting, gathering or battle activity is running.
    fn activity_busy(&self) -> bool {
        self.forging.as_ref().is_some_and(Forging::active)
            || self.smelting.as_ref().is_some_and(Smelting::active)
            || self.cooking.as_ref().is_some_and(Cooking::active)
            || self.mining.as_ref().is_some_and(Type8Engine::active)
            || self.harvesting.as_ref().is_some_and(Type8Engine::active)
            || self.chess.as_ref().is_some_and(ChessService::is_active)
            || self.encounter.as_ref().is_some_and(EncounterEngine::active)
            || self
                .derby
                .as_ref()
                .is_some_and(crate::derby_gateway::DerbyGateway::active)
            || !self.fishing.is_empty()
            || self.soccer.is_some()
    }

    /// Switches to Kart Driver and enters the track. Shared by the developer `realm race` command
    /// and the ordinary NPC menu choice; the caller has already checked admission and `busy`.
    fn begin_kart_race(&mut self, character: &CharacterState) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        let profiles = self
            .profiles
            .as_mut()
            .ok_or_else(|| invalid("kart entry requires class content"))?;
        // (R) First use still unlocks Kart Driver for saves that predate trainers; the
        // option is idempotent. W176 also offers the job from a trainer. The prior job is
        // restored on exit.
        let previous_job = profiles.active();
        let first_time = profiles.unlock(48)?;
        out.replies.push(profiles.select(48, &self.inventory)?);
        if first_time {
            self.give_starter_outfit(48, false, &mut out)?;
        }
        self.ensure_outfit(48, &mut out)?;
        let kart = self.kart_entry.as_mut().unwrap();
        let (replies, scene) = kart.begin(character)?;
        kart.set_previous_job(previous_job);
        out.replies.extend(replies);
        out.scene_zoning = Some(scene);
        out.changed = true;
        Ok(out)
    }

    /// (R) Switch to Demo Derby Driver and open the native type-11 offer. The caller has already
    /// checked the NPC gate and that no other activity is running.
    fn begin_derby(&mut self, character: &CharacterState) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        let profiles = self
            .profiles
            .as_mut()
            .ok_or_else(|| invalid("derby entry requires class content"))?;
        let previous_job = profiles.active();
        let first_time = profiles.unlock(crate::derby_gateway::DEMO_DERBY_DRIVER_PROFILE)?;
        out.replies.push(profiles.select(
            crate::derby_gateway::DEMO_DERBY_DRIVER_PROFILE,
            &self.inventory,
        )?);
        if first_time {
            self.give_starter_outfit(
                crate::derby_gateway::DEMO_DERBY_DRIVER_PROFILE,
                false,
                &mut out,
            )?;
        }
        self.ensure_outfit(crate::derby_gateway::DEMO_DERBY_DRIVER_PROFILE, &mut out)?;
        let derby = self.derby.as_mut().unwrap();
        let offer = derby.begin(character)?;
        derby.set_previous_job(previous_job);
        out.replies.insert(0, END_DIALOG_26_4.to_vec());
        out.replies.push(offer);
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

    /// Adopt a staged minigame reward transaction. The caller has already
    /// committed the activity/run ledger on a clone; a profile reload failure
    /// therefore discards the whole staged gameplay state.
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

    /// W176 (R): teach the job the chosen trainer offers. The caller stages `self`, so a refusal
    /// (a chat line, `changed` false) leaves the player exactly as they were.
    fn learn_job(&mut self, trainer: &crate::profiles::Trainer) -> io::Result<Outcome> {
        use crate::profiles::LearnDenied;
        let mut out = Outcome::default();
        let denied = if self.activity_busy() {
            Some(LearnDenied::Busy)
        } else {
            match self.profiles.as_mut() {
                Some(profiles) => profiles.learn(trainer).err(),
                None => Some(LearnDenied::Unavailable),
            }
        };
        if let Some(denied) = denied {
            out.replies.push(chat::system(&denied.message()));
            return Ok(out);
        }
        // The job is not made active; the player picks it in the job list (R).
        self.give_starter_outfit(trainer.profile, false, &mut out)?;
        out.changed = true;
        out.replies.push(chat::system(&format!(
            "You learned the {} job. Pick it in your job list to switch.",
            trainer.job
        )));
        Ok(out)
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
        let rewards = RewardState::new(
            self.inventory.clone(),
            self.reward_profiles(),
            self.inventory.state().items.len().max(100),
        );
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
                let before = self.inventory.state().clone();
                if self
                    .profiles
                    .as_ref()
                    .is_some_and(|p| *p.state() != next.profiles)
                {
                    let directory = self
                        .profile_content_directory
                        .as_deref()
                        .ok_or_else(|| invalid("fishing XP needs profile content"))?;
                    self.profiles = Some(Profiles::load(directory, Some(next.profiles))?);
                }
                self.inventory = next.inventory;
                out.changed = true;
                self.npc_inventory_events(&before, &mut out)?;
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
                self.restore_job(job, &mut out)?;
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

    /// Route the native derby records while this session owns a match. The final 26/42 Close is
    /// the only payout point; an early, malformed or replayed Close leaves every bank untouched.
    fn handle_derby(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        if inner == [26, 0, 42, 0] {
            let profiles_before = self.reward_profiles();
            let rewards = RewardState::new(
                self.inventory.clone(),
                profiles_before.clone(),
                self.inventory.state().items.len().max(100),
            );
            let outcome = self.derby.as_mut().unwrap().close(inner, &rewards)?;
            if !outcome.accepted {
                return Ok(out);
            }
            let next = outcome
                .reward_state
                .ok_or_else(|| invalid("accepted derby close has no reward state"))?;
            let previous_job = self.derby.as_mut().unwrap().exit();
            self.inventory = next.inventory;
            if next.profiles != profiles_before {
                if self.profiles.is_some() {
                    let directory = self
                        .profile_content_directory
                        .as_deref()
                        .ok_or_else(|| invalid("derby XP needs profile content"))?;
                    self.profiles = Some(Profiles::load(directory, Some(next.profiles.clone()))?);
                } else {
                    return Err(invalid("derby XP needs character profile state"));
                }
            }
            out.replies = outcome.reward_packets;
            if let Some(job) = previous_job {
                self.restore_job(job, &mut out)?;
            }
            out.replies
                .push(crate::kart_gateway::player_status(character.guid, 1));
            out.changed = true;
            self.refresh_npc_quest_eligibility(current_unix_seconds());
            return Ok(out);
        }
        if inner.starts_with(&[39, 0, 5]) {
            let before = self.derby.as_ref().unwrap().phase();
            out.replies = self.derby.as_mut().unwrap().start(inner)?;
            out.changed = self.derby.as_ref().unwrap().phase() != before;
            return Ok(out);
        }
        if inner.starts_with(&[34, 0]) {
            out.replies = self.derby.as_mut().unwrap().handle(inner, now_ms)?;
            out.changed = !out.replies.is_empty();
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
                let profile = self.profiles.as_ref().unwrap().active();
                match self
                    .encounter
                    .as_mut()
                    .unwrap()
                    .begin(battle, character.clone(), profile)
                {
                    Ok(result) => self.apply_encounter(result, out)?,
                    Err(error) => say(out, format!("Cannot start battle {battle}: {error}")),
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

    /// Developer hot reload of authored content: battles, enemies, abilities.
    pub fn reload_content(&mut self, what: &str, out: &mut Outcome) -> io::Result<String> {
        let dir = self.content_dir.clone();
        let mut done = Vec::new();
        let all = what == "all";
        if all || what == "abilities" {
            let path = dir.join("job_abilities.json");
            if path.is_file() {
                if let Some(abilities) = self.abilities.as_mut() {
                    abilities.load_job_abilities(&path)?;
                    let profile = self.profiles.as_ref().unwrap().active();
                    let level = self.job_level(profile);
                    out.replies.extend(
                        self.abilities
                            .as_ref()
                            .unwrap()
                            .activation(profile, level)?,
                    );
                    done.push("job abilities");
                }
            }
        }
        if all || what == "battles" {
            let path = dir.join("field_battles.json");
            if path.is_file() {
                if let Some(encounter) = self.encounter.as_mut() {
                    encounter.reload_field_battles(&path)?;
                    self.activities = crate::activity_portal::Catalogue::load(&path)?;
                    if let Some(chess) = &self.chess {
                        self.activities = self
                            .activities
                            .clone()
                            .with_chess_rows(chess.activities())?;
                    }
                    if let Some(visibility) = self.npc_visibility.as_mut() {
                        for (guid, name, name_id, model_id, position, icon) in
                            encounter.battle_spawners()
                        {
                            visibility
                                .add_static_actor(guid, &name, name_id, model_id, position, icon);
                        }
                    }
                    done.push("battles");
                }
            }
        }
        if all || what == "behaviours" {
            let path = dir.join("behaviours.json");
            if path.is_file() {
                if let Some(encounter) = self.encounter.as_mut() {
                    encounter.load_behaviours(&path)?;
                    done.push("behaviours");
                }
            }
        }
        if all || what == "enemies" {
            let path = dir.join("field_enemies.json");
            if path.is_file() {
                if let Some(encounter) = self.encounter.as_mut() {
                    out.replies.extend(encounter.reload_field_enemies(&path)?);
                    done.push("field enemies");
                }
                self.sync_field_quests(out)?;
            }
        }
        if all || what == "power_ups" {
            let path = dir.join("power_ups.json");
            if path.is_file() {
                if let Some(encounter) = self.encounter.as_mut() {
                    encounter.load_power_ups(&path)?;
                    done.push("power-ups");
                }
            }
        }
        if all || what == "placements" {
            let path = self.placements_path();
            if let Some(visibility) = self.npc_visibility.as_mut() {
                let records = if path.is_file() {
                    let bytes = std::fs::read(&path)?;
                    serde_json::from_slice::<crate::world::PlacementFile>(&bytes)
                        .map_err(|e| invalid(format!("npc_placements.json: {e}")))?
                        .records
                } else {
                    Vec::new()
                };
                out.replies.extend(visibility.replace_placements(&records)?);
                done.push("placed NPCs");
            }
        }
        if done.is_empty() {
            return Err(invalid(
                "unknown target; use realm reload [all|battles|enemies|abilities|behaviours|power_ups|placements]",
            ));
        }
        Ok(done.join(", "))
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
        let mut lines = vec![format!("Battle {battle}, job {profile} (scripted bot):")];
        for level in [1_u32, 5, 10, 15, 20] {
            let mut engine = base.clone();
            // The scripted bot must not be helped by power-up hearts.
            engine.unload_power_ups();
            engine.set_player_job(profile, level);
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
        let busy = self.forging.as_ref().is_some_and(Forging::active)
            || self.smelting.as_ref().is_some_and(Smelting::active)
            || self.cooking.as_ref().is_some_and(Cooking::active)
            || self.mining.as_ref().is_some_and(Type8Engine::active)
            || self.harvesting.as_ref().is_some_and(Type8Engine::active)
            || self.chess.as_ref().is_some_and(ChessService::is_active)
            || self.kart_entry.as_ref().is_some_and(|k| k.active())
            || !self.fishing.is_empty()
            || self.soccer.is_some();
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
        let rewards = RewardState::new(
            self.inventory.clone(),
            self.reward_profiles(),
            self.inventory.state().items.len().max(100),
        );
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
                        let profile = self.profiles.as_ref().map_or(0, Profiles::active);
                        let begun = self
                            .encounter
                            .as_mut()
                            .map(|encounter| encounter.begin(battle, character.clone(), profile));
                        match begun {
                            Some(Ok(result)) => self.apply_encounter(result, &mut out)?,
                            _ => out
                                .replies
                                .push(chat::system("You cannot start that activity here.")),
                        }
                    }
                }
            }
        }
        Ok(Some(out))
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
        let rewards = RewardState::new(
            staged.inventory.clone(),
            staged.reward_profiles(),
            staged.inventory.state().items.len().max(100),
        );
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
            let before = self.inventory.state().clone();
            if self
                .profiles
                .as_ref()
                .is_some_and(|profiles| *profiles.state() != next.profiles)
            {
                let directory = self
                    .profile_content_directory
                    .as_deref()
                    .ok_or_else(|| invalid("chess reward XP needs profile content"))?;
                self.profiles = Some(Profiles::load(directory, Some(next.profiles))?);
            }
            self.inventory = next.inventory;
            out.changed = true;
            self.npc_inventory_events(&before, out)?;
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
        std::fs::write(self.placements_path(), text + "\n")
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
        std::fs::write(self.marks_path(), text + "\n")
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
    /// compete. Reward authority stays server-side in `apply_mining`.
    fn developer_mining(&mut self, now_ms: u64, out: &mut Outcome) -> io::Result<()> {
        let Some(mining) = self.mining.as_ref() else {
            out.replies.push(chat::system(
                "Mining content is not loaded in this campaign.",
            ));
            return Ok(());
        };
        if mining.active() {
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
            .mining
            .as_mut()
            .unwrap()
            .begin(mining::MINER_PROFILE_ID)?;
        self.type8_tick_ms = Some(now_ms);
        self.apply_mining(result, out)?;
        out.replies.push(chat::system(
            "Mining test started; type realm exit to cancel.",
        ));
        Ok(())
    }

    /// Developer-only `realm harvest`: starts the reconstructed Harvesting111
    /// candidate for the accepted Chef job. Dev tools must be enabled, Chef
    /// must be unlocked and active, and no other live activity may compete.
    /// Reward authority stays server-side in `apply_harvesting`.
    fn developer_harvesting(&mut self, now_ms: u64, out: &mut Outcome) -> io::Result<()> {
        let Some(harvesting) = self.harvesting.as_ref() else {
            out.replies.push(chat::system(
                "Harvesting content is not loaded in this campaign.",
            ));
            return Ok(());
        };
        if harvesting.active() {
            out.replies.push(chat::system(
                "A harvesting test is already running. Type realm exit to cancel it.",
            ));
            return Ok(());
        }
        if self.forging.as_ref().is_some_and(Forging::active)
            || self.smelting.as_ref().is_some_and(Smelting::active)
            || self.cooking.as_ref().is_some_and(Cooking::active)
            || self.mining.as_ref().is_some_and(Type8Engine::active)
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
            .harvesting
            .as_mut()
            .unwrap()
            .begin(harvesting::CHEF_PROFILE_ID)?;
        self.type8_tick_ms = Some(now_ms);
        self.apply_harvesting(result, out)?;
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
                self.apply_forging(result, character, &mut out)?;
                self.forging_tick_ms = None;
                return Ok(out);
            }
            if inner.starts_with(&[39, 0]) {
                let result = self.forging.as_mut().unwrap().handle(inner)?;
                self.apply_forging(result, character, &mut out)?;
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
                    self.apply_forging(result, character, &mut out)?;
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
                self.apply_cooking(result, character, &mut out)?;
                return Ok(out);
            }
            if inner.starts_with(&[39, 0]) {
                let result = self.cooking.as_mut().unwrap().handle(inner)?;
                self.apply_cooking(result, character, &mut out)?;
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
                    self.apply_cooking(result, character, &mut out)?;
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
                self.apply_smelting(result, character, &mut out)?;
                return Ok(out);
            }
            if inner.starts_with(&[39, 0]) {
                let result = self.smelting.as_mut().unwrap().handle(inner)?;
                self.apply_smelting(result, character, &mut out)?;
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
                    self.apply_smelting(result, character, &mut out)?;
                } else if text.trim().to_ascii_lowercase().starts_with("realm ") {
                    out.replies.push(chat::system(
                        "Close the smelting game before other opening actions.",
                    ));
                }
                return Ok(out);
            }
        }
        if self.mining.as_ref().is_some_and(Type8Engine::active) {
            if inner.starts_with(&[26, 0, 42, 0]) {
                if inner.len() != 4 {
                    return Err(invalid("invalid mining leave request"));
                }
                let result = self.mining.as_mut().unwrap().cancel();
                self.apply_mining(result, &mut out)?;
                self.type8_tick_ms = None;
                return Ok(out);
            }
            if inner.starts_with(&[39, 0]) {
                let result = self.mining.as_mut().unwrap().handle(inner)?;
                self.apply_mining(result, &mut out)?;
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
                    let result = self.mining.as_mut().unwrap().cancel();
                    self.apply_mining(result, &mut out)?;
                    self.type8_tick_ms = None;
                } else if text.trim().to_ascii_lowercase().starts_with("realm ") {
                    out.replies.push(chat::system(
                        "Close the mining game before other opening actions.",
                    ));
                }
                return Ok(out);
            }
        }
        if self.harvesting.as_ref().is_some_and(Type8Engine::active) {
            if inner.starts_with(&[26, 0, 42, 0]) {
                if inner.len() != 4 {
                    return Err(invalid("invalid harvesting leave request"));
                }
                let result = self.harvesting.as_mut().unwrap().cancel();
                self.apply_harvesting(result, &mut out)?;
                self.type8_tick_ms = None;
                return Ok(out);
            }
            if inner.starts_with(&[39, 0]) {
                let result = self.harvesting.as_mut().unwrap().handle(inner)?;
                self.apply_harvesting(result, &mut out)?;
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
                    let result = self.harvesting.as_mut().unwrap().cancel();
                    self.apply_harvesting(result, &mut out)?;
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
            let profiles = self.profiles.as_mut().unwrap();
            let changed = profiles.active() != id;
            match profiles.select(id, &self.inventory) {
                Ok(reply) => {
                    out.replies.push(reply);
                    out.replies.extend(
                        self.abilities
                            .as_ref()
                            .unwrap()
                            .activation(id, self.job_level(id))?,
                    );
                    self.sync_player_level(&mut out);
                    out.changed |= changed;
                    self.ensure_outfit(id, &mut out)?;
                    self.opening_equipment_events(&mut out)?;
                    self.sync_world_stage(&mut out)?;
                    // T027: neighbours swap the whole attachment group on a job change.
                    if changed && let Some(packet) = self.profile_switch_packet(character.guid)? {
                        out.nearby.push(packet);
                    }
                }
                Err(error) => out
                    .replies
                    .push(chat::system(&format!("Cannot switch classes: {error}"))),
            }
            return Ok(out);
        }
        // Recipe-book start (client 26/24): recipe_book.rs decodes it and checks
        // learned recipe, active job, idle services, a server-known station in
        // range, rank and exact inputs before the service's own `begin_recipe`.
        if let Some(recipe_id) = crate::recipe_book::decode(inner)? {
            if self.activity_busy() {
                out.replies
                    .push(crate::recipe_book::Refusal::Busy.feedback());
                return Ok(out);
            }
            let profile = self.profiles.as_ref().unwrap().active();
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
                    self.apply_cooking(result, character, &mut out)?
                }
                Ok(crate::recipe_book::Started::Smelting(result)) => {
                    self.apply_smelting(result, character, &mut out)?
                }
                Ok(crate::recipe_book::Started::Forging(result)) => {
                    self.forging_tick_ms = Some(now_ms);
                    self.apply_forging(result, character, &mut out)?
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
                            self.apply_cooking(result, character, &mut out)?;
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
                            self.apply_forging(result, character, &mut out)?;
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
                            self.apply_smelting(result, character, &mut out)?;
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
        // Go on a battle start screen (41/108): enter the battle's instanced
        // arena through the same begin() as the opening cave.
        if inner.starts_with(&[41, 0, 108, 0]) {
            let active_profile = self.profiles.as_ref().unwrap().active();
            if let Some(battle) = self.encounter.as_mut().unwrap().take_pending_battle() {
                if !self
                    .abilities
                    .as_ref()
                    .unwrap()
                    .supports_basic(active_profile)
                {
                    out.replies.push(chat::system(
                        "Switch to a combat job before starting a battle.",
                    ));
                    // Keep the start screen up (it hides the HUD): Switch Jobs
                    // is on it, then Go again or close.
                    out.replies.extend(
                        self.encounter
                            .as_mut()
                            .unwrap()
                            .reopen_battle_screen(battle, active_profile),
                    );
                    return Ok(out);
                }
                let result = self.encounter.as_mut().unwrap().begin(
                    battle,
                    character.clone(),
                    active_profile,
                )?;
                self.apply_encounter(result, &mut out)?;
                return Ok(out);
            }
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

    fn near_craft_station(
        &self,
        character: &CharacterState,
        job: crate::recipes::CraftingJob,
    ) -> bool {
        self.crafting_stations().iter().any(|station| {
            station.job == job
                && station.zone_id == character.zone_id
                && station
                    .position
                    .iter()
                    .zip(character.position)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f32>()
                    <= station.radius.powi(2)
        })
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

    fn apply_smelting(
        &mut self,
        result: SmeltingOutcome,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(transaction) = result.transaction {
            // (R) A recipe-book run has no tutorial objective; the profile and
            // station checks still bind the completion.
            if self.profiles.as_ref().unwrap().active() != 14
                || !self.near_craft_station(character, crate::recipes::CraftingJob::Miner)
            {
                return Err(invalid("smelting completion is outside a Miner station"));
            }
            if transaction
                .consumes
                .iter()
                .any(|(id, n)| self.material_count(*id) < u64::from(*n))
            {
                return Err(invalid("smelting ingredients changed before completion"));
            }
            self.consume_materials(transaction.consumes, out)?;
            self.grant_reward(0, &transaction.grants, out)?;
            let quantity = transaction
                .grants
                .iter()
                .filter(|(id, _)| *id == 21785)
                .map(|(_, n)| *n)
                .sum();
            if self.opening_craft_pending(2002) {
                let progress = self
                    .opening
                    .as_mut()
                    .unwrap()
                    .on_event(OpeningEvent::Craft {
                        recipe_id: 2002,
                        quantity,
                    })?;
                self.apply_opening(progress, out)?;
            }
            // (R) The opening binding is the fallback for the unnumbered sample.
            let recipe_id = self
                .recipe_catalog
                .by_key(self.smelting.as_ref().unwrap().recipe_key())
                .and_then(|r| r.recipe_id)
                .unwrap_or(2002);
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

    /// Stage a TYPE8 outcome. The only reward path is the engine's server-side
    /// transaction derived from the configured `outputs` and the collected
    /// quota; client score or win reports never reach it. Grants commit inside
    /// the staged gameplay clone, which the gateway saves before sending.
    fn apply_mining(&mut self, result: Type8Outcome, out: &mut Outcome) -> io::Result<()> {
        if let Some(transaction) = result.transaction {
            let grant = crate::mining::lifecycle::reward_grant(Some(&transaction), 0)?;
            if self.profiles.as_ref().map(Profiles::active) != Some(mining::MINER_PROFILE_ID) {
                return Err(invalid(
                    "mining completion is outside the accepted Miner job",
                ));
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

    /// Stage a Harvesting111 outcome. The only reward path is the engine's
    /// server-side transaction derived from the configured outputs and the
    /// collected quota; client score or win reports never reach it. Grants and
    /// any consumption commit inside the staged gameplay clone, which the
    /// gateway saves before sending.
    fn apply_harvesting(&mut self, result: Type8Outcome, out: &mut Outcome) -> io::Result<()> {
        if let Some(transaction) = result.transaction {
            let grant = crate::harvesting::lifecycle::reward_grant(
                Some(&transaction),
                HARVESTING_GAME_ID,
                0,
            )?;
            if self.profiles.as_ref().map(Profiles::active) != Some(harvesting::CHEF_PROFILE_ID) {
                return Err(invalid(
                    "harvesting completion is outside the accepted Chef job",
                ));
            }
            self.consume_materials(transaction.consumes, out)?;
            let grants: Vec<_> = grant
                .items
                .iter()
                .map(|item| (item.definition_id, item.count))
                .collect();
            self.grant_reward(0, &grants, out)?;
            // (R) Resource identity is the configured output item definition.
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

    fn apply_cooking(
        &mut self,
        result: CookingOutcome,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(transaction) = result.transaction {
            // (R) A recipe-book run has no tutorial objective; the profile and
            // station checks still bind the completion.
            if self.profiles.as_ref().unwrap().active() != 45
                || !self.near_craft_station(character, crate::recipes::CraftingJob::Chef)
            {
                return Err(invalid("cooking completion is outside a Chef station"));
            }
            if transaction
                .consumes
                .iter()
                .any(|(id, n)| self.material_count(*id) < u64::from(*n))
            {
                return Err(invalid("cooking ingredients changed before completion"));
            }
            self.consume_materials(transaction.consumes, out)?;
            self.grant_reward(0, &transaction.grants, out)?;
            if self.opening_craft_pending(2001) {
                let progress = self
                    .opening
                    .as_mut()
                    .unwrap()
                    .on_event(OpeningEvent::Craft {
                        recipe_id: 2001,
                        quantity: 1,
                    })?;
                self.apply_opening(progress, out)?;
            }
            // (R) Reuse the opening's reconstructed recipe binding.
            let recipe_id = self
                .recipe_catalog
                .by_key(self.cooking.as_ref().unwrap().recipe_key())
                .and_then(|r| r.recipe_id)
                .unwrap_or(2001);
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

    fn apply_forging(
        &mut self,
        result: ForgingOutcome,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(transaction) = result.transaction {
            // (R) A recipe-book run has no tutorial objective; the profile and
            // station checks still bind the completion.
            if self.profiles.as_ref().unwrap().active() != 16
                || !self.near_craft_station(character, crate::recipes::CraftingJob::Blacksmith)
            {
                return Err(invalid(
                    "forging completion is outside a Blacksmith station",
                ));
            }
            if transaction
                .consumes
                .iter()
                .any(|(id, n)| self.material_count(*id) < u64::from(*n))
            {
                return Err(invalid("forging ingredients changed before completion"));
            }
            self.consume_materials(transaction.consumes, out)?;
            self.grant_reward(0, &transaction.grants, out)?;
            if self.opening_craft_pending(2003) {
                let progress = self
                    .opening
                    .as_mut()
                    .unwrap()
                    .on_event(OpeningEvent::Craft {
                        recipe_id: 2003,
                        quantity: 1,
                    })?;
                self.apply_opening(progress, out)?;
            }
            // (R) Reuse the opening's reconstructed recipe binding.
            let recipe_id = self
                .recipe_catalog
                .by_key(self.forging.as_ref().unwrap().recipe_key())
                .and_then(|r| r.recipe_id)
                .unwrap_or(2003);
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
            if let (Some(engine), Ok(actor_kind_id)) =
                (self.npc_quests.as_mut(), u32::try_from(definition))
            {
                if actor_kind_id != 0 {
                    let quest =
                        engine.on_event(crate::quest_engine::QuestEvent::KillActorKind {
                            actor_kind_id,
                            count: 1,
                        })?;
                    self.append_npc_quest_outcome(quest, out)?;
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
        }
        if let Some(zone) = result.zoning {
            out.zoning = Some(if result.completed.is_some() {
                self.opening_dungeon_destination(None).unwrap_or(zone)
            } else {
                zone
            });
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
            let mut switched = false;
            if let Some(profiles) = &mut self.profiles {
                switched = profiles.active() != id;
                match profiles.select(id, &self.inventory) {
                    Ok(reply) => {
                        out.replies.push(reply);
                        out.changed = true;
                    }
                    Err(error) => {
                        switched = false;
                        out.replies
                            .push(chat::system(&format!("Cannot switch classes: {error}")));
                    }
                }
            }
            if switched {
                if let Some(packet) = self.profile_switch_packet(guid)? {
                    out.nearby.push(packet);
                }
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

/// Point the client's dialog camera (`ControllerDialog`, switched in by
/// `ReplacePointerAtECAndDispatch` `008d2ba0`) at the NPC, so it zooms in on them
/// with the bubble to their left, as in the original. A zero target falls back to
/// the player (native run 230409, Frida `dialog_camera` hooks):
/// - 26/3 ShowDialog: dwords 2-3 of the leading four (`ShowDialogFromPacket`
///   `009f5ae0` reads object +0x18/+0x1c).
/// - 49/1 QuestInfo: the u64 after the reward bundle, i.e. the packet's last 14
///   bytes start with it (dispatcher `00c7cea2` passes object +0xe0/+0xe4).
/// The 49/13 end screen carries it at offset 6 (set where the turn-in is held).
fn set_dialog_camera_target(packet: &mut [u8], npc_guid: u64) {
    let guid = npc_guid.to_le_bytes();
    if packet.starts_with(&[26, 0, 3, 0]) && packet.len() >= 20 {
        packet[12..20].copy_from_slice(&guid);
    } else if packet.starts_with(&[49, 0, 1, 0]) && packet.len() >= 14 {
        let at = packet.len() - 14;
        packet[at..at + 8].copy_from_slice(&guid);
    }
}

fn dialogue_packet(
    context: &DialogueContext,
) -> Result<Vec<u8>, npc_interaction::NpcInteractionError> {
    dialogue_packet_with(context, None)
}

fn dialogue_packet_with(
    context: &DialogueContext,
    quest_choice: Option<[u32; 5]>,
) -> Result<Vec<u8>, npc_interaction::NpcInteractionError> {
    // The locale export maps 8075 to "Goodbye". The five-word response row is
    // client-reader-backed; its 300/17/3 close tuple follows Lua's AddResponseNode
    // convention (notes/Research/Pieces/071 NPC dialogue branching.md). The body
    // field is a locale greeting when one is recovered; it is never the NPC name.
    let mut choices: Vec<[u32; 5]> = quest_choice.into_iter().collect();
    choices.push(npc_interaction::goodbye_dialogue_choice());
    npc_interaction::encode_dialogue_26_3(&npc_interaction::DialoguePacket26_3 {
        initial_dwords: [context.body_string_id.unwrap_or(0), 0, 0, 0],
        initial_flag: false,
        initial_float_bits: 0,
        choices,
        choice_vectors: [[0; 4]; 2],
        vector_flag: false,
        final_vector: [0; 4],
        float_after_vector_bits: 0,
        final_flags: [false; 3],
        final_float_bits: 0,
        final_dword: 0,
    })
}

fn merchant_packet(
    window: &MerchantWindow,
) -> Result<Vec<u8>, npc_interaction::NpcInteractionError> {
    let entries = window
        .items
        .iter()
        .map(|item| npc_interaction::MerchantEntryV2 {
            // (R) IDs and reference prices are carried in opaque fields by local convention.
            first_six_dwords: [
                item.item_id as u32,
                item.purchase_price_coins.unwrap_or(0),
                item.sell_price_coins.unwrap_or(0),
                0,
                0,
                0,
            ],
            flag_after_six: false,
            final_three_dwords: [0; 3],
            final_flag: false,
        })
        .collect();
    let npc = window.npc_id;
    // (R) 66/42 layout is recovered, while all semantic values remain opaque. The stable content
    // key identifies the merchant in this reconstructed listing.
    npc_interaction::encode_merchant_list_66_42(&npc_interaction::MerchantListV2 {
        initial_i16s: [0; 2],
        initial_dword: npc as u32,
        group: npc_interaction::MerchantGroupV2 {
            fields: [npc as u32, (npc >> 32) as u32, 0, 0, 0, 0, 0],
            entries,
        },
    })
}

/// CommandPacketEndDialog: family 26 subtype 4, header only. Client handler
/// `CommandPacketEndDialog_Handle` (`00aa0870`) requires exactly the 4-byte
/// header; constructor `009f5760` (BaseCommandPacket sub-opcode 4).
const END_DIALOG_26_4: [u8; 4] = [26, 0, 4, 0];

fn quest_offer_packet(
    engine: &crate::quest_engine::QuestEngine,
    quest_id: u32,
) -> Result<Vec<u8>, npc_interaction::NpcInteractionError> {
    quest_offer_packet_with(engine, quest_id, None)
}

fn quest_offer_packet_with(
    engine: &crate::quest_engine::QuestEngine,
    quest_id: u32,
    intro_override: Option<u32>,
) -> Result<Vec<u8>, npc_interaction::NpcInteractionError> {
    // The 49/1 reader's common-prefix dword is the quest ID. Subsequent fields
    // feed ShowQuestStartScreen in client order. Locale IDs come from reviewed
    // split content; missing IDs and offer policy are reconstructed (R). Lua
    // QuestStartHandler supplies its own card icon (454), Show Details and the
    // Accept/Decline buttons, so none are synthesized into this payload.
    let title_id = engine.title_string_id(quest_id).unwrap_or(0);
    npc_interaction::encode_quest_offer_49_1(&npc_interaction::QuestOfferPacket49_1 {
        quest_id,
        intro_text_string_id: intro_override
            .unwrap_or_else(|| engine.intro_text_string_id(quest_id).unwrap_or(0)),
        // Slot order probe-verified in native run 143124: dword 3 is the card
        // title and dword 4 the description; dwords 2 and 5 are not shown.
        field_2: title_id,
        name_string_id: title_id,
        description_string_id: engine.description_string_id(quest_id).unwrap_or(0),
        field_5: title_id,
        members_only: false,
        reward_bundle_body: vec![0; 69],
        trailing_dwords: [0; 2],
        trailing_float_bits: 0,
        trailing_flags: [false; 2],
    })
}

#[cfg(test)]
mod developer_teleport {
    use super::*;
    use std::path::PathBuf;

    fn content_directory() -> PathBuf {
        std::env::var_os("OSFR_OPENING_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../../lab/references/rust-opening-cooking-original-assets")
            })
    }

    fn character() -> CharacterState {
        CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-1063.60595703125, 4.418806552886963, 305.6822509765625],
            rotation: [1.0, 0.0, 0.0],
        }
    }

    fn chat(text: &str) -> Vec<u8> {
        let mut p = vec![15, 0, 1, 0, 0, 0];
        p.extend(1u64.to_le_bytes());
        p.extend(0u64.to_le_bytes());
        for _ in 0..2 {
            p.extend([0; 12]);
            p.extend(0u32.to_le_bytes());
            p.extend(0u32.to_le_bytes());
        }
        p.extend((text.len() as u32).to_le_bytes());
        p.extend(text.as_bytes());
        p.extend([0; 28]);
        p
    }

    #[test]
    fn crafting_movie_refuses_relocation_and_keeps_state() {
        let content = content_directory();
        assert!(content.join("cooking-campaign.json").is_file());
        let mut game = Gameplay::load(&content, None).unwrap();
        game.set_dev_tools(true);
        game.set_role(crate::account_store::AccountRole::Admin);
        game.cooking.as_mut().unwrap().begin(45).unwrap();
        let before = game.state();
        let out = game
            .handle_in_world(&chat("realm goto mae farnum"), &character(), 1_000)
            .unwrap();
        assert!(out.relocation.is_none());
        assert!(
            out.replies
                .iter()
                .any(|reply| String::from_utf8_lossy(reply).contains("Close the cooking game"))
        );
        assert_eq!(game.state(), before);
    }

    /// A flat terrain grid at `height` around the test character, in a temp dir.
    fn flat_terrain(name: &str, height: f32) -> Arc<crate::terrain::Terrain> {
        let p = character().position;
        let mut grid = crate::terrain::HeightGrid::new(
            [p[0] - 100.0, p[2] - 100.0],
            [p[0] + 100.0, p[2] + 100.0],
            2.0,
        )
        .unwrap();
        let corner = |dx: f32, dz: f32| [p[0] + dx, height, p[2] + dz];
        grid.add_triangle([
            corner(-100.0, -100.0),
            corner(100.0, -100.0),
            corner(100.0, 100.0),
        ]);
        grid.add_triangle([
            corner(-100.0, -100.0),
            corner(100.0, 100.0),
            corner(-100.0, 100.0),
        ]);
        let dir = std::env::temp_dir().join(format!("osfr-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("zone_0.hgt"), grid.to_bytes()).unwrap();
        Arc::new(crate::terrain::Terrain::load(&dir).unwrap())
    }

    fn reply_text(out: &Outcome) -> String {
        out.replies
            .iter()
            .map(|reply| String::from_utf8_lossy(reply).into_owned())
            .collect()
    }

    #[test]
    fn realm_goto_coordinates_land_on_the_terrain() {
        let content = content_directory();
        let mut game = Gameplay::load(&content, None).unwrap();
        game.set_dev_tools(true);
        game.set_role(crate::account_store::AccountRole::Admin);
        let start = character();
        let (x, z) = (start.position[0] + 20.0, start.position[2] - 15.0);
        // Without terrain data a ground teleport needs an explicit height.
        let out = game
            .handle_in_world(&chat(&format!("realm goto {x} {z}")), &start, 1_000)
            .unwrap();
        assert!(out.relocation.is_none());
        assert!(
            reply_text(&out).contains("No terrain heights"),
            "{}",
            reply_text(&out)
        );
        let out = game
            .handle_in_world(&chat(&format!("realm goto {x}, 3.5, {z}")), &start, 1_001)
            .unwrap();
        assert_eq!(out.relocation.unwrap().position, [x, 3.5, z]);

        game.set_terrain(Some(flat_terrain("goto", 42.0)));
        let out = game
            .handle_in_world(&chat(&format!("realm goto {x} {z}")), &start, 1_002)
            .unwrap();
        let relocation = out.relocation.expect("lands on the ground");
        assert_eq!(relocation.position, [x, 42.5, z]);
        assert_eq!(relocation.rotation, start.rotation);
        let here = game
            .handle_in_world(&chat("realm here"), &start, 1_003)
            .unwrap();
        assert!(
            reply_text(&here).contains("Ground 42.00"),
            "{}",
            reply_text(&here)
        );
        // Actor names still teleport as before.
        let out = game
            .handle_in_world(&chat("realm goto Roosey"), &start, 1_004)
            .unwrap();
        assert!(out.relocation.is_some());
    }

    #[test]
    fn realm_place_adds_saves_and_removes_npcs_live() {
        let content = content_directory();
        let mut game = Gameplay::load(&content, None).unwrap();
        game.set_dev_tools(true);
        game.set_role(crate::account_store::AccountRole::Admin);
        let dir = std::env::temp_dir().join(format!("osfr-placements-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let _ = std::fs::remove_file(dir.join("npc_placements.json"));
        game.content_dir = dir.clone();
        let start = character();
        game.update_npc_visibility(&start).unwrap();
        let has = |out: &Outcome, sub: u8, guid: u64| {
            out.replies.iter().any(|p| {
                p.starts_with(&[35, 0, sub, 0])
                    && (p.get(4..12) == Some(&guid.to_le_bytes()[..])
                        || p.get(6..14) == Some(&guid.to_le_bytes()[..]))
            })
        };
        let first = crate::world::PLACEMENT_GUID_BASE + 1;
        let out = game
            .handle_in_world(&chat("realm place 168 15623"), &start, 1_000)
            .unwrap();
        assert!(
            reply_text(&out).contains(&format!("Placed NPC {first}")),
            "{}",
            reply_text(&out)
        );
        assert!(has(&out, 2, first), "AddNpc sent at once");
        let saved = game.load_placements();
        assert_eq!(saved.len(), 1);
        assert_eq!((saved[0].model_id, saved[0].name_string_id), (168, 15623));
        assert_eq!(saved[0].position, start.position);
        // Hand-added fields survive the next rewrite.
        let mut edited: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("npc_placements.json")).unwrap())
                .unwrap();
        assert_eq!(edited["records"][0]["source"][0]["kind"], "reconstructed");
        edited["records"][0]["note"] = "guards the bridge".into();
        std::fs::write(dir.join("npc_placements.json"), edited.to_string()).unwrap();

        // On the ground at x, z (needs terrain heights).
        let (x, z) = (start.position[0] + 10.0, start.position[2]);
        let refused = game
            .handle_in_world(&chat(&format!("realm place 182 at {x} {z}")), &start, 1_001)
            .unwrap();
        assert!(reply_text(&refused).contains("No terrain heights"));
        game.set_terrain(Some(flat_terrain("place", 12.0)));
        let second = first + 1;
        let out = game
            .handle_in_world(&chat(&format!("realm place 182 at {x} {z}")), &start, 1_002)
            .unwrap();
        assert!(has(&out, 2, second), "{}", reply_text(&out));
        let placed = game.load_placements();
        assert_eq!(placed[1].position, [x, 12.0, z]);
        // Where you stand, with terrain: on the ground, not at your height.
        let raised = CharacterState {
            position: [start.position[0] - 5.0, 12.5, start.position[2]],
            ..start.clone()
        };
        game.handle_in_world(&chat("realm place 166"), &raised, 1_002)
            .unwrap();
        assert_eq!(game.load_placements()[2].position[1], 12.0);
        game.handle_in_world(
            &chat(&format!("realm unplace {}", second + 1)),
            &start,
            1_002,
        )
        .unwrap();
        assert_eq!(placed[0].extra["note"], "guards the bridge");
        let list = game
            .handle_in_world(&chat("realm placements"), &start, 1_003)
            .unwrap();
        assert!(
            reply_text(&list).contains("2 placed NPCs"),
            "{}",
            reply_text(&list)
        );

        // The nearest is the one where the character stands.
        let out = game
            .handle_in_world(&chat("realm unplace nearest"), &start, 1_004)
            .unwrap();
        assert!(has(&out, 3, first), "RemoveNpc sent: {}", reply_text(&out));
        assert_eq!(game.load_placements().len(), 1);

        // Editing the file then reloading swaps the placed NPCs live.
        std::fs::write(
            dir.join("npc_placements.json"),
            r#"{"schema_version":1,"records":[]}"#,
        )
        .unwrap();
        let out = game
            .handle_in_world(&chat("realm reload placements"), &start, 1_005)
            .unwrap();
        assert!(has(&out, 3, second), "{}", reply_text(&out));
        let bad = game
            .handle_in_world(&chat("realm place troll"), &start, 1_006)
            .unwrap();
        assert!(reply_text(&bad).contains("Usage: realm place"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn live_world_keeps_builds_out_of_content_and_across_restarts() {
        let content = content_directory();
        let world = std::env::temp_dir().join(format!("osfr-world-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&world);
        let start = character();
        let guid;
        {
            let mut game = Gameplay::load(&content, None).unwrap();
            game.set_dev_tools(true);
            game.set_role(crate::account_store::AccountRole::Admin);
            game.set_world_dir(Some(world.clone())).unwrap();
            game.update_npc_visibility(&start).unwrap();
            let out = game
                .handle_in_world(&chat("realm place 168 named Bob the Lost"), &start, 1_000)
                .unwrap();
            let placed = game.load_placements();
            assert_eq!(placed.len(), 1, "{}", reply_text(&out));
            assert_eq!(placed[0].name.as_deref(), Some("Bob the Lost"), "case kept");
            guid = placed[0].guid;
            let add = out
                .replies
                .iter()
                .find(|p| p.starts_with(&[35, 0, 2, 0]))
                .expect("AddNpc sent");
            assert!(
                add.windows(12).any(|w| w == b"Bob the Lost"),
                "the literal name rides in AddNpc"
            );
            game.handle_in_world(&chat("realm mark hunt start"), &start, 1_001)
                .unwrap();
            assert!(world.join("npc_placements.json").is_file());
            assert!(world.join("marks.json").is_file());
            assert!(
                !game.content_dir.join("npc_placements.json").exists(),
                "content untouched"
            );
        }
        // A restart loads the live world again and shows the placed NPC.
        let mut game = Gameplay::load(&content, None).unwrap();
        game.set_dev_tools(true);
        game.set_role(crate::account_store::AccountRole::Admin);
        game.set_world_dir(Some(world.clone())).unwrap();
        let spawned = game.update_npc_visibility(&start).unwrap();
        assert!(
            spawned
                .iter()
                .any(|p| p.starts_with(&[35, 0, 2, 0])
                    && p.windows(8).any(|w| w == guid.to_le_bytes())),
            "placed NPC persists past a restart"
        );
        let marks = game
            .handle_in_world(&chat("realm marks"), &start, 1_002)
            .unwrap();
        assert!(reply_text(&marks).contains("hunt start"));
        std::fs::remove_dir_all(&world).unwrap();
    }

    #[test]
    fn realm_text_assigns_stable_ids_in_the_live_world() {
        let content = content_directory();
        let world = std::env::temp_dir().join(format!("osfr-world-text-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&world);
        let mut game = Gameplay::load(&content, None).unwrap();
        game.set_dev_tools(true);
        game.set_role(crate::account_store::AccountRole::Admin);
        game.set_world_dir(Some(world.clone())).unwrap();
        let first = crate::world_text::FIRST_WORLD_TEXT_ID;
        game.set_stock_text(Some(Arc::new(HashSet::from([
            crate::world_text::locale_hash(first),
        ]))));
        let start = character();
        let out = game
            .handle_in_world(&chat("realm text Find Bob the Lost!"), &start, 1_000)
            .unwrap();
        let id = first + 1;
        assert!(
            reply_text(&out).contains(&format!("Text {id}")),
            "{}",
            reply_text(&out)
        );
        let again = game
            .handle_in_world(&chat("realm text Find Bob the Lost!"), &start, 1_001)
            .unwrap();
        assert!(
            reply_text(&again).contains(&format!("Text {id}")),
            "same text keeps its id"
        );
        let list = game
            .handle_in_world(&chat("realm texts"), &start, 1_002)
            .unwrap();
        assert!(
            reply_text(&list).contains("Find Bob the Lost!"),
            "case kept: {}",
            reply_text(&list)
        );
        assert!(world.join("text.json").is_file());
        std::fs::remove_dir_all(&world).unwrap();
    }

    #[test]
    fn developer_commands_follow_the_account_role() {
        use crate::account_store::AccountRole;
        let content = content_directory();
        let world = std::env::temp_dir().join(format!("osfr-roles-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&world);
        let mut game = Gameplay::load(&content, None).unwrap();
        game.set_dev_tools(true);
        game.set_role(crate::account_store::AccountRole::Admin);
        game.set_world_dir(Some(world.clone())).unwrap();
        let start = character();
        let try_cmd = |game: &mut Gameplay, text: &str| {
            let out = game.handle_in_world(&chat(text), &start, 1_000).unwrap();
            reply_text(&out)
        };
        // Player: no developer command runs.
        game.set_role(AccountRole::Player);
        assert!(!try_cmd(&mut game, "realm mark spot").contains("Marked"));
        assert!(!try_cmd(&mut game, "realm here").contains("Zone 0 at"));
        assert!(!world.join("marks.json").exists());
        // Builder: building commands yes, cheats no.
        game.set_role(AccountRole::Builder);
        assert!(try_cmd(&mut game, "realm mark spot").contains("Marked spot"));
        assert!(try_cmd(&mut game, "realm here").contains("Zone 0 at"));
        assert!(!try_cmd(&mut game, "realm god").contains("God mode"));
        // Admin: everything.
        game.set_role(AccountRole::Admin);
        let god = try_cmd(&mut game, "realm god");
        assert!(god.to_lowercase().contains("god"), "{god}");
        // No --dev-tools: nothing, whatever the role.
        game.set_dev_tools(false);
        assert!(!try_cmd(&mut game, "realm mark other").contains("Marked"));
        std::fs::remove_dir_all(&world).unwrap();
    }

    #[test]
    fn account_roles_run_realm_commands_without_dev_tools() {
        use crate::account_store::AccountRole;
        let content = content_directory();
        let world = std::env::temp_dir().join(format!("osfr-role-cmds-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&world);
        let mut game = Gameplay::load(&content, None).unwrap();
        game.set_world_dir(Some(world.clone())).unwrap();
        let start = character();
        let try_cmd = |game: &mut Gameplay, text: &str| {
            let out = game.handle_in_world(&chat(text), &start, 1_000).unwrap();
            (reply_text(&out), out.dev_command_denied)
        };
        // Canary: neither flag, even an admin runs nothing.
        game.set_role(AccountRole::Admin);
        assert!(!try_cmd(&mut game, "realm mark spot").0.contains("Marked"));
        game.set_role_commands(true);
        game.set_role(AccountRole::Player);
        let (text, denied) = try_cmd(&mut game, "realm mark spot");
        assert!(denied && text.contains("account role"), "{text}");
        assert!(!world.join("marks.json").exists());
        game.set_role(AccountRole::Builder);
        assert!(
            try_cmd(&mut game, "realm mark spot")
                .0
                .contains("Marked spot")
        );
        assert!(try_cmd(&mut game, "realm god").1, "builder gets no cheats");
        game.set_role(AccountRole::Admin);
        assert!(
            try_cmd(&mut game, "realm god")
                .0
                .to_lowercase()
                .contains("god")
        );
        // Wire experiments stay behind --dev-tools for every role.
        assert!(try_cmd(&mut game, "realm mine").0.contains("--dev-tools"));
        std::fs::remove_dir_all(&world).unwrap();
    }

    #[test]
    fn realm_marks_save_list_return_and_remove() {
        let content = content_directory();
        let mut game = Gameplay::load(&content, None).unwrap();
        game.set_dev_tools(true);
        game.set_role(crate::account_store::AccountRole::Admin);
        let dir = std::env::temp_dir().join(format!("osfr-marks-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let _ = std::fs::remove_file(dir.join("marks.json"));
        game.content_dir = dir.clone();
        let start = character();
        let out = game
            .handle_in_world(&chat("realm mark Troll Bridge"), &start, 1_000)
            .unwrap();
        assert!(
            reply_text(&out).contains("Marked troll bridge"),
            "{}",
            reply_text(&out)
        );
        let list = game
            .handle_in_world(&chat("realm marks"), &start, 1_001)
            .unwrap();
        assert!(
            reply_text(&list).contains("troll bridge (zone 0"),
            "{}",
            reply_text(&list)
        );
        let elsewhere = CharacterState {
            position: [
                start.position[0] + 50.0,
                start.position[1],
                start.position[2],
            ],
            ..start.clone()
        };
        let back = game
            .handle_in_world(&chat("realm goto mark troll bridge"), &elsewhere, 1_002)
            .unwrap();
        assert_eq!(back.relocation.unwrap().position, start.position);
        let gone = game
            .handle_in_world(&chat("realm unmark troll bridge"), &start, 1_003)
            .unwrap();
        assert!(
            reply_text(&gone).contains("Removed mark"),
            "{}",
            reply_text(&gone)
        );
        let missing = game
            .handle_in_world(&chat("realm goto mark troll bridge"), &start, 1_004)
            .unwrap();
        assert!(missing.relocation.is_none());
        assert!(reply_text(&missing).contains("No mark named"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn realm_goto_reaches_and_interacts_with_zone_less_roosey() {
        const ROOSEY_ID: u64 = 9_500_000_255;
        let content = content_directory();
        let mut game = Gameplay::load(&content, None).unwrap();
        game.set_dev_tools(true);
        game.set_role(crate::account_store::AccountRole::Admin);
        let start = character();
        let moved = game
            .handle_in_world(&chat("realm goto Roosey"), &start, 1_000)
            .unwrap();
        let relocation = moved.relocation.expect("Roosey is in realm goto targets");
        let near_roosey = CharacterState {
            guid: start.guid,
            zone_id: start.zone_id,
            position: relocation.position,
            rotation: relocation.rotation,
        };
        let interact = npc_interaction::NpcInteractRequest {
            target: ROOSEY_ID,
            trailing_flag: false,
        }
        .encode();
        let menu = game
            .handle_in_world(&interact, &near_roosey, 1_001)
            .unwrap();
        assert!(
            menu.replies
                .iter()
                .any(|packet| packet.starts_with(&[26, 0, 9, 0]))
        );
    }

    #[test]
    fn realm_npx_off_emits_player_enter_clear_only_with_dev_tools() {
        let content = content_directory();
        let mut game = Gameplay::load(&content, None).unwrap();
        game.set_dev_tools(true);
        game.set_role(crate::account_store::AccountRole::Admin);
        let out = game
            .handle_in_world(&chat("realm npx off"), &character(), 1_000)
            .unwrap();

        let mut expected = vec![41, 0, 2, 0];
        expected.extend([0; 17]);
        assert!(out.replies.iter().any(|packet| packet == &expected));
    }

    #[test]
    fn realm_npx_off_does_not_emit_player_enter_without_dev_tools() {
        let content = content_directory();
        let mut game = Gameplay::load(&content, None).unwrap();
        let out = game
            .handle_in_world(&chat("realm npx off"), &character(), 1_000)
            .unwrap();

        assert!(
            !out.replies
                .iter()
                .any(|packet| packet.starts_with(&[41, 0, 2, 0]))
        );
    }

    /// A 98/1 breadcrumb request from `from` to `to` (layout: `pathing.rs`).
    fn path_request(from: [f32; 3], to: [f32; 3]) -> Vec<u8> {
        let mut p = vec![98, 0, 1];
        p.extend(1u64.to_le_bytes());
        p.extend(1u32.to_le_bytes());
        p.extend(0u32.to_le_bytes());
        for v in [from, to] {
            p.extend(v.iter().chain(&[1.0]).flat_map(|f| f.to_le_bytes()));
        }
        p.extend(0u32.to_le_bytes());
        p
    }

    fn path_replies(out: &Outcome) -> Vec<&Vec<u8>> {
        out.replies
            .iter()
            .filter(|r| r.starts_with(&[98, 0, 2]))
            .collect()
    }

    #[test]
    fn path_request_is_answered_from_the_terrain() {
        let content = content_directory();
        let mut game = Gameplay::load(&content, None).unwrap();
        game.set_terrain(Some(flat_terrain("path", 42.0)));
        let start = character();
        let goal = [start.position[0] + 90.0, 42.0, start.position[2]];
        let out = game
            .handle_in_world(&path_request(start.position, goal), &start, 1_000)
            .unwrap();
        let replies = path_replies(&out);
        assert_eq!(replies.len(), 1, "one 98/2 reply");
        let reply = replies[0];
        // kind 1, aux 0, then 19 points (90 m every 5 m); interior points sit on the ground.
        assert_eq!(&reply[3..11], &[1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(&reply[11..15], &19i32.to_le_bytes());
        let y = |i: usize| f32::from_le_bytes(reply[19 + i * 16..23 + i * 16].try_into().unwrap());
        assert!((y(5) - 42.0).abs() < 0.1, "{}", y(5));
    }

    #[test]
    fn path_request_without_terrain_gets_the_failure_reply() {
        let content = content_directory();
        let mut game = Gameplay::load(&content, None).unwrap();
        let start = character();
        let goal = [start.position[0] + 50.0, 4.0, start.position[2]];
        let out = game
            .handle_in_world(&path_request(start.position, goal), &start, 1_000)
            .unwrap();
        let expected = [98, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(out.replies, vec![expected.to_vec()]);
    }

    #[test]
    fn path_requests_are_capped_per_second_and_refusals_are_silent() {
        let content = content_directory();
        let mut game = Gameplay::load(&content, None).unwrap();
        game.set_terrain(Some(flat_terrain("path-rate", 42.0)));
        let start = character();
        let goal = [start.position[0] + 50.0, 42.0, start.position[2]];
        let ask = |game: &mut Gameplay, packet: &[u8], now| {
            game.handle_in_world(packet, &start, now).unwrap()
        };
        let valid = path_request(start.position, goal);
        let answered = (0..8)
            .filter(|i| !path_replies(&ask(&mut game, &valid, 5_000 + i)).is_empty())
            .count();
        assert_eq!(answered, crate::pathing::MAX_PER_SECOND as usize);
        assert!(
            !path_replies(&ask(&mut game, &valid, 6_000)).is_empty(),
            "next second"
        );
        // Too far, and a truncated packet: no reply and no error.
        let far = path_request(start.position, [start.position[0] + 400.0, 0.0, 0.0]);
        assert!(ask(&mut game, &far, 9_000).replies.is_empty());
        assert!(ask(&mut game, &valid[..20], 9_000).replies.is_empty());
        // The POI teleport request (kind 2) is not this module's.
        let mut poi = valid.clone();
        poi[11] = 2;
        assert!(path_replies(&ask(&mut game, &poi, 9_000)).is_empty());
    }
}

#[cfg(test)]
mod ordinary_quit_routing {
    use super::*;
    use std::path::PathBuf;

    fn content_directory() -> PathBuf {
        std::env::var_os("OSFR_OPENING_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../../lab/references/rust-opening-cooking-original-assets")
            })
    }
    fn character() -> CharacterState {
        CharacterState {
            guid: 7,
            zone_id: 0,
            position: [0.0; 3],
            rotation: [0.0; 3],
        }
    }
    fn quit(instance: i32) -> Vec<u8> {
        let mut b = vec![39, 0, 6];
        for n in [instance, -1, -1] {
            b.extend(n.to_le_bytes());
        }
        b
    }
    fn remove(instance: i32) -> Vec<u8> {
        let mut b = vec![39, 0, 19];
        for n in [instance, -1, 46] {
            b.extend(n.to_le_bytes());
        }
        b
    }
    fn snapshot(game: &Gameplay) -> serde_json::Value {
        serde_json::to_value(game.state()).unwrap()
    }

    #[test]
    fn ordinary_cooking_quit_cancels_atomically_and_malformed_replays_roll_back() {
        let content = content_directory();
        assert!(content.join("cooking-campaign.json").is_file());
        let mut game = Gameplay::load(&content, None).unwrap();
        let character = character();
        game.cooking.as_mut().unwrap().begin(45).unwrap();
        let instance = game
            .cooking
            .as_ref()
            .unwrap()
            .state()
            .active
            .as_ref()
            .unwrap()
            .instance_id;
        let before = snapshot(&game);
        let out = game
            .handle_in_world(&quit(instance), &character, 1_000)
            .unwrap();
        let after = snapshot(&game);
        assert_eq!(out.replies, vec![remove(instance)]);
        assert!(after["cooking"]["active"].is_null());
        assert_eq!(before["inventory"], after["inventory"]);
        assert_eq!(before["quests"], after["quests"]);
        assert_eq!(before["opening"], after["opening"]);
        assert_eq!(
            before["cooking"]["completed"],
            after["cooking"]["completed"]
        );
        assert_eq!(
            before["cooking"]["next_instance_id"],
            after["cooking"]["next_instance_id"]
        );

        // A replay of the stale packet is not a second cancellation.
        let replay = game
            .handle_in_world(&quit(instance), &character, 2_000)
            .unwrap();
        assert!(replay.replies.is_empty());
        assert_eq!(snapshot(&game), after);

        // A malformed recognized packet rejects before the staged bundle commits.
        game.cooking.as_mut().unwrap().begin(45).unwrap();
        let live = snapshot(&game);
        let mut malformed = quit(instance + 1);
        malformed.push(0);
        assert!(game.handle_in_world(&malformed, &character, 3_000).is_err());
        assert_eq!(snapshot(&game), live);
    }
}

#[cfg(test)]
mod w64_regional_quest_tests {
    use super::*;

    const SAMANTHA_ID: u64 = 9_000_000_189;
    const INTRODUCE_YOURSELF_ID: u32 = 3_000_021;

    #[test]
    fn live_quest_offers_require_real_level_and_the_active_job() {
        let source = bundled_content();
        let scratch = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../../lab/build/w120/eligibility-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&scratch).unwrap();
        std::fs::copy(source.join("jobs.json"), scratch.join("jobs.json")).unwrap();
        for folder in ["quests", "npcs"] {
            std::fs::create_dir_all(scratch.join(folder)).unwrap();
            for entry in std::fs::read_dir(source.join(folder)).unwrap() {
                let path = entry.unwrap().path();
                std::fs::copy(&path, scratch.join(folder).join(path.file_name().unwrap())).unwrap();
            }
        }
        let path = scratch.join("quests/adventurer.json");
        let mut file: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let set_requirement = |file: &mut serde_json::Value, job: Option<i32>| {
            let quest = file["records"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|q| q["id"] == INTRODUCE_YOURSELF_ID)
                .unwrap();
            quest["min_level"] = serde_json::json!(3);
            quest["job"] = serde_json::json!(job);
            std::fs::write(&path, serde_json::to_vec(file).unwrap()).unwrap();
        };
        set_requirement(&mut file, None);
        let mut game = Gameplay::load(&reference(), None).unwrap();
        game.profiles = Some(Profiles::load(&reference(), Some(native_test_profiles())).unwrap());
        game.bind_npc_quest_catalogue(&scratch, 1).unwrap();
        assert_eq!(
            game.npc_quests.as_ref().unwrap().eligibility().player_level,
            1,
            "the active job starts at rank 1"
        );
        assert!(
            !game
                .npc_quests
                .as_ref()
                .unwrap()
                .can_offer(INTRODUCE_YOURSELF_ID, SAMANTHA_ID)
        );
        game.profiles.as_mut().unwrap().set_experience(200);
        game.refresh_npc_quest_eligibility(0);
        assert_eq!(
            game.npc_quests.as_ref().unwrap().eligibility().player_level,
            3,
            "eligibility follows the active job's XP"
        );
        assert!(
            game.npc_quests
                .as_mut()
                .unwrap()
                .offer(INTRODUCE_YOURSELF_ID, SAMANTHA_ID)
                .changed
        );

        set_requirement(&mut file, Some(43));
        let mut profile = game.profiles.as_ref().unwrap().state().clone();
        assert!(profile.unlocked.contains(&43));
        assert_ne!(profile.active, 43);
        profile.experience.insert(43, 200);
        game.profiles = Some(Profiles::load(&reference(), Some(profile)).unwrap());
        game.bind_npc_quest_catalogue(&scratch, 1).unwrap();
        assert!(
            !game
                .npc_quests
                .as_ref()
                .unwrap()
                .can_offer(INTRODUCE_YOURSELF_ID, SAMANTHA_ID)
        );
        game.profiles
            .as_mut()
            .unwrap()
            .select(43, &game.inventory)
            .unwrap();
        game.refresh_npc_quest_eligibility(0);
        assert_eq!(
            game.npc_quests.as_ref().unwrap().eligibility().player_level,
            3,
            "eligibility follows the newly active job's XP"
        );
        assert!(
            game.npc_quests
                .as_mut()
                .unwrap()
                .offer(INTRODUCE_YOURSELF_ID, SAMANTHA_ID)
                .changed
        );
        std::fs::remove_dir_all(scratch).unwrap();
    }

    fn level_three_world_game(label: &str) -> (Gameplay, CharacterState, PathBuf) {
        let source = bundled_content();
        let scratch = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../../lab/build/f111/{label}-{}",
            std::process::id()
        ));
        if scratch.exists() {
            std::fs::remove_dir_all(&scratch).unwrap();
        }
        std::fs::create_dir_all(&scratch).unwrap();
        std::fs::copy(source.join("jobs.json"), scratch.join("jobs.json")).unwrap();
        for folder in ["quests", "npcs"] {
            std::fs::create_dir_all(scratch.join(folder)).unwrap();
            for entry in std::fs::read_dir(source.join(folder)).unwrap() {
                let path = entry.unwrap().path();
                std::fs::copy(&path, scratch.join(folder).join(path.file_name().unwrap())).unwrap();
            }
        }
        let path = scratch.join("quests/adventurer.json");
        let mut file: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let quest = file["records"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|quest| quest["id"] == INTRODUCE_YOURSELF_ID)
            .unwrap();
        quest["min_level"] = serde_json::json!(3);
        quest["job"] = serde_json::Value::Null;
        std::fs::write(&path, serde_json::to_vec(&file).unwrap()).unwrap();

        let mut game = Gameplay::load(&reference(), None).unwrap();
        game.profiles = Some(Profiles::load(&reference(), Some(native_test_profiles())).unwrap());
        // Exercise the non-opening request path, which does not refresh eligibility
        // while switching jobs. Only the NPC request path can make the offer legal.
        game.opening = None;
        game.bind_npc_quest_catalogue(&scratch, 1).unwrap();

        let npc_file: serde_json::Value = serde_json::from_slice(
            &std::fs::read(source.join("npcs/cobblestone_village.json")).unwrap(),
        )
        .unwrap();
        let giver = npc_file["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["npc_id"].as_u64() == Some(SAMANTHA_ID))
            .unwrap();
        let position: [f32; 3] = serde_json::from_value(giver["position"].clone()).unwrap();
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position,
            rotation: [1.0, 0.0, 0.0],
        };
        game.update_npc_visibility(&character).unwrap();
        (game, character, scratch)
    }

    #[test]
    fn npc_request_path_refreshes_eligibility_after_a_request_handler_job_switch() {
        let (mut game, character, scratch) = level_three_world_game("npc-refresh");
        let mut switch = vec![26, 0, 13, 0];
        switch.extend(43i32.to_le_bytes());
        assert!(
            game.handle_in_world(&switch, &character, 1_000)
                .unwrap()
                .changed
        );
        assert_eq!(game.profiles.as_ref().unwrap().active(), 43);

        game.handle_in_world(&packet_for_npc(SAMANTHA_ID), &character, 1_001)
            .unwrap();
        game.handle_in_world(&select_npc_option(SAMANTHA_ID, 1), &character, 1_002)
            .unwrap();
        let chosen = game
            .handle_in_world(&choose_quest_row(), &character, 1_003)
            .unwrap();
        assert!(
            chosen
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0, 1, 0])),
            "the NPC request path must refresh eligibility after the job switch"
        );
        assert_eq!(
            game.npc_quests.as_ref().unwrap().state().pending_offer,
            Some(INTRODUCE_YOURSELF_ID)
        );
        std::fs::remove_dir_all(scratch).unwrap();
    }

    #[test]
    fn denied_accept_after_a_request_handler_job_switch_changes_nothing_and_requests_no_save() {
        let (mut game, character, scratch) = level_three_world_game("npc-denied");
        let mut switch = vec![26, 0, 13, 0];
        switch.extend(43i32.to_le_bytes());
        assert!(
            game.handle_in_world(&switch, &character, 1_000)
                .unwrap()
                .changed
        );
        assert_eq!(game.profiles.as_ref().unwrap().active(), 43);

        game.handle_in_world(&packet_for_npc(SAMANTHA_ID), &character, 1_001)
            .unwrap();
        game.handle_in_world(&select_npc_option(SAMANTHA_ID, 1), &character, 1_002)
            .unwrap();
        let offered = game
            .handle_in_world(&choose_quest_row(), &character, 1_003)
            .unwrap();
        assert!(
            offered
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0, 1, 0]))
        );

        let mut back = vec![26, 0, 13, 0];
        back.extend(48i32.to_le_bytes());
        assert!(
            game.handle_in_world(&back, &character, 2_000)
                .unwrap()
                .changed
        );
        let before = game.npc_quests.as_ref().unwrap().state().clone();
        assert_eq!(before.pending_offer, Some(INTRODUCE_YOURSELF_ID));
        assert_eq!(
            before.quests[&INTRODUCE_YOURSELF_ID].status,
            crate::quest_engine::QuestStatus::Offered
        );

        let mut accept = vec![49, 0, 2, 0, 0, 0];
        accept.extend(INTRODUCE_YOURSELF_ID.to_le_bytes());
        accept.push(1);
        let refused = game.handle_in_world(&accept, &character, 3_000).unwrap();
        // Gateway (src/gateway.rs:1758-1759) requests persistence only from `changed`.
        assert!(!refused.changed, "a denied accept must not request a save");
        assert!(refused.relocation.is_none());
        assert!(refused.zoning.is_none());
        assert!(refused.scene_zoning.is_none());
        assert!(!refused.pets_changed);
        assert_eq!(game.npc_quests.as_ref().unwrap().state(), &before);
        assert!(
            !refused
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0, 3, 0])),
            "a denied accept must not activate the quest"
        );
        std::fs::remove_dir_all(scratch).unwrap();
    }

    fn reference() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-opening-cooking-original-assets")
    }

    fn bundled_content() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
    }

    fn native_test_profiles() -> ProfileState {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/reports/round5b-native-20261014T053834Z/character.json");
        let saved: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).expect("native test save exists"))
                .expect("native test save parses");
        serde_json::from_value(saved["gameplay"]["profiles"].clone())
            .expect("native gameplay snapshot has the profile state")
    }

    fn packet_for_npc(npc_id: u64) -> Vec<u8> {
        npc_interaction::NpcInteractRequest {
            target: npc_id,
            trailing_flag: false,
        }
        .encode()
    }

    fn select_npc_option(npc_id: u64, option_id: i32) -> Vec<u8> {
        let mut packet = vec![26, 0, 10, 0];
        packet.extend(npc_id.to_le_bytes());
        packet.extend(option_id.to_le_bytes());
        packet
    }

    fn offer_chat(text: &str) -> Vec<u8> {
        let mut packet = vec![15, 0, 1, 0, 0, 0];
        packet.extend(1_u64.to_le_bytes());
        packet.extend(0_u64.to_le_bytes());
        for _ in 0..2 {
            packet.extend([0; 12]);
            packet.extend(0_u32.to_le_bytes());
            packet.extend(0_u32.to_le_bytes());
        }
        packet.extend((text.len() as u32).to_le_bytes());
        packet.extend(text.as_bytes());
        packet.extend([0; 28]);
        packet
    }

    fn expected_samantha_dialogue() -> Vec<u8> {
        // Quest choice row ("Introduce Yourself") first, then Goodbye. Dwords 2-3
        // carry Samantha's GUID as the dialog camera target.
        let mut packet = vec![26, 0, 3, 0];
        packet.extend(24_940_u32.to_le_bytes()); // Samantha's greeting
        packet.extend([0; 4]);
        packet.extend(SAMANTHA_ID.to_le_bytes());
        packet.extend([0; 5]);
        packet.extend(2_u32.to_le_bytes());
        for value in [301_u32, 17, 94_247, 0, 18, 300, 17, 8075, 0, 18] {
            packet.extend(value.to_le_bytes());
        }
        packet.extend([0; 64]);
        assert_eq!(packet.len(), 133);
        packet
    }

    fn expected_blank_dialogue() -> Vec<u8> {
        // Diagnostic variant v4: Goodbye only, no quest ids.
        let mut packet = vec![26, 0, 3, 0];
        packet.extend([0; 21]);
        packet.extend(1_u32.to_le_bytes());
        for value in [300_u32, 17, 8075, 0, 18] {
            packet.extend(value.to_le_bytes());
        }
        packet.extend([0; 64]);
        packet
    }

    fn choose_quest_row() -> Vec<u8> {
        let mut packet = vec![26, 0, 6, 0];
        packet.extend(npc_interaction::QUEST_CHOICE_RESPONSE.to_le_bytes());
        packet
    }

    fn expected_quest_offer(_prefix: u16, intro_id: u32) -> Vec<u8> {
        // Native layout (run 142216): no u16 prefix; quest id + five dwords; 114 bytes.
        let mut packet = vec![49, 0, 1, 0, 0, 0];
        packet.extend(INTRODUCE_YOURSELF_ID.to_le_bytes());
        for value in [intro_id, 94_247, 94_247, 94_248, 94_247] {
            packet.extend(value.to_le_bytes());
        }
        packet.push(0);
        packet.extend([0; 69]);
        packet.extend([0; 14]);
        assert_eq!(packet.len(), 114);
        packet
    }

    fn offer_test_game(dev_tools: bool) -> (Gameplay, CharacterState) {
        let content = bundled_content();
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        game.profiles = Some(Profiles::load(&reference, Some(native_test_profiles())).unwrap());
        game.bind_npc_quest_catalogue(&content, 1).unwrap();
        game.set_dev_tools(dev_tools);
        game.set_role(crate::account_store::AccountRole::Admin);

        let npc_file: serde_json::Value = serde_json::from_slice(
            &std::fs::read(content.join("npcs/cobblestone_village.json")).unwrap(),
        )
        .unwrap();
        let samantha = npc_file["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["npc_id"].as_u64() == Some(SAMANTHA_ID))
            .unwrap();
        let position: [f32; 3] = serde_json::from_value(samantha["position"].clone()).unwrap();
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position,
            rotation: [1.0, 0.0, 0.0],
        };
        game.update_npc_visibility(&character).unwrap();
        game.handle_in_world(&packet_for_npc(SAMANTHA_ID), &character, 1_000)
            .unwrap();
        let talk = game
            .handle_in_world(&select_npc_option(SAMANTHA_ID, 1), &character, 1_001)
            .unwrap();
        // Talk opens the dialog with the quest as a choice; choosing it sends 49/1.
        assert!(!talk.replies.iter().any(|p| p.starts_with(&[49, 0, 1, 0])));
        assert!(
            talk.replies
                .iter()
                .any(|p| p == &expected_samantha_dialogue())
        );
        let chosen = game
            .handle_in_world(&choose_quest_row(), &character, 1_002)
            .unwrap();
        assert!(chosen.replies.iter().any(|p| p.starts_with(&[49, 0, 1, 0])));
        assert_eq!(
            game.last_npc_dialogue
                .as_ref()
                .map(|(_, _, _, packet)| packet),
            Some(&expected_samantha_dialogue())
        );
        (game, character)
    }

    #[test]
    fn journal_requests_route_through_the_npc_quest_engine() {
        let (mut game, character) = offer_test_game(false);
        let mut accept = vec![49, 0, 2, 0, 0, 0];
        accept.extend((INTRODUCE_YOURSELF_ID as i32).to_le_bytes());
        accept.push(1);
        game.handle_in_world(&accept, &character, 1_005).unwrap();
        let state = |game: &Gameplay| game.npc_quests.as_ref().unwrap().state().clone();

        // 47/12 selects the owned active quest and the state is marked for saving.
        let mut select = vec![47, 0, 12];
        select.extend((INTRODUCE_YOURSELF_ID as i32).to_le_bytes());
        let out = game.handle_in_world(&select, &character, 1_006).unwrap();
        assert!(out.changed);
        assert_eq!(state(&game).selected_quest_id, Some(INTRODUCE_YOURSELF_ID));

        // A forged id changes nothing and answers nothing; a malformed one is an error.
        let before = state(&game);
        let mut forged = vec![47, 0, 12];
        forged.extend(999_999i32.to_le_bytes());
        let out = game.handle_in_world(&forged, &character, 1_007).unwrap();
        assert!(!out.changed && out.replies.is_empty());
        assert_eq!(state(&game), before);
        assert!(
            game.handle_in_world(&[47, 0, 12, 1, 0, 0], &character, 1_008)
                .is_err()
        );

        // 47/13: wire 0 = unlocked = client auto-switch set; wire 1 = locked.
        assert!(
            game.handle_in_world(&[47, 0, 13, 0], &character, 1_009)
                .unwrap()
                .changed
        );
        assert!(state(&game).auto_switch_selected);
        assert!(
            game.handle_in_world(&[47, 0, 13, 1], &character, 1_010)
                .unwrap()
                .changed
        );
        assert!(!state(&game).auto_switch_selected);

        // 49/15 for an id the server never registered offers nothing.
        let before = state(&game);
        let mut crumb = vec![49, 0, 15, 0, 0, 0];
        crumb.extend(999_999i32.to_le_bytes());
        let out = game.handle_in_world(&crumb, &character, 1_011).unwrap();
        assert!(!out.changed);
        // W175: an unlisted original id is refused with one chat line.
        assert_eq!(
            out.replies,
            vec![chat::system(crate::quest_journal::BREADCRUMB_REFUSED)]
        );
        assert_eq!(state(&game), before);
    }

    // ---- W175: the 49/15 original-id crosswalk and 45/7 client-reported goals ----

    /// "Introduce Yourself" at Highroad Junction: enabled, no prerequisite, no level gate.
    const CROSSWALK_QUEST: u32 = 3_000_022;
    /// "Farming 101": disabled in the catalogue today, so the engine refuses to offer it.
    const DISABLED_QUEST: u32 = 3_000_006;

    fn quest_game() -> (Gameplay, CharacterState) {
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        game.profiles = Some(Profiles::load(&reference, Some(native_test_profiles())).unwrap());
        game.bind_npc_quest_catalogue(&bundled_content(), 1)
            .unwrap();
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.0; 3],
            rotation: [1.0, 0.0, 0.0],
        };
        (game, character)
    }

    fn breadcrumb_packet(quest_id: u32) -> Vec<u8> {
        let mut packet = vec![49, 0, 15, 0, 0, 0];
        packet.extend((quest_id as i32).to_le_bytes());
        packet
    }

    fn claim_packet(client_id: i32) -> Vec<u8> {
        let mut packet = vec![45, 0, 7];
        packet.extend(client_id.to_le_bytes());
        packet
    }

    fn quest_state(game: &Gameplay) -> crate::quest_engine::QuestState {
        game.npc_quests.as_ref().unwrap().state().clone()
    }

    #[test]
    fn binding_the_catalogue_loads_the_shipped_crosswalk() {
        let (game, _) = quest_game();
        assert!(!game.breadcrumb_crosswalk.is_empty());
        // Every listed target has a definition, so every one is registered with its giver.
        let targets: BTreeSet<u32> = game.breadcrumb_crosswalk.values().copied().collect();
        let registered: BTreeSet<u32> = game.breadcrumb_quests.keys().copied().collect();
        assert_eq!(targets, registered);
    }

    #[test]
    fn breadcrumb_start_goes_through_the_original_id_crosswalk() {
        let (mut game, character) = quest_game();
        game.register_breadcrumb_crosswalk(std::collections::BTreeMap::from([
            (1807, CROSSWALK_QUEST),
            (1806, DISABLED_QUEST),
        ]));
        let refusal = chat::system(crate::quest_journal::BREADCRUMB_REFUSED);
        let before = quest_state(&game);

        // An unlisted original id, a regional id used as an original one and a forged id are
        // each answered with one chat line and change nothing. So does a listed id whose
        // quest the engine will not offer (disabled content).
        for (step, id) in [1808, CROSSWALK_QUEST, 999_999, 1806]
            .into_iter()
            .enumerate()
        {
            let out = game
                .handle_in_world(&breadcrumb_packet(id), &character, 2_000 + step as u64)
                .unwrap();
            assert!(!out.changed, "id {id}");
            assert_eq!(out.replies, vec![refusal.clone()], "id {id}");
            assert_eq!(quest_state(&game), before, "id {id}");
        }

        // A listed original id offers its REGIONAL quest (49/1), once.
        let out = game
            .handle_in_world(&breadcrumb_packet(1807), &character, 2_010)
            .unwrap();
        assert!(out.changed);
        assert!(out.replies.iter().any(|p| p.starts_with(&[49, 0, 1, 0])));
        assert!(!out.replies.contains(&refusal));
        let offered = quest_state(&game);
        assert_eq!(offered.pending_offer, Some(CROSSWALK_QUEST));
        assert_eq!(offered.quests.len(), 1);
        // Pressed again with the offer still open: shown again, nothing new made.
        let again = game
            .handle_in_world(&breadcrumb_packet(1807), &character, 2_011)
            .unwrap();
        assert!(!again.changed);
        assert!(again.replies.iter().any(|p| p.starts_with(&[49, 0, 1, 0])));
        assert_eq!(quest_state(&game), offered);
    }

    #[test]
    fn an_unlisted_breadcrumb_is_refused_without_a_quest_engine() {
        let mut game = Gameplay::load(&reference(), None).unwrap();
        assert!(game.npc_quests.is_none());
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.0; 3],
            rotation: [1.0, 0.0, 0.0],
        };
        let out = game
            .handle_in_world(&breadcrumb_packet(1807), &character, 6_000)
            .unwrap();
        assert!(!out.changed);
        assert_eq!(
            out.replies,
            vec![chat::system(crate::quest_journal::BREADCRUMB_REFUSED)]
        );
    }

    /// Two synthetic quests that each ask the client to judge one goal.
    fn client_goal_engine() -> crate::quest_engine::QuestEngine {
        use crate::quest_engine::{
            CharacterEligibility, ClientTrigger, ObjectiveKind, QuestDefinition, QuestEngine,
            QuestObjective, QuestRequirements,
        };
        let quest = |id: u32, client_id: u32, trigger: ClientTrigger| QuestDefinition {
            id,
            title: format!("Client goal {id} (R)"),
            description: String::new(),
            giver_npc_id: 1,
            turnin_npc_id: 1,
            prerequisite_quest_ids: vec![],
            requirements: QuestRequirements::default(),
            ordered: true,
            objectives: vec![QuestObjective {
                id: 1,
                required: 1,
                description: "Judged by the client (R)".into(),
                kind: ObjectiveKind::ClientReported { client_id, trigger },
            }],
            reward: crate::rewards::RewardGrant {
                coins: 5,
                items: vec![],
                job_xp: std::collections::BTreeMap::new(),
                display: crate::rewards::RewardDisplay {
                    source_guid: 1,
                    player_guid: 1,
                    reward_kind: 0,
                    icon_id: 0,
                    name_id: 0,
                    rows: vec![],
                },
            },
        };
        QuestEngine::new(
            1,
            vec![
                quest(
                    7001,
                    55,
                    ClientTrigger::UiEvent {
                        event: "onTestEvent".into(),
                    },
                ),
                quest(7002, 77, ClientTrigger::FirstMovement { mouse: true }),
                quest(
                    7003,
                    73,
                    ClientTrigger::LookAt {
                        guid: 0x1122_3344_5566_7788,
                        offset: 1.5,
                        range: 12,
                        angle_degrees: 45,
                    },
                ),
            ],
            CharacterEligibility {
                member: true,
                member_quest_limit: 10,
                nonmember_quest_limit: 10,
                ..Default::default()
            },
            None,
        )
        .unwrap()
    }

    fn engine_mut(game: &mut Gameplay) -> &mut crate::quest_engine::QuestEngine {
        game.npc_quests.as_mut().unwrap()
    }

    #[test]
    fn client_goal_claims_move_only_an_open_goal_of_an_active_quest() {
        let (mut game, character) = quest_game();
        game.npc_quests = Some(client_goal_engine());

        // Nothing is active yet: a claim for a real id changes nothing and registers nothing.
        let out = game
            .handle_in_world(&claim_packet(55), &character, 3_000)
            .unwrap();
        assert!(!out.changed && out.replies.is_empty());

        // 7001 becomes active; 7002 stays offered.
        assert!(engine_mut(&mut game).offer(7001, 1).changed);
        assert!(engine_mut(&mut game).accept(7001).changed);
        assert!(engine_mut(&mut game).offer(7002, 1).changed);
        let before = quest_state(&game);

        // A forged id moves nothing; the client is told to watch goal 55 (45/10, exact bytes)
        // and not goal 77, whose quest is only offered.
        let mut watch = vec![45, 0, 10];
        watch.extend(55u32.to_le_bytes());
        watch.extend(11u32.to_le_bytes());
        watch.extend(b"onTestEvent");
        let out = game
            .handle_in_world(&claim_packet(99), &character, 3_001)
            .unwrap();
        assert!(!out.changed);
        assert_eq!(out.replies, vec![watch]);
        assert_eq!(quest_state(&game), before);

        // A claim for the goal of the merely offered quest changes nothing.
        let out = game
            .handle_in_world(&claim_packet(77), &character, 3_002)
            .unwrap();
        assert!(!out.changed && out.replies.is_empty());
        assert_eq!(quest_state(&game), before);

        // Malformed claims are errors and change nothing.
        for bad in [
            claim_packet(0),
            claim_packet(-1),
            claim_packet(i32::MIN),
            vec![45, 0, 7, 55, 0, 0],
            vec![45, 0, 7, 55, 0, 0, 0, 0],
            vec![45, 0, 7],
        ] {
            assert!(game.handle_in_world(&bad, &character, 3_003).is_err());
            assert_eq!(quest_state(&game), before);
        }

        // The real claim advances the goal once: update and completion are sent, the quest
        // waits for its turn-in and nothing is paid.
        let out = game
            .handle_in_world(&claim_packet(55), &character, 3_004)
            .unwrap();
        assert!(out.changed);
        assert!(
            out.replies
                .iter()
                .any(|p| p.starts_with(&[49, 0, 9, 0, 0, 0]))
        );
        assert!(
            out.replies
                .iter()
                .any(|p| p.starts_with(&[49, 0, 10, 0, 0, 0]))
        );
        assert!(!out.replies.iter().any(|p| p.starts_with(&[45, 0, 10])));
        let state = quest_state(&game);
        assert_eq!(state.quests[&7001].counts, vec![1]);
        assert_eq!(
            state.quests[&7001].status,
            crate::quest_engine::QuestStatus::Active
        );
        assert_eq!(state.quests[&7002].counts, vec![0]);

        // A repeat, and a claim for the other quest's goal, change nothing.
        for id in [55, 77] {
            let out = game
                .handle_in_world(&claim_packet(id), &character, 3_005)
                .unwrap();
            assert!(!out.changed && out.replies.is_empty());
            assert_eq!(quest_state(&game), state);
        }
    }
    #[test]
    fn a_look_goal_claim_advances_only_an_open_goal_of_an_active_quest() {
        let (mut game, character) = quest_game();
        game.npc_quests = Some(client_goal_engine());
        assert!(engine_mut(&mut game).offer(7003, 1).changed);
        assert!(engine_mut(&mut game).accept(7003).changed);
        let before = quest_state(&game);

        // The client receives the recovered 45/6 actor form: id, GUID, raw vertical
        // offset, range, and angle in degrees.
        let out = game
            .handle_in_world(&claim_packet(99), &character, 5_000)
            .unwrap();
        let mut watch = vec![45, 0, 6];
        watch.extend(73u32.to_le_bytes());
        watch.extend(0x1122_3344_5566_7788u64.to_le_bytes());
        watch.extend(1.5f32.to_le_bytes());
        watch.extend(12u32.to_le_bytes());
        watch.extend(45u32.to_le_bytes());
        assert_eq!(watch.len(), 27);
        assert_eq!(out.replies, vec![watch]);
        assert_eq!(quest_state(&game), before);

        let out = game
            .handle_in_world(&claim_packet(73), &character, 5_001)
            .unwrap();
        assert!(out.changed);
        assert!(
            out.replies
                .iter()
                .any(|p| p.starts_with(&[49, 0, 10, 0, 0, 0]))
        );
        assert!(!out.replies.iter().any(|p| p.starts_with(&[50, 0, 1])));
        let state = quest_state(&game);
        assert_eq!(state.quests[&7003].counts, vec![1]);
        assert_eq!(
            state.quests[&7003].status,
            crate::quest_engine::QuestStatus::Active
        );

        // Claim replay changes nothing.
        let replay = game
            .handle_in_world(&claim_packet(73), &character, 5_002)
            .unwrap();
        assert!(!replay.changed && replay.replies.is_empty());
        assert_eq!(quest_state(&game), state);
    }

    #[test]
    fn a_goal_that_opens_again_is_rearmed_not_registered_twice() {
        let (mut game, character) = quest_game();
        game.npc_quests = Some(client_goal_engine());
        let watching = |out: &Outcome, sub: u8| {
            out.replies
                .iter()
                .filter(|p| p.starts_with(&[45, 0, sub]))
                .count()
        };
        assert!(engine_mut(&mut game).offer(7002, 1).changed);
        assert!(engine_mut(&mut game).accept(7002).changed);

        // 45/11 FirstMovement with the mouse flag, once.
        let out = game
            .handle_in_world(&claim_packet(99), &character, 4_000)
            .unwrap();
        let mut first = vec![45, 0, 11];
        first.extend(77u32.to_le_bytes());
        first.push(1);
        assert_eq!(out.replies, vec![first]);
        let out = game
            .handle_in_world(&claim_packet(99), &character, 4_001)
            .unwrap();
        assert!(out.replies.is_empty());

        // Abandon and take it again: the client still holds the registration, so it gets a
        // 45/9 re-arm and no second 45/11.
        assert!(engine_mut(&mut game).abandon(7002).changed);
        let out = game
            .handle_in_world(&claim_packet(99), &character, 4_002)
            .unwrap();
        assert!(out.replies.is_empty());
        assert!(engine_mut(&mut game).offer(7002, 1).changed);
        assert!(engine_mut(&mut game).accept(7002).changed);
        let out = game
            .handle_in_world(&claim_packet(99), &character, 4_003)
            .unwrap();
        let mut rearm = vec![45, 0, 9];
        rearm.extend(77u32.to_le_bytes());
        assert_eq!(out.replies, vec![rearm]);
        assert_eq!(watching(&out, 11), 0);
    }

    fn tracked_pins(out: &Outcome) -> Vec<Vec<u8>> {
        out.replies
            .iter()
            .filter(|packet| packet.starts_with(&[47, 0, 14]))
            .cloned()
            .collect()
    }

    fn select_packet(quest_id: u32) -> Vec<u8> {
        let mut select = vec![47, 0, 12];
        select.extend((quest_id as i32).to_le_bytes());
        select
    }

    #[test]
    fn take_me_there_pin_follows_the_selected_quest() {
        let (mut game, character) = offer_test_game(false);
        let mut accept = vec![49, 0, 2, 0, 0, 0];
        accept.extend((INTRODUCE_YOURSELF_ID as i32).to_le_bytes());
        accept.push(1);
        let out = game.handle_in_world(&accept, &character, 1_005).unwrap();
        // Accepting does not select the quest, and nothing is pinned for an unselected one.
        assert_eq!(
            game.npc_quests.as_ref().unwrap().state().selected_quest_id,
            None
        );
        assert!(tracked_pins(&out).is_empty());

        // Selecting the active quest pins its talk objective once: Ricky Danger's spot.
        let select = select_packet(INTRODUCE_YOURSELF_ID);
        let first = game.handle_in_world(&select, &character, 1_006).unwrap();
        let mut expected = vec![47, 0, 14, 1];
        expected.extend(INTRODUCE_YOURSELF_ID.to_le_bytes());
        expected.extend(0u32.to_le_bytes());
        expected.extend(1u32.to_le_bytes());
        expected.extend(9_500_000_254u64.to_le_bytes());
        expected.extend(0u32.to_le_bytes());
        for float in [-1909.511f32, -44.3937, 462.83, 1.0] {
            expected.extend(float.to_le_bytes());
        }
        expected.extend(0u32.to_le_bytes());
        assert_eq!(tracked_pins(&first), vec![expected]);

        // Repeating the selection, or any unrelated packet, does not resend it.
        let again = game.handle_in_world(&select, &character, 1_007).unwrap();
        assert!(tracked_pins(&again).is_empty());

        // A forged id is refused by the engine, so the pin stays where it was.
        let forged = select_packet(999_999);
        let out = game.handle_in_world(&forged, &character, 1_008).unwrap();
        assert!(tracked_pins(&out).is_empty());

        // Leaving the pin's zone clears it; coming back sends it again.
        let away = CharacterState {
            zone_id: 9_600,
            ..character.clone()
        };
        let out = game.handle_in_world(&select, &away, 1_009).unwrap();
        assert_eq!(tracked_pins(&out), vec![vec![47, 0, 14, 0]]);
        let out = game.handle_in_world(&select, &character, 1_010).unwrap();
        assert_eq!(tracked_pins(&out).len(), 1);
        assert_eq!(tracked_pins(&out)[0].len(), 48);
    }

    #[test]
    fn kill_objective_pins_its_field_enemy() {
        let (mut game, character) = offer_test_game(false);
        let engine = game.npc_quests.as_mut().unwrap();
        // "Defeat the Hooligan Delinquents" (kind 930050), a real enabled quest.
        assert!(engine.dev_mark_offered(3_030_007));
        assert!(engine.accept(3_030_007).changed);
        let content = bundled_content();
        let file: serde_json::Value =
            serde_json::from_slice(&std::fs::read(content.join("field_enemies.json")).unwrap())
                .unwrap();
        let home = file["enemies"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|enemy| enemy["definition_id"] == 930_050)
            .min_by_key(|enemy| enemy["guid"].as_u64().unwrap())
            .unwrap();
        let out = game
            .handle_in_world(&select_packet(3_030_007), &character, 1_005)
            .unwrap();
        let pins = tracked_pins(&out);
        assert_eq!(pins.len(), 1);
        assert_eq!(
            pins[0][16..24],
            home["guid"].as_u64().unwrap().to_le_bytes()
        );
        let position: [f32; 3] = serde_json::from_value(home["position"].clone()).unwrap();
        for (axis, value) in position.iter().enumerate() {
            assert_eq!(pins[0][28 + 4 * axis..32 + 4 * axis], value.to_le_bytes());
        }
    }

    #[test]
    fn an_objective_without_a_known_place_sends_no_pin() {
        let (mut game, character) = offer_test_game(false);
        let engine = game.npc_quests.as_mut().unwrap();
        // Kind 930096 has no field enemy, so the kill step has no known place.
        assert!(engine.dev_mark_offered(3_110_002));
        assert!(engine.accept(3_110_002).changed);
        let out = game
            .handle_in_world(&select_packet(3_110_002), &character, 1_005)
            .unwrap();
        assert!(out.changed);
        assert!(tracked_pins(&out).is_empty());
        let again = game
            .handle_in_world(&select_packet(3_110_002), &character, 1_006)
            .unwrap();
        assert!(tracked_pins(&again).is_empty());
    }

    #[test]
    fn native_save_samantha_uses_localized_quest_offer_and_reply_state() {
        let content = bundled_content();
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        game.profiles = Some(Profiles::load(&reference, Some(native_test_profiles())).unwrap());
        game.bind_npc_quest_catalogue(&content, 1).unwrap();
        let eligibility = game.npc_quests.as_ref().unwrap().eligibility();
        assert!(eligibility.member);
        assert_eq!(eligibility.member_quest_limit, 100);
        let active = game.profiles.as_ref().unwrap().active();
        let active_job = game
            .npc_quests
            .as_ref()
            .unwrap()
            .job_profile_ids()
            .find(|(_, profile)| *profile == active)
            .unwrap()
            .0;
        assert_eq!(eligibility.job_levels.len(), 1);
        assert_eq!(
            eligibility.job_levels.get(active_job),
            Some(&eligibility.player_level)
        );

        game.set_game_rules(crate::game_rules::GameRuleSet::new(
            crate::game_rules::MembershipState::new(false, None),
            crate::game_rules::RulePolicy::default(),
        ));
        assert!(!game.npc_quests.as_ref().unwrap().eligibility().member);
        game.set_game_rules(GameRuleSet::default());
        assert!(game.npc_quests.as_ref().unwrap().eligibility().member);

        let npc_file: serde_json::Value = serde_json::from_slice(
            &std::fs::read(content.join("npcs/cobblestone_village.json")).unwrap(),
        )
        .unwrap();
        let samantha = npc_file["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["npc_id"].as_u64() == Some(SAMANTHA_ID))
            .unwrap();
        let position: [f32; 3] = serde_json::from_value(samantha["position"].clone()).unwrap();
        let mut character = CharacterState {
            guid: 1,
            zone_id: 0,
            position,
            rotation: [1.0, 0.0, 0.0],
        };

        let menu = game
            .handle_in_world(&packet_for_npc(SAMANTHA_ID), &character, 1_000)
            .unwrap();
        let menu_packet = menu
            .replies
            .iter()
            .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
            .expect("Samantha returns a radial menu");
        let option_count = i32::from_le_bytes(menu_packet[13..17].try_into().unwrap());
        assert_eq!(
            option_count, 1,
            "new quests are offered after selecting Talk"
        );

        let talk = game
            .handle_in_world(&select_npc_option(SAMANTHA_ID, 1), &character, 1_001)
            .unwrap();
        assert!(talk.replies.iter().any(|p| p.starts_with(&[26, 0, 3, 0])));
        let first_offer = game
            .handle_in_world(&choose_quest_row(), &character, 1_001)
            .unwrap();
        let first_offer_packet = first_offer
            .replies
            .iter()
            .find(|packet| packet.starts_with(&[49, 0, 1, 0]))
            .expect("selecting Talk sends QuestInfo 49/1 for the eligible quest");
        let offer_fields = npc_interaction::decode_quest_offer_49_1(first_offer_packet).unwrap();
        assert_eq!(offer_fields.quest_id, INTRODUCE_YOURSELF_ID);
        assert_eq!(offer_fields.intro_text_string_id, 94246);
        assert_eq!(offer_fields.name_string_id, 94247);
        assert_eq!(offer_fields.description_string_id, 94248);
        assert_eq!(offer_fields.field_5, 94247);
        assert!(!offer_fields.members_only);

        let mut decline = vec![49, 0, 2, 0, 0, 0];
        decline.extend(INTRODUCE_YOURSELF_ID.to_le_bytes());
        decline.push(0);
        let declined = game.handle_in_world(&decline, &character, 1_002).unwrap();
        assert!(
            !declined
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0, 3, 0]))
        );
        assert_eq!(
            game.npc_quests.as_ref().unwrap().state().pending_offer,
            None
        );
        assert_eq!(
            game.npc_quests
                .as_ref()
                .unwrap()
                .progress(INTRODUCE_YOURSELF_ID),
            None
        );

        game.handle_in_world(&packet_for_npc(SAMANTHA_ID), &character, 1_003)
            .unwrap();
        game.handle_in_world(&select_npc_option(SAMANTHA_ID, 1), &character, 1_004)
            .unwrap();
        let offer = game
            .handle_in_world(&choose_quest_row(), &character, 1_004)
            .unwrap();
        assert!(
            offer
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0, 1, 0]))
        );

        let mut accept = vec![49, 0, 2, 0, 0, 0];
        accept.extend((INTRODUCE_YOURSELF_ID as i32).to_le_bytes());
        accept.push(1);
        let accepted = game.handle_in_world(&accept, &character, 1_005).unwrap();
        assert!(
            accepted
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0, 3, 0]))
        );
        assert_eq!(
            game.npc_quests.as_ref().unwrap().state().pending_offer,
            None
        );
        assert_eq!(
            game.npc_quests
                .as_ref()
                .unwrap()
                .progress(INTRODUCE_YOURSELF_ID)
                .unwrap()
                .status,
            crate::quest_engine::QuestStatus::Active
        );
        let direct_talk = game
            .npc_quests
            .as_ref()
            .unwrap()
            .clone()
            .on_event(crate::quest_engine::QuestEvent::TalkToNpc {
                npc_id: 9_500_000_254,
            })
            .unwrap();
        assert!(direct_talk.outbound.iter().any(|message| matches!(
            message,
            crate::quest_engine::OutboundMessage::Quest(
                crate::quest_engine::QuestMessage::QuestEnd { quest_id }
            ) if *quest_id == INTRODUCE_YOURSELF_ID
        )));

        // W92 now requires the explicit referral target, rather than a hollow
        // quest's accepting conversation. Exercise the real target route.
        let target_id = 9_500_000_254;
        let target = npc_file["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["npc_id"].as_u64() == Some(target_id))
            .unwrap();
        character.position = serde_json::from_value(target["position"].clone()).unwrap();
        game.update_npc_visibility(&character).unwrap();
        game.handle_in_world(&packet_for_npc(target_id), &character, 1_006)
            .unwrap();
        let talk = game
            .handle_in_world(&select_npc_option(target_id, 1), &character, 1_007)
            .unwrap();
        // The end screen waits behind a dialog choice, like the offer.
        assert!(
            !talk
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0, 13, 0]))
        );
        let chosen = game
            .handle_in_world(&choose_quest_row(), &character, 1_007)
            .unwrap();
        assert!(
            chosen
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0, 13, 0, 0, 0])
                    && packet.len() == 99
                    && u64::from_le_bytes(packet[6..14].try_into().unwrap()) == target_id
                    && u32::from_le_bytes(packet[14..18].try_into().unwrap())
                        == INTRODUCE_YOURSELF_ID)
        );
        let dialogue = talk
            .replies
            .iter()
            .find(|packet| packet.starts_with(&[26, 0, 3, 0]))
            .expect("referral target returns a ShowDialog packet");
        assert!(npc_interaction::decode_dialogue_26_3(dialogue).is_ok());

        let mut end_reply = vec![49, 0, 14, 0, 0, 0];
        end_reply.extend(INTRODUCE_YOURSELF_ID.to_le_bytes());
        let completed = game.handle_in_world(&end_reply, &character, 1_008).unwrap();
        assert!(
            completed
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0, 4, 0]))
        );
        assert_eq!(completed.replies.last(), Some(&vec![26, 0, 4, 0]));
        assert_eq!(
            game.state().npc_quests.unwrap().quests[&INTRODUCE_YOURSELF_ID].status,
            crate::quest_engine::QuestStatus::Completed
        );

        let inventory = game.inventory.clone();
        game.profiles
            .as_mut()
            .unwrap()
            .select(1, &inventory)
            .unwrap();
        game.profiles.as_mut().unwrap().add_experience(75).unwrap();
        game.refresh_npc_quest_eligibility(current_unix_seconds());
        assert_eq!(
            game.npc_quests
                .as_ref()
                .unwrap()
                .eligibility()
                .job_levels
                .get("adventurer"),
            Some(&2)
        );
    }

    #[test]
    fn developer_quest_offer_variants_match_exact_diagnostic_bytes() {
        let (mut game, character) = offer_test_game(true);
        let dialogue = expected_samantha_dialogue();
        let offer = |prefix, intro| expected_quest_offer(prefix, intro);
        let cases: [(&str, Vec<Vec<u8>>); 7] = [
            ("v0", vec![dialogue.clone(), offer(0, 94_246)]),
            ("v1", vec![offer(0, 94_246)]),
            ("v2", vec![offer(0, 0)]),
            ("v3", vec![offer(0, 94_247)]),
            ("v4", vec![expected_blank_dialogue(), offer(0, 94_246)]),
            ("v5", vec![offer(1, 94_246)]),
            ("v6", vec![offer(0, 0), offer(0, 94_246)]),
        ];

        for (index, (variant, expected)) in cases.into_iter().enumerate() {
            let result = game
                .handle_in_world(
                    &offer_chat(&format!("realm offer {INTRODUCE_YOURSELF_ID} {variant}")),
                    &character,
                    1_010 + index as u64,
                )
                .unwrap();
            let actual = result
                .replies
                .iter()
                .filter(|packet| {
                    packet.starts_with(&[26, 0, 3, 0]) || packet.starts_with(&[49, 0, 1, 0])
                })
                .cloned()
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "variant {variant}");
        }
    }

    /// `realm here` (dev tools) reports the zone, position and nearest actors;
    /// Studio's /api/command runs the same chat path.
    #[test]
    fn developer_here_reports_position_and_neighbours() {
        let (mut game, character) = offer_test_game(true);
        let result = game
            .handle_in_world(&offer_chat("realm here"), &character, 1_000)
            .unwrap();
        let text = String::from_utf8_lossy(&result.replies.concat()).to_string();
        assert!(text.contains("Zone 0 at x"), "{text}");
        assert!(text.contains(" m, "), "lists nearby actors: {text}");
        assert_eq!(
            crate::chat::local_request("realm here"),
            offer_chat("realm here")
        );
    }

    #[test]
    fn developer_quest_offer_command_requires_dev_tools() {
        let (mut game, character) = offer_test_game(false);
        let result = game
            .handle_in_world(
                &offer_chat(&format!("realm offer {INTRODUCE_YOURSELF_ID} v1")),
                &character,
                1_010,
            )
            .unwrap();
        assert!(
            !result
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0, 1, 0]))
        );
    }

    fn spawned_actors(packets: &[Vec<u8>]) -> Vec<(u64, [f32; 3])> {
        packets
            .iter()
            .filter(|p| p.starts_with(&[35, 0, 2, 0]))
            .map(|p| {
                let f = |o: usize| f32::from_le_bytes(p[o..o + 4].try_into().unwrap());
                (
                    u64::from_le_bytes(p[4..12].try_into().unwrap()),
                    [f(37), f(41), f(45)],
                )
            })
            .collect()
    }

    /// Overworld battle: clicking the walking Troll Summoner shows the start
    /// screen (41/114 not launched + 41/107 ready). Go (41/108) needs a combat
    /// job, then zones into the instanced arena instead of fighting in place.
    #[test]
    fn battle_go_requires_a_combat_job_and_zones_into_the_arena() {
        const SPAWNER: u64 = 100_000_096_000;
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-440.0, 0.8, 1292.0],
            rotation: [1.0, 0.0, 0.0],
        };
        let mut click = vec![26, 0, 8, 0];
        click.extend(SPAWNER.to_le_bytes());
        click.push(0);
        let mut go = vec![41, 0, 108, 0];
        go.extend([0; 8]);

        let screen = game.handle_in_world(&click, &character, 1_000).unwrap();
        let details = screen
            .replies
            .iter()
            .find(|p| p.starts_with(&[41, 0, 114, 0]))
            .expect("start screen");
        // The preview bundle lists the four prizes (Item entries).
        for item in [10_482_i32, 3_023, 75_354, 11_002] {
            assert!(
                details.windows(4).any(|w| w == item.to_le_bytes()),
                "prize {item} on the start screen"
            );
        }
        assert!(
            screen
                .replies
                .iter()
                .any(|p| p.starts_with(&[41, 0, 107, 0]))
        );
        let refused = game.handle_in_world(&go, &character, 2_000).unwrap();
        assert!(refused.zoning.is_none(), "no combat job: no battle");
        assert!(
            refused
                .replies
                .iter()
                .any(|p| p.starts_with(&[41, 0, 114, 0])),
            "the start screen stays up so Switch Jobs is reachable"
        );

        let inventory = game.inventory.clone();
        game.profiles
            .as_mut()
            .unwrap()
            .select(43, &inventory)
            .unwrap();
        game.handle_in_world(&click, &character, 3_000).unwrap();
        let entered = game.handle_in_world(&go, &character, 4_000).unwrap();
        let zone = entered.zoning.expect("Go zones into the battle arena");
        assert_eq!(zone.name, "sg_random_encounter_clearing");
        // Go without a fresh start screen does nothing.
        assert!(
            game.handle_in_world(&go, &character, 5_000)
                .unwrap()
                .zoning
                .is_none()
        );
    }

    /// Inside the arena only the summoner stands at first. Hitting it below
    /// 75/50/25% calls in two small trolls each with the cast animation and
    /// the troll spawn puff (35/16); it takes no damage until they are down;
    /// defeating it wins the battle and reports the quest kill (930096).
    #[test]
    fn troll_summoner_battle_summons_waves_and_is_won_on_the_summoner() {
        const SUMMONER: u64 = 100_000_096_001;
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let encounter = game.encounter.as_mut().unwrap();
        let world = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-440.0, 0.8, 1292.0],
            rotation: [1.0, 0.0, 0.0],
        };
        let begun = encounter.begin(96, world, 43).unwrap();
        assert_eq!(begun.zoning.unwrap().name, "sg_random_encounter_clearing");
        let ready = encounter.on_zone_ready(43, 1_000).unwrap();
        let first = spawned_actors(&ready.replies);
        assert_eq!(first.len(), 1, "only the summoner at first");
        assert_eq!(first[0].0, SUMMONER);

        let at = first[0].1;
        let mut now = 2_000;
        let mut waves = Vec::new();
        let mut defeated = Vec::new();
        let mut won = false;
        for _ in 0..40 {
            now += 700;
            let out = encounter.attack(SUMMONER, at, 43, 0, now).unwrap();
            defeated.extend(out.defeated.iter().copied());
            won |= out.replies.iter().any(|p| p.starts_with(&[39, 0, 18]));
            if !out.defeated.is_empty() {
                let reward = out.rewards.first().expect("battle reward");
                assert_eq!(reward.coins, 25);
                assert_eq!(reward.job_xp, 50, "battles level the combat job");
                assert!(
                    reward.items.len() == 1
                        && [10_482, 3_023, 75_354, 11_002].contains(&reward.items[0].0),
                    "one prize from the wheel: {:?}",
                    reward.items
                );
                // The loot wheel lands on the same prize (39/45, by name id).
                let name = [
                    (10_482, 6_666),
                    (3_023, 422_907),
                    (75_354, 131_098),
                    (11_002, 422_909),
                ]
                .iter()
                .find(|(item, _)| *item == reward.items[0].0)
                .unwrap()
                .1;
                let wheel = out
                    .replies
                    .iter()
                    .find(|p| p.starts_with(&[39, 0, 45]))
                    .expect("loot wheel landing");
                assert!(wheel.windows(4).any(|w| w == (name as i32).to_le_bytes()));
                break;
            }
            let called = spawned_actors(&out.replies);
            if called.is_empty() {
                continue;
            }
            assert!(
                out.replies.iter().any(|p| p.starts_with(&[35, 0, 16, 0])),
                "spawn puff"
            );
            assert!(
                out.replies
                    .iter()
                    .any(|p| p.starts_with(&[35, 0, 8, 0]) && p[4..12] == SUMMONER.to_le_bytes()),
                "summon cast"
            );
            waves.push(called.len());
            // Shielded while the wave stands: a hit does no damage.
            now += 700;
            let shielded = encounter.attack(SUMMONER, at, 43, 0, now).unwrap();
            let hit = shielded
                .replies
                .iter()
                .find(|p| p.starts_with(&[32, 0, 7, 0]))
                .expect("the hit is still shown");
            assert_eq!(
                &hit[28..32],
                &0_u32.to_le_bytes(),
                "no damage while shielded"
            );
            for (guid, position) in called {
                for _ in 0..4 {
                    now += 700;
                    let out = encounter.attack(guid, position, 43, 0, now).unwrap();
                    defeated.extend(out.defeated.iter().copied());
                }
            }
        }
        assert_eq!(waves, vec![2, 2, 2], "summon waves at 75/50/25%");
        assert_eq!(
            defeated.iter().filter(|&&d| d == 930_097).count(),
            6,
            "all minions"
        );
        assert_eq!(
            defeated.last(),
            Some(&930_096),
            "summoner kill id for the quest"
        );
        assert!(won, "game over (won) on the summoner");

        // The save right after the win loads again (run 150132 refused it:
        // the battle id was recorded in `rewarded`), back in the world.
        let saved = game.state();
        assert!(
            !saved.encounter.as_ref().unwrap().rewarded.contains(&96),
            "battles are repeatable"
        );
        let mut reloaded = Gameplay::load(&reference, Some(saved)).expect("save after a win loads");
        assert!(
            !reloaded.encounter.as_ref().unwrap().active(),
            "returned to the world"
        );
        // The character, still saved in the arena, goes back where it started.
        let back = reloaded
            .abandon_battle_for_reconnect()
            .expect("battle start location");
        assert_eq!(back.zone_id, 0);
        assert!(reloaded.supports_location(&back));
        assert!(
            reloaded.abandon_battle_for_reconnect().is_none(),
            "only once"
        );
    }

    /// Alone, the summoner runs from a chasing player for about 4 s of
    /// running, then rests (com_stunned, no attacks) for about 3 s; it does not
    /// run from a player who stays away (R).
    #[test]
    fn lone_troll_summoner_runs_then_rests_without_attacking() {
        const SUMMONER: u64 = 100_000_096_001;
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let encounter = game.encounter.as_mut().unwrap();
        let world = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-440.0, 0.8, 1292.0],
            rotation: [1.0, 0.0, 0.0],
        };
        encounter.begin(96, world, 43).unwrap();
        let ready = encounter.on_zone_ready(43, 1_000).unwrap();
        let mut troll = spawned_actors(&ready.replies)[0].1;
        let summoner_moves = |replies: &[Vec<u8>]| {
            replies
                .iter()
                .filter(|p| p.starts_with(&[125, 0]) && p[2..10] == SUMMONER.to_le_bytes())
                .map(|p| {
                    let f = |o: usize| f32::from_le_bytes(p[o..o + 4].try_into().unwrap());
                    [f(10), f(14), f(18)]
                })
                .collect::<Vec<_>>()
        };

        // A player 30 m away is watched, not run from.
        let far = [troll[0], troll[1], troll[2] - 30.0];
        for step in 1..=20_u64 {
            let out = encounter.tick(far, 43, 1_000 + step * 100).unwrap();
            assert!(
                summoner_moves(&out.replies).is_empty(),
                "no running from afar"
            );
        }

        // A player chasing 3 m behind it: it runs, then rests without attacking.
        let mut moves = 0;
        let mut rested_at = None;
        let mut hits_while_resting = 0;
        for step in 1..=80_u64 {
            let now = 3_000 + step * 100;
            let player = [troll[0], troll[1], troll[2] - 3.0];
            let out = encounter.tick(player, 43, now).unwrap();
            for at in summoner_moves(&out.replies) {
                moves += 1;
                troll = at;
            }
            for p in &out.replies {
                if p.starts_with(&[35, 0, 8, 0])
                    && p[4..12] == SUMMONER.to_le_bytes()
                    && p[12..16] == 1401_i32.to_le_bytes()
                {
                    rested_at.get_or_insert(now);
                }
                if rested_at.is_some_and(|at| now < at + 3_000) && p.starts_with(&[32, 0, 7, 0]) {
                    hits_while_resting += 1;
                }
            }
        }
        let rested_at = rested_at.expect("the summoner rests");
        assert!(
            (6_500..=7_600).contains(&rested_at),
            "rest after ~4 s of running ({rested_at})"
        );
        assert!(moves >= 20, "it ran ({moves} moves)");
        assert_eq!(hits_while_resting, 0, "no attacks while resting");
    }

    /// The Medic (profile 11) bar carries Incision, Triage and First Aid
    /// (data/content/job_abilities.json). In a battle Triage strikes for 3.2x
    /// the basic attack and heals 254, spending all 100 energy; First Aid
    /// heals 250 on a 15 s cooldown; refusals use the client's messages.
    #[test]
    fn medic_job_abilities_strike_heal_and_refuse() {
        const SUMMONER: u64 = 100_000_096_001;
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let bar = game.abilities.as_ref().unwrap().activation(11, 1).unwrap();
        let definitions: Vec<i32> = bar
            .iter()
            .filter(|p| p.starts_with(&[36, 0, 13, 0]))
            .map(|p| i32::from_le_bytes(p[4..8].try_into().unwrap()))
            .collect();
        assert_eq!(definitions.len(), 3, "basic, Triage, First Aid");
        assert_eq!(&definitions[1..], &[4899, 4901]);
        let toolbar = bar.last().unwrap();
        assert!(toolbar.starts_with(&[36, 0, 5, 0]));
        assert_eq!(
            toolbar.len(),
            4 + 8 + 3 * 41 + 5 * 4,
            "three slots, five empty"
        );

        let triage = game
            .abilities
            .as_ref()
            .unwrap()
            .job_slot(11, 1, 1)
            .unwrap()
            .clone();
        let first_aid = game
            .abilities
            .as_ref()
            .unwrap()
            .job_slot(11, 2, 1)
            .unwrap()
            .clone();
        let encounter = game.encounter.as_mut().unwrap();
        let world = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-440.0, 0.8, 1292.0],
            rotation: [1.0, 0.0, 0.0],
        };
        encounter.begin(96, world, 11).unwrap();
        let ready = encounter.on_zone_ready(11, 1_000).unwrap();
        let at = spawned_actors(&ready.replies)[0].1;
        let damage = |out: &EncounterOutcome| {
            out.replies
                .iter()
                .find(|p| p.starts_with(&[32, 0, 7, 0]))
                .map(|p| u32::from_le_bytes(p[28..32].try_into().unwrap()))
        };
        let refusal = |out: &EncounterOutcome| {
            out.replies
                .iter()
                .find(|p| p.starts_with(&[36, 0, 1, 0]))
                .map(|p| i32::from_le_bytes(p[4..8].try_into().unwrap()))
        };

        // Self-centred request (guid 1) picks the summoner in reach.
        let hit = encounter.job_ability(&triage, 1, at, 11, 2_000).unwrap();
        assert_eq!(damage(&hit), Some(240), "75 x 3.2");
        assert!(
            hit.replies.iter().any(|p| p.starts_with(&[35, 0, 35, 0])),
            "heal number"
        );
        assert!(
            hit.replies.iter().any(|p| p.starts_with(&[38, 0, 13, 0])),
            "energy spent"
        );
        let again = encounter.job_ability(&triage, 1, at, 11, 3_500).unwrap();
        assert_eq!(refusal(&again), Some(285), "not enough energy");

        let healed = encounter.job_ability(&first_aid, 1, at, 11, 3_600).unwrap();
        assert!(healed.replies.iter().any(|p| p.starts_with(&[38, 0, 1, 0])));
        let soon = encounter.job_ability(&first_aid, 1, at, 11, 5_000).unwrap();
        assert_eq!(refusal(&soon), Some(287), "cooldown");

        // Out of reach: no target, nothing spent.
        let far = [at[0] + 20.0, at[1], at[2]];
        let missed = encounter
            .job_ability(&triage, SUMMONER, far, 11, 60_000)
            .unwrap();
        assert!(matches!(refusal(&missed), Some(285) | Some(288)));
    }

    /// The overworld Troll Summoner walks a small loop around its home while
    /// a player is near enough to see it, and stays put otherwise.
    #[test]
    fn battle_marker_wanders_only_when_seen() {
        const SPAWNER: u64 = 100_000_096_000;
        let home = [-440.0_f32, 0.8, 1290.0];
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let encounter = game.encounter.as_mut().unwrap();
        let moves = |out: &EncounterOutcome| {
            out.replies
                .iter()
                .filter(|p| p.starts_with(&[125, 0]) && p[2..10] == SPAWNER.to_le_bytes())
                .map(|p| {
                    let f = |o: usize| f32::from_le_bytes(p[o..o + 4].try_into().unwrap());
                    [f(10), f(14), f(18)]
                })
                .collect::<Vec<_>>()
        };
        let far = [home[0] + 500.0, home[1], home[2]];
        for step in 1..=30_u64 {
            assert!(moves(&encounter.tick(far, 11, step * 100).unwrap()).is_empty());
        }
        let near = [home[0] + 20.0, home[1], home[2]];
        let mut seen = Vec::new();
        for step in 31..=200_u64 {
            seen.extend(moves(&encounter.tick(near, 11, step * 100).unwrap()));
        }
        assert!(seen.len() > 20, "it walks ({} moves)", seen.len());
        // Close by, it stops and faces the player: no further walking.
        let last = *seen.last().unwrap();
        let close = [last[0] + 3.0, last[1], last[2]];
        encounter.tick(close, 11, 20_100).unwrap();
        for step in 202..=260_u64 {
            assert!(
                moves(&encounter.tick(close, 11, step * 100).unwrap()).is_empty(),
                "stands still near the player"
            );
        }
        for p in &seen {
            let d = ((p[0] - home[0]).powi(2) + (p[2] - home[2]).powi(2)).sqrt();
            assert!(d <= 5.01, "stays near home ({d})");
            assert_eq!(p[1], home[1], "keeps its home height");
        }
    }

    /// With terrain heights the marker walks on the slope and still reaches
    /// every waypoint (its goals are snapped to the ground too).
    #[test]
    fn battle_marker_follows_the_terrain() {
        const SPAWNER: u64 = 100_000_096_000;
        let home = [-440.0_f32, 0.8, 1290.0];
        let slope = |x: f32| 0.8 + (x - home[0]) * 0.5;
        let mut grid = crate::terrain::HeightGrid::new(
            [home[0] - 50.0, home[2] - 50.0],
            [home[0] + 50.0, home[2] + 50.0],
            2.0,
        )
        .unwrap();
        let v = |dx: f32, dz: f32| [home[0] + dx, slope(home[0] + dx), home[2] + dz];
        grid.add_triangle([v(-50.0, -50.0), v(50.0, -50.0), v(50.0, 50.0)]);
        grid.add_triangle([v(-50.0, -50.0), v(50.0, 50.0), v(-50.0, 50.0)]);
        let dir = std::env::temp_dir().join(format!("osfr-wander-hgt-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("zone_0.hgt"), grid.to_bytes()).unwrap();
        let terrain = Arc::new(crate::terrain::Terrain::load(&dir).unwrap());

        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        game.set_terrain(Some(terrain));
        let encounter = game.encounter.as_mut().unwrap();
        let near = [home[0] + 20.0, home[1], home[2]];
        let mut seen = Vec::new();
        for step in 1..=400_u64 {
            let out = encounter.tick(near, 11, step * 100).unwrap();
            seen.extend(
                out.replies
                    .iter()
                    .filter(|p| p.starts_with(&[125, 0]) && p[2..10] == SPAWNER.to_le_bytes())
                    .map(|p| {
                        let f = |o: usize| f32::from_le_bytes(p[o..o + 4].try_into().unwrap());
                        [f(10), f(14), f(18)]
                    }),
            );
        }
        assert!(seen.len() > 20, "it walks ({} moves)", seen.len());
        for p in &seen {
            assert!((p[1] - slope(p[0])).abs() < 0.06, "on the ground: {p:?}");
        }
        // All four waypoints reached: it visits every quadrant around home.
        let quadrants: std::collections::BTreeSet<(bool, bool)> = seen
            .iter()
            .map(|p| (p[0] > home[0], p[2] > home[2]))
            .collect();
        assert_eq!(quadrants.len(), 4, "loops around its home");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Job skills are level-locked: a level-5 skill is an empty bar slot and
    /// is refused at level 1, and appears once the job reaches level 5.
    #[test]
    fn job_skills_unlock_at_their_level() {
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let dir = std::env::temp_dir().join(format!("osfr-job-levels-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("job_abilities.json");
        std::fs::write(
            &path,
            r#"{"schema_version":1,"jobs":[{"profile_id":11,"slots":[
                {"slot":1,"definition_id":4901,"name_id":3784,"icon_id":260,"heal":250,"cooldown_ms":15000,"min_level":1},
                {"slot":2,"definition_id":4902,"name_id":548869300,"icon_id":11639,"damage_multiplier":1.5,"cooldown_ms":5000,"min_level":5}]}]}"#,
        )
        .unwrap();
        let abilities = game.abilities.as_mut().unwrap();
        abilities.load_job_abilities(&path).unwrap();
        let definitions = |bar: &[Vec<u8>]| {
            bar.iter()
                .filter(|p| p.starts_with(&[36, 0, 13, 0]))
                .map(|p| i32::from_le_bytes(p[4..8].try_into().unwrap()))
                .collect::<Vec<_>>()
        };
        let level_one = abilities.activation(11, 1).unwrap();
        assert!(
            !definitions(&level_one).contains(&4902),
            "locked at level 1"
        );
        assert!(abilities.job_slot(11, 2, 1).is_none());
        let level_five = abilities.activation(11, 5).unwrap();
        assert!(
            definitions(&level_five).contains(&4902),
            "unlocked at level 5"
        );
        assert!(abilities.job_slot(11, 2, 5).is_some());
        assert!(abilities.unlocks_between(11, 4, 5));
        assert!(!abilities.unlocks_between(11, 1, 4));
        // Same toolbar length either way: the locked slot is an empty entry.
        assert_eq!(
            level_one.last().unwrap().len() + 37,
            level_five.last().unwrap().len()
        );
        std::fs::remove_dir_all(dir).ok();
    }

    /// W158 patch B: `realm reload power_ups` loads the drop tables (it used to
    /// be an unknown target), and a broken file is refused with the old tables kept.
    #[test]
    fn power_up_tables_reload_by_name() {
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let dir = std::env::temp_dir().join(format!("osfr-power-reload-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/power_ups.json"),
            dir.join("power_ups.json"),
        )
        .unwrap();
        game.content_dir = dir.clone();
        let mut out = Outcome::default();
        let reply = game.reload_content("power_ups", &mut out).unwrap();
        assert!(reply.contains("power-ups"), "{reply}");
        std::fs::write(dir.join("power_ups.json"), b"{ not json").unwrap();
        assert!(game.reload_content("power_ups", &mut out).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// W158 slice 2b: a big heart's group heal leaves the engine, reaches the
    /// gameplay outcome through `apply_encounter`, and heals a hurt member.
    #[test]
    fn group_heal_reaches_the_outcome_and_heals_a_hurt_member() {
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let heal = crate::encounter::GroupHeal {
            percent: 50,
            kill: (7, 9),
        };
        let mut out = Outcome::default();
        game.apply_encounter(
            EncounterOutcome {
                group_heal: Some(heal),
                ..Default::default()
            },
            &mut out,
        )
        .unwrap();
        assert_eq!(out.group_heal, Some(heal));
        game.receive_field_damage(400, 1_000, false).unwrap();
        let before = game.encounter.as_ref().unwrap().player_health().0;
        let healed = game.receive_group_heal(50).unwrap();
        let (now, max) = game.encounter.as_ref().unwrap().player_health();
        assert!(now > before && now <= max, "{before} -> {now} of {max}");
        assert!(healed.changed && !healed.replies.is_empty());
        // At full health a group heal says nothing and changes nothing.
        game.encounter.as_mut().unwrap().heal_full();
        let quiet = game.receive_group_heal(50).unwrap();
        assert!(!quiet.changed && quiet.replies.is_empty());
    }

    /// Lead: a 4-byte 167/1/4 refresh returns the whole activity list, so it is
    /// answered at most every 250 ms (amplification bound, R). The client's own
    /// re-asks (job change, membership) are further apart than that.
    #[test]
    fn activity_portal_refresh_is_throttled() {
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let who = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-440.0, 0.8, 1292.0],
            rotation: [1.0, 0.0, 0.0],
        };
        let refresh = [167, 0, 1, 4];
        let first = game.handle_in_world(&refresh, &who, 10_000).unwrap();
        assert!(!first.replies.is_empty(), "the first refresh is answered");
        let burst = game.handle_in_world(&refresh, &who, 10_100).unwrap();
        assert!(burst.replies.is_empty(), "a burst refresh is dropped");
        let later = game.handle_in_world(&refresh, &who, 10_250).unwrap();
        assert_eq!(
            later.replies, first.replies,
            "after 250 ms it is answered again"
        );
    }

    /// W143: Inspect shows only unlocked jobs with ranks and the active job's
    /// worn items, each with its definition row; nothing for an unworn slot.
    #[test]
    fn public_look_holds_only_jobs_and_active_worn_items() {
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let look = game.public_look().expect("profiles are loaded");
        let profiles = game.profiles.as_ref().unwrap().state().clone();
        assert_eq!(look.active_profile, profiles.active);
        assert_eq!(look.profiles.len(), profiles.unlocked.len());
        let state = game.inventory.state().clone();
        let worn = state
            .equipment
            .iter()
            .filter(|e| e.profile_id == profiles.active && e.item_id != 0)
            .count();
        assert_eq!(look.items.len(), worn);
        assert!(look.items.iter().all(|i| !i.definition_record.is_empty()));
        // Another profile's outfit never leaks into the active look.
        game.profiles.as_mut().unwrap().unlock(11).unwrap();
        let after = game.public_look().unwrap();
        assert_eq!(after.items, look.items);
        assert!(after.profiles.iter().any(|&(id, _)| id == 11));
    }

    /// T027: an active-job equip or unequip, and a job switch, produce the
    /// neighbour packets the gateway fans out; alt-job changes and no-op
    /// replays stay local.
    #[test]
    fn live_equipment_and_job_changes_are_ready_for_neighbours() {
        fn i32_at(bytes: &[u8], at: usize) -> i32 {
            i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
        }
        fn text_at(bytes: &[u8], at: &mut usize) -> String {
            let length = i32_at(bytes, *at) as usize;
            let text = String::from_utf8(bytes[*at + 4..*at + 4 + length].to_vec()).unwrap();
            *at += 4 + length;
            text
        }
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let equipped_profile = game
            .inventory
            .state()
            .equipment
            .iter()
            .find(|entry| {
                entry.item_id != 0
                    && game
                        .inventory
                        .state()
                        .items
                        .iter()
                        .any(|item| item.id == entry.item_id)
            })
            .expect("the fixture has an equipped job")
            .profile_id;
        game.profiles
            .as_mut()
            .unwrap()
            .select(equipped_profile, &game.inventory)
            .unwrap();
        let alt_profile = 35;
        game.profiles.as_mut().unwrap().unlock(alt_profile).unwrap();
        game.ensure_outfit(alt_profile, &mut Outcome::default())
            .unwrap();
        let who = CharacterState {
            guid: 42,
            zone_id: 0,
            position: [-440.0, 0.8, 1292.0],
            rotation: [1.0, 0.0, 0.0],
        };
        let state = game.inventory.state().clone();
        let active = game.profiles.as_ref().unwrap().active();
        let worn = state
            .equipment
            .iter()
            .find(|entry| {
                entry.profile_id == active
                    && entry.item_id != 0
                    && state.items.iter().any(|item| item.id == entry.item_id)
            })
            .unwrap()
            .clone();
        let item = state
            .items
            .iter()
            .find(|item| item.id == worn.item_id)
            .unwrap()
            .clone();

        // Unequip: an empty model is the client's removal path.
        let mut unequip = vec![42, 0, 2, 0];
        unequip.extend(worn.slot.to_le_bytes());
        unequip.extend(active.to_le_bytes());
        let removed = game.handle_in_world(&unequip, &who, 1_000).unwrap();
        assert!(removed.changed);
        let change = removed
            .nearby
            .iter()
            .find(|packet| packet.starts_with(&[35, 0, 6, 0]))
            .expect("an unequip is broadcast");
        assert_eq!(u64::from_le_bytes(change[4..12].try_into().unwrap()), 42);
        assert_eq!(i32_at(change, 12), 0);
        let mut at = 16;
        assert_eq!(text_at(change, &mut at), "");
        assert_eq!(text_at(change, &mut at), "");
        assert_eq!(text_at(change, &mut at), "");
        assert_eq!(i32_at(change, at), 0); // tint
        assert_eq!(i32_at(change, at + 4), 0); // effect
        assert_eq!(i32_at(change, at + 8), worn.slot);
        assert_eq!(i32_at(change, at + 12), active);
        assert_eq!(change.len(), at + 24);

        // Equip it again: one slot with the real attachment row.
        let mut equip = vec![42, 0, 3, 0];
        equip.extend(item.id.to_le_bytes());
        equip.extend(active.to_le_bytes());
        equip.extend(worn.slot.to_le_bytes());
        let worn_again = game.handle_in_world(&equip, &who, 2_000).unwrap();
        assert!(worn_again.changed);
        let change = worn_again
            .nearby
            .iter()
            .find(|packet| packet.starts_with(&[35, 0, 6, 0]))
            .expect("an equip is broadcast");
        assert_eq!(u64::from_le_bytes(change[4..12].try_into().unwrap()), 42);
        assert_eq!(i32_at(change, 12), item.id);
        let mut at = 16;
        assert!(!text_at(change, &mut at).is_empty());
        text_at(change, &mut at);
        text_at(change, &mut at);
        assert_eq!(i32_at(change, at + 8), worn.slot);

        // A replay is not a change, so nobody is told.
        let replay = game.handle_in_world(&equip, &who, 3_000).unwrap();
        assert!(!replay.changed);
        assert!(replay.nearby.is_empty());

        // An alt job's outfit is not visible to a stranger.
        let alt = game
            .inventory
            .state()
            .equipment
            .iter()
            .find(|entry| entry.profile_id == alt_profile && entry.item_id != 0)
            .unwrap()
            .clone();
        let mut alt_unequip = vec![42, 0, 2, 0];
        alt_unequip.extend(alt.slot.to_le_bytes());
        alt_unequip.extend(alt_profile.to_le_bytes());
        let local = game.handle_in_world(&alt_unequip, &who, 4_000).unwrap();
        assert!(local.changed);
        assert!(local.nearby.is_empty(), "an alt job stays local");
        let mut alt_equip = vec![42, 0, 3, 0];
        alt_equip.extend(alt.item_id.to_le_bytes());
        alt_equip.extend(alt_profile.to_le_bytes());
        alt_equip.extend(alt.slot.to_le_bytes());
        assert!(
            game.handle_in_world(&alt_equip, &who, 5_000)
                .unwrap()
                .nearby
                .is_empty()
        );

        // A job switch sends the whole new outfit (35/7).
        let mut switch = vec![26, 0, 13, 0];
        switch.extend(alt_profile.to_le_bytes());
        let switched = game.handle_in_world(&switch, &who, 6_000).unwrap();
        assert!(switched.changed);
        let packet = switched
            .nearby
            .iter()
            .find(|packet| packet.starts_with(&[35, 0, 7, 0]))
            .expect("a job switch is broadcast");
        assert_eq!(u64::from_le_bytes(packet[4..12].try_into().unwrap()), 42);
        let expected = game
            .inventory
            .state()
            .equipment
            .iter()
            .filter(|entry| entry.profile_id == alt_profile && entry.item_id != 0)
            .count() as i32;
        assert_eq!(i32_at(packet, 12), expected);
        assert_eq!(game.public_look().unwrap().active_profile, alt_profile);

        // Switching back reports the previous outfit.
        let mut back = vec![26, 0, 13, 0];
        back.extend(active.to_le_bytes());
        let switched = game.handle_in_world(&back, &who, 7_000).unwrap();
        let packet = switched
            .nearby
            .iter()
            .find(|packet| packet.starts_with(&[35, 0, 7, 0]))
            .expect("the next job switch is broadcast");
        let expected = game
            .inventory
            .state()
            .equipment
            .iter()
            .filter(|entry| entry.profile_id == active && entry.item_id != 0)
            .count() as i32;
        assert_eq!(i32_at(packet, 12), expected);
    }

    /// W141: 36/12 resolves only this job's own toolbar definitions; a miss is
    /// silent; unsupported 36/10 modes, other bars and empty or level-locked
    /// slots get the client failure string. None of it touches game state.
    #[test]
    fn ability_definition_lookup_and_unsupported_casts_are_explicit() {
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let profiles = game.profiles.as_mut().unwrap();
        profiles.unlock(11).unwrap();
        profiles.select(11, &game.inventory).unwrap();
        let level = game.job_level(11);
        let loaded: Vec<Vec<u8>> = game
            .abilities
            .as_ref()
            .unwrap()
            .activation(11, level)
            .unwrap()
            .into_iter()
            .filter(|p| p.starts_with(&[36, 0, 13, 0]))
            .collect();
        assert!(loaded.len() >= 3, "basic, Triage, First Aid at level 1");
        let who = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-440.0, 0.8, 1292.0],
            rotation: [1.0, 0.0, 0.0],
        };
        let failure = [vec![36, 0, 1, 0], 286_i32.to_le_bytes().to_vec()].concat();
        let state = |game: &Gameplay| {
            (
                serde_json::to_string(&game.summary()).unwrap(),
                serde_json::to_string(&game.encounter.as_ref().unwrap().state()).unwrap(),
            )
        };
        let before = state(&game);
        for definition in &loaded {
            let mut ask = vec![36, 0, 12, 0];
            ask.extend(&definition[4..8]);
            let out = game.handle_in_world(&ask, &who, 5_000).unwrap();
            assert_eq!(out.replies, vec![definition.clone()]);
            assert!(!out.changed);
        }
        for id in [0, -1, 62_002, 62_004, 999_999, i32::MIN, i32::MAX] {
            let mut ask = vec![36, 0, 12, 0];
            ask.extend(id.to_le_bytes());
            let out = game.handle_in_world(&ask, &who, 5_000).unwrap();
            assert!(out.replies.is_empty(), "locked, other-job or unknown {id}");
        }
        assert!(
            game.handle_in_world(&[36, 0, 12, 0, 1, 2], &who, 5_000)
                .is_err()
        );
        let cast = |bar: i32, slot: i32, tail: &[u8], mode: i32| {
            let mut p = vec![36, 0, 10, 0];
            for v in [bar, slot, mode] {
                p.extend(v.to_le_bytes());
            }
            p.extend(tail);
            p
        };
        let target = 1_u64.to_le_bytes();
        for refused in [
            cast(1, 1, &[0; 16], 1), // ground mode on a real slot
            cast(1, 0, &[0; 16], 1), // ground mode on the basic slot
            cast(2, 0, &target, 0),  // another bar
            cast(1, 9, &target, 0),  // past the toolbar
            cast(1, 3, &target, 0),  // Medic slot 3 unlocks at level 5
            cast(1, 7, &target, 0),  // empty slot
        ] {
            let out = game.handle_in_world(&refused, &who, 5_000).unwrap();
            assert_eq!(out.replies, vec![failure.clone()], "{refused:?}");
        }
        assert_eq!(state(&game), before, "refusals change nothing");
    }

    /// Final-era scaling on the level-1 baseline: max health x1.75 every four
    /// levels, weapon damage x1.75 per power tier (levels 4, 5, 8, 12).
    #[test]
    fn player_health_and_damage_scale_with_job_level() {
        const SUMMONER: u64 = 100_000_096_001;
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let encounter = game.encounter.as_mut().unwrap();
        let health = |packets: &[Vec<u8>]| {
            let p = packets
                .iter()
                .find(|p| p.starts_with(&[38, 0, 1, 0]))
                .unwrap();
            (
                u32::from_le_bytes(p[4..8].try_into().unwrap()),
                u32::from_le_bytes(p[8..12].try_into().unwrap()),
            )
        };
        assert!(
            encounter.set_player_job(32, 1).is_empty(),
            "level 1 is the baseline"
        );
        assert_eq!(
            health(&encounter.set_player_job(32, 10)),
            (3522, 3522),
            "full stays full"
        );
        assert_eq!(health(&encounter.set_player_job(32, 20)), (14_270, 14_270));
        assert_eq!(
            health(&encounter.set_player_job(32, 10)),
            (3522, 3522),
            "capped down"
        );

        let world = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-440.0, 0.8, 1292.0],
            rotation: [1.0, 0.0, 0.0],
        };
        encounter.begin(96, world, 11).unwrap();
        let ready = encounter.on_zone_ready(11, 1_000).unwrap();
        let at = spawned_actors(&ready.replies)[0].1;
        let hit = encounter.attack(SUMMONER, at, 11, 0, 2_000).unwrap();
        let dealt = hit
            .replies
            .iter()
            .find(|p| p.starts_with(&[32, 0, 7, 0]))
            .map(|p| u32::from_le_bytes(p[28..32].try_into().unwrap()));
        // Medic 75 at weapon tier 4 (x1.75^3), or a x1.5 crit.
        assert!(matches!(dealt, Some(402) | Some(603)), "{dealt:?}");
    }

    /// Job identity: Brawler is the tank, Wizard the most fragile; Ninja crits
    /// far more often than Medic (x1.5 hits flagged critical).
    #[test]
    fn combat_jobs_differ_in_health_and_crits() {
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let encounter = game.encounter.as_mut().unwrap();
        let max = |e: &mut EncounterEngine, profile: i32| {
            e.set_player_job(profile, 1);
            let p = e.set_player_job(profile, 20);
            u32::from_le_bytes(p[0][8..12].try_into().unwrap())
        };
        let brawler = max(encounter, 43);
        let ninja = max(encounter, 2);
        let medic = max(encounter, 11);
        let wizard = max(encounter, 12);
        assert!(
            brawler > ninja && ninja >= medic && medic > wizard,
            "{brawler} {ninja} {medic} {wizard}"
        );

        let crits = |e: &mut EncounterEngine, profile: i32| {
            e.set_player_job(profile, 1);
            (0..1000).filter(|_| e.roll_crit()).count()
        };
        let ninja_crits = crits(encounter, 2);
        let medic_crits = crits(encounter, 11);
        assert!(
            (150..=250).contains(&ninja_crits),
            "ninja ~20%: {ninja_crits}"
        );
        assert!((20..=80).contains(&medic_crits), "medic ~5%: {medic_crits}");
    }

    /// Studio cheat console through the normal chat path: sim, battle, win,
    /// reload (refused mid-fight), level.
    #[test]
    fn developer_cheat_console_drives_a_battle() {
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        game.set_dev_tools(true);
        game.set_role(crate::account_store::AccountRole::Admin);
        let inventory = game.inventory.clone();
        game.profiles
            .as_mut()
            .unwrap()
            .select(43, &inventory)
            .unwrap();
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-440.0, 0.8, 1292.0],
            rotation: [1.0, 0.0, 0.0],
        };
        let text = |out: &Outcome| String::from_utf8_lossy(&out.replies.concat()).to_string();
        let mut now = 1_000;
        let mut run = |game: &mut Gameplay, command: &str| {
            now += 1_000;
            game.handle_in_world(&offer_chat(&format!("realm {command}")), &character, now)
                .unwrap()
        };

        let sim = run(&mut game, "sim 96");
        assert!(text(&sim).contains("level  1:"), "{}", text(&sim));
        assert!(text(&sim).contains("level 20:"), "{}", text(&sim));

        let started = run(&mut game, "battle 96");
        assert!(started.zoning.is_some(), "battle zones into the arena");
        let refused = run(&mut game, "reload battles");
        assert!(
            text(&refused).contains("Reload failed"),
            "{}",
            text(&refused)
        );
        game.encounter
            .as_mut()
            .unwrap()
            .on_zone_ready(43, 500_000)
            .unwrap();
        let won = run(&mut game, "win");
        assert!(
            won.replies.iter().any(|p| p.starts_with(&[39, 0, 18])),
            "game over"
        );
        assert!(game.fight_log().iter().any(|e| e.kind == "defeat"));
        let log = run(&mut game, "fightlog");
        assert!(text(&log).contains("Fight log:"));
        run(&mut game, "exit");

        let reload = run(&mut game, "reload all");
        assert!(text(&reload).contains("Reloaded:"), "{}", text(&reload));
        assert!(
            reload.replies.iter().any(|p| p.starts_with(&[36, 0, 5, 0])),
            "bar re-sent"
        );

        let level = run(&mut game, "level 10");
        assert!(text(&level).contains("level 10"), "{}", text(&level));
        assert!(
            game.encounter.as_ref().unwrap().player_health().1 > 1_000,
            "health scaled"
        );
    }

    /// Battle spawners (data/content/field_battles.json) are world-style AddNpc
    /// actors sent by proximity, so the client keeps and renders them.
    #[test]
    fn battle_spawner_spawns_at_zone_ready() {
        const SPAWNER: u64 = 100_000_096_000;
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let encounter = game.encounter.as_mut().unwrap();
        assert!(encounter.is_battle_spawner(SPAWNER));
        let far = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-1900.0, -40.0, 450.0],
            rotation: [1.0, 0.0, 0.0],
        };
        let spawned = |packets: &[Vec<u8>]| {
            packets
                .iter()
                .any(|p| p.starts_with(&[35, 0, 2, 0]) && p[4..12] == SPAWNER.to_le_bytes())
        };
        assert!(
            !spawned(&game.update_npc_visibility(&far).unwrap()),
            "not sent from afar"
        );
        let near = CharacterState {
            position: [-435.0, 0.7, 1289.6],
            ..far
        };
        let near_packets = game.update_npc_visibility(&near).unwrap();
        assert!(spawned(&near_packets), "sent when near");
    }

    /// Behaviour presets (data/content/behaviours.json): the summoner's
    /// `skittish: true` maps to the skittish preset, field enemies default to
    /// melee, and a record can pick `stationary` or override a preset's speed.
    #[test]
    fn behaviour_presets_drive_enemy_movement() {
        const HOOLIGAN: u64 = 100_000_004_154;
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let encounter = game.encounter.as_mut().unwrap();
        let melee = encounter.behaviour(HOOLIGAN).unwrap().clone();
        assert_eq!(melee.mode, crate::encounter::Mode::Melee);
        assert_eq!(
            (melee.move_speed, melee.reach, melee.attack_ms),
            (3.0, 2.5, 1000)
        );
        let summoner = encounter
            .behaviour(100_000_096_001)
            .expect("battle 96 summoner");
        assert_eq!(summoner.mode, crate::encounter::Mode::Skittish);
        assert_eq!((summoner.run_ms, summoner.leash), (4000, 12.0));

        // A copy of the field enemies with the Hooligans given a behaviour.
        let dir = std::env::temp_dir().join(format!("osfr-behaviours-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let source =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/field_enemies.json");
        let mut enemies: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&source).unwrap()).unwrap();
        let with = |enemies: &mut serde_json::Value, behaviour: serde_json::Value| {
            for e in enemies["enemies"].as_array_mut().unwrap() {
                if e["name"] == "Hooligan" {
                    e["behaviour"] = behaviour.clone();
                }
            }
            let path = dir.join("field_enemies.json");
            std::fs::write(&path, enemies.to_string()).unwrap();
            path
        };
        let moves = |out: &EncounterOutcome| -> Vec<[f32; 3]> {
            out.replies
                .iter()
                .filter(|p| p.starts_with(&[125, 0]) && p[2..10] == HOOLIGAN.to_le_bytes())
                .map(|p| {
                    let f = |o: usize| f32::from_le_bytes(p[o..o + 4].try_into().unwrap());
                    [f(10), f(14), f(18)]
                })
                .collect()
        };
        let home = [-1971.408_f32, -34.51335, 539.8358];
        let near = [home[0] + 6.0, home[1], home[2]];

        let path = with(&mut enemies, "stationary".into());
        encounter.reload_field_enemies(&path).unwrap();
        encounter.set_active_quests(&BTreeSet::from([3_030_007]));
        assert_eq!(
            encounter.behaviour(HOOLIGAN).unwrap().mode,
            crate::encounter::Mode::Stationary
        );
        for step in 1..=20_u64 {
            let out = encounter.tick(near, 2, 50_000 + step * 250).unwrap();
            assert!(moves(&out).is_empty(), "a stationary enemy never moves");
        }
        let beside = [home[0] + 1.5, home[1], home[2]];
        let swing = encounter.tick(beside, 2, 60_000).unwrap();
        assert!(
            swing.replies.iter().any(|p| p.starts_with(&[32, 0, 7, 0])),
            "but it still swings within reach"
        );

        // Twice the default speed: two 250 ms ticks cover 3 m, not 1.5 m.
        let path = with(
            &mut enemies,
            serde_json::json!({"preset": "melee", "move_speed": 6.0}),
        );
        encounter.reload_field_enemies(&path).unwrap();
        encounter.set_active_quests(&BTreeSet::from([3_030_007]));
        encounter.tick(near, 2, 70_000).unwrap();
        let first = moves(&encounter.tick(near, 2, 70_250).unwrap());
        let travelled = ((first[0][0] - home[0]).powi(2) + (first[0][2] - home[2]).powi(2)).sqrt();
        assert!((travelled - 3.0).abs() < 0.05, "moved {travelled} m");

        // Reach is tunable on the fly; nonsense is refused.
        let path = with(
            &mut enemies,
            serde_json::json!({"preset": "melee", "reach": 4.0}),
        );
        encounter.reload_field_enemies(&path).unwrap();
        assert_eq!(encounter.behaviour(HOOLIGAN).unwrap().reach, 4.0);
        let path = with(
            &mut enemies,
            serde_json::json!({"preset": "melee", "reach": -1.0}),
        );
        assert!(encounter.reload_field_enemies(&path).is_err());
        let path = with(
            &mut enemies,
            serde_json::json!({"preset": "melee", "move_speed": 6.0}),
        );
        encounter.reload_field_enemies(&path).unwrap();

        // Mistakes are refused and change nothing.
        let path = with(&mut enemies, "sneaky".into());
        let error = encounter
            .reload_field_enemies(&path)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("unknown behaviour preset 'sneaky'"),
            "{error}"
        );
        assert_eq!(encounter.behaviour(HOOLIGAN).unwrap().move_speed, 6.0);
        let typo = dir.join("behaviours.json");
        std::fs::write(
            &typo,
            r#"{"schema_version":1,"presets":{"melee":{"move_sped":9}}}"#,
        )
        .unwrap();
        assert!(encounter.load_behaviours(&typo).is_err());
        assert_eq!(encounter.behaviour(100_000_096_001).unwrap().run_ms, 4000);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Quest-gated overworld enemies (data/content/field_enemies.json): hidden
    /// until Delinquent Dilemma is active, then spawned, fought with the opening's
    /// proven attack packets, removed on defeat, respawned, and hidden again when
    /// the quest ends. Played natively in run 002731 (Medic, X to attack).
    #[test]
    fn quest_gated_field_enemies_spawn_fight_and_respawn() {
        const HOOLIGAN: u64 = 100_000_004_154;
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        let encounter = game
            .encounter
            .as_mut()
            .expect("opening content loads the encounter engine");
        assert!(encounter.field_guids().contains(&HOOLIGAN));
        assert!(
            !encounter.is_field_enemy(HOOLIGAN),
            "hidden until the quest is active"
        );

        let shown = encounter.set_active_quests(&BTreeSet::from([3_030_007]));
        let spawns = shown
            .replies
            .iter()
            .filter(|p| p.starts_with(&[35, 0, 2, 0]))
            .count();
        assert_eq!(spawns, 4, "four Hooligans appear with the quest");
        assert!(encounter.is_field_enemy(HOOLIGAN));

        // Profile 2 deals 100 per swing; 220 HP falls on the third swing.
        let near = [-1971.408, -34.51335, 541.0];
        let mut defeated = Vec::new();
        for swing in 0..3_u64 {
            let out = encounter
                .attack(HOOLIGAN, near, 2, 0, 10_000 + swing * 700)
                .unwrap();
            defeated.extend(out.defeated.iter().copied());
        }
        assert_eq!(
            defeated,
            vec![930_050],
            "a defeat reports the quest kill id"
        );
        assert!(
            encounter
                .attack(HOOLIGAN, near, 2, 0, 13_000)
                .unwrap()
                .replies
                .is_empty(),
            "a defeated enemy cannot be hit"
        );

        // Respawn after 30 s; the player stands far away so nothing aggroes.
        let far = [-1700.0, -30.0, 400.0];
        let respawned = encounter.tick(far, 2, 10_000 + 30_000 + 1_500).unwrap();
        assert!(
            respawned
                .replies
                .iter()
                .any(|p| p.starts_with(&[35, 0, 2, 0]) && p[4..12] == HOOLIGAN.to_le_bytes()),
            "the Hooligan respawns"
        );

        let hidden = encounter.set_active_quests(&BTreeSet::new());
        assert_eq!(
            hidden
                .replies
                .iter()
                .filter(|p| p.starts_with(&[35, 0, 3, 0]))
                .count(),
            4,
            "they leave when the quest is no longer active"
        );
        assert!(!encounter.is_field_enemy(HOOLIGAN));
    }

    /// Cobblestone starter chain, as played natively in run 215840: Introduce Yourself
    /// (Samantha -> Ricky), Call the Crew! (Ricky -> Shakey -> Ricky), Back for More
    /// (Ricky -> Samantha). Steps are ordered: Ricky does not count before Shakey.
    #[test]
    fn cobblestone_starter_chain_runs_in_order() {
        const RICKY: u64 = 9_500_000_254;
        const SHAKEY: u64 = 9_500_000_941;
        const CALL_THE_CREW: u32 = 3_000_059;
        const BACK_FOR_MORE: u32 = 3_000_060;
        let content = bundled_content();
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).unwrap();
        game.profiles = Some(Profiles::load(&reference, Some(native_test_profiles())).unwrap());
        game.bind_npc_quest_catalogue(&content, 1).unwrap();

        let mut positions = std::collections::BTreeMap::new();
        for entry in std::fs::read_dir(content.join("npcs")).unwrap() {
            let file: serde_json::Value =
                serde_json::from_slice(&std::fs::read(entry.unwrap().path()).unwrap()).unwrap();
            for record in file["records"].as_array().unwrap() {
                if let Ok(position) = serde_json::from_value::<[f32; 3]>(record["position"].clone())
                {
                    positions.insert(record["npc_id"].as_u64().unwrap(), position);
                }
            }
        }
        let clock = std::cell::Cell::new(2_000_u64);
        let talk = |game: &mut Gameplay, npc: u64| -> (CharacterState, Vec<Vec<u8>>) {
            let character = CharacterState {
                guid: 1,
                zone_id: 0,
                position: positions[&npc],
                rotation: [1.0, 0.0, 0.0],
            };
            game.update_npc_visibility(&character).unwrap();
            clock.set(clock.get() + 10);
            game.handle_in_world(&packet_for_npc(npc), &character, clock.get())
                .unwrap();
            let out = game
                .handle_in_world(&select_npc_option(npc, 1), &character, clock.get() + 1)
                .unwrap();
            (character, out.replies)
        };
        let has = |replies: &[Vec<u8>], prefix: &[u8], quest: u32| {
            replies
                .iter()
                .any(|p| p.starts_with(prefix) && p.get(6..10) == Some(&quest.to_le_bytes()[..]))
        };
        let quest_reply = |subtype: u8, quest: u32, accept: bool| {
            let mut packet = vec![49, 0, subtype, 0, 0, 0];
            packet.extend(quest.to_le_bytes());
            if subtype == 2 {
                packet.push(accept as u8);
            }
            packet
        };
        let offer_and_accept = |game: &mut Gameplay, npc: u64, quest: u32| {
            let (character, _) = talk(game, npc);
            let offer = game
                .handle_in_world(&choose_quest_row(), &character, 3_000)
                .unwrap();
            assert!(has(&offer.replies, &[49, 0, 1, 0], quest), "offer {quest}");
            game.handle_in_world(&quest_reply(2, quest, true), &character, 3_001)
                .unwrap();
            character
        };
        let turn_in =
            |game: &mut Gameplay, character: &CharacterState, quest: u32, end_text: u32| {
                let end = game
                    .handle_in_world(&choose_quest_row(), character, 3_002)
                    .unwrap();
                let screen = end
                    .replies
                    .iter()
                    .find(|p| {
                        p.starts_with(&[49, 0, 13, 0])
                            && p.get(14..18) == Some(&quest.to_le_bytes()[..])
                    })
                    .unwrap_or_else(|| panic!("end screen {quest}"));
                // Offset 6 carries the turn-in NPC's GUID for the client's actor lookup.
                assert!(
                    [SAMANTHA_ID, RICKY]
                        .contains(&u64::from_le_bytes(screen[6..14].try_into().unwrap())),
                    "end screen {quest} names its NPC"
                );
                assert_eq!(screen[18..22], end_text.to_le_bytes(), "end text {quest}");
                let done = game
                    .handle_in_world(&quest_reply(14, quest, true), character, 3_003)
                    .unwrap();
                assert!(
                    done.replies.iter().any(|p| p.starts_with(&[49, 0, 4, 0])),
                    "complete {quest}"
                );
            };
        let status = |game: &Gameplay, quest: u32| {
            game.npc_quests
                .as_ref()
                .unwrap()
                .progress(quest)
                .map(|p| p.status.clone())
        };

        offer_and_accept(&mut game, SAMANTHA_ID, INTRODUCE_YOURSELF_ID);
        let (at_ricky, _) = talk(&mut game, RICKY);
        turn_in(&mut game, &at_ricky, INTRODUCE_YOURSELF_ID, 0);

        offer_and_accept(&mut game, RICKY, CALL_THE_CREW);
        let (_, early) = talk(&mut game, RICKY);
        assert!(
            !has(&early, &[49, 0, 13, 0], CALL_THE_CREW),
            "Ricky before Shakey must not complete"
        );
        let (_, at_shakey) = talk(&mut game, SHAKEY);
        let shakey_dialogue = at_shakey
            .iter()
            .find(|p| p.starts_with(&[26, 0, 3, 0]))
            .expect("Shakey answers with a dialog");
        assert_eq!(
            npc_interaction::decode_dialogue_26_3(shakey_dialogue)
                .unwrap()
                .initial_dwords[0],
            94_258,
            "Shakey says his step line"
        );
        assert!(
            has(&at_shakey, &[49, 0, 9, 0], CALL_THE_CREW),
            "Shakey completes step 1"
        );
        let (at_ricky, _) = talk(&mut game, RICKY);
        turn_in(&mut game, &at_ricky, CALL_THE_CREW, 94_257);

        offer_and_accept(&mut game, RICKY, BACK_FOR_MORE);
        let (at_samantha, _) = talk(&mut game, SAMANTHA_ID);
        turn_in(&mut game, &at_samantha, BACK_FOR_MORE, 94_265);
        for quest in [INTRODUCE_YOURSELF_ID, CALL_THE_CREW, BACK_FOR_MORE] {
            assert_eq!(
                status(&game, quest),
                Some(crate::quest_engine::QuestStatus::Completed)
            );
        }
    }
}

#[cfg(test)]
mod w95_vendor_coin_store_regressions {
    use super::*;

    fn reference() -> PathBuf {
        bundled_content()
    }

    fn bundled_content() -> PathBuf {
        PathBuf::from(
            std::env::var_os("OSFR_W95_TEST_CONTENT").expect("private vendor content required"),
        )
    }

    fn count_definition(game: &Gameplay, definition_id: i32) -> u32 {
        game.inventory
            .state()
            .items
            .iter()
            .filter(|item| item.definition_id == definition_id)
            .map(|item| item.count)
            .sum()
    }

    #[test]
    #[ignore = "requires OSFR_W95_TEST_CONTENT pointing at private play content"]
    fn coin_store_and_vendor_buys_preserve_live_inventory_and_saved_state() {
        let reference = reference();
        let mut game = Gameplay::load(&reference, None).expect("gameplay loads");
        let mut state = game.state();
        state.inventory.items.clear();
        for equipped in &mut state.inventory.equipment {
            equipped.item_id = 0;
        }
        state.inventory.coins = 1_000_000;
        game = Gameplay::load(&reference, Some(state)).expect("prepared login state loads");

        // Simulate a reward granted after login while CoinShop still owns its login snapshot.
        game.inventory
            .grant(0, 8, 1)
            .expect("post-login item grant succeeds");
        assert_eq!(count_definition(&game, 8), 1);

        let content = bundled_content();
        let vendors: serde_json::Value = serde_json::from_slice(
            &std::fs::read(content.join("vendors.json")).expect("vendor content reads"),
        )
        .expect("vendor content parses");
        let stores: serde_json::Value = serde_json::from_slice(
            &std::fs::read(content.join("stores.json")).expect("store content reads"),
        )
        .expect("store content parses");
        let mut selected = None;
        for vendor in vendors["vendors"].as_array().expect("vendors are an array") {
            let npc_id = vendor["npc_id"].as_u64().expect("NPC id is an integer");
            let Some((zone_id, position)) = game
                .npc_interaction
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .location(npc_id)
            else {
                continue;
            };
            for item in vendor["item_ids"].as_array().expect("stock is an array") {
                let Some(item_id) = item.as_i64().and_then(|id| i32::try_from(id).ok()) else {
                    continue;
                };
                if item_id == 8 || !game.inventory.has_definition(item_id) {
                    continue;
                }
                let Some(price) = stores["items"]
                    .as_array()
                    .expect("store items are an array")
                    .iter()
                    .find(|row| row["item_id"].as_i64() == Some(i64::from(item_id)))
                    .and_then(|row| row["purchase_price_coins"].as_u64())
                else {
                    continue;
                };
                selected = Some((npc_id, zone_id, position, item_id, price as u32));
                break;
            }
            if selected.is_some() {
                break;
            }
        }
        let (npc_id, zone_id, position, item_id, price) =
            selected.expect("a priced, defined vendor item exists");

        let mut interaction = game.npc_interaction.as_ref().unwrap().lock().unwrap();
        let menu = interaction
            .interact(npc_id, zone_id, position, None)
            .expect("vendor interaction opens");
        let shop_option = menu
            .options
            .iter()
            .find(|option| option.action == npc_interaction::NpcAction::Shop)
            .expect("vendor has a shop option");
        let mut select = vec![26, 0, 10, 0];
        select.extend(npc_id.to_le_bytes());
        select.extend(shop_option.id.to_le_bytes());
        interaction
            .select(&select, zone_id, position, None)
            .expect("vendor shop opens");
        drop(interaction);

        let coins_before = game.inventory.state().coins;
        let purchased_before = count_definition(&game, item_id);
        let mut request = vec![165, 0, 4, 0];
        request.extend(npc_id.to_le_bytes());
        request.extend(99001u32.to_le_bytes());
        request.extend(item_id.to_le_bytes());
        request.extend(1u32.to_le_bytes());
        request.extend(0u32.to_le_bytes());
        let character = CharacterState {
            guid: 7,
            zone_id,
            position,
            rotation: [0.0; 3],
        };
        let outcome = game
            .handle_in_world(&request, &character, 1_000)
            .expect("vendor Coin Store request routes");

        assert!(outcome.changed, "successful purchase requests a save");
        assert_eq!(game.inventory.state().coins, coins_before - price);
        assert_eq!(count_definition(&game, item_id), purchased_before + 1);
        assert_eq!(count_definition(&game, 8), 1, "post-login reward survives");

        let vendor_request =
            npc_interaction::encode_merchant_buy_66_43(&npc_interaction::MerchantBuyRequestV2 {
                initial_dwords: [item_id as u32, 0],
                context_dwords: [0; 3],
                context_bytes: Vec::new(),
                trailing_dwords: [1, 0, 99002],
            })
            .expect("vendor request encodes");
        let vendor_outcome = game
            .handle_in_world(&vendor_request, &character, 2_000)
            .expect("66/43 vendor request routes");
        assert!(vendor_outcome.changed);
        let response = vendor_outcome
            .replies
            .last()
            .expect("vendor result reply exists");
        assert_eq!(response.len(), 56);
        assert_eq!(&response[..4], &[66, 0, 44, 0]);
        assert_eq!(u32::from_le_bytes(response[48..52].try_into().unwrap()), 1);
        assert_eq!(game.inventory.state().coins, coins_before - (price * 2));
        assert_eq!(count_definition(&game, item_id), purchased_before + 2);
        assert_eq!(
            count_definition(&game, 8),
            1,
            "post-login reward still survives"
        );

        let before_refusal = game.state();
        let unknown_request =
            npc_interaction::encode_merchant_buy_66_43(&npc_interaction::MerchantBuyRequestV2 {
                initial_dwords: [i32::MAX as u32, 0],
                context_dwords: [0; 3],
                context_bytes: Vec::new(),
                trailing_dwords: [1, 1, 99003],
            })
            .expect("unknown-item request encodes");
        let refusal = game
            .handle_in_world(&unknown_request, &character, 3_000)
            .expect("unknown vendor item gets a refusal");
        assert!(!refusal.changed);
        assert_eq!(game.state(), before_refusal);
        let response = refusal.replies.last().expect("refusal reply exists");
        assert_eq!(response.len(), 56);
        assert_eq!(&response[..4], &[66, 0, 44, 0]);
        assert_eq!(u32::from_le_bytes(response[48..52].try_into().unwrap()), 0);

        let saved = serde_json::to_vec(&game.state()).expect("changed state serializes");
        let saved: GameplayState = serde_json::from_slice(&saved).expect("saved state parses");
        let restored = Gameplay::load(&reference, Some(saved)).expect("saved state reloads");
        assert_eq!(restored.inventory.state().coins, coins_before - (price * 2));
        assert_eq!(count_definition(&restored, item_id), purchased_before + 2);
        assert_eq!(count_definition(&restored, 8), 1);
    }
}

#[cfg(test)]
mod w104_quest_tests {
    use super::*;
    use crate::quest_engine::{
        CharacterEligibility, ObjectiveKind, QuestDefinition, QuestEngine, QuestObjective,
        QuestRequirements,
    };
    use crate::rewards::{RewardDisplay, RewardGrant};

    fn content() -> PathBuf {
        std::env::var_os("W104_CONTENT")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../../lab/build/w104/scratch/content")
            })
    }

    fn game(kind: ObjectiveKind, required: u32) -> Gameplay {
        let mut game = Gameplay::load(&content(), None).unwrap();
        let definition = QuestDefinition {
            id: 1,
            title: "Synthetic hook (R)".into(),
            description: String::new(),
            giver_npc_id: 1,
            turnin_npc_id: 1,
            prerequisite_quest_ids: vec![],
            requirements: QuestRequirements::default(),
            ordered: true,
            objectives: vec![QuestObjective {
                id: 1,
                required,
                description: "Synthetic (R)".into(),
                kind,
            }],
            reward: RewardGrant {
                coins: 0,
                items: vec![],
                job_xp: Default::default(),
                display: RewardDisplay {
                    source_guid: 1,
                    player_guid: 1,
                    reward_kind: 0,
                    icon_id: 0,
                    name_id: 0,
                    rows: vec![],
                },
            },
        };
        let mut inactive = definition.clone();
        inactive.id = 2;
        let mut engine = QuestEngine::new(
            1,
            vec![definition, inactive],
            CharacterEligibility {
                member: true,
                member_quest_limit: 30,
                ..Default::default()
            },
            None,
        )
        .unwrap();
        assert!(engine.offer(1, 1).changed);
        assert!(engine.accept(1).changed);
        game.npc_quests = Some(engine);
        game
    }

    fn count(game: &Gameplay, expected: u32) {
        let engine = game.npc_quests.as_ref().unwrap();
        assert_eq!(engine.progress(1).unwrap().counts, [expected]);
        assert!(engine.progress(2).is_none());
    }

    fn stage(
        game: &mut Gameplay,
        stage_id: u32,
        profile: i32,
        trigger: OpeningEvent,
    ) -> CharacterState {
        let mut state = game.opening.as_ref().unwrap().state().clone();
        for definition in game.opening.as_ref().unwrap().stages() {
            let progress = state.stages.get_mut(&definition.id).unwrap();
            if definition.id < stage_id {
                progress.accepted = true;
                progress.completed = true;
                progress.rewarded = true;
                progress.counts = definition.objectives.iter().map(|g| g.required).collect();
            } else if definition.id == stage_id {
                progress.accepted = true;
                progress.counts = definition.objectives.iter().map(|g| g.required).collect();
                *progress.counts.last_mut().unwrap() = 0;
            }
        }
        // Memory-only fixture; no campaign content or user save changes.
        game.opening = Some(OpeningEngine::load(&content(), Some(state)).unwrap());
        let profiles = game.profiles.as_mut().unwrap();
        profiles.unlock(profile).unwrap();
        profiles.select(profile, &game.inventory).unwrap();
        let target = game
            .opening
            .as_ref()
            .unwrap()
            .actors()
            .iter()
            .find(|a| match (&a.action, &trigger) {
                (
                    Some(crate::opening::OpeningTrigger::Craft { recipe_id: a }),
                    OpeningEvent::Craft { recipe_id: b, .. },
                ) => a == b,
                _ => false,
            })
            .unwrap();
        CharacterState {
            guid: 1,
            zone_id: target.zone_id,
            position: target.position,
            rotation: [0., 0., 1.],
        }
    }

    #[test]
    #[ignore = "private W104 scratch package; no native client"]
    fn collect_hook_grant_failure_and_inactive_quest() {
        let mut game = game(ObjectiveKind::CollectItem { item_id: 1195 }, 2);
        let mut out = Outcome::default();
        assert!(game.grant_reward(0, &[(i32::MAX, 1)], &mut out).is_err());
        count(&game, 0);
        game.grant_reward(0, &[(1195, 1)], &mut out).unwrap();
        count(&game, 1);
        game.grant_reward(0, &[(1195, 5)], &mut out).unwrap();
        count(&game, 2);
    }

    #[test]
    #[ignore = "private W104 scratch package; no native client"]
    fn equip_hook_accepts_owned_slot_and_ignores_replay() {
        let mut game = game(
            ObjectiveKind::EquipItem {
                definition_id: 68878,
            },
            1,
        );
        let mut out = Outcome::default();
        game.grant_reward(0, &[(68878, 1)], &mut out).unwrap();
        let item = game
            .inventory
            .state()
            .items
            .iter()
            .filter(|i| i.definition_id == 68878)
            .max_by_key(|i| i.id)
            .unwrap()
            .id;
        let mut packet = vec![42, 0, 3, 0];
        packet.extend(item.to_le_bytes());
        packet.extend(1i32.to_le_bytes());
        packet.extend(4i32.to_le_bytes());
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.; 3],
            rotation: [0., 0., 1.],
        };
        game.handle_in_world(&packet, &character, 1_000).unwrap();
        count(&game, 1);
        game.handle_in_world(&packet, &character, 1_001).unwrap();
        count(&game, 1);
    }

    #[test]
    #[ignore = "private W104 scratch package; no native client"]
    fn gathering_payout_hook_and_minigame_hook() {
        let mut game = game(ObjectiveKind::GatherResource { resource_id: 2884 }, 2);
        game.profiles.as_mut().unwrap().unlock(45).unwrap();
        game.profiles
            .as_mut()
            .unwrap()
            .select(45, &game.inventory)
            .unwrap();
        let mut out = Outcome::default();
        game.apply_harvesting(Type8Outcome::default(), &mut out)
            .unwrap();
        count(&game, 0);
        let mut win = mining::payload_packet(111_000_001, "OnGameWonMsg").unwrap();
        win[11..15].copy_from_slice(&111i32.to_le_bytes());
        game.apply_harvesting(
            Type8Outcome {
                transaction: Some(crate::mining::Type8Transaction {
                    consumes: vec![],
                    grants: vec![(2884, 2)],
                }),
                completed: true,
                replies: vec![win.clone()],
                ..Default::default()
            },
            &mut out,
        )
        .unwrap();
        count(&game, 2);
        let mut minigame = self::game(ObjectiveKind::FinishMinigame { minigame_id: 111 }, 1);
        minigame.profiles.as_mut().unwrap().unlock(45).unwrap();
        minigame
            .profiles
            .as_mut()
            .unwrap()
            .select(45, &minigame.inventory)
            .unwrap();
        // A server result without a payout never emits a completion event.
        minigame
            .apply_harvesting(
                Type8Outcome {
                    replies: vec![win.clone()],
                    ..Default::default()
                },
                &mut out,
            )
            .unwrap();
        count(&minigame, 0);
        minigame
            .apply_harvesting(
                Type8Outcome {
                    transaction: Some(crate::mining::Type8Transaction {
                        consumes: vec![],
                        grants: vec![(2884, 1)],
                    }),
                    completed: true,
                    replies: vec![win],
                    ..Default::default()
                },
                &mut out,
            )
            .unwrap();
        count(&minigame, 1);
    }

    #[test]
    #[ignore = "private W104 scratch package; no native client"]
    fn gather_node_hook_checks_accepted_progress() {
        let mut game = game(ObjectiveKind::GatherResource { resource_id: 1001 }, 2);
        stage(
            &mut game,
            910004,
            45,
            OpeningEvent::Craft {
                recipe_id: 2001,
                quantity: 1,
            },
        );
        let opening = game.opening.as_ref().unwrap();
        let definition = opening.active_stage().unwrap();
        let index = definition
            .objectives
            .iter()
            .position(|g| {
                matches!(
                    g.trigger,
                    crate::opening::OpeningTrigger::Gather { resource_id: 1001 }
                )
            })
            .unwrap();
        let mut state = opening.state().clone();
        state.stages.get_mut(&definition.id).unwrap().counts = definition
            .objectives
            .iter()
            .enumerate()
            .map(|(i, g)| if i < index { g.required } else { 0 })
            .collect();
        let actor = opening
            .actors()
            .iter()
            .find(|a| {
                matches!(
                    a.action,
                    Some(crate::opening::OpeningTrigger::Gather { resource_id: 1001 })
                )
            })
            .unwrap()
            .clone();
        game.opening = Some(OpeningEngine::load(&content(), Some(state)).unwrap());
        let mut character = CharacterState {
            guid: 1,
            zone_id: actor.zone_id,
            position: [0.; 3],
            rotation: [0., 0., 1.],
        };
        game.opening
            .as_mut()
            .unwrap()
            .on_event(OpeningEvent::ClassSelected { profile_id: 45 })
            .unwrap();
        game.on_location(&character, 999).unwrap();
        let mut packet = vec![26, 0, 10, 0];
        packet.extend(actor.guid.to_le_bytes());
        packet.extend(1i32.to_le_bytes());
        game.handle_in_world(&packet, &character, 1_000).unwrap();
        count(&game, 0);
        character.position = actor.position;
        game.on_location(&character, 1_001).unwrap();
        game.handle_in_world(&packet, &character, 1_001).unwrap();
        count(&game, 1);
        game.handle_in_world(&packet, &character, 1_002).unwrap();
        count(&game, 2);
    }

    #[test]
    #[ignore = "private W104 scratch package; no native client"]
    fn cooking_craft_hook() {
        let mut game = game(ObjectiveKind::CraftRecipe { recipe_id: 70 }, 1);
        let character = stage(
            &mut game,
            910004,
            45,
            OpeningEvent::Craft {
                recipe_id: 2001,
                quantity: 1,
            },
        );
        let mut out = Outcome::default();
        game.apply_cooking(CookingOutcome::default(), &character, &mut out)
            .unwrap();
        count(&game, 0);
        game.apply_cooking(
            CookingOutcome {
                transaction: Some(crate::cooking::CookingTransaction {
                    consumes: vec![],
                    grants: vec![(2884, 1)],
                    experience_points: 0,
                }),
                completed: true,
                ..Default::default()
            },
            &character,
            &mut out,
        )
        .unwrap();
        count(&game, 1);
    }

    #[test]
    #[ignore = "private W104 scratch package; no native client"]
    fn smelting_craft_hook() {
        let mut game = game(ObjectiveKind::CraftRecipe { recipe_id: 2002 }, 1);
        let character = stage(
            &mut game,
            910005,
            14,
            OpeningEvent::Craft {
                recipe_id: 2002,
                quantity: 1,
            },
        );
        let mut out = Outcome::default();
        game.apply_smelting(
            SmeltingOutcome {
                transaction: Some(crate::smelting::SmeltingTransaction {
                    consumes: vec![],
                    grants: vec![(21785, 1)],
                    experience_points: 0,
                }),
                completed: true,
                ..Default::default()
            },
            &character,
            &mut out,
        )
        .unwrap();
        count(&game, 1);
    }

    #[test]
    #[ignore = "private W104 scratch package; no native client"]
    fn forging_craft_hook() {
        let mut game = game(ObjectiveKind::CraftRecipe { recipe_id: 11 }, 1);
        let character = stage(
            &mut game,
            910006,
            16,
            OpeningEvent::Craft {
                recipe_id: 2003,
                quantity: 1,
            },
        );
        let mut out = Outcome::default();
        game.apply_forging(
            ForgingOutcome {
                transaction: Some(crate::forging::ForgingTransaction {
                    consumes: vec![],
                    grants: vec![(1195, 1)],
                    experience_points: 0,
                }),
                completed: true,
                ..Default::default()
            },
            &character,
            &mut out,
        )
        .unwrap();
        count(&game, 1);
    }

    #[test]
    #[ignore = "private W104 scratch package; no native client"]
    fn movement_hook_area_and_radius() {
        for kind in [
            ObjectiveKind::EnterArea { area_id: 0 },
            ObjectiveKind::VisitPosition {
                zone_id: 0,
                position: [0.; 3],
                radius: 1.,
            },
        ] {
            let mut game = game(kind, 1);
            let mut character = CharacterState {
                guid: 1,
                zone_id: 1,
                position: [0.; 3],
                rotation: [0., 0., 1.],
            };
            game.on_location(&character, 1_000).unwrap();
            count(&game, 0);
            character.zone_id = 0;
            game.on_location(&character, 1_001).unwrap();
            count(&game, 1);
        }
    }
}

#[cfg(test)]
mod t013_objective_events {
    use super::*;
    use crate::quest_engine::{
        CharacterEligibility, ObjectiveKind, QuestDefinition, QuestEngine, QuestObjective,
        QuestRequirements,
    };
    use crate::rewards::{RewardDisplay, RewardGrant};
    use serde_json::json;
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);
    const FISHING_T0: u64 = 1_000_000;
    const FISHING_ACTOR: crate::fishing::Handle = crate::fishing::Handle {
        id: 0x1122_3344,
        seq: 0x5566_7788,
    };

    fn reference() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-opening-cooking-original-assets")
    }

    fn game() -> Gameplay {
        let path = reference();
        assert!(
            path.join("opening.json").is_file(),
            "T013 requires the private opening reference at {}",
            path.display()
        );
        Gameplay::load(&path, None).expect("opening reference loads")
    }

    fn eligibility() -> CharacterEligibility {
        CharacterEligibility {
            member: true,
            member_quest_limit: 30,
            ..CharacterEligibility::default()
        }
    }

    fn definition(id: u32, kind: ObjectiveKind, required: u32) -> QuestDefinition {
        definition_many(id, vec![(1, kind, required)], true)
    }

    fn definition_many(
        id: u32,
        goals: Vec<(u32, ObjectiveKind, u32)>,
        ordered: bool,
    ) -> QuestDefinition {
        QuestDefinition {
            id,
            title: "Synthetic T013 objective (R)".into(),
            description: String::new(),
            giver_npc_id: 1,
            turnin_npc_id: 1,
            prerequisite_quest_ids: vec![],
            requirements: QuestRequirements::default(),
            ordered,
            objectives: goals
                .into_iter()
                .map(|(id, kind, required)| QuestObjective {
                    id,
                    required,
                    description: "Synthetic T013 step (R)".into(),
                    kind,
                })
                .collect(),
            reward: RewardGrant {
                coins: 0,
                items: vec![],
                job_xp: Default::default(),
                display: RewardDisplay {
                    source_guid: 1,
                    player_guid: 1,
                    reward_kind: 0,
                    icon_id: 0,
                    name_id: 0,
                    rows: vec![],
                },
            },
        }
    }

    fn install(game: &mut Gameplay, kind: ObjectiveKind, required: u32) {
        install_many(game, definition(1, kind, required));
    }

    fn install_many(game: &mut Gameplay, definition: QuestDefinition) {
        let mut engine = QuestEngine::new(1, vec![definition], eligibility(), None)
            .expect("synthetic quest engine");
        assert!(engine.offer(1, 1).changed);
        assert!(engine.accept(1).changed);
        game.npc_quests = Some(engine);
    }

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(tag: &str) -> Self {
            let directory = std::env::temp_dir().join(format!(
                "osfr-t013-{tag}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&directory).unwrap();
            Self(directory)
        }

        fn copy_reference_without_opening() -> Self {
            let scratch = Self::new("no-opening");
            copy_tree(&reference(), &scratch.0);
            for name in [
                "opening.json",
                "smelting.json",
                "mining.json",
                "harvesting.json",
                "forging-campaign.json",
                "cooking-campaign.json",
            ] {
                let path = scratch.0.join(name);
                if path.is_file() {
                    fs::remove_file(path).unwrap();
                }
            }
            scratch
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn copy_tree(source: &Path, destination: &Path) {
        fs::create_dir_all(destination).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let target = destination.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    fn type8_board(goals: usize) -> crate::mining::Type8BoardSpec {
        use crate::mining::{SIDE, Type8BoardSpec, Type8Goal, Type8Kind, Type8Output};
        const LEFT: usize = 3;
        const TOP: usize = 3;
        const PLAYABLE: usize = 8;
        const BOTTOM: usize = TOP + PLAYABLE - 1;
        const OUTPUTS: [i32; 4] = [2739, 879, 65834, 2436];
        const VALUES: [u16; 4] = [601, 602, 603, 604];
        let values = &VALUES[..goals];
        let mut shape = vec![0_i32; SIDE * SIDE];
        for dy in 0..PLAYABLE {
            for dx in 0..PLAYABLE {
                shape[(TOP + dy) * SIDE + LEFT + dx] = 1;
            }
        }
        for (slot, value) in values.iter().enumerate() {
            shape[BOTTOM * SIDE + LEFT + slot] = -i32::from(*value);
        }
        for x in LEFT..LEFT + PLAYABLE {
            shape[(TOP + PLAYABLE) * SIDE + x] = 4;
        }
        Type8BoardSpec {
            width: SIDE,
            height: SIDE,
            shape,
            selectable: vec![401, 402, 403, 404],
            collectable: values.to_vec(),
            tiles: [401, 402, 403, 404]
                .iter()
                .map(|value| Type8Kind {
                    value: *value,
                    category: 400,
                    icon_id: i32::from(*value) - 400,
                })
                .chain(values.iter().map(|value| Type8Kind {
                    value: *value,
                    category: 600,
                    icon_id: 700 + i32::from(*value),
                }))
                .collect(),
            goals: values
                .iter()
                .enumerate()
                .map(|(slot, value)| Type8Goal {
                    index: slot as i32,
                    value: *value,
                    quota: 1,
                    per_reward: 1,
                    icon_id: 700 + i32::from(*value),
                })
                .collect(),
            outputs: values
                .iter()
                .enumerate()
                .map(|(slot, value)| Type8Output {
                    value: *value,
                    definition_id: OUTPUTS[slot],
                    cluster_quantity: 1,
                })
                .collect(),
            duration_seconds: 180,
            seed: 112,
            wildcard_percent: 0,
            collectable_percent: 0,
            blocker_percent: 0,
            powerup_percent: 0,
            meter_id: 0,
            reconstructed_rules: true,
            rule_notice: "Synthetic private F112 fixture; original level tables unrecovered."
                .into(),
        }
    }

    fn gathering_game(profile: i32, goals: usize, quest: QuestDefinition) -> (Gameplay, Scratch) {
        let scratch = Scratch::new("type8");
        let mining = json!({
            "schema_version": 1,
            "game_id": crate::mining::MINING_GAME_ID,
            "profile_id": crate::mining::MINER_PROFILE_ID,
            "name_id": 7910,
            "description_id": 7911,
            "icon_id": 909,
            "spec": type8_board(goals),
        });
        let harvesting = json!({
            "schema_version": 1,
            "game_id": HARVESTING_GAME_ID,
            "profile_id": harvesting::CHEF_PROFILE_ID,
            "name_id": 100834,
            "description_id": 94150,
            "icon_id": 908,
            "preload_file": harvesting::FARMING_PRELOAD_FILE,
            "launch_movie": harvesting::FARMING_LAUNCH_MOVIE,
            "spec": type8_board(goals),
        });
        fs::write(
            scratch.0.join(mining::CONTENT_FILE),
            serde_json::to_vec(&mining).unwrap(),
        )
        .unwrap();
        fs::write(
            scratch.0.join(harvesting::CONTENT_FILE),
            serde_json::to_vec(&harvesting).unwrap(),
        )
        .unwrap();
        let mut game = game();
        game.mining = Some(
            Type8Engine::load(
                &scratch.0,
                mining::CONTENT_FILE,
                mining::MINING_GAME_ID,
                mining::FIRST_MINING_INSTANCE,
                None,
            )
            .expect("synthetic mining content"),
        );
        game.harvesting = Some(
            Type8Engine::load(
                &scratch.0,
                harvesting::CONTENT_FILE,
                HARVESTING_GAME_ID,
                harvesting::FIRST_INSTANCE_ID,
                None,
            )
            .expect("synthetic harvesting content"),
        );
        install_many(&mut game, quest);
        select(&mut game, profile);
        (game, scratch)
    }

    fn gather_node(game: &Gameplay, name: &str, skip: usize) -> (u64, CharacterState) {
        let actor = game
            .gathering_nodes
            .as_ref()
            .expect("gathering nodes")
            .actors()
            .filter(|actor| actor.name == name)
            .nth(skip)
            .expect("gathering node")
            .clone();
        (
            actor.guid,
            CharacterState {
                guid: 1,
                zone_id: 0,
                position: actor.position,
                rotation: [0., 0., 1.],
            },
        )
    }

    fn node_click(guid: u64) -> Vec<u8> {
        let mut packet = vec![26, 0, 8, 0];
        packet.extend(guid.to_le_bytes());
        packet.push(0);
        packet
    }

    fn type8_frame(sub: u8, instance: i32) -> Vec<u8> {
        let mut packet = vec![39, 0, sub];
        packet.extend(instance.to_le_bytes());
        packet.extend((-1i32).to_le_bytes());
        packet.extend((-1i32).to_le_bytes());
        packet
    }

    fn type8_payload(instance: i32, text: &str) -> Vec<u8> {
        let mut packet = type8_frame(14, instance);
        packet.extend(((text.len() + 1) as i32).to_le_bytes());
        packet.extend(text.as_bytes());
        packet.push(0);
        packet
    }

    fn cooking_movie_request(sub: u8, instance: i32) -> Vec<u8> {
        let mut packet = vec![39, 0, sub];
        packet.extend(instance.to_le_bytes());
        packet.extend((-1i32).to_le_bytes());
        packet.extend((-1i32).to_le_bytes());
        packet
    }

    fn play_cooking(game: &mut Gameplay, character: &CharacterState, base_ms: u64) {
        let instance = game
            .cooking
            .as_ref()
            .unwrap()
            .state()
            .active
            .as_ref()
            .expect("cooking run")
            .instance_id;
        send(
            game,
            character,
            &cooking_movie_request(5, instance),
            base_ms,
        );
        send(
            game,
            character,
            &crate::cooking::payload_packet(instance, "FRServer_ReadyForGameData").unwrap(),
            base_ms + 1,
        );
        for stage in [1, 2, 3, 6, 17, 7] {
            for kind in 0..4 {
                let text = format!("FRServer_StatUpdate\t{stage}\t{kind}\t1)");
                send(
                    game,
                    character,
                    &crate::cooking::payload_packet(instance, &text).unwrap(),
                    base_ms + 2,
                );
            }
        }
        for kind in 4..8 {
            let text = format!("FRServer_StatUpdate\t-1\t{kind}\t6)");
            send(
                game,
                character,
                &crate::cooking::payload_packet(instance, &text).unwrap(),
                base_ms + 3,
            );
        }
        send(
            game,
            character,
            &crate::cooking::payload_packet(instance, "FRServer_GameWon\t900\t0").unwrap(),
            base_ms + 4,
        );
    }

    fn send(game: &mut Gameplay, character: &CharacterState, packet: &[u8], now: u64) -> Outcome {
        game.handle_in_world(packet, character, now)
            .expect("gameplay frame")
    }

    fn finish_type8(
        game: &mut Gameplay,
        character: &CharacterState,
        instance: i32,
        goals: usize,
        base_ms: u64,
    ) {
        for (step, packet) in [
            type8_frame(5, instance),
            type8_payload(instance, "OnConnectMsg"),
            type8_payload(instance, "OnStartGameMsg"),
            type8_payload(instance, "OnStartLevelMsg"),
        ]
        .iter()
        .enumerate()
        {
            send(game, character, packet, base_ms + step as u64 * 100);
        }
        for slot in 0..goals {
            let text = format!("OnLevelEventMsg\t50\t{}\t{}", 3 + slot, 10);
            send(
                game,
                character,
                &type8_payload(instance, &text),
                base_ms + 500 + slot as u64,
            );
        }
        send(
            game,
            character,
            &type8_payload(instance, "OnNextLevelMsg"),
            base_ms + 600,
        );
    }

    fn kart_character() -> CharacterState {
        CharacterState {
            guid: 1,
            zone_id: 0,
            position: [10., 20., 30.],
            rotation: [0., 0., 1.],
        }
    }

    fn kart_gateway(scratch: &Scratch) -> crate::kart_gateway::KartGateway {
        use crate::{
            kart_entry::{ProxiedVehicle, RaceConfig, RaceEntryContent, RaceEntryContext},
            kart_gateway::KartGatewayContent,
            kart_progress_native::ProgressPayload,
            kart_racing::RewardSchedule,
            kart_service::{
                GateWaypoint, KartServiceContent, RaceBounds, RaceStep, ScheduledFrame,
            },
            zone_transition::ZoneTransition,
        };
        let context = RaceEntryContext {
            session: 123,
            instance_id: 1008,
            encounter_id: 917566,
            scene: ZoneTransition {
                envelope_flag: 1,
                scene: "sg_kt_track".into(),
                type_id: 2,
                position: [719., 72., 421., 1.],
                rotation: [0., 0., 1., 0.],
                sky: "sky.xml".into(),
                geometry_id: 101,
                ..Default::default()
            },
        };
        let details = |state: u32| {
            let mut bytes = vec![0u8; 391];
            bytes[..4].copy_from_slice(&[41, 0, 114, 0]);
            for (offset, word) in [
                (4, context.encounter_id),
                (8, context.instance_id),
                (12, context.encounter_id),
                (16, state),
            ] {
                bytes[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
            }
            bytes[32..40].copy_from_slice(&context.session.to_le_bytes());
            for i in 0..3 {
                bytes[51 + i * 4..55 + i * 4]
                    .copy_from_slice(&context.scene.position[i].to_le_bytes());
            }
            bytes
        };
        let progress = ProgressPayload {
            racer_guid: 0,
            slot: 0,
            unknown_0c_bits: 0,
            unknown_10_bits: 0x7f7f_ffff,
            laps_completed: 0,
            local_racer: 1,
            lap_limit: 2,
            unknown_1d_bits: 0,
            best_lap_bits: 0,
            total_time_bits: 0,
            tail_0x29: 0,
            tail_0x31: 0,
            tail_0x39: 0,
            tail_0x41: 0,
        };
        let step = |at_ms, frame| RaceStep { at_ms, frame };
        let race = KartServiceContent {
            speedway: 31,
            laps: 2,
            reward: Some(RewardSchedule { coins: 40, xp: 60 }),
            gates: (1..=4)
                .map(|i| GateWaypoint {
                    position: [i as f32 * 10.0, 0.0, 0.0],
                    radius: 3.0,
                })
                .collect(),
            schedule: vec![
                step(3_026, ScheduledFrame::GameState(4)),
                step(3_026, ScheduledFrame::NpcCount(0)),
                step(3_353, ScheduledFrame::GameState(5)),
                step(8_246, ScheduledFrame::GameState(6)),
                step(8_694, ScheduledFrame::GameState(7)),
                step(9_290, ScheduledFrame::GameState(8)),
                step(9_290, ScheduledFrame::Countdown(4)),
                step(10_074, ScheduledFrame::Countdown(3)),
                step(10_529, ScheduledFrame::Countdown(2)),
                step(11_954, ScheduledFrame::Countdown(1)),
                step(13_241, ScheduledFrame::Countdown(0)),
                step(14_505, ScheduledFrame::GameState(9)),
            ],
            progress_template: ProgressPayload {
                racer_guid: 0,
                ..progress
            },
            start_position: [0.0, 0.0, 0.0],
            bounds: RaceBounds {
                max_speed_mps: 60.0,
                max_step_ms: 60_000,
                max_race_ms: 600_000,
                max_claims: 64,
                max_observations: 4_096,
            },
            gate_band: None,
            gate_curtains: Vec::new(),
            pickups: Vec::new(),
            pickup_respawn: None,
            weapon_effects: false,
        };
        let content = KartGatewayContent {
            schema_version: 1,
            details_loading: details(2),
            details_ready: details(4),
            vehicle_guid_offset: 0,
            race: Some(race),
            entry: RaceEntryContent {
                race_config: RaceConfig { words: vec![0; 21] },
                entry_vehicles: vec![ProxiedVehicle {
                    body_asset: "test.cdt".into(),
                    first_attribute: 400.,
                    attribute_block: 123u64.to_le_bytes().to_vec(),
                }],
                client_data: vec![0; 85],
                player_count: 1,
                npc_count: 0,
                progress: vec![ProgressPayload {
                    racer_guid: 123,
                    slot: 0,
                    unknown_0c_bits: 0,
                    unknown_10_bits: 0,
                    laps_completed: 0,
                    local_racer: 0,
                    lap_limit: 2,
                    unknown_1d_bits: 0,
                    best_lap_bits: 0,
                    total_time_bits: 0,
                    tail_0x29: 123,
                    tail_0x31: 0,
                    tail_0x39: 0,
                    tail_0x41: 0,
                }],
            },
            context,
        };
        fs::write(
            scratch.0.join("kart_entry.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        crate::kart_gateway::KartGateway::load(&scratch.0, None)
            .unwrap()
            .unwrap()
    }

    fn kart_register() -> Vec<u8> {
        let mut packet = vec![33, 0, 9, 0];
        packet.extend(1u64.to_le_bytes());
        packet.extend([0; 8]);
        packet
    }

    fn kart_go(encounter: u32, instance: u32) -> Vec<u8> {
        let mut packet = vec![41, 0, 108, 0];
        packet.extend(encounter.to_le_bytes());
        packet.extend(instance.to_le_bytes());
        packet.extend(1u64.to_le_bytes());
        packet
    }

    fn kart_gate(gate: f32) -> Vec<u8> {
        let mut packet = vec![33, 0, 7, 0];
        packet.extend(1u64.to_le_bytes());
        for value in [gate * 10.0, 0.0, 0.0, 1.0] {
            packet.extend(value.to_le_bytes());
        }
        packet
    }

    fn count(game: &Gameplay, expected: u32) {
        count_id(game, 1, expected);
    }

    fn count_id(game: &Gameplay, quest_id: u32, expected: u32) {
        let progress = game
            .npc_quests
            .as_ref()
            .expect("quest engine installed")
            .progress(quest_id)
            .expect("active quest exists");
        assert_eq!(progress.counts, vec![expected]);
    }

    fn select(game: &mut Gameplay, profile: i32) {
        let profiles = game.profiles.as_mut().expect("reference profiles");
        profiles.unlock(profile).expect("profile exists");
        profiles
            .select(profile, &game.inventory)
            .expect("profile can be selected");
    }

    fn station(game: &Gameplay, recipe_id: u32) -> CharacterState {
        let actor = game
            .opening
            .as_ref()
            .expect("reference opening")
            .actors()
            .iter()
            .find(|actor| {
                matches!(
                    actor.action,
                    Some(crate::opening::OpeningTrigger::Craft { recipe_id: id })
                        if id == recipe_id
                )
            })
            .expect("reference crafting station")
            .clone();
        CharacterState {
            guid: 1,
            zone_id: actor.zone_id,
            position: actor.position,
            rotation: [0., 0., 1.],
        }
    }

    fn fishing_request(sub: u16, body: &[u8]) -> Vec<u8> {
        let mut packet = vec![138, 0];
        packet.extend(sub.to_le_bytes());
        packet.extend(body);
        packet
    }

    fn fishing_actor_bytes() -> Vec<u8> {
        [
            FISHING_ACTOR.id.to_le_bytes(),
            FISHING_ACTOR.seq.to_le_bytes(),
        ]
        .concat()
    }

    fn fishing_register() -> Vec<u8> {
        let mut body = fishing_actor_bytes();
        body.push(0);
        fishing_request(2, &body)
    }

    fn fishing_cast() -> Vec<u8> {
        let mut body = fishing_actor_bytes();
        for coordinate in [10.0f32, 0.0, 0.0, 1.0] {
            body.extend(coordinate.to_le_bytes());
        }
        body.push(0);
        fishing_request(6, &body)
    }

    fn fishing_reel() -> Vec<u8> {
        let mut body = fishing_actor_bytes();
        body.push(1);
        fishing_request(7, &body)
    }

    fn reach_engine(position: [f32; 3], radius: f32) -> QuestEngine {
        let directory = std::env::temp_dir().join(format!(
            "osfr-t013-reach-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(directory.join("quests")).unwrap();
        fs::create_dir_all(directory.join("npcs")).unwrap();
        let source = json!([
            {"kind": "reconstructed", "evidence": "(R) T013 synthetic reach objective"}
        ]);
        fs::write(
            directory.join("quests/synthetic.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "records": [{
                    "id": 7001,
                    "title_string_id": 1,
                    "giver_npc_id": 700,
                    "prerequisite_quest_ids": [],
                    "objectives": [{
                        "kind": "reach_poi",
                        "id": 1,
                        "poi_id": 9,
                        "zone_id": 0,
                        "position": position,
                        "radius": radius,
                        "source": source.clone()
                    }],
                    "rewards": [],
                    "source": source.clone()
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            directory.join("jobs.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "records": [{
                    "id": "synthetic",
                    "name": "Synthetic",
                    "profile_id": null,
                    "category": "freestyle",
                    "max_level": 20,
                    "members_only": false,
                    "trainer_npc_ids": [],
                    "quest_ids": [7001],
                    "abilities": [],
                    "source": source
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            directory.join("npcs/synthetic.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "records": [{
                    "npc_id": 700,
                    "quest_ids_given": [7001],
                    "quests_given": [],
                    "quests_accepted": [{"quest_id": 7001}]
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        let mut engine = QuestEngine::from_split_content(&directory, 1, eligibility(), None)
            .expect("reach quest");
        assert!(engine.offer(7001, 700).changed);
        assert!(engine.accept(7001).changed);
        let _ = fs::remove_dir_all(directory);
        engine
    }

    #[test]
    fn collect_reward_grant_advances_the_active_collect_objective() {
        let mut game = game();
        install(&mut game, ObjectiveKind::CollectItem { item_id: 1195 }, 2);
        let mut out = Outcome::default();
        game.grant_reward(0, &[(1195, 1)], &mut out).unwrap();
        count(&game, 1);

        let before = game.state();
        assert!(game.grant_reward(0, &[(i32::MAX, 1)], &mut out).is_err());
        assert_eq!(game.state(), before);
        count(&game, 1);

        game.grant_reward(0, &[(1195, 1)], &mut out).unwrap();
        count(&game, 2);
    }

    #[test]
    fn ordinary_harvest_nodes_advance_gather_twice_through_gameplay() {
        let (mut game, _scratch) = gathering_game(
            harvesting::CHEF_PROFILE_ID,
            4,
            definition(1, ObjectiveKind::GatherResource { resource_id: 2739 }, 2),
        );
        let (first_guid, first) = gather_node(&game, "Mushrooms", 0);
        game.tick(&first, 1_000).unwrap();
        let clicked = send(&mut game, &first, &node_click(first_guid), 1_100);
        assert!(clicked.changed);
        let first_id = game
            .harvesting
            .as_ref()
            .unwrap()
            .state()
            .active
            .as_ref()
            .expect("first harvest run")
            .instance_id;
        finish_type8(&mut game, &first, first_id, 4, 2_000);
        count(&game, 1);

        send(&mut game, &first, &[26, 0, 42, 0], 3_000);
        let (second_guid, second) = gather_node(&game, "Mushrooms", 1);
        send(&mut game, &second, &node_click(second_guid), 3_100);
        let second_id = game
            .harvesting
            .as_ref()
            .unwrap()
            .state()
            .active
            .as_ref()
            .expect("second harvest run")
            .instance_id;
        finish_type8(&mut game, &second, second_id, 4, 4_000);
        count(&game, 2);
    }

    #[test]
    fn recipe_book_cooking_completes_twice_through_gameplay() {
        let mut game = game();
        let recipe_id = 70;
        install(&mut game, ObjectiveKind::CraftRecipe { recipe_id }, 2);
        select(&mut game, 45);
        let character = station(&game, 2001);
        for definition in [2862, 2875, 2884, 2900] {
            game.inventory.grant(0, definition, 8).unwrap();
        }

        for round in 0..2u64 {
            let mut start = vec![26, 0, 24, 0];
            start.extend(recipe_id.to_le_bytes());
            send(&mut game, &character, &start, 1_000 + round * 2_000);
            assert!(game.cooking.as_ref().unwrap().active());
            let instance = game
                .cooking
                .as_ref()
                .unwrap()
                .state()
                .active
                .as_ref()
                .unwrap()
                .instance_id;
            if round == 0 {
                let before = game.state();
                assert!(
                    game.handle_in_world(
                        &crate::cooking::payload_packet(instance, "FRServer_GameWon\t900\t0")
                            .unwrap(),
                        &character,
                        1_010,
                    )
                    .is_err(),
                    "an early win is refused"
                );
                assert_eq!(game.state(), before);
                count(&game, 0);
            }
            play_cooking(&mut game, &character, 2_000 + round * 2_000);
            assert_eq!(
                game.npc_quests
                    .as_ref()
                    .unwrap()
                    .progress(1)
                    .unwrap()
                    .counts,
                vec![round as u32 + 1]
            );
            send(
                &mut game,
                &character,
                &[26, 0, 42, 0],
                3_000 + round * 2_000,
            );
        }
        count(&game, 2);
    }

    #[test]
    fn ordinary_mining_node_advances_gather_and_minigame_through_gameplay() {
        let (mut game, _scratch) = gathering_game(
            crate::mining::MINER_PROFILE_ID,
            1,
            definition_many(
                1,
                vec![
                    (1, ObjectiveKind::GatherResource { resource_id: 2739 }, 1),
                    (2, ObjectiveKind::FinishMinigame { minigame_id: 18 }, 1),
                ],
                false,
            ),
        );
        let (node_guid, character) = gather_node(&game, "Copper Vein", 0);
        game.tick(&character, 1_000).unwrap();
        game.profiles
            .as_mut()
            .unwrap()
            .select(1, &game.inventory)
            .unwrap();
        let before = game.state();
        let refused = send(&mut game, &character, &node_click(node_guid), 1_100);
        assert!(
            refused.replies.iter().any(|packet| {
                String::from_utf8_lossy(packet).contains("Switch to the Miner job")
            })
        );
        assert_eq!(game.state(), before);
        assert_eq!(
            game.npc_quests
                .as_ref()
                .unwrap()
                .progress(1)
                .unwrap()
                .counts,
            vec![0, 0]
        );

        select(&mut game, crate::mining::MINER_PROFILE_ID);
        send(&mut game, &character, &node_click(node_guid), 1_200);
        let instance = game
            .mining
            .as_ref()
            .unwrap()
            .state()
            .active
            .as_ref()
            .expect("mining run")
            .instance_id;
        finish_type8(&mut game, &character, instance, 1, 2_000);
        assert_eq!(
            game.npc_quests
                .as_ref()
                .unwrap()
                .progress(1)
                .unwrap()
                .counts,
            vec![1, 1]
        );
    }

    #[test]
    fn kart_completion_reaches_quests_only_after_a_paid_claim() {
        let scratch = Scratch::new("kart");
        let character = kart_character();
        let mut kart = kart_gateway(&scratch);
        kart.begin(&character).unwrap();
        kart.set_previous_job(43);
        let mut game = game();
        select(&mut game, 48);
        install(
            &mut game,
            ObjectiveKind::FinishMinigame { minigame_id: 31 },
            1,
        );
        game.kart_entry = Some(kart);

        send(&mut game, &character, &kart_register(), 100_000);
        let active = game
            .kart_entry
            .as_ref()
            .unwrap()
            .state()
            .active
            .as_ref()
            .unwrap()
            .context
            .clone();
        send(
            &mut game,
            &character,
            &kart_go(active.encounter_id, active.instance_id),
            100_500,
        );
        let green = 106_500;
        game.tick(&character, green).unwrap();
        let mut now = green;
        for gate in [1., 2., 3., 4., 1., 2., 3.] {
            now += 1_000;
            send(&mut game, &character, &kart_gate(gate), now);
        }

        game.profiles
            .as_mut()
            .unwrap()
            .select(1, &game.inventory)
            .unwrap();
        let before_refusal = (
            game.inventory.state().clone(),
            game.inventory.state().coins,
            game.profiles.as_ref().unwrap().state().clone(),
            game.npc_quests.as_ref().unwrap().state().clone(),
        );
        now += 1_000;
        send(&mut game, &character, &kart_gate(4.0), now);
        assert_eq!(game.inventory.state().coins, 0, "no unpaid reward");
        assert_eq!(
            game.profiles
                .as_ref()
                .unwrap()
                .state()
                .experience
                .get(&48)
                .copied()
                .unwrap_or(0),
            0
        );
        assert_eq!(
            game.npc_quests
                .as_ref()
                .unwrap()
                .progress(1)
                .unwrap()
                .counts,
            vec![0],
            "a refused payout does not complete the quest"
        );
        assert_eq!(
            (
                game.inventory.state().clone(),
                game.inventory.state().coins,
                game.profiles.as_ref().unwrap().state().clone(),
                game.npc_quests.as_ref().unwrap().state().clone(),
            ),
            before_refusal
        );

        select(&mut game, 48);
        now += 1_000;
        send(&mut game, &character, &kart_gate(4.0), now);
        assert_eq!(game.inventory.state().coins, 40);
        assert_eq!(
            game.profiles
                .as_ref()
                .unwrap()
                .state()
                .experience
                .get(&48)
                .copied()
                .unwrap_or(0),
            60
        );
        count(&game, 1);

        let paid = (
            game.inventory.state().clone(),
            game.profiles.as_ref().unwrap().state().clone(),
            game.npc_quests.as_ref().unwrap().state().clone(),
        );
        now += 1_000;
        send(&mut game, &character, &kart_gate(4.0), now);
        assert_eq!(
            (
                game.inventory.state().clone(),
                game.profiles.as_ref().unwrap().state().clone(),
                game.npc_quests.as_ref().unwrap().state().clone(),
            ),
            paid,
            "a replayed finish neither pays nor advances the quest"
        );
        count(&game, 1);
    }

    #[test]
    fn fishing_pays_both_objectives_twice_and_replay_changes_nothing() {
        let mut game = game();
        install_many(
            &mut game,
            definition_many(
                1,
                vec![
                    (1, ObjectiveKind::CollectItem { item_id: 2884 }, 3),
                    (2, ObjectiveKind::FinishMinigame { minigame_id: 595 }, 3),
                ],
                false,
            ),
        );
        let content: serde_json::Value =
            serde_json::from_str(crate::fishing_service::FishingService::content_json()).unwrap();
        for fish in content["species"].as_array().unwrap() {
            let key = fish["key"].as_str().unwrap();
            for grade in 1..=4 {
                game.fishing.bind_item_definition(key, grade, 2884).unwrap();
            }
        }
        let rewards = crate::rewards::RewardState::new(
            game.inventory.clone(),
            game.profiles.as_ref().unwrap().state().clone(),
            100,
        );
        game.fishing
            .begin(
                1,
                595,
                9001,
                "sacred_grove_shallows",
                1,
                crate::fishing::Vec4 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
                rewards,
            )
            .unwrap();
        let instance = game.fishing.instance_id(1).unwrap();
        let mut start = vec![39, 0, 5];
        start.extend(instance.to_le_bytes());
        start.extend((-1i32).to_le_bytes());
        start.extend((-1i32).to_le_bytes());

        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.; 3],
            rotation: [0., 0., 1.],
        };
        let started = send(&mut game, &character, &start, FISHING_T0);
        assert!(!started.replies.is_empty(), "the lifecycle start routes");
        send(&mut game, &character, &fishing_register(), FISHING_T0 + 1);

        let economy = |game: &Gameplay| {
            (
                game.inventory.state().clone(),
                game.inventory.state().coins,
                game.profiles.as_ref().unwrap().state().clone(),
                game.npc_quests.as_ref().unwrap().state().clone(),
            )
        };
        for caught in 1..=3u32 {
            let at = FISHING_T0 + u64::from(caught) * 10_000;
            send(&mut game, &character, &fishing_cast(), at + 10);
            game.tick(&character, at + 1_510).unwrap();
            game.tick(&character, at + 2_410).unwrap();
            let outcome = send(&mut game, &character, &fishing_reel(), at + 2_420);
            assert!(outcome.changed);
            assert_eq!(
                game.npc_quests
                    .as_ref()
                    .unwrap()
                    .progress(1)
                    .unwrap()
                    .counts,
                vec![caught, caught]
            );
            if caught == 2 {
                let before = economy(&game);
                let replay = send(&mut game, &character, &fishing_reel(), at + 2_430);
                assert!(!replay.changed, "a replayed reel is not a catch");
                assert_eq!(economy(&game), before);
                assert_eq!(
                    game.npc_quests
                        .as_ref()
                        .unwrap()
                        .progress(1)
                        .unwrap()
                        .counts,
                    vec![2, 2],
                    "the refusal leaves both live objectives unchanged"
                );
            }
        }
        assert_eq!(
            game.npc_quests
                .as_ref()
                .unwrap()
                .progress(1)
                .unwrap()
                .counts,
            vec![3, 3]
        );
    }

    #[test]
    fn scene_entry_advances_the_active_area_objective() {
        let mut game = game();
        install(&mut game, ObjectiveKind::EnterArea { area_id: 0 }, 1);
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.; 3],
            rotation: [0., 0., 1.],
        };
        game.initialize_world(&character, 1_000).unwrap();
        count(&game, 1);

        let mut refused = game_without_job(ObjectiveKind::EnterArea { area_id: 0 });
        let before = refused.npc_quests.as_ref().unwrap().state().clone();
        refused
            .on_location(
                &CharacterState {
                    zone_id: 1,
                    ..character
                },
                2_000,
            )
            .unwrap();
        assert_eq!(refused.npc_quests.as_ref().unwrap().state(), &before);
        count(&refused, 0);
    }

    #[test]
    fn scene_entry_advances_the_active_reach_objective() {
        let mut game = game();
        game.npc_quests = Some(reach_engine([0.; 3], 2.0));
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [1.; 3],
            rotation: [0., 0., 1.],
        };
        game.initialize_world(&character, 1_000).unwrap();
        count_id(&game, 7001, 1);

        let mut refused = game_without_job(ObjectiveKind::EnterArea { area_id: 0 });
        refused.npc_quests = Some(reach_engine([0.; 3], 2.0));
        let before = refused.npc_quests.as_ref().unwrap().state().clone();
        refused
            .on_location(
                &CharacterState {
                    position: [10.; 3],
                    ..character
                },
                2_000,
            )
            .unwrap();
        assert_eq!(refused.npc_quests.as_ref().unwrap().state(), &before);
        count_id(&refused, 7001, 0);
    }

    #[test]
    fn scene_entry_advances_area_and_reach_without_opening_content() {
        let scratch = Scratch::copy_reference_without_opening();
        let mut game = Gameplay::load(&scratch.0, None).expect("package without opening.json");
        assert!(game.opening.is_none());
        install_many(
            &mut game,
            definition_many(
                1,
                vec![
                    (1, ObjectiveKind::EnterArea { area_id: 0 }, 1),
                    (
                        2,
                        ObjectiveKind::VisitPosition {
                            zone_id: 0,
                            position: [0.; 3],
                            radius: 2.,
                        },
                        1,
                    ),
                ],
                false,
            ),
        );
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [1.; 3],
            rotation: [0., 0., 1.],
        };
        game.initialize_world(&character, 1_000).unwrap();
        assert_eq!(
            game.npc_quests
                .as_ref()
                .unwrap()
                .progress(1)
                .unwrap()
                .counts,
            vec![1, 1]
        );
    }

    fn game_without_job(kind: ObjectiveKind) -> Gameplay {
        let mut game = game();
        install(&mut game, kind, 1);
        game
    }
}

#[cfg(test)]
mod t018_station_refresh {
    use super::*;
    use crate::world::{PLACEMENT_GUID_BASE, Placement, PlacementFile};
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);
    const STATION: i32 = 990001;
    const FIRST: [f32; 3] = [321.0, 0.0, 654.0];
    const SECOND: [f32; 3] = [1_321.0, 0.0, 1_654.0];

    fn content_directory() -> PathBuf {
        std::env::var_os("OSFR_OPENING_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../../lab/references/rust-opening-cooking-original-assets")
            })
    }

    fn live_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "osfr-station-refresh-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn character_at(position: [f32; 3]) -> CharacterState {
        CharacterState {
            guid: 1,
            zone_id: 0,
            position,
            rotation: [0.0, 0.0, 1.0],
        }
    }

    fn placement(position: [f32; 3]) -> Placement {
        Placement {
            guid: PLACEMENT_GUID_BASE + 1,
            model_id: 1733,
            name_string_id: 0,
            name: None,
            position,
            heading: 0.0,
            scale: 1.0,
            extra: serde_json::Map::new(),
        }
    }

    fn write_placements(dir: &Path, positions: &[[f32; 3]]) {
        let records = positions.iter().copied().map(placement).collect();
        fs::write(
            dir.join("npc_placements.json"),
            serde_json::to_vec(&PlacementFile {
                schema_version: 1,
                records,
            })
            .unwrap(),
        )
        .unwrap();
    }

    fn start_request() -> Vec<u8> {
        let mut packet = vec![26, 0, 24, 0];
        packet.extend(STATION.to_le_bytes());
        packet
    }

    fn stock_recipe(state: &mut GameplayState) {
        let mut next = state
            .inventory
            .items
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        for (definition_id, count) in [(2739, 5), (879, 3)] {
            state.inventory.items.push(crate::inventory::InventoryItem {
                definition_id,
                tint: 0,
                id: next,
                count,
                consumed_count: 0,
                stored_count: 0,
                last_cast_time: 0,
                rental: None,
                ability_count: 0,
                activate_enabled: true,
            });
            next += 1;
        }
        state.inventory.next_instance_id = Some(i64::from(next));
    }

    fn station_game(dir: &Path) -> Gameplay {
        let content = content_directory();
        assert!(content.join("opening.json").is_file());
        let mut state = Gameplay::load(&content, None).unwrap().state();
        let profiles = state.profiles.as_mut().unwrap();
        profiles.unlocked.insert(14);
        profiles.active = 14;
        stock_recipe(&mut state);
        let mut game = Gameplay::load(&content, Some(state)).unwrap();
        game.set_world_dir(Some(dir.to_path_buf())).unwrap();
        game
    }

    fn reload(game: &mut Gameplay) {
        let dir = game.live_dir();
        game.set_world_dir(Some(dir)).unwrap();
    }

    fn start(game: &mut Gameplay, at: [f32; 3]) -> Outcome {
        game.handle_in_world(&start_request(), &character_at(at), 2_000)
            .unwrap()
    }

    fn complete(game: &mut Gameplay, at: [f32; 3]) -> io::Result<()> {
        game.apply_smelting(
            SmeltingOutcome {
                transaction: Some(crate::smelting::SmeltingTransaction {
                    consumes: Vec::new(),
                    grants: Vec::new(),
                    experience_points: 0,
                }),
                completed: true,
                ..Default::default()
            },
            &character_at(at),
            &mut Outcome::default(),
        )
    }

    #[test]
    fn moved_station_changes_recipe_start_and_completion_coordinates() {
        let dir = live_dir();
        fs::create_dir_all(&dir).unwrap();
        write_placements(&dir, &[FIRST]);

        let mut game = station_game(&dir);
        assert!(game.near_craft_station(&character_at(FIRST), crate::recipes::CraftingJob::Miner));
        write_placements(&dir, &[SECOND]);
        reload(&mut game);
        assert!(!game.near_craft_station(&character_at(FIRST), crate::recipes::CraftingJob::Miner));
        assert!(game.near_craft_station(&character_at(SECOND), crate::recipes::CraftingJob::Miner));

        let refused = start(&mut game, FIRST);
        assert!(String::from_utf8_lossy(&refused.replies.concat()).contains("Move closer"));
        assert!(
            game.smelting
                .as_ref()
                .is_none_or(|smelting| !smelting.active())
        );
        start(&mut game, SECOND);
        assert!(game.smelting.as_ref().is_some_and(Smelting::active));

        let mut completion_game = station_game(&dir);
        write_placements(&dir, &[FIRST]);
        reload(&mut completion_game);
        start(&mut completion_game, FIRST);
        write_placements(&dir, &[SECOND]);
        reload(&mut completion_game);
        let error = complete(&mut completion_game, FIRST).unwrap_err();
        assert!(error.to_string().contains("outside a Miner station"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn removed_station_changes_recipe_start_and_completion_coordinates() {
        let dir = live_dir();
        fs::create_dir_all(&dir).unwrap();
        write_placements(&dir, &[FIRST]);

        let mut game = station_game(&dir);
        assert!(game.near_craft_station(&character_at(FIRST), crate::recipes::CraftingJob::Miner));
        write_placements(&dir, &[]);
        reload(&mut game);
        assert!(!game.near_craft_station(&character_at(FIRST), crate::recipes::CraftingJob::Miner));
        let refused = start(&mut game, FIRST);
        assert!(String::from_utf8_lossy(&refused.replies.concat()).contains("Move closer"));
        assert!(
            game.smelting
                .as_ref()
                .is_none_or(|smelting| !smelting.active())
        );

        let mut completion_game = station_game(&dir);
        write_placements(&dir, &[FIRST]);
        reload(&mut completion_game);
        start(&mut completion_game, FIRST);
        write_placements(&dir, &[]);
        reload(&mut completion_game);
        let error = complete(&mut completion_game, FIRST).unwrap_err();
        assert!(error.to_string().contains("outside a Miner station"));

        let _ = fs::remove_dir_all(dir);
    }
}
