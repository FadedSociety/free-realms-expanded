# W94: key-pair login broker and credential failure limits

2026-10-16. W94 headless implementation and test gates: **PROVEN**. Complete M1.2 / LAN or internet acceptance: **PARTLY PROVEN**, pending Claude's review, launcher integration, gateway handoff hardening and native acceptance. No native client, shared service, launcher, release binary, manifest, shared module, save or project-status note was changed. Other workers' changes were preserved. No dependency was added.

## Files written

Project paths are relative to `project/prototype/osfr-tools/`:

- `src/bin/osfr-login.rs`: AUTH2 TCP broker (loopback, or a private/VPN address with `--allow-lan`); account-store writer lock; native Session redemption; peer-keyed login transports with caps, deadlines and existing token buckets; reuse of existing native character handlers.
- `src/login_auth.rs`: authenticated handle returned only after atomic token redemption; constant-time enrolled-key/hash comparisons; signature verification before unknown/disabled-account refusal; bounded per-account failure accounting using `net_policy::Bucket`.
- `tests/login_auth.rs`: deterministic account lockout, recovery, successful-login reset and unknown-hash capacity/expiry checks.
- `tests/login_broker.rs`: headless launcher and native LoginUdp_6 clients, real process/socket route checks, forced datagram loss and bounded reliable retries, private trace diagnostics on failure.

Evidence lives under the assigned `lab/build/w94/`: this report, `final-tests.log`, `broker-final.log`, `canary.log`, `client-loss-red.log`, `broker-diagnostic.log`, `security-scan.txt`, `source-hashes.json`, Cargo output and synthetic test directories. `osfr-account.rs` needed no edit: its existing key provisioning and cooperative writer lock are reused.

## Threat model and implementation boundary

Broker frames, signatures, public keys, native strings, tokens and transport peers are hostile. The operator's account-store path, resource paths, stable server ID, key files and filesystem are trusted. The login event loop serializes its writes and holds `AccountStoreLock` for its lifetime; the existing CLI refuses the same lock. No client field selects a file path or account ID. Missing stores and invalid schema fail startup in account-store mode; packet traffic cannot provision accounts there.

`--account-store` now always selects authenticated mode. It requires `--server-id-file` containing exactly 32 stable identity bytes. `--auth-bind` defaults to `127.0.0.1:20043`. Since Security Model decision 2 (2026-10-16) it also accepts a private-network address (the `net_policy::allowed` rule the UDP binds use) when `--allow-lan` is given, and peers must pass the same rule; public and wildcard addresses are refused at startup. The VPN supplies confidentiality for the playtest. Public play needs the launcher-embedded tunnel ([[Public Transport]]). Without an account-store flag, the existing loopback `local-session` development flow remains available; exposed development login is rejected.

The adapter consumes the Session token before calling `LoginService`. It supplies the verified, server-held account handle to the existing hash-based account lookup, never a handle from the native packet. Before each authenticated request it reconstructs the existing service from current disk state. This prevents interleaved login peers from saving stale cached account copies while respecting ownership of `login.rs`. It also reloads resource tables; throughput is unmeasured. A future lead-owned shared-store API can replace this concrete adapter if measurement warrants it. All account-store writers must obey the same serialized-owner invariant; this task does not prove unrelated cross-process writers obey it.

## Exact launcher contract (R)

Every item in this section is reconstructed policy, not historical Free Realms authentication.

