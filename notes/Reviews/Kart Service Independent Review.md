# Kart service independent review

2026-10-09. Independent bounded acceptance of `src/kart_service.rs` as it sits
on disk (worker still finalizing; no implementation file was edited). Findings
lead, checks follow.

## Verdict

The authority design is sound and the money path is closed: a client gate hit is
only a claim, validated against the owned player GUID, the ordered gate index
from the racing session, an absolute server clock, a rate floor and a speed
bound; every attempt counts against a bounded budget; and the service never
grants, prices or touches inventory - it exposes the finished session exactly
once through `take_finished_session`. The absolute-clock, restore and clone
fixes are present and covered by tests. What is still not native-playable is the
geometry: the captured 31 gate hits are one incomplete lap, so any multi-lap
gate list is reconstructed content, and the first-party start/finish-line XML
carries timer semantics but no coordinates.

## Money authority (verified)

- `take_finished_session` is the only hand-off and is one-shot (`reward_taken`
  set before returning); `finish_flag` requires `Phase::Finished` with no taken
  reward; `snapshot`/`restore` persist and re-validate the flag (`reward_taken`
  only with `Finished`); `clone` carries it. No grant, price, item or quest call
  exists in the module; `reward_priced()` only reports that the content carries
  a schedule for the lead's registry.
- A finished race cannot be replayed for a second offer: after
  `take_finished_session`, `finish_flag()` is false and `tick`/`handle_inner`
  report `reward_ready: false`. `begin()` after a finish starts a new race with
  its own one-shot, which is correct.

## Claim validation (verified)

- Ownership: every claim carries the racer GUID and `own_guid` refuses anything
  but `context.player_guid`; the entry session and the player GUID are separate
  fields, so the captured session value can never be substituted for the real
  character GUID.
- Ordering: `claim_gate` maps the position to the first containing gate and
  requires it to equal `RaceSession::next_checkpoint()`; stale, skipped and
  outside-volume claims are typed refusals. Every refused claim still increments
  `attempts` (checked before the rate/speed/gate tests), so a flood cannot
  finish anything.
- Clock: schedule offsets are absolute (`countdown_started_ms + at_ms`),
  `start_race(due_at)` starts the race at the scheduled time, `progress_frame`
  derives elapsed from `race_started_ms`, and the race window aborts past
  `max_race_ms`. Clock regression yields `ClaimRate` (saturating subtraction),
  and the racing session also reports `ClockRegression`.
- Bounds and finiteness: content validation refuses non-finite start/gate/radius
  values, empty/oversized gate lists, unordered or oversized schedules and
  out-of-range states/countdowns; claims refuse non-finite positions; restore
  refuses non-finite `last_position`.

## Native usability findings

**S1, medium - the captured route is one incomplete lap, so multi-lap gate
content is reconstructed.** All 31 captured `33/7` hits carry
`laps_completed = 0` in the concurrent `33/14` stream. The lap counter for the
local racer turns 1 at t+51.83 s, about 0.2 s after the last recorded hit
(t+51.635 s), so the capture covers exactly one lap's gate sequence and no
gate repetition. The route is 1,670.5 m with a 150.9 m gap between the first
and last hit, 91.9 m from the spawn to the first hit and 62.9 m from the last
hit back to the spawn. The private test content therefore repeats the same 31
gates for 3 laps by construction; that repetition is a content decision, not a
captured fact. Evidence: `gate-lap-mapping.json`.

**S2, medium - the finish line sits just past gate 31, not on it.** The lap
increment ~0.2 s after the last hit is ~7-10 m at the captured speeds, so the
start/finish line is a separate volume after the last gate. Private content
should place the start position and finish line there rather than treating gate
31 as the line.

**S3, medium - gate separation is not validated and `gate_index_at` takes the
first containing sphere.** With reconstructed volumes, two overlapping spheres
silently attribute a claim to the earlier gate (`StaleGate`/`SkippedGate`).
Nothing rejects overlapping gates or checks that consecutive gates are
reachable within `max_speed_mps`. The derived private package uses 0.5 m radii,
which is smaller than the distance travelled per tick at race speed
(~0.7 m per 16 ms at 40 m/s); widen the volumes and add an overlap check.

**S4, low - `observations` is unbounded.** `33/6` and `33/17` increments use
`saturating_add` to `u32::MAX` with no budget; they carry no authority, but the
"bounded" claim only holds for `attempts`.

**S5, low - restore does not re-validate non-zero context.** `begin` rejects
zero session/player GUID/instance/encounter, but `restore` accepts the context
as-is; a hand-edited snapshot with `player_guid = 0` fails closed (every claim
is foreign) yet should be reported as `CorruptSnapshot`. `observations` is also
restored without a bound.

## Geometry sourcing (the lead's question)

**Can original gate coordinates come from first-party XML?** Not from the
exported files so far. `lab/extracted/client-assets/pack_21/003847.xml`
contains `<AreaDefinition name="StartingLineFloorArea" id="4213" shape="floor">`
with `<Property type="Timer" StartTimer="1"/>` and the matching
`FinishLineFloorArea` (4214, `StartTimer="0"`), but no coordinates; the
`shape="floor"` geometry is stored elsewhere. `lab/extracted/community-pack-check/
Assets_008/ss_kt_trackAreas.xml` (1,892 B) holds only environment spheres (Bed
380/570 r500, four Ocean spheres and a WaterFall). So the first-party route
proves the start/finish line *areas and their timer semantics* exist and gives
their ids, but not their vertices yet; reading the area geometry needs the
community area/mesh reader, not a custom parser.

**Captured op150?** The racing index holds 238 op150 records, but they are
22-byte inner frames (29-byte outer), not large world blobs. op35
(`BasePlayerUpdatePacket`) has 750 index records whose bytes were never
exported as `.bin` files, so neither is usable as gate geometry from the
exported capture. The only in-capture geometry remains the 31 client-claimed
`33/7` positions and the `33/5` transform `(795.0, 86.19, 365.0)`.

## Checks executed

- `scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml
  --test kart_service`: 16 passed, 1 ignored.
- Full kart set (`kart_service`, `kart_entry`, `kart_gateway`, `kart_rewards`,
  `kart_progress_native`, `zone_transition`, `kart_racing`): 13+5+9+13+11+16+9
  = **76 passed, 0 failed, 4 ignored** (`gate-kart-suite.log`).
- Private valid-31 gate: `--test kart_service -- --ignored` with
  `OSFR_KART_SERVICE_CAPTURE` set: 1 passed - the captured 31-hit route is
  accepted end to end (`gate-kart-service-valid31.log`).
- The kart_entry private capture re-encode gate still passes (previous review
  bundle, unchanged).
- New read-only probe over the pinned capture: `probe-gate-lap-mapping.py`,
  output `gate-lap-mapping.json` (index histogram, per-hit lap mapping,
  consecutive steps, revisit pairs, op35/op150 export status).

## Residual risk

No native run exists: the service is a capture re-encode plus synthetic
vectors, the gate positions are client claims, and the gate volumes are
reconstructed. The lead owns the native gate and the money registry; nothing in
this review proves a payable race.

Evidence bundle: `lab/reports/kart-service-independent-20261009/`.
