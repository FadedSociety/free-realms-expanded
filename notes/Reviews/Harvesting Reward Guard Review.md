# Harvesting Reward Guard Review

Independent read-only review, 2026-10-09 (all times UTC). Candidates reviewed:

| File | SHA-256 |
| --- | --- |
| `prototype/osfr-tools/src/gameplay.rs` | `996d33865966fcf8b7d0cc5f0e761f2afdeff9230cb64758e0f303a3ac21d8fa` |
| `prototype/osfr-tools/src/inventory.rs` | `dccbf8006bd4d81950410825406b927d7dd1c3f55b89445a4d680ff7395d51a7` |
| `prototype/osfr-tools/tests/type8_harvesting_gateway.rs` | `ee8a3c5e5177e0f40a7791e57d9b7ae35134de6a27006329b9f1b707d94044ff` |
| `prototype/osfr-tools/tests/harvesting_reward_catalogue.rs` (new) | `1afb2ba186985dddd5daf33dad5e300ee60a80525be4313008e397238441c7ba` |

Preserved pre-edit copies (`lab/reports/harvesting-reward-guard-fix-20261009T133308Z/preserved-before-edit/`):
`gameplay.rs` `9e8b56ed5f3a2f78ae029b4cf51172ba18169c7381f1bb43e176361706c9c1a7`,
`inventory.rs` `5deace9351f98e4b2b0f594701084ddf45bbde85c48e19f23d2870923f41639f`,
`type8_harvesting_gateway.rs` `39f5f6a88fcaa3ee325346a0a15c32e527d2a11ee918c5f0e9fa8679a54934d7`.
Private proof: `lab/reports/harvesting-reward-guard-review-20261009T1343Z/` (`logs/01-focused.log`,
`logs/02-private-guard-probe.json`, `guard-probe/`, `proof-manifest.json`, `final-hashes.txt`).
No source, test, content, save, UI or process edit was made by this review; only this note and that
private report were written.

## Verdict

The guard is correct, bounded and pre-offer. All five independent private probes pass, the focused
rerun is green, and the diff stays inside the four claimed files with the claimed line bounds.
No defect found. Two qualifications are recorded under Limits.

## Verified

1. **Bounded diff.** `gameplay.rs` +12/−1, `inventory.rs` +6/−0,
   `tests/type8_harvesting_gateway.rs` +10/−0 (diff of the preserved copies against the candidates).
   The only logic changes are: one early `let inventory = Inventory::load(directory, inventory)?;`
   reused in the `Self` construction, a loop that refuses any `engine.spec().outputs` definition id
   absent from that catalogue (`"harvesting reward definition <id> is not in the loaded inventory"`),
   and `Inventory::has_definition` (`self.content.definitions.contains_key(&definition_id)`).
2. **Before any offer / no partial mutate.** The refusal comes out of `Gameplay::load` before the
   engine is stored and before any grant path can run (line 360-367, above `Some(engine)` and the
   `Self { .. }`). Probe `refused_load_conserves`: after a valid load, swapping in the shipped
   outputs makes the next load refuse while the caller's `GameplayState` JSON and the
   `harvesting.json` bytes stay identical.
3. **Old real package refuses, new real package loads.** The old 1701-definition base catalogue
   (`lab/references/opening-journal-text-v2/content/inventory.json`) has 1701 definitions / 1647 ids
   and lacks **2885** and **2890** while defining 2887 and 2877; the new 1703-definition harvesting
   package (`lab/references/opening-harvesting-native-20261009T132512Z/content/inventory.json`) has
   1703 definitions / 1649 ids with all four present. Probe `old_missing_two_refused`: old catalogue +
   shipped outputs → `error: harvesting reward definition 2885 is not in the loaded inventory`;
   probe `new_package_all_four_load`: the harvesting package → `Gameplay::load` ok,
   `state.harvesting.is_some() == true`.
4. **Late/unknown definitions are named.** Probe `late_unknown_refused`: outputs
   `[2877, 2739, 879, 99999]` refuse with `harvesting reward definition 99999 …`, so the check is not
   first-slot-only.
5. **Legacy/no-file behaviour preserved.** Probe `legacy_no_content_bytes`: the legacy
   `GameplayState` JSON round-trips to the identical value and identical serialization, a content dir
   without `harvesting.json` loads with `harvesting.is_none()`, and reloading with that state
   succeeds. The early load is the same single call with the same arguments as before (diff shows
   only the move), so a no-harvesting save gains no new failure.
6. **Late output validation, full bag, once-only unchanged.** The diff touches none of these paths;
   `tests/type8_harvesting_gateway.rs` (`full_bag_rolls_back_the_win_and_retry_grants_after_space`,
   `quota_win_grants_the_configured_bundles_once_per_run`), `tests/harvesting_review.rs`
   (`harvested_bundles_grant_once_from_a_serialized_completion_state`) and `tests/inventory.rs`
   (`invalid_requests_and_failed_grants_leave_all_state_unchanged`) all pass in the rerun below.

## Exact commands and results

Focused rerun (workdir `prototype/osfr-tools`, private toolchain wrapper; log
`logs/01-focused.log`, exit 0):

```
project/scripts/cargo.ps1 test --test harvesting_reward_catalogue --test type8_harvesting_gateway \
  --test harvesting --test harvesting_review --test harvesting_job_binding --test inventory
```

Result: harvesting 17, job_binding 3, harvesting_review 4, reward_catalogue 4, inventory 12 (+2
ignored), type8_harvesting_gateway 11 — **51 passed, 0 failed, 2 ignored**.

Private probe crate (links the candidate library by path; workdir
`lab/reports/harvesting-reward-guard-review-20261009T1343Z`, log
`logs/02-private-guard-probe.json`, exit 0):

```
cargo.exe run --offline --quiet --manifest-path guard-probe/Cargo.toml
```

Result: `old_missing_two_refused` PASS, `new_package_all_four_load` PASS,
`late_unknown_refused` PASS, `legacy_no_content_bytes` PASS,
`refused_load_conserves` PASS — `all_pass: true`.

The fix receipt's RED log (their pre-patch run: 1 passed / 3 failed) is preserved at
`lab/reports/harvesting-reward-guard-fix-20261009T133308Z/logs/01-red.log`
(`3624388741593b74148dbe578eec157100f696d3e4ac4ce849ba07a62e33e3fa`); this review ran only
against the candidate revision.

## Limits

- The early unconditional catalogue load can surface an inventory error sooner than a later
  multi-fault error would have (error ordering only); the call, arguments and result are unchanged,
  and valid or no-harvesting saves gain no new failure.
- The six targets above were rerun; the mining/type8-gateway targets in the fix receipt's 95-test
  focused set were **not** rerun here because `src/mining.rs` was being edited concurrently
  (mtime 2026-10-09T13:39:34Z). That change is separate and is neither reviewed nor attributed here.
- Probes copy content into OS temp directories and delete nothing outside them; the real content,
  saves and processes were untouched.
- Nit (warning only): the new test declares `const BUNDLES: [i32; 4]` unused, so the compiler warns.
  The other warnings in the focused log (`MIN_CHAIN_TILES`, `CHAIN_SCORE`, …) come from the
  concurrent mining edit and are not part of this fix.
