# Harvesting Chef Binding

Independent read-only review (2026-10-09, reviewer workspace
`/root/mining_review`). No production, test, shared note, save, UI, process or
release file was modified; owned artifacts only: this note and
`lab/reports/harvesting-chef-binding-review-20261009T125909Z/`.

## Reviewed revision (hashes verified in-session)

| Path | SHA-256 |
| --- | --- |
| `prototype/osfr-tools/src/harvesting.rs` | `272711da0e6b72fceac7227490e1190b46f2c1028cba856cdf78bcaa3576d477` |
| `prototype/osfr-tools/tests/harvesting_job_binding.rs` | `20ae461056c650909cc09ee23f85c61e42f1a294da83dc5d2c571a24aff9593f` |
| `prototype/osfr-tools/tests/harvesting.rs` | `b01020e483377cc85c4f85a4296a95b27450e6dd18a7e61935caa8004d8bc96a` |
| `prototype/osfr-tools/tests/harvesting_review.rs` | `80ca4bc4e91ce6cc06ded9bf827cfc1920084401188f968b839a821b520ffa29` |
| frozen, untouched: `src/mining.rs` / `src/gameplay.rs` / `src/lib.rs` / `src/encounter.rs` | `e6e421d8…` / `077ef906…` / `376f936a…` / `68cfc114…` |
| release `target/release/osfr-gateway.exe` | `7d5be63fd3cb1814e15211c07423ddf472d63f0f236410d7790c6b5b5d475f39` |

Evidence read: `notes/Research/Harvesting Native Launch Preflight.md`, the
preflight bundle `lab/reports/harvesting-native-preflight-20261009T133000Z/`
(incl. `preflight-evidence.json`, `hash-check.log` 31 checks / 0 failures), and
the builder bundle `lab/reports/harvesting-job-binding-fix-20261009T140000Z/`
(`production.patch`, `dependent-tests.patch`, RED/GREEN logs,
`final-evidence.json`).

## Verdict

Correct, minimal binding repair with no material finding. Chef profile 45 is
the recovered rotation job (wiki Chef rev 46136, `870b47c6…`: Chef XP comes
from the harvesting and cooking minigames), and harvesting was the only
candidate whose `profile_id` was a placeholder (`game_id`). The fix adds
`CHEF_PROFILE_ID: i32 = 45`, binds it in `content()`, and refuses every other
profile in `validate_content` — including the old `profile_id == game_id`
shape and a Farmer-like `15` (Farmer is a separate housing-farm progression).
All six supported regions bind Chef, gap profiles stay unlaunchable, and the
shared frozen engine refuses `begin` for any other profile and stamps 45 into
the `39/16` info packet. Schema, board geometry, quotas, produce sets, output
bundles, instance namespace, launch assets and the quest gather path are
unchanged.

## Independently verified

Private probe crate `lab/reports/harvesting-chef-binding-review-20261009T125909Z/probe`
(frozen `harvesting.rs` compiled from a hash-verified fixture), 4/4 passed,
log `logs/01-probe.log`:

1. `review_six_regions_bind_chef45_and_shape_documented` — exactly six
   supported regions `[109, 111, 113, 115, 117, 119]`; each `content(game_id)`
   reports `profile_id == 45 != game_id`, validates, keeps
   `schema_version 1`, `FarmingAssets.txt`, `game_farming.gfx`, the 10x10
   reconstructed board (shape length = width*height), four goals/values
   601..604 with quota 15 / per_reward 5 / bundle quantity 1, and outputs in
   the recovered produce order. Region 111 matches the pre-change preflight
   receipt exactly: name `100834`, description `94150`, icon `908`
   (`HARVESTING_ICON_ID`), outputs `[2885, 2887, 2890, 2877]`
   (stickfruit/stainberry/tumtumnip/spiralmint), namespace `58_000_001`,
   content file `harvesting.json`.
2. `review_wrong_or_farmer_profile_refused` — `validate_content` refuses
   profile `111` with "must bind the recovered Chef profile 45, not 111" and
   refuses a Farmer-like `15`; Chef 45 stays valid.
3. `review_gap_regions_and_malformed_content_unchanged` — gap ids
   `58/110/112/114/116/118/120/217` still refuse launch; tampered launch assets
   are refused; a labelled quota tuning stays valid (the validator requires
   only nonzero quotas; the value is reconstructed); a bundle definition
   outside the pinned produce table is refused; a `schema_version 2` drift is
   refused by the shared engine at load (documented split: the harvesting
   validator does not police schema, the engine's game/schema binding does);
   `write_content(dir, 110)` is refused while `write_content(dir, 111)` writes
   `harvesting.json` with `game_id 111`, `profile_id 45`, unchanged schema.
4. `review_engine_begin_gate_and_info_packet` — the emitted `harvesting.json`
   loads; `begin(45)` succeeds and the `39/16` packet carries name `100834`,
   description `94150`, profile `45` and `FarmingAssets.txt`; `begin(111)` and
   `begin(14)` are refused with no active run.

Focused targets re-run in a private target directory, all green
(`logs/02-focused-set.log`): `harvesting` 17, `harvesting_job_binding` 3,
`harvesting_review` 4. Builder provenance reproduced from the bundle:
`repo-red-harvesting_job_binding.log` 0 passed / 2 failed (old source bound
profile 109 and validated a wrong profile), `repo-green-focused.log` 17+3+4
passed, `scratch-red.log`/`scratch-green.log` match, scoped `rustfmt --check`
clean.

Scope of the change window (12:52–12:56Z, `logs/03-scope-check.log`): the only
production source touched is `src/harvesting.rs`; the three test files are the
new one plus the two authorized one-line updates, which match
`dependent-tests.patch` exactly (`harvesting.rs:499` →
`CHEF_PROFILE_ID`; `harvesting_review.rs` → `begin(CHEF_PROFILE_ID)`). Frozen
hashes are unchanged (`gameplay.rs 077ef906…`, `mining.rs e6e421d8…`,
`lib.rs 376f936a…`, `encounter.rs 68cfc114…`, release `7d5be63f…`), and the
quest gather grant at `gameplay.rs:1219` (rainleaf 2884 / gummyfruit 2875) is
intact.

## Boundaries

- The pinned pre-fix `harvesting.rs` bytes (`4e5196ba…`, recorded in the
  preflight receipts) were not preserved in the workspace, so this review
  verifies scope by frozen hashes, the authorized patch, the pre-change
  receipts and behavioral probes rather than a byte diff of the old file.
- Harvesting remains an unregistered candidate (not in `lib.rs`, no gateway
  entry); the launch is not wired natively by this change.
- Still open and unchanged by design: world entry for Harvesting 111, the
  locally authored `39/16` reward bundles and `26/12` launcher, the locale-4
  versus wiki-6 produce question, and ingredient-conservation policy (the
  gathering path still grants 2884/2875 once per newly counted interaction).
- Native testing is the lead's; this review made no UI, process or native
  claim. `validate_content` intentionally does not police the quota value or
  `schema_version`; both are labelled reconstructions enforced elsewhere
  (nonzero quota in the validator, schema/game binding in the shared engine).