1. Store a separate Ed25519 private key for each trusted server profile, with OS-only profile permissions. A profile binds the trusted endpoint/tunnel configuration AND the pinned 32-byte server identity. Never select another server's private key merely because a peer advertises its identity. The server receives only the public key through operator-assisted `osfr-account create` / `add-key`; no private key or password crosses the broker connection. Account handles are case-sensitive ASCII, 3..32 bytes, letters/digits/underscore/hyphen.
2. Connect to the profile's broker endpoint: a literal loopback, RFC 1918, 100.64.0.0/10 or IPv6 unique-local address (no DNS; public, wildcard and link-local refused). Each TCP connection carries exactly one request. Prefix the request body with its **u32 little-endian byte length**, excluding the prefix. Maximum body length is 192 bytes. No HTTP, JSON, newline, base64 or NUL terminator is used.
3. Challenge body: ASCII `AUTH2` (5 bytes), command byte `01`, u32 little-endian handle byte length, exact handle bytes. Body length is 13..42 bytes.
4. Read exactly **69 reply bytes**. Byte 0 is status (`1` success, `0` refusal). Challenge success: bytes `[1,33)` = configured server ID, `[33,65)` = random nonce, `[65,69)` = u32 little-endian lifetime, currently 60 seconds. Compare the returned identity with the profile's pinned identity before signing. A valid-format unknown or disabled account receives the same challenge shape. All refusals are 69 zero bytes; syntax/capacity/rate failures disclose no account detail. Socket exhaustion/deadlines may close without a reply.
5. Sign the exact bytes `ASCII("osfr-login-v1") || server_id[32] || nonce[32] || handle_bytes` with the profile's Ed25519 private key. There are no length prefixes, separators or terminators inside this signed transcript. The fixed-width ID and nonce make the concatenation unambiguous. Expiry is enforced against the server's stored issuance time, not a client timestamp.
6. Open a **new** TCP connection. Proof body is exactly 134 bytes: `AUTH2`, command byte `02`, nonce (32 bytes), public key (32 bytes), Ed25519 signature (64 bytes). Prefix it with `134u32.to_le_bytes()`. The server derives the account failure key from its nonce record, never a proof-supplied account handle. A well-framed proof consumes its challenge, including failure and locked-account refusal.
7. Read exactly 69 bytes again. Proof success: byte 0 = `1`, bytes `[1,33)` = **32 lowercase ASCII hexadecimal token bytes**, bytes `[33,69)` = zero. Refusal is all zero. Challenge and token each expire exactly 60 seconds after their respective issuance, using monotonic server time. Do not retry the same proof; obtain a new challenge after refusal, respecting rate/lock deadlines.
8. Pass the token unchanged into the native client's existing **Session** launch setting / LoginUdp_6 opcode-1 first string. Fingerprint remains ordinary bounded metadata; it grants no authority. The native packet stays `01 || u32le(Session length) || Session || u32le(Fingerprint length) || Fingerprint`. Exact end and both strings are validated before token consumption. Reliable transport retries are deduplicated before application handling.
9. Keep the token only in memory/private parent pipes long enough to launch this client. Do not persist it as a login credential or print it to general logs. A new native connection needs a fresh token. The login server atomically redeems it once, binds the resulting account to that peer and rejects reuse. Restart discards pending challenges/tokens. The launcher stores its private key, account handle and trusted server profile; it stores no password or reusable server-issued bearer credential. Encrypted backup/recovery UI remains launcher work.
10. Existing native character selection returns its separate gateway ticket/key. The broker Session token is **not** a gateway ticket. No gateway edit is needed to accept this W94 Session token: login consumes it. Gateway v2 one-use handoffs remain a separate lead-owned change, described below.

Server setup adds `--account-store <operator accounts.json> --server-id-file <stable raw 32-byte identity file> [--auth-bind 127.0.0.1:20043]` to the existing resource/key/identity/gateway options. Generate/preserve the identity using an OS CSPRNG and communicate its pin through the trusted server profile; do not accept an untrusted advertisement as a pin. The server emits an `auth_listening` event with address and protocol only. This task adds no private-key file format or key-generation UI.

## Limits and deliberate policy choices (R)

- Per broker source IP: `IpLimiter`, burst 20 and refill 20 connections/minute, maximum 1,024 IP keys. **Every connection counts**, including successful and malformed attempts; this is stricter than failure-only accounting. A complete login uses two connections. Tunnel/NAT users share this budget; forwarded-IP fields are never trusted. Idle IP keys are evicted by the existing limiter.
- Global broker admission: existing `Bucket`, burst 20 and refill 2 connections/second. Native token redemption has a separate per-source-IP `IpLimiter` with burst/refill 20/minute and 1,024 keys, applied before account-store I/O. Unauthenticated non-login requests fail before loading account data.
- Account: five failures in a five-minute window lock the handle hash for 60 seconds. Locked attempts do not extend the lock. Recovery resets its bucket; successful authentication clears that account's failures. Unknown hashes receive identical accounting. At most 1,024 account entries; idle entries expire after ten minutes; capacity refuses new entries rather than evicting live locks. Restart clears counters without bypassing signature verification.
- Broker: 16 connections, 192 body bytes/connection, fixed 69-byte reply, three-second total read/write deadline, at most eight accepts and one bounded read/write per connection per iteration. Nonblocking sockets share the SOE loop; no thread pool, web server or async dependency.
- Login UDP: at most 32 peer sessions, handshake IP burst 32/refill 20 per second with 1,024 limiter keys, peer datagram burst 256/refill 200 per second. Thirty-second pre-authentication deadline, 120-second idle timeout, ten-minute absolute lifetime. Exact handshake retries preserve codec/RC4/reliability state; conflicting requests cannot replace an occupied peer. Existing receive/send memory caps remain. Expiry/retransmission polling runs even when admission rejects traffic.
- W83's 256-entry challenge and token caps are preserved. This does not implement the older design's tighter two-pending-grants/account policy. Memory remains bounded; tightening that separate outstanding-grant policy is logged for lead review.

