use osfr_tools::game_rules::{
    AccountChatFlags, GameAction, GameRuleSet, IntegerEncoding, MemberFeature, MembershipState,
    QuestLimits, RulePolicy, TradeAllowedFlag, encode_membership_activation,
    encode_membership_subscription_info, encode_trade_allowed_flag_entries,
};
use osfr_tools::startup_selfdata::{build_self, parse_self};
use std::{fs, path::Path};

const ALL_MEMBER_FEATURES: [MemberFeature; 11] = [
    MemberFeature::ZoneEntry,
    MemberFeature::MinigameLaunch,
    MemberFeature::QuestAcceptance,
    MemberFeature::ItemEquip,
    MemberFeature::HousingLotEntry,
    MemberFeature::HousingFixtureUse,
    MemberFeature::MountUse,
    MemberFeature::PetSummon,
    MemberFeature::StorePurchase,
    MemberFeature::MembersOnlyChatChannel,
    MemberFeature::ExpiredJobTrial,
];

#[test]
fn default_policy_is_member_and_permissive_until_configured() {
    let rules = GameRuleSet::default();
    assert!(rules.membership.is_active_at(0));
    for feature in ALL_MEMBER_FEATURES {
        assert!(rules.may(GameAction::MemberOnly(feature), 0));
    }
    assert!(rules.may(GameAction::Pvp, 0));
    assert!(rules.may(GameAction::Trade, 0));
    assert!(rules.may(GameAction::Chat, 0));
    assert!(rules.may(GameAction::OpenChat, 0));
    assert!(rules.may(
        GameAction::StartMountTrial {
            cooldown_remaining_seconds: 0
        },
        0
    ));
    assert!(rules.may(
        GameAction::StartPetTrial {
            cooldown_remaining_seconds: 0
        },
        0
    ));
}

#[test]
fn nonmember_and_expired_member_are_denied_membership_gates() {
    let nonmember = GameRuleSet::new(MembershipState::new(false, None), RulePolicy::default());
    let expired = GameRuleSet::new(MembershipState::new(true, Some(100)), RulePolicy::default());
    for feature in ALL_MEMBER_FEATURES {
        assert!(!nonmember.may(GameAction::MemberOnly(feature), 0));
        assert!(expired.may(GameAction::MemberOnly(feature), 99));
        assert!(!expired.may(GameAction::MemberOnly(feature), 100));
    }
}

#[test]
fn configured_global_toggles_trials_and_quest_limits_are_applied() {
    let policy = RulePolicy {
        pvp_enabled: false,
        trading_enabled: false,
        chat_enabled: false,
        open_chat_enabled: true,
        mount_trials_enabled: true,
        pet_trials_enabled: false,
        china_environment: false,
        quest_limits: QuestLimits {
            nonmember: Some(2),
            member: Some(5),
        },
        card_trade_flags: vec![
            TradeAllowedFlag {
                key: 10,
                allowed: false,
            },
            TradeAllowedFlag {
                key: 11,
                allowed: true,
            },
        ],
    };
    let member_rules = GameRuleSet::new(MembershipState::new(true, None), policy.clone());
    let nonmember_rules = GameRuleSet::new(MembershipState::new(false, None), policy);

    assert!(!member_rules.may(GameAction::Pvp, 0));
    assert!(!member_rules.may(GameAction::Trade, 0));
    assert!(!member_rules.may(GameAction::CardTrade { rule_key: 11 }, 0));
    assert!(!member_rules.may(GameAction::Chat, 0));
    assert!(!member_rules.may(GameAction::OpenChat, 0));
    assert!(member_rules.may(
        GameAction::StartMountTrial {
            cooldown_remaining_seconds: 0
        },
        0
    ));
    assert!(!member_rules.may(
        GameAction::StartMountTrial {
            cooldown_remaining_seconds: 1
        },
        0
    ));
    assert!(!member_rules.may(
        GameAction::StartPetTrial {
            cooldown_remaining_seconds: 0
        },
        0
    ));
    assert!(member_rules.may(GameAction::AcceptQuest { active_count: 4 }, 0));
    assert!(!member_rules.may(GameAction::AcceptQuest { active_count: 5 }, 0));
    assert!(nonmember_rules.may(GameAction::AcceptQuest { active_count: 1 }, 0));
    assert!(!nonmember_rules.may(GameAction::AcceptQuest { active_count: 2 }, 0));

    let card_trade_rules = GameRuleSet::new(
        MembershipState::new(false, None),
        RulePolicy {
            card_trade_flags: vec![
                TradeAllowedFlag {
                    key: 10,
                    allowed: false,
                },
                TradeAllowedFlag {
                    key: 11,
                    allowed: true,
                },
            ],
            ..RulePolicy::default()
        },
    );
    assert!(!card_trade_rules.may(GameAction::CardTrade { rule_key: 10 }, 0));
    assert!(card_trade_rules.may(GameAction::CardTrade { rule_key: 11 }, 0));
    assert!(card_trade_rules.may(GameAction::CardTrade { rule_key: 12 }, 0));
}

