# Kart Racing Restoration

Status: bounded recovery and module contract, 2026-10-09 02:15 UTC; independent-review repairs applied 2026-10-09 03:17 UTC. The original racing schema is recovered far enough to build a validated session contract; the lap/checkpoint/reward numbers are still reconstructed and clearly labelled. **No native acceptance is claimed.** This note is the KART RACING slice of the requested MMO puzzle-piece work; integration into the gateway, content and launcher belongs to the lead.

Owned artifacts (nothing shared was modified):

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/kart_racing.rs` | New module: wire decoder, lobby definition encoder, bounded session contract |
| `prototype/osfr-tools/tests/kart_racing.rs` | Builder tests: 13 cases; repaired contract (flags, clock, names, snapshots, reward authority) |
| `prototype/osfr-tools/tests/kart_review.rs` | Independent guard suite: 12 cases, written RED before the repair then GREEN |
| `../lab/reports/kart-racing-20261009T0158Z/` | Analysis scripts, wire schema, session trace, test vectors, oracle, run logs |
| `../lab/reports/kart-review-20261009T024747Z/` | Independent review bundle (archive scan, capture verification, original findings) |
| `../lab/reports/kart-review-repair-20261009T031700Z/` | Repair bundle: pre-repair sources, RED/GREEN logs, capture re-check, hashes |
| `../lab/decompiled/kart-racing-20261009T0158Z/` | Native `VehicleRace*` class inventory extracted from the Ghidra export |

The module is not registered in `src/lib.rs` (a shared file); the test target compiles the exact source with `#[path = "../src/kart_racing.rs"]`. Lead integration step: add `pub mod kart_racing;` to `src/lib.rs` and drop that include.

## Engine verdict (checked before choosing tools)

Kart Racing is a **native vehicle-race controller inside the world client**, not a Flash minigame. Evidence:

- The Ghidra export carries MSVC RTTI/vftable names for `ControllerVehicleRace`, `BaseVehicleRacePacket`, `VehicleRacePacketSetRaceConfig`, `VehicleRacePacketRegisterVehicleRacePlayer`, `VehicleRacePacketGateHit`, `VehicleRacePacketUpdateVehicleRaceProgressData`, the proxied mine/missile/pickup classes and `VehicleRaceResultsDataSource` (62 classes over 216 matched lines; addresses in `../lab/decompiled/kart-racing-20261009T0158Z/vehicle-race-classes.json`: `VehicleRaceProcessor` 0x00ba2b90, `RacerInfo` 0x00ba2830, `ControllerVehicleRace` 0x00afc510).
- The same export separates the Flash path: `ControllerFlashMiniGame` and `ClientMiniGameManager` serve the movie-based crafting minigames, which is why cooking/forging have `game_*.gfx` assets.
- The client asset manifest has no `game_racing.gfx`/`game_karts.gfx`; the racing UI pieces are small (`karts1.gfx` 5,064 B, `derbyCrown.gfx`, `derbyResults.gfx`). Kart bodies are normal world models (`kt_kart07_body_bbe`, `kart08_*`).
- The archived original traffic carries racing as a subtype stream of the tunneled world protocol (inner 33; every opcode-33 body in this capture sits on outer 5 in both directions) rather than a separate socket.

## Original wire schema (recovered)

Source: `../lab/references/OpenSourceFreeRealms/ofrserver/Packets/packets_racing_minigame.pcapng` (SHA-256 `4a289592f8105f7df6857cf030071bb9a0d797bd192cd8d5f68e2e7efc9b5986`), 45,378 frames, 12,053 datagrams, all CRCs valid under the existing two-byte model, two `invalid child length` frames recorded. Bodies were decoded to `../lab/reports/original-minigame-packets/racing-*.bin` by the earlier minigame analysis; this slice re-read them read-only.

Frame shape (checked byte-exactly by `decode_racing_frame`):

