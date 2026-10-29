---
type: research
task: W76
updated: 2026-10-15
status: Researched
plan: Build Plan 1.2
---

# Multi Login Design

Design only. No code, saves, native processes, manifests, launcher files or release binaries were changed. Claude owns integration and native acceptance. This note specifies ten simultaneous accounts; it does not claim ten-player gameplay or historical authentication fidelity.

## Decision update 2026-10-15: key pairs, not passwords (The owner)

**This supersedes the password, Argon2 and credential-broker sections below.** The owner's point: anyone can host a private server from our code. A malicious host can change their server to log passwords as players type them, and people reuse passwords. Hashing on the server does not help, because the server sees the password before hashing it. So **no password ever goes to a server**:

- **Enrolment.** The launcher generates an **Ed25519 key pair per server** (unlinkable across servers). The private key never leaves the player's device; it is stored in the player's profile with OS-only permissions, and an optional local passphrase can come later. The server stores only the **public key**, bound to the account. Sign-up needs an invite code or admin approval (R).
- **Login.** The broker sends a random single-use 32-byte challenge, bound to the server identity and an expiry. The launcher signs `"osfr-login-v1" || server_id || challenge || account handle`. The server verifies the signature against the stored public key and issues the same 60-second one-use Session token as designed below. A signature is fresh on every login, so a captured exchange cannot be replayed.
- **What a malicious host learns:** a public key that is useless anywhere else. It never learns a reusable secret.
- **Recovery.** The launcher exports an encrypted backup file and recovery code. Failing that, an admin re-binds the account to a new public key after verifying the player out of band.
- **Crates:** `ed25519-dalek` (signatures), `getrandom` (challenges, tokens, keys), `subtle` (constant-time compares). **Argon2 is dropped.** There is no password database and no slow hashing, so the two-worker hash pool and password lockouts are unnecessary; per-IP and per-account rate limits stay. The owner approved these crates on 2026-10-15.
- Everything else in this design stands: the peer session table, the one-use gateway grant, separate account/character id maps, the bounds, and Session/Fingerprint (the native client still only sees a short-lived token).

## Decision (original W76 text)

Use one login event loop, a peer-keyed session table, one authoritative account store, Argon2id credentials checked before native launch, and short-lived single-use grants. Keep the native packet layouts. Split account and character ownership into two registry maps; preserve all existing numeric IDs and save paths.

**The native LoginUdp_6 request does not supply username/password fields.** Its two strings are Session and Fingerprint. The current Rust variable `login_name` is a local interpretation of Session. Do not put a password in Fingerprint or implement the unrelated password-command family as though this client already sends it. Evidence is below.

Every proposed limit, authentication rule, grant format, sign-up operation and migration rule below is **reconstructed server policy (R)**. Future code must carry `(R)` comments at those rules. Native observations and current Rust behavior are identified separately.

## Verified baseline and provenance

Paths in this section are relative to `project/`; line numbers describe the checkout inspected for W76. Re-check them before implementation because other builders share this checkout.

