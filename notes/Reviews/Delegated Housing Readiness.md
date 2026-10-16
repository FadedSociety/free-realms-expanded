# Delegated housing readiness review

Independent read-only audit, 2026-10-09 20:14 UTC. Scope: housing pieces 025-030 plus the
housing/farming integration. Owned artifacts: `lab/reports/delegated-housing-audit-20261009/` and
this note. No source, test, content, save, launcher, UI, process or release file was modified, and
other workers' in-flight edits were left untouched.

## Checks run (all green today)

| Check | Result | Log |
| --- | --- | --- |
| Focused cargo targets `housing housing_native piece_025..piece_030` | exit 0; 121 passed / 0 failed / 3 ignored | `focused-cargo-tests.log` |
| Private codec corpus (`OSFR_HOUSING_CAPTURE_DIR`) | 1 passed; all 44 captured S2C bodies manifest-hash verified and round-tripped (subtypes 2:8, 40:9, 41:8, 44:19) | `private-codec-corpus.log` |
| Python piece checkers `test_piece_02*.py` | Ran 100 tests, OK | `python-piece-checkers.log` |
| 127 routing probe against real `Gameplay` | 127/2, /40, /41, /44, /53: 0 replies, `changed=false`, unchanged state, no error on both `handle_in_world` and `handle` | `routing-probe.log` |
| `game_store` snapshot scope probe | saved keys exactly `version`, `character`, `gameplay`; no housing token | `snapshot-probe.json` |

The three ignored tests are the private gates (codec corpus, 029 shipped-file vocabulary, 030
export). I ran the codec one manually with the 44-vector corpus copied from the independent review
report; the other two stay unexecuted here.

## Current integration state

| Layer | Artifact | Registered | Callers / routing | Status |
| --- | --- | --- | --- | --- |
| 006 placement state machine | `src/housing.rs` `3A971E9A...` | **No** (`lib.rs` has no `pub mod housing;`) | none | 26 tests pass; own `save_state`/`load_state` take a caller path and are never called |
| 021-024 property/visit/permissions/catalogue | `src/restoration/piece_02x.rs` | **No** (no `mod restoration`) | none | test-only |
| 025 client wire | `piece_025.rs` | **No** | none | parsers for C2S 1/3/5/6/19/20 only; S2C 28/39/51 and the opcode-5 nested block undecoded |
| 026 inventory transactions | `piece_026.rs` | **No** | none | separate `FixtureItemLedger`, not the real inventory; pool-then-lot two-file crash window |
| 027 fixtures/limits | `piece_027.rs` | **No** | none | grid 25/5 cm and 15-degree step recovered; collision, vertical placement, landmarks unrecovered |
| 028 save migration | `piece_028.rs` | **No** | none | local schema only; community server persists no housing row and no original save document exists |
| 029 decoration discovery | `piece_029.rs` | **No** | none | action vocabulary and gates; server definition table, requirement semantics, Elevate call site unrecovered |
| 030 native acceptance | `piece_030.rs` | **No** | none | transcript schema and six gates; prints `native evidence complete: false`; no capture exists |
| S2C codec | `src/housing_native.rs` `9091118F...` | yes (`pub mod housing_native;`) | zero production call sites | root's own build log warns `encode_inner`/`decode_inner` are never used |
| Gateway routing | `src/gateway.rs` | - | inner 13/10/125/126/52 only | 127 falls through to `Gameplay`; no branch exists |
| Persistence | `src/game_store.rs` / `GameplayState` | - | - | snapshot = character + gameplay; `GameplayState` has 12 fields and none is housing/lot/fixture |
| Content | launcher packages | - | - | no housing/house/fixture content file in any runtime content directory |

The routing probe is the decisive evidence: the client's housing family is accepted by the parser
and silently swallowed by gameplay, so no housing reply can ever reach the client in the current
build. The snapshot probe shows why restart cannot preserve a lot either: housing state has no home
in the atomic save.

## Blockers versus a true native-ready flow

1. **No 127 dispatch.** `Gameplay` has no housing branch; all five implemented S2C subtypes are
   dropped (probe). Nothing else matters until this is wired.
2. **No registration.** `housing.rs` and pieces 021-030 are test-only `#[path]` includes; only
   `housing_native` is compiled, and it is dead code in production.
