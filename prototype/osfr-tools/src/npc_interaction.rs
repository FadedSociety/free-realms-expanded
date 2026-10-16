//! Content-backed NPC interaction state for the restoration server.
//!
//! Native evidence: command 26/9 interaction-list reader `00aa2360` -> `00aa2220` ->
//! `00aa1600`/`00aa04d0`; choice request 26/10 constructor `00a36f80` and sender
//! `00a39090`; NPC request 26/8 constructor `008b6bc0` and sender `0092efe0`.
//! Merchant messages are family 66/subtypes 42-46 in dispatcher `00b48a10`.
//! See `notes/Research/NPC Interaction.md` for recovered fields and limits.

use osfr_tools::{
    coin_shop::{
        CoinShop, CoinShopOutcome, RESULT_FAILURE, RESULT_INVALID_MERCHANT,
        RESULT_SERVICE_UNAVAILABLE,
    },
    content::NpcRole,
    quest_engine::{QuestEngine, QuestOutcome, QuestStatus},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

#[cfg(not(test))]
use crate::world::OPEN_WORLD_ZONE;
#[cfg(test)]
const OPEN_WORLD_ZONE: u32 = 0; // Mirrors world::OPEN_WORLD_ZONE for standalone source tests.

const NPC_FAMILY: u16 = 26;
const INTERACTION_LIST: u16 = 9;
const INTERACTION_SELECT: u16 = 10;
const NPC_INTERACT: u16 = 8;
const MERCHANT_FAMILY: u16 = 66;
const MERCHANT_LIST: u16 = 42;
const MERCHANT_BUY: u16 = 43;
const MERCHANT_HIDE: u16 = 46;
const QUEST_FAMILY: u16 = 49;
const QUEST_OFFER: i32 = 1;
const COIN_STORE_FAMILY: u16 = 165;
const COIN_STORE_BUY: u16 = 4;
const COIN_STORE_SELL: u16 = 5;
const TALK_STRING_ID: u32 = 3300;
// Supplied locale-id-to-text export: locale row 8075 is exactly "Goodbye".
const GOODBYE_STRING_ID: u32 = 8075;
// (R) The native menu's generic Interact label is reused for the Shop choice because its
// localized Shop label was not recovered. Choice behavior is authoritative in the server.
const SHOP_STRING_ID: u32 = 432156;
// (R) The existing server reach is 12 m. Native prompt acquisition has separate client rules.
const INTERACTION_RADIUS: f64 = 12.0;
const MAX_NPC_FILE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_NPC_TOTAL_BYTES: u64 = 32 * 1024 * 1024;
const MAX_NPCS: usize = 20_000;
const MAX_MENU_OPTIONS: usize = 128;
const MAX_NAME_BYTES: usize = 1024;
const MAX_STORES_BYTES: u64 = 16 * 1024 * 1024;
const MAX_STORE_ITEMS: usize = 100_000;
const MAX_VENDOR_FILE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_VENDOR_ITEMS: usize = 100_000;
const MAX_WIRE_PACKET_BYTES: usize = 16 * 1024 * 1024;
const MAX_MERCHANT_STRING_BYTES: usize = 64 * 1024;
const MAX_QUEST_BUNDLE_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NpcAction {
    Talk,
    TurnInQuest { quest_id: u32 },
    Shop,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractionOption {
    pub id: i32,
    pub action: NpcAction,
    pub string_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractionMenu {
    pub npc_id: u64,
    pub options: Vec<InteractionOption>,
    /// Exact family 26/subtype 9 reader layout. (R) npc_id is the content key until runtime GUID
    /// mapping is supplied by the lead.
    pub packet: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantItem {
    pub item_id: i32,
    pub purchase_price_coins: Option<u32>,
    pub sell_price_coins: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantWindow {
    pub npc_id: u64,
    /// Human-readable vendor stock evidence from the region NPC content record.
    pub stock_description: Option<String>,
    /// (R) Region stock has prose names but no item IDs; only explicit mapped IDs are listed.
    pub items: Vec<MerchantItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogueContext {
    pub body_string_id: Option<u32>,
    /// Content-backed quest title IDs the client can resolve; no generic dialogue body was found.
    pub quest_title_string_ids: Vec<u32>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum NpcInteractionError {
    UnknownNpc,
    OutOfRange,
    WrongZone,
    MissingPosition,
    NoInteraction,
    NoOpenShop,
    InvalidOption,
    WrongMerchant,
    InvalidPacket,
    NeedsQuestEngine,
    CoinShop(io::ErrorKind),
}

pub enum NpcSelection {
    Talk(DialogueContext),
    Quest(QuestOutcome),
    Shop(MerchantWindow),
}

#[derive(Deserialize)]
struct NpcRegionFile {
    schema_version: u32,
    records: Vec<NpcRecord>,
}

#[derive(Deserialize)]
struct NpcRecord {
    npc_id: u64,
    name: String,
    #[serde(default)]
    zone_id: Option<u32>,
    #[serde(default)]
    position: Option<[f32; 3]>,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    quest_ids_given: Vec<u32>,
    #[serde(default)]
    quests_given_text: Vec<String>,
    #[serde(default)]
    quests_given: Vec<NpcQuest>,
    #[serde(default)]
    quests_accepted: Vec<NpcQuest>,
    #[serde(default)]
    vendor_stock: Option<String>,
    /// Locale id of the NPC's greeting, shown as the dialog body when nothing else
    /// speaks (e.g. Samantha 24940 = guide hash 1591220599, via lab/reports/locale-id-map.json).
    #[serde(default)]
    greeting_string_id: Option<u32>,
}

#[derive(Deserialize)]
struct NpcQuest {
    #[serde(default)]
    quest_id: Option<u32>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    title_string_id: Option<u32>,
}

#[derive(Deserialize)]
struct StoreFile {
    schema_version: u32,
    items: Vec<StorePrice>,
}

#[derive(Deserialize)]
struct VendorFile {
    schema_version: u32,
    vendors: Vec<VendorRecord>,
}

#[derive(Deserialize)]
struct VendorRecord {
    npc_id: u64,
    #[serde(default)]
    item_ids: Vec<i32>,
}

#[derive(Clone, Deserialize)]
struct StorePrice {
    item_id: i32,
    #[serde(default)]
    purchase_price_coins: Option<u32>,
    #[serde(default)]
    sell_price_coins: Option<u32>,
}

struct ActiveInteraction {
    npc_id: u64,
    options: Vec<InteractionOption>,
    shop_open: bool,
    shop_items: BTreeSet<i32>,
}

pub struct NpcInteraction {
    npcs: BTreeMap<u64, NpcRecord>,
    prices: BTreeMap<i32, StorePrice>,
    vendor_items: BTreeMap<u64, BTreeSet<i32>>,
    active: Option<ActiveInteraction>,
}

impl NpcInteraction {
    pub fn load(npc_dir: &Path, stores_path: &Path) -> io::Result<Self> {
        if !npc_dir.is_dir() {
            return Err(invalid("NPC content path is not a directory"));
        }
        let mut files: Vec<PathBuf> = fs::read_dir(npc_dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "json")
            })
            .collect();
        files.sort();
        if files.is_empty() {
            return Err(invalid("NPC content directory has no JSON files"));
        }

        let mut total_bytes = 0u64;
        let mut npcs = BTreeMap::new();
        for path in files {
            let bytes = read_bounded(&path, MAX_NPC_FILE_BYTES)?;
            total_bytes = total_bytes.saturating_add(bytes.len() as u64);
            if total_bytes > MAX_NPC_TOTAL_BYTES {
                return Err(invalid("NPC content exceeds 32 MiB"));
            }
            let region: NpcRegionFile = serde_json::from_slice(&bytes)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            if region.schema_version != 1 {
                return Err(invalid("unsupported NPC content schema"));
            }
            for npc in region.records {
                validate_npc(&npc)?;
                if npcs.insert(npc.npc_id, npc).is_some() {
                    return Err(invalid("duplicate NPC content ID"));
                }
                if npcs.len() > MAX_NPCS {
                    return Err(invalid("NPC content exceeds 20,000 records"));
                }
            }
        }

        let bytes = read_bounded(stores_path, MAX_STORES_BYTES)?;
        let store: StoreFile = serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if store.schema_version != 1 || store.items.len() > MAX_STORE_ITEMS {
            return Err(invalid("unsupported or oversized store catalogue"));
        }
        let mut prices = BTreeMap::new();
        for price in store.items {
            if price.item_id <= 0
                || price
                    .purchase_price_coins
                    .into_iter()
                    .chain(price.sell_price_coins)
                    .any(|value| value > i32::MAX as u32)
                || prices.insert(price.item_id, price).is_some()
            {
                return Err(invalid("invalid or duplicate store price row"));
            }
        }
        let vendor_path = npc_dir
            .parent()
            .ok_or_else(|| invalid("NPC content directory has no parent"))?
            .join("vendors.json");
        let vendor_bytes = read_bounded(&vendor_path, MAX_VENDOR_FILE_BYTES)?;
        let vendor_file: VendorFile = serde_json::from_slice(&vendor_bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if vendor_file.schema_version != 1 || vendor_file.vendors.len() > MAX_NPCS {
            return Err(invalid("unsupported or oversized vendor stock file"));
        }
        let mut vendor_items = BTreeMap::new();
        let mut total_vendor_items = 0usize;
        for vendor in vendor_file.vendors {
            let npc = npcs
                .get(&vendor.npc_id)
                .ok_or_else(|| invalid("vendor stock references an unknown NPC"))?;
            let item_count = vendor.item_ids.len();
            total_vendor_items = total_vendor_items.saturating_add(item_count);
            // (R) This content role and explicit item list authorize which NPC may sell which
            // priced items. The native client does not provide this server authorization rule.
            if !has_role(npc, NpcRole::Vendor)
                || total_vendor_items > MAX_VENDOR_ITEMS
                || vendor
                    .item_ids
                    .iter()
                    .any(|id| *id <= 0 || !prices.contains_key(id))
            {
                return Err(invalid("invalid vendor stock mapping"));
            }
            let stock: BTreeSet<i32> = vendor.item_ids.into_iter().collect();
            if stock.len() != item_count || vendor_items.insert(vendor.npc_id, stock).is_some() {
                return Err(invalid("duplicate vendor stock mapping"));
            }
        }
        Ok(Self {
            npcs,
            prices,
            vendor_items,
            active: None,
        })
    }

    /// Supply an explicit item-ID mapping for a vendor. (R) Current NPC content records stock
    /// as prose; this mapping must come from a separately reviewed content source.
    pub fn set_vendor_stock_items(
        &mut self,
        npc_id: u64,
        item_ids: &[i32],
    ) -> Result<(), NpcInteractionError> {
        let npc = self
            .npcs
            .get(&npc_id)
            .ok_or(NpcInteractionError::UnknownNpc)?;
        if !has_role(npc, NpcRole::Vendor)
            || item_ids.len() > MAX_STORE_ITEMS
            || item_ids
                .iter()
                .any(|item_id| *item_id <= 0 || !self.prices.contains_key(item_id))
        {
            return Err(NpcInteractionError::InvalidOption);
        }
        let stock: BTreeSet<i32> = item_ids.iter().copied().collect();
        if stock.len() != item_ids.len() {
            return Err(NpcInteractionError::InvalidOption);
        }
        self.vendor_items.insert(npc_id, stock);
        Ok(())
    }

    pub fn location(&self, npc_id: u64) -> Option<(u32, [f32; 3])> {
        let npc = self.npcs.get(&npc_id)?;
        Some((npc.zone_id.unwrap_or(OPEN_WORLD_ZONE), npc.position?))
    }

    pub fn interact(
        &mut self,
        npc_id: u64,
        zone_id: u32,
        position: [f32; 3],
        quests: Option<&QuestEngine>,
    ) -> Result<InteractionMenu, NpcInteractionError> {
        let npc = self
            .npcs
            .get(&npc_id)
            .ok_or(NpcInteractionError::UnknownNpc)?;
        check_range(npc, zone_id, position)?;
        let options = self.build_options(npc, quests)?;
        let packet = interaction_list_packet(npc, &options)?;
        // (R) Keep one active NPC/menu per session until another interaction or close.
        self.active = Some(ActiveInteraction {
            npc_id,
            options: options.clone(),
            shop_open: false,
            shop_items: BTreeSet::new(),
        });
        Ok(InteractionMenu {
            npc_id,
            options,
            packet,
        })
    }

    /// Consume the exact client 26/10 body: u64 NPC GUID followed by i32 choice id.
    pub fn select(
        &mut self,
        packet: &[u8],
        zone_id: u32,
        position: [f32; 3],
        mut quests: Option<&mut QuestEngine>,
    ) -> Result<NpcSelection, NpcInteractionError> {
        if packet.len() != 16 || packet[..4] != [NPC_FAMILY as u8, 0, INTERACTION_SELECT as u8, 0] {
            return Err(NpcInteractionError::InvalidPacket);
        }
        let npc_id = u64::from_le_bytes(packet[4..12].try_into().unwrap());
        let choice_id = i32::from_le_bytes(packet[12..16].try_into().unwrap());
        let npc = self
            .npcs
            .get(&npc_id)
            .ok_or(NpcInteractionError::UnknownNpc)?;
        check_range(npc, zone_id, position)?;
        let active = self
            .active
            .as_mut()
            .ok_or(NpcInteractionError::NoInteraction)?;
        if active.npc_id != npc_id {
            return Err(NpcInteractionError::NoInteraction);
        }
        let option = active
            .options
            .iter()
            .find(|option| option.id == choice_id)
            .ok_or(NpcInteractionError::InvalidOption)?;

        match option.action {
            NpcAction::Talk => Ok(NpcSelection::Talk(self.dialogue_context(npc))),
            NpcAction::Shop => {
                active.shop_open = true;
                active.shop_items = self.vendor_items.get(&npc_id).cloned().unwrap_or_default();
                Ok(NpcSelection::Shop(self.merchant_window(npc)))
            }
            NpcAction::TurnInQuest { quest_id } => {
                let engine = quests
                    .as_deref_mut()
                    .ok_or(NpcInteractionError::NeedsQuestEngine)?;
                Ok(NpcSelection::Quest(engine.begin_turnin(quest_id, npc_id)))
            }
        }
    }

    /// Route only active-NPC Coin Store buys/sells through the existing idempotent validator.
    ///
    /// A rejected sell (165/5) is answered with a client-readable 165/6 failure completion
    /// instead of an error, so the native window is told the sale did not happen (W148). Buys keep
    /// returning the error; their refusals are answered by the 66/43 route.
    pub fn handle_coin_store(
        &mut self,
        shop: &mut CoinShop,
        packet: &[u8],
        zone_id: u32,
        position: [f32; 3],
    ) -> Result<CoinShopOutcome, NpcInteractionError> {
        let is_sell = packet.starts_with(&[COIN_STORE_FAMILY as u8, 0, COIN_STORE_SELL as u8, 0]);
        let result = self
            .check_coin_store(packet, zone_id, position)
            .and_then(|()| {
                shop.handle(packet)
                    .map_err(|error| NpcInteractionError::CoinShop(error.kind()))
            });
        match result {
            Err(error) if is_sell => Ok(shop.refuse_sell(packet, sell_refusal_code(&error))),
            other => other,
        }
    }

    /// Session, range, length and merchant checks shared by Coin Store buys and sells.
    fn check_coin_store(
        &self,
        packet: &[u8],
        zone_id: u32,
        position: [f32; 3],
    ) -> Result<(), NpcInteractionError> {
        let active = self
            .active
            .as_ref()
            .ok_or(NpcInteractionError::NoOpenShop)?;
        if !active.shop_open {
            return Err(NpcInteractionError::NoOpenShop);
        }
        let npc = self
            .npcs
            .get(&active.npc_id)
            .ok_or(NpcInteractionError::UnknownNpc)?;
        check_range(npc, zone_id, position)?;
        let subtype = match packet.get(..4) {
            Some(header) if header[..2] == COIN_STORE_FAMILY.to_le_bytes() => {
                u16::from_le_bytes(header[2..4].try_into().unwrap())
            }
            _ => return Err(NpcInteractionError::InvalidPacket),
        };
        let expected_len = match subtype {
            COIN_STORE_BUY => 28,
            COIN_STORE_SELL => 20,
            _ => return Err(NpcInteractionError::InvalidPacket),
        };
        if packet.len() != expected_len {
            return Err(NpcInteractionError::InvalidPacket);
        }
        let merchant_id = u64::from_le_bytes(packet[4..12].try_into().unwrap());
        // (R) Content NPC keys are used as the Coin Store merchant ID until an original binding
        // between runtime actor GUIDs and merchant IDs is recovered.
        // Recovered (W148): the NpcMerchantV2 window sells through `00c77030`, which sends the
        // session merchant GUID globals `01beea08`/`01beea0c`; no store to them exists in the
        // client's code, so they stay the null GUID. The legacy window's data source also starts
        // at the null pair (ctor `00cfef39`, reader `00cf16c0`). A native sell therefore names no
        // merchant (0). The server's own open-shop session is the authority, so a null merchant
        // means "the shop this player has open" (R); any other value must match that shop.
        let null_sell = subtype == COIN_STORE_SELL && merchant_id == 0;
        if merchant_id != active.npc_id && !null_sell {
            return Err(NpcInteractionError::WrongMerchant);
        }
        if subtype == COIN_STORE_BUY {
            let item_id = i32::from_le_bytes(packet[16..20].try_into().unwrap());
            if !active.shop_items.contains(&item_id) {
                return Err(NpcInteractionError::InvalidOption);
            }
        }
        Ok(())
    }

    /// Parse an in-game 66/43 buy request for an open vendor session.
    ///
    /// (R) Requiring an active shop is this server's session gate. The client field meanings and
    /// transaction rules are unresolved, so this only parses the request; it changes no balance.
    pub fn handle_merchant_request_66_43(
        &self,
        packet: &[u8],
    ) -> Result<MerchantBuyRequestV2, NpcInteractionError> {
        if !self.active.as_ref().is_some_and(|active| active.shop_open) {
            return Err(NpcInteractionError::NoOpenShop);
        }
        decode_merchant_buy_66_43(packet)
    }

    /// (R) Enforce reconstructed stock, active-shop and proximity policy for a 66/43 buy.
    pub fn validate_merchant_item_66_43(
        &self,
        item_id: i32,
        zone_id: u32,
        position: [f32; 3],
    ) -> Result<(), NpcInteractionError> {
        let active = self
            .active
            .as_ref()
            .ok_or(NpcInteractionError::NoOpenShop)?;
        if !active.shop_open {
            return Err(NpcInteractionError::NoOpenShop);
        }
        if !active.shop_items.contains(&item_id) {
            return Err(NpcInteractionError::InvalidOption);
        }
        let npc = self
            .npcs
            .get(&active.npc_id)
            .ok_or(NpcInteractionError::UnknownNpc)?;
        check_range(npc, zone_id, position)
    }

    /// Return the native 66/46 hide message and clear this session's active merchant.
    pub fn close(&mut self, npc_id: u64) -> Result<Vec<u8>, NpcInteractionError> {
        let active = self
            .active
            .as_ref()
            .ok_or(NpcInteractionError::NoInteraction)?;
        if active.npc_id != npc_id {
            return Err(NpcInteractionError::NoInteraction);
        }
        if !active.shop_open {
            return Err(NpcInteractionError::NoOpenShop);
        }
        // (R) Clear server session state when the exact native 66/46 hide packet is sent.
        self.active = None;
        let mut packet = MERCHANT_FAMILY.to_le_bytes().to_vec();
        packet.extend(MERCHANT_HIDE.to_le_bytes());
        Ok(packet)
    }

    /// Clear a client-closed conversation while preserving an open vendor session.
    pub fn close_dialogue(&mut self) {
        if self.active.as_ref().is_some_and(|active| !active.shop_open) {
            self.active = None;
        }
    }

    pub fn quest_title_string_id(&self, npc_id: u64, quest_id: u32) -> Option<u32> {
        let npc = self.npcs.get(&npc_id)?;
        npc.quests_given
            .iter()
            .chain(&npc.quests_accepted)
            .find(|entry| entry.quest_id == Some(quest_id))
            .and_then(|entry| entry.title_string_id)
    }

    /// Candidate for the reconstructed Talk-to-offer policy (R). The client
    /// proves a generic 26/9 option list, not which server choice starts 49/1.
    pub fn offerable_quest_for_talk(&self, npc_id: u64, quests: &QuestEngine) -> Option<u32> {
        let npc = self.npcs.get(&npc_id)?;
        if !has_role(npc, NpcRole::QuestGiver) {
            return None;
        }
        let mut seen = BTreeSet::new();
        let mut ids = npc.quest_ids_given.clone();
        ids.extend(quests.quest_ids_for_giver(npc_id));
        ids.into_iter()
            .find(|quest_id| seen.insert(*quest_id) && quests.can_offer(*quest_id, npc_id))
    }

    /// Reconstructed target association (R): same-zone NPC notes mark a quest title as a target.
    pub fn named_target_quest_for_talk(&self, npc_id: u64, quests: &QuestEngine) -> Option<u32> {
        let target = self.npcs.get(&npc_id)?;
        let zone_id = target.zone_id?;
        for giver in self
            .npcs
            .values()
            .filter(|giver| giver.zone_id == Some(zone_id))
        {
            for binding in &giver.quests_given {
                let Some(quest_id) = binding.quest_id else {
                    continue;
                };
                if !quests
                    .quest_ids_for_giver(giver.npc_id)
                    .any(|bound| bound == quest_id)
                    || !quests
                        .progress(quest_id)
                        .is_some_and(|progress| progress.status == QuestStatus::Active)
                {
                    continue;
                }
                let Some(title) = binding
                    .title
                    .as_deref()
                    .filter(|title| !title.trim().is_empty())
                else {
                    continue;
                };
                let marker = format!("{} (target", title.trim().to_lowercase());
                if target
                    .quests_given_text
                    .iter()
                    .any(|line| line.to_lowercase().contains(&marker))
                {
                    return Some(quest_id);
                }
            }
        }
        None
    }

    fn build_options(
        &self,
        npc: &NpcRecord,
        quests: Option<&QuestEngine>,
    ) -> Result<Vec<InteractionOption>, NpcInteractionError> {
        // (R) New offers are returned after Talk; the native client only proves that
        // 26/9 carries generic actions and that 49/1 opens a separate quest screen.
        let mut options = vec![(NpcAction::Talk, TALK_STRING_ID)];
        if let Some(engine) = quests {
            let mut seen = BTreeSet::new();
            for quest_id in npc
                .quests_accepted
                .iter()
                .filter_map(|entry| entry.quest_id)
            {
                if !seen.insert(quest_id)
                    || engine
                        .progress(quest_id)
                        .is_none_or(|progress| progress.status != QuestStatus::Active)
                {
                    continue;
                }
                options.push((
                    NpcAction::TurnInQuest { quest_id },
                    self.quest_title_string_id(npc.npc_id, quest_id)
                        .unwrap_or(0),
                ));
            }
        }
        if has_role(npc, NpcRole::Vendor) {
            options.push((NpcAction::Shop, SHOP_STRING_ID));
        }
        if options.len() > MAX_MENU_OPTIONS {
            return Err(NpcInteractionError::InvalidOption);
        }
        Ok(options
            .into_iter()
            .enumerate()
            .map(|(index, (action, string_id))| InteractionOption {
                // (R) The native menu proves choice 1 for Talk; subsequent choice IDs are local.
                id: index as i32 + 1,
                action,
                string_id,
            })
            .collect())
    }

    fn dialogue_context(&self, npc: &NpcRecord) -> DialogueContext {
        // (R) Prefer the first UCDT name ID; the native ShowDialog payload is not serialized here.
        let mut title_ids: Vec<u32> = npc
            .quests_given
            .iter()
            .filter_map(|quest| quest.title_string_id)
            .collect();
        title_ids.sort_unstable();
        title_ids.dedup();
        DialogueContext {
            // The native ShowDialog field is a dialogue body, not an NPC-name field.
            // NPCs without a recovered greeting stay empty.
            body_string_id: npc.greeting_string_id,
            quest_title_string_ids: title_ids,
        }
    }

    fn merchant_window(&self, npc: &NpcRecord) -> MerchantWindow {
        // (R) Item IDs require an explicit vendor mapping; never show the global price catalogue.
        let items = self
            .vendor_items
            .get(&npc.npc_id)
            .into_iter()
            .flatten()
            .filter_map(|item_id| {
                let price = self.prices.get(item_id)?;
                Some(MerchantItem {
                    item_id: price.item_id,
                    purchase_price_coins: price.purchase_price_coins,
                    sell_price_coins: price.sell_price_coins,
                })
            })
            .collect();
        MerchantWindow {
            npc_id: npc.npc_id,
            stock_description: npc.vendor_stock.clone(),
            items,
        }
    }
}

fn has_role(npc: &NpcRecord, role: NpcRole) -> bool {
    let name = match role {
        NpcRole::Vendor => "vendor",
        NpcRole::QuestGiver => "quest_giver",
        NpcRole::Ambient => "ambient",
    };
    npc.roles.iter().any(|value| value == name)
}

/// Result code a refused sell reports to the client (R mapping onto the client's result names).
fn sell_refusal_code(error: &NpcInteractionError) -> u32 {
    match error {
        NpcInteractionError::InvalidPacket => RESULT_FAILURE,
        NpcInteractionError::CoinShop(_) => RESULT_SERVICE_UNAVAILABLE,
        _ => RESULT_INVALID_MERCHANT,
    }
}

fn check_range(
    npc: &NpcRecord,
    zone_id: u32,
    position: [f32; 3],
) -> Result<(), NpcInteractionError> {
    let npc_zone = npc.zone_id.unwrap_or(OPEN_WORLD_ZONE);
    let npc_position = npc.position.ok_or(NpcInteractionError::MissingPosition)?;
    if position
        .iter()
        .chain(npc_position.iter())
        .any(|value| !value.is_finite())
    {
        return Err(NpcInteractionError::OutOfRange);
    }
    // (R) Regional `zone_id`s are content package labels inside the one
    // continent (local world zone 0); there the distance check decides reach.
    if zone_id != npc_zone && zone_id != OPEN_WORLD_ZONE {
        return Err(NpcInteractionError::WrongZone);
    }
    let distance_squared: f64 = position
        .iter()
        .zip(npc_position)
        .map(|(player, target)| {
            let delta = f64::from(*player) - f64::from(target);
            delta * delta
        })
        .sum();
    if distance_squared > INTERACTION_RADIUS * INTERACTION_RADIUS {
        return Err(NpcInteractionError::OutOfRange);
    }
    Ok(())
}

fn interaction_list_packet(
    npc: &NpcRecord,
    options: &[InteractionOption],
) -> Result<Vec<u8>, NpcInteractionError> {
    if npc.name.len() > MAX_NAME_BYTES || options.len() > MAX_MENU_OPTIONS {
        return Err(NpcInteractionError::InvalidOption);
    }
    let mut packet = vec![NPC_FAMILY as u8, 0, INTERACTION_LIST as u8, 0];
    // (R) The region's reconstructed NPC key is used until the live actor GUID is supplied.
    packet.extend(npc.npc_id.to_le_bytes());
    packet.push(0);
    packet.extend((options.len() as i32).to_le_bytes());
    for option in options {
        // Native menu record as used by opening_world::menu: seven dwords per choice.
        for value in [option.id, 0, option.string_id as i32, 0, 0, 0, 0] {
            packet.extend(value.to_le_bytes());
        }
    }
    packet.extend((npc.name.len() as u32).to_le_bytes());
    packet.extend(npc.name.as_bytes());
    packet.extend([1, 0]);
    Ok(packet)
}

fn validate_npc(npc: &NpcRecord) -> io::Result<()> {
    if npc.npc_id == 0
        || npc.name.is_empty()
        || npc.name.len() > MAX_NAME_BYTES
        || npc.roles.is_empty()
        || npc
            .position
            .is_some_and(|position| position.iter().any(|value| !value.is_finite()))
    {
        return Err(invalid("invalid NPC content record"));
    }
    Ok(())
}

fn read_bounded(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(invalid("content is not a bounded regular file"));
    }
    fs::read(path)
}

/// The client request parsed by family 26/subtype 8 (`008b6bc0`, sent by `0092efe0`).
/// The native bytes prove an eight-byte target value and one trailing boolean byte; target
/// meaning is still caller-supplied until a capture/client field trace names it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NpcInteractRequest {
    pub target: u64,
    pub trailing_flag: bool,
}

pub fn decode_npc_interact_26_8(packet: &[u8]) -> Result<NpcInteractRequest, NpcInteractionError> {
    if packet.len() != 13 || packet[..4] != [26, 0, NPC_INTERACT as u8, 0] || packet[12] > 1 {
        return Err(NpcInteractionError::InvalidPacket);
    }
    Ok(NpcInteractRequest {
        target: u64::from_le_bytes(packet[4..12].try_into().unwrap()),
        trailing_flag: packet[12] == 1,
    })
}

impl NpcInteractRequest {
    pub fn encode(self) -> Vec<u8> {
        let mut packet = vec![26, 0, NPC_INTERACT as u8, 0];
        packet.extend(self.target.to_le_bytes());
        packet.push(u8::from(self.trailing_flag));
        packet
    }
}

/// Raw-but-typed field order from the ShowDialog reader `00a9ef10`.
/// Field names describe wire order only; no dialogue semantics are inferred here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialoguePacket26_3 {
    pub initial_dwords: [u32; 4],
    pub initial_flag: bool,
    pub initial_float_bits: u32,
    pub choices: Vec<[u32; 5]>,
    pub choice_vectors: [[u32; 4]; 2],
    pub vector_flag: bool,
    pub final_vector: [u32; 4],
    pub float_after_vector_bits: u32,
    pub final_flags: [bool; 3],
    pub final_float_bits: u32,
    pub final_dword: u32,
}

/// Encode client family 26/subtype 3 using the exact field sequence read by `00a9ef10`.
pub fn encode_dialogue_26_3(message: &DialoguePacket26_3) -> Result<Vec<u8>, NpcInteractionError> {
    if message.choices.len() > 5 {
        return Err(NpcInteractionError::InvalidPacket);
    }
    let mut packet = Vec::with_capacity(93 + message.choices.len() * 20);
    packet.extend([26, 0, 3, 0]);
    put_words(&mut packet, &message.initial_dwords);
    packet.push(u8::from(message.initial_flag));
    packet.extend(message.initial_float_bits.to_le_bytes());
    put_words(&mut packet, &[message.choices.len() as u32]);
    for choice in &message.choices {
        put_words(&mut packet, choice);
    }
    for vector in message.choice_vectors {
        put_words(&mut packet, &vector);
    }
    packet.push(u8::from(message.vector_flag));
    put_words(&mut packet, &message.final_vector);
    packet.extend(message.float_after_vector_bits.to_le_bytes());
    for flag in message.final_flags {
        packet.push(u8::from(flag));
    }
    packet.extend(message.final_float_bits.to_le_bytes());
    packet.extend(message.final_dword.to_le_bytes());
    Ok(packet)
}

pub fn decode_dialogue_26_3(packet: &[u8]) -> Result<DialoguePacket26_3, NpcInteractionError> {
    if packet.len() < 93 || packet.len() > MAX_WIRE_PACKET_BYTES || packet[..4] != [26, 0, 3, 0] {
        return Err(NpcInteractionError::InvalidPacket);
    }
    let mut cursor = WireCursor::new(&packet[4..]);
    let initial_dwords = cursor.words::<4>()?;
    let initial_flag = cursor.boolean()?;
    let initial_float_bits = cursor.u32()?;
    let count = cursor.u32()? as usize;
    if count > 5 || packet.len() != 93 + count * 20 {
        return Err(NpcInteractionError::InvalidPacket);
    }
    let mut choices = Vec::with_capacity(count);
    for _ in 0..count {
        choices.push(cursor.words::<5>()?);
    }
    let choice_vectors = [cursor.words::<4>()?, cursor.words::<4>()?];
    let vector_flag = cursor.boolean()?;
    let final_vector = cursor.words::<4>()?;
    let float_after_vector_bits = cursor.u32()?;
    let final_flags = [cursor.boolean()?, cursor.boolean()?, cursor.boolean()?];
    let final_float_bits = cursor.u32()?;
    let final_dword = cursor.u32()?;
    cursor.finish()?;
    Ok(DialoguePacket26_3 {
        initial_dwords,
        initial_flag,
        initial_float_bits,
        choices,
        choice_vectors,
        vector_flag,
        final_vector,
        float_after_vector_bits,
        final_flags,
        final_float_bits,
        final_dword,
    })
}

/// Client selection response: reader `009f5890` stores one i32 (R dialog index).
pub fn decode_dialogue_response_26_6(packet: &[u8]) -> Result<i32, NpcInteractionError> {
    if packet.len() != 8 || packet[..4] != [26, 0, 6, 0] {
        return Err(NpcInteractionError::InvalidPacket);
    }
    Ok(i32::from_le_bytes(packet[4..8].try_into().unwrap()))
}

/// Server EndDialog command: native constructor `009f5760` has an empty body.
pub fn encode_dialogue_close_26_4() -> Vec<u8> {
    vec![26, 0, 4, 0]
}

/// Local conversation row based on the recovered Lua `AddResponseNode` tuple.
/// Native reader `00a9b480` reads five DWORDs; display resolves DWORD 3 as a
/// `Global.Text.<id>` locale key, then passes DWORDs 5 and 4 as background set
/// and image IDs. Lua `DialogTest` uses response background set 18. `300` and
/// `17` remain reconstructed response/opaque fields (R); image ID 0 means no
/// separate image, and locale row 8075 is "Goodbye".
/// (R) Response id for a dialog row that opens a quest offer.
pub const QUEST_CHOICE_RESPONSE: i32 = 301;

/// Dialog row that leads to a quest offer, labelled with the quest title.
/// Same five-dword layout as [`goodbye_dialogue_choice`].
pub fn quest_dialogue_choice(title_string_id: u32) -> [u32; 5] {
    [QUEST_CHOICE_RESPONSE as u32, 17, title_string_id, 0, 18]
}

pub fn goodbye_dialogue_choice() -> [u32; 5] {
    [300, 17, GOODBYE_STRING_ID, 0, 18]
}

/// Client quest accept/decline: native `QuestReply` stores quest id and bool.
/// Wire (captured natively, run 143124): u16 family 49 + u32 subtype 2, u32
/// quest id, u8 accept — 11 bytes, e.g. `31 00 02 00 00 00 d5 c6 2d 00 01`.
pub fn decode_quest_reply_49_2(packet: &[u8]) -> Result<(u32, bool), NpcInteractionError> {
    if packet.len() != 11 || packet[..6] != [49, 0, 2, 0, 0, 0] || packet[10] > 1 {
        return Err(NpcInteractionError::InvalidPacket);
    }
    let quest_id = u32::from_le_bytes(packet[6..10].try_into().unwrap());
    if quest_id == 0 {
        return Err(NpcInteractionError::InvalidPacket);
    }
    Ok((quest_id, packet[10] == 1))
}

/// Client `QuestEndReplyPacket` (`00a864c0`, sender `00a97ef0`) is a native
/// six-byte family-49 header followed by the quest ID: 10 bytes total.
pub fn decode_quest_end_reply_49_14(packet: &[u8]) -> Result<u32, NpcInteractionError> {
    if packet.len() != 10 || packet[..6] != [49, 0, 14, 0, 0, 0] {
        return Err(NpcInteractionError::InvalidPacket);
    }
    let quest_id = u32::from_le_bytes(packet[6..10].try_into().unwrap());
    (quest_id != 0)
        .then_some(quest_id)
        .ok_or(NpcInteractionError::InvalidPacket)
}

/// Fields decoded from quest family 49/subtype 1 (`00c7b7a0`). After the u16
/// family + u32 subtype header the first dword is the quest ID, followed by
/// five ShowQuestStartScreen fields: intro text, name, description, badge
/// image and profile icon (the meaning of the last two is still (R)).
/// Its first boolean is the client `membersOnly` argument. The reward bundle and
/// final fields retain their native order. The first trailing dword is forwarded
/// as Lua's `undeclinable` value; the second dword, float and two final booleans
/// have no resolved meaning.
///
/// Wire layout verified natively (Frida, run 142216): the client's
/// `ReadQuestPacketPrefix` (`00c7acd0`) reads u16 family + u32 subtype, then
/// `ReadFields` (`00c7b7a0`) reads six dwords (quest id + five screen fields), a
/// bool, the reward bundle, 8 bytes, a float and two bools — 114 bytes with an
/// empty bundle — and `00c7bb60` rejects any trailing bytes. An earlier layout
/// with an extra u16 prefix and a sixth screen field was 120 bytes and was
/// silently discarded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestOfferPacket49_1 {
    pub quest_id: u32,
    pub intro_text_string_id: u32,
    /// Wire dword 2: not passed to Lua `ShowStartScreen` (native run 143124
    /// probe); meaning unresolved (R). Our builder sends the title id here.
    pub field_2: u32,
    /// Wire dword 3: Lua `name` (the quest card title). Probe-verified.
    pub name_string_id: u32,
    /// Wire dword 4: Lua `description` (shown under Show Details). Probe-verified.
    pub description_string_id: u32,
    /// Wire dword 5: not passed to Lua badge/icon/level; meaning unresolved (R).
    pub field_5: u32,
    pub members_only: bool,
    /// Serialized ClientRewardBundle body (69-byte empty base, followed by counted entries).
    pub reward_bundle_body: Vec<u8>,
    /// First dword is the client `undeclinable` argument; second remains unresolved.
    pub trailing_dwords: [u32; 2],
    pub trailing_float_bits: u32,
    pub trailing_flags: [bool; 2],
}

pub fn encode_quest_offer_49_1(
    message: &QuestOfferPacket49_1,
) -> Result<Vec<u8>, NpcInteractionError> {
    if !(69..=MAX_QUEST_BUNDLE_BYTES).contains(&message.reward_bundle_body.len()) {
        return Err(NpcInteractionError::InvalidPacket);
    }
    let mut packet = Vec::with_capacity(6 + 4 + 20 + 1 + message.reward_bundle_body.len() + 14);
    packet.extend(QUEST_FAMILY.to_le_bytes());
    packet.extend(QUEST_OFFER.to_le_bytes());
    packet.extend(message.quest_id.to_le_bytes());
    put_words(
        &mut packet,
        &[
            message.intro_text_string_id,
            message.field_2,
            message.name_string_id,
            message.description_string_id,
            message.field_5,
        ],
    );
    packet.push(u8::from(message.members_only));
    packet.extend(&message.reward_bundle_body);
    put_words(&mut packet, &message.trailing_dwords);
    packet.extend(message.trailing_float_bits.to_le_bytes());
    for flag in message.trailing_flags {
        packet.push(u8::from(flag));
    }
    Ok(packet)
}

pub fn decode_quest_offer_49_1(packet: &[u8]) -> Result<QuestOfferPacket49_1, NpcInteractionError> {
    if packet.len() < 114
        || packet.len() > MAX_WIRE_PACKET_BYTES
        || packet[..6] != [49, 0, 1, 0, 0, 0]
    {
        return Err(NpcInteractionError::InvalidPacket);
    }
    let reward_end = packet.len() - 14;
    let reward_start = 6 + 4 + 20 + 1;
    if reward_end - reward_start < 69 || reward_end - reward_start > MAX_QUEST_BUNDLE_BYTES {
        return Err(NpcInteractionError::InvalidPacket);
    }
    let mut cursor = WireCursor::new(&packet[6..]);
    let quest_id = cursor.u32()?;
    let fields = cursor.words::<5>()?;
    let members_only = cursor.boolean()?;
    let reward_bundle_body = packet[reward_start..reward_end].to_vec();
    let mut tail = WireCursor::new(&packet[reward_end..]);
    let trailing_dwords = tail.words::<2>()?;
    let trailing_float_bits = tail.u32()?;
    let trailing_flags = [tail.boolean()?, tail.boolean()?];
    tail.finish()?;
    Ok(QuestOfferPacket49_1 {
        quest_id,
        intro_text_string_id: fields[0],
        field_2: fields[1],
        name_string_id: fields[2],
        description_string_id: fields[3],
        field_5: fields[4],
        members_only,
        reward_bundle_body,
        trailing_dwords,
        trailing_float_bits,
        trailing_flags,
    })
}

/// Opaque merchant row fields in native wire order (`00b29cd0`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantEntryV2 {
    pub first_six_dwords: [u32; 6],
    pub flag_after_six: bool,
    pub final_three_dwords: [u32; 3],
    pub final_flag: bool,
}

/// The nested merchant record: six dwords (`00b33020`) and the counted entries (`00b2e620`).
/// `fields[6]` is not on the wire (see `MerchantListV2`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantGroupV2 {
    pub fields: [u32; 7],
    pub entries: Vec<MerchantEntryV2>,
}

