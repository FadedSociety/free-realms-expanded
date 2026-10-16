# Accounts and Characters

Snapshot: 2026-10-13. W11 adds an account-owned character store and isolated request handlers. This is a Rust candidate slice, not login/gateway integration or native acceptance.

## Recovered client facts

- `ClientAccountManager::OnAccountInfo` at `00719330` copies three server values into membership, maximum-character, and admin fields. The Build Guide locates the callback and its state-machine use at `notes/Build Guides/Gameplay/Character and Customization.md:70-78`. `python -I scripts/native_map/nmq.py show 00719330` confirms the three destination fields and logs. No underage or open-chat value appears in this account-info record.
- The client stores multiple character rows. `CharacterSelectInfoReply` contains `Status`, `CanBypassServerLock`, and an `Entities/RowData` list (`00fa7730`, `00fa6e30`, `00fa61e0`). Each row has a 64-bit `EntityKey`, `ServerId`, `Status`, and base64 `Payload` (`00fa4370`). The guide records these readers and `OnRequestCharacterList` at `00719550` in `.../Gameplay/Character and Customization.md:237-242`. The native-map reader for `00fa7730` is confirmed; `00fa4370` is likely and names each field.
- `CharacterCreateReply` carries `Status` and 64-bit `EntityKey` (`00fa5ce0`). Client create statuses are 1 success, 2/3 failure, 4 name taken, 5 temporary name taken, 6 too many characters, and 7 invalid name (`.../Gameplay/Character and Customization.md:243`, `:285-290`). The client request record is four strings and 21 signed integers (`:172-184`).
- Name-check request 210 has five strings; reply 211 has first and last names, an `i32` result, and suggestion rows. `00710e20` handles results 1, 2, 3, 5, 6, 7-10, -1, and 4. Result meanings and client-facing thresholds are in `.../Gameplay/Character and Customization.md:249-251`. The server-side validation rules remain unverified by the client.
- Delete reply reader `00fa5e50` reads `EntityKey`, `Status`, and `Payload` through `00fa43e0`. `nmq dis 00fa5e50 96` confirms the reader builds event 10 and calls the record reader. `007198b0` is still an unknown-labeled callback; its bytes were inspected, but its success policy is not treated as recovered.
- Character-login client statuses are 1 success, 2 account in use, 6 unable to load, and 7 unable to assign a gateway (`.../Gameplay/Character and Customization.md:245`).
- `MaxCharacters > 1` opens the add-character flow; the supplied maximum is server data and its original value is unknown (`.../Gameplay/Character and Customization.md:32`, `:290`). Existing `notes/Research/Pieces/095 Multi-character persistence.md` separately records the prior single-character save boundary and its local migration policy.

Native-map queries used the existing read-only `lab/native-map/run/map.sqlite` data through `project/scripts/native_map/nmq.py`. Confirmed labels were checked with `show`; unknown labels for delete/name request functions were checked with `dis` before use. No decompiled source or raw client asset was copied into the repository.

## W11 model and reconstructed policy

- `Account` stores a monotonic local account ID, a 32-byte login-name hash, client membership flag, reconstructed underage/open-chat booleans, slot policy, and ordered character GUIDs. Raw login names are not persisted.
- `CharacterRecord` holds the existing validated `CharacterIdentity` plus optional `CharacterState`. The old identity validator requires GUID 1, so the stored identity keeps that local sentinel; each reply clones it and substitutes the allocated character GUID.
- `AccountStore` has JSON schema version 1, a persisted account-ID counter, and a global character-GUID counter. Both counters advance monotonically and are validated against existing rows. Account and character ownership references are checked on load/save.
- Slot maximum, character/account ID allocation, case-insensitive global name uniqueness, and delete status values 1/2 are reconstructed policy (R). The configured maximum must be supplied by the login integration; no historical slot count is claimed.
- Name length checks use the client message thresholds: first and last 2-14 characters and 15 combined. Profanity status 3 is represented, but no profanity list or native server filter was recovered. Name suggestions are currently empty because their successful-result wire encoding is not established here.
- The list-row envelope follows the client fields. Its opaque character payload reuses the existing local `login::selection_reply` appearance bytes and base64-encodes them. That payload's field schema is not independently proven by the current client reader. Character create calls `Resources::resolve`, the same resource-backed validation path used by `login.rs`; it does not duplicate creator table checks.
- Store writes call `character_store::replace_bytes`, retaining its synced same-directory temporary file, atomic rename, and Windows retry behavior. Migration reads schema-versioned `CharacterState` plus the existing identity file and leaves both legacy files unchanged.

