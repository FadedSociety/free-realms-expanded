use super::*;
use crate::{
    character_store::CharacterState,
    game_store::{self, Snapshot},
    gameplay::Gameplay,
    gifts::{self, Parcel, Via},
    mail,
    redeem_codes::{Code, Codes},
};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-reward-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn store(&self) -> PathBuf {
        self.0.join("accounts.json")
    }
    fn character(&self) -> PathBuf {
        self.0.join("character.json")
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn state() -> CharacterState {
    CharacterState {
        guid: 70,
        zone_id: 1,
        position: [0.0; 3],
        rotation: [0.0; 3],
    }
}
fn game() -> Gameplay {
    Gameplay::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("data/release"),
        None,
    )
    .unwrap()
}
fn snapshot(game: &Gameplay) -> Snapshot {
    Snapshot {
        character: state(),
        gameplay: Some(game.state()),
        pets: None,
    }
}
fn parcel() -> Parcel {
    Parcel {
        key: None,
        name: "Synthetic gift".into(),
        items: vec![(70430, 1)],
        coins: 5,
        from: "Synthetic sender".into(),
        note: String::new(),
        via: Via::Staff,
        character: None,
    }
}
fn codes(store: &Path) {
    let mut codes = Codes::default();
    for (name, items) in [("COINS", vec![]), ("ITEMS", vec![(70430, 1)])] {
        codes.codes.insert(
            name.into(),
            Code {
                coins: 7,
                items,
                enabled: true,
                max_uses: 1,
                per_character: 1,
                ..Code::default()
            },
        );
    }
    codes.save(&gifts::codes_path(store)).unwrap();
}
fn redeem(store: &Path, game: &mut Gameplay, name: &str) -> gifts::claim::Reply {
    gifts::claim::intercept(
        Some((store, 7)),
        game,
        &crate::chat::local_request(&format!("realm redeem {name}")),
        &state(),
        1000,
    )
    .unwrap()
}
fn prepare(scratch: &Scratch, game: &mut Gameplay) -> Batch {
    let store = scratch.store();
    let mail_path = mail::mailbox_path(&store, 70);
    mail::deliver(
        &mail_path,
        &mail::Letter {
            from: "Synthetic sender".into(),
            subject: "Reward".into(),
            body: "Claim once".into(),
            attachments: mail::Attachments {
                coins: 3,
                items: vec![],
            },
        },
        &mail::LIMITS,
    )
    .unwrap();
    let id = gifts::deliver(&store, 7, parcel()).unwrap();
    let _ = gifts::take_delivered(&store);
    codes(&store);
    game_store::save(&scratch.character(), &snapshot(game)).unwrap();
    let scope = Scope::begin();
    let reply = mail::chat::intercept(
        &mut mail::chat::Slot::default(),
        Some(&store),
        game,
        &crate::chat::local_request("realm mail claim 1"),
        &state(),
        1000,
    )
    .unwrap();
    assert!(reply.changed);
    let mut packet = vec![108, 0, 3, 0];
    packet.extend(1u32.to_le_bytes());
    packet.extend(1u32.to_le_bytes());
    packet.extend((gifts::CLAIM_TAG | u64::from(id)).to_le_bytes());
    let reply = gifts::claim::intercept(Some((&store, 7)), game, &packet, &state(), 1000).unwrap();
    assert!(reply.changed);
    assert!(redeem(&store, game, "COINS").changed);
    redeem(&store, game, "ITEMS");
    // Earlier bundle children read their own staged state: neither reward can pay twice.
    assert!(!redeem(&store, game, "COINS").changed);
    assert!(mail::mark_claimed(&mail_path, 1).is_err());
    scope.finish()
}

#[test]
fn rejected_save_leaves_mail_gifts_and_code_uses_untouched() {
    let scratch = Scratch::new();
    let mut game = game();
    let batch = prepare(&scratch, &mut game);
    let mut invalid = snapshot(&game);
    invalid.character.position[0] = f32::NAN;
    assert!(batch.commit(&scratch.character(), &invalid).is_err());
    assert!(
        !mail::load(&mail::mailbox_path(&scratch.store(), 70))
            .unwrap()
            .messages[0]
            .claimed
    );
    assert_eq!(
        gifts::load(&gifts::box_path(&scratch.store(), 7))
            .unwrap()
            .pending
            .len(),
        1
    );
    assert!(
        Codes::load(&gifts::codes_path(&scratch.store()))
            .unwrap()
            .codes
            .values()
            .all(|c| c.uses == 0)
    );
    assert!(gifts::take_delivered(&scratch.store()).is_empty());
}