| bytes | field |
| --- | --- |
| `u16 LE` | outer opcode; 5 in every opcode-33 body of this capture, in both directions. Outer opcode is not a direction field here. |
| `u8` | tunnel flag; 0 or 1 in this capture (12,855 flag-0 / 9,534 flag-1 of 22,389 opcode-33 bodies). Preserved verbatim; its meaning is not recovered. |
| `u32 LE` | inner length, exactly equal to the remaining bytes |
| `u16 LE` | inner opcode, always 33 (0x21) for racing |
| `u16 LE` | racing subtype |
| n | subtype payload (archived file length minus 11) |

Recovered subtype table (payload direction, count, archived file length):

| subtype | direction | count | length | note |
| --- | --- | --- | --- | --- |
| 1 | S to C | 7 | 15 | counter advances 3..9 across the session |
| 2 | C to S | 8 | 23 | client join, carries a fixed 64-bit id plus a counter |
| 3, 4 | S to C | 1 each | 15 | same shape as 1, single value 1 / 7 |
| 5 | S to C | 8182 | 96 | bulk racer stream (85-byte payload) |
| 6 | C to S | 4679 | 96 | bulk client racer stream (85-byte payload) |
| 7 | C to S | 31 | 35 | |
| 8 | both | 3 | 31 | |
| 9 | C to S | 1 | 27 | session id + `4ac46a2a`-class trailer |
| 10 | both | 9 | 23 | session id, then a 32-bit counter descending 4..0 |
| 11 | S to C | 6 | 5803-6177 | kart body asset + full config block |
| 14 | S to C | 9358 | 84 | bulk race-progress stream (73-byte payload) |
| 15 | S to C | 50 | 43 | id + three f32 (position-shaped) + `1.0, 1.0` + a constant 1324 |
| 16 | S to C | 13 | 15 | |
| 17 | C to S | 33 | 35 | |
| 18 | S to C | 4 | 15 | values 5,3,1,4 |
| 22 | C to S | 1 | 19 | session id |
| 23 | S to C | 1 | 11 | empty payload |
| 34 | S to C | 1 | 95 | one-off pre-race block with f32 values 716/82/423 |

Other inner opcodes observed in the same capture and kept as context: 39/7, 39/23, 41/2, 41/106, 41/107, 41/108, 41/109, 41/114, 41/120, 41/121, 41/132, 45/8, 47/14, 49/8 (68 quest-objective updates), 102/1, **102/2 (2,393 B)**, 141/1, 141/2, 147/1, 167/1.

Direction in the table above is derived from the capture's UDP four-tuples (flow 0 = server `64.37.170.6:20267` -> client `192.168.1.145:61759`, flow 1 the reverse), not from the outer opcode or the tunnel flag. The independent archive scan is recorded in `../lab/reports/kart-review-20261009T024747Z/capture-verification.json`.

Subtype 11 layout, verified on all six captured frames:

| bytes | field | recovered value (first frame) |
| --- | --- | --- |
| `u32 LE` | name byte count | 22 |
| name length | kart body asset name (length-prefixed, no NUL terminator) | `kt_kart08_body_bbe.cdt` |
| 4 | first attribute `f32` | 400.0 (`00 00 c8 43`, immediately after the name in all six frames) |

Other captured bodies carry `sg_kt_speedster_body_bbec_002.cdt` (33) and `sg_kt_zipster_body_bbec_002.cdt` (31) with the same layout, followed by part tables (`..._frtire_bbe.adr`, `..._body_bbe.adr`, `Vehicle_Idle`, `Vehicle_Drive`, `Vehicle_Boost`, `Vehicle_Crash_*` animation names). The declared count equals the exact name length in all six frames; the "NUL terminator" in the first draft of this note was the first byte of the 400.0 attribute.

Subtype 14 (progress stream) embeds the same 64-bit session id as subtypes 2/9/22 and holds `0x7f7fffff` (`FLT_MAX`) slots where a lap time would be unknown. The float layout of subtypes 5/6/14 is **not** decoded here, so no position is trusted (see below).

## Original identity data

