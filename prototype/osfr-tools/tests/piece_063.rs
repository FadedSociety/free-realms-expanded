//! Dungeon party reward acceptance for `src/restoration/piece_063.rs`.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` sha256
//!   `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3` -
//!   `FUN_008a4420(param_1, sub, a, b, c)` stores `param_1[1] = 0x27` (39, the minigame family)
//!   and `param_1[2] = sub`, then installs `BaseMiniGamePacket::vftable`. The reward-flow
//!   constructors call it as `MiniGameRewardStatusPacket` (`FUN_009b9d10`, sub 0x16 = 22),
//!   `MiniGameLootWheelSetItemToLandOnPacket` (`FUN_009b9e20`, sub 0x2d = 45),
//!   `MiniGameLootWheelOnRotationStoppedPacket` (`FUN_009aa0c0`, sub 0x2e = 46) and
//!   `MiniGameUpdateRewardPacket` (`FUN_009b9eb0`, sub 0x44 = 68).
//! * the client minigame dispatcher decodes 39/45 and 39/68 (cases 0x2d and 0x44 call
//!   `FUN_009b9e20`/`FUN_009b9eb0` and then read the body through `FUN_009b9700`/`FUN_009bfd30`),
//!   while 39/46 is the client's own rotation-stopped report.
//! * `lab/references/OpenSourceFreeRealms/ofrserver/Server/LibSOE/Core/PacketCodes.cs` sha256
//!   `f3f1287671de00cd009dd86ceaeec4765be581cd823d1da3306910361ccd0869` and
//!   `lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs`
//!   sha256 `b8e568af2b5e69b64b4c2642b9a788ebe2bcb1d6444ff2811e9d182a436fdb75` name the same
//!   four sub-opcodes on family 39.
//! * Three original retail capture frames reassembled under
//!   `lab/reports/original-minigame-packets/`: 39/7 `demoderby-43469-flow1-frame41374-op39-sub7.bin`
//!   (22 B, sha256 `2f37481af7abca619b5754b35dc2ed02efea21c473f125e3a0098d862ea73309`), 39/7
//!   `racing-24992-flow1-frame42933-op39-sub7.bin` (22 B, sha256
//!   `2a3d4fd4ca2b7e8a3b1ff662c88eee151c5166ebbf7d1bfc624be59a7a95b75c`) and 39/23
//!   `demoderby-23088-flow0-frame5078-op39-sub23.bin` (30 B, sha256
//!   `a6aa79938ed60a9255d0a51df9e20a02d1d2b065f97fecdbd0971f5487bdbbf9`). Their post-opcode
//!   bytes are the recovered header `sub | i32 state | i32 group | i32 game`.
//! * `lab/decompiled/client-ui/score-screen/scripts/frame_1/DoAction.as` (original client UI) sends
//!   `OnLootWheelSpinRequest`, applies the server's landing slice through `StopLootWheelAt` and sends
//!   `OnLootWheelRotationStopped` when the wheel stops - so the server, not the client, decides the
//!   prize and a repeated stop report must not grant twice.
//!
//! GAP this test does not close: the per-message bodies of 39/22, 39/45, 39/46 and 39/68 were not
//! recovered (no original serializer, no captured family-39 reward traffic), and no original dungeon
//! reward table or per-member payout rule was recovered. Only the family header, the reward
//! sub-opcode surface and the reconstructed party-completion policy are asserted.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_063;` in `src/restoration`. Integration step for the lead: register the
//! module and drop the include.
#[path = "../src/restoration/piece_063.rs"]
#[allow(dead_code)]
mod piece_063;

use piece_063::{
    DungeonRewardLedger, MINIGAME_FAMILY, MiniGameRewardMessage, RewardBundle, RewardError,
    RewardFlow, parse_minigame_header,
};

const OWNER: u64 = 0x0000_0040_0000_0001;
const MEMBER: u64 = 0x0000_0060_0000_0002;
const MATE: u64 = 0x0000_0060_0000_0003;
const STRANGER: u64 = 0x0000_0090_0000_dead;
const GAME: i32 = 411;
const INSTANCE: i32 = 90_000_017;

/// The three original capture frames above, verbatim.
const CAPTURE_VECTORS: [(&str, &str); 3] = [
    (
        "demoderby 39/7",
        "0500010f000000 2700 07 41001400 ffffffffffffffff",
    ),
    (
        "racing 39/7",
        "0500010f000000 2700 07 3e000e00 ffffffffffffffff",
    ),
    (
        "demoderby 39/23",
        "05000117000000 2700 17 ffffffffffffffffffffffff 0000000000000000",
    ),
];

fn bytes(hex: &str) -> Vec<u8> {
    let compact: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
    assert_eq!(compact.len() % 2, 0, "hex fixture must be whole bytes");
    (0..compact.len() / 2)
        .map(|i| u8::from_str_radix(&compact[i * 2..i * 2 + 2], 16).unwrap())
        .collect()
}

