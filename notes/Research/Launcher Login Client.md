# W100 launcher key-pair client

2026-10-16. **Implemented, acceptance blocked.** This is a scoped worker handoff,
not an integration, native acceptance or LAN/internet-readiness claim. No native
client was started. No prototype, save, UI source, shared project-status note,
release binary or unrelated worker edit was changed.

## Owned artifacts

- `src/auth.rs`: trusted per-server profiles, profile-directory/key storage,
  OS randomness, public-key export, pin-before-sign AUTH2 client, typed errors,
  zeroizing buffers and SessionToken drop.
- `src/lib.rs`: module declaration and `key_command(args)` only, plus its module
  documentation. Existing endpoint, file-safety and SessionToken helpers reused.
- `Cargo.toml`, `Cargo.lock`, `DEPENDENCIES.md`: only the four authorized direct
  dependencies and their lockfile closure/licences. No existing locked package
  version changed.
- `tests/auth.rs`: six process-isolated auth integration tests, including a real
  broker/operator enrolment test. Each child receives scratch APPDATA; the real
  user's profile and game folders are never used.
- `tests/support/auth_wire.rs`: three socket/validation tests included in the
  library test binary. They require no access to private profile files.
- `tests/auth_canary.py`: identity-pin mutation runner. A failing baseline blocks
  mutation. It restores its owned source and refuses unexpected concurrent edits.
- This handoff report. Raw command output is in `lab/build/w100/`.

## API and CLI handoff

```rust,ignore
use osfr_launcher::{auth, key_command};

// Trusted operator/tunnel configuration; obtain the pin out of band.
let profile = auth::Profile::new(
    "realm-id", "Realm label", "127.0.0.1:20043", trusted_identity, account_handle,
)?;
auth::save_profile(&profile)?; // New profile only; reuses an existing CLI key.
let profile = auth::load_profile("realm-id")?;
let token = auth::login(&profile)?; // Existing osfr_launcher::SessionToken.

// Lead-owned main.rs early dispatch, before starting the GUI:
let command = key_command(std::env::args().skip(1));
```

`key_command` returns `None` for ordinary UI arguments. For `--new-key <id>` and
`--print-public-key <id>`, it returns `Some(Result<String, String>)`. Its sole
successful output is the public key in lowercase hex. The lead's entry hook must
print that public value or the generic error, then exit. **That hook is not
currently in main.rs**, which this task explicitly forbids editing. Both the
hook and the UI LoginProvider wiring remain lead-owned; no callable executable
CLI or authenticated Play-button claim is made here.

Profiles live exclusively at `%APPDATA%/FreeRealmsExpanded/profiles/<id>/`:
`profile.json` is bounded trusted metadata and `key.ed25519` is a raw 32-byte
seed. Missing/relative APPDATA has no executable-directory fallback. Keys are
created exclusively, never silently replaced or rotated. Existing metadata must
match the requested profile before signing. A profile ID cannot contain path
syntax or Windows device names. Symlinks, junctions and hard-linked files are
rejected using the existing launcher helpers.

Windows objects receive protected OWNER RIGHTS DACLs at creation, with no broad
inherited ACEs. Existing DACLs must match the expected private DACL. The Unix
branch uses 0700 directories and 0600 files, but it was not compiled/tested here;
the assigned target is x86_64-pc-windows-gnullvm. OS profile paths, the logged-in
OS user and its trusted launcher UI are local trust boundaries. A compromised
same-user process or administrator is outside this storage boundary.

AUTH2 uses literal loopback broker/tunnel addresses, connect timeouts of 2 s and
absolute read/write exchange deadlines of 3 s. DNS is deliberately excluded.
Every proof uses a new connection and is never retried. Refused, unknown,
disabled and locked accounts share `AuthError::Refused` and one generic message.
The wire provides no distinct lock status; no account-existence inference is
invented. A connected server closing without a reply maps to `TryLater`.

Seeds and all replies are zeroizing. The locked feature graph also enables
dalek's signing-key zeroization through `std -> alloc -> zeroize`. The existing
SessionToken now zeroizes its string on drop; it has no Debug/Display. The
existing launch code formats its native SessionId argument and drops the
original token. Its necessary argument/OS/client copies remain the existing
native handoff boundary; no bearer credential is persisted or logged here.