## Provenance: original facts versus reconstruction

No new native fact was recovered in W94 and no community source was copied. The original packet facts below reuse the byte-checked W76 record in `project/notes/Research/Multi Login Design.md:59-71`; W94 did not re-disassemble them.

| Fact | Provenance |
|---|---|
| Native opcode-1 strings mean Session and Fingerprint, not username/password. | `00fa6d10`, member offsets +4/+0x38 and field-name references at `00fa6d2e` / `00fa6d4b`; W76 note line 67. |
| The native request has one type byte and two u32le-length strings without appended terminators. | OnConnect `00fa9620`; binary writer `00fa6800`; string writer `00911c30`; W76 note lines 65-66. Existing independent serializer and new headless route tests preserve that shape. |
| Schema 3, public-key validation, strict Ed25519 verification, entropy and one-use token state already existed. | Current `src/account_store.rs`, `src/login_auth.rs`, `src/bin/osfr-account.rs`; W83 report `lab/runs/luna/W83-20261016T051414Z/final.md` and its `lab/build/w83/W83-evidence.md`. This is current Rust implementation evidence, not an original-client fact. |
| Broker protocol, signature domain/server binding, handles, lifetimes, quotas and adapter policy are reconstructed. | Key-pair decision at Multi Login Design lines 13-22; server policies there and Security Model L2-L4/L6/I10-I12/T4-T6. `(R)` comments identify these rules in source. AUTH2 is W94's exact replacement for the superseded password AUTH1 framing. |

## Gate commands and outputs

Working directory: `project`. Every Cargo invocation used `scripts/cargo.ps1` and `CARGO_TARGET_DIR=%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\w94`.

```text
scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test login --test login_auth --test login_broker --test account --test multiplayer --bin osfr-login --bin osfr-account
osfr-account: test result: ok. 2 passed; 0 failed; 0 ignored
osfr-login:   test result: ok. 4 passed; 0 failed; 0 ignored
account:      test result: ok. 4 passed; 0 failed; 0 ignored
login:        test result: ok. 11 passed; 0 failed; 2 ignored
login_auth:   test result: ok. 10 passed; 0 failed; 0 ignored
login_broker: test result: ok. 5 passed; 0 failed; 0 ignored
multiplayer:  test result: ok. 8 passed; 0 failed; 1 ignored
EXIT 0

scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --lib account_store::tests
test result: ok. 4 passed; 0 failed; 0 ignored; 77 filtered out
EXIT 0

scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --lib net_policy::tests
test result: ok. 3 passed; 0 failed; 0 ignored; 78 filtered out
EXIT 0

scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test login_broker
[final test-client retry and IP-recovery changes]
test result: ok. 5 passed; 0 failed; 0 ignored; finished in 7.16s
EXIT 0
```

Total distinct gates: **51 passed, 0 failed; 3 existing private-oracle/native-content tests ignored**. The last broker rerun replaces its earlier five results; it is not added again to the total. Good signatures, wrong keys, wrong-server signatures, challenge/token replay, exact 60-second expiry, key revocation, unknown/disabled replies, account lockout/recovery, IP refusal/refill, store locking, native malformed bounds, concurrent distinct accounts, unchanged account bytes, occupied-peer protection, dev mode, TCP trickling/expiry and forced UDP loss are covered. Token expiry/account recovery use injected monotonic instants in the core tests; no 60-second wall-clock native-client test was run.

Initial red route test: one failure, `osfr-login: unknown login option --auth-bind`, before broker implementation. Development fixture corrections supplied nonempty synthetic resource tables, used the public reply API and retained the child stdout pipe. An initial `--nocapture` invocation failed because PowerShell argument forwarding removed the delimiter; gate commands above omit it.

Canary: temporarily bypassed only `key.verify_strict(...)`, ran `--test login_auth --test login_broker`, and restored original source bytes in `finally`. Cargo stopped at the failing auth suite:

```text
signature_is_bound_to_server_handle_and_nonce ... FAILED
malformed_lengths_and_malleated_signature_are_refused_without_panicking ... FAILED
weak_keys_and_bad_signatures_never_create_tokens ... FAILED
test result: FAILED. 7 passed; 3 failed
CANARY_EXIT 101
RESTORED_SIGNATURE_CHECK True
```

The restored production files passed the complete scoped gate run. An intermittent native test timeout during a later broker rerun was not conclusively attributed from that original run. Diagnostic rerun passed. Forced loss then reproduced a concrete headless-client defect: it sent reliable data only once and never polled retransmissions while waiting. `client-loss-red.log` records opcode-1 timeout and a trace containing only handshake responses. The client now polls the existing sender on 100 ms receive timeouts, bounded to five seconds/256 packets. Its final route test deliberately drops the first login datagram and passes; no retry of an already consumed application token is manufactured.

