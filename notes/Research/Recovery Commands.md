# Recovery commands

Run from `%USERPROFILE%\Documents\Projects\CartoonMMO\project` unless specified. Completed outputs must not be overwritten; choose a new destination for another pass.

## Rust tools

```powershell
.\scripts\cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml
.\scripts\cargo.ps1 build --release --manifest-path .\prototype\osfr-tools\Cargo.toml
.\scripts\cargo.ps1 fmt --check --manifest-path .\prototype\osfr-tools\Cargo.toml
.\scripts\cargo.ps1 run --release --manifest-path .\prototype\osfr-tools\Cargo.toml -- catalog '..\lab\working\OSFR.Client\OSFR Client' '..\lab\manifests\rust-asset-catalog.tsv'
.\scripts\cargo.ps1 run --release --manifest-path .\prototype\osfr-tools\Cargo.toml -- extract '..\lab\working\OSFR.Client\OSFR Client' '..\lab\extracted\client-assets'
```

The named catalog and extraction output already exist. Those commands intentionally refuse to overwrite them. Tests can be rerun without touching the game assets.

## Managed recovery

For each `.dll` or `.exe` in the supplied server directory, the completed run used this form with a matching assembly name and output folder:

```powershell
$env:DOTNET_ROOT = [IO.Path]::GetFullPath('..\lab\tools\dotnet')
$env:DOTNET_CLI_TELEMETRY_OPTOUT = '1'
& '..\lab\tools\ilspy\ilspycmd.exe' -p -o '..\lab\decompiled\server\OSFRServer' '..\lab\working\OSFR.Server\OSFR Server\OSFRServer.exe'
```

See `lab/manifests/server-decompilation.json` and the seven `ilspy-*.log` files for actual outcomes. Do not rerun onto the completed output without selecting a fresh destination.

## Native first pass

```powershell
$env:JAVA_HOME = [IO.Path]::GetFullPath('..\lab\tools\jdk\jdk-21.0.12.1+1')
$env:PATH = (Join-Path $env:JAVA_HOME 'bin') + ';' + $env:PATH
$env:GHIDRA_HEADLESS_MAXMEM = '4G'
& '..\lab\tools\ghidra\ghidra_12.1.4_PUBLIC\support\analyzeHeadless.bat' `
  '..\lab\decompiled\ghidra-projects' FreeRealms `
  -import '..\lab\working\OSFR.Client\OSFR Client\FreeRealms.exe' `
  -max-cpu 4 -analysisTimeoutPerFile 900 `
  -scriptPath '.\scripts' `
  -postScript ExportNative.java '..\lab\decompiled\client\FreeRealms' `
  -log '..\lab\manifests\ghidra-FreeRealms.log' `
  *> '..\lab\manifests\ghidra-FreeRealms-console.log'