#[test]
fn every_interrupted_write_recovers_rewards_and_consumption_once() {
    // mailbox, gift box, code file, character. Fail before each replacement and after all
    // replacements (journal cleanup); replay must converge without paying a second time.
    for fail_at in 0..=4 {
        let scratch = Scratch::new();
        let mut game = game();
        let mut batch = prepare(&scratch, &mut game);
        let character = scratch.character();
        let expected = snapshot(&game);
        batch.reads.insert(
            character.clone(),
            disk_read(&character, MAX_FILE_BYTES).unwrap(),
        );
        batch.writes.insert(
            character.clone(),
            game_store::snapshot_bytes(&character, &expected).unwrap(),
        );
        assert_eq!(batch.writes.len(), 4);
        let mut writes = 0;
        let result = batch.commit_with(&character, |path, bytes| {
            if writes == fail_at {
                return Err(io::Error::other("injected write failure"));
            }
            writes += 1;
            crate::character_store::replace_bytes_unchecked(path, bytes)?;
            if fail_at == 4 && writes == 4 {
                return Err(io::Error::other("injected crash after final replacement"));
            }
            Ok(())
        });
        assert!(result.is_err());
        // A disconnect's stale snapshot must not overwrite a partially committed reward.
        assert!(game_store::save(&character, &expected).is_err());
        assert_eq!(recover(&scratch.0).unwrap(), 1);
        assert_eq!(recover(&scratch.0).unwrap(), 0);
        assert_eq!(
            game_store::load_or_create(&character, state()).unwrap(),
            expected
        );
        assert!(
            mail::load(&mail::mailbox_path(&scratch.store(), 70))
                .unwrap()
                .messages[0]
                .claimed
        );
        let pending = gifts::load(&gifts::box_path(&scratch.store(), 7))
            .unwrap()
            .pending;
        assert_eq!(pending.iter().filter(|p| p.via == Via::Staff).count(), 0);
        assert_eq!(pending.iter().filter(|p| p.via == Via::Code).count(), 1);
        let mut codes = Codes::load(&gifts::codes_path(&scratch.store())).unwrap();
        for code in ["COINS", "ITEMS"] {
            assert_eq!(codes.codes[code].uses, 1);
            assert!(
                codes
                    .redeem_for(code, 70, 7, false, crate::redeem_codes::today())
                    .is_err()
            );
        }
    }
}

#[test]
fn concurrent_mail_delivery_refuses_stale_batch_without_overwriting_it() {
    let scratch = Scratch::new();
    let mut game = game();
    let batch = prepare(&scratch, &mut game);
    let path = mail::mailbox_path(&scratch.store(), 70);
    mail::deliver(
        &path,
        &mail::Letter {
            from: "Synthetic sender".into(),
            subject: "New".into(),
            body: "Arrived while claim staged".into(),
            ..mail::Letter::default()
        },
        &mail::LIMITS,
    )
    .unwrap();
    assert!(
        batch
            .commit(&scratch.character(), &snapshot(&game))
            .is_err()
    );
    let mailbox = mail::load(&path).unwrap();
    assert_eq!(mailbox.messages.len(), 2);
    assert!(!mailbox.messages[0].claimed);
    assert_eq!(recover(&scratch.0).unwrap(), 0);
}

#[test]
fn failed_scope_and_savepoint_discard_writes_and_notices() {
    let scratch = Scratch::new();
    let store = scratch.store();
    {
        let _scope = Scope::begin();
        gifts::deliver(&store, 7, parcel()).unwrap();
        assert_eq!(
            gifts::load(&gifts::box_path(&store, 7))
                .unwrap()
                .pending
                .len(),
            1
        );
        {
            let _failed = Scope::savepoint();
            gifts::deliver(&store, 8, parcel()).unwrap();
        }
        assert!(
            gifts::load(&gifts::box_path(&store, 8))
                .unwrap()
                .pending
                .is_empty()
        );
    }
    assert!(
        gifts::load(&gifts::box_path(&store, 7))
            .unwrap()
            .pending
            .is_empty()
    );
    assert!(gifts::take_delivered(&store).is_empty());
    // Scope is restored after unwinding too.
    let _ = std::panic::catch_unwind(|| {
        let _scope = Scope::begin();
        panic!("synthetic panic");
    });
    gifts::deliver(&store, 9, parcel()).unwrap();
    assert_eq!(gifts::take_delivered(&store), [9].into());
}

