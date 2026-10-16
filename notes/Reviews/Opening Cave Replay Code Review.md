# Opening Cave Replay Code Review

Independent review (2026-10-09, reviewer workspace `/root/mining_review`).
Scope: the post-campaign opening-cave replay repair, reviewed read-only; no
production source, test, shared note, process, save or UI was touched. Owned
artifacts only: this note and the private bundle
`lab/reports/opening-cave-replay-review-20261009T115404Z/`.

## Reviewed revision (hashes verified in-session)

| File | SHA-256 | Claimed | Match |
| --- | --- | --- | --- |
| `prototype/osfr-tools/src/gameplay.rs` | `032ead50cbe92e9c4c91c2b20391f2c532c7e382a5e757f884f64d6c96dce904` | `032ead50…` | yes |
| `prototype/osfr-tools/src/opening.rs` | `0c2859b6d9961b725835257ad457f0d9b6945ddd8675bff5d00e18072af5b921` | `0c2859b6…` | yes |
| `prototype/osfr-tools/tests/opening_cave_replay.rs` | `3c6791eed7fc96b8ffc1397c2514b359c74201270d4d16486f68e23f48823a09` | `3c6791ee…` | yes |
| pre-task `gameplay.rs` (prerepair copy) | `fd1a9a1d026640cc17ad9a50aa9abe79f3df8891d17d86fb87ee800f0296fa72` | `fd1a9a1d…` | yes |
| `target/release/osfr-gateway.exe` | `4d7226e066d9262a211dd9ebb58f8f0d4310dab2da826bce0e8898ca31b902bc` | build hash | yes (artifact hash) |

Provenance of the baseline: the prerepair `gameplay.rs` copy hash equals the
`src/gameplay.rs` hash recorded by the previous independent review
(`notes/Reviews/Type8 Gateway Integration Handoff.md`, line 7), and
`src/opening.rs` is byte-identical between prerepair and shipped code. The
shipped diff is bounded to `gameplay.rs` (81 insertions / 25 deletions) plus
the new test file; `opening.rs` adds one read-only accessor (`stages()`).

## Verdict

The replay gate is content-derived and fails closed on every tested refusal
path; entry rebuilds a real new instance with all enemies at content max
health; repeat clears grant nothing; the original pending-objective
(stage-2 first entry) path and the magic-door branch are unchanged. No
materials finding. Native-client acceptance is out of scope for this review
and remains lead-controlled.

## What was independently verified

Pinned-snapshot evidence timeline (all times UTC, 2026-10-09):

| Time | Evidence |
| --- | --- |
| 11:54Z | input hashes verified: `gameplay.rs 032ead50…`, `opening.rs 0c2859b6…`, `tests/opening_cave_replay.rs 3c6791ee…`, prerepair `fd1a9a1d…`, artifact `4d7226e0…` |
| 11:58:31Z | shipped `tests/opening_cave_replay.rs` 5 passed / 0 failed |
| 11:59:34Z | private content variants generated (`logs/00-make-variants.log`) |
| 11:59:39Z | independent probe 5 passed / 0 failed |
| 12:00:17Z | source checks: no hardcoded id; prerepair→shipped diff 81 insertions / 25 deletions |
| 12:00:25Z | adjacent set: opening 7(+2), opening_gateway 0(+1), encounter 15(+2) |
| 12:03–12:06Z | concurrent Mining/score drift lands (see next section) |
| 12:06:23Z | full-suite re-run 1397/0/57 over 141 binaries (overlapped drift window) |
| 12:07:04Z | review release rebuild `c6405d26…` from drifted source |

Source (read directly, not accepted from the builder report):

- `completed_dungeon_replay` ([gameplay.rs:1625]) reads the stages already
  parsed by `OpeningEngine` via `stages()` and derives giver, stage and
  encounter from the `dungeon_completed` objective; no guid, stage id or
  encounter id literal exists in the file (`logs/04-source-check.log`: no
  `411`, no giver guid).
- The branch is reachable only for the ordinary `26/0/10/0` selection-1
  variant, in range, with the encounter inactive
  ([gameplay.rs:903-931, 1064-1098]).
- `handle_in_world` executes on a staged clone and commits `*self = staged`
  only on success ([gameplay.rs:549, 631]), so every refused/erroring frame
  leaves the whole persisted state untouched.
- `EncounterEngine::begin` allocates a new instance id, resets player health
  and rebuilds all enemies from content ([encounter.rs:333-370]); completion
  rewards are emitted only when `rewarded.insert(id)` is new
  ([encounter.rs:784]); `EncounterState` validation rejects rewarded ids that
  are not real dungeons ([encounter.rs:255]), so the gate's reward proof
  cannot be satisfied by an unknown encounter.

Behavior (private probe crate, `probe/tests/review_probe.rs`, 5/5 passed;
log `logs/02-review-probe.log`):

1. `review_refusals_preserve_whole_state_and_do_not_wedge_the_gate` — wrong
   guide, selections 0 and 2, out-of-range, and two malformed frames all
   leave the full serialized `GameplayState` value-identical; the correct
   guide still starts the run afterwards.
