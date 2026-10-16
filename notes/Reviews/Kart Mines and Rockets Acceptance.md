# Kart mines and rockets acceptance

October 10, 2026. Native firing and saved-wire lifecycle accepted on v20. Mine visuals after owner movement, explosions and hits remain open.

## Implementation and review

- Recovered exact layouts for create/update/destroy missile24/26/25, mine29/31/30, and explosion27/28. Production codecs preserve unknown fields explicitly.
- Lead corrected research errors by tracing called consumers: create matrices are owner-local; missile updates are absolute world matrices; mine updates use inverse-owner translation. Translation slots12/13/14 are proven from the known pickup matrix. Missile update slot15 overlaps the linear velocity triple and is zero in this pure position stream.
- Opt-in `weapon_effects` keeps older content behavior. FIFO use requires a validated session-bound pose before consuming a weapon. Guards cover invalid basis, NaN, future/regressed clocks, implausible movement, foreign owners, actor capacity, allocation history and saved lifetimes. The invented one-second pose expiry was removed in v20 after native idle-cadence evidence.
- Reconstructed tuning: rocket35m/s and4s lifetime; mine12s lifetime; updates at most10Hz; ±3m placement. These are local choices, not original balance measurements.
- Explosion codecs exist, but lifecycle emission is deliberately pending the unknown effect arguments. No hit/knockback relay is enabled. A solo race cannot establish PvP hits.
- Two DeepSeek Flash/max workers handled scoped research/code. Lead reviewed deeper consumers, interrupted finished research, took over final implementation fixes and tests, and integrated the production module.

## Automated gate

Private evidence: `../../../lab/reports/kart-weapons-probe-v18-20261009/`.

- Focused84passed/0failed/1ignored; full1729passed/0failed/82ignored; nine existing private gateway race/reward checks passed.
- All15 weapon asset dependencies resolve through the production native delivery reader with size/CRC/hash checks.
- Layout checker: eight layouts/412payload bytes/70citations; corrected consumer checker:85citations.
- Release SHA256: `347c85e8aa0047548c676e774967e10642f9f9c9974fb04f8e0d78c53c77f966`. Source/log/content fingerprints and executable copy are frozen in `build-receipt.json`.
- Preserve the worker's failed initial log; lead's final test evidence is separate. Initial service regression tests failed before behavior was implemented.

## Native diagnostic

The v18 test failed. the owner tried left-click and Ctrl: 17 C33/19 item-use requests
reached the server, but no S33/20 activation or weapon actor frames were returned.
The validated wire audit found direct C33/6 poses while saved `weapon_pose` remained
null. Binary session dispatch handles reliable gameplay only; direct one-byte
tunnel datagrams are not forwarded to the gateway. The fresh-pose guard therefore
correctly refused weapon consumption. The claim that left-click is unbound is
not established by this capture and is retracted. Input diary and failed captures
are preserved. Harness finalized with client/login/server stopped; native success
remains unchecked.

Next repair: test-first narrow CRC/bounded-compression decoder for direct C33/6,
session-bound gateway dispatch, forged direct-use rejection, then a new v19 frozen
build and native rocket/mine test. Do not reuse v18 automated passes as proof of
the transport change.

Run `client-baseline-20261010T054758Z`, copied paid character359coins/25Kart DriverXP. Two boxes beside the starting grid carry rocket1 and mine3; the remaining layout and fixed mixed drops are diagnostic. This is not the canonical15-position full-race acceptance.

- [x] Rocket activation/flame visible in the original client; saved wire proves forward travel and exactly-once expiry. Continuous visual flight is not claimed.
- [ ] Mine visible, original arming animation, position stays where dropped after owner movement, expires once.
- [x] Independent saved-wire audit, frame hashes/CRCs, normal exit, unchanged inventory and XP on v20.
- [ ] Explosion effect arguments and actual weapon hit/knockback acceptance with a live rival.

Controls: `realm race` in local chat, one Go click, Alt uses the visible head item. Ctrl also produced C33/19 requests in the diagnostic; its activation depends on the same validation.

## Direct transport and quiet placement correction, v19/v20

