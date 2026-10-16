//! (R) Launcher key-pair authentication, not recovered Free Realms protocol.
//! See notes/Research/Multi Login Design.md, 2026-10-15 key-pair decision.
//!
//! Threat boundary: handles, nonces, public keys, signatures and Session bytes
//! are hostile. AccountStore and the stable per-server ID are operator-owned.
//! The caller serializes this state, supplies a monotonic Instant, and applies
//! per-IP/account rate limits before calling it. No sockets or private keys live
//! here. Do not log challenges, signatures, tokens or the signed transcript.

use crate::account_store::AccountStore;
use ed25519_dalek::{Signature, VerifyingKey};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fmt,
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;

// (R) Fixed local budgets and lifetimes; not original server limits.
pub const MAX_CHALLENGES: usize = 256;
pub const MAX_TOKENS: usize = 256;
pub const AUTH_LIFETIME: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthError {
    InvalidCredentials,
    Capacity,
    RandomUnavailable,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidCredentials => "invalid credentials",
            Self::Capacity => "authentication capacity reached",
            Self::RandomUnavailable => "secure randomness unavailable",
        })
    }
}

impl std::error::Error for AuthError {}

/// (R) Exact case-sensitive ASCII handle, 3..=32 bytes, hashed for the store.
/// This is an account handle, never a character name or native Fingerprint.
pub fn login_handle_hash(handle: &str) -> Result<[u8; 32], AuthError> {
    if !(3..=32).contains(&handle.len())
        || !handle
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
    {
        return Err(AuthError::InvalidCredentials);
    }
    Ok(Sha256::digest(handle.as_bytes()).into())
}

/// (R) Ed25519 public keys must be exactly 32 bytes and not low-order points.
pub fn validate_public_key(bytes: &[u8]) -> Result<[u8; 32], AuthError> {
    let bytes = <[u8; 32]>::try_from(bytes).map_err(|_| AuthError::InvalidCredentials)?;
    let key = VerifyingKey::from_bytes(&bytes).map_err(|_| AuthError::InvalidCredentials)?;
    if key.is_weak() {
        return Err(AuthError::InvalidCredentials);
    }
    Ok(bytes)
}

/// (R) Exact launcher transcript: domain || server ID || nonce || handle.
/// Server ID and nonce are fixed 32-byte values, so concatenation is unambiguous.
/// Use a stable, operator-configured server ID, never one chosen by a login peer.
/// Expiry is bound to the stored nonce; clients cannot extend it.
pub fn signing_message(
    server_id: &[u8; 32],
    nonce: &[u8; 32],
    handle: &str,
) -> Result<Vec<u8>, AuthError> {
    login_handle_hash(handle)?;
    let mut message = b"osfr-login-v1".to_vec();
    message.extend_from_slice(server_id);
    message.extend_from_slice(nonce);
    message.extend_from_slice(handle.as_bytes());
    Ok(message)
}

struct PendingChallenge {
    account_id: Option<u64>,
    handle: String,
    issued: Instant,
}

struct PendingToken {
    digest: [u8; 32],
    account_id: u64,
    login_hash: [u8; 32],
    public_key: [u8; 32],
    issued: Instant,
}

/// In-memory one-use grants. Restart discards pending authentication, not accounts.
/// No Debug/Serialize implementation: this state must not appear in traces.
pub struct LoginAuth {
    server_id: [u8; 32],
    challenges: BTreeMap<[u8; 32], PendingChallenge>,
    tokens: Vec<PendingToken>,
}

impl LoginAuth {
    pub fn new(server_id: [u8; 32]) -> Self {
        Self {
            server_id,
            challenges: BTreeMap::new(),
            tokens: Vec::new(),
        }
    }

    /// (R) Issue the same challenge shape for unknown/unenrolled handles to
    /// avoid disclosing existence in challenge replies. Only verification grants a token.
    pub fn issue_challenge(
        &mut self,
        store: &AccountStore,
        handle: &str,
        now: Instant,
    ) -> Result<[u8; 32], AuthError> {
        self.expire(now);
        let hash = login_handle_hash(handle)?;
        let id = store.account_id_by_login_hash(&hash);
        if self.challenges.len() >= MAX_CHALLENGES {
            return Err(AuthError::Capacity);
        }
        // (R) Bound even improbable RNG collisions; never replace a live nonce.
        for _ in 0..4 {
            let mut nonce = [0; 32];
            getrandom::getrandom(&mut nonce).map_err(|_| AuthError::RandomUnavailable)?;
            if let std::collections::btree_map::Entry::Vacant(entry) = self.challenges.entry(nonce)
            {
                entry.insert(PendingChallenge {
                    account_id: id,
                    handle: handle.to_owned(),
                    issued: now,
                });
                return Ok(nonce);
            }
        }
        Err(AuthError::RandomUnavailable)
    }