2. `review_pending_objective_still_starts_the_first_run` — the pre-existing
   stage-2 path (accepted stage, dungeon objective unmet) still starts the
   bound run at content max health and an active run is never replaced.
3. `review_damaged_run_and_repeat_clears_are_inert` — after damaging the boss
   and leaving through `realm exit`, the next replay gets a new instance id
   and every enemy back at content max health; two repeat clears leave
   inventory, stage progress, rewarded set and profile/XP values unchanged,
   including after closing the results screen.
4. `review_variant_content_rebinds_giver_and_dungeon` — private content
   variant (giver moved to actor `…059`, objective rebound to a new dungeon
   `412` with enemy health 111/222/333/444): the replay follows the content
   exactly, and the original giver can no longer start it.
5. `review_ambiguous_binding_fails_closed` — private content variant with one
   giver owning two stages bound to two different dungeons: no replay, state
   unchanged.

Regression targets (log `logs/03-adjacent-set.log`): `opening` 7 passed
(+2 ignored), `opening_gateway` 0 passed (+1 ignored, ignored by design),
`encounter` 15 passed (+2 ignored). The shipped replay suite re-run:
`tests/opening_cave_replay.rs` 5 passed / 0 failed
(`logs/01-shipped-opening-cave-replay.log`).

Full suite re-run (log `logs/06-review-full-suite.log`): 1397 passed / 0 failed /
57 ignored across 141 binaries (EXIT=0), matching the lead-reported count.
Boundary: the run window (12:01–12:06Z) overlapped the concurrent Mining/score
edits that landed 12:03–12:06Z, so it corroborates the count but is not a
pinned-revision-only run; the pinned-revision evidence is the replay targets
and probe executed 11:58–12:00Z. The lead re-runs the full suite after the
Mining handoff.

Release build: the pinned artifact `4d7226e0…` is preserved at
`lab/tools/osfr-gateway-after-replay-4d7226e0.exe` (verified byte-identical to
the production `target/release/osfr-gateway.exe`, 4 245 334 bytes,
11:44:45Z). A review rebuild (`cargo build --locked --release`, log
`logs/07-release-build.log`) produced `c6405d26…` from the drifted post-review
source (Mining hooks present), so the two hashes are not a reproducibility
comparison.

## Revision drift during review (explicitly uncovered)

The pinned snapshot was verified at 11:54Z. During the 12:03–12:06Z window a
concurrent writer changed the tree (per the root message: Mining optional-hook
integration in `gameplay.rs`/`lib.rs`/`mining.rs` and the encounter score
change). Current, **not covered by this review**:

| File | mtime (UTC) | current SHA-256 |
| --- | --- | --- |
| `src/gameplay.rs` | 12:06:13 | `8f0a0d2978423eddbece843f6065f93f7ee70b2b159f39a5b2a7fc26fd163177` |
| `src/encounter.rs` | 12:03:54 | `68cfc114b65ae9f8648ef94bf3a566789f772a988daba87c2c02c2cc2de4e65e` |
| `src/lib.rs` | 12:04:48 | `376f936ad4b8f5272d23df9848084db475332562f8c8390c957c156c45a14eb8` |
| `src/mining.rs` | 12:06:32 | `a66073ca463d84dc6e293a5ee3c8eef8ca861a40d23d7a6eb4acd0e4a0d6e4e2` |

`src/opening.rs` (`0c2859b6…`) and `tests/opening_cave_replay.rs`
(`3c6791ee…`) were unchanged by that drift.

## Claim boundaries

- `golden2` and `opening_gateway1` exist only as lead-reported private oracle
  runs. No such harness or test name is present in the workspace, and the
  `opening_gateway` and encounter-oracle tests are `#[ignore]` by design. That
  claim is therefore not independently reproduced here; the code-level and
  behavioral checks above are the independent evidence.
- The gate keys on story-stage completion, not on
  `OpeningEngine::complete()`: optional discoveries may still be outstanding
  when a replay starts. This matches the repair intent (post-campaign cave
  restart) and is noted as a boundary, not a defect.
- The pinned release artifact is identified by hash and preserved in the lab;
  byte-identical rebuild reproducibility was not proven (the mid-review
  rebuild ran from drifted source and is not comparable).

## Reproduce

```powershell
$env:CARGO_HOME='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\cargo'
$env:RUSTUP_HOME='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\rustup'
$env:PATH="$env:CARGO_HOME\bin;$env:PATH"
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\reports\opening-cave-replay-review-20261009T115404Z\target'
Set-Location '%USERPROFILE%\Documents\Projects\CartoonMMO\project\prototype\osfr-tools'
cargo test --locked --test opening_cave_replay
cargo test --locked --test opening --test opening_gateway --test encounter
Set-Location '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\reports\opening-cave-replay-review-20261009T115404Z\probe'
$env:REVIEW_CONTENT='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\references\opening-journal-text-v2\content'
$env:REVIEW_GATEWAY_REF='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\references\rust-gateway-reference'
$env:REVIEW_VARIANT_ROOT='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\reports\opening-cave-replay-review-20261009T115404Z'
cargo test --test review_probe -- --nocapture --test-threads=1
```
