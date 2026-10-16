# Harvesting Gateway Independent Review

Independent read-only review (2026-10-09, reviewer workspace
`/root/mining_review`). No production, test, shared note, save, UI, process or
release file was modified; owned artifacts only: this note and
`lab/reports/harvesting-gateway-review-20261009T132030Z/`.

## Reviewed revision (hashes verified in-session)

| Path | SHA-256 |
| --- | --- |
| `prototype/osfr-tools/src/gameplay.rs` | `9e8b56ed5f3a2f78ae029b4cf51172ba18169c7381f1bb43e176361706c9c1a7` |
| `prototype/osfr-tools/src/lib.rs` | `bb1d8755a8b6a60660faf38cd0961c1d75822578cfa632d64a370b5c1a06a322` |
| `prototype/osfr-tools/tests/type8_harvesting_gateway.rs` | `39f5f6a88fcaa3ee325346a0a15c32e527d2a11ee918c5f0e9fa8679a54934d7` |
| unchanged: `src/harvesting.rs` / `src/mining.rs` / `src/gateway.rs` | `272711da…` / `e6e421d8…` / `60110c14…` |
| preserved pre-edit: `gameplay.rs` / `lib.rs` | `077ef906…` / `376f936a…` (builder bundle `harvesting-gateway-20261009T150000Z/preserved-before-edit/`) |

The frozen diff is bounded: `gameplay.rs` +192/−2, `lib.rs` +1 (registering
`pub mod harvesting;`), plus the new gateway test file. The quest gather path
(the 2884/2875 grant block) is not touched by the diff; harvesting content,
mining engine and gateway are unchanged.

## Verdict

The gateway integration itself is correct and complete for a dev-gated
candidate: optional `GameplayState.harvesting` with serde default/skip, typed
Harvesting111/Chef45/farming-asset binding, fail-closed load for missing,
corrupt or mis-bound content, dev/Chef/exclusivity gates, active-run-only
dispatch with stale-instance inertness, server-authoritative type8 timer and
results, and a staged once-only reward transaction.

**One material finding (actionable, content-side):** there is **no pre-offer
guard for the configured reward definitions**. The real frozen campaign
catalogue (`inventory.json`) defines only two of the four region-111 bundles —
`2887` (stainberry) and `2877` (spiralmint) — and lacks `2885` (stickfruit) and
`2890` (tumtumnip), yet the shipped config binds all four. `Gameplay::load`
accepts that content and `realm harvest` offers and starts the run; the failure
only appears when a reached win tries to grant, the frame is refused with
`unknown reward definition`, and the run is retained in `LevelComplete` with
nothing granted. The win is not silently accepted (state is untouched), but a
player can start and complete an unpayable game. The builder's own tests
substitute `[2739, 879, 65834, 2436]` (all present) so the shipped binding is
never exercised end-to-end. Recommend validating the configured
`spec.outputs` definition ids against the loaded inventory catalogue at
`Gameplay::load` (refuse the content) or at the offer/begin gate (refuse the
launch), so the missing definitions are caught before a run starts; the worker
preparing the four exact definitions will also close the gap.

## Independently verified

Private probe crate `lab/reports/harvesting-gateway-review-20261009T132030Z/probe`
(4/4 passed, log `logs/03-probe.log`):

1. `review_real_package_missing_bundle_definitions_are_not_guarded_before_offer`
   — the frozen catalogue lacks `2885`/`2890` and defines `2887`/`2877`/`2875`/`2884`;
   `harvesting::write_content(111)` places the shipped config next to that
   package; `realm harvest` starts instance `58_000_001` (no load/offer guard);
   a crafted `LevelComplete` win then returns `unknown reward definition`, the
   frame is refused, the snapshot is unchanged, the run stays `LevelComplete`
   and nothing is granted. This is the actionable missing guard, not a silent
   accept.
2. `review_old_saves_and_missing_or_wrong_content_fail_closed` — a legacy save
   without `harvesting` round-trips byte-identically; without `harvesting.json`
   no field is serialized and a harvesting snapshot fails with "harvesting
   snapshot requires its content"; wrong `game_id 110`, wrong `profile_id 14`,
   wrong `preload_file` and corrupt JSON are all refused at load.
3. `review_dev_job_exclusivity_and_stale_instance_dispatch` — dev tools off
   refuses with no state change; locked Chef and a non-Chef active job refuse;
   a live mining run blocks harvesting (mining guard message, no run started,
   snapshot unchanged); while harvesting is live, stale-instance frames are
   inert and class/interact frames are blocked with no state change.
4. `review_once_only_atomic_grants_with_manifest_definitions` — with the
   manifest-present substitutions the win grants 3 clusters per bundle once,
   replaying `OnNextLevelMsg` grants nothing, and a save/reload restart cannot
   replay the win into a second grant.