/// 66/42 body as the client reads it (`00b48a10` case 0x2a -> `00b40a60` -> `00b3d5f0`).
///
/// Recovered (W148): the reader re-reads the 4-byte message header (the signed-16 pair at
/// `00b2dd60`), so nothing but the header precedes `initial_dword`; the nested record
/// (`00b3aa30` -> `00b33020`) is exactly six dwords (a scalar, two dword pairs, a scalar) before
/// the entry count read by `00b2e620`. Earlier versions wrote an extra i16 pair and a seventh
/// record dword, which shifts every later field and leaves unread bytes, so the client rejects
/// the whole list. The record's second pair (`fields[3]`, `fields[4]`) is the merchant GUID the
/// client keeps at manager +0x3e8f0 and copies into every 66/43 request (`00b43310`).
///
/// `initial_i16s` and `group.fields[6]` are not on the wire and must be zero; they remain only
/// until `gameplay.rs::merchant_packet` stops filling the old shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantListV2 {
    pub initial_i16s: [i16; 2],
    /// One scalar precedes the single nested merchant record in `00b3d5f0`.
    pub initial_dword: u32,
    pub group: MerchantGroupV2,
}

pub fn encode_merchant_list_66_42(
    message: &MerchantListV2,
) -> Result<Vec<u8>, NpcInteractionError> {
    if message.group.entries.len() > MAX_VENDOR_ITEMS
        || message.initial_i16s != [0; 2]
        || message.group.fields[6] != 0
    {
        return Err(NpcInteractionError::InvalidPacket);
    }
    let mut packet = Vec::new();
    packet.extend([66, 0, MERCHANT_LIST as u8, 0]);
    put_words(&mut packet, &[message.initial_dword]);
    let [f0, f1, f2, f3, f4, f5, _] = message.group.fields;
    put_words(&mut packet, &[f0, f1, f2, f3, f4, f5]);
    put_words(&mut packet, &[message.group.entries.len() as u32]);
    for entry in &message.group.entries {
        put_words(&mut packet, &entry.first_six_dwords);
        packet.push(u8::from(entry.flag_after_six));
        put_words(&mut packet, &entry.final_three_dwords);
        packet.push(u8::from(entry.final_flag));
    }
    if packet.len() > MAX_WIRE_PACKET_BYTES {
        return Err(NpcInteractionError::InvalidPacket);
    }
    Ok(packet)
}

