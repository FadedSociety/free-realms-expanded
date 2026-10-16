# Verified recovery workflow

2026-10-08 (Pacific). Reproducible guide for the Free Realms recovery: what was verified, which route produced each artifact, the exact local commands, and the acceptance gates a change must pass. Status labels matter: **verified** (re-run today or receipted), **recorded** (manifest exists, not re-run today), **inferred** (pseudocode or hypothesis), and **incomplete** (known gap).

Companion notes: [[Reviews/Recovery Workflow Evidence Review]] (author self-review, correctly labeled not independent) and [[Reviews/Recovery Workflow Independent Review]] (independent re-derivation; lead acceptance of this guide remains open). Corrections requested by that review were applied on 2026-10-09; the disposition table is `lab/reports/recovery-workflow-corrections-20261009T000722Z/correction-receipt.md`. Raw author bundle: `lab/reports/recovery-workflow-20261008T234338Z/` (private, untracked).

## Preserved inputs

Work only from preserved copies; never write into `lab/originals`.

| Archive | Bytes | SHA-256 |
| --- | ---: | --- |
| `lab/originals/OSFR.Server.zip` | 1,061,437 | `cb6806c12fc26a4a5f1cb28b9b695b2de52a6d33ae35a55b46c2e6bcd3888ff5` |
| `lab/originals/OSFR.Client.zip` | 1,815,328,743 | `61167cec14879ffe22669c60bc08bc49dad9dc34f075305b27b587e40956609c` |

Both hashes were **re-verified by full SHA-256 read on 2026-10-08** (server 0.08 s, client 2.0 s) and match the intake manifests (`lab/manifests/OSFR.*-intake.json`). Re-check any time with `Get-FileHash -Algorithm SHA256` (section "Exact local commands"). Full file-level inventories are in `lab/manifests/OSFR.Server-intake.json`, `OSFR.Client-intake.json` and the `*-zip-inventory.json` files. Extracted working copies live in `lab/working`.

## Recovery routes

**Managed .NET (server): ILSpy recovers decompiled (reconstructed) C# source, not original authored source.** ILSpy rebuilds code from IL: declared type and member names survive in .NET metadata, but local names, comments and original formatting do not, and the `.csproj` files are synthesized (7 present, no `bin`/`obj`). Seven assemblies decompiled with exit code 0 into `lab/decompiled/server` (769 `.cs` files: OSFRServer 3, Gateway 19, LibSOE 104, CommandLine 47, DotNetZip 96, log4net 239, Newtonsoft.Json 261). Outcome per assembly: `lab/manifests/server-decompilation.json` plus seven `ilspy-*.log`. Limits: the count includes third-party dependencies, and the recovered projects have **not** been rebuilt or proven semantically equivalent.

**Native x86 (client): Ghidra gives inferred pseudocode, not source.** `FreeRealms.exe` (25,067,520 bytes, PE32) was analyzed in `lab/decompiled/ghidra-projects` and exported (`lab/manifests/native-recovery.json`). The first analysis hit its 900-second budget (partial), then the export completed in 661 s: 153,423 recovered internal functions, 0 failures; a read-only reopen counted 153,968 total function records in the saved project's `saved_project_inspection` (153,423 internal + 545 external; the export summary itself reports `external_functions_skipped: 0`, so the 545 comes from the project inspection, not the export loop). Artifacts: `lab/decompiled/client/FreeRealms/pseudocode.c` (95,176,801 bytes, `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`) and `functions.tsv` (5,231,774 bytes, `993e041ec32aed63742bdcfac529bc7edf916fdce2f208df1071692fcf1c171c`). Export success does not validate types, names, control flow, or runtime behavior, and native DLLs plus `FreeRealmsMac.exe` were not analyzed.

**Rust reimplements behavior; it never translates either output automatically.** Per `AGENTS.md` and [[Design/Rust Revival Roadmap]], pseudocode and decompiled C# are evidence used to identify behavior; each Rust slice is written deliberately and verified with its own oracle. A recovered method body, a successful SOE handshake, or a walking emulator is not a complete game.

**Community readers come before any custom format work.** Formats are ForgeLight-family. Pinned sources under `lab/references`: ForgeLight Toolbox (`a8d7fae0c81ed410b43ddba279d645b74ee905a1`), DbgPack (`8feac971cc21449b20f1cb58d0f1ff900fd6b849`), FR ADR Toolkit (`5acfdf8d6006fa6f43d1d3605f076a6be7393e72`). Evidence: DbgPack extracted `Assets_008.pack` (6,074 entries, 45,765,884 bytes) with every SHA-256/CRC-32/metadata match against the Rust export (`lab/manifests/community-pack-check.json`); FR ADR Toolkit read Chatdy/warpstone actor, mesh, material and skeleton files with a static preview (`lab/extracted/community-asset-check/report.json`). The Rust pack reader predates that search in time, but byte-equality now holds. Write a custom reader only for a demonstrated gap, and record the community search first.

