//! Kart reward registration/payout acceptance for `src/kart_rewards.rs`.
//!
//! ORIGINAL evidence used by these vectors (all outside Git):
//! * `notes/Research/Kart Race Finish Protocol.md` — no captured frame reaches the lap limit, so the
//!   finish sequence and its payout are not observed; `Finished` is client-derived from the
//!   server-sent lap limit.
//! * `notes/Research/Reward Toast and Deferred Bank Findings.md` — the reward family is base opcode
//!   `50` with a **byte** subtype (`FUN_00b88ad0`, reader `0x00b89190`); subtype 1 is the reward
//!   bundle whose reader `0x008e7930` places coin award at inner offset 4, XP/stars at 8 and reward
//!   kind at 16. An empty-entry inner packet is 72 bytes.
//! * `src/mystery_chest.rs` — the compiled precedent for that 50/1 layout (item kind 16).
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` — `FUN_008d86e0` (38/14) and `FUN_008d8910`
//!   (38/18) profile XP readers reused through `Profiles::experience_packets`.
//!
//! Everything numeric in the starter registration is RECONSTRUCTED: no captured payout exists.
//!
//! Exercises the registered production modules.
use osfr_tools::{inventory, kart_racing, kart_rewards, profiles};

use inventory::{Inventory, InventoryState};
use kart_racing::{
    CheckpointOutcome, KartRacingError, Phase, RaceResultProposal, RaceSession, RewardSchedule,
    SessionSnapshot, TrackLayout,
};
use kart_rewards::{
    KART_DRIVER_JOB_ID, KartRewardRegistry, REWARD_FAMILY, REWARD_SUBTYPE_BUNDLE, RewardError,
    RewardRegistration, claim_race_reward,
};
use profiles::{ProfileState, Profiles};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

/// Captured Wildwood speedway id (`kart_racing::SPEEDWAYS`).
const WILDWOOD: u16 = 31;
/// An id no registration covers.
const UNREGISTERED: u16 = 77;
/// The other class in the fixture: a job whose XP must never receive the race reward.
const OTHER_JOB: i32 = 43;
const SOURCE_GUID: u64 = 0x0f54_ca17_bf5c_0001;
const PLAYER_GUID: u64 = 0x1122_3344_5566_7788;

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn signed(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

/// Reconstructed inventory + profile content for one character: a Kart Driver (48) and a second
/// class (43), no items, no equipment. Synthetic but real content: both loaders validate every
/// proof, span and bound exactly as they do for a private package.
struct Content {
    directory: PathBuf,
}

impl Content {
    fn new(active: i32) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-kart-rewards-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self { directory }.with_profiles(active, 0)
    }

    /// Rewrite `profiles.json` with an active job and a starting XP balance for that job.
    fn with_profiles(self, active: i32, xp: u32) -> Self {
        let mut template = vec![5, 0, 1, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0];
        template.extend(0i32.to_le_bytes());
        let mut proofs = Vec::new();
        let mut unlocked = BTreeSet::new();
        for id in [KART_DRIVER_JOB_ID, OTHER_JOB] {
            unlocked.insert(id);
            let raw: Vec<_> = [1i32, 0, 0, 1, 3, 0, 3, 0]
                .into_iter()
                .flat_map(i32::to_le_bytes)
                .collect();
            proofs.push(json!({
                "id": id,
                "offset": template.len(),
                "length": raw.len(),
                "sha256": digest(&raw),
                "serialized_hex": hex(&raw),
                "rank_offset": 0,
                "equipment_offset": 12,
                "equipment_length": 16,
            }));
            template.extend(raw);
        }
        let active_offset = template.len();
        template.extend(active.to_le_bytes());
        let mut experience = serde_json::Map::new();
        if xp > 0 {
            experience.insert(active.to_string(), json!(xp));
        }
        let profiles = json!({
            "schema_version": 1,
            "active_offset": active_offset,
            "active_sha256": digest(&active.to_le_bytes()),
            "profiles": proofs,
            "default_state": {
                "version": 1,
                "active": active,
                "unlocked": unlocked.iter().copied().collect::<Vec<_>>(),
                "experience": experience,
            },
            "starter_items": {},
        });
        fs::write(
            self.directory.join("profiles.json"),
            serde_json::to_vec(&profiles).unwrap(),
        )
        .unwrap();
        self.write_inventory(0)
    }

    /// Rewrite `inventory.json` with a coin balance and the one supported profile.
    fn write_inventory(self, coins: u32) -> Self {
        let mut packet = vec![5, 0, 1, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0];
        let coins_offset = packet.len();
        packet.extend((coins as i32).to_le_bytes());
        let profiles: serde_json::Value =
            serde_json::from_slice(&fs::read(self.directory.join("profiles.json")).unwrap())
                .unwrap();
        let profile_ids: Vec<i32> = profiles["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["id"].as_i64().unwrap() as i32)
            .collect();
        // One zero-count equipment region per supported profile, in ascending wire order.
        let mut equipment: Vec<(i32, usize)> = Vec::new();
        for id in &profile_ids {
            equipment.push((*id, packet.len()));
            packet.extend(0i32.to_le_bytes());
        }
        let items_offset = packet.len();
        packet.extend(0i32.to_le_bytes());
        let length = packet.len();
        packet[3..7].copy_from_slice(&((length - 7) as u32).to_le_bytes());
        packet[9..13].copy_from_slice(&((length - 13) as u32).to_le_bytes());
        let span = |kind: &str, offset: usize, length: usize| json!({"kind":kind,"offset":offset,"length":length,"profile_id":0,"sha256":digest(&packet[offset..offset+length])});
        let mut spans: Vec<serde_json::Value> = vec![span("coins", coins_offset, 4)];
        for (id, offset) in &equipment {
            spans.push(json!({"kind":"equipment","offset":offset,"length":4,"profile_id":id,"sha256":digest(&packet[*offset..*offset+4])}));
        }
        spans.push(span("items", items_offset, 4));
        let content = json!({
            "schema_version": 1,
            "player_guid": PLAYER_GUID,
            "default_state": {"coins": coins, "items": [], "equipment": [], "next_instance_id": null},
            "profiles": profile_ids
                .iter()
                .map(|id| json!({"id":id,"allowed_classes":[1],"slots":[]}))
                .collect::<Vec<_>>(),
            "spans": spans,
            "definitions": [],
        });
        fs::write(
            self.directory.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        self
    }

    fn profiles(&self) -> Profiles {
        Profiles::load(&self.directory, None).unwrap()
    }

    fn inventory(&self) -> Inventory {
        Inventory::load(&self.directory, None).unwrap()
    }

    fn reload(&self, profiles: &ProfileState, inventory: &InventoryState) -> (Profiles, Inventory) {
        (
            Profiles::load(&self.directory, Some(profiles.clone())).unwrap(),
            Inventory::load(&self.directory, Some(inventory.clone())).unwrap(),
        )
    }
}

impl Drop for Content {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn schedule(coins: u32, xp: u32) -> RewardSchedule {
    RewardSchedule { coins, xp }
}

/// Drive one 1-lap, two-checkpoint session to `Finished`.
fn finished_session(speedway: u16, reward: Option<RewardSchedule>) -> RaceSession {
    let layout = TrackLayout::new(speedway, 1, vec![7, 8], reward).unwrap();
    let mut session = RaceSession::open(0x9152_ca91, layout, 0).unwrap();
    session.begin(0).unwrap();
    session.start(0).unwrap();
    assert!(matches!(
        session.on_checkpoint(10, 7),
        CheckpointOutcome::CheckpointAccepted { index: 0, .. }
    ));
    assert!(matches!(
        session.on_checkpoint(20, 8),
        CheckpointOutcome::LapCompleted { lap: 1, .. }
    ));
    assert_eq!(session.phase(), Phase::Finished);
    session
}

fn racing_session(speedway: u16, reward: RewardSchedule) -> RaceSession {
    let layout = TrackLayout::new(speedway, 1, vec![7, 8], Some(reward)).unwrap();
    let mut session = RaceSession::open(0x9152_ca91, layout, 0).unwrap();
    session.begin(0).unwrap();
    session.start(0).unwrap();
    assert_eq!(session.phase(), Phase::Racing);
    session
}

fn wildwood_price(registry: &KartRewardRegistry) -> RewardSchedule {
    registry.registration(WILDWOOD).unwrap().schedule()
}

fn final_result(session: &RaceSession) -> RaceResultProposal {
    session.finish(1_000).unwrap()
}

#[test]
fn a_finished_registered_race_pays_once_with_real_coins_xp_and_the_recovered_toast() {
    let content = Content::new(KART_DRIVER_JOB_ID);
    let mut profiles = content.profiles();
    let mut inventory = content.inventory();
    let registry = KartRewardRegistry::starter_defaults();
    let price = wildwood_price(&registry);
    let mut session = finished_session(WILDWOOD, Some(price));

    let payout = claim_race_reward(
        &registry,
        &mut session,
        &mut inventory,
        &mut profiles,
        SOURCE_GUID,
        PLAYER_GUID,
    )
    .unwrap();

    assert!(payout.changed);
    assert_eq!(payout.coins, price.coins);
    assert_eq!(payout.xp, price.xp);
    assert_eq!(inventory.state().coins, price.coins);
    assert_eq!(
        profiles
            .state()
            .experience
            .get(&KART_DRIVER_JOB_ID)
            .copied(),
        Some(price.xp)
    );
    assert!(session.snapshot().reward_claimed);
    let result = final_result(&session);
    assert_eq!(result.laps_completed, 1);
    assert_eq!(result.placement, None);
}

#[test]
fn a_second_claim_never_pays_again() {
    let content = Content::new(KART_DRIVER_JOB_ID);
    let mut profiles = content.profiles();
    let mut inventory = content.inventory();
    let registry = KartRewardRegistry::starter_defaults();
    let price = wildwood_price(&registry);
    let mut session = finished_session(WILDWOOD, Some(price));
    claim_race_reward(
        &registry,
        &mut session,
        &mut inventory,
        &mut profiles,
        SOURCE_GUID,
        PLAYER_GUID,
    )
    .unwrap();
    let coins = inventory.state().coins;
    let xp = profiles.state().clone();

    assert_eq!(
        claim_race_reward(
            &registry,
            &mut session,
            &mut inventory,
            &mut profiles,
            SOURCE_GUID,
            PLAYER_GUID,
        )
        .unwrap_err(),
        RewardError::Racing(KartRacingError::RewardAlreadyClaimed)
    );
    assert_eq!(inventory.state().coins, coins);
    assert_eq!(profiles.state(), &xp);
}

#[test]
fn a_forged_or_unfinished_race_aborts_without_a_payout() {
    let content = Content::new(KART_DRIVER_JOB_ID);
    let mut profiles = content.profiles();
    let mut inventory = content.inventory();
    let registry = KartRewardRegistry::starter_defaults();

    // A race still on track cannot pay, even though its layout carries a price.
    let mut racing = racing_session(WILDWOOD, schedule(1_000, 1_000));
    assert_eq!(
        claim_race_reward(
            &registry,
            &mut racing,
            &mut inventory,
            &mut profiles,
            SOURCE_GUID,
            PLAYER_GUID,
        )
        .unwrap_err(),
        RewardError::Racing(KartRacingError::NotFinished)
    );

    // A finished race whose own price is not the registered price is refused.
    let mut forged = finished_session(WILDWOOD, Some(schedule(999_999, 999_999)));
    assert_eq!(
        claim_race_reward(
            &registry,
            &mut forged,
            &mut inventory,
            &mut profiles,
            SOURCE_GUID,
            PLAYER_GUID,
        )
        .unwrap_err(),
        RewardError::ScheduleNotRegistered {
            speedway: WILDWOOD,
            coins: 999_999,
            xp: 999_999,
        }
    );
    assert_eq!(inventory.state().coins, 0);
    assert!(profiles.state().experience.is_empty());
    assert!(!forged.snapshot().reward_claimed);
    assert!(!racing.snapshot().reward_claimed);
}

#[test]
fn an_unregistered_speedway_is_refused() {
    let content = Content::new(KART_DRIVER_JOB_ID);
    let mut profiles = content.profiles();
    let mut inventory = content.inventory();
    let registry = KartRewardRegistry::starter_defaults();
    let mut session = finished_session(UNREGISTERED, Some(schedule(10, 10)));
    assert_eq!(
        claim_race_reward(
            &registry,
            &mut session,
            &mut inventory,
            &mut profiles,
            SOURCE_GUID,
            PLAYER_GUID,
        )
        .unwrap_err(),
        RewardError::Unregistered {
            speedway: UNREGISTERED
        }
    );
    assert_eq!(inventory.state().coins, 0);
    assert!(!session.snapshot().reward_claimed);
}

#[test]
fn experience_can_never_land_on_another_job() {
    let content = Content::new(OTHER_JOB);
    let mut profiles = content.profiles();
    let mut inventory = content.inventory();
    let registry = KartRewardRegistry::starter_defaults();
    let price = wildwood_price(&registry);
    let mut session = finished_session(WILDWOOD, Some(price));
    assert_eq!(
        claim_race_reward(
            &registry,
            &mut session,
            &mut inventory,
            &mut profiles,
            SOURCE_GUID,
            PLAYER_GUID,
        )
        .unwrap_err(),
        RewardError::MissingKartDriverJob { active: OTHER_JOB }
    );
    assert_eq!(inventory.state().coins, 0);
    assert!(profiles.state().experience.is_empty());
    assert!(!session.snapshot().reward_claimed);
}

#[test]
fn unpriced_and_zero_schedules_are_refused() {
    let content = Content::new(KART_DRIVER_JOB_ID);
    let mut profiles = content.profiles();
    let mut inventory = content.inventory();
    let registry = KartRewardRegistry::starter_defaults();
    let mut unpriced = finished_session(WILDWOOD, None);
    assert_eq!(
        claim_race_reward(
            &registry,
            &mut unpriced,
            &mut inventory,
            &mut profiles,
            SOURCE_GUID,
            PLAYER_GUID,
        )
        .unwrap_err(),
        RewardError::Unpriced { speedway: WILDWOOD }
    );
    assert!(!unpriced.snapshot().reward_claimed);

    // A registration that prices nothing can never be built.
    assert_eq!(
        KartRewardRegistry::new(vec![RewardRegistration::reconstructed(WILDWOOD, 0, 0)])
            .unwrap_err(),
        RewardError::EmptySchedule { speedway: WILDWOOD }
    );
    assert_eq!(
        KartRewardRegistry::new(vec![
            RewardRegistration::reconstructed(WILDWOOD, 1, 0),
            RewardRegistration::reconstructed(WILDWOOD, 2, 0),
        ])
        .unwrap_err(),
        RewardError::DuplicateRegistration { speedway: WILDWOOD }
    );
}

#[test]
fn coin_overflow_rolls_back_the_whole_payout() {
    let content = Content::new(KART_DRIVER_JOB_ID).write_inventory(i32::MAX as u32);
    let mut profiles = content.profiles();
    let mut inventory = content.inventory();
    let registry = KartRewardRegistry::starter_defaults();
    let price = wildwood_price(&registry);
    let mut session = finished_session(WILDWOOD, Some(price));
    assert_eq!(
        claim_race_reward(
            &registry,
            &mut session,
            &mut inventory,
            &mut profiles,
            SOURCE_GUID,
            PLAYER_GUID,
        )
        .unwrap_err(),
        RewardError::CoinOverflow {
            balance: i32::MAX as u32,
            award: price.coins,
        }
    );
    assert_eq!(inventory.state().coins, i32::MAX as u32);
    assert!(profiles.state().experience.is_empty());
    assert!(!session.snapshot().reward_claimed);
}

#[test]
fn experience_overflow_rolls_back_the_staged_coin_grant() {
    let content = Content::new(KART_DRIVER_JOB_ID).with_profiles(KART_DRIVER_JOB_ID, u32::MAX);
    let mut profiles = content.profiles();
    let mut inventory = content.inventory();
    let registry = KartRewardRegistry::starter_defaults();
    let price = wildwood_price(&registry);
    let mut session = finished_session(WILDWOOD, Some(price));
    assert_eq!(
        claim_race_reward(
            &registry,
            &mut session,
            &mut inventory,
            &mut profiles,
            SOURCE_GUID,
            PLAYER_GUID,
        )
        .unwrap_err(),
        RewardError::ExperienceOverflow {
            balance: u32::MAX,
            award: price.xp,
        }
    );
    assert_eq!(
        inventory.state().coins,
        0,
        "the staged coin grant must roll back"
    );
    assert_eq!(
        profiles.state().experience.get(&KART_DRIVER_JOB_ID),
        Some(&u32::MAX)
    );
    assert!(!session.snapshot().reward_claimed);
}

#[test]
fn a_serialized_restart_does_not_pay_twice() {
    let content = Content::new(KART_DRIVER_JOB_ID);
    let mut profiles = content.profiles();
    let mut inventory = content.inventory();
    let registry = KartRewardRegistry::starter_defaults();
    let price = wildwood_price(&registry);
    let mut session = finished_session(WILDWOOD, Some(price));
    claim_race_reward(
        &registry,
        &mut session,
        &mut inventory,
        &mut profiles,
        SOURCE_GUID,
        PLAYER_GUID,
    )
    .unwrap();

    let snapshot: SessionSnapshot =
        serde_json::from_slice(&serde_json::to_vec(&session.snapshot()).unwrap()).unwrap();
    let mut restored_session = RaceSession::restore(snapshot).unwrap();
    let (mut restored_profiles, mut restored_inventory) =
        content.reload(profiles.state(), inventory.state());
    let coins = restored_inventory.state().coins;
    let experience = restored_profiles.state().clone();

    assert_eq!(
        claim_race_reward(
            &registry,
            &mut restored_session,
            &mut restored_inventory,
            &mut restored_profiles,
            SOURCE_GUID,
            PLAYER_GUID,
        )
        .unwrap_err(),
        RewardError::Racing(KartRacingError::RewardAlreadyClaimed)
    );
    assert_eq!(restored_inventory.state().coins, coins);
    assert_eq!(restored_profiles.state(), &experience);
    assert!(restored_session.snapshot().reward_claimed);
}

#[test]
fn the_starter_registration_is_reconstructed_complete_and_validated() {
    let registry = KartRewardRegistry::starter_defaults();
    assert_eq!(registry.rows().count(), 5);
    for row in registry.rows() {
        assert!(row.coins > 0 || row.xp > 0);
        assert!(row.coins <= i32::MAX as u32);
        assert!(row.xp <= i32::MAX as u32);
        assert!(
            row.provenance.contains("RECONSTRUCTED"),
            "starter rows must state that the numbers are reconstructed"
        );
    }
    assert_eq!(registry.registration(UNREGISTERED), None);

    // A table that cannot describe a payable race is rejected whole.
    assert_eq!(
        KartRewardRegistry::new(vec![RewardRegistration::reconstructed(0, 5, 5)]).unwrap_err(),
        RewardError::ZeroSpeedway
    );
    assert_eq!(
        KartRewardRegistry::new(vec![RewardRegistration::reconstructed(
            WILDWOOD,
            u32::MAX,
            1
        )])
        .unwrap_err(),
        RewardError::AwardTooLarge {
            coins: u32::MAX,
            xp: 1
        }
    );
}

#[test]
fn the_payout_emits_the_recovered_38_and_50_replies() {
    let content = Content::new(KART_DRIVER_JOB_ID);
    let mut profiles = content.profiles();
    let mut inventory = content.inventory();
    let registry = KartRewardRegistry::starter_defaults();
    let price = wildwood_price(&registry);
    let mut session = finished_session(WILDWOOD, Some(price));
    let payout = claim_race_reward(
        &registry,
        &mut session,
        &mut inventory,
        &mut profiles,
        SOURCE_GUID,
        PLAYER_GUID,
    )
    .unwrap();

    // Coins (38/19), then the profile XP update (38/14, with 38/18 ahead of it on a rank-up), then
    // the recovered 50/1 reward toast. The two families frame their subtype differently - 38 uses
    // `u16` and reward family 50 uses a single byte - so the assertions read each one as it is.
    assert_eq!(payout.replies.len(), 3);
    assert_eq!(&payout.replies[0][..4], &[38, 0, 19, 0]);
    assert_eq!(&payout.replies[1][..4], &[38, 0, 14, 0]);
    assert_eq!(
        &payout.replies[2][..3],
        &[50, 0, REWARD_SUBTYPE_BUNDLE],
        "the toast is 50/1, not 50/1/0"
    );
    assert_eq!(signed(&payout.replies[0], 4), price.coins as i32);

    let toast = payout.replies.last().unwrap();
    assert_eq!(toast.len(), 72);
    assert_eq!(&toast[..3], &[50, 0, REWARD_SUBTYPE_BUNDLE]);
    assert_eq!(toast[3], 1, "success flag");
    assert_eq!(signed(toast, 4), price.coins as i32);
    assert_eq!(signed(toast, 8), price.xp as i32);
    assert_eq!(signed(toast, 12), 0);
    assert_eq!(
        signed(toast, 16),
        0,
        "currency/XP toast carries no item-notification kind"
    );
    assert_eq!(u16::from_le_bytes([toast[0], toast[1]]), REWARD_FAMILY);
    assert_eq!(
        f32::from_le_bytes(toast[28..32].try_into().unwrap()),
        1.0,
        "multiplier"
    );
    assert_eq!(
        &toast[40..56],
        &[SOURCE_GUID.to_le_bytes(), PLAYER_GUID.to_le_bytes()].concat()
    );
    assert_eq!(signed(toast, 64), 0, "no bundle entries");
}
