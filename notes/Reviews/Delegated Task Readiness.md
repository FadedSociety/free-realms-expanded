# Delegated Task Readiness v2 (corrected)

Read-only reconciliation of all **100 work items**: 6 initial direct-worker systems (001-006), 93 delegated pieces (007-099, of which 028-048 were root-dispatched), and the coordinator item 100. Machine rows: `notes/Reviews/Delegated Task Readiness.json` (version 2). v1 is preserved at `Delegated Task Readiness.v1.json` / `.v1.md`; builder: `../lab/reports/delegated-task-audit-20261009/build_readiness_audit_v2.py`.

## Corrections in this revision

1. **Native acceptance is split from protocol/unit/codec coverage.** 51 of 93 pieces overlap existing work, but only **32** overlap *natively accepted gameplay* (character, cave, crafting-family hardening, opening, combat, economy), **5** are native-runtime verified but non-gameplay (inventory catalogue, transport), **9** are protocol/codec/unit coverage only (Type8 launch machinery, housing codec), and **5** are partial overlaps, extensions or tooling.
2. **Housing codec and Type8 launch are NOT native gameplay successors.** They are reviewed codec/model and protocol work; `notes/Reviews/Delegated Housing Readiness.md` and `Delegated Racing Readiness.md` confirm missing routing/state/authority/content.
3. **Harvesting is no longer "blocked on missing tiles".** v2 is registered and routed with valid content and runtime guards (`lab/references/opening-harvesting-native-20261009T142624Z`: 11x11 padded, 90 playable cells, drop row 10, eight registered tile kinds, validation green). The whole native round remains open. Mining supersedes its older blockers via `lab/references/opening-mining-starter-20261009T180011Z` (private candidate, 40-percent/6+6-goal starter), native round still open.
4. 081 remains the one piece with a live call site (`inventory.rs:544`, alias `instance_ids::allocate_from_snapshot`).
5. Registration alone never resolves a row: the JSON gates name routing/state/authority/content work per piece, and candidate rows state it explicitly.

## Headline (validated from JSON)

| Metric | Value |
| --- | --- |
| Work items audited | 100 (6 initial systems + 93 delegated + 1 coordinator) |
| Delegated modules registered in `lib.rs` | 0 |
| Delegated modules compiled via `#[path]` include / with a live call site | 1 / 1 (081) |
| Pieces overlapping existing production or reviewed work | 51 |
| - overlap with natively accepted gameplay | 32 |
| - overlap with native-runtime verified non-gameplay | 5 |
| - overlap with protocol/codec/unit coverage only | 9 (Type8 launch 015/018; housing codec 021-027) |
| - partial overlap, extension or supporting tooling | 5 |
| Pieces with no existing production coverage | 42 (candidates; registration alone insufficient) |
| Delegated pieces with complete artifacts | 93/93 (module + Rust test + py script + py test + note + lab bundle) |
| Superseded lab markers | 008, 015, 099 (one each) |
| Type8Engine evidence | 16 occurrences in `gameplay.rs` (Mining/Harvest dispatch), protocol path - not native gameplay acceptance of launched minigames |

## Initial direct-worker systems (001-006)

| Item | Modules | Registration | Current native state |
| --- | --- | --- | --- |
| 001 Mining | `mining.rs` | registered, routed, guarded | **whole native round OPEN**; superseded by `opening-mining-starter-20261009T180011Z` (private candidate, reconstructed 40%/6+6 goals) |
| 002 Harvesting | `harvesting.rs` | registered, routed, valid content + runtime guards | **whole native round OPEN**, NOT blocked on missing tiles; superseded by `opening-harvesting-native-20261009T142624Z` (v2 package, validation green) |
| 003 Fishing | `fishing.rs` | not registered, 0 callsites | no native run; routing/state/authority/content missing per `notes/Reviews/Delegated Fishing Readiness.md` |
| 004 Kart | `kart_lobby_native.rs`, `kart_progress_native.rs`, `kart_racing.rs` | lobby/progress registered; racing unregistered | codec work only; `notes/Reviews/Delegated Racing Readiness.md` confirms missing routing/state/authority/content |
| 005 Soccer | `soccer.rs`, `soccer_match.rs` | registered | protocol/unit coverage in focused tests; not a native match |
| 006 Housing | `housing_native.rs`, `housing.rs` | native registered; core `housing.rs` unregistered | codec/model coverage only; `notes/Reviews/Delegated Housing Readiness.md` confirms missing routing/state/authority/content |

## Overlap groups (delegated pieces)

| Coverage | Systems and live modules | Pieces | Meaning |
| --- | --- | --- | --- |
| native gameplay accepted | character (`identity.rs`,`login.rs`,`character_store.rs`), cave (`encounter.rs`,`opening*.rs`,`gameplay.rs`), crafting family (`cooking.rs`,`smelting.rs`,`forging.rs` + minigame lifecycle hardening), opening (`opening.rs`,`opening_journal.rs`,`quests.rs`), combat (`encounter.rs`,`combat.rs`), economy (`game_store.rs`,`inventory.rs`) | 32: 049-050, 012-014, 016-017, 051-058, 059-065, 066-072, 073, 078, 086 | subject already served natively; pieces are hardening/verification or alternate slices, not the live path |
| native-runtime verified, non-gameplay | catalogue (`inventory.rs`,`item_categories.rs`), transport (`soe.rs`,`soe_send.rs`,`session.rs`) | 5: 081, 082, 094, 096, 097 | runtime paths observed, but this does not accept the pieces' gameplay claims |
| protocol/codec/unit coverage only | Type8 launch machinery (`mining.rs`,`harvesting.rs`, crafting modules), housing codec (`housing_native.rs`) | 9: 015, 018, 021-027 | NOT native gameplay; housing/racing readiness reviews list the missing routing/state/authority/content |
| partial / extension / tooling | dev-teleport vs warpstones (077), single-vs-multi character (095), world loaders (079, 080, 098) | 5 | one half exists; the other half is new and unaccepted |

Everything else (42 pieces: derby, chess, checkers, TCG, pets, shops, trading, friends, chat channels, party, guilds, mounts, emotes, collections, postman, NPC placements, job progressions, cosmetics, gear attributes, housing save/discovery/native-acceptance slices) has **no existing production coverage** and remains a candidate only.

## What stands from v1

- `restoration` is not declared in `lib.rs`; zero `restoration::` references exist; 92 of 93 piece modules have no include and no call site.
- A child's "ready for lead review" means artifact-complete, not playable; every JSON row carries its own `missing_gates` excerpt, `coverage_level` and `native_state`.
- `notes/Current Progress.md` remains the native-acceptance authority; no piece row promotes anything into it.

## Lead actions

1. For the 32 native-gameplay overlaps, decide per piece whether it replaces or hardens the accepted path; only 081 currently changes live behaviour (`inventory.rs:544`).
2. For codec/protocol overlaps (015/018, 021-027), treat the specialized readiness reviews (Housing/Fishing/Racing) as the gate: registration alone does not resolve routing, state/authority or content.
3. Mining and harvesting keep their own open native rounds with the superseding receipts above; Type8-adjacent pieces must not be described as native-accepted gameplay.
