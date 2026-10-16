# W90 routed application fuzz evidence

Status: Gateway coverage proven; complete Security Model T3 remains partly proven.
Only `tests/app_fuzz.rs` and this note are authored artifacts. Product code is untouched.

## Scope and provenance

Threat model (R): an untrusted Player sends malformed application messages and replays requests.
Every test Gateway has dev tools OFF and role Player. No socket, native client, user save,
global installation, dependency, commit, or external service is used.

Route inventory comes from the current Rust source, not from older notes:

- `src/gateway.rs:450`, `application_handler`: 10 labels and 30 family/subtype selectors,
  including the `33/*` wildcard. The source inventory test derives these selectors and
  fails when an existing label acquires an uncovered selector.
- `src/gameplay.rs:803`, `handle_npc_packet`; `:2000`, `handle_in_world`;
  `:3425`, `handle_opening_inner`; `:4782`, non-opening `handle`.
- Active packet widths reuse `kart_entry` / `kart_service` constants. Movie payloads reuse
  `forging::payload_packet`; chat framing reuses `chat::local_request`.
- Reference loading follows `tests/coin_shop.rs`, `cooking_gameplay.rs`, and `gateway.rs`.
  Deterministic loops follow W81's `tests/transport_fuzz.rs` style.

No new original-client function fact is recovered here. Synthetic NPCs, quest rewards,
chest bindings, wallet, stock, role, caps and fixture setup are reconstructed test policy (R).
No C# reference source is copied.

Private inputs stay in lab: by default
`lab/reports/round59-native-20261015T124443Z/content` and
`lab/references/rust-gateway-reference`. Overrides are `OSFR_APP_FUZZ_CONTENT` and
`OSFR_GATEWAY_REFERENCE`. Gateways validate reference hashes through the existing loader.
Each test copies the prepared package into its own PID/serial directory under
`lab/build/w90`, authors a small NPC/shop/quest/chest overlay there, and removes only that
directory after verifying its canonical path. The kart test uses a synthetic reference
in its active instance, because the production reconnect loader deliberately abandons races.

## Covered vs routed

