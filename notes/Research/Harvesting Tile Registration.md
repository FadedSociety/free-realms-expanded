# Harvesting Tile Registration

Status: tile-registration repair implemented and locally verified, 2026-10-09 13:51 UTC (updated
14:0x UTC after the authorized narrow `tests/harvesting.rs` expectation fix; every focus-family target
is green). Evidence label: **RECONSTRUCTED content registration over ORIGINAL client tile-generator
semantics**. No native client run was performed and no native acceptance is claimed; the frozen
content package `280f5cd7fcca711aaf7e8118ef04195b3bb1241413498325288aca10dff94d20` is untouched.

Owned paths for this repair:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/harvesting.rs` | regular tile-kind registration + `validate_content` guard |
| `prototype/osfr-tools/tests/harvesting_tile_registration.rs` | 9 focused vectors (RED first) |
| `prototype/osfr-tools/tests/harvesting.rs` | authorized narrow update of the stale 8-kind expectation (lines 512-518 only) |
| `lab/reports/harvesting-tile-registration-20261009-1351Z/` | RED/GREEN logs, family/lib/mining logs, fresh content export |
| this note | provenance, boundary, remaining owner action |

## Defect

The emitted harvesting content advertised `spec.selectable = [401, 402, 403, 404]` while
`spec.tiles` carried only the four collectables `601..604`. The shared engine emits one
`OnTileInfoMsg\t<value>\t<value - category>` per tile-list entry
(`src/mining.rs::initialize_board`), so only `601..604` were ever registered on the client.

The original client makes that fatal for regular tiles:

- `core/TileGenerator.as::Initialize` fills `aNames` for empty (0), wildcard (200), blocker (800),
  hazard-lock (301), the powerups (900..953), the lootdrops (1000..1002) and the dropzone (100); it
  names neither the selectable values nor the collectables.
- `core/TileGenerator.as::SetTileInfo(value, index)` is the only writer of those entries. It derives
  the branch as `Math.floor(value / 100) * 100`, stores `"SelectableTile" + index` for selectables and
  `aCollectableNames[index * 2]` for collectables, and the server's second field is therefore the
  one-based kind index.
- `core/TileGenerator.as::generate_selectable` dereferences `aNames[_nValue]` directly, so a value the
  server spawns but never registers cannot be drawn.

The RED run captured exactly that: `OnTileInfoMsg` reached the wire as
`[(601, 1), (602, 2), (603, 3), (604, 4)]` only, the written tile values were
`[601, 602, 603, 604]`, and `validate_content` accepted documents with missing, duplicated or
wrong-category regular mappings (`red-tests.log`, 1 passed / 7 failed).

## Repair

- `src/harvesting.rs` gains the recovered category constants (`TILE_SELECTABLE_CATEGORY = 400`,
  `TILE_COLLECTABLE_CATEGORY = 600`) and `REGULAR_TILE_VALUES = [401, 402, 403, 404]`.
- `board_spec` now registers the four regular kinds **first** (`category = 400`, `icon_id` 1..4,
  mirroring the mining spec's selectable entries) and then the four collectables in the recovered
  produce order; `selectable` is emitted from the same constant.
- A new `validate_tile_kinds` guard runs inside `validate_content`, which the writer already
  traverses (`write_content` -> `content_json` -> `content` -> `validate_content`). It refuses:
  a kind whose `category` differs from the original `floor(value / 100) * 100` branch, a duplicate
  value, a selectable or collectable value with no registered kind, and a kind that belongs to
  neither recovered set.
- Preserved unchanged: collectable values/order, quotas, `per_reward`, goal icons, bundle
  definitions, seed, duration, spawn percentages, profile 45, `FarmingAssets.txt`,
  `game_farming.gfx`, board shape/geometry and the reconstruction label.

The fixed writer's output for all six regional ids is exported for the root's versioned package at
`lab/reports/harvesting-tile-registration-20261009-1351Z/fresh-content/harvesting-<game>.json`,
with the wire pairs and per-file sha256 in `tile-registration.json`. The existing `280f5…`
package was not modified.

## Verification

| Command | Result | Log |
| --- | --- | --- |
| `cargo.ps1 test --test harvesting_tile_registration` (baseline) | 1 passed, 7 failed — wire set `601..604` only, writer values `601..604`, `validate_content` accepted corrupted kinds | `red-tests.log` |
| `cargo.ps1 test --test harvesting_tile_registration` | 9 passed, 0 failed, 0 warnings | `green-tests.log` |
| `cargo.ps1 test --test harvesting` (after the authorized narrow update) | 17 passed, 0 failed | `harvesting-17.log` |
| `cargo.ps1 test --test harvesting --test harvesting_tile_registration --test harvesting_review --test harvesting_job_binding --test harvesting_reward_catalogue --test type8_harvesting_gateway` | 17 + 9 + 4 + 3 + 4 + 11 passed | `harvesting-family.log` |
| `cargo.ps1 test --test harvesting_review --test harvesting_job_binding --test harvesting_reward_catalogue --test type8_harvesting_gateway --test harvesting_tile_registration` | 4 + 3 + 4 + 11 + 9 passed (earlier split receipt) | `harvesting-family-rest.log` |
| `cargo.ps1 test --lib` | 24 passed | `lib-tests.log` |
| `cargo.ps1 test --test mining_shape_parity --test mining_tile_rendering` | 4 + 3 passed | `mining-smoke.log` |
| `rustfmt --edition 2024 --check src/harvesting.rs tests/harvesting_tile_registration.rs` | clean for both owned files | `fmt-check.log` |

The wire vectors drive the real engine handshake (`begin` -> `OnConnectMsg` -> `OnStartGameMsg` ->
`OnStartLevelMsg`) and assert the emitted `OnTileInfoMsg` pairs are
`(401,1)(402,2)(403,3)(404,4)(601,1)(602,2)(603,3)(604,4)` for every one of the six regional
specs. The refusal vectors mutate a cloned `HarvestContent` only; nothing is written outside a
private temp directory.

## Owner action completed (authorized narrow update)

`prototype/osfr-tools/tests/harvesting.rs::emitted_spec_is_consumable_by_a_type8_engine_loader`
pinned the pre-fix contract at lines 512-518 (`tiles.len() == 4`, all `category == 600`, tiles equal
to the collectable list). Under explicit root authorization that block alone was replaced with the
exact eight-kind registration contract:

```rust
assert_eq!(c.spec.tiles.len(), 8);
assert_eq!(
    c.spec.tiles.iter().map(|t| (t.value, t.category, t.icon_id)).collect::<Vec<_>>(),
    vec![(401,400,1),(402,400,2),(403,400,3),(404,400,4),
         (601,600,706),(602,600,710),(603,600,667),(604,600,707)]
);
assert_eq!(c.spec.tiles[4..].iter().map(|t| t.value).collect::<Vec<_>>(), c.spec.collectable);
assert_eq!(c.spec.tiles[4].icon_id, 706, "Gummyfruit bundle icon");
```

The collectable subset order and the Gummyfruit icon pin are preserved, so the test keeps its
meaningful coverage. The file hash moved from `b01020e4…` to `9ac8a2f1…`; `--test harvesting` is
17 passed / 0 failed and the whole focus family is green (see the verification table). No other
source or test file was touched.

No native claim: the engine-level payload assertion proves what the server sends, not what the
client renders. The next real gate stays a client run against the root's newly versioned content.

Cross-links: [[Harvesting Restoration]], [[Harvesting Native Mouse Driver]], [[../MMO Work Queue]],
[[../Design/MMO Worker Contract]].
