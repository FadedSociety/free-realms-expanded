# Harvesting runtime and border review

Status: independent read-only review of the harvesting runtime tile guard and the padded border
repair, run 2026-10-09 14:24-14:35 UTC. Frozen hashes at review time:
`src/harvesting.rs` `5f94acd5c57cb85f…`, `src/gameplay.rs` `263375fc3f7ad656…`,
`src/mining.rs` `6c59ae0c…`, `tests/harvesting_border.rs` `27aefba5…`,
`tests/harvesting_runtime_tiles.rs` `cb348836…`.
Verdict: **both repairs hold; one narrow runtime gap is demonstrated with a real counterexample,
and the smallest additional gate is proposed without claiming the original shape.**

Owned paths written by this review:

| Path | Role |
| --- | --- |
| `notes/Reviews/Harvesting Runtime and Border Review.md` | this review |
| `lab/reports/harvesting-runtime-border-review-20261009T142450Z/` | probe crate, run logs, hash manifest |

No source, content, save, UI, release or process file was edited. The builder's own border GREEN log
was blocked by an Application Control policy on a temporary target directory; this review did not
work around the policy. Instead both public checks were re-run with the established project target
(`scripts/cargo.ps1`, `prototype/osfr-tools/target`), which the policy allows, and the worker targets
passed normally (see below).

## What was independently verified

| Check | Method | Result |
| --- | --- | --- |
| Writer output shape | real `harvesting::write_content(dir, 111)` | 11x11; row 0 all `0`; column 0 all `0`; 90 playable `1`s at x1..10 / y1..9; drop row 10 = `0` then ten `4`s |
| Preserved parameters | same output | 8 tile kinds, 4 goals (quota 15 / per_reward 5), 4 outputs, seed 111, `FarmingAssets.txt`, `game_farming.gfx`, Chef profile 45 |
| Real engine run | `Type8Engine::load` + recovered handshake on the writer output | run reaches `Playing`; generated board row 0 and column 0 stay `0`, every playable cell is populated |
| Writer guard at runtime | `Gameplay::load` on the frozen 4-kind package | refused with `harvesting tile registration: …`; the persisted file is byte-identical afterwards |
| Legacy / Mining compatibility | `Gameplay::load` on the frozen Mining package and on a copy with `harvesting.json` removed | both load normally with `harvesting: None` |
| Worker targets | `scripts/cargo.ps1 test --test harvesting_border --test harvesting_runtime_tiles` | **6 passed, 0 failed** (3 + 3) |
| Review probe | own crate, four tests against the public API | **4 passed, 0 failed** |

## Finding: the runtime guard does not refuse unpadded geometry

**Counterexample (actual path).** A persisted `harvesting.json` that registers all eight kinds
(so `validate_tile_kinds_parts` is satisfied) but keeps the pre-border 10x10 shape - playable
x0..9 / y0..8, drop row 9 - still loads through `Gameplay::load`. The review probe writes the
builder's own pre-border eight-kind export
(`lab/reports/harvesting-tile-registration-20261009-1351Z/fresh-content/harvesting-111.json`) over
the package content and the load returns `Ok`; the file is unchanged by the attempt
(`unpadded_eight_kind_persisted_config_still_loads`). `src/gameplay.rs:369` reuses
`harvesting::validate_tile_kinds_parts(&spec.selectable, &spec.collectable, &tiles)` only, so the
border rule lives in the writer (`padded_engine_shape`) and not on the load path.

**Why it matters.** The border exists because the original `ServerGrid.as`/`Grid.as` fall loops use
`while (… > 0)` and never visit index 0 (`notes/Research/Harvesting Border Compatibility.md`). A
foreign, hand-edited or older config with eight correct kinds but index-0 playable cells would load
and then behave unlike the original client. The current writer output is padded, so this is not a
regression of the shipped content - it is a load-path gap for any content the writer did not
produce.