fn bundle() -> RewardBundle {
    RewardBundle {
        coins: 50,
        items: vec![(14794, 1)],
    }
}

fn opened_ledger() -> DungeonRewardLedger {
    let mut ledger = DungeonRewardLedger::new();
    ledger
        .open(INSTANCE, GAME, &[OWNER, MEMBER, MATE], bundle())
        .expect("a three-member party opens a run");
    ledger
}

#[test]
fn original_capture_frames_decode_with_the_recovered_minigame_header() {
    // sub | i32 state | i32 group | i32 game, with any trailing bytes left opaque.
    let expected = [
        (7u8, 0x0014_0041i32, -1i32, -1i32, 0usize),
        (7, 0x000e_003e, -1, -1, 0),
        (23, -1, -1, -1, 8),
    ];
    for ((label, hex), (sub, state, group, game, tail)) in CAPTURE_VECTORS.iter().zip(expected) {
        let frame = bytes(hex);
        assert_eq!(
            u16::from_le_bytes([frame[7], frame[8]]),
            MINIGAME_FAMILY,
            "{label}"
        );
        assert_eq!(frame[9], sub, "{label}");
        let header =
            parse_minigame_header(&frame[9..]).unwrap_or_else(|error| panic!("{label}: {error:?}"));
        assert_eq!(header.sub, sub, "{label}");
        assert_eq!(header.state_id, state, "{label}");
        assert_eq!(header.group_id, group, "{label}");
        assert_eq!(header.game_id, game, "{label}");
        assert_eq!(header.extra.len(), tail, "{label}");
        assert_eq!(header.extra, frame[22..].to_vec(), "{label}");
    }

    // The header is exactly 13 bytes; anything shorter is truncated, never guessed.
    assert_eq!(
        parse_minigame_header(&bytes("05 00 01 0c000000 2700 07 4100")),
        Err(RewardError::MalformedHeader)
    );
    assert!(parse_minigame_header(&bytes("05 00 01 0f000000 2700 07 41001400 ffffffff")).is_ok());
}

#[test]
fn reward_sub_opcodes_match_the_client_constructors_and_the_pinned_enums() {
    let table = [
        (
            22u8,
            "MiniGameRewardStatusPacket",
            RewardFlow::ServerToClient,
        ),
        (
            45,
            "MiniGameLootWheelSetItemToLandOnPacket",
            RewardFlow::ServerToClient,
        ),
        (
            46,
            "MiniGameLootWheelOnRotationStoppedPacket",
            RewardFlow::ClientToServer,
        ),
        (68, "MiniGameUpdateRewardPacket", RewardFlow::ServerToClient),
    ];

    assert_eq!(MINIGAME_FAMILY, 0x27);
    assert_eq!(MiniGameRewardMessage::ALL.len(), table.len());
    for (sub, name, flow) in table {
        let message = MiniGameRewardMessage::from_sub(sub)
            .unwrap_or_else(|| panic!("sub-opcode {sub} from the client binary must be known"));
        assert_eq!(message.sub_opcode(), sub);
        assert_eq!(message.class_name(), name);
        assert_eq!(message.flow(), flow);
    }
    assert_eq!(MiniGameRewardMessage::from_sub(44), None);
    assert_eq!(MiniGameRewardMessage::from_sub(47), None);
    assert_eq!(MiniGameRewardMessage::from_sub(69), None);
}

#[test]
fn party_finish_grants_completion_and_one_parcel_per_eligible_member() {
    let mut ledger = opened_ledger();

    // The finishing blow is the members' business: any party member can report the win.
    let outcome = ledger.finish(INSTANCE, MEMBER).unwrap();
    assert_eq!(outcome.instance_id, INSTANCE);
    assert_eq!(outcome.game_id, GAME);
    assert_eq!(outcome.eligible, vec![OWNER, MEMBER, MATE]);
    assert_eq!(outcome.completed, vec![OWNER, MEMBER, MATE]);
    assert_eq!(outcome.shared, bundle());
    assert!(ledger.is_finished(INSTANCE));

    // Every eligible member has their own pending parcel; claiming one does not consume another's.
    for player in [OWNER, MEMBER, MATE] {
        assert_eq!(ledger.pending(INSTANCE, player), Ok(true), "{player}");
    }
    assert_eq!(ledger.claim(INSTANCE, MATE).unwrap(), bundle());
    assert_eq!(ledger.pending(INSTANCE, MATE), Ok(false));
    assert_eq!(ledger.pending(INSTANCE, OWNER), Ok(true));
    assert_eq!(ledger.claim(INSTANCE, OWNER).unwrap(), bundle());
    assert_eq!(ledger.claim(INSTANCE, MEMBER).unwrap(), bundle());
}

