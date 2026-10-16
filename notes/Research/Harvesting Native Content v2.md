# Harvesting Native Content v2

Private immutable harvesting content package rebuilt from the frozen v1 after the
source/runtime/border work completed. Read-only for the repository: no source,
save, UI, release or process was changed; the only edits live in the new package
plus this note and the private report bundle.

| | |
| --- | --- |
| Package | `../lab/references/opening-harvesting-native-20261009T142624Z/` |
| Source package (unmutated) | `../lab/references/opening-harvesting-native-20261009T132512Z/` |
| Evidence bundle | `../lab/reports/harvesting-native-content-v2-20261009T142137Z/` |
| File proof | 41 identical / 1 modified (`content/harvesting.json`) / 3 added metadata files; v1 re-hashed unchanged (`v2-file-proof.json`) |
| Inventory | `0b95b0b1c032a2e0f864f8227bfe693fe58181863eaf182d35b6b9dbc4d9877a` (1703 definitions, byte-identical) |
| `content/harvesting.json` | `280f5cd7…` -> `c08f264e898a5e8703690c8c02ee4cfce277e271efbf4170674c0e3b8ab4d30f` |
| Writer | current `harvesting::write_content`; source `a977c46678aaf55e…`, binary `4744348b58d69251…` |

## What changed

The writer's current reconstruction emits an 11x11 grid whose row 0 and column 0
are `0` (inert under the original `checkFalls`/`addColumnFall` loops), 90 regular
cells at x1..10 / y1..9, and the drop row at y10 over x1..10. Eight tile kinds
are registered (401..404 category 400, 601..604 category 600). Unchanged from
the writer's previous reconstruction: quotas 15 / per-reward 5, seed 111,
outputs `[2885, 2887, 2890, 2877]`, profile 45, `FarmingAssets.txt`,
`game_farming.gfx`, duration 180 s, spawn percentages and meter 0.

v1 -> v2 leaf diff (`changed_spec_keys`): `width`, `height`, `shape`,
`tiles` (4 -> 8 registered kinds) and `rule_notice`; every other field is
byte-equal. The padded border and its notice are reconstructions over the
recovered `x > 0` / `y > 0` loop evidence, not recovered geometry.

## Validation (private probe, real package)

`probe/src/main.rs` writes the file with the verified writer and then runs the
preflight against the real package. All checks passed (`v2-validation.json`):

- `Gameplay::load` accepts the package; the four outputs resolve exactly once in
  the 1703-definition inventory.
- A gateway win through the real inventory grants each of the four real bundle
  ids exactly once (coins unchanged, 7 -> 11 rows, `completed` 0 -> 1,
  profiles byte-equal); a forged `FRServer_GameWon` grants nothing and a
  duplicate `OnNextLevelMsg` changes nothing.
- A full bag fails the win frame, leaves the snapshot byte-identical with the run
  still `LevelComplete`, and the retry commits after space.
- `game_store` save/reload keeps exactly one of each bundle, rebinds the same
  instance, and a reloaded `OnNextLevelMsg` grants nothing.

Harvesting family rerun at the frozen revision: `harvesting_border` 3,
`harvesting_runtime_border` 2, `harvesting_runtime_tiles` 3,
`harvesting_tile_registration` 9, `harvesting` 17, `harvesting_review` 4,
`harvesting_job_binding` 3, `harvesting_reward_catalogue` 4 and
`type8_harvesting_gateway` 11 — 56 passed, 0 failed (`family-tests.log`).

## Limits

- Geometry, quotas, cadence and rewards remain reconstructed; this note claims no
  native or historical fidelity. Native acceptance stays with the lead.
- The v2 package is immutable and separate; shared content, saves, launcher,
  release and UI are untouched (root-controlled).
