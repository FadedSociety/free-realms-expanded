# Free Realms Expanded launcher

Standalone Rust launcher. It prepares an existing, writable client copy and
starts its local `FreeRealms.exe`. Choose that copy and an optional locale pair
in Settings. A realm's pinned patch manifest can install or repair files in that
copy. It never executes a batch file or runs Python.

The current W98 skin follows the owner's direction A: deep-blue glass panels,
raised bumpy frames, glossy green controls, attached tabs and an imagegen forest
backdrop. Four orange vines grow from the banner wordmark on launch. Art sources,
hashes and slice boxes are listed in `assets/ui/README.md`; review captures and
the readability audit are in `lab/reports/rust-launcher-w98/`.

To join a friend's realm, open **Settings → Join a realm** and enter the game
address, the 64-digit server identity supplied by the host, and your account
handle. Check the short fingerprint against the host, then choose **Create my
key**. The launcher stores the private key and trusted profile in your Windows
user profile, adds the realm to the list, and shows the public key. Copy that
public key and send it to the host for enrollment. It is safe to share; never
send a private key. Return to **Realms** and choose the saved account beside
PLAY.

The trusted server identity comes from the host, never from a network response.
Login uses the local secure tunnel at `127.0.0.1:20043`; the host must enroll the
public key for the supplied account handle. A refused login explains that the
host may still need to add the key. The broker intentionally returns one
generic refusal for unknown, disabled, wrong-key and locked accounts.

The launcher keeps the command-line key tools for operator setup and recovery:

```powershell
& $exe --new-key my-realm
& $exe --print-public-key my-realm
```

`--new-key` writes an owner-only key under the current user's
`APPDATA\FreeRealmsExpanded\profiles\<id>` directory and prints only the
public key. `--print-public-key` exports the public half for an existing local
profile. These commands run before UI startup and return their output directly.
The controller option controls this launcher window. It does not inject game
input or start the existing in-game Python bridge. Settings are session-only.

From this directory on Windows, using the project's private toolchain:

```powershell
$lab = (Resolve-Path ../../lab).Path
$env:CARGO_TARGET_DIR = Join-Path $lab 'build/w98/target'
./build.ps1 build
./build.ps1 test
$exe = Join-Path $env:CARGO_TARGET_DIR 'x86_64-pc-windows-gnullvm/debug/osfr-launcher.exe'
& $exe
```

`build.ps1` calls `../scripts/cargo.ps1` with `--offline --locked` and the
installed `x86_64-pc-windows-gnullvm` target. It selects the sibling lab's pinned
llvm-mingw UCRT toolchain. For another location, set `LLVM_MINGW_ROOT` or pass
`-LlvmMingw <unpacked-toolchain>`. Install/cache those prerequisites beforehand;
the script does not download anything. Without `CARGO_TARGET_DIR`, output goes
to `lab/build/w86b/target`. Add `-Release` for a release build (not measured here).

The target receives Rust's `+crt-static`: the failing old artifact had empty
rustflags and imported the missing `libunwind.dll`. Never add llvm-mingw's bin
folder to PATH: host build scripts then select an incompatible compiler.
Start the executable from PowerShell or cmd; Git Bash's loader is unsuitable
for this UCRT build. The launcher keeps its OpenGL renderer.

Controller: left stick/D-pad moves focus, A selects, B returns, LB advances,
Start toggles Settings. Tab/Shift+Tab, Space/Enter, Escape and mouse also work.
The original `scripts/launcher/gamepad.json` is compiled in, not duplicated.

Capture the running launcher without any game launch (output folder must exist;
Play is disabled for the entire capture run, including controller activation):

```powershell
$reports = Join-Path $lab 'reports/rust-launcher-20261015'
New-Item -ItemType Directory -Force $reports | Out-Null
New-Item -ItemType Directory -Force (Join-Path $lab 'build/w86b') | Out-Null
$env:OSFR_LAUNCHER_LOG = Join-Path $lab 'build/w86b/home-review.log'
& $exe --screenshot (Join-Path $reports 'launcher.png')
$env:OSFR_LAUNCHER_LOG = Join-Path $lab 'build/w86b/settings-review.log'
& $exe --settings-page --screenshot (Join-Path $reports 'settings.png')
Remove-Item Env:OSFR_LAUNCHER_LOG
```

The screenshot comes from the native window's egui renderer and the app then
closes. `--settings-page` can also open Settings without capture mode. Existing
output files are refused. Capture failure, timeout, invalid arguments and
`run_native` errors return a nonzero exit code.

W86c adds `--window-size WIDTHxHEIGHT` for bounded layout review, using logical
egui points (640–3840 by 480–2160); default remains 940 x 790. At this PC's 150%
display scaling, `--window-size 853x533` captures 1280 x 800 physical pixels;
`--window-size 1280x800` captures 1920 x 1200. Short windows use a smaller banner
and tighter spacing. The standard single-realm Home and Settings fit at
1280 x 800 physical pixels without scrolling; longer realm lists may scroll. Captures remain
unable to prepare or launch the game at any size.

The skin bundles Lilita One for titles and captions, keeps Ubuntu body text and
readable address fonts, and uses egui's real controls for keyboard/controller
focus, Space/Enter activation and accessibility. Font and art provenance are in
DEPENDENCIES.md. W86c's final screenshots and before/after audit are under
`lab/reports/rust-launcher-20261016/`; `notes/Design/Rust Launcher.md` records the
13-test result and focus-ring canary. Physical controller actions remain open.