#[test]
fn duplicate_and_replayed_finish_are_rejected_without_extra_grants() {
    let mut ledger = opened_ledger();
    ledger.finish(INSTANCE, OWNER).unwrap();

    // The same win again, from any member, is a refusal.
    assert_eq!(
        ledger.finish(INSTANCE, OWNER),
        Err(RewardError::AlreadyFinished)
    );
    assert_eq!(
        ledger.finish(INSTANCE, MATE),
        Err(RewardError::AlreadyFinished)
    );

    // A stale/replayed instance id can never be re-opened for a second payout.
    assert_eq!(
        ledger.open(INSTANCE, GAME, &[OWNER, MEMBER], bundle()),
        Err(RewardError::AlreadyFinished)
    );
    assert!(ledger.open(INSTANCE + 1, GAME, &[OWNER], bundle()).is_ok());

    // The loot-wheel stop is a one-shot claim per member.
    assert_eq!(ledger.claim(INSTANCE, OWNER).unwrap(), bundle());
    assert_eq!(
        ledger.claim(INSTANCE, OWNER),
        Err(RewardError::AlreadyClaimed)
    );
    assert_eq!(
        ledger.claim(INSTANCE, OWNER).unwrap_err(),
        RewardError::AlreadyClaimed
    );
}

#[test]
fn ineligible_and_unfinished_requests_are_refused() {
    let mut ledger = opened_ledger();

    // Nothing may be claimed before the run is completed.
    assert_eq!(ledger.claim(INSTANCE, OWNER), Err(RewardError::NotFinished));
    assert_eq!(
        ledger.finish(INSTANCE, STRANGER),
        Err(RewardError::NotEligible)
    );
    assert_eq!(
        ledger.claim(INSTANCE, STRANGER),
        Err(RewardError::NotEligible)
    );
    assert_eq!(
        ledger.pending(INSTANCE, STRANGER),
        Err(RewardError::NotEligible)
    );

    ledger.finish(INSTANCE, OWNER).unwrap();
    assert_eq!(
        ledger.claim(INSTANCE, STRANGER),
        Err(RewardError::NotEligible)
    );
    assert_eq!(
        ledger.pending(INSTANCE + 9, OWNER),
        Err(RewardError::UnknownRun)
    );
    assert_eq!(
        ledger.claim(INSTANCE + 9, OWNER),
        Err(RewardError::UnknownRun)
    );
    assert_eq!(
        ledger.finish(INSTANCE + 9, OWNER),
        Err(RewardError::UnknownRun)
    );
    assert!(!ledger.is_finished(INSTANCE + 9));
}

#[test]
fn malformed_rosters_are_refused_at_open() {
    let mut ledger = DungeonRewardLedger::new();
    assert_eq!(
        ledger.open(INSTANCE, GAME, &[], bundle()),
        Err(RewardError::EmptyRoster)
    );
    assert_eq!(
        ledger.open(INSTANCE, GAME, &[OWNER, OWNER], bundle()),
        Err(RewardError::DuplicateParticipant)
    );
    assert_eq!(
        ledger.open(INSTANCE, GAME, &[OWNER, STRANGER], bundle()),
        Ok(piece_063::Reply::Opened {
            instance_id: INSTANCE,
            game_id: GAME,
        })
    );
    assert_eq!(
        ledger.open(INSTANCE, GAME, &[OWNER], bundle()),
        Err(RewardError::AlreadyOpen)
    );
    assert_eq!(
        ledger.open(0, GAME, &[OWNER], bundle()),
        Err(RewardError::InvalidInstance)
    );
}

#[test]
fn run_state_snapshot_reports_completions_and_claims() {
    let mut ledger = opened_ledger();
    ledger.finish(INSTANCE, OWNER).unwrap();
    ledger.claim(INSTANCE, MEMBER).unwrap();
    let snapshot = ledger.state();

    assert_eq!(snapshot["runs"].as_array().map(|rows| rows.len()), Some(1));
    assert_eq!(snapshot["runs"][0]["instance_id"], INSTANCE);
    assert_eq!(snapshot["runs"][0]["game_id"], GAME);
    assert_eq!(snapshot["runs"][0]["finished"], true);
    assert_eq!(snapshot["runs"][0]["coins"], 50);
    assert_eq!(
        snapshot["runs"][0]["participants"]
            .as_array()
            .map(|rows| rows.len()),
        Some(3)
    );
    assert_eq!(
        snapshot["runs"][0]["claimed"]
            .as_array()
            .map(|rows| rows.len()),
        Some(1)
    );
    assert_eq!(snapshot["runs"][0]["claimed"][0], MEMBER);
    assert_eq!(
        snapshot["finished"].as_array().map(|rows| rows.len()),
        Some(1)
    );
}