| Claim | Inspected evidence |
|---|---|
| Login has one transport session; a different valid handshake replaces it, including one from another peer. Later traffic is accepted only for that session's peer. | `prototype/osfr-tools/src/bin/osfr-login.rs:459-483`, `:496-507`, `:521`. |
| Session already owns independent codec, reliable receiver/sender, LoginService, RX/TX RC4, activity time and counters. Retransmissions use the queued ciphertext. | `prototype/osfr-tools/src/bin/osfr-login.rs:166-215`, `:309-351`; `prototype/osfr-tools/src/login.rs:30-37`. |
| Login already expires its single session after 120 seconds idle or sender failure. Activity is refreshed after control decoding, before individual control validation. | `prototype/osfr-tools/src/bin/osfr-login.rs:244-259`, `:533-552`. |
| The opcode-1 reader bounds two strings at 1,024 and 4,096 bytes, discards the second, requires end-of-message, and accepts only the first string equal to `local_session`. There is no password verification in this route. | `prototype/osfr-tools/src/login.rs:293-329`. `--session` supplies `local_session`: `prototype/osfr-tools/src/bin/osfr-login.rs:72`, `:112-122`. |
| Account lookup hashes the accepted Session bytes with SHA-256. Missing accounts can be created automatically. A LoginService caches its own AccountStore; create/delete clone that cached copy and save it. Merely making ten services would risk lost updates. | `prototype/osfr-tools/src/login.rs:205-258`, `:349-362`, `:463-468`. Lost update is a source-derived concurrency risk, not an observed native failure. |
| The store has separate account and character counters, both initially 1. Each advances independently with checked addition. Its two persistent maps are already distinct. | `prototype/osfr-tools/src/account_store.rs:76-85`, `:114-118`, `:140-153`, `:239-256`. |
| PlayerRegistry combines account IDs and character GUIDs in `guid_owners`. Join checks/inserts both numbers; leave removes both. Thus account 2 can incorrectly conflict with another account's character 2. | `prototype/osfr-tools/src/players.rs:89-103`, `:119-148`, `:153-164`. Counterexample is inferred from this code. |
| Account-store character selection performs ownership checks, but account 1 / character 1 receives the fixed configured ticket. Other pairs receive a deterministic truncated HMAC. | `prototype/osfr-tools/src/login.rs:384-431`; ownership helper `prototype/osfr-tools/src/account.rs:416-443`, `select_and_bind_session`. The bypass is exactly `login.rs:414-418`. |
| The existing binding ticket authenticates a domain string, account ID, character GUID and login-name hash. It has no expiry or nonce. Verification checks length then XOR-folds bytes. This is an attempt to avoid early exit, not evidence of a compiler-guaranteed constant-time comparison. | `prototype/osfr-tools/src/login.rs:552-604`. |
| Gateway derives the account from the selected character's stored owner, accepts the same 1/1 fixed-ticket bypass, otherwise verifies HMAC, and uses `characters/<guid>.json`. | `prototype/osfr-tools/src/bin/osfr-gateway.rs:742-773`, `:798-812`; path helper `:580-586`. |
| Selection can load/copy/save a character before world admission. A concurrent builder added a teardown guard that skips saving when another session already owns that character. This narrows the duplicate risk but does not establish ownership before selection-time writes or cover an account selecting another character. The residual risk is source-derived, not a native reproduction. | Selection/load/write: `prototype/osfr-tools/src/bin/osfr-gateway.rs:774-812`; admission: `:1226-1227`, `:450-472`; new cleanup guard: `:2451-2459`. Preserve it and extend ownership checks rather than reimplementing that fix. |
| Gateway already has a multi-session event loop. Its ordinary account-store path is not a one-session server. It removes, handles/ticks, and reinserts sessions. | `prototype/osfr-tools/src/bin/osfr-gateway.rs:1804-1808`, `:1825-1899`. Do not rebuild it. |
| The runner's binding reader parses only ticket and GUID; the shared gateway later parses the whole login, including version and end-of-message. Authentication must not mutate saves before that full validation. | `prototype/osfr-tools/src/bin/osfr-gateway.rs:1575-1583`, `:1169-1185`; `prototype/osfr-tools/src/gateway.rs:480-495`. |
| CharacterLoginReply currently puts the same ticket string in both ticket and key positions. Login configuration requires an ASCII ticket of 16, 24 or 32 bytes. Preserve the 32-ASCII-byte form during this change. | `prototype/osfr-tools/src/login.rs:175-187`, `:682-705`. These are current Rust constraints; acceptance of the new random value still needs a native run. |
| Account writes already use bounded versioned JSON, validation and the character store's atomic replacement helper. Account records currently have no credential field. A concurrent builder advanced storage to schema 2 with a separate account-role map; load accepts schema 1 and assigns Player roles. | `prototype/osfr-tools/src/account_store.rs:17-29`, `:76-92`, `:330-375`; `prototype/osfr-tools/src/account.rs:34-42`. Preserve this role data; W76 must not allocate schema 2 again. |
| Native run 023623 proves migration, selection and world entry for one account, not HMAC validation. Its reported first account/character and the current 1/1 branches explain the bypass. | `notes/Reviews/Native Login Accounts 023623.md:3-18`; `prototype/osfr-tools/src/login.rs:414-418`; `prototype/osfr-tools/src/bin/osfr-gateway.rs:760-764`. The review does **not** explicitly record which verifier ran. `notes/Design/Build Ledger.md:37` records the HMAC-native gate as open. No new run was performed for W76. |
| The dev harness injects a constant SessionId. It cannot exercise per-account credentials unchanged. | `scripts/run-local-baseline.py:248-258`. |
| Loopback restrictions exist in login configuration/receive and gateway receive. Binding/exposure is separate M1.1 work. | `prototype/osfr-tools/src/bin/osfr-login.rs:82-83`, `:468`; `prototype/osfr-tools/src/login.rs:177-179`; `prototype/osfr-tools/src/bin/osfr-gateway.rs:1831`. |

The latest native review was also checked for scope: `notes/Reviews/Native Round5 050310.md:7` records login/character-select/world-entry passing; it provides no two-client or HMAC-path acceptance. The living plan explicitly assigns multi-login, collision/stale-save fixes, roles and native multiplayer as separate items (`notes/Design/Build Plan.md:53-57`). W76 coordinates with 1.4; it does not duplicate a second collision fix. Schema-2 role storage and the teardown guard above appeared during this read-only task; their implementation/native completion is not asserted here.

### Native request: what the client actually sends

Source: `lab/native-map/run/map.sqlite`, accepted `label` rows and original x86 bytes read by `python -I scripts/native_map/nmq.py`. The image location is selected in `scripts/native_map/nmq.py:186`. This note contains findings, not copied decompiled source.

| Native fact | Address / file provenance and check |
|---|---|
| OnConnect builds request type 1 and copies two credential-provider string objects into record offsets +4 and +0x38. It dispatches by the transport mode. | `00fa9620`, label **unknown**, `notes/Build Guides/Network/Login and Account.md:456`. Verified with `nmq dis 00fa9620 427`: type at `00fa9667`; copies at `00fa969b-00fa96b0` and `00fa9710-00fa972a`; sender calls at `00fa9751` / `00fa976e`. Do not rely on the label's TCP/UDP descriptions. |
| The binary serializer writes one type byte, then a length-prefixed string from +4 and another from +0x38. Lengths are native little-endian u32 byte counts; contents are copied without an appended terminator. | `00fa6800`, label **speculative**, checked with `nmq dis 00fa6800 106`; calls at `00fa6852` / `00fa685e` to `00911c30`. `nmq dis 00911c30 143` checks raw 4-byte length then that many bytes. Native map label is corroborated by `prototype/osfr-tools/src/login.rs:310-313` and the independent serializer fixture test `prototype/osfr-tools/tests/login.rs:78-91`. |
| The parallel named serializer identifies the same record members as **Session** (+4) and **Fingerprint** (+0x38). This supplies field meanings, rather than the Rust variable names. | `00fa6d10`, likely; `notes/Build Guides/Network/Login and Account.md:779`; checked with `nmq dis 00fa6d10 104`: offsets at `00fa6d28` / `00fa6d45`, literal names at `00fa6d2e` / `00fa6d4b`. |
| The sending routine serializes with `00fa6800` and sends data/length through transport virtual slot +0x50. The ExternalLoginUdpApi constructor initializes mode zero. | `nmq dis 00fa7830 140` (`00fa7892`, `00fa78ad-00fa78b6`); `nmq dis 00fa3da0 150` (`00fa3dcd` stores zero at external API +4). This contradicts the old label assigning the binary branch to TCP; trust the bytes. |
| A separate LoginCommand_SendPassword class exists: class ID 410, version 7, writer `014440e0`, reader `01445420`. It is a different protocol family, not the two-string ExternalLogin request. Its historical field semantics and routing from this native launch are not established here. | `nmq command 410`, `nmq label 01445420`, `nmq dis 014440e0 180`; `notes/Build Guides/Network/Login and Account.md`, LoginCommand_SendPassword section. Do not infer a password field from a class name or from `0124f610`'s UserName/Password **field-name keys**. |

