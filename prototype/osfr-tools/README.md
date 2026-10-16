# OSFR Rust prototype

Local Free Realms recovery tools and a minimal Rust gateway for the supplied Windows client. The gateway owns transport, login dispatch, movement updates, and character persistence. It currently supports the captured character **GUID 1 in zone 0**, using 46 private initialization messages from the supplied emulator as reference data.

The actual client has reached World Ready against this Rust gateway, recovered an intentionally dropped reliable packet, walked with saved position changes, and restored its saved position/rotation after starting a fresh gateway and client. See the [gateway milestone](../../notes/Research/Rust%20Gateway%20Milestone.md) for exact run evidence and limits. Full gameplay remains outstanding.

All commands below run from the project root. The toolchain wrapper selects the private Windows GNU Rust installation under `../lab/tools`.

```powershell
.\scripts\cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml
.\scripts\cargo.ps1 build --release --manifest-path .\prototype\osfr-tools\Cargo.toml
```

## Gateway and private reference data

| Component | Responsibility |
| --- | --- |
| `soe` | Seeded CRC, bounded zlib framing, ordered receive, duplicates, gaps, rollover, and fragment reassembly |
| `soe_send` | Bounded send window and queue, fragmentation, selective/cumulative ACKs, timed retransmission, and retry failure |
| `session` | `CGAPI_527` negotiation, bounded cooked control bundles, keepalive, and clock reflection |
| `gateway` | Login/ready sequencing, bounded application bundles and tunnels, proven character fields, movement, and current time replies |
| `character_store` | Versioned JSON snapshot, validation, bounded loading, and atomic file replacement |
| `game_store` | Unified version-2 character/inventory/quest/combat snapshot and version-1 migration |
| `gameplay` | Staged domain transitions, atomic rewards, native commands and initialization |
| `inventory` / `item_categories` | Typed owned items/equipment, validated mutations and community-backed category references |
| `quests` / `combat` / `chat` | Reconstructed local objective/checkpoint rules, training enemy and native chat feedback |
| `osfr-gateway` | Loopback UDP socket, session lifecycle, dispatch, saves, and JSON event logs |

The private reference directory is `../lab/references/rust-gateway-reference`. Its manifest identifies and SHA-256 checks 46 captured messages: eight login replies and 38 ready replies. The gateway independently checks the supported packet sequence and proven field offsets, patches saved position/rotation into character initialization, and preserves the remaining reference bytes. These templates are game-derived data outside Git; the source checkout alone cannot initialize this client.

`scripts/prepare-gateway-reference.py` prepares a **new** reference directory from the verified original-client trace and recovered character data. It rejects incomplete streams and existing output directories. The current prepared reference is for client version `1.910.1.530630`, GUID 1, zone 0; a different character or zone needs further implementation and evidence.

Start a service for manual use:

```powershell
& '.\prototype\osfr-tools\target\release\osfr-gateway.exe' 127.0.0.1:20260 --reference '..\lab\references\rust-gateway-reference' --state '..\lab\state\rust-character.json'
```

Without `--seconds`, the service runs until stopped; an idle client session expires after 60 seconds. Add `--seconds 120` for a bounded service. This command starts only the gateway. Client launch, the local asset-manifest HTTP endpoint, configuration, and screenshots are handled by the harness below. Keep character saves outside the repository and reuse the same state path when restarting.

The gateway rejects nonloopback binds and zero-length CRC negotiation. A duplicate handshake retains its response; a rejected replacement retains the active session. Valid replacements get a new CRC seed. Reliable receiver errors, invalid application messages, and exhausted retransmissions terminate the affected session.

## Bounded actual-client harness

The prepared Windows lab includes the supplied client, private Universal Modder/Python runtime, and legacy DirectX dependency. This harness creates a disposable client copy, backs up configuration, starts the Rust gateway, serves the supplied asset manifest on loopback, relays/captures UDP, and takes GPU-safe screenshots. It sends no mouse or keyboard input.

```powershell
& '..\lab\tools\universal-modder-env\Scripts\python.exe' '.\scripts\run-local-baseline.py' --seconds 90 --rust-reference '..\lab\references\rust-gateway-reference' --state-file '..\lab\state\rust-character.json'
```

The duration must be 10–900 seconds. To exercise retransmission, append `--drop-server-sequence 6`; only its first transmission is dropped. Walk manually in the visible game window. Repeating the command with the same state file exercises restoration in a new gateway/client process. Each run records logs, screenshots, datagrams, and before/after snapshots under `../lab/runs/client-baseline-*`, then stops its own processes. Omitting `--rust-reference` selects the supplied emulator baseline instead.

Movement updates are saved immediately. The executable uses a version-2 snapshot containing the character and optional gameplay, bounded to 1 MiB. The version-1 character-only store/API remains supported for migration and tests, with its original 16-KiB bound. Character validation rejects GUID zero, nonfinite values and components outside ±1,000,000. Saves sync a same-directory temporary file and replace the destination; validation/write/replace failure preserves the prior save. Only one writer should use a state file.

## Playable local gameplay

Prepare private inventory, quest, combat and category content using `scripts/prepare-gameplay-*.py` and `scripts/prepare-item-categories.py`; each script documents its required source arguments. Existing prepared content is `../lab/references/rust-gameplay`. Original data and private oracle output stay outside Git.

```powershell
& '..\lab\tools\universal-modder-env\Scripts\python.exe' scripts/run-local-baseline.py `
  --seconds 600 --rust-reference '..\lab\references\rust-gateway-reference' `
  --content '..\lab\references\rust-gameplay' `
  --state-file '..\lab\state\rust-gameplay-character.json'
