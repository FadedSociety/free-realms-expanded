use ed25519_dalek::{Signer, SigningKey};
use osfr_tools::{
    account::{AccountFlags, SlotPolicy},
    account_store::AccountStore,
    login_auth::{
        AuthError, LoginAuth, MAX_CHALLENGES, MAX_TOKENS, login_handle_hash, signing_message,
    },
};
use std::time::{Duration, Instant};

const SERVER: [u8; 32] = [42; 32];
const HANDLE: &str = "acct_001";

#[test]
fn credential_failure_lockout_recovery_success_reset_and_unknown_capacity() {
    use osfr_tools::login_auth::LoginFailures;
    let now = Instant::now();
    let hash = login_handle_hash(HANDLE).unwrap();
    let mut limits = LoginFailures::default();
    for _ in 0..5 {
        assert!(limits.allow(hash, now));
        limits.failed(hash, now);
    }
    assert!(!limits.allow(hash, now));
    assert!(!limits.allow(hash, now + Duration::from_secs(59)));
    let later = now + Duration::from_secs(60);
    assert!(limits.allow(hash, later));
    for _ in 0..4 {
        limits.failed(hash, later);
    }
    limits.succeeded(&hash);
    assert!(limits.allow(hash, later));
    limits.failed(hash, later);
    assert!(limits.allow(hash, later));
    for n in 1u32..1024 {
        let mut hash = [0; 32];
        hash[..4].copy_from_slice(&n.to_le_bytes());
        assert!(limits.allow(hash, later));
    }
    assert!(!limits.allow([255; 32], later));
    assert!(limits.allow([255; 32], later + Duration::from_secs(600)));
}

fn setup() -> (AccountStore, u64, SigningKey, LoginAuth, Instant) {
    // Synthetic test key only. Production private keys belong to the launcher.
    let key = SigningKey::from_bytes(&[7; 32]);
    let mut store = AccountStore::new();
    let id = store
        .create_account(
            login_handle_hash(HANDLE).unwrap(),
            AccountFlags::default(),
            SlotPolicy { max_characters: 2 },
        )
        .unwrap();
    store
        .add_public_key(id, &key.verifying_key().to_bytes())
        .unwrap();
    (store, id, key, LoginAuth::new(SERVER), Instant::now())
}

fn login(
    auth: &mut LoginAuth,
    store: &AccountStore,
    key: &SigningKey,
    now: Instant,
) -> Result<String, AuthError> {
    let nonce = auth.issue_challenge(store, HANDLE, now)?;
    let signature = key.sign(&signing_message(&SERVER, &nonce, HANDLE)?);
    auth.verify(
        store,
        &nonce,
        &key.verifying_key().to_bytes(),
        &signature.to_bytes(),
        now,
    )
}

#[test]
fn good_signature_returns_a_one_use_ascii_session_bound_to_the_account() {
    let (store, id, key, mut auth, now) = setup();
    let token = login(&mut auth, &store, &key, now).unwrap();
    assert_eq!(token.len(), 32);
    assert!(
        token
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    );
    assert_eq!(auth.redeem(&store, token.as_bytes(), now).unwrap(), id);
    assert_eq!(
        auth.redeem(&store, token.as_bytes(), now),
        Err(AuthError::InvalidCredentials)
    );
}

#[test]
fn wrong_key_and_replayed_challenge_are_refused_even_after_failure() {
    let (store, _, key, mut auth, now) = setup();
    let nonce = auth.issue_challenge(&store, HANDLE, now).unwrap();
    let other = SigningKey::from_bytes(&[8; 32]);
    let message = signing_message(&SERVER, &nonce, HANDLE).unwrap();
    assert_eq!(
        auth.verify(
            &store,
            &nonce,
            &other.verifying_key().to_bytes(),
            &other.sign(&message).to_bytes(),
            now
        ),
        Err(AuthError::InvalidCredentials)
    );
    assert_eq!(
        auth.verify(
            &store,
            &nonce,
            &key.verifying_key().to_bytes(),
            &key.sign(&message).to_bytes(),
            now
        ),
        Err(AuthError::InvalidCredentials)
    );
    let nonce = auth.issue_challenge(&store, HANDLE, now).unwrap();
    let signature = key
        .sign(&signing_message(&SERVER, &nonce, HANDLE).unwrap())
        .to_bytes();
    auth.verify(
        &store,
        &nonce,
        &key.verifying_key().to_bytes(),
        &signature,
        now,
    )
    .unwrap();
    assert_eq!(
        auth.verify(
            &store,
            &nonce,
            &key.verifying_key().to_bytes(),
            &signature,
            now
        ),
        Err(AuthError::InvalidCredentials)
    );
}

