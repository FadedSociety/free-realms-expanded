# Free Realms Expanded launcher

Standalone Rust launcher. It prepares an existing, writable client copy and
starts its local `FreeRealms.exe`. Choose that copy and an optional locale pair
in Settings. It never downloads a client, executes a batch file, or runs Python.

The current W98 skin follows the owner's direction A: deep-blue glass panels,
raised bumpy frames, glossy green controls, attached tabs and an imagegen forest
backdrop. Four orange vines grow from the banner wordmark on launch. Art sources,
hashes and slice boxes are listed in `assets/ui/README.md`; review captures and
the readability audit are in `lab/reports/rust-launcher-w98/`.

Login is visibly a **local compatibility stub**, behind `LoginProvider` in
`src/lib.rs`. Only literal loopback endpoints work until remote key-pair login
is available.
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
and tighter spacing, with a visible scrollbar when needed. Captures remain
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
