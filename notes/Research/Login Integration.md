---
type: research
updated: 2026-10-13
scope: W55 login/account integration and finished-module registration
---

# Login Integration (W55)

## Result

`LoginService` now routes character-list, name-check, create, delete, and character-login requests through the W11 account handlers. Accounts are keyed by SHA-256 of the accepted local login name. First use migrates the existing save and identity through `AccountStore::load_or_migrate`; migration uses a temporary `CharacterState` projection for schema-5/6 gateway saves, and leaves both source files byte-identical. Account writes are staged and saved after successful create/delete.

The gateway resolves the selected GUID in its own `AccountStore`, checks that account's ownership and the login-issued binding ticket, and loads `characters/<guid>.json`. It does not accept a client-supplied account ID. On first adoption of the legacy single-character save, the gateway validates the snapshot and copies the complete file to the per-character path, retaining optional gameplay, housing, and pet fields. Session reset and deletion clear the selected binding.

The legacy one-character list row, character-select reply, and gateway-enter reply remain covered by the existing serializer fixtures and selected-character tests. The successful login/account-info payload now reports the configured six-character maximum so the client can expose character creation. The private startup oracle remains byte-identical.

`chess`, `guilds`, and `matchmaking` are registered in `src/lib.rs`; their tests now import the crate modules. This is API registration only. No gateway or gameplay routing was added for those services.

## Recovered client facts and provenance

- `ClientAccountManager::OnAccountInfo` at `00719330` copies membership, maximum-character, and admin values into client state. The client record contains no underage/open-chat values. Provenance: `notes/Research/Accounts and Characters.md`, Recovered client facts; native-map `show 00719330`; Build Guide `Gameplay/Character and Customization.md:70-78`.
- The character-list client stores multiple rows. `CharacterSelectInfoReply` and its row readers are at `00fa7730`, `00fa6e30`, `00fa61e0`, and `00fa4370`; `OnRequestCharacterList` is at `00719550`. The row carries a 64-bit `EntityKey`, server/status fields, and an opaque payload. The `00fa4370` label is likely, so its field names are not treated as confirmed. Provenance: `Accounts and Characters.md`; `Gameplay/Character and Customization.md:237-242`.
- `MaxCharacters > 1` opens the add-character flow, but the historical maximum is unknown. Provenance: `Accounts and Characters.md`; `Gameplay/Character and Customization.md:32, 290`.
- The create reply carries status plus 64-bit entity key (`00fa5ce0`); client statuses include success 1, name taken 4, too many characters 6, and invalid name 7. Name-check request/reply IDs are 210/211, and `00710e20` handles the client result values. Provenance: `Accounts and Characters.md`; `Gameplay/Character and Customization.md:172-184, 243, 249-251, 285-290`.
- The delete reply reader at `00fa5e50` reads an entity key/status/payload through `00fa43e0`; disassembly confirms it builds event 10. Callback `007198b0` remains unknown-labeled, so its success policy is not claimed as recovered. Character-login client statuses include 1, 2, 6, and 7. Provenance: `Accounts and Characters.md`; native-map `dis 00fa5e50 96`; `Gameplay/Character and Customization.md:245`.
- The existing gateway runner's account-store mode resolves GUID ownership and keeps per-character saves beside the configured state. This is prior Rust integration evidence, not recovered client authentication behavior. Provenance: `notes/Research/Multiplayer Routing.md`, W35 implementation.
- Chess packet/movie facts are recorded in `notes/Research/Chess Service.md`; guild family-129 layouts in `notes/Research/Guilds.md`; matchmaking families 141/175 in `notes/Research/Matchmaking Service.md`. W55 adds no new client fact for those modules.

## Reconstructed policy and configuration (R)

- `--max-characters` defaults to 6. The configured value is persisted in a newly created account's slot policy. Six is a local setting; it is not a recovered historical slot count.
- `--underage` defaults to false and `--open-chat` defaults to true. These booleans are local policy because the native account-info record does not provide them.
- Admin status defaults to false in the account-info reply; no admin field is inferred from the local login name.
- The login service accepts a configured local login name, not a recovered external authentication flow. It stores only the SHA-256 login-name hash, never the raw name.
- Login-to-gateway binding uses a private shared HMAC key from `--handoff-key-file`. The ticket binds account ID, character GUID, and login-name hash. The gateway derives ownership from its store and validates the ticket; the client does not supply account ID authority. A fixed configured gateway ticket remains the compatibility route for account 1 / character 1.
- Account/GUID allocation, case-insensitive global name uniqueness, local delete status behavior, and the slot limit are W11 reconstructed policy. The temporary schema projection is a migration adapter, not a claim about a native account-store format.
- Per-character save filenames and the initial full-snapshot copy are local persistence policy. Cross-file crash atomicity between account-store mutation and gameplay saves is not established.

