---
type: design
task: W86
updated: 2026-10-16
status: W86b build, tests and window captures proven; full launcher partly proven
plan: Build Plan 3.5 and 1.7
---

# Rust Launcher

W86 supplies a standalone `project/launcher/` crate and the toolkit decision.
**W86b now builds and opens the Rust UI on this Windows machine.** Offline build,
12 tests and both running-window screenshot gates pass; see the W86b evidence
below. W86's initial cache failure is retained as historical evidence. No game
client was launched. This is not full acceptance of Build Plan 1.7 or 3.5.

## Toolkit decision

Choose **egui/eframe 0.33.3**, with `default-features = false` and only `glow`,
`default_fonts` and `accesskit`. Use gilrs 0.11.0 with its XInput backend.
This is a native Windows window with the existing repository banner, green Play
button, cream panels, server rows, settings and visible focus. No web runtime,
art generation or replacement asset format is needed.

| Candidate | Windows package size considerations | Controller navigation | Flash-portal skinning | Licence |
|---|---|---|---|---|
| egui/eframe | Native executable. Select OpenGL and omit WGPU; include fonts and the existing 2,329,331-byte PNG. Actual release size unmeasured. | gilrs actions can use existing egui focus and Space activation. No new focus manager. | Existing PNG plus coloured widgets/strokes/radii; custom painting is available if later art demands it. | MIT OR Apache-2.0 |
| iced | Native executable; WGPU or tiny-skia renderer selection affects its dependency closure. No same-settings Windows build was measured, so no numeric or relative size claim. | gilrs requires an event/subscription adapter and application focus actions. | Widget themes or custom drawing can implement the look. More retained message/state plumbing for this small screen (design judgement). | MIT |
| Slint | Native compiled UI; software rendering is an option. It could be compact, but that was not measured here. | Bridge gilrs into Slint's event loop and focus/key APIs. | Declarative custom graphics suit the look, at the cost of a second UI language and build step. | GPLv3, royalty-free desktop licence with attribution conditions, or commercial terms |

The choice is based on the smallest understandable implementation, not an
unmeasured claim that egui has the smallest binary. W86b produced a debug GUI
binary; no release-package measurement or three-toolkit benchmark was made.
Release settings use size optimization, thin LTO and stripped symbols; measure
the actual Windows artifact once its dependencies can be fetched.

Primary sources inspected for this decision:

- [eframe 0.33.3 manifest](https://raw.githubusercontent.com/emilk/egui/0.33.3/crates/eframe/Cargo.toml): selectable OpenGL/WGPU, fonts and accessibility features.
- [iced renderer documentation](https://docs.iced.rs/iced/enum.Backend.html) and [licence](https://github.com/iced-rs/iced/blob/master/LICENSE).
- [Slint licence terms](https://slint.dev/terms-and-conditions): inspect the applicable terms before distributing any Slint alternative.
- [gilrs 0.11 documentation](https://docs.rs/gilrs/0.11.0/gilrs/) and [features](https://docs.rs/crate/gilrs/0.11.0/features): Windows controller input and XInput selection.

### Ponytail ladder applied

1. The requested skeleton needs a UI and client preparation, not a patcher,
   downloader, news service, account system or new server route.
2. Reuse the existing banner and `gamepad.json`; port the inspected minimal
   launcher's preparation and launch behavior. Do not wrap or execute Python.
3. Use std for files, bounded reads, argument parsing, processes, endpoints and
   the single preparation thread/channel. One existing Kernel32 query supplies
   the hard-link count; no Windows wrapper or regex crate is needed in the core.
4. Reuse toolkit focus, widgets and screenshot support. No separate navigation,
   rendering, worker-pool, configuration or screenshot framework.
5. Reuse the project's serde family for the controller JSON. Add eframe for the
   required UI, gilrs because the owner requested it, and PNG-only image support for
   the existing art and renderer capture. Direct dependencies and licences are
   in `launcher/DEPENDENCIES.md`; full transitive/font inventory remains open.

## Single launch implementation

`launcher/src/lib.rs` is the Rust successor to `scripts/launcher/friend_launcher.py`.
Play calls `LaunchPlan::build`, `apply`, then `launch`. It does not invoke the
Python launcher or execute `FreeRealms.bat`; it parses the one executable command
as data and spawns the supplied EXE directly with `Command::args` and its client
folder as the working directory.

The inspected Python launcher operates on an existing client folder. Contrary
to the plan's broad "client copy" wording, it does not copy a full installation.
The skeleton likewise asks for a separate, writable copy already provided by the
user. No client installation, download or patch subsystem was invented for W86.

Preserved steps:

- Validate endpoint and client files, EXE MZ signature and locale pair.
- Read ClientConfig, write Baseline with the selected Server, sink HTTP(S)
  addresses to `127.0.0.1:9`, and sink the legacy lvsadp host.
- Set window width/height, Windowed mode, Maximized=0 and Master=0.000000.
- Optionally overlay the two supplied en_us locale files, bounded to 64 MiB each.
- Back up replaced files under `.friend-launcher-backups` before any writes.
- Filter old GUID/ticket/key/Session/server arguments; append controlled Server,
  SessionId, locale 8 and offline account-management URL overrides.
- Launch the local EXE without a command shell or console window.

Intentional improvements over the reference: bounded UTF-8 configuration reads,
junction/reparse rejection, atomic per-file replacement, stale-plan detection,
best-effort rollback on normal write failures, and exactly one locale override.
The small argument reader accepts the actual supplied batch syntax and quoted
values; shell expansion, metacharacters and unsupported argument names fail
closed. It is intentionally not a batch interpreter. Backup directory names use
an epoch-nanosecond/PID suffix instead of the Python UTC timestamp format.

Python files are outside worker ownership and were not edited. At lead
integration, treat the Python launcher as a frozen parity reference and retire
its user entry point after Rust launch acceptance; there must not be two
maintained preparation implementations. That consolidation gate is still open.

## UI, controller and login boundary

Home contains the supplied banner, selectable address rows, a bounded Add realm
field, a clearly labelled status/player-count placeholder and Play. Settings
contains client/overlay paths, window size presets and dimensions, and the
controller-navigation toggle. Settings and custom server entries are session
only; no profile/config persistence was added. Preparation runs in one thread
so locale I/O does not freeze the window; duplicate Play is disabled.

`src/controller.rs` embeds the existing JSON at compile time. Its 0.25 deadzone
and A=Space, B=X, LB=Tab, Start=Escape bindings supply contextual launcher roles:
left stick/D-pad moves focus, A selects, B returns, LB advances and Start toggles
Settings. Hold repeat is reconstructed launcher policy (350 ms then 120 ms).
Input is consumed only while this window is focused and enabled; no SendInput,
game input injection or global hotkeys exist in this crate.

This is **launcher navigation**, not a port of the in-game camera/mouse bridge.
That existing Python bridge is not started by this launcher. Its right-stick
camera, LT cursor mode and gameplay keys remain a separately owned port/in-game
acceptance task. The toggle is labelled accordingly. Physical controller input,
focus rings and layout still need a running-window review.

Login is explicitly labelled **local stub / W83 pending**. `LoginProvider`
returns an opaque `SessionToken` without Debug/Display. `LocalLoginStub` accepts
literal loopback only, with no DNS lookup, using the existing compatibility
SessionId. Remote authentication returns an error before writes. There is no
password form, key store, enrolment, signing code, broker or network connection.
W83 can supply the trait object and return only a short-lived native Session;
the token must never be printed or included in ordinary errors. The future
native client command line still exposes its short-lived bearer to local users
who can inspect processes; key-pair login does not remove that limitation.

`--screenshot <unused.png>` requests the running window's own egui framebuffer,
saves PNG and closes. Capture mode disables Play both in the widget and in its
handler. It has a 15-second deadline and cannot launch the game. This code path
is now verified by W86b's Home and Settings captures below.

## Provenance and reconstruction

No new native functions, packets or server rules were recovered in W86. This is
a launcher implementation, not a native behavior investigation. All new UI,
settings bounds, navigation timing and backup/rollback rules are reconstructed
launcher policy; no historical launcher fidelity is claimed.

| Fact used | Provenance |
|---|---|
| Existing preparation, settings, backup and direct launch behavior | `scripts/launcher/friend_launcher.py:38-52,54-111,114-178,181-232,246-275`; inspected source, not a newly recovered client fact |
| Existing controller layout | `scripts/launcher/gamepad.json:1-26`; `gamepad_bridge.py:55-97,183-199,227-276`; input-injection behavior is reference only, not copied into this skeleton |
| Existing project banner/art direction | `README.md:1-3` and `docs/images/banner.png`; used unchanged, no raw client asset copied into Git |
| Per-server Ed25519 design, private key remains local, Session handoff | `notes/Research/Multi Login Design.md:16-24`; W83 interface requirement, not implemented authentication |
| Native login record is Session/Fingerprint | Prior W76 research table in `Multi Login Design.md`, native addresses `00fa6800` and `00fa6d10`; not re-disassembled for W86 and not needed for a new packet writer |
| One-account native select/world entry is established, not multi-login/key-pair acceptance | `notes/Reviews/Native Login Accounts 023623.md:3-24`; latest Round5 review and current `login.rs` equality route also inspected |
| Windows hard-link count | [GetFileInformationByHandle](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfileinformationbyhandle), [BY_HANDLE_FILE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/ns-fileapi-by_handle_file_information); native API, not game reverse engineering; exercised by the hard-link test |

Reference SHA-256 at inspection:

```text
friend_launcher.py C903542F4CF2E2E93E3EB39994D13CA3183EE6A8C4AD8B9F6511E8767BC487E5
gamepad.json D2829F229DDCA5A1DE2CB41A3F86D9488AAB36C97021553E375BE07750778A40
gamepad_bridge.py 33202DAEFC2844678C9C7E10B65A3954D21B9CC785661BEF37C96F8D21FE6528
banner.png 161F6E944229405D8954057AA112AFAC51EC92DFB89FBCC72FDFF2F88DBC4BDB
```

## Threat model and self security review

Scope: the owned launcher sources. A local user chooses the client/overlay and
server address. Files, endpoints and batch contents may be malformed. A remote
host must not obtain a reusable secret or select a filesystem path. Other
same-user processes with arbitrary filesystem/process access are outside this
slice's isolation boundary. No service/Studio listener or remote request parser
was added. Only the supplied EXE is executed, and only on Play.

Applied `secure-code-guardian` while building and `security-reviewer` to the
finished owned source. Automated source searches preceded manual review of
every spawn, file write, path check, login branch, token/error flow and unsafe
block. No installed semgrep/gitleaks/cargo-audit was found. Clippy is also absent.
This is a self-review, not the lead's independent integration review or a
complete transitive-dependency vulnerability audit.

| Severity | Finding / resolution |
|---|---|
| High, prevented | A compatibility Session must not be sent to a remote server. Literal-loopback-only stub rejects remote and DNS endpoints before file application. Regression test passes. |
| High, prevented | Executing a batch or retaining its old auth fields could run commands or reuse credentials. Direct EXE spawn, bounded allowlisted parser, auth-field removal and controlled overrides. Injection tests pass; deliberate filter removal fails the canary. |
| High, prevented | File replacement through links could modify unrelated files. Canonical containment, no symlinks/junctions/reparse points, hard-link rejection and exclusive backup creation. Windows hard-link/junction tests pass. |
| Medium, fixed | Applying a stale plan or failing midway could overwrite edits or leave mismatched configuration/locale. Preflight content check, backup all targets, atomic replacement and rollback. Stale-plan and read-only-target failure tests pass. Crash/power-loss atomicity across all files is not provided. |
| Medium, open for W83 | Native Session appears in the client process command line. No bearer logs, no Debug/Display and no private key handling here. Short-lived one-use W83 tokens and profile access restrictions remain necessary. |
| Informational, open | Dependency closure, UI/controller execution and screenshot cannot be audited/accepted without fetched dependencies and a built binary. |

No known unresolved exploitable high/critical finding was found in the reviewed
core. That does not establish complete application security. Runtime UI and
dependency review remain open.

## Executed gates and exact results (initial W86; superseded by W86b below)

All Rust commands ran through `project/scripts/cargo.ps1`. All build output used
`CARGO_TARGET_DIR=%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\w86`.
No server/native client, global tool installation, commit, agent or unrelated
file edit was performed. The worker did not edit the shared Plan/Ledger; the
lead owns their integration update.

From `project/launcher`:

```text
& ../scripts/cargo.ps1 build
error: no matching package named `eframe` found
location searched: crates.io index
note: offline mode (via `--offline`) can sometimes cause surprising resolution failures
Exit 1

& ../scripts/cargo.ps1 test
error: no matching package named `eframe` found
location searched: crates.io index
note: offline mode (via `--offline`) can sometimes cause surprising resolution failures
Exit 1
```

Attempted bounded registry fetch with `CARGO_NET_OFFLINE=false`, 8-second timeout
and zero retries. It also failed:

```text
& ../scripts/cargo.ps1 fetch
download of config.json failed
Could not connect to index.crates.io:443 over proxy 127.0.0.1
Exit 1
```

An earlier fetch with inherited retry defaults was interrupted through its own
tool session. No shared helper/service was killed. No approval escalation was
available or requested. A temporary windows-sys core check also exposed a
pre-existing private GNU toolchain issue: bundled dlltool lacks its assembler
subprocess. The final core no longer depends on that wrapper and compiles with
one ordinary Kernel32 import. The eventual GUI dependency graph may still need
working GNU import-library tools or an already approved Windows toolchain.

To check useful work offline, generated ignored `.verification/Cargo.toml`:

```toml
[package]
name = "osfr-launcher-core-check"
version = "0.1.0"
edition = "2024"
[workspace]
[lib]
path = "../src/lib.rs"
```

This is an explicit **core-only fallback manifest**, not the launcher gate.
It compiles the same production `lib.rs` and its same `src/tests.rs`:

```text
& ../scripts/cargo.ps1 test --offline --manifest-path .verification/Cargo.toml
running 8 tests
tests::endpoint_and_login_boundaries ... ok
tests::hard_link_target_is_refused ... ok
tests::invalid_config_or_target_never_changes_files ... ok
tests::omitted_overlay_and_stale_plan_preserve_local_files ... ok
tests::arguments_reject_batch_commands_and_missing_ini ... ok
tests::failed_replace_rolls_back_completed_changes ... ok
tests::preparation_preserves_inputs_backs_up_and_filters_auth ... ok
tests::junction_locale_is_refused ... ok
test result: ok. 8 passed; 0 failed
Doc-tests: 0 tests
Exit 0
```

Canary: copy only owned lib/tests into ignored `.verification/canary`, change
the auth-field match arm from dropping arguments to retaining them, and run:

```text
& ../scripts/cargo.ps1 test --offline --manifest-path .verification/canary/Cargo.toml preparation_preserves_inputs_backs_up_and_filters_auth
tests::preparation_preserves_inputs_backs_up_and_filters_auth ... FAILED
assertion failed: old guid/ticket/key/cassessionid arguments must be absent
test result: FAILED. 0 passed; 1 failed; 7 filtered out
Exit 1 (expected)
```

Production source was not mutated for this canary. A read-only check binary
under `.verification/inspect.rs` called only `LaunchPlan::build` against the
supplied client in `lab/working/OSFR.Client/OSFR Client`:

```text
& ../scripts/cargo.ps1 run --offline --manifest-path .verification/Cargo.toml --bin inspect -- '../../lab/working/OSFR.Client/OSFR Client'
READ-ONLY supplied-client plan: 2 targets; no writes or client launch
Exit 0
```

The source is formatted only within owned files. A first rustfmt proxy call
without toolchain environment failed; the direct private executable succeeded:

```text
& ../../lab/tools/rustup/toolchains/stable-x86_64-pc-windows-gnu/bin/rustfmt.exe --edition 2024 --check src/lib.rs src/main.rs src/controller.rs src/tests.rs
Exit 0 (no output)

& ../scripts/cargo.ps1 clippy --offline --manifest-path .verification/Cargo.toml --all-targets -- -D warnings
error: 'cargo-clippy.exe' is not installed for the toolchain 'stable-x86_64-pc-windows-gnu'.
Exit 1
```

### Initial W86 open gates (superseded where noted by W86b)

- Full launcher build/test and Cargo.lock resolution. Controller source test
  `existing_layout_and_contextual_navigation` is written but has not executed.
- Running-window PNG in `lab/reports/rust-launcher-20261015/`. **No screenshot
  exists from this task**; no mockup is substituted. Exact capture commands are
  in `launcher/README.md`; capture mode prohibits game launch.
- Visual readability/focus review on the actual screen; physical pad testing.
  A redesign-skill readability audit informed spacing, contrast and states,
  but a look-and-fix loop could not run without a window.
- Actual client setup/launch and clean-second-PC acceptance, owned by the lead.
  The worker tested preparation with synthetic files and performed a read-only
  plan of the supplied client; it did not launch the game.
- Full dependency/font licences and vulnerability review, package size, W83
  wiring, and retirement of the old launcher entry point at integration.
- Patching, downloads, news feed and real live status remain later 3.5 work.
  Profile persistence and the in-game gilrs bridge were not part of this slice.

Ponytail: reused the Python launch contract, existing art/layout, std file and
process APIs, toolkit focus and serde. Deliberately omitted a patcher, downloader,
new art, second Python runtime path, auth implementation, input injector,
generalized UI framework and server changes.

## W86b: Windows startup and screenshot acceptance (2026-10-16)

**Verdict: PROVEN for W86b's build, tests and two captures.** Independent lead
review remains pending. The requested evidence folder retains its 20261015 name.
No client, gateway, save, Python launcher, shared manifest or shared plan/ledger
was changed. The lead owns the Build Plan/Ledger integration update.

### Observed causes, with provenance

Two independent startup failures were reproduced. Neither required WGPU.

1. **Loader failure before Rust starts.** Native `System.Diagnostics.Process`
   reported **-1073741515 / 0xC0000135**, not success, with empty stdout/stderr.
   `llvm-readobj --coff-imports` showed `libunwind.dll`. Supplying only that DLL
   beside a lab copy let the same launcher reach Rust and reveal failure 2.
   The preserved failing artifact's Cargo fingerprint has **`rustflags: []`**;
   the intended `+crt-static` setting had not reached that build. A diagnostic
   build with only `-C target-feature=+crt-static` removed the non-system DLL
   import, as does the final recipe. An initially tried extra linker `-static`
   was removed as redundant. The final EXE runs with no copied runtime beside
   it; the toolchain's existing `libunwind.a` supplies the runtime. Evidence is in
   `lab/build/w86b/root-cause.txt` and `imports-final.txt`.
2. **Banner upload panics before the first frame.**
   `lab/build/w86b/startup-before.log` records successful WGL creation, OpenGL
   3.3 and AMD Radeon 8060S, followed by:

   ```text
   Texture "Free Realms Expanded banner" has size 2172x724, but the maximum texture side is 2048
   ```

   The process returned 101. The exact assertion is cached
   `egui-0.33.3/src/context.rs:2255-2262`; the initial 2048 limit is
   `src/input_state/mod.rs:371`. Eframe creates the app in
   `eframe-0.33.3/src/native/glow_integration.rs:294-318`, before the first frame
   delivers the renderer limit. This is egui's initial limit, not evidence of a
   2048 GPU hardware limit. `launcher/src/main.rs:23` now uses the existing image
   crate's aspect-preserving `thumbnail` before upload. The existing banner file
   remains unchanged.

The upstream [llvm-mingw discussion of dynamic unwind dependencies](https://github.com/mstorsjo/llvm-mingw/issues/525)
describes static linking as an alternative to supplying runtime DLLs; our
fingerprints, import inspection and native execution are the acceptance evidence.
No new original-game facts, native functions, packet
formats or server rules were recovered. Banner fitting, logging and capture
failure handling are reconstructed launcher policy (R).

### Minimal changes

- `launcher/build.ps1`: repeatable offline/locked gnullvm recipe through the
  existing `scripts/cargo.ps1`; explicit clang linker and static unwind runtime.
  llvm-mingw/bin is never added to PATH. Paths derive from the checkout, with a
  toolchain override for other machines.
- `launcher/src/main.rs`: fit banner; check/log `run_native`'s Result; return a
  failing exit code for invalid or incomplete captures; preserve Play's existing
  capture-mode guard; create PNGs exclusively, including at the actual write.
  Explicit f32 stroke widths remove five compiler future-compatibility warnings.
- `launcher/src/diagnostics.rs`: opt-in `OSFR_LAUNCHER_LOG`, exclusive file
  creation, 1 MiB budget, rendering/startup targets only, panic/error capture.
- `Cargo.toml` / `Cargo.lock`: directly use the existing transitive `log 0.4.34`
  (MIT OR Apache-2.0). No new package was resolved. Glow and toolkit licences
  stay unchanged. The static LLVM runtime's Apache 2.0 with LLVM exceptions is
  recorded in `launcher/DEPENDENCIES.md`; complete distribution audit is open.
- `launcher/README.md`, `DEPENDENCIES.md` and this note: current recipe/evidence.

### Commands and gate outputs

From `project/` in PowerShell:

```powershell
$env:CARGO_TARGET_DIR = [IO.Path]::GetFullPath('../lab/build/w86-llvm')
./launcher/build.ps1 build
# Finished `dev` profile [unoptimized + debuginfo]; exit 0
./launcher/build.ps1 test
# lib:  test result: ok. 8 passed; 0 failed
# main: test result: ok. 4 passed; 0 failed
# doc:  test result: ok. 0 passed; 0 failed
# exit 0
python -I ../lab/build/w86b/verify_capture.py
# home: exit=0 expected=0
# settings: exit=0 expected=0
# existing PNG refused: exit=1 expected=1
# missing argument refused: exit=1 expected=1
# missing parent refused: exit=1 expected=1
# unknown option refused: exit=1 expected=1
# existing diagnostic log refused: exit=1 expected=1
```

`verify_capture.py` starts the native EXE with `--screenshot <launcher.png>` and
`--settings-page --screenshot <settings.png>` using Windows CreateProcess through
stdlib subprocess, with 25-second bounds. It saves full arguments, exit codes
and stderr in `capture-results.json`; it never starts the game. Both final logs
end in `Screenshot saved: 1410x1185` and `run_native returned successfully`.

| Capture in `lab/reports/rust-launcher-20261015/` | Actual renderer output / inspection |
|---|---|
| `launcher.png` | 1410 x 1185, 1,540,596 bytes; banner, Realms, local address, stub login and disabled Play all visible |
| `settings.png` | 1410 x 1185, 1,477,689 bytes; client/locale fields, dimensions, controller toggle and Back all visible |

Both images were opened and visually inspected. At this machine's 150% scale,
940 x 790 logical units produce 1410 x 1185 pixels. Controller enumeration says
connected; no physical button-navigation acceptance was performed. Home's
existing status-dot glyph renders as a small box; status text is readable and
this follow-up did not restyle the UI.

SHA-256:

```text
launcher.png 9750ed1473abf9640d296ac9de7307db96c55678175170e149940eff83a1a15c
settings.png 7fe84a5eae6f5f691fda4f795e81aa70f5160780c506ed18ccab6669bf6cee93
```

Regression/canary: `bundled_banner_fits_startup_texture_limit` was run **before**
adding `thumbnail`; it failed with the exact 2172/2048 panic, exit 1. Evidence:
`lab/build/w86b/banner-regression-before.log`. It passes after the fix. The two
other new tests prove exclusive capture output and the diagnostic byte budget;
the original eight core and one controller test still pass.

Only owned Rust files were formatted:

```text
private rustfmt.exe --edition 2024 --check launcher/src/main.rs launcher/src/diagnostics.rs
exit 0
git diff --check -- launcher "notes/Design/Rust Launcher.md"
exit 0
```

The final debug EXE is 119,785,472 bytes. No release-size or distribution claim.

### Threat model and self security review (W86b)

Scope is local CLI/environment-selected diagnostic and PNG output. Remote
players cannot choose these paths. Same-user arbitrary filesystem/process
access is outside this slice's isolation boundary. No network listener,
authentication route, shell launch or client preparation change was introduced.

Applied Secure Code Guardian while building. Security Reviewer self-pass used
automated `rg` searches for writes, paths, processes, network, tokens and unsafe
code before manually reading the whole owned diff and new files. No installed
semgrep, gitleaks or cargo-audit was found; this is not a full dependency scan.

| Severity | Finding and resolution |
|---|---|
| Low, fixed | A file could appear between screenshot CLI validation and `image::save_buffer`, causing overwrite. PNG encoder now uses `create_new`; unit test and process refusal checks pass. |
| Low, prevented | New diagnostic output could overwrite files or grow indefinitely. Exclusive creation and 1 MiB cap; existing-log process test and byte-budget test pass. |
| Info, fixed | Old CLI/capture failures could return success. Invalid arguments and unsuccessful capture now return nonzero; successful PNG requires an explicit completion flag. |
| No remaining finding in scope | Startup logging is opt-in, target-filtered and excludes form values/launch arguments/tokens. Play's widget and handler remain guarded throughout capture mode. Static linker path is passed via Cargo's environment, never shell-expanded. |

### Remaining gaps and workspace note

W86b's required build/test/window gates are complete. Lead independent review,
physical controller actions, actual client launch, second-PC acceptance,
release packaging/licences, key-pair login integration and the rest of 3.5
remain open. Shared Plan/Ledger/Tasks/MODLOG were outside this worker's ownership.

An early command resolved `../lab` from `project/launcher`, producing accidental
scratch output at `project/lab/build/w86-llvm` and failing to find the linker.
All acceptance commands then used the correct sibling lab. Automatic approval
review rejected recursive cleanup as `blocked by policy`; a safer reversible
move preserved that scratch target intact at `lab/build/w86b/misplaced-target`
after checking both absolute locations. No scratch target remains under
`project/lab/build/w86-llvm`; it is not part of the launcher artifact.
An attempted PowerShell stderr redirect also hit `NativeCommandError`; gates
were rerun normally and their actual outputs are recorded above.

Ponytail: reused Glow, the existing image decoder/PNG encoder, std filesystem and
process APIs, cached `log`, and the private Cargo wrapper. Deliberately did not
build a WGPU fallback, custom renderer/capture framework, downloader, installer,
new auth route or game-client changes.