#[test]
fn typed_chat_uses_account_flags_and_the_configured_environment_exception() {
    let underage = GameRuleSet::default().with_account_chat_flags(AccountChatFlags {
        underage: true,
        open_chat_enabled: false,
    });
    assert!(underage.may(GameAction::Chat, 0));
    assert!(!underage.may(GameAction::OpenChat, 0));

    let account_open = GameRuleSet::default().with_account_chat_flags(AccountChatFlags {
        underage: true,
        open_chat_enabled: true,
    });
    assert!(account_open.may(GameAction::OpenChat, 0));

    let china = GameRuleSet::new(
        MembershipState::default(),
        RulePolicy {
            china_environment: true,
            ..RulePolicy::default()
        },
    )
    .with_account_chat_flags(AccountChatFlags {
        underage: true,
        open_chat_enabled: false,
    });
    assert!(china.may(GameAction::OpenChat, 0));
}

#[test]
fn account_info_values_encode_member_and_nonmember_records() {
    assert_eq!(
        GameRuleSet::default().account_info_callback_bytes(3, false, 0),
        [1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        GameRuleSet::new(MembershipState::new(false, None), RulePolicy::default())
            .account_info_callback_bytes(1, true, 0),
        [0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0]
    );
    let expired = GameRuleSet::new(MembershipState::new(true, Some(10)), RulePolicy::default());
    assert_eq!(
        expired.account_info_callback_bytes(1, false, 10),
        [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]
    );
}

/// Dispatch case 0x64 (0x00948b5c) passes allowTrailing = 0 to 0x008f6e30, whose reader
/// 0x008d5ee0 reads the i16 opcode then one i32, so the inner payload is exactly
/// [100, 0, member, 0, 0, 0].
#[test]
fn membership_wire_messages_match_native_reader_layouts() {
    assert_eq!(
        encode_membership_activation(true),
        [5, 0, 1, 6, 0, 0, 0, 100, 0, 1, 0, 0, 0]
    );
    assert_eq!(
        encode_membership_activation(false),
        [5, 0, 1, 6, 0, 0, 0, 100, 0, 0, 0, 0, 0]
    );
}

/// L048: client reader 008d6600 reads the i16 opcode, a bool for +0x468 and a bool for +0x469,
/// and 008f7300 refuses any unread byte, so the inner payload is exactly [189, 0, l, r].
#[test]
fn membership_subscription_info_matches_the_native_reader() {
    for (l, r) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        assert_eq!(
            encode_membership_subscription_info(l == 1, r == 1),
            [5, 0, 1, 4, 0, 0, 0, 189, 0, l, r]
        );
    }
}

#[test]
fn card_trade_flag_tree_encodes_both_native_integer_modes() {
    let flags = [
        TradeAllowedFlag {
            key: 7,
            allowed: true,
        },
        TradeAllowedFlag {
            key: 42,
            allowed: false,
        },
    ];
    assert_eq!(
        encode_trade_allowed_flag_entries(&flags, IntegerEncoding::Compact),
        [2, 7, 1, 0x2a, 2, 0]
    );
    assert_eq!(
        encode_trade_allowed_flag_entries(&flags, IntegerEncoding::NetworkByteOrder),
        [0, 0, 0, 2, 0, 0, 0, 7, 0, 0, 0, 1, 0, 0, 0, 42, 0, 0, 0, 0]
    );
}

#[test]
#[ignore = "requires private startup reference via OSFR_GATEWAY_REFERENCE"]
fn captured_self_account_flags_round_trip_without_disclosing_identity() {
    let reference = std::env::var("OSFR_GATEWAY_REFERENCE")
        .expect("private reference path required for startup byte oracle");
    let packet = fs::read(Path::new(&reference).join("0007-login.bin")).unwrap();
    let decoded = parse_self(&packet);
    assert!(!decoded.account.underage);
    assert!(!decoded.account.open_chat);
    assert_eq!(decoded.account.membership_status, 1);
    assert!(!decoded.account.show_member_nag);
    assert!(!decoded.account.grandfathered);
    assert_eq!(build_self(&decoded), packet);
}
