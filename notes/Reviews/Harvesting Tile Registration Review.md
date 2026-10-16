# Harvesting tile registration review

Status: independent read-only review of the harvesting tile-registration repair, run
2026-10-09 14:04-14:20 UTC. Frozen source verified at
`src/harvesting.rs` sha256 `9435797dcced2cc8a2c16839455d24b75ad38f006bb27f303c6376b8ee03fcec`
and the authorized test update at `tests/harvesting.rs` sha256
`52e946f194b4b4b1538edc9e3b7f426f6449f62cd27f1e9515447bff199d684d`.
Verdict: **the repair holds under independent re-derivation; one scope disclosure is recorded
(finding 3) and no native acceptance is claimed.**

Owned paths written by this review:

| Path | Role |
| --- | --- |
| `notes/Reviews/Harvesting Tile Registration Review.md` | this review |
| `lab/reports/harvesting-tile-registration-review-20261009T140426Z/` | probe, run log, hash manifest |

No source, content, save, package, process or UI file was edited. The old content package
(`opening-harvesting-native-20261009T132512Z`) is unchanged (verified below).

## What the repair does

The original client names a tile only through `core/TileGenerator.as::SetTileInfo(value, index)`
feeding `aNames[value]`, which `generate_selectable`/`generate_collectable` dereference
(`lab/decompiled/minigames/harvesting/scripts/scripts/__Packages/core/TileGenerator.as` lines 127-140).
The previous package registered only the four collectables (601..604), so the four regular
selectables (401..404) were drawn from an undefined `aNames` entry and the native gate was blocked.
The repair registers the regular kinds first - value 401..404, category 400, icons 1..4 - then the
regional collectables, preserving the recovered order and per-region icons; the shared engine emits
`OnTileInfoMsg<value>\t<value - category>` per entry (`src/mining.rs:1123-1131`), which for both
groups yields the client's indices 1..4.

## Independently verified

All checks below were re-derived in the review's own probe
`lab/reports/harvesting-tile-registration-review-20261009T140426Z/probe` (3 tests, path dependency
on the frozen crate), not taken from the builder's logs.

| Check | Method | Result |
| --- | --- | --- |
| Frozen hashes | sha256 of the three files | match the payload exactly (`9435797d...`, `1f451b0d...`, `52e946f1...`) |
| Eight-kind contract, six regions | real `harvesting::content(region)` for 109/111/113/115/117/119 | every region emits exactly 8 kinds: `(401,400,1)..(404,400,4)` in order then `(601,600,x)..(604,600,w)` in order; goal icon equals its tile icon; four distinct icons; four distinct bundles |
| Preservation | field-by-field compare of the frozen writer against the builder's `fresh-content/harvesting-<region>.json` | identical for `tiles`, `goals`, `outputs`, `selectable`, `collectable` and every header field (all six regions) |
| Region 111 contract | goal/output/produce check | goals quota 15 / per_reward 5 over 601..604; outputs `2885/2887/2890/2877`; seed = 111; duration 180; `FarmingAssets.txt`; `game_farming.gfx`; `reconstructed_rules: true` |
| Guard: missing kind | `validate_content` on a spec without 401 | refused: `value 401 has no registered tile kind; the client dereferences aNames[value] and cannot draw it` |
| Guard: duplicate kind | `validate_content` with a repeated 401 | refused: `duplicate tile kind for value 401` |
| Guard: wrong category | `validate_content` with 401 carrying category 600 | refused: `tile kind 401 carries category 600 but its value implies 400` |
| Guard: out-of-set kind | `validate_content` with an extra value 499 | refused: `tile kind 499 is neither selectable nor collectable` |
| Guard runs before the writer | `write_content` for the unsupported generic region | returns an error and creates no `harvesting.json`; a supported region writes and re-validates |
| Old package immutable | sha256 of `content/harvesting.json` in the 20261009T132512Z package | still `280f5cd7fcca711aaf7e8118ef04195b3bb1241413498325288aca10dff94d20` |
| Fresh export for region 111 | sha256 of `fresh-content/harvesting-111.json` | `29538eb4970743c3eac5f11d62782e492279dce5d22303033229b6c70f663699` (the "11129538" export) |

The builder's own evidence was read but not relied on: `receipt.json` (17 + 9 + 4 + 3 + 4 + 11 +
24 + 4 + 3 = 79 passed, 0 failed), `red-tests.log` (1 passed / 7 failed before the fix) and
`green-tests.log` (9 passed) are consistent with the frozen files, and the RED failures name exactly
the four defects the guard now covers.

## Findings

1. **Repair is correct and minimal.** The change adds only the four regular kinds and the guard; the
   collectables, their icons (region 111: 715/713/717/712), their order, the goals/quotas, seed,
   rewards, assets and profile are byte-identical between the frozen writer output and the previous
   package's contract. Region 109's icons (706/710/667/707) are its own recovered produce set, which
   is why the updated `tests/harvesting.rs` expectation differs from region 111's - both are
   correct for their region.
2. **The authorized test update tightened, not weakened, the assertion.** `tests/harvesting.rs`
   now pins the exact eight `(value, category, icon)` tuples and the `OnTileInfo` index mapping
   instead of a loose subset check; the previous hash recorded by the builder is
   `b01020e483377cc85c4f85a4296a95b27450e6dd18a7e61935caa8004d8bc96a`.
3. **Scope disclosure - the guard is writer-side only.** The shared `Type8Engine::load`
   (`src/mining.rs:253`) reads and binds the content file without calling
   `harvesting::validate_content`. The review probe demonstrates it: a spec with the regular kinds
   stripped is refused by the harvesting guard yet still loads through the engine (`3 passed`,
   including `generic_engine_load_bypasses_the_harvest_guard`). The repair therefore guarantees that
   the *writer* cannot emit an unregistered set; it does not protect an already-written or
   hand-edited package at load time. A gateway/engine-side guard remains available as a follow-up
   (root's call); nothing in this repair claims otherwise.
4. **No native acceptance.** The registration now matches the client's own `aNames`/`SetTileInfo`
   contract and the engine's `OnTileInfoMsg` mapping at the content and code level, but no client
   run was made by the builder or by this review, and the previously blocked package remains the
   only content package on disk. Rendering proof still requires a native run.

## Commands and results

Run from `%USERPROFILE%\Documents\Projects\CartoonMMO` with the private toolchain
(`lab/tools/cargo`, `CARGO_TARGET_DIR` inside the review directory).

| Command | Result |
| --- | --- |
| `cargo test --test tile_registration_review` in the review probe | **3 passed, 0 failed**: `writer_emits_eight_pairs_for_every_region_and_matches_the_export`, `writer_guard_refuses_each_broken_registration_before_writing`, `generic_engine_load_bypasses_the_harvest_guard` |
| sha256 checks over the three frozen files, the old package content and the fresh export | all match the payload / builder receipt |

Skills applied: **RTK** (normal commands, condensed evidence), **ponytail** (three focused tests
against the real public writer/guard/engine API instead of new machinery), **caveman** (terse
reporting; the persisted review stays complete), **asset-pipeline** (the tile/registration check is
the game-asset readiness question: the client's own movie-name table is the authority for what can
be drawn). `game-automation` was not applied - the native window stays with the lead and this review
makes no native claim.

Cross-links: [[Harvesting Tile Registration]], [[Harvesting Native Content Review]],
[[Harvesting Native Content]], [[../Design/MMO Worker Contract]].