Therefore native LoginUdp_6 supplies **no distinct username or password** on the verified path. Session is an external-authentication bearer value; Fingerprint is client-supplied metadata and cannot prove identity. The replacement meaning of Session below is (R), not a recovered historical account service.

## Threat model and boundaries (R)

Attackers can send arbitrary packets, replay grants, use another account's GUID, open many source ports, submit huge/truncated strings, and race logins. An authenticated account is still hostile. Local account files, the handoff key and grant directory are trusted operator data; clients cannot choose their paths. Passwords, raw Session values, tickets, fingerprints, player names and chat must never enter ordinary logs or this note.

Keep login/gateway exposure unchanged in this task. The password channel below listens only on a separate loopback socket. For a second PC, forward that socket over an authenticated encrypted tunnel; protect game UDP with the reviewed M1.0/M1.1 deployment transport as well. RC4 with a distributed compatibility key is not a confidential, authenticated password channel. Do not reuse or expose Studio. A later HTTPS front end may replace the tunnel; it is not necessary to this local ten-account slice. No custom TLS, web framework or public registration site is included.

Build Plan 1.0's security-model review remains a prerequisite for non-loopback use (`notes/Design/Build Plan.md:21`, `:51`). W76 defines login controls, not approval to expose the service.

## Credentials and sign-up (R)

1. **Operator-assisted sign-up, offline:** add `osfr-account create --store <operator-path>`. Read a bounded account handle and password from a private input stream supplied by a no-echo UI, never command-line arguments or environment variables. Confirm the password in that UI. Create an empty account, not a copy of the legacy character. A server-owned limit of 16 enrolled accounts is sufficient for ten friends. No packet auto-creates accounts.
2. Use ASCII account handles of 3-32 bytes, letters/digits/underscore/hyphen, **case sensitive**. Store only SHA-256 of the exact handle bytes, as an index, plus the password PHC string. Do not lowercase/re-hash existing identities. Passwords are exact UTF-8 bytes, 12-128 bytes, without NUL; no trimming or normalization. These are account handles, not character display names.
3. Add nullable `password_phc` and `credential_generation` to Account and advance the document to **schema 3**, preserving the concurrent schema-2 roles. A null value is **disabled**, never passwordless. Offline `osfr-account enroll --account-id <id>` sets credentials only when the operator-supplied handle hashes to the existing account hash. An incompatible legacy handle requires an explicit operator binding operation preserving the old ID; do not silently adopt it on first network login.
4. Login server holds an exclusive `create_new` ownership lock beside the store for its lifetime. Offline account CLI acquires that same lock and refuses a live server. A stale lock requires an operator to verify the recorded PID/start identity and recover it; never auto-delete it on age alone. After sign-up, the running login server is the only AccountStore writer. Gateway reads the store; lead must route any M1.5 role mutations through the same serialized owner rather than letting another cached process overwrite credentials/roles. This is an integration invariant, not a claim that role mutation routing already exists. No new database or file-lock dependency is needed.
5. Add one separate loopback **credential broker** in the login process and one small `osfr-account login` helper for the launcher. Frame one request as `u32le body_len`, magic/version `AUTH1`, then the existing u32-length string encoding for handle and password. Bound the body to 192 bytes, require exact end, and accept one request per TCP connection. Reply has fixed length: one status byte plus 32 token bytes, zero-filled on any refusal. Never echo inputs. No native packet is added.
6. The helper obtains input from the launcher/no-echo UI through bounded stdin and sends it to its loopback broker/tunnel endpoint. It returns the Session token only over a private pipe to the launching parent, never the console or a general log. The lead replaces `SessionId=local-session` with that token. The password never enters native command-line arguments; the short-lived Session bearer does, which remains a local-process disclosure limitation. Restrict process/run-directory access and redact launcher/client logs before sharing them.
7. Hash with RustCrypto **argon2 0.5.3**, Argon2id v19, explicit `m=19456 KiB, t=2, p=1`, 32-byte output, and a fresh 16-byte OS salt. Store the PHC encoding. Use `PasswordVerifier::verify_password`; do not compare passwords or hash strings with `==`. Validate stored PHC algorithm/version/parameter bounds before queuing work: verification uses parameters from the stored PHC, not the Argon2 instance. Fail startup on unsupported/excessive stored parameters.
8. Unknown/disabled handles use one precomputed dummy PHC with the same policy, performing one verification before the same generic refusal. Malformed framing and exhausted quotas may fail before hashing. Constant-time hash/tag primitives do not make the whole endpoint constant-time; bound work and avoid user-enumerating responses.
9. Password verification runs on **two std::thread workers**, each doing at most one Argon2 operation. A `sync_channel` holds at most 16 queued requests. The event loop owns account state, counters and replies; workers receive only a PHC/password/request generation and return results. Reject stale results after timeout/disconnect or credential-generation change. This keeps hashing away from the 10 ms SOE poll loop. Document `ponytail: two fixed workers and JSON suffice for ten accounts; measure before changing persistence or scheduling`.
10. Before hashing: per broker source IP, 60 attempts/minute with burst 20; global starts, 2/second with burst 20. Five failures for an account-hash key within 5 minutes lock it for 60 seconds. A locked attempt neither extends the lock nor starts Argon2. Track unknown hashes too; cap the failure map at 1,024 entries and reject new buckets when full, retaining live locks. Expire counters after 10 minutes. Reset the account failure bucket on success. For tunnels/NAT, IP limits are deliberately shared; never trust a client-supplied forwarded IP. All counters are process-local; restart clears lockouts but does not remove password verification.
11. On success issue 16 random bytes encoded as **32 lowercase hex ASCII characters**. Store SHA-256(token) mapped to account ID, credential generation and a 60-second `Instant` deadline in a bounded map (max 32, max two pending grants/account). Opcode 1 consumes it exactly once and binds that account to this login session. Fingerprint is bounded/ignored for authority. Another source peer or a new session cannot reuse it. A reliable retransmission is deduplicated before application handling, rather than consuming it again. Restart drops all pending login grants.