```

The first run used absolute equivalents of these paths. The analysis budget is 900 seconds; each exported function has a 30-second decompiler timeout. The script writes pseudocode, a per-function status table, and a summary. Inspect the analysis log as well as the export summary: an export count does not establish that all executable behavior was discovered. Resume or re-export the saved Ghidra project with a fresh output path instead of reimporting over it.

The completed first pass reached its analysis time limit, then exported 153,423 internal functions and saved the project. A read-only reopening used the same headless launcher and project arguments with `-process FreeRealms.exe -noanalysis -readOnly -scriptPath .\scripts -postScript InspectNative.java ..\lab\decompiled\client\FreeRealms\inspection.json`. That output already exists. Inspection confirmed 545 additional external function records whose library code was not exported.


## Community checks and actual-client baseline

```powershell
& '..\lab\tools\universal-modder-env\Scripts\um.exe' scan '..\lab\working\OSFR.Client\OSFR Client'
& '..\lab\tools\universal-modder-env\Scripts\python.exe' 'scripts\check-community-pack.py'
& '..\lab\tools\universal-modder-env\Scripts\python.exe' 'scripts\check-community-assets.py'
& '..\lab\tools\universal-modder-env\Scripts\python.exe' 'scripts\run-local-baseline.py' --seconds 90
.\scripts\verify-soe-capture.ps1 -CaptureDirectory '..\lab\captures\legacy-gateway-20261008T014620082859Z'
```

The two completed community checks refuse their existing destinations; select fresh paths in those wrappers before repeating them. They invoke existing upstream readers, not another custom format parser. Pinned source and dependency details are in [[Toolbox]].

The baseline creates fresh disposable client/server copies, backs up eight INI files with Universal Modder, adds the signed official x86 D3DX9_31 component from the private tools directory, and serves the supplied asset manifest's CRC/size on loopback. It forwards and records local UDP with an 8-MiB receive buffer, captures the actual window without input, and stops its exact test PIDs after the bounded duration. Raw packets, logs, original INIs, and screenshots remain in the newly named lab run directory. It does not perform walking input automatically; the recorded short walking check was separately authorized by the owner. See [[Client Baseline]] for evidence and measured menu coordinates.

PowerShell can consume a bare `--` when calling a `.ps1` wrapper. Use a named array when Cargo needs that separator, for example:

```powershell
.\scripts\cargo.ps1 -CargoArgs @('run','--release','--manifest-path','prototype/osfr-tools/Cargo.toml','--bin','soe-probe','--','127.0.0.1:20260')
```

## Rust gateway and persistence acceptance

The prepared private reference exists at `..\lab\references\rust-gateway-reference`. Build release before starting the actual-client harness:

```powershell
.\scripts\cargo.ps1 build --release --manifest-path prototype/osfr-tools/Cargo.toml
& '..\lab\tools\universal-modder-env\Scripts\python.exe' scripts\run-local-baseline.py `
  --seconds 120 --rust-reference '..\lab\references\rust-gateway-reference' `
  --state-file '..\lab\state\rust-character.json' --drop-server-sequence 6
& '..\lab\tools\universal-modder-env\Scripts\python.exe' scripts\run-local-baseline.py `
  --seconds 90 --rust-reference '..\lab\references\rust-gateway-reference' `
  --state-file '..\lab\state\rust-character.json'
& '..\lab\tools\universal-modder-env\Scripts\python.exe' scripts\check-gateway-session.py --run

$env:OSFR_GATEWAY_REFERENCE = [IO.Path]::GetFullPath('..\lab\references\rust-gateway-reference')
$env:OSFR_GATEWAY_CAPTURE = [IO.Path]::GetFullPath('..\lab\runs\client-baseline-20261008T020205Z')
.\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','gateway','actual_original_client_capture_when_explicitly_configured','--','--ignored','--exact')
```

The harness itself sends no keyboard/mouse input. The authorized walking check and restart screenshot used Universal Modder separately; exact steps and hashes are in each completed run's `ui-checks.json`. The completed acceptance report is `lab/manifests/rust-gateway-milestone.json`; details and limits are in [[Rust Gateway Milestone]]. Preserve existing snapshots when testing alternatives.

## Rust gameplay slice

```powershell
& '..\lab\tools\universal-modder-env\Scripts\python.exe' scripts/run-local-baseline.py `
  --seconds 600 --rust-reference '..\lab\references\rust-gateway-reference' `
  --content '..\lab\references\rust-gameplay' `
  --state-file '..\lab\state\rust-gameplay-character.json'
$env:OSFR_GAMEPLAY_CONTENT = (Resolve-Path '..\lab\references\rust-gameplay').Path
$env:OSFR_GATEWAY_REFERENCE = (Resolve-Path '..\lab\references\rust-gateway-reference').Path
$env:OSFR_GATEWAY_CAPTURE = (Resolve-Path '..\lab\runs\client-baseline-20261008T020205Z').Path
.\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','gameplay','--','--ignored','--nocapture')
```

The harness now allows 10–900 seconds for gameplay checks. Preserve the separate earlier position-only state. Content preparation scripts are `prepare-gameplay-inventory.py`, `prepare-gameplay-quests.py`, `prepare-gameplay-combat.py` and `prepare-item-categories.py`; use each script's `--help` for exact private inputs. Do not regenerate over preserved evidence. [[Gameplay Milestone]] records controls, commands, chat spam filtering, accepted runs and the inventory-search limitation.
