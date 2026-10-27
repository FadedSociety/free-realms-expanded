//! Account membership and local game-rule policy.
//!
//! Native facts and reconstructed choices are deliberately separated. The
//! only membership expiry and most global toggles represented here are local
//! policy (R); the client messages inspected so far do not carry an expiry or
//! a general PvP/trade/chat rule set.

const TUNNEL_PREFIX: [u8; 3] = [5, 0, 1];

/// The membership state trusted by account-owned server code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MembershipState {
    pub is_member: bool,
    /// (R) Local expiry time. No expiry timestamp was found in the client
    /// membership packets inspected for this module.
    pub expires_at_unix_seconds: Option<u64>,
}

impl MembershipState {
    pub const fn new(is_member: bool, expires_at_unix_seconds: Option<u64>) -> Self {
        Self {
            is_member,
            expires_at_unix_seconds,
        }
    }

    pub const fn is_active_at(self, now_unix_seconds: u64) -> bool {
        self.is_member
            && match self.expires_at_unix_seconds {
                Some(expiry) => now_unix_seconds < expiry,
                None => true,
            }
    }
}

impl Default for MembershipState {
    fn default() -> Self {
        // (R) Revival policy: accounts are members unless configuration
        // supplies an explicit account membership value.
        Self::new(true, None)
    }
}

/// Membership-tagged client content/actions. The client visibly exposes
/// these flags; only item/slot equip has a mapped local denial path. Other
/// server enforcement remains reconstructed policy (R).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemberFeature {
    ZoneEntry,
    MinigameLaunch,
    QuestAcceptance,
    ItemEquip,
    HousingLotEntry,
    HousingFixtureUse,
    MountUse,
    PetSummon,
    StorePurchase,
    MembersOnlyChatChannel,
    ExpiredJobTrial,
}

/// Account action checked by [`GameRuleSet::may`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameAction {
    MemberOnly(MemberFeature),
    /// The supplied count is the number of quests already active.
    AcceptQuest {
        active_count: u32,
    },
    Pvp,
    Trade,
    CardTrade {
        rule_key: u32,
    },
    Chat,
    OpenChat,
    /// The supplied cooldown comes from account/gameplay state, not a value
    /// decoded by this policy module.
    StartMountTrial {
        cooldown_remaining_seconds: u32,
    },
    StartPetTrial {
        cooldown_remaining_seconds: u32,
    },
}

/// Optional per-account quest limits from self-data or explicit local config.
/// The original client selects between two values, but their historical
/// numbers have not been established.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct QuestLimits {
    pub nonmember: Option<u32>,
    pub member: Option<u32>,
}

/// The account bytes that the client uses before allowing typed chat.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AccountChatFlags {
    pub underage: bool,
    pub open_chat_enabled: bool,
}

/// Explicit policy switches. These are configurable revival policy (R), not
/// recovered global client flags. Defaulting them on is a permissive local
/// choice so an unconfigured restoration remains playable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RulePolicy {
    pub pvp_enabled: bool,
    pub trading_enabled: bool,
    pub chat_enabled: bool,
    pub open_chat_enabled: bool,
    pub mount_trials_enabled: bool,
    pub pet_trials_enabled: bool,
    /// (R) Local environment exception matching the client's China branch.
    pub china_environment: bool,
    pub quest_limits: QuestLimits,
    /// Server-provided card-trade flags keyed by opaque dword ids.
    pub card_trade_flags: Vec<TradeAllowedFlag>,
}

impl Default for RulePolicy {
    fn default() -> Self {
        Self {
            pvp_enabled: true,
            trading_enabled: true,
            chat_enabled: true,
            open_chat_enabled: true,
            mount_trials_enabled: true,
            pet_trials_enabled: true,
            china_environment: false,
            quest_limits: QuestLimits::default(),
            card_trade_flags: Vec::new(),
        }
    }
}

/// One policy entry point for membership and server-configured gates.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct GameRuleSet {
    pub membership: MembershipState,
    pub policy: RulePolicy,
    pub account_chat_flags: AccountChatFlags,
}

impl GameRuleSet {
    pub fn new(membership: MembershipState, policy: RulePolicy) -> Self {
        Self {
            membership,
            policy,
            account_chat_flags: AccountChatFlags::default(),
        }
    }

    pub const fn with_account_chat_flags(mut self, flags: AccountChatFlags) -> Self {
        self.account_chat_flags = flags;
        self
    }

    /// Canonical callback values for `ClientAccountManager::OnAccountInfo`.
    /// Expiry is applied here as local policy; the callback's native tuple is
    /// not itself a recovered packet encoding.
    pub fn account_info_callback_bytes(
        &self,
        maximum_characters: u32,
        is_admin: bool,
        now_unix_seconds: u64,
    ) -> [u8; 12] {
        encode_account_info_values(AccountInfoValues {
            is_member: self.membership.is_active_at(now_unix_seconds),
            maximum_characters,
            is_admin,
        })
    }