## Provenance

**No original-client fact was newly recovered or claimed by W100.** Everything
in this authentication implementation is reconstructed policy (R), marked in
the module and relevant storage/network rules. No community source was copied.

- Exact AUTH2 framing, statuses, offsets, signing transcript, 60 s challenge,
  two-connection proof and token semantics: `notes/Research/Key-pair Login
  Contract.md`, "Exact launcher contract", lines 24-39.
- Independent reference client and synthetic resource/startup recipe:
  `prototype/osfr-tools/tests/login_broker.rs:37-231`; operator flags inspected
  directly in `src/bin/osfr-account.rs:14-149` of that crate.
- Per-server key and trusted-pin decision: `notes/Research/Multi Login Design.md`,
  lines 13-22. Its later password text is superseded, not implemented.
- Local exposure, secret storage, generic failures and protected transport:
  `notes/Design/Security Model.md`, L1-L3, L8, I10-I12, read alongside the newer
  key-pair decision and exact contract. The old password/Argon2 proposal is not
  the current contract.
- The contract references existing native Session/Fingerprint evidence at
  `00fa9620`, `00fa6800`, `00911c30`, `00fa6d10`, byte-checked in W76 at
  `Multi Login Design.md:59-71`. W100 did not re-disassemble them and adds no
  native-packet builder; it returns the launcher's existing SessionToken.

## Gate commands and evidence

Commands ran from `project`. Every launcher build used `launcher/build.ps1` and
its gnullvm target. `CARGO_TARGET_DIR` was explicitly set to
`../lab/build/w100/launcher`; server binaries went to `../lab/build/w100/server`.

```powershell
./scripts/cargo.ps1 generate-lockfile --offline --manifest-path launcher/Cargo.toml
# exit 0: only approved crypto packages/closure added.

$env:CARGO_TARGET_DIR = [IO.Path]::GetFullPath('../lab/build/w100/server')
./scripts/cargo.ps1 build --offline --locked --manifest-path prototype/osfr-tools/Cargo.toml --bin osfr-login --bin osfr-account
# exit 0: Finished dev profile in 1m 05s; 9 existing server-library warnings.

$env:CARGO_TARGET_DIR = [IO.Path]::GetFullPath('../lab/build/w100/launcher')
./launcher/build.ps1 test
./launcher/build.ps1 build
python -I launcher/tests/auth_canary.py
```

PowerShell 5's native stderr redirection interacted with the wrapper's Stop
policy. Logged build runs therefore used Python `subprocess.run` to invoke the
same unmodified build.ps1, with stdout/stderr captured together. No alternate
Cargo build path was used. `red-tests.log` records the pre-implementation
unresolved `auth` / `key_command` imports. `first-tests.log` records compilation
without launcher warnings, 8 existing library and 5 existing GUI tests passing,
then all 6 new integration cases failing at private-file access.

The current library and integration executables were subsequently rebuilt by
build.ps1 test (the build failed only in the concurrently edited GUI). Running
those exact compiled artifacts directly produced:

```text
lab/build/w100/library-tests.log:
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

lab/build/w100/auth-tests.log:
running 6 tests
test result: FAILED. 0 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out
All six stop at AuthError::KeyStorage before their broker/signature assertions.

lab/build/w100/canary-baseline.log:
baseline build/test failed; no mutation performed.
BLOCKED: baseline failed; pin canary was not run. See canary-baseline.log.
```

The 11 green library tests include all 3 new checks: fragmented fixed-reply
framing/refusal/invalid status/disconnect; trickled bytes cannot extend the
absolute deadline; invalid profile paths, non-loopback/DNS endpoints and bad key
command arity fail closed. They do not establish the storage/login integration.

### Precise blockers