## Exact local commands

Run from `%USERPROFILE%\Documents\Projects\CartoonMMO\project`. Completed outputs are never overwritten. ILSpy `-o`, the InspectNative output argument and captured report files take a fresh destination; **the two community scripts do not**: `scripts/check-community-pack.py` and `scripts/check-community-assets.py` hardcode `lab/extracted/community-pack-check`, `lab/extracted/community-asset-check` and (pack script) `lab/manifests/community-pack-check.json`, then use `mkdir(exist_ok=False)` / `open("x")` so a re-run fails immediately; there is no fresh-destination parameter. Re-check the community readers read-only instead (bounded set below). All toolchains are private under `lab/tools`; no global installs are assumed.

```powershell
# Toolchain activation
$lab = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab'
$env:DOTNET_ROOT = "$lab\tools\dotnet"                 # ILSpy
$env:JAVA_HOME   = "$lab\tools\jdk\jdk-21.0.12.1+1"    # Ghidra
$py = "$lab\tools\universal-modder-env\Scripts\python.exe"

# Preserved-archive re-verification (read-only)
Get-FileHash -Algorithm SHA256 "$lab\originals\OSFR.Server.zip"
Get-FileHash -Algorithm SHA256 "$lab\originals\OSFR.Client.zip"

# Managed route: fresh output directory required
& "$lab\tools\ilspy\ilspycmd.exe" -p -o '<fresh-output-dir>' "$lab\working\OSFR.Server\OSFR Server\OSFRServer.exe"

# Native route: resume or re-export from the saved project; do not reimport over it
$env:GHIDRA_HEADLESS_MAXMEM = '4G'
& "$lab\tools\ghidra\ghidra_12.1.4_PUBLIC\support\analyzeHeadless.bat" `
  "$lab\decompiled\ghidra-projects" FreeRealms `
  -process FreeRealms.exe -noanalysis -readOnly `
  -scriptPath '.\scripts' -postScript InspectNative.java '<fresh-output>.json'

# Community readers (both scripts refuse to overwrite earlier output)
& $py scripts\check-community-pack.py --pack Assets_008.pack
& $py scripts\check-community-assets.py

# Rust: ordinary acceptance, release build, test inventory
.\scripts\cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml
.\scripts\cargo.ps1 build --release --manifest-path .\prototype\osfr-tools\Cargo.toml
.\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--','--list')
.\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--','--list','--ignored')
```

The wrapper passes arguments to Cargo unchanged, so `--` must travel inside `-CargoArgs` (a bare `--` is consumed by PowerShell). Asset tools refuse existing destinations; the named catalog `lab/manifests/rust-asset-catalog.tsv` and extraction `lab/extracted/client-assets` already exist. Bounded launch, gateway, session and gameplay harness forms are retained in [[Recovery Commands]]; preserve each completed run directory when re-testing.

**Bounded re-verification set (cheap, read-only).** In a clean shell the bare names `dotnet`, `java`, `cargo`, `um` and `ilspycmd` are NOT FOUND (verified 2026-10-09T00:07Z); use full paths, or prepend `lab\tools\dotnet`, `$env:JAVA_HOME\bin`, `lab\tools\cargo\bin` and `lab\tools\universal-modder-env\Scripts` to PATH.

```powershell
& "$lab\tools\dotnet\dotnet.exe" --version
$env:DOTNET_ROOT = "$lab\tools\dotnet"; & "$lab\tools\ilspy\ilspycmd.exe" --version
& "$lab\tools\jdk\jdk-21.0.12.1+1\bin\java.exe" -version
.\scripts\cargo.ps1 --version   # wrapper prepends lab\tools\cargo\bin itself
& "$lab\tools\universal-modder-env\Scripts\um.exe" --version
& "$lab\tools\ghidra\ghidra_12.1.4_PUBLIC\support\analyzeHeadless.bat"   # usage; exit 1 with no arguments