    /// Consume the challenge on the first attempt, including bad signatures.
    /// Only a currently enrolled key may mint a Session token. Token minting is
    /// deliberately private to this path; callers cannot mint for arbitrary IDs.
    pub fn verify(
        &mut self,
        store: &AccountStore,
        nonce: &[u8],
        public_key: &[u8],
        signature: &[u8],
        now: Instant,
    ) -> Result<String, AuthError> {
        self.expire(now);
        let nonce = <[u8; 32]>::try_from(nonce).map_err(|_| AuthError::InvalidCredentials)?;
        let challenge = self
            .challenges
            .remove(&nonce)
            .ok_or(AuthError::InvalidCredentials)?;
        let key_bytes = validate_public_key(public_key)?;
        let account_id = challenge.account_id.ok_or(AuthError::InvalidCredentials)?;
        let login_hash = login_handle_hash(&challenge.handle)?;
        if !binding_is_current(store, account_id, &login_hash, &key_bytes) {
            return Err(AuthError::InvalidCredentials);
        }
        let signature =
            Signature::from_slice(signature).map_err(|_| AuthError::InvalidCredentials)?;
        let key =
            VerifyingKey::from_bytes(&key_bytes).map_err(|_| AuthError::InvalidCredentials)?;
        key.verify_strict(
            &signing_message(&self.server_id, &nonce, &challenge.handle)?,
            &signature,
        )
        .map_err(|_| AuthError::InvalidCredentials)?;
        if self.tokens.len() >= MAX_TOKENS {
            return Err(AuthError::Capacity);
        }
        for _ in 0..4 {
            // (R) 128 bits of OS entropy, encoded as 32 lowercase ASCII bytes
            // for the existing native Session field. Store only its SHA-256.
            let mut random = [0; 16];
            getrandom::getrandom(&mut random).map_err(|_| AuthError::RandomUnavailable)?;
            let mut token = String::with_capacity(32);
            const HEX: &[u8; 16] = b"0123456789abcdef";
            for byte in random {
                token.push(HEX[(byte >> 4) as usize] as char);
                token.push(HEX[(byte & 15) as usize] as char);
            }
            let digest: [u8; 32] = Sha256::digest(token.as_bytes()).into();
            if self
                .tokens
                .iter()
                .any(|pending| bool::from(pending.digest.ct_eq(&digest)))
            {
                continue;
            }
            self.tokens.push(PendingToken {
                digest,
                account_id,
                login_hash,
                public_key: key_bytes,
                issued: now,
            });
            return Ok(token);
        }
        Err(AuthError::RandomUnavailable)
    }

    /// Atomically consume a Session token and return its server-owned account ID.
    /// Check the *current* store so key removal also disables pending tokens.
    pub fn redeem(
        &mut self,
        store: &AccountStore,
        token: &[u8],
        now: Instant,
    ) -> Result<u64, AuthError> {
        self.expire(now);
        if token.len() != 32
            || !token
                .iter()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b))
        {
            return Err(AuthError::InvalidCredentials);
        }
        let digest: [u8; 32] = Sha256::digest(token).into();
        let mut matched = None;
        // ponytail: scan at most 256 records. Constant-time digest comparison
        // and a full scan avoid secret-dependent hash-map or prefix lookups.
        for (index, pending) in self.tokens.iter().enumerate() {
            if bool::from(pending.digest.ct_eq(&digest)) {
                matched = Some(index);
            }
        }
        let pending = self
            .tokens
            .swap_remove(matched.ok_or(AuthError::InvalidCredentials)?);
        if !binding_is_current(
            store,
            pending.account_id,
            &pending.login_hash,
            &pending.public_key,
        ) {
            return Err(AuthError::InvalidCredentials);
        }
        Ok(pending.account_id)
    }

    /// Also safe to call from the owner's timer when no requests arrive.
    pub fn expire(&mut self, now: Instant) {
        let fresh = |issued| {
            now.checked_duration_since(issued)
                .is_some_and(|age| age < AUTH_LIFETIME)
        };
        self.challenges.retain(|_, entry| fresh(entry.issued));
        self.tokens.retain(|entry| fresh(entry.issued));
    }
}

fn binding_is_current(store: &AccountStore, id: u64, hash: &[u8; 32], key: &[u8; 32]) -> bool {
    store
        .account(id)
        .is_some_and(|account| &account.login_name_hash == hash)
        && store.public_keys(id).is_some_and(|keys| keys.contains(key))
}