3. **No persistence.** Housing has its own file API that nobody calls, and it is not part of
   `game_store::Snapshot`; the 026 pool+lot pair is not crash-atomic.
4. **No real inventory linkage.** Furniture conservation is a parallel ledger; `grant_instance` /
   `consume_instance` seams into `Inventory` are not driven, so placement cannot change a backpack.
5. **No content.** Property rows exist only as the community `Houses.json` (31 rows) outside Git;
   the server-side fixture-definition/action table is unrecovered, so definitions would be
   reconstructed content.
6. **Wire gaps.** S2C 28/39/51 bodies unrecovered; `127/2` first dword is the client placement
   selection id (not item identity); the `127/40` 94-byte tail is partially decoded; `127/53` has
   only a synthetic vector.
7. **No native capture.** Item 030's own evaluator still reports incomplete evidence; no housing
   packet has been exchanged with the client under this stack.
8. **Housing/farming.** The Farmer housing-farm progression is separate from Chef harvesting
   (Harvesting Job Binding), and farm instances 58/63/60 are not joined to housing properties
   (piece 039 gap). Piece 029 only provides gates, not the definition table.

## Exact build/content/save/launch steps once integrated

1. Build: `scripts/cargo.ps1 build --release --manifest-path prototype/osfr-tools/Cargo.toml`
   (current release `osfr-gateway.exe`, 4,781,567 B, built 2026-10-09 12:42 local).
2. Content: add the housing content file (properties + reconstructed fixture definitions) to the
   package `content/` directory, e.g. `lab/references/<package>/content/housing.json`.
3. Save: keep housing inside the game_store snapshot
   `lab/state/<branch>/character.json` (version bump) or a journaled companion restored at
   `Gateway::load_gameplay`; the current standalone path is never called.
4. Launch (contract recorded in `lab/runs/client-baseline-20261009T151307Z/manifest.json`):
   `lab/tools/universal-modder-env/Scripts/python.exe scripts/run-local-baseline.py --seconds <N>
   --rust-reference lab/references/rust-gateway-reference --content <package>/content
   --state-file lab/state/<branch>/character.json --native-login
   --identity-file lab/state/<branch>/identity.json [--dev-tools]`.
5. Capture the six acceptance verbs for item 030 (enter, place, move, remove, guest visit, restart
   and re-enter) as `EvidenceOrigin::NativeCapture` rows.

## Findings to send builders

- **Integration lead (critical):** register `housing` + the restoration modules; add the 127
  dispatch and reply writer; fold housing persistence into the atomic snapshot. Today the client
  cannot see or keep anything housing-related.
- **025:** wire the recovered C2S parsers; decode the opcode-5 nested block and opcode-1 words;
  recover S2C 28/39/51 (and a real 127/53 vector); keep the 127/2 first dword uninterpreted.
- **026 / 081:** replace the standalone ledger with real inventory instances before placement
  mutates a backpack; remove the two-file crash window.
- **027:** collision/floor and vertical semantics remain a proxy; do not claim placement legality
  beyond the recovered arithmetic.
- **028:** needs registration plus a startup restore call and a decision on the canonical document;
  no original save exists to migrate from.
- **029:** the fixture-definition table, requirement semantics and Elevate remain unrecovered;
  effects and seating are not simulated.
- **030:** the evaluator is ready but blocked on the wire gate and a native capture.
- **Farming:** the housing-farm progression and farm-instance-to-property joins are open; keep Chef
  harvesting and housing farms separate.

## Ready evidence and limits

Ready: 121 focused Rust checks, 100 python checker tests, the 44-body codec corpus, and independent
reviews that accepted the codec delta, the 127/40 fixture-record prefix, the placement-selection
identity and the 94-byte tail decomposition within their stated bounds. Nothing here was tested in
the native client, and no production path currently consumes any housing module.

Artifacts: `lab/reports/delegated-housing-audit-20261009/` - `audit-report.json`,
`routing_probe.rs`, `routing-probe.log`, `snapshot-probe.json`, `focused-cargo-tests.log`,
`private-codec-corpus.log`, `python-piece-checkers.log`, `codec-corpus/`, `final-hashes.txt`.
