# Free Realms Expanded launcher (W86)

Standalone Rust skeleton. It prepares an existing, writable client copy and
starts its local `FreeRealms.exe`. Choose that copy and an optional locale pair
in Settings. It never downloads a client, executes a batch file, or runs Python.

Login is visibly a **local compatibility stub**, behind `LoginProvider` in
`src/lib.rs`. Only literal loopback endpoints work until W83 is connected.
The controller option controls this launcher window. It does not inject game
input or start the existing in-game Python bridge. Settings are session-only.

From this directory, using the project's private toolchain:

```powershell
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\w86'
& ../scripts/cargo.ps1 build
& ../scripts/cargo.ps1 test
& ../scripts/cargo.ps1 run
```

Controller: left stick/D-pad moves focus, A selects, B returns, LB advances,
Start toggles Settings. Tab/Shift+Tab, Space/Enter, Escape and mouse also work.
The original `scripts/launcher/gamepad.json` is compiled in, not duplicated.

Capture the running launcher without any game launch (output folder must exist;
Play is disabled for the entire capture run, including controller activation):

```powershell
& ../scripts/cargo.ps1 run -- --screenshot ../../lab/reports/rust-launcher-20261015/launcher.png
& ../scripts/cargo.ps1 run -- --settings-page --screenshot ../../lab/reports/rust-launcher-20261015/settings.png
```

The screenshot comes from the native window's egui renderer and the app then
closes. `--settings-page` can also open Settings without capture mode.

**Current evidence:** core tests passed in an offline std-only check manifest.
Full build/test and screenshots are blocked by missing cached crates and the
unreachable registry proxy. UI compilation, physical controller navigation and
client launch remain unverified. Read `notes/Design/Rust Launcher.md` for exact
commands, toolkit decision, provenance, security review and open gates.