## Lead integration steps

1. In a lead-owned integration change, register `account` and `account_store` in `prototype/osfr-tools/src/lib.rs`.
2. Extend `LoginService` in `src/login.rs` to load the account store using an authentication-derived login-name hash. On first use, call `AccountStore::load_or_migrate` with the existing character save and identity paths. Route character-list, name-check, create, delete, and character-login requests through these pure handlers; save the account store after successful create/delete mutations.
3. Keep slot maximum and the underage/open-chat interpretation explicit configuration/policy (R). Populate account-info membership, maximum-character, and admin values from account state when that response is added.
4. At successful select, bind the trusted session to `(account_id, character_guid)`. Pass that binding to the gateway, validate ownership again on entry/reconnect, and load the selected character's world state. Clear the binding when the session ends or the character is deleted. Do not accept a client-supplied account ID as authority.
5. Coordinate account-store writes with per-character gameplay saves before claiming transactional deletion or creation. Then run the two-character list/create/select/delete flow against the native client and verify the named reply readers and opaque payload; W11 tests do not establish either gate.

## W11 artifacts and checks

- `prototype/osfr-tools/src/account.rs`: account/flags/slot models and pure list, name-check, create, delete, and select/bind handlers.
- `prototype/osfr-tools/src/account_store.rs`: versioned store, monotonic allocators, atomic save/load, and legacy adoption.
- `prototype/osfr-tools/tests/account.rs` and `tests/fixtures/account/`: synthetic account, two-character, ownership, status, reload, and migration checks.

Red/green commands from the repository root, with `CARGO_TARGET_DIR=%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W11`:

| Command | Result |
|---|---|
| `& .\scripts\cargo.ps1 test --manifest-path prototype\osfr-tools\Cargo.toml --test account` before modules existed | RED: missing `tests\..\src\account.rs`, exit 1. |
| Same command after implementation | GREEN: 5 passed, 0 failed. It also ran the included `character_store` atomic-failure unit check. |
| Private toolchain `rustfmt.exe --edition 2024` on the two modules and `tests/account.rs` | Formatting completed; only owned Rust files were passed. |

The test run emitted existing `housing.rs` dead-code warning and expected unused-variant warnings for statuses that the pure handlers expose for later integration. No crate-wide formatting, native client, or gateway test was run.

## Remaining gates

- `login.rs`, `gateway.rs`, and `lib.rs` still use the existing single-identity flow; W11 modules are intentionally not registered.
- Account-info reply construction, server-authentication lookup, actual slot settings, profanity policy, suggestion encoding, and underage/open-chat semantics need lead decisions and client-backed evidence where available.
- Native reply framing and the inner character payload remain unverified. The local `login.rs` helpers have legacy serializer fixtures; those fixtures do not prove current native client acceptance.
- Migration is tested only with synthetic v2 character-state and v1 identity documents. A production migration must identify the active save/identity pair and coordinate gameplay ownership without rewriting the originals.
- Two-character native creation, selection, deletion, reconnect, gateway ownership, and cross-file crash recovery remain untested.

## Skills and method

- RTK: ran normal commands first; used its condensed output as returned.
- Ponytail: reused `Resources::resolve` and the existing atomic replacement helper; no dependency or shared-file changes.
- Caveman: concise work updates; persisted evidence and gaps remain explicit.
- `reverse-engineering`: read the existing native-map function facts and inspected unknown-labeled functions with `nmq dis`.
- `game-recon`, `game-research-websearch`, and `asset-pipeline` were not needed for this code-only handler/store slice.