1. **Restricted Windows token versus owner-only DACL.** A direct Win32 probe
   returned `SetFileSecurityW = 0, GetLastError = 5` (access denied). Creating an
   object with the private descriptor succeeds, but this restricted process
   cannot reopen/read/create children through its OWNER RIGHTS-only DACL.
   The final implementation creates private objects atomically and fails closed;
   no sandbox identity, shared group or permissive fallback was added to the
   ACL. A handle obtained when creating a zero-content probe file successfully
   reported canonical `D:P(A;;FA;;;OW)`, confirming the expected file descriptor
   spelling. **Private-file round trip/permissions, actual login/profile route,
   real-broker enrolment, wrong-pin no-proof, refused-proof and login timeout
   integration tests need a normal user-token run by the lead.**
2. **Concurrent UI edits.** `build.log` initially found `theme.rs` missing.
   `canary-baseline.log` subsequently found 11 GUI errors and 6 warnings in an
   unfinished theme/main API transition. After main.rs changed, the latest
   `final-build.log` reports 3 GUI errors and 7 warnings: `egui::ShapeIdx` is
   unresolved, `ColorImage::source_size` is missing, and the Launcher::new call
   lacks two new arguments. These files belong to another worker and were
   preserved. The later `final-tests.log`, after another main.rs update, reports
   the remaining 2 theme errors and 6 warnings. The mandatory warning-free full
   build/test gate remains open.
3. **CLI entry hook.** Main dispatch and authenticated UI integration cannot be
   supplied without editing the explicitly forbidden `src/main.rs`. The tested
   library handler is ready for the lead; the executable does not yet route it.

No valid pin-disable canary or full green build/test result is claimed. The
canary runner requires the baseline to pass before mutation. The server binaries
exist; no scratch broker survived a test (the failing integration did not reach
broker startup). Private-ACL zero-content probes at `lab/build/w100/acl-create-probe`
and `acl-file-probe`, plus failed isolated test directories, were retained because
the restricted token cannot remove the private probes. They contain no client
private keys or bearer tokens; cleanup needs the normal user token.

## Security-reviewer self-review

Written authorization is W100's launcher-only local test scope. Applied
secure-code-guardian, test-master and security-reviewer; no game recon, asset,
native reverse engineering, game automation or delegation was needed.
`security-scan.txt` records the bounded automated production pattern scan before
manual review of profile/path boundaries, crypto, framing, timeouts, errors,
logging, zeroization and Windows FFI/resource cleanup. Auth production has zero
unwrap/expect/panic/printing/log calls. This is a source-pattern self-review,
not an independent audit or a vulnerability-database dependency scan.

| Severity | Finding | Disposition |
|---|---|---|
| Medium | A malformed key read through the ordinary shared file helper could drop bytes before a zeroizing wrapper was established. | Fixed: read at most 33 bytes directly into a preallocated Zeroizing buffer, avoiding reallocations; exact length must be 32, including on error paths. |
| Medium | Post-creation permission hardening left avoidable empty-object ACL transitions and failed under the restricted token. | Fixed design: attach protected ACLs at creation; verify existing ACLs without changing them. Sandbox-dependent integration remains blocked, not relaxed. |
| Low | Per-read timeouts alone permit a trickled response to extend login indefinitely. | Prevented with one absolute exchange deadline; independently tested with 100 ms trickled bytes. |
| Info | Wire replies cannot distinguish lockout from unknown/disabled/wrong-key refusal. | Shared generic Refused message; TryLater only on connection closure. No existence-sensitive error mapping. |
| Info | Native handoff necessarily copies the bearer into the existing launch arguments. | Original token/reply buffers wipe on drop. No disk/log persistence added; downstream OS/client copies remain the existing boundary. |
| Integration | Full private-storage/signing/CLI/native acceptance remains unverified. | Explicit blockers and runnable gates above; lead review required before integration. |

No new unresolved credential-bypass finding was identified by source review.
That statement does not override the missing runtime gates. Same-user/administrator
filesystem races are outside this local storage boundary; untrusted network
traffic chooses neither file paths nor enrolled profile/pin/key bindings.

Ponytail: reused the existing SessionToken, endpoint validator, bounded metadata
reader and symlink/hardlink checks, stdlib TCP, existing serde_json and the approved
crypto crates. Deliberately did not build custom crypto, DNS/HTTP/TLS/async
layers, a database, key rotation/recovery UI, extra dependencies or UI changes.