pub fn decode_merchant_list_66_42(packet: &[u8]) -> Result<MerchantListV2, NpcInteractionError> {
    if packet.len() < 36
        || packet.len() > MAX_WIRE_PACKET_BYTES
        || packet[..4] != [66, 0, MERCHANT_LIST as u8, 0]
    {
        return Err(NpcInteractionError::InvalidPacket);
    }
    let mut cursor = WireCursor::new(&packet[4..]);
    let initial_dword = cursor.u32()?;
    let [f0, f1, f2, f3, f4, f5] = cursor.words::<6>()?;
    let entry_count = cursor.u32()? as usize;
    if entry_count > MAX_VENDOR_ITEMS || entry_count > cursor.remaining() / 38 {
        return Err(NpcInteractionError::InvalidPacket);
    }
    let mut entries = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        entries.push(MerchantEntryV2 {
            first_six_dwords: cursor.words::<6>()?,
            flag_after_six: cursor.boolean()?,
            final_three_dwords: cursor.words::<3>()?,
            final_flag: cursor.boolean()?,
        });
    }
    cursor.finish()?;
    Ok(MerchantListV2 {
        initial_i16s: [0; 2],
        initial_dword,
        group: MerchantGroupV2 {
            fields: [f0, f1, f2, f3, f4, f5, 0],
            entries,
        },
    })
}