`OSFR_LAUNCHER_LOG` enables an exclusive, bounded 1 MiB file of startup/rendering
diagnostics and panic/error output. Its parent must exist and the file must be
unused. It does not log launch arguments, tokens or form contents. Omit the
variable for normal operation.

**Current evidence (W86b, 2026-10-16):** offline build and 12 tests pass. Home and
Settings each saved a 1410 x 1185 PNG through their own running-window capture.
Physical controller navigation, actual client launch, release packaging and
second-PC acceptance remain unverified. See `notes/Design/Rust Launcher.md` for
root-cause logs, provenance, security review and the remaining full-launcher gates.

## W98 skin evidence (2026-10-16)

W98 replaces the earlier geometry-only button and panel surfaces with bundled
imagegen sprites. The nine-slice renderer holds corners fixed, tiles the bumpy
edge strips, and stretches only the plain panel centre. The capture-only options
`--test-focus-play` and `--test-vine-growth-ms 0..1500` exist to produce review
images; screenshot mode disables Play and closes after capture. Build and test
results, including any host-specific test limitations, are recorded in
`lab/reports/rust-launcher-w98/README.md`.

## W112 art and login evidence (2026-10-16)

W112 moves vine growth to the banner's right edge and checks every revealed
pixel against protected text and starburst rectangles. It gives the list row,
text field and status plank smooth bevels, aligns repeating button/panel edges
to complete bead periods, and draws a cover-fitted backdrop over the full
viewport. PLAY uses the selected realm's stored profile when its profile ID is
set and passes the resulting session token through the existing launch path.
The W112 report contains before/after captures, per-gate test output, art
provenance and remaining limitations.

## W121 pinned patching (2026-10-16)

Settings applies to the selected realm. Set the existing client folder, the
full HTTPS manifest URL, its trusted SHA-256 pin, and optionally a mirror
manifest URL. Obtain the pin independently from the realm operator. Never use
a hash supplied by the same untrusted download as the trust anchor. Pins,
URLs and realm choices are session-only, like the existing launcher settings.
There is no default download host or bundled community-client distribution.

Normal patch checks start automatically once the folder, URL and pin are set.
**Verify and repair** repeats the same check and rehashes existing files;
same-size tampering is detected. PLAY stays disabled for configured patches
until that exact folder/realm/URL/pin selection verifies successfully. With no
patch URL, the existing local-client workflow remains available. File count,
bytes and current file appear on both pages. Cancel preserves resumable data;
closing the launcher signals cancellation too.

The version-1 JSON manifest has this shape (the example digest is SHA-256 of
an empty file):

```json
{"version":1,"files":[{"path":"data/example.bin","size":0,"sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}]}
```

File URLs are relative to the manifest's directory. A mirror serves the exact
same manifest and relative file tree. Serve Content-Length on every response;
resumed responses must have an exact `206 Content-Range`, or return the whole
file with `200` to restart. Redirects, content compression, URL credentials,
queries and fragments are rejected. HTTPS uses Windows certificate validation
and TLS 1.2; plain HTTP is accepted only for literal loopback test servers.
No cookies, system credentials or login tokens enter patch requests.

Downloads go to `<file>.osfr-part` alongside their destination. A valid complete
partial can be installed without another download. Interrupted partials resume
with Range; a bad final hash discards their contents and retries from zero.
Three attempts use 100/200/400 ms backoff and try the optional mirror. Only a
size/hash-verified, synced partial replaces the installed file, using a
same-directory atomic rename. Existing files survive failed verification.

Manifest limits (R): 4 MiB, 50,000 files, 16 GiB per file, 128 GiB total.
Paths use portable ASCII components, at most 220 bytes, with `/` separators;
traversal, Windows device names/ADS, duplicate case aliases, overlapping paths,
launcher-state names, junctions, symlinks and hard-linked targets/partials are
rejected. Use a private, user-owned client folder: hostile simultaneous local
filesystem mutation is outside the existing launcher's isolation boundary.
Do not run two patchers on the same copy. Atomicity is per file, not across
the whole manifest; a failed update must finish before PLAY unlocks.

Cancellation is checked between hash/I/O blocks and every 20 ms in backoff.
Synchronous WinHTTP calls have 3-second stage timeouts; cancellation waits for
an in-flight call to return. Manifest and file body attempts also have 30-second
and 120-second deadlines. A very large/slow file may require another resume.

W121 uses std, existing serde, Windows WinHTTP and Windows CNG SHA-256. No Cargo
dependency or lockfile change; `getrandom` stays at 0.2.17. It also centres field
text, increases panel inner padding, fits both standard compact pages, and
replaces raw profile IDs with the saved-profile dropdown and Local login.

`--test-patch-progress --screenshot <unused.png>` renders synthetic progress for
visual review only. It cannot download, prepare or launch anything and is
rejected without screenshot mode. Actual HTTP transfer/resume and the UI's
worker/PLAY route are separately tested. Full output, canary, threat model,
security self-review and before/final captures are in
`lab/reports/rust-launcher-w121/README.md`. Native game launch, public HTTPS host,
fresh-PC acceptance and normal-token stored-profile tests remain lead gates.

W131 adds the key-pair setup flow to Settings. `--test-join-flow --screenshot
<unused.png>` captures a synthetic example at any supported `--window-size`; it
does not create a key, save a profile, or contact a server. The default window
is 940 x 790 logical points. At 150% display scale, use `--window-size 853x533`
for a 1280 x 800 physical capture.