Focused targets re-run in private target directories, all green
(`logs/04-focused-set.log`, `logs/05-focused-set-2.log`): `harvesting` 17,
`harvesting_job_binding` 3, `harvesting_review` 4, `mining` 21,
`mining_review` 13, `type8_gateway` 10, `type8_harvesting_gateway` 11 (79
tests). Builder provenance matches the bundle: `red-oldcompile.log` (test does
not compile against the preserved pre-edit sources), the three-run RED→GREEN
history (`first-run.log` 8/3, `second-run.log` 9/2, `third-run.log` 11/0) and
`green-focused.log` (11+10+21+17+3+4). One environment note: in the
mining-score target dir the `mining` test binary was blocked once by the OS
Application Control policy (`os error 4551`); it ran green in the other
private target dir, and no source or shared artifact was affected.

## Boundaries

- Native acceptance is the lead's; this review is source- and gateway-level
  (no UI, process, save or native launch touched).
- Harvesting stays a dev-gated candidate (`realm harvest` + `--dev-tools`);
  the world entry, `39/16` bundle payload and `26/12` launcher authorship, the
  locale-4 vs wiki-6 produce question and ingredient-conservation policy remain
  open and unchanged.
- Fields, regional goals, quota and output values are unchanged from the
  Chef-binding revision; no stew/gather substitution was introduced.
- The missing-definition finding is a content/config gap, not a gateway logic
  defect: the gateway refuses the unpayable win and preserves state.

## Repair applied (2026-10-09, authorized)

Test-first repair of the missing-reward-definition guard, in the owned files
only (`src/gameplay.rs`, `src/inventory.rs`,
`tests/type8_harvesting_gateway.rs` plus the new
`tests/harvesting_reward_catalogue.rs`). `src/mining.rs` (concurrent reshuffle
work) and the harvesting module/package were not touched.

RED (before the patch, `logs/01-red.log`): 3 failed / 1 passed — `Gameplay::load`
accepted the shipped region-111 content although the catalogue lacks `2885`
and `2890`, accepted a wrong/late output definition, and a refused-load
expectation could not hold.

GREEN (`logs/02-green.log`, `logs/03-green-focused.log`): the new catalogue
test is 4/4 and the focused set is 95 tests green — `harvesting` 17,
`harvesting_job_binding` 3, `harvesting_review` 4,
`harvesting_reward_catalogue` 4, `inventory` 12 (+2 ignored), `mining` 21,
`mining_review` 13, `type8_gateway` 10, `type8_harvesting_gateway` 11.

The bounded change:

- `Inventory::has_definition(definition_id)` — read-only catalogue lookup.
- `Gameplay::load` loads the inventory once
  (`let inventory = Inventory::load(directory, inventory)?;`) and reuses that
  same instance for the struct field; no duplicate load.
- After the harvesting game/profile/asset binding checks, every
  `engine.spec().outputs` definition id must exist in that catalogue, else the
  load fails with
  `harvesting reward definition <id> is not in the loaded inventory`.
  Scope is Harvesting111 only; mining keeps its shipped `outputs: []`.
- `tests/type8_harvesting_gateway.rs` now asserts that the real (missing
  definition) content refuses the gateway load and then restores its
  manifest-present fixture content for the remaining assertions; the other 10
  gateway tests (including full-bag atomic rollback/retry and once-only grants)
  are unchanged and green.

File hashes after the repair (before → after):

| File | Before | After |
| --- | --- | --- |
| `src/gameplay.rs` | `9e8b56ed…` | `996d33865966fcf8b7d0cc5f0e761f2afdeff9230cb64758e0f303a3ac21d8fa` (+12/−1) |
| `src/inventory.rs` | `5deace93…` | `dccbf8006bd4d81950410825406b927d7dd1c3f55b89445a4d680ff7395d51a7` (+6/−0) |
| `tests/type8_harvesting_gateway.rs` | `39f5f6a8…` | `ee8a3c5e5177e0f40a7791e57d9b7ae35134de6a27006329b9f1b707d94044ff` (+10/−0) |
| `tests/harvesting_reward_catalogue.rs` | new | `1afb2ba186985dddd5daf33dad5e300ee60a80525be4313008e397238441c7ba` |

The pre-edit bytes are preserved under
`lab/reports/harvesting-reward-guard-fix-20261009T133308Z/preserved-before-edit/`.
Concurrent `src/mining.rs` reshuffle work (`ec611ec0…`) is outside this
repair; `src/harvesting.rs` (`272711da…`) and `src/lib.rs` (`bb1d8755…`) are
unchanged.