Scoped `rustfmt --edition 2024 --check` on the four owned files: exit 0. `git -c core.whitespace=cr-at-eol diff --check -- <owned tracked files>`: exit 0. Independent line-whitespace assertions include the untracked new test. Existing tracked CRLF style was preserved; an unconfigured Git whitespace check flags its CR characters. Final hashes and exact log output are saved beside this report. No crate-wide formatter ran.

## Security-reviewer self-review

Written authorization was W94's local auth/test scope. Applied secure-code-guardian, test-master, Ponytail full and Caveman; no game recon, asset, native RE or automation workflow was needed. Automated source-pattern scanning preceded the final manual authentication, authorization, parsing, I/O, crypto, logging and lifecycle review. `security-scan.txt` records the final production sites. This is a bounded source-pattern audit plus tests, not a dependency-vulnerability database scan or an independent review.

| Severity | Finding | Disposition |
|---|---|---|
| Medium | Admission rejection originally skipped the loop's expiry/retransmission phase. A sustained rejected flood could starve cleanup. | Fixed: `run` no longer continues around cleanup on quota refusal. Manual branch review; full scoped gates passed. No sustained hostile load measurement performed. |
| Medium | Unauthenticated native traffic originally loaded the account store before credential admission. | Fixed: invalid non-login traffic fails without loading it; token attempts are IP-budgeted before store I/O. |
| Medium | Per-session LoginService caches could overwrite another login peer's account mutation. | Fixed for this process's serialized login requests by refreshing existing handlers under the lifetime writer lock. Native/global cross-process L4 acceptance remains lead-owned. |
| Low | W83 used ordinary enrolled-key/hash comparisons and rejected unknown identities before signature work. | Fixed: `subtle::ConstantTimeEq` for fixed-width bindings, full enrolled-key scan; unknown/disabled accounts perform signature verification. The entire endpoint is not claimed constant-time. All network refusals remain generic. |
| Low | Test client did not drive reliable retries while waiting for UDP responses. | Forced-loss red test, then fixed using existing ReliableSender; final real route passes with first datagram dropped. Original intermittent timeout's exact cause remains unproven. |
| Info | Existing LoginService lacks the lead-owned shared-store interface. | Logged: refresh reloads resource tables on each authenticated request. Performance and ten-player abuse/load acceptance are unmeasured. |
| High, pre-existing outside W94 | Current character-selection/gateway ticket policy still includes deterministic v1 and the account-1/character-1 fixed-ticket path. | Logged for lead: this change authenticates native Session only. No claim that L3/T5 gateway replay protection is complete. |

No new unresolved W94 credential-bypass finding was identified. The sole production `unwrap` added to TCP framing converts exactly four bytes only after `input.len() >= 4`; input growth is bounded by the remaining bytes of a 196-byte stack buffer. Fixed proof offsets are accessed only after exact length 134. Native string lengths are checked before slicing. No new `unsafe`, unbounded socket read, password handling, remote-selected path, secret log field, Studio exposure or dependency was added.

## Exact lead-owned gaps

- No `osfr-gateway.rs` edit is necessary for W94 Session redemption. For complete L3, implement the design's separate v2 gateway grants: authenticated ownership, 60-second fixed-field MAC record, atomic one-use consumption before any save load/write, current key/credential binding, typed ownership reservation and removal of the 1/1/v1 bypasses. That work touches shared `login.rs` and the gateway and was not performed here.
- `LoginService::new` still rejects a non-loopback gateway address (`src/login.rs:177-179`). Remote two-PC realm advertisement needs lead-owned M1.1 integration. The broker may now share the VPN address (decision 2).
- Launcher profile/key storage, pin checking, broker invocation and native Session handoff need launcher work. The contract above is concrete and tested headlessly; no launcher was edited.
- Claude's independent review, native token launch/character selection/world entry, native HMAC/v2 gateway acceptance, T4-T6 cross-process/native cases, encrypted-tunnel deployment and I10-I12 abuse/load/security gates remain open. No LAN/internet readiness claim follows from these tests.
- W83's outstanding-grant caps and resource reload cost are explicit policy/performance limitations. The current lock is cooperative serialization, not an OS ACL; all other account-store writers need the same ownership discipline.
- Build Plan, Ledger, Current Progress and MODLOG are shared lead-owned notes and were not edited by this worker.

Ponytail: reused LoginAuth, AccountStore/lock, existing character handlers and packet layouts, net_policy Bucket/IpLimiter, ReliableSender/Receiver, stdlib nonblocking sockets and approved crypto crates. Deliberately did not build custom crypto, another bucket algorithm, a web/async framework, database, launcher, gateway protocol or dependency.