| Item | Value | Source |
| --- | --- | --- |
| Lobby game type, Kart Race | 1 | pinned Sanctuary `LobbyGameDefinition` comment |
| Lobby game type, Demo Derby | 2 | same |
| Lobby locale ids | 3052 "Kart Race Lobby", 3053 "Demo Derby Lobby" | supplied client locale export (`lab/reports/locale-id-to-text.json`) |
| Kart Driver job | GUID 48, name id 2917, 20 levels, badge 7494 | supplied server `Customize/PacketSendSelfToClient.json`, "Start of Kart Driver" |
| Demo Derby Driver job | GUID 49, name id 2919 | same file |
| Speedways | Wildwood 31/18206, Thunder Falls 32/3606, Frost Ridge 33/382855, Tangletrack 34/382856, Sandstorm 39/401623 | pinned Sanctuary `PointOfInterests.json`; wiki "Speedways" (rev 46668) lists the same five |
| Kart items | Race Karts (3722), Demo Derby Karts (3723), `Team Rockabilly Nitrous System` slot-B part | client locale ids; wiki rev 43914 |
| Client achievement string | `Kart Racing Champion` | native FreeRealms.exe through the Ghidra export |

The wiki corpus (`../lab/references/freerealms-wiki/wiki.sqlite`) was queried read-only; its SHA-256 is still `b9d7b9f1adabb42db54c3253bf43fddc249d6d70e3f2075b134a9441e2fcd911`, matching the recorded verification. Search output with revision URLs is preserved in the report folder.

## Bounded contract

`src/kart_racing.rs` defines:

| Item | Behaviour |
| --- | --- |
| `decode_racing_frame` | Exact tunnel validation (outer 5/6, flag 0/1 preserved losslessly, declared length equals available, inner opcode 33) and subtype split |
| `vehicle_body_asset` | Subtype 11 name byte count plus the length-prefixed asset name; rejects empty, truncated, embedded-NUL and non-UTF-8 windows |
| `LobbyGameDefinition::encode` | Byte-identical to the unchanged community serializer for the bounded entry subset. **Not** the native 102/2 wire framing (the capture prefixes a `u32` remaining length and carries six rows plus further blocks) |
| `TrackLayout` | Speedway id, lap count, ordered unique non-zero checkpoint ids, optional reward schedule; rejects empty/zero/duplicate/oversized definitions and a schedule that prices nothing |
| `RaceSession` | `Lobby -> Countdown -> Racing -> Finished/Abandoned`; checkpoint order enforcement; `Duplicate` for re-delivery; lap timing with best-lap tracking; `Finished` only after the configured last lap |
| `checkpoint clock` | A timestamp older than the current lap start is rejected (`ClockRegression`) before any state change; `start` and `finish` reject regressions too |
| `snapshot`/`restore` | Reconnect keeps laps, best lap, next gate, timestamps **and the price schedule**; incoherent progress, claimed markers outside a finished priced run, and empty schedules are rejected as `CorruptSnapshot` |
| `finish` | `RaceResultProposal` with laps, total time, best lap; only valid on a `Finished` race; an audit record, never a grant token |
| `claim_reward` | Takes no proposal: derives the award from the session's own `Finished`, fully-counted, explicitly priced run; once-only; `Unpriced` never grants |
| `observe_racer_state` | Accepts the two recovered payload sizes as opaque diagnostics and never changes progress |

Deliberate boundaries, by requirement: no kart physics, no client-authoritative position, no invented reward numbers. `placement` and reward stay unknown/unpriced until original evidence exists.

## Verification

