# Toolbox

## Installed skill packs

| Source | Entry point | Installed location |
| --- | --- | --- |
| [reverse-skill](https://github.com/zhaoxuya520/reverse-skill) | `$reverse-skill-router` | `%USERPROFILE%\.codex\skills\reverse-skill-router` |
| [Ponytail](https://github.com/dietrichgebert/ponytail) | `$ponytail` | `%USERPROFILE%\.codex\skills\ponytail` |
| [i-have-adhd](https://github.com/ayghri/i-have-adhd) | `$i-have-adhd` | `%USERPROFILE%\.codex\skills\i-have-adhd` |
| [Caveman](https://github.com/juliusbrussee/caveman) | `$caveman` | `%USERPROFILE%\.codex\skills\caveman` |
| [Universal Modder](https://github.com/rehan-remade/universal-modder) | `$mod-any-game` | `%USERPROFILE%\.codex\skills\mod-any-game` |

Ponytail includes six skills. Caveman includes 22 skills. The reverse-skill adapter uses the full package at `%USERPROFILE%\.codex\tools\reverse-skill`, preserving its router, scenario modules, scripts, and references. Universal Modder's eleven skills were installed previously.

New skills are available on the next Codex turn. Installing communication skills does not select a permanent style; invoke the desired one explicitly.

## Applications

- **Obsidian 1.14.4:** `%USERPROFILE%\AppData\Local\Programs\Obsidian\Obsidian.exe`
- **RTK 0.50.0:** installed with WinGet as `rtk-ai.rtk`. Its Codex hook is registered in `%USERPROFILE%\.codex\hooks.json`. Restart Codex to load the integration; review its hook trust state if prompted. Direct `rtk` commands also work in a newly opened terminal.
- RTK telemetry is disabled; no opt-in was given.

## Recovery tools

The following are private to `%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools`; they do not require replacing the machine's global toolchains.

| Tool | Installed version | Path beneath lab/tools | Use |
| --- | --- | --- | --- |
| .NET SDK | 10.0.401 | `dotnet/dotnet.exe` | Runs ILSpy and supports modern reference projects |
| ILSpyCmd | 11.1.0.9782 | `ilspy/ilspycmd.exe` | Managed C# recovery |
| Rust | 1.99.0, Windows GNU | `cargo/bin/cargo.exe`; `rustup` | Original Rust tools; includes rustfmt |
| Ghidra | 12.1.4 | `ghidra/ghidra_12.1.4_PUBLIC` | Native x86 analysis and inferred pseudocode |
| Temurin JDK | 21.0.12.1+1 | `jdk/jdk-21.0.12.1+1` | Ghidra runtime |
| Universal Modder | 0.2.0 | `universal-modder-env/Scripts/um.exe` | Reconnaissance, reversible config backups, GPU window capture |
| Legacy DirectX component | x86 D3DX9_31 | `directx-june2010/x86/d3dx9_31.dll` | Supplied-client prerequisite; copied only to disposable baseline runs |

Downloaded SDK, Rust installer, JDK, and Ghidra archives were checked against their official published checksums. ILSpy was installed as a local .NET tool from its package feed.

Run Cargo through `project/scripts/cargo.ps1`; it sets the private CARGO_HOME and RUSTUP_HOME for the command. For ILSpy set DOTNET_ROOT to `lab/tools/dotnet`. For Ghidra set JAVA_HOME to the private JDK.

The optional reverse-skill MCP services remain unconfigured. Universal Modder is installed from pinned revision `baff1e5d01f63ae7cc81a049b6f4cf60e6c91dec` in a private Python environment. Its Windows helpers and ffmpeg GPU capture are configured under `%LOCALAPPDATA%/universal-modder`; ffmpeg's official BtbN download passed its published SHA-256 check. Selected encoder: AMD AMF. The environment also holds `pefile` and plotting dependencies for community-reader validation.

Source-only community checkouts under `lab/references` include ForgeLight Toolbox, DbgPack, and FR ADR Toolkit. Their exact revisions and per-file hashes are in `SOURCE.json`. The toolbox/DbgPack pair successfully extracted one supplied pack; FR ADR Toolkit read selected actor, mesh, material, and skeleton files. See [[Universal Modder Assessment]] and [[Community Asset Validation]].

Microsoft's signed DirectX June 2010 redistributable is retained privately. Only its required x86 DLL was extracted for the local run, with no system runtime installation. Source, imports, hashes, and signature results are in `lab/manifests/client-directx-runtime.json`.

Exact repository revisions and skill checksums are recorded in `setup-manifest.json` at the project root.

The Rust gateway additionally uses `serde`/`serde_json` for bounded private reference and character-save documents, and `sha2` for template integrity checks. Exact versions are locked in `prototype/osfr-tools/Cargo.lock`. `scripts/run-local-baseline.py --rust-reference ... --state-file ...` runs a fresh client against the release Rust binary; `scripts/check-gateway-session.py --run` checks live session replacement and stale-seed rejection without launching the game. See [[Rust Gateway Milestone]].