#[test]
fn signature_is_bound_to_server_handle_and_nonce() {
    let (store, _, key, mut auth, now) = setup();
    for changed in 0..3 {
        let nonce = auth.issue_challenge(&store, HANDLE, now).unwrap();
        // Independently build the specified transcript, rather than use the helper.
        let mut message = b"osfr-login-v1".to_vec();
        message.extend_from_slice(if changed == 0 { &[43; 32] } else { &SERVER });
        message.extend_from_slice(if changed == 1 { &[0; 32] } else { &nonce });
        message.extend_from_slice(if changed == 2 {
            b"acct_002"
        } else {
            HANDLE.as_bytes()
        });
        assert!(matches!(
            auth.verify(
                &store,
                &nonce,
                &key.verifying_key().to_bytes(),
                &key.sign(&message).to_bytes(),
                now
            ),
            Err(AuthError::InvalidCredentials)
        ));
    }
}

#[test]
fn expiry_is_exactly_sixty_seconds_for_challenges_and_tokens() {
    let (store, _, key, mut auth, now) = setup();
    let nonce = auth.issue_challenge(&store, HANDLE, now).unwrap();
    let signature = key
        .sign(&signing_message(&SERVER, &nonce, HANDLE).unwrap())
        .to_bytes();
    assert_eq!(
        auth.verify(
            &store,
            &nonce,
            &key.verifying_key().to_bytes(),
            &signature,
            now + Duration::from_secs(60)
        ),
        Err(AuthError::InvalidCredentials)
    );
    let token = login(&mut auth, &store, &key, now).unwrap();
    assert_eq!(
        auth.redeem(&store, token.as_bytes(), now + Duration::from_secs(60)),
        Err(AuthError::InvalidCredentials)
    );
    let token = login(&mut auth, &store, &key, now).unwrap();
    assert!(
        auth.redeem(
            &store,
            token.as_bytes(),
            now + Duration::from_millis(59_999)
        )
        .is_ok()
    );
}

#[test]
fn table_caps_refuse_growth_and_expired_entries_free_capacity() {
    let (store, _, key, mut auth, now) = setup();
    for _ in 0..MAX_CHALLENGES {
        auth.issue_challenge(&store, HANDLE, now).unwrap();
    }
    assert_eq!(
        auth.issue_challenge(&store, HANDLE, now),
        Err(AuthError::Capacity)
    );
    let later = now + Duration::from_secs(60);
    assert!(auth.issue_challenge(&store, HANDLE, later).is_ok());

    let mut auth = LoginAuth::new(SERVER);
    for _ in 0..MAX_TOKENS {
        login(&mut auth, &store, &key, now).unwrap();
    }
    assert_eq!(
        login(&mut auth, &store, &key, now),
        Err(AuthError::Capacity)
    );
    assert!(login(&mut auth, &store, &key, later).is_ok());
}

