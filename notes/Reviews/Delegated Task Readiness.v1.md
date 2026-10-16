# Delegated Task Readiness (100-row reconciliation)

Read-only audit, generated 2026-10-09 from `notes/WorkItems/*.md`, `notes/MMO Work Queue.md`, the dispatch ledger and root snapshot, every actual piece module/test/note, `prototype/osfr-tools/src/lib.rs`, all `#[path]` includes, non-piece call sites, and `lab/reports/piece-*-*` evidence bundles. Machine rows: `notes/Reviews/Delegated Task Readiness.json` (100 rows; builder script kept at `../lab/reports/delegated-task-audit-20261009/build_readiness_audit.py`).

## Bottom line

- **0 of 93 delegated piece modules (007-099) are registered in `lib.rs`, and `restoration` itself is not declared there.** Zero `restoration::` references exist anywhere in `src/*.rs`.
- **Exactly one piece is compiled into the crate today: 081**, via `src/inventory.rs:17` (`#[path = "restoration/piece_081.rs"]`, `pub mod instance_ids`). It has 0 cross-module call sites, so no behaviour changed yet.
- **0 of 100 tasks are native-testable as delivered.** Every delegated artifact is a candidate: code + tests + checker + note + lab bundle, unreachable from the running binary. "Ready for review" in a child report means "artifact exists", not "playable".
- All **93/93** delegated pieces have complete artifacts (module, owned Rust test, python script, python test, evidence note) and at least one private lab bundle.
- Work items 001-006 are the pre-existing production systems (root-owned); 028-048 were root-dispatched with their own receipts. Specialized Housing/Racing/Fishing/Harvesting audits remain with their assigned reviewers and are not re-derived here.

## Integration-ready shortlist (artifacts complete, unaccepted)

| Piece | Why it is near-ready | Concrete evidence | Missing gate |
| --- | --- | --- | --- |
| 081 | Only piece already compiled into the crate | `inventory.rs` include; 12+2 Rust / 8 python green; found and closed a real id-reuse defect | wire `InstanceCounter` into the grant path; retire 2 characterisation tests |
| 094 | Validates a load-bearing production policy (stale-frame rejection) | 5+10+7 Rust / 10 python green; crc_length=0 and 3-byte CRC both admit stale frames | register module; native client restart against running gateway |
| 096 | Typed world init re-encodes 12 captured slots byte-identically | 9+1 Rust / 7 python green; 34 slots deliberately still captured | register; call `verify_captured` in `Gateway::load_reference` |
| 069 + 086 | Economy correctness pair with a named real defect | 069: 9 Rust / 11 python, 26-check checker; 086: 10 Rust / 16 python, flags `apply_quest` crediting coins before items | wire `consider/commit/recover`; fix quest reward ordering |
| 095 | Versioned multi-character store with byte-identical rollback | 10 Rust / 5 python green; restart test re-executes the binary | register; `identity`/`login` still select one character |
| 079/080/098 | World-loading trio (chunk survey, BVH collision, GCNK/DMOD/DSKE/ADR/GR2 loaders) | 2,344 chunks surveyed; 281/281 collision columns; 10+1 Rust / 8 python for 098 | register each; 080<->098 collision semantics still open |
| 051-058 | Combat slice with recovered wire + capture-backed tests | 051: 443 enemy rows; 052: 256 native movement msgs; 054/055/056: reach/status ledgers | register; wire into `encounter.rs` (lead-owned) |
| 015 + 018 | Native test-entry machinery; already has review artifacts | `Activity Coverage Receipt Review`, `Type8 Gateway Integration Handoff`; 015 corrected today and its audit re-passes 24 tests | register `piece_018`; convert `OpeningActor` -> `ActorView`; lead native run |
| 022/025/027 + 060-065 | Housing/dungeon chain; heaviest native potential | 022: 12 Rust with capture re-encode; 060-063 instance/reward/gate/grading; 065 replay grader (nothing passes today - needs one fresh run) | register every module; two-client session for 099-based checks |

Existing independent-review artifacts already reference 015/018, 058, 060, 069, 081, 083-086. None of those reviews changes the registration/callsite facts above; they are candidate evidence, not acceptance.

## Cross-cutting issues the lead should hold

- 086 -> 069: `quests.rs::enemy_defeated` sets `completed` and `rewarded` in one step, which 069's ledger would read as an already-granted replay. Fix belongs at the integration point.
- 081 vs production: `max(live id)+1` allocation reuses retired instance ids; 081's counter is the replacement, currently uncalled.
- 084: leaked-server `BuyBackRequest=15` contradicts the shipped client constructor (=12); client pinned, buyback handler still missing.
- 097: stock catalogue has 113 dangling `NameId`s, 29 missing categories, 340 `Param1` conflicts - useful as an integration gate list.
- 082: inventory-search crash root cause is NOT established; the "unresolved data" hypothesis was disproven. Do not ship a fix as "the" cause.
- 051: local opening reuses wolf definition 3156 for all five wolves vs community 3156-3160.
- Repo-wide `cargo test` failures reported by children are transient sibling-target breaks, not failures of the audited pieces.

## What must change before any native claim

1. Add `pub mod restoration;` (or per-module declarations) to `prototype/osfr-tools/src/lib.rs` - currently absent; zero piece modules are visible to the crate outside 081.
2. Replace each test-side `#[path]` include with `use crate::restoration::piece_NNN` at the lead's chosen call sites; a declaration alone still yields "registered but never called".
3. Wire at least one real caller per system (gateway/gameplay/inventory paths) and re-run the piece's RED/GREEN evidence against the release build.
4. Only then run lead-native acceptance. `notes/Current Progress.md` remains the authority on what is natively accepted (opening campaign, cave/boss, mystery chest, mining/harvesting/soccer partials); no delegated piece changes that list.

Treat every row's `missing_gates` field in the JSON as the named next evidence for that piece; rows are intentionally skeptical and carry no playable claims.