# Exact read-only smoke commands (stdout only; no extraction, no writes)
& "$lab\tools\universal-modder-env\Scripts\python.exe" "$lab\reports\recovery-workflow-20261008T234338Z\sample-oracle.py"
& "$lab\tools\universal-modder-env\Scripts\python.exe" "$lab\reports\recovery-workflow-20261008T234338Z\community-reader-smoke.py"
```

`cargo test -- --list` counts are timestamped: the author bundle listed 183 (154 ordinary + 29 ignored) at 2026-10-08T23:46Z; a fresh listing during independent review on 2026-10-09T00:07Z listed 208 (179 ordinary + 29 ignored) because `ordinary_quit` entries were concurrently added to `cooking.rs`, `smelting.rs`, `forging.rs` and `gameplay.rs`. The last recorded full-suite result is 154 pass / 0 failed (MODLOG, [[Current Progress]]); re-list before quoting any count. The bundle also retains a 5-test pack-reader subset and a 3-test wire subset. None of this re-extracts assets or re-analyzes binaries.

## Acceptance gates

Every behavior, content or runtime change follows this order; no gate may be skipped or simulated by a later one. Evidence-only edits (notes, receipts, reports) are not runtime changes: they need correct sourcing, reproducible commands and review of changed acceptance claims, but they do not need to pass the native-replay gate.

1. **First test (failing regression).** Freeze a vector or capture and show it RED against the current build before changing behavior.
2. **Narrow implementation.** Change the smallest responsible code; the same frozen test must go GREEN and unrelated tests must stay green.
3. **Original oracle.** Compare against a compiled supplied or community writer/serializer, or an original-client capture. Fixtures must state provenance: original client, supplied emulator, community reference, or synthetic serializer.
4. **Native replay: screenshot, wire, save.** Run the actual client through ordinary input; retain screenshots plus transport receipt and immutable save hashes, and stop all test PIDs.
5. **Independent review.** A separate reviewer re-derives the result from raw evidence and writes a receipt. Author self-review does not satisfy this gate.

Worked example (2026-10-08 proximity-radius repair): RED and GREEN plus patch and proof in `lab/reports/native-interaction-radius-regression-v1/` (`baseline-red.txt`, `candidate-green.txt`, `production-candidate.patch`, `proof.json`); independent re-derivation in `lab/reports/native-interaction-radius-independent-regression-review-v1/` (`independent-baseline-red.txt`, `independent-candidate-green.txt`, `independent-compiled-community-oracle.txt`, `independent-integrated-production-green.txt`); release build and SHA in `lab/reports/native-interaction-radius-release-v1/`; native acceptance and transport receipt in [[Reviews/Native Queen and Prompt 203139]] (`lab/runs/client-baseline-20261008T203139Z/analysis/native-queen-prompt-final-proof.json`, SHA-256 `1332451d1312374027ca8c116f23ac81206fa98735169a3079bc0b5faec1b8c2`, release `ea297301afc5462be7f40142d2a6b4709eb94af6b3556de721dfa407d9a89063`).

## Verification boundaries

| Boundary | Entry point | Proves | Does not prove |
| --- | --- | --- | --- |
| Ordinary unit tests | `cargo test` (154 pass, 29 ignored) | In-repo deterministic logic: pack directory, transport, stores | Original game behavior or content |
| Private writer oracles | `--ignored` tests, env-gated | Byte equality with supplied/community serializers | Protocol completeness |
| Integration | Loopback gateway/session checks (`scripts/check-gateway-session.py --run`) | Live session replacement, stale-seed rejection, persistence | Native client acceptance |
| Native replay | `lab/runs/client-baseline-*` receipts | Actual client wire, saves, screenshots, stopped services | Historical completeness, other zones, multiplayer |
| Content ledger | [[Opening Historical Coverage Ledger]] | Documents known historical gaps | n/a (evidence record) |

Rust gateway initialization still depends on **46 private captured emulator messages**; replacing them with typed serialization is an open task. State manifests (`project/recovery-manifest.json` at the repo root, plus `lab/manifests/rust-*-milestone.json`) mix snapshot fields from different dates; treat the newest milestone note as authoritative.

## Not yet complete (historical scope)

Twelve of 17 local opening stages are rewarded; the campaign stops at the Queen checkpoint (stage 13 unaccepted). Also open: original micro-tutorial gates (camera globes, movement barriers, Chugawug rescue), three missing region-1 sticker quests, exploration collections, original gathering/farm dungeons, full class progression, the native inventory-search access violation, original Soccer/Racing (traces only), quest 1617 Petty Yeti identity, original quest/reward/AI balance (most local stages are reconstructed with `original_quest_id:null`), and native DLL/`FreeRealmsMac.exe` analysis. See [[Gameplay Coverage]], [[Opening Historical Coverage Ledger]], [[Current Progress]].

## Evidence hygiene

Keep decompiled code, pseudocode, captures, and raw assets in `lab` (outside Git). Tracked notes carry paths, counts and hashes only, never decompiled bytes, asset dumps, secrets, or private capability URLs. Immutable saves and completed run directories are preserved: re-tests write new directories. Independent receipts live under `lab/reports` and are cited by SHA-256.