#[test]
fn malformed_lengths_and_malleated_signature_are_refused_without_panicking() {
    let (store, _, key, mut auth, now) = setup();
    for length in [0, 1, 31, 33, 64, 1024] {
        let nonce = auth.issue_challenge(&store, HANDLE, now).unwrap();
        assert_eq!(
            auth.verify(&store, &nonce, &vec![0; length], &[0; 64], now),
            Err(AuthError::InvalidCredentials)
        );
    }
    for length in [0, 1, 32, 63, 65, 1024] {
        let nonce = auth.issue_challenge(&store, HANDLE, now).unwrap();
        assert_eq!(
            auth.verify(
                &store,
                &nonce,
                &key.verifying_key().to_bytes(),
                &vec![0; length],
                now
            ),
            Err(AuthError::InvalidCredentials)
        );
    }
    assert_eq!(
        auth.verify(&store, &[0; 31], &[0; 32], &[0; 64], now),
        Err(AuthError::InvalidCredentials)
    );
    let nonce = auth.issue_challenge(&store, HANDLE, now).unwrap();
    let mut signature = key
        .sign(&signing_message(&SERVER, &nonce, HANDLE).unwrap())
        .to_bytes();
    signature[63] |= 0x80;
    assert_eq!(
        auth.verify(
            &store,
            &nonce,
            &key.verifying_key().to_bytes(),
            &signature,
            now
        ),
        Err(AuthError::InvalidCredentials)
    );
    for token in [Vec::new(), vec![0; 32], vec![b'a'; 31], vec![b'a'; 33]] {
        assert_eq!(
            auth.redeem(&store, &token, now),
            Err(AuthError::InvalidCredentials)
        );
    }
}

#[test]
fn revoked_keys_disable_pending_challenges_and_tokens() {
    let (mut store, _, key, mut auth, now) = setup();
    let token = login(&mut auth, &store, &key, now).unwrap();
    let nonce = auth.issue_challenge(&store, HANDLE, now).unwrap();
    store
        .remove_public_key(1, &key.verifying_key().to_bytes())
        .unwrap();
    assert_eq!(
        auth.redeem(&store, token.as_bytes(), now),
        Err(AuthError::InvalidCredentials)
    );
    let signature = key
        .sign(&signing_message(&SERVER, &nonce, HANDLE).unwrap())
        .to_bytes();
    assert_eq!(
        auth.verify(
            &store,
            &nonce,
            &key.verifying_key().to_bytes(),
            &signature,
            now
        ),
        Err(AuthError::InvalidCredentials)
    );
    assert_eq!(
        login(&mut auth, &store, &key, now),
        Err(AuthError::InvalidCredentials)
    );
}

#[test]
fn handles_are_bounded_and_unknown_accounts_fail_generically() {
    let (store, _, key, mut auth, now) = setup();
    for handle in [
        "",
        "ab",
        "bad handle",
        "../acct",
        "ééé",
        "abcdefghijklmnopqrstuvwxyz0123456",
    ] {
        assert_eq!(
            auth.issue_challenge(&store, handle, now),
            Err(AuthError::InvalidCredentials)
        );
    }
    let nonce = auth.issue_challenge(&store, "acct_999", now).unwrap();
    let signature = key
        .sign(&signing_message(&SERVER, &nonce, "acct_999").unwrap())
        .to_bytes();
    assert_eq!(
        auth.verify(
            &store,
            &nonce,
            &key.verifying_key().to_bytes(),
            &signature,
            now
        ),
        Err(AuthError::InvalidCredentials)
    );
}

#[test]
fn weak_keys_and_bad_signatures_never_create_tokens() {
    let (store, _, key, mut auth, now) = setup();
    let nonce = auth.issue_challenge(&store, HANDLE, now).unwrap();
    // Encoded Edwards identity is a low-order point, not a usable login key.
    let mut identity = [0; 32];
    identity[0] = 1;
    assert_eq!(
        osfr_tools::login_auth::validate_public_key(&identity),
        Err(AuthError::InvalidCredentials)
    );
    assert_eq!(
        auth.verify(&store, &nonce, &identity, &[0; 64], now),
        Err(AuthError::InvalidCredentials)
    );
    let nonce = auth.issue_challenge(&store, HANDLE, now).unwrap();
    assert_eq!(
        auth.verify(
            &store,
            &nonce,
            &key.verifying_key().to_bytes(),
            &[0; 64],
            now
        ),
        Err(AuthError::InvalidCredentials)
    );
    assert_eq!(
        auth.redeem(&store, &[b'0'; 32], now),
        Err(AuthError::InvalidCredentials)
    );
}