| Gateway label | Live selectors covered |
|---|---|
| social_chat | 15/1, including explicit invalid UTF-8 |
| quick_chat | 67/2, 67/3 |
| npc_interaction | 26/4, 6, 8, 10, 23 |
| social_router | 26/14, 15, 17, 30; 40/1, 3, 4, 6 |
| quest_engine | 49/2, 49/14, including pending accept/turn-in and reward replay |
| kart | 26/42; 33/* (2, 6, 7, 9, 10, 17, 19, 22, 65535); 41/108; 102/1 |
| combat | 36/10 |
| inventory | 42/2, 3, 8; 35/34 |
| merchant | 66/43; 165/4, 5; open-shop follow-up and transaction replay |
| gameplay.handle_in_world | Unknown family/subtype; additional branches listed below |

Additional gameplay routes: class selection 26/13, chest 42/15 and 42/16,
encounter exit 41/109, and 39/u8 subtypes 5, 6, 7, 14 in each active
Forging, Smelting, Cooking, Mining and Harvesting handler. Non-opening Combat/inventory/
quest fallback is loaded separately. Gateway login, Ready, Loaded, movement, camera,
time, tunnel and bundle length boundaries are also exercised.

The corpus covers every prefix truncation of each seed, three trailing shapes,
sliding u32::MAX fields, invalid boolean bytes, invalid UTF-8, unknown IDs/subtypes,
foreign activity instances, early actual client win requests, and replay.
Duplicate mutations are removed with a stdlib BTreeSet. No random seed or fuzzer dependency.
Content and trace are each run ON/OFF. Checks compare the complete canonical snapshot,
catch panics, bound replies plus forwarded bytes to 1 MiB/128 messages, and send a valid
time request after each main-corpus case. Each movie also accepts a valid start after
the hostile sequence; active kart accepts valid registration. Shop/NPC and pending chest/
quest checks verify valid continuations to observe state absent from canonical snapshots.
The canary deliberately changes one wallet coin and verifies the equality oracle fails.

## Finding 1 — Low: unknown chest use consumes pending confirmation

Handler: 42/15, routed through Gateway into `MysteryChests::handle`.
Defect: `src/mystery_chest_loot.rs:132` takes `pending` before checking that the requested
item exists and matches the confirmation. The failed request leaves canonical save data
unchanged but invalidates the pending valid Check.

Minimal INNER packet sequence (synthetic instance IDs):

1. `2a 00 10 00 a1 bb 0d 00` — Check owned instance 900001.
2. `2a 00 0f 00 04 bc 0d 00` — Use unknown instance 900100.
3. `2a 00 0f 00 a1 bb 0d 00` — Confirm the original instance; incorrectly refused.

Each is wrapped by the existing 5/flag/length envelope. Reproducer:
`finding_1_unknown_chest_use_must_preserve_pending_confirmation`, marked
`#[ignore = "W90 finding 1"]`. It asserts the intended correct behavior and fails on
the inspected product. Impact is denial of this session's pending action; no extra
reward, lost chest/key, privilege escalation or remote panic was observed.
Owner remedy: validate the requested owned item and matching pending ID before taking
the pending confirmation. No product fix is made by W90.

## Commands and gates

All Rust commands set:

```powershell
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\w90'
```

Compile: `scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test app_fuzz --no-run -j 2`
passed. Initial isolated target compilation took 2m46s; test execution is measured separately.

Use a named array to preserve the test-runner `--` through PowerShell:

```powershell
& .\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','app_fuzz','-j','2','--','--test-threads=1','--nocapture')
& .\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','app_fuzz','finding_1','-j','2','--','--ignored','--nocapture')
```

The regular gate reports 10 passed, 0 failed, 1 ignored. Coverage reports 10/10 labels,
30/30 selectors. Counted corpora: 10,392 main mutations, 5,907 active mutations,
2,598 fallback mutations, plus the separate sequence/boundary checks.
Final regular output (test runtime excludes compilation):

```text
W90 active corpus: 5907 mutations, elapsed 21.9679908s
W90 non-opening fallback: 2598 mutations
W90 routed corpus: 10392 mutations, elapsed 14.7010485s
W90 labels covered/routed: 10/10
W90 selectors covered/routed: 30/30
test result: ok. 10 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 50.38s
```

`lab/build/w90/app-fuzz-run.txt` records the final verification. The isolated ignored
reproducer reports `FAILED. 0 passed; 1 failed; 0 ignored; 10 filtered out; finished in 0.98s`
as expected; `lab/build/w90/app-fuzz-finding-1.txt` records its verification.

Inspected source SHA-256 values at completion:

```text
7a036b8bcc323b763f111d873c01873d3dcd59635878763b8d84e1c87678be5e gateway.rs
d9a6c50b241f4fdb0ab8170827020cf2cf9e72b5a21f24818d51f6dc8d70decc gameplay.rs
b0364996623102f9011412ff2882ed9cb43393719eea7df91f16f8c3b3913fca inventory.rs
2325b3dac47e5957b0456a44e7a4366897d9283869e028744edaf6660b3eb3b9 mystery_chest_loot.rs
```

Only the owned file is formatted/checked with
`lab/tools/rustup/toolchains/stable-x86_64-pc-windows-gnu/bin/rustfmt.exe --edition 2024 [--check] tests/app_fuzz.rs`.
The automated security scan searched the owned diff for sockets, commands, elevated
roles, dev-tools enablement, secrets and filesystem mutations; manual review followed.
No harness security finding remains. The sole logged product finding is Low (#1).

## Open gates and limits

- Shared social packets are forwarded by Gateway. The CLI's private
  `bin/osfr-gateway.rs:1175` `Session::dispatch_message` and `:1246` `commit_outcome`
  are outside this library harness. Their social parsers, shared-world mutations,
  transport queues and durable save-before-send transaction are not proven by these tests.
  Completing that part of T3 needs lead-owned Session access or a process/network harness.
- This deterministic bounded corpus is not exhaustive over all gameplay phases, numeric
  values, content packages, load, memory allocation failure, or simultaneous players.
- Roles/identity remain server-owned fixture configuration. This is not T4/T5/T7 or
  a native client acceptance claim. The private content inputs must be available.
- Security Model T3 requires pending grants unchanged; finding 1 keeps that condition open.

Ponytail: reused real Gateway/loaders, existing packet builders, typed snapshots, prepared
content, stdlib deterministic loops and panic capture. No product fix, dependency,
generic fuzz framework, mock gateway, shared-module seam, or native-client automation.
