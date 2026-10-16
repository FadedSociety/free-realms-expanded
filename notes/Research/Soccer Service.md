---
type: research
topic: soccer-service
updated: 2026-10-13
---

# Soccer Service

Task W37 reconciles the Soccer service with [[Build Guides/Gameplay/Soccer]]. The guide is the build specification; this note records the client evidence, reconstructed service rules, commands, and remaining validation gaps.

## Client evidence

These facts come from the reviewed native map at `lab/native-map/run/map.sqlite`, not from OSFR-held packet captures or server JSON.

| Address | Accepted native label | Supported fact |
|---|---|---|
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` (confirmed) | Routes family-76 Soccer packet classes. |
| `00b794b0` | `SoccerPacketUpdatePlayerState::ReadBody` (confirmed) | Reads a 62-byte body: `u64` guid, `u8` state, `u8` aux, `u32` counter, then three float4s. Payload offsets are `0x00`, `0x08`, `0x09`, `0x0a`, `0x0e`, `0x1e`, `0x2e`. |
| `00cdc5c0` | `SoccerPlayer::SendUpdatePlayerState` (confirmed) | Writes the client sub-14 state body. |
| `01072020` | `SoccerPlayerBase::SetState` (confirmed) | On a state change, increments the counter at player `+0xb0`. The wire `u32` is not transport replay state. |
| `01072610` | `SoccerPlayerBase::NormalizeIntoSlot50` (confirmed) | Normalizes xyz when its squared length exceeds the client threshold, writes the direction into player `+0x50..+0x5c`, and forces `w` to zero. |
| `00b793c0` | `SoccerPacketUpdatePlayerControls::ReadBody` (confirmed) | Reads a 20-byte body: `u64` guid, two float direction components, and a raw `u32`. |
| `00b79580` | `SoccerPacketImpulseBall::ReadBody` (confirmed) | Reads 36 bytes: `u32`, float4, float, float, `u32`, float. Field meanings and server emission are not established by this reader. |
| `00cde9b0` | `SoccerBall::ApplyOrSendImpulse` (confirmed) | A client method exists; its name does not establish a server output contract. |
| `00afb4a0` | `ControllerSoccer::ApplyInputActions` (confirmed) | Processes controller action entries; this does not identify sub-16 bits as kick/pass. |
| `00b7be70`, `00b7c070` | `SoccerProcessor::UpdateTimerView`, `SoccerProcessor::UpdateScoreboard` (confirmed) | Feed supplied time and team scores into UI callbacks. |
| `00b7de60` | `ApplySoccerUiState` (confirmed) | Dispatches wait, ready, halftime, goal, sudden-death, timer, and result UI callbacks. It does not recover server state values or transitions. |

`01074b50` has an accepted `unknown` label. Its bytes were inspected, but no semantic label is claimed: the disassembly shows a four-float copy into player `+0xf0..+0xfc` and a charge-ratio store at `+0xa4`. `notes/Reviews/Soccer Direction Semantics Review.md` records the reviewed dataflow tying vecA to action direction and aux to charge. The charge conversion is `aux / 255`; this does not supply a server power scale.

## Reconstructed service rules (R)

The following are implementation policies, not recovered retail Soccer rules:

- Four slots per team and passive bot fill for one-human local testing. Bots have no AI and cannot score.
- Match geometry, origin spawn, gravity, movement budgets, possession radii, kick power, goal crossing, restart behavior, 180-second fallback halves, and side switching.
- The current sub-14 kick/pass state-byte allowlist and classification; the native reader establishes the field width, not the complete retail state table.
- The encounter-mode offer, generic minigame result route, one-star win mapping, tie-as-loss behavior, elapsed-time rounding, and configured winner grant.
- Soccer output markers, possession fields, score relay, candidate sub-13 field mapping, and timeout action. The gateway chooses its timeout threshold.
- Disconnect cancels the participant's active run without reward, removes the player, releases held possession, and cancels a solo match when no human remains.

`SoccerService` requires every joined human to be in the shared `MinigameRuns::Playing` state before kickoff. It passes score changes and final outcomes through `MinigameRuns`, settles configured grants through `rewards::apply_grant`, and retains the run-derived grant key to prevent a duplicate payment. A client cannot author a goal, ball transform, or authoritative impulse.

The service can serialize selected R output candidates using the local codec. Only the sub-13 body shape in that emitted set has a cited native reader here. Tests of the codec do not prove retail-client acceptance or match playability.

## Commands and results

Native map queries were read-only:

```text
python -I scripts/native_map/nmq.py brief 00b80ac0 00b794b0 00cdc5c0 01072020 01072610 00b793c0 00b79580 00cde9b0 00afb4a0 00b7be70 00b7c070 00b7de60
python -I scripts/native_map/nmq.py show 00b794b0
python -I scripts/native_map/nmq.py show 00b793c0 00b79580
python -I scripts/native_map/nmq.py show 01072020 01072610
python -I scripts/native_map/nmq.py show 00b7de60 00b7be70 00b7c070
python -I scripts/native_map/nmq.py label 01074b50
```

Result: the listed labels were accepted as confirmed. `01074b50` was reported `[accepted:unknown]`.

```text
python -I scripts/native_map/nmq.py dis 01074b50 800
python -I scripts/native_map/nmq.py dis 01072610 96
```

Result: the first disassembly exposed the vecA destination and aux-derived ratio stores described above; the second showed the direction normalization guard and destination. These checks support field-dataflow claims only.

Content queries:

```text
python -I ../lab/content/tools/cq.py vid-grep 'Soccer Gate'
python -I ../lab/content/tools/cq.py wiki 'Soccer Star'
```

Result: video matches show overworld Soccer Gate waypoints, not match play or results. The wiki describes solo/team minigames and Soccer Star progression; it is fan-wiki context only.

Focused Rust command, with target output outside the repository:

```powershell
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W37'
..\..\scripts\cargo.ps1 test --test soccer_service --test soccer_match --test soccer
```

Final result: passed. `soccer`: 11 passed, 1 ignored optional private corpus byte-round-trip; `soccer_match`: 27 passed; `soccer_service`: 16 passed. The build reported one unrelated dead-code warning in `src/housing.rs` (`Lot::instance_index`). An earlier pre-edit attempt was blocked by errors in the unowned `src/bin/osfr-gateway.rs` (`NameData` unresolved and moved-value borrows); that file was left untouched, and the final focused run completed successfully.

The seven owned Rust files were formatted individually with the private toolchain's `rustfmt.exe --edition 2024`; no crate-wide formatter was run.

## Open questions and gaps

- No native Soccer match was launched or inspected. Visual synchronization, collision, goal effects, and client acceptance remain unverified.
- Original Soccer launch/zone-entry exchange, team/roster contract, state-word mapping, server output layouts, half/overtime transitions, disconnect semantics, result close, and Soccer Star progression/rewards remain unknown.
- The sub-13 field meanings, authoritative sender direction, and whether the retail server emits that packet remain unknown.
- The generic minigame result sequence is a tested R integration candidate. It is unknown whether it coexists with Soccer-specific result packets in a retail session.
- Passive bots exist only for solo service testing. They do not provide opponent behavior or historical bot behavior.

## Test reconciliation (W50)

- `soccer_native_wire`: replaced stale `adapter gap` wording with the W37 configured charge-policy and server-verified pass-target gate. Native evidence establishes `aux / 255`; power scale and target rules remain reconstructed.
- `soccer_review`: the synthetic `GoalMade` candidate now expects `Evidence::Reconstructed`. W37 excludes older traces as client evidence and does not recover the server goal-output contract.
- From `prototype/osfr-tools`, ran:

  ```powershell
  $env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W50'
  ..\..\scripts\cargo.ps1 test --test soccer_native_wire --test soccer_review --test soccer --test soccer_match --test soccer_service
  ```

  Result: 72 passed, 0 failed, 1 ignored optional private-corpus test. One unrelated dead-code warning remains at `src/housing.rs:567`.