/// Client 66/43 buy fields in serializer order (`00b3ab20`). `initial_dwords[0]`
/// is the Lua-selected item key, `trailing_dwords[0]` the quantity and
/// `trailing_dwords[2]` the generated request id (`00b43310`); mapping those to
/// our catalogue is reconstructed policy (R). The supplied client price is ignored.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantBuyRequestV2 {
    pub initial_dwords: [u32; 2],
    pub context_dwords: [u32; 3],
    pub context_bytes: Vec<u8>,
    pub trailing_dwords: [u32; 3],
}

pub fn encode_merchant_buy_66_43(
    message: &MerchantBuyRequestV2,
) -> Result<Vec<u8>, NpcInteractionError> {
    if message.context_bytes.len() > MAX_MERCHANT_STRING_BYTES {
        return Err(NpcInteractionError::InvalidPacket);
    }
    let mut packet = Vec::with_capacity(40 + message.context_bytes.len());
    packet.extend([66, 0, MERCHANT_BUY as u8, 0]);
    put_words(&mut packet, &message.initial_dwords);
    put_words(&mut packet, &message.context_dwords);
    put_words(&mut packet, &[message.context_bytes.len() as u32]);
    packet.extend(&message.context_bytes);
    put_words(&mut packet, &message.trailing_dwords);
    Ok(packet)
}