v19 adds a CRC-verified, bounded-compression direct C33/6 decoder. Only that exact
pose shape is admitted through the established UDP session; item use stays in
the reliable stream. Dispatch retains save-before-commit/reply behavior. Three
synthetic decoder tests and a private real Session regression pass; removing the
dispatch branch reproduces the private test failure. v19 full gate1732/0/83,
private kart9/0, and Session1/0.

Finalized v19 run062127 uses frozen19f76222. Lead observed Alt rocket activation
and a flame ahead of the kart. Later shared inputs supplied additional actors:
the independent audit verifies three rockets (37/38/38updates, ~138m travel,
~4s lifetime) and two mines (110updates each, ~12s lifetime), all47,560 frame
hashes and47,539gateway CRCs, no unchecked controls, inventory/XP unchanged,
and all processes stopped. Other actor visuals and native hits are not claimed.
The harness reached its600-second bound; do not claim normal menu exit.

This run exposes the invented one-second placement expiry: valid quiet C33/6
poses are separated by42.703/32.812/29.750/28.532seconds at essentially the
same position. Cadence evidence and decoder are under the v19 report. A read-only
DeepSeek review independently confirms sparse stationary traffic and no recovered
pose TTL; the precise runtime sender gate remains unresolved. No fixed30-second
heartbeat is claimed.

v20 keeps the last accepted session placement for visual actor creation and mine
updates. New poses still require finite fields, valid basis, allowed controls,
host word and bounded movement; future stamps refuse. Race window, ownership,
actor bounds and cleanup/reconnect guards remain. Cached placement cannot prove
hits or justify damage. The quiet45-second-use regression fails before this
correction and passes afterward for both rocket and mine. Full1733/0/83,
focused88/0/1, private kart9/0, Session1/0. Release frozen:
`d481865ceed01dbde67adcca2e72e94e35eb7926aa22feb0ff003b01736fe53b`.
Private reports: `../../../lab/reports/kart-weapons-probe-v20-20261010/`.
An initial private check used the wrong canonical v15 fixture; the failed log is
preserved, correct grid diagnostic fixture passed. Canonical saves/content remain preserved.

## Fresh native v20 firing acceptance

Finalized run `client-baseline-20261010T064034Z` uses the exact frozen release
above. The lead clicked the track with rocket at the toolbar head and observed
activation/flame ahead and the toolbar advancing. Ctrl with mine at the head
also advances the toolbar. The mine itself is behind the kart and was not
visually accepted. Two client C33/19 uses receive two S33/20 responses and one
create/update/destroy lifecycle for each weapon. Diary and original screenshots
are preserved in the run's `manual-ui/` directory.

The independent wire audit proves rocket37updates,136.220m translation and4.000s
lifetime; mine110updates, zero owner-local translation change while idle and
12.000s lifetime. Every28,619 frame hash and28,600gateway CRC verifies; no
unchecked controls. Inventory, XP and359coins remain unchanged. Escape, Exit
Game, Yes closes the client with exit code0 after421.735seconds; the harness
finalizes with client/login/server stopped. This is a normal menu exit, not the
600-second timeout seen in v19.

Independent `audit-quiet-use.py` correlates each use/create with the latest
CRC-verified direct pose: rocket14.875seconds old, mine30.437seconds old, both
at the same stationary position. This is concrete native proof of the idle
correction. It does not establish an original heartbeat interval.

Reproduce from the project directory with the private Python:

```powershell
$py = '../lab/tools/wiki-env/Scripts/python.exe'
$report = '../lab/reports/kart-weapons-probe-v20-20261010'
$run = '../lab/runs/client-baseline-20261010T064034Z'
& $py "$report/audit-run.py" $run
& $py "$report/audit-weapons.py" $run
& $py "$report/audit-quiet-use.py" $run
```

Receipts are under the v20 report's run-named subdirectory. The original build
receipt remains immutable and therefore retains its pre-playtest pending fields;
the native receipts and this section record the later result. Only two starting
grid diagnostic pickups were tested; this does not accept the canonical track
layout. Mine arming/placement after owner movement, explosion arguments and
actual hits/knockback against a live rival remain unfinished. No damage or
explosion emission is enabled.