```

For manual service-only startup, add `--content '..\lab\references\rust-gameplay'` to the gateway command and use the gameplay state path. A saved gameplay snapshot requires content; the executable refuses to discard it. The real client has completed a reconstructed Robgoblin quest, received 25 coins and one item, equipped pants through native UI, then completed an 18.456-second checkpoint course for 10 more coins. Fresh startup restored wallet 35, reward quantity 10, equipment and progress. See [actual-client evidence](../../notes/Research/Gameplay%20Milestone.md).

Chat commands include `realm help`, `realm quest`, `realm attack`, `realm respawn`, `realm race start` and `realm status`. C opens gear; I opens Stuff. The native client suppresses similar repeated chat attempts for up to 30 seconds, including unsuccessful retries. The local enemy rules are 100 HP, 25 damage, 600-ms cooldown and an 8-unit range. Quest900001 and the checkpoint course are explicit reconstructions; original quest/minigame completion is not established.

Application bundles stage all mutations. Quest reward flags, wallet and item grants commit together. The executable saves the staged snapshot before changing live state or queueing replies. Invalid commands give feedback without modifying progress; unfinished activity clocks reset on a fresh session.

## Verification

The Windows ordinary suite passes **73 tests**, with **eight private-input oracle tests ignored by default**. Ordinary checks cover transport, parsing, inventory/equipment/rewards, quest and activity rules, combat validation, malformed bundles, version migration and save preservation. The new private inventory, quest, combat, complete-loop and category oracles were explicitly run. See [fixture provenance](tests/fixtures/README.md), [transport provenance](tests/fixtures/TRANSPORT.md) and the gameplay milestone for exact evidence.

The opt-in real-UDP session checker launches a bounded gateway using a fresh save. It checks duplicate/rejected/replacement handshakes, distinct seeds, stale CRC rejection, valid ping replies, and loopback-only binding. It sends no application/game messages and writes evidence outside Git.

```powershell
& '..\lab\tools\universal-modder-env\Scripts\python.exe' '.\scripts\check-gateway-session.py' --run
```

Run each private oracle explicitly. Use `-CargoArgs @(...)` when forwarding Cargo's `--` separator through the PowerShell wrapper.

The compiled supplied `LibSOE.dll` decoder checks Rust-generated compression frames:

```powershell
.\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','soe_transport','compiled_legacy_decoder_accepts_rust_compression','--','--ignored','--exact','--nocapture')
```

The supplied-server capture replay verifies local hashes, 1,253 frames, and 46 reassembled messages totaling 627,566 bytes. That capture used synthetic client requests:

```powershell
.\scripts\verify-soe-capture.ps1 -CaptureDirectory '..\lab\captures\legacy-gateway-20261008T014620082859Z'
```

The actual original-client capture oracle replays recorded client application messages through the Rust gateway, compares its first 46 replies byte for byte, and checks time replies. This test does not launch the client; the bounded harness is the live check.

```powershell
$env:OSFR_GATEWAY_REFERENCE = (Resolve-Path '..\lab\references\rust-gateway-reference').Path
$env:OSFR_GATEWAY_CAPTURE = (Resolve-Path '..\lab\runs\client-baseline-20261008T020205Z').Path
.\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','gateway','actual_original_client_capture_when_explicitly_configured','--','--ignored','--exact','--nocapture')
```

## Asset tools and one-shot probe

The asset reader catalogs linked legacy `.pack` directories and exports raw bytes with CRC-32 verification. All 24 supplied packs have been extracted and checked; the unmodified ForgeLight Toolbox/DbgPack community extractor independently matched one complete pack.

```powershell
.\scripts\cargo.ps1 -CargoArgs @('run','--release','--manifest-path','prototype/osfr-tools/Cargo.toml','--','catalog','..\lab\working\OSFR.Client\OSFR Client','..\lab\manifests\rust-asset-catalog.tsv')
.\scripts\cargo.ps1 -CargoArgs @('run','--release','--manifest-path','prototype/osfr-tools/Cargo.toml','--','extract','..\lab\working\OSFR.Client\OSFR Client','..\lab\extracted\client-assets')
```

Outputs must be new paths. Extraction uses numbered filenames and `catalog.tsv` to preserve duplicate logical names. Files are raw assets, not converted models/textures/scripts. Failed extraction leaves partial output for inspection and returns an error; catalog rows are written only after CRC validation.

`soe-probe` remains a separate investigation tool: loopback only, one valid handshake reply, then exit, with a ten-second idle timeout. It does not run the gateway or initialize a character.

```powershell
.\scripts\cargo.ps1 -CargoArgs @('run','--release','--manifest-path','prototype/osfr-tools/Cargo.toml','--bin','soe-probe','--','127.0.0.1:20260')
```

## Current limits

This is one local character in one reference zone. Initialization still depends on the 46 captured templates. Movement accepts bounded client coordinates; authoritative physics, collision, and world simulation are not implemented. Login is a local compatibility flow, without an account service or character selection.

The supported gameplay is one reconstructed quest/training enemy/checkpoint course and validated inventory mutations. Original combat abilities/AI, original quest completion, original minigames, jobs, crafting, gathering, housing, social systems, travel and multiplayer remain unsupported. Native item consumption is unverified. Inventory category browsing works, but text search caused one native access violation and needs diagnosis; some obsolete icon-delivery requests return 404. The existing Windows client remains required. See the [coverage ledger](../../notes/Research/Gameplay%20Coverage.md).