pub fn decode_merchant_buy_66_43(
    packet: &[u8],
) -> Result<MerchantBuyRequestV2, NpcInteractionError> {
    if packet.len() < 40
        || packet.len() > MAX_WIRE_PACKET_BYTES
        || packet[..4] != [66, 0, MERCHANT_BUY as u8, 0]
    {
        return Err(NpcInteractionError::InvalidPacket);
    }
    let mut cursor = WireCursor::new(&packet[4..]);
    let initial_dwords = cursor.words::<2>()?;
    let context_dwords = cursor.words::<3>()?;
    let string_len = cursor.u32()? as usize;
    if string_len > MAX_MERCHANT_STRING_BYTES || cursor.remaining() != string_len + 12 {
        return Err(NpcInteractionError::InvalidPacket);
    }
    let context_bytes = cursor.take(string_len)?.to_vec();
    let trailing_dwords = cursor.words::<3>()?;
    cursor.finish()?;
    Ok(MerchantBuyRequestV2 {
        initial_dwords,
        context_dwords,
        context_bytes,
        trailing_dwords,
    })
}

fn put_words<const N: usize>(out: &mut Vec<u8>, values: &[u32; N]) {
    for value in values {
        out.extend(value.to_le_bytes());
    }
}

struct WireCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> WireCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], NpcInteractionError> {
        let end = self
            .offset
            .checked_add(len)
            .filter(|end| *end <= self.bytes.len())
            .ok_or(NpcInteractionError::InvalidPacket)?;
        let bytes = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(bytes)
    }

    fn u16(&mut self) -> Result<u16, NpcInteractionError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, NpcInteractionError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn words<const N: usize>(&mut self) -> Result<[u32; N], NpcInteractionError> {
        let mut words = [0; N];
        for word in &mut words {
            *word = self.u32()?;
        }
        Ok(words)
    }

    fn boolean(&mut self) -> Result<bool, NpcInteractionError> {
        Ok(self.take(1)?[0] != 0)
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }

    fn finish(self) -> Result<(), NpcInteractionError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(NpcInteractionError::InvalidPacket)
        }
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