Broker admission: at most 16 accepted connections, one 3-second read deadline, one 15-second total authentication deadline including the queue, and 192 request bytes per connection. Use nonblocking stdlib sockets in the same loop; a slow TCP writer cannot block SOE handling. Process a bounded amount of work per iteration and retain no unbounded credential buffer. Drop password buffers after verification; no `Debug` derivation or secret-bearing errors.

Dependency decision, after checking `prototype/osfr-tools/Cargo.toml:9-13` and Cargo.lock: the checkout provides SHA-256, serde and compression, not password hashing, OS randomness or a dedicated constant-time compare. Add only `argon2`, `getrandom`, and `subtle` as direct dependencies, locked by the lead. `getrandom::fill` supplies salts, grants and handoff-key generation; fail closed on RNG failure. Reuse the existing HMAC-SHA256 construction; use `subtle::ConstantTimeEq` for fixed-size HMAC/tag checks instead of the ordinary XOR fold, with an independent vector regression test. No password KDF or RNG is written here. A direct `subtle` declaration is required even if Argon2 brings it transitively.

External sources consulted 2026-10-15: [RustCrypto Argon2 0.5.3 API](https://docs.rs/argon2/0.5.3/argon2/) (PHC hashing/verifying, stored parameters), [OWASP password storage](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html) (Argon2id minimum used above), [getrandom 0.3.4](https://docs.rs/getrandom/0.3.4/getrandom/) (OS random bytes), [subtle 2.6.1 ConstantTimeEq](https://docs.rs/subtle/2.6.1/subtle/trait.ConstantTimeEq.html) (comparison primitive). Proposed versions are API references, not a dependency installation or audit result.

## Peer sessions and shared ownership (R)

Replace `Option<Session>` with `HashMap<SocketAddr, Session>`. SocketAddr includes source port, so two clients behind one IP stay distinct. Reuse the existing Session fields and reliable transport limits. Keep shared resources/config outside the sessions (`Arc<Resources>` is enough), and keep AccountStore only in the loop's service context. Each LoginService keeps only its authentication/selected-character state. Pass `&mut AccountStore` through `handle`; stage and atomically save mutations, then commit the authoritative in-memory copy. Lists and name checks read that same current store.

For identical peer + Request, resend the saved SessionResponse; never reset RC4 or reliable sequence numbers. For a different Request from an occupied peer, refuse it until the old session disconnects or expires. That avoids an unauthenticated handshake kicking an authenticated session. It also means an abrupt client restart must use a fresh source port or wait for expiry. Do not key the table by the client-chosen connection ID alone.

Bounds: 32 login sessions total, 32 per source IP (allows ten clients behind NAT/loopback); 30 seconds maximum before authentication; 120 seconds idle after authentication; 10 minutes absolute session lifetime. Preserve the existing per-session 256 KiB send queue and 128 KiB receive buffer (`prototype/osfr-tools/src/bin/osfr-login.rs:190-207`). Update activity only for accepted controls/application progress, not any decodable junk. Poll every session fairly on each loop iteration; one malformed peer, exhausted sender or timeout removes only its entry. An error creating one Session cannot escape `run()` through `?` and terminate everyone.

Handshake limiter: per source IP 20/second, burst 32; cap its table at 1,024 addresses and idle-expire entries. Enforce caps before allocating Session/resources. Login packet limiter: per peer 200 datagrams/second, burst 256, with transport ACK/control progress included in that budget. Refuse flood traffic without generating a response amplification loop. Limits are deliberately generous starting policies for ten accounts, not measured throughput claims.

## Gateway HMAC handoff and replay protection (R)

Preserve the 32-character ASCII ticket and identical ticket/key fields in CharacterLoginReply. A long self-contained JWT is unnecessary and changes a crypto-sensitive client boundary. Use the existing shared key, upgraded by the lead to 32 OS-random bytes and kept in a private key file for both processes.

On successful opcode 7, verify ownership against the authoritative store and create a **new immutable grant record** in an operator-configured private `handoffs/` directory. Record version 2, account ID, character GUID, login-name hash, 16-byte random nonce, issue/expiry epoch seconds, and credential generation. Lifetime is 60 seconds. Encode the MAC input in a fixed documented order with fixed-width little-endian integers and the domain `osfr-login-binding-v2` followed by a NUL. Compute HMAC-SHA256 and use its first 16 bytes as the 32-hex-character ticket. Include every record field in the MAC; do not MAC serialized JSON text.

Store records under **SHA-256(ticket)** filenames, not raw tickets or client paths. Write the immutable new record with `create_new`, `write_all`, then `sync_all`; only after completion send success. An incomplete/crashed write fails record validation and cannot authenticate. Do not check existence and then use an overwriting replacement operation for grant creation. Limit the directory to 128 records (including incomplete writes) and two unexpired grants per account; refuse issuance on capacity/I/O failure. A writer removes its own failed partial record; startup prunes invalid records under the trusted private directory. Repeat opcode 7 for the same selection in the same session returns its cached unexpired ticket; selecting again after expiry issues a fresh grant. Selection/deletion revokes that session's pending prior grant. No state is granted by a ticket until gateway validation.

Gateway admission order, on its single event loop:

1. Parse the **entire** gateway login using the shared bounded parser, including expected version and no trailing bytes. Require ticket length 32 and lowercase hexadecimal encoding before deriving a safe digest filename. Do not load/copy/save a character yet.
2. Read at most a small fixed record bound (1 KiB), validate schema, fixed field lengths, `issued <= now < expiry`, `expiry-issued <= 60`, credential generation, and selected GUID. Load the current AccountStore; derive ownership from it, never a client-supplied account ID. Compare the expected 16-byte HMAC with `ConstantTimeEq`. A grant issued under a different key/account/character/generation fails.
3. Check/reserve **both typed ownership keys** for this transport session before loading a save. Enforce one live gateway session per account and per character, including sessions admitted but not yet in the world. A duplicate cannot disconnect the existing owner, change its snapshot or save anything.
4. Consume with `OpenOptions::create_new(true)` on `<ticket-digest>.used`; sync the marker before activation. A pre-existing marker rejects replay. This is an atomic claim across processes; check-then-write or an in-memory replay set alone is insufficient. On marker/sync failure release the reservation and fail closed. On later load failure keep the grant spent, release ownership, and require a fresh selection. A crash after consumption may require login again; safety wins over automatic reuse.
5. Load the character save, apply validated identity, and set the session's admitted binding. Commit startup/login state only after the full login succeeds. Only a session that still owns **both** keys may write that character's snapshot, including selection-time initialization, routine commits and teardown. Cleanup of an unadmitted/rejected session never saves. Release reservations on every error, disconnect and timeout, including pre-world sessions. A transport retransmission must hit the existing reliable deduplication/session cache, not consume a new grant.

Remove both 1/1 special cases, including their test-fixture branches. Account 1 uses the same v2 ticket issuance and consuming verifier as every account. Normal account-store mode refuses the old deterministic v1 ticket and fixed ticket. Keep any identity-only development mode explicitly separate and loopback-only; it cannot be an implicit fallback for failed authentication.

Used markers survive process restart until their grant expires. Cleanup first removes the expired grant, then its marker; an absent grant always rejects. Never delete a live marker merely because a process restarted. Wall-clock validation uses a forward-moving effective clock within a process; a detected backward jump fails grant validation/issuance closed until clock recovery. Restart does not renew stored expiries. Shared directory ACLs and trustworthy host time are required; restoring an old entire server filesystem is outside this replay model.

Do not log grant filenames, token digests, record nonce, MAC input, keys or ticket strings. Log only a bounded reason code, opaque session ID and internal account/GUID IDs. Native client logs may print credentials (`00711220`, `nmq label 00711220`); captures/logs containing ticket/key data stay private and need redaction before review artifacts are shared.

## ID collision and save migration (R)

Use two `BTreeMap<u64, SessionId>` fields: `account_owners`, `character_owners`. This fixes the responsible layer without renumbering every save or trying to reserve a high bit that has never been proved safe for EntityKey. Equal numbers across the maps are legal; duplicate numbers within either map are not. Check both keys before inserting either; a failed reservation must leave neither key behind. `join` reuses the reservation owned by that same SessionId; it cannot claim someone else's key. `leave`/`release_identity` remove entries only when that SessionId owns them.

**ID/save migration is intentionally identity-preserving.** Existing AccountStore IDs, both next counters, CharacterRecord ownership, world GUIDs, `characters/<guid>.json`, social paths and legacy source snapshots stay unchanged. The schema-2-to-3 migration preserves roles and adds only `password_phc: null` and credential generation 0 to each existing account. Schema 1 first passes through the existing role migration, then gains the disabled credential fields. Existing accounts are disabled until explicit enrollment; first network traffic cannot claim the old save. The private credential generation is covered by issued grants and advances on credential replacement.

Migration procedure: stop services; hash/copy the store and all saves; acquire exclusive store ownership; strictly decode/validate schema 1 or 2; retain every other field and role; write schema 3 through the existing atomic replace helper; reload/validate; compare every old account/character ID/counter/role and every save byte/hash. Unknown schema or invalid counters fail without writing. Enrollment is a separate transaction. Keep the backup; rollback requires stopped services, restoration of matching files/binary, and invalidation of all grants (rotate the handoff key and clear the stopped system's grant directory). Never rewrite GUIDs to make collisions disappear.

## Ordered function-level changes and work split

All entries are proposed; this W76 worker owns only this note. File paths below are relative to `prototype/osfr-tools/` unless prefixed `project/`. Future Luna workers need explicit file ownership; they must not edit the lead's shared paths opportunistically.

| Order | File / function | Minimal change | Owner |
|---|---|---|---|
| 1 | `src/players.rs`: `PlayerRegistry::{new,join,leave}`; new `reserve_identity`, `release_identity`, `owns_identity` | Replace mixed map with two maps; reserve before save loading, release only the caller's entries; no persistent renumbering. Coordinate once with M1.4. | Luna, a dedicated players-file assignment |
| 2 | `src/account.rs`: `Account`; `src/account_store.rs`: `load`, `save`, `validate`, `create_account`; new explicit `enroll_credentials` | Schema 3; optional PHC and generation; preserve schema-2 roles and schema-1 conversion; disabled-until-enrolled semantics. Keep separate monotonic counters. | Luna, dedicated account files, coordinated with role owner; lead reviews migration |
| 3 | new `src/login_auth.rs`: `hash_password`, `verify_password`, `AuthLimiter`, `issue_session`, `consume_session`; new `src/bin/osfr-account.rs`: `run` | Only credential hashing/limits/broker framing and small operator/client CLI. Private bounded stdin, store ownership lock for offline commands; no native UI or web server. Use existing Reader/string encoding through narrowly exposed helpers. | Luna, new files; lead registers module and dependencies |
| 4 | `src/login.rs`: `LoginConfig`, `LoginService::{new,reset_connection,authenticate_account,store_and_account,handle,character_list_reply}` | Remove Session-name equality and packet auto-account creation; consume broker Session token. Pass current shared store instead of caching a per-session copy. Keep existing character handlers, transactional save pattern and native reply layouts. | Lead shared integration, or one explicitly reassigned login-file worker |
| 5 | `src/bin/osfr-login.rs`: `Options::{parse,service}`, `Session::{new,handle}`, `run` | Own shared Resources/store/auth context; peer table, caps/expiry/fair poll, isolated per-peer failures; poll separate loopback credential listener and bounded hash results. No global async runtime. | Lead |
| 6 | new `src/login_handoff.rs`: `issue`, `verify`, `consume`, `revoke`, `prune`; `src/login.rs`: `gateway_binding_ticket`, `verify_gateway_binding_ticket`, opcode 7 | Small concrete grant-record helper using existing HMAC/SHA-256/atomic file helpers, v2 fixed MAC encoding and create-new replay marker; cached per-session selection. Delete 1/1 bypass. | Luna helper file/tests; lead routes login.rs and module exposure |
| 7 | `src/gateway.rs`: extract/reuse full login parser from `parse_message`; `src/bin/osfr-gateway.rs`: `login_character_binding`, `Session::{select_character_from_login,dispatch_message,commit_outcome}`, `Server::{accept_handshake,end_session}` | Share bounded full parse; validate/claim ticket and reserve typed keys before load; extend existing teardown guard so every save requires ownership; remove bypass; release pre-world reservations. Cap gateway transports at 32, with 30-second unadmitted deadline and fair cleanup. Keep existing world/session loop. | Lead; depends on 1 and 6 |
| 8 | `Cargo.toml`, `Cargo.lock`, `src/lib.rs`; `project/scripts/run-local-baseline.py` and M1.7 launch handoff | Pin/register the three justified crypto crates/helper modules; inject broker token into SessionId; keep independent client run directories/capture streams. Remove secret-bearing argument logging. | Lead only |
| 9 | Focused tests below, new security review, Build Plan/Ledger/Current Progress and native review | Review actual route behavior, not helper presence; mark Wired only after integration and Native-verified only after acceptance. | Luna tests within assigned paths; lead integration/review/native |

No database, OAuth/JWT service, generalized authentication trait, new transport implementation, async runtime, public sign-up page, role system or launcher redesign is included. Roles remain M1.5; this change must not enable dev tools for all authenticated players.

## New regression tests: names and assertions

These are **specifications**, not tests executed by W76. Use synthetic account handles and existing private appearance fixtures; never introduce player names/chat. Table-driven cases can share a test where the same boundary is exercised.

| Target / new test | Required assertion |
|---|---|
| `players`: `account_and_character_ids_have_separate_namespaces` | Accounts 1 and 2 with characters 2 and 3 can both reserve/join; cross-map equality is legal. Duplicate account or duplicate character fails without a partial insertion. |
| `players`: `reservation_survives_until_world_join_and_releases_only_owner` | Pre-world reservation prevents a second session; wrong-session release is inert; error/leave releases both keys; legitimate next login works. |
| `account`: `legacy_enrollment_preserves_ids_roles_counters_and_save_bytes` | Schema 1 and role-bearing schema 2 become disabled schema 3; IDs, counters, roles, ownership, legacy inputs and every character-save byte remain equal; enrollment requires the existing hash. |
| `account`: `bad_schema_or_failed_replace_leaves_store_unchanged` | Unknown schema, duplicate/reused IDs, bad PHC parameters and simulated atomic-write failure leave original bytes intact. No half-enrolled record. |
| `login_auth`: `argon2_verification_uses_salted_phc_and_generic_refusal` | Same password has distinct salts/hashes; correct password succeeds; wrong/unknown/disabled cases give the identical fixed-size refusal; malformed/excessive PHC is refused before work. |
| `login_auth`: `lockout_and_limits_are_bounded_and_expire` | Fifth failure locks for 60 seconds; blocked attempts do not extend it; fake-clock expiry permits retry; shared-IP ten-account burst passes; global/hash queue and 1,024-key caps hold. |
| `login_auth`: `session_grant_is_single_use_expiring_and_account_bound` | Successful password verification gives exact 32 ASCII hex bytes; consumption returns only its bound account once; expired/replayed/cross-generation grants fail; restart loses pending grants. |
| `login_auth`: `slow_broker_and_stale_hash_result_do_not_block_or_bind_udp` | Slow/truncated 192-byte frame expires; queue never exceeds 16 plus two running; UDP session ticks continue; disconnected request generation cannot receive a late token. |
| login binary: `ten_interleaved_peers_keep_independent_rc4_and_reliable_state` | Ten socket peers on one IP interleave handshake/login/list; replies decrypt for the right peer; dropped ACK/retry is byte-identical; one disconnect does not reset nine sessions. Exercise actual Session/run logic, not just a HashMap. |
| login binary: `handshake_retry_replacement_caps_and_expiry_are_local` | Identical handshake resends without reset; changed Request cannot evict live peer; 33rd peer is refused; malformed/timeout peer is removed alone; expiry recovers capacity. |
| `login`: `shared_store_interleaved_create_delete_keeps_both_accounts` | A and B authenticate, alternate character creates/deletes, then list/reload; all accepted mutations and unique GUIDs persist, with no cached-store overwrite. |
| `login`: `wrong_account_requests_cannot_list_select_or_delete_other_character` | No other-account character is listed; guessed GUID select/delete fails without store/save change or ticket. Pre-auth requests fail. |
| `login_handoff`: `v2_ticket_binds_every_field_including_first_account` | Every pair including 1/1 gets v2 ticket; changing account, GUID, hash, nonce, expiry, generation or key fails; malformed encodings fail safely. Keep an independent HMAC vector. |
| `login_handoff`: `consume_is_atomic_persistent_and_expires` | Two threads/processes racing one token get one winner; recreated verifier after restart rejects spent token; expiry/future-issued record fails; expired cleanup never reopens a ticket. |
| `login_handoff`: `issue_failure_and_revocation_do_not_return_success` | RNG/disk/cap/sync failure yields no usable success ticket; same-session retry caches a valid ticket; delete/reselect revokes old pending grant. |
| `multiplayer`: `two_native_style_login_paths_reach_hmac_gateway_entry` | From opcode 1 through real login service/character selection into running gateway: distinct accounts load their own GUID/save, both reach world_ready; no fixed ticket accepted. Include migrated 1/1. |
| `multiplayer`: `duplicate_or_malformed_login_never_saves_over_active_owner` | Distinct valid ticket for the same active account, replayed ticket, wrong GUID, wrong version and trailing/truncated login are rejected; active session continues and its latest save hash/state is unchanged. |
| binary integration: `admitted_timeout_releases_account_and_character` | Valid ticket followed by no world-entry expires/release; next freshly authenticated login succeeds; cleanup of rejected transport performs no save. |
| binary integration: `credential_and_ticket_material_is_absent_from_logs` | Inject known synthetic handle/password/token/key/fingerprint and scan ordinary trace/stdout/stderr: none appears, including parse/I/O failures. Keep raw native capture checks private. |

Implementation gates from `project/` (future, **not run** for this design-only task):

```powershell
& .\scripts\cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml --test account --test login --test players --test multiplayer
& .\scripts\cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml --test login_auth --test login_handoff
& .\scripts\cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml --bin osfr-login
```

Lead adds binary-local tests where private runner functions are needed; no crate-wide formatting. The native two-client gate below and a security-reviewer pass on the **implemented** diff are mandatory. A content checker adds no proof for this design because no content changes are proposed.

## Native acceptance script for two clients (lead only; not executed)

This is an ordered runbook for the proposed interfaces, not an executable script or evidence that these flags exist today. Use two independent native client copies, ideally two PCs after M1.0/M1.1. On one host, use separate working directories and source ports. Do not start two copies of the current dev harness: both must use **one** scratch login server and **one** scratch gateway/store.

1. Create an isolated scratch run directory under `lab/working/`, empty schema-3 account store, per-character saves, private 32-byte handoff key and private handoff directory. Freeze binary/source hashes and client image hashes. Preserve original campaigns untouched. Start neither client yet.
2. Offline, run proposed `osfr-account create --store <scratch accounts.json>` twice through a no-echo/private-input wrapper, enrolling two synthetic accounts. Also test a copy of legacy 1/1: migrate and enroll it explicitly, then use it as account A. Verify all input save hashes remain unchanged. These two enrollment cases can use separate fresh run directories.
3. Start the lead-built gateway on a free scratch loopback UDP port, `--studio-port 0`, account-store mode, no dev tools, same key and new grant-directory option. Start exactly one login server on another free port with that gateway address, same store/key/directory, and its own loopback broker port. For a two-PC run the reviewed protected transport supplies endpoints; never widen Studio/broker directly.
4. On each client machine, call proposed `osfr-account login` from the launcher's private pipe with the account's handle/password and loopback broker/tunnel endpoint. Capture its returned Session token in memory. Launch the original client copy with `Server=<same login endpoint>` and `SessionId=<that token>`. Do not pass a password to the native client, print the token, or store it in the published run command. Each client gets its own private capture/log directory.
5. **Overlap login work:** pause A at character select, start B, then return to A and request the list/select. Both remain responsive; A is not replaced by B's handshake. Capture two peer/session IDs and account IDs, without credential strings.
6. In each account create one supported character through native UI, or select its scratch character. A cannot list/select/delete B's GUID. Request character login on both within the grant lifetime. Record redacted server events proving **v2 verification and atomic consumption** for each, including account 1 if migrated. A shared key file or world_ready alone is insufficient HMAC evidence.
7. Both clients enter world and remain connected concurrently for at least 120 seconds; both can move. Confirm two admitted account/character bindings and two independent save paths. Other-player rendering/chat/group correctness belongs to 1.3/1.4/1.6 and is not required to pass this narrower login gate.
8. While A remains active, use a disposable protocol client for a fresh-ticket duplicate account login, a replay of A's spent ticket, a mutated ticket and an expired ticket. All fail with bounded refusals; A remains connected. Compare A's latest snapshot bytes/hash before and after each rejected attempt. Perform the replay after a scratch gateway restart too; then obtain fresh tickets and restore both clients normally.
9. Exit B normally; A continues moving. Authenticate B again with a **new** Session grant and new HMAC ticket, then verify its saved GUID/gameplay. Exit both, restart scratch services and relog each to prove ownership/save persistence. Keep the two accounts distinct after restart.
10. Negative password/unknown-account attempts give the same refusal. Trigger the five-attempt lockout on a disposable account; verify a timed retry succeeds and the other client continues. No secrets appear in ordinary logs. Preserve private packet evidence, redacted screenshots, verifier/admission events, source/binary/run hashes, per-save before/after hashes, exit reasons and exact private commands in a new native review.

Acceptance result must name both native run IDs and state which checks passed/failed. This W76 design has no native run ID and does not close Build Plan 1.2.

## Security-reviewer pass on this design

Read-only self-review using secure-code-guardian, rust-engineer, reverse-engineering, codebase-memory (Verify tier), Ponytail full and Caveman for chat. No agents or native client were started. Automated source queries preceded manual auth/path/crypto review; no exploit, SAST package installation or implementation audit is claimed.

| Severity | Finding / location | Resolution in this design |
|---|---|---|
| High | Native Session/Fingerprint misread as name/password (`login.rs:310-314`; native `00fa6d10`). | Password broker before launch; opaque Session only; no Fingerprint authority. |
| High | Deterministic/reusable tickets and first-account bypass (`login.rs:414-418`, `:552-581`; gateway runner `:760-764`). | Nonce/expiry/generation-bound v2 grants; durable exclusive consume; remove both bypasses. Implementation/native proof open. |
| High | Cached per-session store could lose another account's changes (`login.rs:255`, `:349-362`, `:463-468`). | One event-loop store owner, offline provisioning lock, staged writes. |
| High | Selection-time write precedes ownership (gateway runner `:774-812`); a concurrent cleanup guard now covers another live character owner (`:2451-2459`). | Preserve that guard; full parse and typed reservation before load; owner-checked saves on all paths. |
| Medium | Account/character numeric alias (`players.rs:124-148`). | Two independent maps; preserve save identities. |
| Medium | Hashing or slow broker can stall login traffic (new proposed password boundary). | Two bounded hash workers; nonblocking bounded connection handling, deadlines and rate limits. |
| Medium | Native bearer appears in client launch/log context (`scripts/run-local-baseline.py:257-258`; native `00711220`). | Short-lived one-use values, private pipes/process/run access and redacted evidence. Same-user local compromise remains outside the authentication boundary. |

Findings have design mitigations, **not implemented fixes**. An independent review of the integrated code, threat model and native evidence remains open.

## W76 evidence commands and results

Executed from `project/`, read-only except writing this note:

- Read AGENTS, worker contract, RTK, Ponytail/Caveman, secure-code-guardian/authentication reference, rust-engineer, security-reviewer, reverse-engineering and codebase-memory skills. Read Build Plan/Ledger, Login Integration, Build Guides index/Login and Account, Current Progress, MODLOG/Tasks, Native Login Accounts 023623 and relevant newest Round5 review entries.
- `rg` and line-numbered `python -I` source reads over login, account/store, players, gateway runners/shared parser, existing tests, Cargo manifest/lock and dev harness. Results are the baseline table above. Initial broad `rg --files`/`git status` warned about an unrelated inaccessible `tests/tmppep0efg1/`; targeted evidence reads succeeded. One incorrectly quoted source search and one SQL shell command failed; corrected quoted paths / a Python stdin script supplied the relevant evidence. No state was inferred from those failures.
- Code graph: `list_projects` selected `cartoonmmo`; `search_graph` found `authenticate_account`, ticket helpers and `load_or_migrate`; `trace_path` inbound for `verify_gateway_binding_ticket` found `Session::select_character_from_login`, then `Session::dispatch_message`. `check_index_coverage` returned **8 paths, no_recorded_issue / metadata_match**, generation `2026-10-16T01:56:50Z`; best-effort coverage is not exhaustive proof. Material claims were checked in source.
- `python -I scripts/native_map/nmq.py --help`; `strings`, `label`, `show`, `callers`, `command 410`; exact disassembly commands in the native table. All decisive native queries succeeded. Unknown/speculative labels were checked against bytes before use. Read-only SQLite access used `mode=ro`; raw/decompiled output was not written into the repository.
- Primary-source web reads: Argon2/PHC, getrandom, subtle and OWASP links above. PHC source-page requests failed; the published Argon2 API supplied the stored-parameter behavior. No new crate was fetched or installed.
- Design document/source-location checks and final scope check are recorded below after writing. No Rust behavior changed, so no compilation, Rust test execution, content check, scratch server or native run was performed. All new tests and acceptance steps remain implementation gates.

Ponytail: reuse Session/reliable transport, account handlers, bounded Reader, JSON validation/atomic replacement, SHA-256/HMAC and the existing gateway event loop. Deliberately omit a database, public auth site, custom TLS/KDF/RNG, JWT, transport rewrite, identity renumbering and unrequested gameplay work.

### Final document check (executed)

`python -I -` checked the document's explicit source paths and ranges, nine decisive current-behavior anchors, UTF-8 integrity and balanced code fences. Output:

```text
Explicit source citations: 39
Citation target/range errors: 0
Behavior anchors checked: 9
UTF-8 replacement characters: 0
Code fences balanced: True
```

The first check caught one out-of-range manifest citation and later detected account-store line shifts while the role worker edited. The note was corrected and the check above passed. A final coverage refresh reported `metadata_changed` for account_store.rs and osfr-gateway.rs; direct source reads and the anchors above supersede the earlier graph metadata. The new schema-2 role migration and duplicate-cleanup guard are retained in the proposal.

Source SHA-256 values at this check (line citations are tied to this read; later concurrent edits require rebasing):

```text
prototype/osfr-tools/src/login.rs 770edeb4fa0986ce5cba71e744309718ced59e70cb429d1e1a4a34de805f350a
prototype/osfr-tools/src/bin/osfr-login.rs c6313f393c1f6973cead90a4391628c94d683c047e2885cfaab68465cb1f41d7
prototype/osfr-tools/src/account_store.rs 59feb6ffeccadd97e5736cb5608644a9abc844382192c35d5c76e334af1e2d2b
prototype/osfr-tools/src/players.rs a8ba0449aeb699be707cd1f5fed331962f0f11d5d629be88d9c0ead25ec2df59
prototype/osfr-tools/src/bin/osfr-gateway.rs a520a975ee1b7dba7a468c19388c69e1f3cee2fa5f69348d0319bbbc9cb54fac
prototype/osfr-tools/src/gateway.rs 3d37ec30fb269ce21d7d178e90c03b34e284529d37ca6e127aa8a5ebd5e5615a
```
