# Rust gateway milestone

The supplied Free Realms client **1.910.1.530630** now logs in to the local Rust gateway, enters the reference zone, walks, saves its position, and returns to that position after starting a fresh Rust process and client. This milestone was verified on October 7, 2026 PDT (October 8 UTC). The original native client remains the compatibility reference.

## Actual-client evidence

All paths in this note are relative to `%USERPROFILE%\Documents\Projects\CartoonMMO`.

| Check | Result | Evidence |
| --- | --- | --- |
| Login/world entry on Rust | World Ready and Initialized - Client; CRC trailer 3 bytes, buffer 512 | `lab/runs/client-baseline-20261008T022638Z` |
| Deliberate reliable loss | Server sequence 6 dropped, resent byte-identically 140 ms later, then cumulatively ACKed through 37 | Same run, `analysis/transport-report.json` |
| Independent trace validation | 1,479 frame hashes and 1,477 post-session CRCs valid; largest server data datagram 334 bytes | Same transport report |
| Actual walking | Authorized W hold 900 ms; four position messages, three changed-state saves, displacement 7.334 world units | Same run, `ui-checks.json`, screenshots, raw frames and server events |
| Fresh process restoration | Server PID 3688 replaced by 5656; fresh client also launched | `lab/runs/client-baseline-20261008T023143Z/manifest.json` |
| Restored state | Position and rotation match the saved snapshot exactly as three little-endian f32 values | `lab/manifests/rust-gateway-milestone.json` |
| Client acceptance of saved position | World Ready reports `(-1412.029541, -28.219860, 344.728607)`; restored world screenshot inspected | Restart run, `client/Logs/FreeRealms.log`, `rust-restored-world_small.png` |
| Cleanup | Both runs stopped their exact client/server process handles | Both manifests; aggregate milestone report |

The walking run used release SHA-256 `3f6bcafe362a0fd3390da293e74a0b2e3151b9d39f7a27e290ba26ba63e52180`. The final build used for restart adds rejection of unsupported saved zones: `84e3937c67ffb35d765bf97c79a2a7cbaa11033a55143e6f13c4fca7cd36f6a1`. The character snapshot is `lab/state/rust-character.json`; each run preserves its own before/after snapshot.

The actual client also resolves an ACK uncertainty: before recovery its cumulative opcode 21 stayed at sequence 5 while opcode 17 selectively acknowledged sequences 7–37. After sequence 6 arrived, cumulative ACK advanced to 37. This agrees with the pinned Sanctuary reference and independently supports the Rust sender's interpretation.

The launch harness sends no input itself. Parent-session game input is separately recorded in each run's `ui-checks.json`, so the harness's `input_automation: false` does not mean there was no authorized UI check. The restart check only dismissed the observed news modal; no walking key was sent.

## Implemented behavior

Rust owns loopback UDP sessions, CRC/compression framing, ordered receive, bounded fragmented delivery, reliable sending, selective/cumulative acknowledgement, retransmission, duplicate suppression, control bundles, clock reflection, application bundles, login validation, ready/loaded state, movement updates, time replies, and versioned character snapshots. Candidate session rejection preserves the current session; duplicate handshakes return the same response; replacement sessions change the CRC seed.

Character writes use a same-directory temporary file, flush it, and replace the prior snapshot. Invalid/corrupt existing files are rejected rather than reset. The store is a single-writer local snapshot, not a transactional multiplayer database or a guarantee against directory-metadata loss after power failure. GUID 1 and the reference zone ID 0 are the supported scope. Zone ID 0 is distinct from the captured geometry/type value 5; other zones are rejected rather than assigned incompatible geometry.

World initialization still uses **46 captured emulator application messages** stored privately in `lab/references/rust-gateway-reference`. SHA-256, lengths, manifest layout, and proven self/zone offsets are validated before use. Only the proven GUID, position, rotation, and zone fields are patched. Captured packets, game assets, and recovered code remain outside Git. No C# server process runs during the Rust checks.

## Reproduce

From `%USERPROFILE%\Documents\Projects\CartoonMMO\project`:

```powershell
.\scripts\cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml
.\scripts\cargo.ps1 build --release --manifest-path prototype/osfr-tools/Cargo.toml

# Fresh disposable client; recorded loopback traffic; bounded cleanup.
& '..\lab\tools\universal-modder-env\Scripts\python.exe' scripts\run-local-baseline.py `
  --seconds 120 --rust-reference '..\lab\references\rust-gateway-reference' `
  --state-file '..\lab\state\rust-character.json' --drop-server-sequence 6

# Reconnect to a fresh Rust process using the same save; omit the loss injection.
& '..\lab\tools\universal-modder-env\Scripts\python.exe' scripts\run-local-baseline.py `
  --seconds 90 --rust-reference '..\lab\references\rust-gateway-reference' `
  --state-file '..\lab\state\rust-character.json'

# Synthetic live session controls, with no game launch.
& '..\lab\tools\universal-modder-env\Scripts\python.exe' scripts\check-gateway-session.py --run
```

For manual service operation, the release binary accepts `osfr-gateway.exe 127.0.0.1:PORT --reference DIR --state FILE`. Omit `--seconds` for continuous operation, or supply a bound. The client also needs the copied D3DX component, matching loopback asset-manifest service, and local launcher configuration; the harness provides these. Starting the gateway alone does not configure or launch the client.

The prepared reference already exists; do not regenerate over it. Its builder is `scripts/prepare-gateway-reference.py`, which checks the captured input hashes and recovered serializer layout.

## Verification and limits

Formatting, release build, and **43 ordinary tests** pass. Three private-input tests are ignored by default rather than silently passing without input. The actual-client gateway oracle was explicitly run and passed: 44 reliable requests, exactly 46 original initialization replies, seven dynamic time replies, and one world-entry signal. The two earlier compiled-decoder/emulator-capture oracles were also exercised in the preceding transport milestone; they were not needlessly rerun here.

The final binary passes all seven live UDP session checks. Evidence is `lab/runs/gateway-session-check-20261008T023232Z-0da6f701/report.json`. These include duplicate handshakes, rejected CRC-zero replacement preserving a valid session, distinct replacement seeds, old-seed rejection, new-seed acceptance, nonloopback bind rejection, and bounded exit. Unit checks additionally cover loss/reordering/duplicates, sequence rollover, fragmentation/wire limits, retry exhaustion, corrupt snapshots, failed replacement, malformed bundles, and reference integrity.

This proves a short local compatibility slice. The native client still logs texture/image/rendering warnings. Existing quest UI, starter inventory, NPCs, and announcements may come from captured initialization; their appearance does not establish working rules. Quests, combat, reward/inventory mutations, multiple accounts, multiplayer, further zones, long-running stability, and a replacement Rust client remain unverified or unimplemented.

Next: select one NPC/quest with surviving evidence, implement the interaction/objective/reward path in Rust, and verify its reward survives another restart. Track evidence and implementation coverage separately.