**Smallest additional gate (proposal, root's call - not implemented here).** After the existing
tile-kind check, derive the board bounds once (the engine already computes `left/top/width/height`)
and refuse when the derived playable rectangle touches index 0, i.e. `left == 0 || top == 0` while
any playable cell exists. That is a condition on existing values, not a template match: it does not
assert the original 11x11 padded shape, does not forbid other borders, and does not attempt original
geometry fidelity, which remains unrecovered. Equivalently, refuse when row 0 or column 0 carries a
code other than `0` (empty) or `4` (dropzone).

## Limits

- The original level template (`MatchThreeLevelData.txt`) is absent, so the original grid dimensions
  and border values remain unknown; the 11x11 padded shape is the accepted minimal reconstruction,
  not recovered original geometry.
- The counterexample is a persisted-content path only; no live save, package or client run was
  touched. No native acceptance is claimed by this review or by the repaired modules.
- The builder's blocked GREEN log (`harvesting-border-fix-20261009-1417Z/green-tests.log`) is
  superseded by this review's normal-policy runs, which are recorded here; the policy itself was
  left untouched.

Skills applied: **RTK** (normal commands, condensed evidence), **ponytail** (one counterexample,
one minimal gate proposal, no wider rule), **caveman** (terse reporting; the persisted review stays
complete), **asset-pipeline** (the border/geometry question is asset-readiness against the client's
own fall loops). `game-automation` was not applied - the native window stays with the root.

Cross-links: [[Harvesting Border Compatibility]], [[Harvesting Tile Registration]],
[[Harvesting Tile Registration Review]], [[../Design/MMO Worker Contract]].

## Follow-up review: runtime edge gate (2026-10-09 14:45 UTC)

The demonstrated counterexample is fixed. New frozen hashes at follow-up time:
`src/harvesting.rs` `a977c46678aaf55e…`, `src/gameplay.rs` `d65f89a87cccb122…`,
`src/mining.rs` `6c59ae0c…`, `tests/harvesting_runtime_border.rs` `3ce83c9fe1ae9172…`.
The new immutable package `lab/references/opening-harvesting-native-20261009T142624Z` carries 45
files with `content/harvesting.json` = `c08f264e898a5e8703690c8c02ee4cfce277e271efbf4170674c0e3b8ab4d30f`.

**What the new gate is.** `harvesting::validate_border` refuses any shape code on row 0 or column 0
that is neither empty (`0`) nor a dropzone (`4`), with the message naming the index-0 border. It is
reused at runtime by `gameplay.rs:373` immediately after the tile-kind check, so a persisted
document is refused before any offer. It is a two-condition edge rule, not a template match: it does
not assert 11x11, does not fix the border width, and does not claim the unrecovered original shape.

**Probe 2 (own crate, `probe/tests/runtime_border_followup.rs`).**

| Check | Result |
| --- | --- |
| Preserved pre-border eight-kind export (`29538eb4…`) written over the new package copy | refused with a border message; whole-tree fingerprint unchanged |
| Column-0-only variant (10x11, rectangle at x0..9) | refused; message names `x 0` |
| New package content (`c08f264e…`) | loads with an active harvesting state |
| Non-template padded geometry (12x12, border 2, same 10x9 rectangle) | loads, unchanged files - the gate is not an exact-template overmatch |
| Mining package and a legacy copy without `harvesting.json` | both stay permissive (`harvesting: None`) |

The executed run is `followup-probe-run.log` (**2 passed, 0 failed**). The worker's own new target
was re-run under the standard project target and passed (`worker-runtime-border-run.log`,
**2 passed, 0 failed**; matching the builder's RED 1/1 -> GREEN 2).

**Qualification of the earlier section.** The four-test probe from the first pass, including
`unpadded_eight_kind_persisted_config_still_loads`, documented the pre-fix runtime state. That
counterexample is now refused by `validate_border`; the earlier probe remains valid evidence of what
the runtime did before this gate and should not be read as the current behaviour.

**Package gate (read-only, `package-gate.py` / `package-gate.json`).** Against the previous package:
41 files identical, 1 changed (`content/harvesting.json`), 3 added (`STAMP-v2.txt`,
`harvesting-v2-proof.json`, `harvesting-v2-receipt.json`), 0 removed. The campaign catalogue and
every other content file are byte-identical; the new `harvesting.json` matches the reviewed
contract (11x11, inert row 0 and column 0, 90 playable cells at x1..10 / y1..9, drop row 10, eight
kinds, quotas 15 / per_reward 5, outputs `2885/2887/2890/2877`, Chef 45, farming assets).

**Unexecuted check (policy, not bypassed).** A third follow-up test that would assert the shipped
`harvesting.json` is byte-identical to a fresh writer run could not execute: after the test binary
was rebuilt, the Application Control policy blocked it (`os error 4551`) at
`prototype/osfr-tools/target/debug/deps/runtime_border_followup-….exe`. No rename, copy or other
workaround was attempted. The package gate above substitutes a static structural comparison; the
byte-identity check remains for the root's normal focused run.