#[test]
fn redeeming_preserves_other_accounts_delivery_markers() {
    let scratch = Scratch::new();
    let store = scratch.store();
    codes(&store);
    gifts::deliver(&store, 8, parcel()).unwrap();
    let mut game = game();
    let scope = Scope::begin();
    redeem(&store, &mut game, "ITEMS");
    let batch = scope.finish();
    batch
        .commit(&scratch.character(), &snapshot(&game))
        .unwrap();
    assert_eq!(gifts::take_delivered(&store), [7, 8].into());
}

#[test]
fn reward_save_preserves_existing_housing_and_pet_documents() {
    let scratch = Scratch::new();
    let game = game();
    let mut before = snapshot(&game);
    before.pets = Some(serde_json::json!({"synthetic": true}));
    game_store::save(&scratch.character(), &before).unwrap();
    game_store::save_housing_state(
        &scratch.character(),
        &serde_json::json!({"synthetic": "house"}),
    )
    .unwrap();
    let scope = Scope::begin();
    gifts::deliver(&scratch.store(), 7, parcel()).unwrap();
    scope
        .finish()
        .commit(&scratch.character(), &snapshot(&game))
        .unwrap();
    assert_eq!(
        game_store::load_or_create(&scratch.character(), state())
            .unwrap()
            .pets,
        before.pets
    );
    assert_eq!(
        game_store::load_housing_state::<serde_json::Value>(&scratch.character()).unwrap(),
        Some(serde_json::json!({"synthetic": "house"}))
    );
}

#[test]
fn character_claim_resend_is_published_even_without_store_writes() {
    let scratch = Scratch::new();
    let game = game();
    let scope = Scope::begin();
    gifts::mark_resend(&scratch.store(), 7);
    let batch = scope.finish();
    assert!(batch.is_empty());
    assert!(gifts::take_resend(&scratch.store()).is_empty());
    batch
        .commit(&scratch.character(), &snapshot(&game))
        .unwrap();
    assert_eq!(gifts::take_resend(&scratch.store()), [7].into());
    assert!(!scratch.character().exists());
}

#[test]
fn limited_stock_and_gift_consumption_recover_with_the_character() {
    let scratch = Scratch::new();
    let mut game = game();
    let mut catalog =
        gifts::Catalog::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")).unwrap();
    catalog.gifts.truncate(1);
    let gift = &mut catalog.gifts[0];
    gift.enabled = true;
    gift.live_event = None;
    gift.stock_per_event = Some(1);
    gift.per_player = Some(1);
    let id = gifts::deliver(
        &scratch.store(),
        7,
        Parcel::of(gift, "Synthetic sender", Via::Offer, 70),
    )
    .unwrap();
    let toggles = gifts::Toggles::default();
    let events = gifts::Events::new();
    let stock = gifts::Stock::default();
    let rules = gifts::Rules {
        catalog: &catalog,
        toggles: &toggles,
        events: &events,
        stock: &stock,
    };
    game_store::save(&scratch.character(), &snapshot(&game)).unwrap();
    let scope = Scope::begin();
    let taken = gifts::take(&scratch.store(), 7, &[id], 70, &rules, 1000).unwrap();
    game.gift_grant(taken[0].coins, &taken[0].items, 1000)
        .unwrap();
    let mut batch = scope.finish();
    batch.reads.insert(
        scratch.character(),
        disk_read(&scratch.character(), MAX_FILE_BYTES).unwrap(),
    );
    batch.writes.insert(
        scratch.character(),
        game_store::snapshot_bytes(&scratch.character(), &snapshot(&game)).unwrap(),
    );
    let mut count = 0;
    assert!(
        batch
            .commit_with(&scratch.character(), |path, bytes| {
                count += 1;
                if count == 2 {
                    return Err(io::Error::other("synthetic stock/box failure"));
                }
                crate::character_store::replace_bytes_unchecked(path, bytes)
            })
            .is_err()
    );
    assert_eq!(recover(&scratch.0).unwrap(), 1);
    assert_eq!(
        game_store::load_or_create(&scratch.character(), state()).unwrap(),
        snapshot(&game)
    );
    assert!(
        gifts::load(&gifts::box_path(&scratch.store(), 7))
            .unwrap()
            .pending
            .is_empty()
    );
    assert!(gifts::take(&scratch.store(), 7, &[id], 70, &rules, 1000).is_err());
    assert!(
        !gifts::Stock::read(&scratch.store())
            .unwrap()
            .taken
            .is_empty()
    );
}
