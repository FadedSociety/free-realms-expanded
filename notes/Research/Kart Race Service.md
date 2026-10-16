# Kart Native Race Service Implementation

Implementation, 2026-10-09 (UTC). Scope: the server-side kart race - lifecycle, control, gate
authority, finish and the one-shot reward hand-off - behind the already-implemented entry
(`kart_entry`) and the lead-owned gateway/content/save/UI. No shared-file edits; nothing here grants
or prices anything.

Owned artifacts:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/kart_service.rs` | service: content, context, schedule, gate validation, finish |
| `prototype/osfr-tools/tests/kart_service.rs` | synthetic vectors + ignored private capture gate |
| `lab/reports/kart-service-20261009/probe-kart-service.py` | recovery reproducer (`proof.json`) |
| `lab/reports/kart-service-20261009/build-private-content.py` | private content package builder |
| `lab/reports/kart-service-20261009/kart-service-content.json` | ready-to-wire content (31 gates, cadence, template) |
| `lab/reports/kart-service-20261009/kart-service-context.json` | ready-to-wire context (session/player guid, instance, encounter) |
| `lab/reports/kart-service-20261009/*.log`, `receipt.json`, `final-hashes.txt` | run evidence |

## Verdict

The recovered lifecycle is implemented and validated. A `33/9` register is answered by the entry
handler; this service then drives the recovered cadence (`33/1` states 4-9, `33/4` npc count,
`33/10` countdown 4..0 with the client echo accepted), validates client claims (`33/7` gate hit,
`33/17` pickup hit, `33/6` bulk) and finishes a race by sending the final `33/14` progress block.
Every client claim must name the owned player guid, land inside the ordered next gate volume, arrive
on an advanced server clock and stay under the content speed bound; refused claims change nothing.
A finish is only reachable through the complete, ordered, speed-bounded route, and the reward
hand-off is a one-shot exposure of the finished `kart_racing::RaceSession` for the lead's reward
registry (job/profile gate, priced track, real inventory).

## Recovered contract

**Countdown `33/10`.** The dispatcher `FUN_00ba74d0` case `0xa` parses the packet through
`FUN_00ba1850` -> `FUN_00ba04f0`, which reads `u64 | u32` (the second read is bounds-checked and a
short read zero-fills). The dispatcher stores the **4-byte value at `processor+0x1b4`** and sets a
pending flag at `+0x1c2`; the parsed guid has no consumer. `FUN_00ba6300` then, when its timer
expires, calls `FUN_007071f0(value)`, raises `FUN_009797a0("Countdown:SetCount", ...)`, clears the
flag and **sends the same value back with the local racer guid** - which is the captured client
echo. The slot is initialised to `4`.

**Client-sent subtypes have no client parse case.** The dispatcher's top-level switch covers
`0x1, 0x3, 0x4, 0x5, 0x8, 0xa, 0xb, 0xc..0x10, 0x12, 0x14, 0x15, 0x17..0x1f, 0x21, 0x22`; the
client-sent subtypes `2, 6, 7, 9, 17, 19, 22` are absent, so the server owns their validation.
`33/6` is an 85-byte block whose interior is unrecovered and which carries no guid; `33/7` and
`33/17` are 24-byte `u64 guid | 3 x f32 position | f32 1.0` claims.

**Capture cadence** (offsets from the `33/9` register frame, `probe-kart-service.py`): state 3 at
+995 ms (entry handler), state 4 + `33/4` npcs=7 at +3026, state 5 +3353, state 6 +8246, state 7
+8694, state 8 +9290, countdown 4,3,2,1,0 at +9290, +10075, +10529, +11954, +13242, state 9
+14505; the client echoes countdown 3,2,1,0. The 31 `33/7` hits run +19.5 s to +71.1 s, one local
guid, peak step speed 40.52 m/s, straight-line 1670 m.

**Finish.** There is no server finish packet: the client's results row is derived from
`lap limit <= laps completed` and its `Finished` column from `lap limit < laps + 1`. The server
finishes a race by sending the final `33/14` with `laps == limit`; `33/12` is a racer removal, not a
finish, and no finish appears in the capture (the stream stops on lap 2 of 3).

## Service shape

`KartServiceContent` carries the recovered cadence, the ordered gate volumes, the unmapped `33/14`
template fields and the bounds. `RaceContext { session, player_guid, instance_id, encounter_id }`
binds the race to the entry the lead started: `session` is the entry identity and `player_guid` is
the real character guid used by every client claim, so the captured guid is never silently reused.

The API (all times are absolute server milliseconds):

| Call | Behaviour |
| --- | --- |
| `new(content)` | validates gates, schedule, bounds and template once |
| `set_gate(EntryGate)` | off by default; every entry point refuses while disabled |
| `begin(context, now_ms)` | binds the race; schedule offsets are relative to this instant |
| `tick(now_ms)` | emits every due schedule step in content order; starts the race at countdown 0; aborts past `max_race_ms` |
| `handle_inner(inner, now_ms)` | `33/7`/`33/17`/`33/6`/`33/10` in; inner `33/14` progress out |
| `start_race(now_ms)` | the explicit countdown-0 seam (`tick` calls it automatically) |
| `session() -> Option<&RaceSession>` | the registered racing session (laps, times, phase) |
| `finish_flag()` | true while a validated finish is pending |
| `take_finished_session()` | one-shot `&mut RaceSession` for `kart_rewards::claim_race_reward` |
| `snapshot() -> Option<ServiceSnapshot>` / `restore(content, snapshot)` | save/reload, fully bound to the content |
| `clone()` | Gameplay-clone support; the session rehydrates from its own snapshot and the gate is preserved |

### Independent-review fixes (v2)

* **Gate attribution (S3).** The expected gate is evaluated first, so overlapping physical volumes
  can never let a skipped or stale claim advance: a position inside the expected volume is accepted
  even when it also sits inside a neighbour; otherwise the nearest later containing volume is
  `SkippedGate` and the nearest earlier one is `StaleGate`, with `OutsideGates` when nothing contains
  it. A dedicated test drives one overlapping point through two consecutive gates.
* **Observation budget (S4).** `33/6` and `33/17` no longer share the gate claim budget: they have
  their own `RaceBounds::max_observations` (default `120_000` = 600 s at 200 Hz) and refuse with
  `ObservationBudget` when exhausted, so the capture's thousands of high-rate bulk frames can never
  abort a legitimate race. Content written before the field existed still loads through the serde
  default, and a compatibility test pins that.
* **Radius floor.** Content must use at least `MIN_GATE_RADIUS_M = 3.0 m` per gate volume: the
  captured sampling gap between neighbouring hits is 9.8 m, so a smaller volume could not attribute
  a sample at all. The private package uses exactly 3.0 m; overlapping volumes are allowed and are
  handled by the attribution rule above.
* **Restore binding (S5).** `restore` now validates the snapshot context exactly like `begin`
  (non-zero session, player guid, instance and encounter), binds the snapshot to the content bounds
  (`CorruptSnapshot { field: "bounds" }` on any limit change), bounds-checks the observation counter
  and the clock chain (`countdown_started_ms <= race_started_ms <= last_claim_ms`), so a saved race
  cannot be re-judged with different limits or a zeroed context.

The 31 captured positions are a **player's trajectory**, not the track: the private package is a
test fixture, and the real gate volumes and the finish line are the lead's independent
reconstruction.

Validation and refusals: `ForeignGuid`, `StaleGate`, `SkippedGate`, `OutsideGates`,
`NonFinitePosition`, `ImplausibleSpeed`, `ClaimRate`, `ClaimBudget`, `RaceTimeout`,
`CountdownSpoof`, `InvalidPhase`, `GateDisabled`, `NoOpenRace`. Refusals change nothing except the
bounded attempt counter (`attempts`, the flood guard) - a dedicated test compares the whole snapshot
except that counter. Malformed input is a typed `ServiceError`; `restore` refuses a snapshot whose
speedway, laps, checkpoints, reward or `session_id` does not match the content, so an altered price
or layout can never resume into a payable race.

Authority boundaries kept honest: the gate geometry is reconstructed content (the capture carries
only client positions), `33/6` and pickups are observations with no effect, other racers are not
simulated, and progress is emitted per accepted gate instead of at the captured ~0.3 s cadence.
Nothing in this module grants, prices or mutates inventory; the reward registry stays the lead's.

## Lead route and seams

1. Register `pub mod kart_service;` in `src/lib.rs` (lead-owned) and give `Gameplay` a `KartService`
   clone plus the private content (like `kart_entry.json`): `serde_json::from_slice::<KartServiceContent>`.
2. After the entry handler's register/join, call `begin(RaceContext { session, player_guid, instance_id, encounter_id }, now_ms)`.
3. Call `tick(now_ms)` from the gameplay tick; append `replies` to the outcome (inner frames only -
   the gateway wraps them) and act on `abort` by returning the player to the world.
4. Route inner `33` frames that the entry handler does not own to `handle_inner(inner, now_ms)`.
5. When `finish_flag()` is true, call `take_finished_session()` and run the reward registry; stage the
   service clone so a failed claim can roll back.

## Verification

```
& scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test kart_service
# 19 passed; 0 failed; 1 ignored
& scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml \
    --test kart_service --test kart_entry --test kart_racing --test kart_progress_native
# 19/13/13/9 passed, 2 ignored
$env:OSFR_KART_SERVICE_CAPTURE='.../lab/reports/original-minigame-packets'
$env:OSFR_KART_SERVICE_CONTENT='.../kart-service-20261009/kart-service-content.json'
$env:OSFR_KART_SERVICE_CONTEXT='.../kart-service-20261009/kart-service-context.json'
& scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test kart_service '--' --ignored
# 1 passed: all 31 captured hits accepted, exactly one reconstructed lap
& lab/tools/universal-modder-env/Scripts/python.exe lab/reports/kart-service-20261009/probe-kart-service.py
& lab/tools/universal-modder-env/Scripts/python.exe lab/reports/kart-service-20261009/build-private-content.py
```

Test-first evidence: the target failed before the module existed (`red.log`, exit 1: `couldn't find
file tests\..\src\kart_service.rs`), then passed. The private gate pins the captured frames through
the index sha256 map, replays the real positions and timings through the validator, and (when the
private package is present) loads it and replays through the package itself.

## Remaining gaps

* Gate geometry, countdown field semantics beyond the value slot, the `33/6` interior and the
  `33/14` unmapped fields stay unrecovered; gates are reconstructed content and the unmapped bytes
  come from the captured template.
* No finish is captured, so the exact server frame order at the finish is a reconstruction: the
  client's own derivation (`lap limit <= laps`) is the contract that is implemented.
* Other racers are not simulated; the captured server countdown frames carry other racers' guids and
  the client ignores the guid, so the service sends the owned guid (documented deviation).
* Pickups, gates-hit effects, derby and the return scene remain the lead's/other slices' work; the
  capture's `33/5`/`33/14` continuous stream is not re-authored.

## Skills applied

RTK (normal commands first), Ponytail (one flat service, reuse of `kart_entry` framing,
`kart_progress_native` layout and `kart_racing` lap authority), Caveman (concise reporting; this note
stays complete), `reverse-engineering` (dispatcher/reader/consumer recovery from the pinned export
instead of guessing), `game-recon` (ForgeLight-family, managed/native split already pinned). No
game-automation: the worker does not control the shared game window.

Cross-links: [[Research/Kart Native Race Entry Contract]], [[Research/Kart Race Finish Protocol]],
[[Research/Kart Local Racer and Gate Binding]], [[Research/Kart Vehicle Configuration and Input]],
[[Reviews/Delegated Racing Readiness]].