## Verification

Working directory: `project/prototype/osfr-tools`.

| Command | Result |
|---|---|
| Private `rustfmt.exe --edition 2024 --config skip_children=true` on W55-owned Rust files only | Passed. No crate-wide formatting was run. |
| `& '..\..\scripts\cargo.ps1' test --test login --test piece_050 --test multiplayer --test chess --test guilds --test matchmaking --no-fail-fast` | Passed: chess 12/12, guilds 11/11, login 11 passed / 2 ignored, matchmaking 11 passed / 1 ignored, multiplayer 1 passed / 1 ignored, piece_050 10/10. Total 56 passed, 4 ignored. |
| Private startup oracle with `OSFR_GATEWAY_REFERENCE` and `OSFR_GATEWAY_CAPTURE` set to the preserved paths, then `& '..\..\scripts\cargo.ps1' 'test' '--test' 'gateway' '--' '--ignored'` | Passed: 2 passed, 0 failed, 8 filtered. Captured startup sequence remains byte-identical. |
| `& '..\..\scripts\cargo.ps1' build --release --bin osfr-gateway --bin osfr-login` | Passed in the default target directory; both release binaries built. Existing dead-code warnings in chess/housing only. |
| Earlier full `& '..\..\scripts\cargo.ps1' test --no-fail-fast` run | Exit 1 after 1,970 passed, 11 failed, 91 ignored. Four other-owner targets failed: `content_npc_placement` (1), `content_npcs` (1), `pets` (8), `progression_extras` (1). The two NPC targets rejected concurrent JSON classification values; pets rejected a new `R` source value; progression's source-content count changed from 595 to 649. Logs: `%TEMP%\osfr-w55-full-suite.stdout.log` and `.stderr.log`. |
| Intermediate full-suite rerun | Initially blocked before test execution by `E0277` at other-owner `tests/movement.rs:337` (`[usize; 256]` lacked `Default`). The movement owner changed those counters to maps; no W55 or movement-path edit was made here. |
| Latest full `& '..\..\scripts\cargo.ps1' test --no-fail-fast` rerun | Exit 1 after 1,971 passed, 15 failed, 91 ignored. Three other-owner targets failed: `movement` (8 passed / 2 failed), `npc_interaction` (4 passed / 5 failed), and `pets` (1 passed / 8 failed). The earlier content-NPC and progression-extras failures no longer reproduced. All other executed targets reported 0 failed. Logs: `%TEMP%\osfr-w55-final2-suite.stdout.log` and `.stderr.log`. The full-suite gate remains open for these concurrent-owner failures. |

Latest full-suite per-target results (all unlisted executed targets had 0 failed):

| Target | Passed | Failed | Ignored |
|---|---:|---:|---:|
| `tests/login.rs` | 11 | 0 | 2 |
| `tests/chess.rs` | 12 | 0 | 0 |
| `tests/guilds.rs` | 11 | 0 | 0 |
| `tests/matchmaking.rs` | 11 | 0 | 1 |
| `tests/multiplayer.rs` | 1 | 0 | 1 |
| `tests/piece_050.rs` | 10 | 0 | 0 |
| `tests/soccer.rs` | 11 | 0 | 1 |
| `tests/soccer_match.rs` | 27 | 0 | 0 |
| `tests/soccer_native_wire.rs` | 8 | 0 | 0 |
| `tests/soccer_review.rs` | 10 | 0 | 0 |
| `tests/soccer_service.rs` | 16 | 0 | 0 |
| `tests/movement.rs` | 8 | 2 | 0 |
| `tests/npc_interaction.rs` | 4 | 5 | 0 |
| `tests/pets.rs` | 1 | 8 | 0 |

## Native acceptance plan and remaining gates

Run against a disposable account store and two-character test account in the native client:

1. Log in and create two characters; verify the create status and both rows in character selection.
2. Select each character in turn and enter the world. Verify each gateway binding is accepted only for its selected GUID and that each character loads its own save.
3. Delete one character, log out, and log in again. Verify only the remaining character is listed, the deleted GUID cannot be selected, and the remaining character still enters with its saved state.
4. Preserve the client version, request/reply capture, per-character save hashes, and account-store hash. Do not use a real player account or existing campaign save.

This native two-character flow was not run. Also open: a clean full-suite run after the movement, NPC-interaction, and pets owners resolve their target failures; native validation of the account-info payload and character-list opaque row payload; recovery of original slot/chat policies; and transactional recovery across account-store and gameplay-save writes.