| Check | Result | Log |
| --- | --- | --- |
| RED, naive module | 8 of 11 tests fail on real assertions (order, laps, duplicate, reconnect, malformed frames, oracle bytes, reward) | `../lab/reports/kart-racing-20261009T0158Z/rust-kart-racing-red.txt` |
| GREEN, module | 11 of 11 pass | `rust-kart-racing-green.txt` |
| Full crate suite | 38 suites, 239 passed, 0 failed, 36 ignored (includes this target) | `rust-kart-racing-full-suite.txt` |
| Formatting | `cargo fmt --check` exit 0 with empty output | `fmt-check.txt` |
| Community serializer oracle | 80 bytes, SHA-256 `e088a1f906696bf853fffa5766afebf7a990e31635210f7b7cb2a26309f7e113` produced by the pinned, unchanged Sanctuary sources; asserted byte-for-byte in the test | `oracle/lobby-game-definition-community.bin` |
| Capture vectors | 6 frames asserted against their recorded capture SHA-256 before decoding | `kart-racing-test-vectors.json` |
| Wiki corpus | Unchanged: `b9d7b9f1...` before and after the read-only queries | `wiki-search-kart-racing.json`, `wiki-search-kart-driver.json` |
| Review repair RED | 18 builder-target and 28 review-target compile failures naming the missing contracts (flag field, flag 0 acceptance, name validation, clock variants, schedule in snapshot, no-arg claim) | `../lab/reports/kart-review-repair-20261009T031700Z/rust-kart-repair-red-builder.txt`, `...-red-review.txt` |
| Review repair GREEN | `kart_racing` 13 passed / `kart_review` 12 passed / 0 failed, post-format | `../lab/reports/kart-review-repair-20261009T031700Z/rust-kart-repair-green.txt` |
| Capture re-check after repair | 22,389 opcode-33 bodies, 0 declared-length mismatches, flag counts unchanged, all vectors and the oracle hash still match | `../lab/reports/kart-review-repair-20261009T031700Z/capture-recheck.txt` |

The oracle compiles only pinned community files (`PacketWriter.cs`, `ISerializableType.cs`, `LobbyGameDefinition.cs`); its vectors use original locale ids and its output is the test's golden bytes.

## Native gates (not claimed)

1. Subtype-to-class mapping: the native class names are recovered, the concrete subtype numbers each class serialises are not.
2. Opcode 102/2 lattice: the captured 2,393-byte lobby payload starts with a `u32` remaining-length field (2378), a six-row body (`37`/`3030` pair present in row 0) and further blocks; the community writer's bytes are a subset, not the native framing, and must not be sent as a live 102/2 response. The full original catalogue framing is open.
3. Lap count, gate ids and gate order per speedway: `VehicleRacePacketGateHit` exists, the concrete numbers are open.
4. Reward numbers for the racing quests (`Fighting for Funds` etc.) and race payout: open; the module refuses to guess and refuses to grant from anything but an explicitly priced, fully-counted, finished run.
5. Bulk stream layout (subtypes 5/6/14): open; positions stay non-authoritative.
6. Gate geometry/physics: unrecovered, so reconstructed tracks stay unpriced and are not reward-eligible.
7. Native acceptance: the real client has not launched against this code; the lead owns that run.

## Skills applied

| Skill | Concrete benefit here |
| --- | --- |
| RTK | Command output kept condensed; only the decisive test lines and error text were read. |
| Ponytail | Bounded one-struct session with no trait objects, no timers, no per-racer map, no new dependencies; `ponytail:` comment marks the deliberate ceiling. |
| Caveman | Terse, evidence-first reporting; commands, ids, hashes and errors quoted exactly. |
| game-recon | Drove the "engine before tools" step: fingerprint the client, separate native controller from Flash minigame, pick the reimplementation route. |
| game-research-websearch | Used the archived wiki corpus with revision provenance instead of live scraping; dead/absent articles stay labelled. |
| reverse-engineering | Ghidra RTTI/vftable extraction produced the real class inventory and the wire-schema work used the preserved capture rather than guessing. |
| asset-pipeline | Not applicable: this slice consumes asset names, it does not convert sprites or models. |

## Reproduce

```powershell
$lab = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab'
$report = "$lab\reports\kart-racing-20261009T0158Z"
& "$lab\tools\wiki-env\Scripts\python.exe" "$report\analyze-kart-racing-wire.py"
& "$lab\tools\wiki-env\Scripts\python.exe" "$report\extract-vehicle-race-classes.py"
& "$lab\tools\dotnet\dotnet.exe" run --project "$report\oracle" -- "$report\oracle\vectors.json" "$report\oracle\lobby-game-definition-community.bin"
cd '%USERPROFILE%\Documents\Projects\CartoonMMO\project'
.\scripts\cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml --test kart_racing
.\scripts\cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml --test kart_review
```

Cross-links: [[Gameplay Coverage]], [[Free Realms Recovery]], [[../Design/Rust Revival Roadmap]].
