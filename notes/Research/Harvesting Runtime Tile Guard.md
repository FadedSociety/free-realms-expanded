# Harvesting Runtime Tile Guard

Status: implemented; RED and GREEN both captured, 2026-10-09 (UTC). Receipt:
`lab/reports/harvesting-runtime-tiles-20261009-1414Z/`. Evidence label: **RECONSTRUCTED runtime gate
over ORIGINAL client tile-generator semantics**. No native claim.

Owned paths for this guard:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/gameplay.rs` | optional Harvest load gate before any offer |
| `prototype/osfr-tools/src/harvesting.rs` | `validate_tile_kinds_parts` slice entry (visibility + wrapper only) |
| `prototype/osfr-tools/tests/harvesting_runtime_tiles.rs` | 3 vectors (RED first) |
| `lab/reports/harvesting-runtime-tiles-20261009-1414Z/` | RED/GREEN/lib logs, receipt |

## Why

`notes/Research/Harvesting Tile Registration.md` fixed the *writer*: the emitted content registers all
eight values (401..404 at category 400, 601..604 at category 600). The shared engine stays permissive
by design (Mining's contract), so a *persisted* `harvesting.json` that advertises
`selectable = [401..404]` while registering only the collectables still parsed and loaded: the client
only learns a value in `core/TileGenerator.as::SetTileInfo` and `generate_selectable` dereferences
`aNames[value]`, so every regular tile would silently stay undrawable.

## RED (baseline, before the gate)

`cargo.ps1 test --test harvesting_runtime_tiles` on the pre-gate source:

```
test valid_eight_kind_content_loads ... ok
test missing_content_legacy_and_mining_stay_permissive ... ok
test malformed_persisted_harvesting_configs_are_refused_without_mutation ... FAILED
missing regular 401: a malformed tile registration loaded without refusal
```

The RED proves the four malformed persisted configs (missing regular, duplicate, wrong category,
out-of-set kind) all loaded through `Gameplay::load` today.

## Change

- `harvesting::validate_tile_kinds_parts(selectable, collectable, tiles)` is the existing
  registration check expressed over plain slices (the typed `validate_tile_kinds(spec)` is now a
  thin wrapper), so `gameplay.rs` reuses one parser instead of duplicating it.
- `Gameplay::load`, inside the optional Harvest branch and after the profile/asset checks, calls that
  helper on the engine's parsed spec before `Some(engine)` is returned; a refusal is an
  `InvalidData` error naming the tile registration. No geometry, quota, seed, reward or content
  field changes; the generic engine is untouched (Mining keeps its permissive contract).

## GREEN

| Command | Result | Log |
| --- | --- | --- |
| `cargo.ps1 test --test harvesting_runtime_tiles` | 3 passed, 0 failed (each refusal also asserts the save/content tree is byte-identical) | `green-tests.log` |
| `cargo.ps1 test --lib` | 24 passed | `lib-tests.log` |

The test also pins the two permissive cases: a campaign with no `harvesting.json` still loads
(legacy), and a Mining-only content directory still loads with `harvesting` absent.

## Owner action (outside this guard's scope)

`tests/type8_harvesting_gateway.rs` builds a synthetic Harvest fixture whose `spec()` registers only
the four collectables (`tiles: VALUES…category 600`). The new gate correctly refuses it, so that
target now reports 9 failures whose cause is exactly
`harvesting tile registration: content: value 401 has no registered tile kind` (captured in
`harvesting-family.log`). Its owner needs one line in that fixture: prepend the four regular kinds
`(401..404, category 400)` to `spec().tiles` — the same contract the writer emits. That file is
outside this task's ownership and was left untouched.

## Limits

- This gate runs at `Gameplay::load`; it does not change the shared engine, so a caller that builds a
  `Type8Engine` directly (tests, probes) still gets the permissive behaviour.
- The refusal is a load error only: no save is written, no packet is sent and no offer is reached.
- The current `src/harvesting.rs` also carries the separately-receipted border fix
  (`notes/Research/Harvesting Border Fix.md`); this receipt's GREEN ran on the guard-only revision.