    pub fn may(&self, action: GameAction, now_unix_seconds: u64) -> bool {
        let member = self.membership.is_active_at(now_unix_seconds);
        match action {
            GameAction::MemberOnly(_) => member,
            GameAction::AcceptQuest { active_count } => {
                let limit = if member {
                    self.policy.quest_limits.member
                } else {
                    self.policy.quest_limits.nonmember
                };
                match limit {
                    Some(limit) => active_count < limit,
                    None => true,
                }
            }
            GameAction::Pvp => self.policy.pvp_enabled,
            GameAction::Trade => self.policy.trading_enabled,
            GameAction::CardTrade { rule_key } => {
                self.policy.trading_enabled
                    && self
                        .policy
                        .card_trade_flags
                        .iter()
                        .find(|entry| entry.key == rule_key)
                        .is_none_or(|entry| entry.allowed)
            }
            GameAction::Chat => self.policy.chat_enabled,
            GameAction::OpenChat => {
                self.policy.chat_enabled
                    && self.policy.open_chat_enabled
                    && (!self.account_chat_flags.underage
                        || self.account_chat_flags.open_chat_enabled
                        || self.policy.china_environment)
            }
            GameAction::StartMountTrial {
                cooldown_remaining_seconds,
            } => self.policy.mount_trials_enabled && cooldown_remaining_seconds == 0,
            GameAction::StartPetTrial {
                cooldown_remaining_seconds,
            } => self.policy.pet_trials_enabled && cooldown_remaining_seconds == 0,
        }
    }
}

/// One decoded entry from the server's `TradeCommand_SetTradeAllowed` flag
/// tree. Key semantics are opaque; absent keys fall through to local R policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TradeAllowedFlag {
    pub key: u32,
    pub allowed: bool,
}

/// Integer representation selected by the client's serialization stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegerEncoding {
    /// Client compact integer encoding used when its network-order switch is 0.
    Compact,
    /// Four-byte big-endian integer encoding when its switch is nonzero.
    NetworkByteOrder,
}

/// Encode only the count-prefixed flag-tree fields of command 348. The
/// command class/version, AccountID/session base fields, and service routing
/// wrapper belong to the separate trade-command framing layer.
pub fn encode_trade_allowed_flag_entries(
    flags: &[TradeAllowedFlag],
    encoding: IntegerEncoding,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    append_serial_integer(
        &mut bytes,
        u32::try_from(flags.len()).expect("too many trade rule entries"),
        encoding,
    );
    for flag in flags {
        append_serial_integer(&mut bytes, flag.key, encoding);
        append_serial_integer(&mut bytes, u32::from(flag.allowed), encoding);
    }
    bytes
}

pub(crate) fn append_serial_integer(out: &mut Vec<u8>, value: u32, encoding: IntegerEncoding) {
    if encoding == IntegerEncoding::NetworkByteOrder {
        out.extend_from_slice(&value.to_be_bytes());
        return;
    }

    let complement = !value;
    let negative = complement < value;
    let magnitude = if negative { complement } else { value };
    let extra_bytes = ((32 - magnitude.leading_zeros()).saturating_sub(4)).div_ceil(8) as usize;
    let mut header = (magnitude & 0x0f) as u8;
    header |= ((extra_bytes as u8 * 2) | u8::from(negative)) << 4;
    out.push(header);
    for byte_index in 0..extra_bytes {
        out.push((magnitude >> (4 + byte_index * 8)) as u8);
    }
}

/// Canonical little-endian value tuple consumed by the account-info callback
/// at 0x00719330. This is not itself a proven network packet serializer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountInfoValues {
    pub is_member: bool,
    pub maximum_characters: u32,
    pub is_admin: bool,
}

pub fn encode_account_info_values(values: AccountInfoValues) -> [u8; 12] {
    let mut bytes = [0; 12];
    bytes[0..4].copy_from_slice(&u32::from(values.is_member).to_le_bytes());
    bytes[4..8].copy_from_slice(&values.maximum_characters.to_le_bytes());
    bytes[8..12].copy_from_slice(&u32::from(values.is_admin).to_le_bytes());
    bytes
}

/// Encode the native opcode-100 membership update. Dispatch case 0x64 (0x00948b5c)
/// calls the strict wrapper 0x008f6e30, whose reader 0x008d5ee0 reads the i16
/// opcode and then one i32; 0x00a12470 stores that i32 at game-state +0x464.
pub fn encode_membership_activation(is_member: bool) -> Vec<u8> {
    let mut inner = Vec::with_capacity(6);
    inner.extend_from_slice(&100u16.to_le_bytes());
    inner.extend_from_slice(&u32::from(is_member).to_le_bytes());
    tunnel(&inner)
}

/// Encode PacketMembershipSubscriptionInfo (opcode 189). Client reader 008d6600 reads the i16
/// opcode, then two bool bytes; 008f7300 refuses unread trailing bytes. Dispatch 00949a01
/// stores them at ClientPcData +0x468 and +0x469. `Ui.IsPlayerLifetimeMember` (00c199e0)
/// reads +0x468; nothing in the client reads +0x469, which OSFR names IsReferee (R).
pub fn encode_membership_subscription_info(lifetime_member: bool, referee: bool) -> Vec<u8> {
    tunnel(&[189, 0, u8::from(lifetime_member), u8::from(referee)])
}

fn tunnel(inner: &[u8]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(TUNNEL_PREFIX.len() + 4 + inner.len());
    packet.extend_from_slice(&TUNNEL_PREFIX);
    packet.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    packet.extend_from_slice(inner);
    packet
}
