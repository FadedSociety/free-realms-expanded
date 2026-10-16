# Harvest Nodes — W57

## Scope and provenance

This imports the Legacy overworld collection-node tables into typed content. It does not implement node interaction or claim that these placements and policies were recovered from the client. The Harvesting minigame described in `Build Guides/Gameplay/Minigame Framework.md` is minigame 111/type 8; that page does not validate the world collection-node spawn tables. `src/harvesting.rs` was read-only reference for this task.

Only data rows were taken from FreeRealms-Legacy revision `9e082025391177d5a33ee18c714ddd06b457a695` (AGPL-3.0); no Legacy source code was copied.

| Source | SHA-256 |
| --- | --- |
| `CollectionNodeTypes.json` | `0a58e58696414ff5649567d4bc1121b0825dc5bbb5cee648b5c84089469e3dee` |
| `CollectionNodePools.json` | `4162ed2330967ffeacca1ec6d1e41ca26b4105564d8f32e3460c7b34db4351f8` |
| `CollectionNodeSpawns/1/briarwood-mushrooms.json` | `66795417e698aad350af206a5a07f42d29530f7672f7d468d10e55bc86deafef` |
| `CollectionNodeSpawns/1/briarwood-mushrooms-rare.json` | `8f0c7123eab19adea7608522b7eb74b20e221fd40dc14c44f5baefae7607c7f0` |
| Client `Resources/Models.txt` | `0bad5a154c3ef855744349eb03eb3ab02e7d8dfc3d29fd0f601257fce95076be` |
| Legacy `ClientItemDefinitions.json` | `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025` |
| `lab/reports/locale-id-to-text.json` | `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844` |

## Imported content and evidence boundary

`data/content/harvest_nodes.json` contains two node types, two pools and 76 spawns, all in zone 1. Spawn IDs are unique (1–78, with 15 and 16 absent); position components and headings are finite. The normal mushroom pool has 66 spawn rows, max 12 active nodes and a 60-second Legacy respawn value. The rare pool has 10 rows, max 2 active nodes and a 300-second Legacy respawn value.

W57 coverage changed from 0 node types / 0 pools / 0 spawns to 2 node types / 2 pools / 76 spawns.

The client `Models.txt` verifies model 249 as `sg_collection_node_mushrooms_01.adr` and model 674 as `sg_collection_node_mushrooms_rare_01.adr`. The eight distinct loot IDs, 11081–11088, exist in the pinned `ClientItemDefinitions.json`; the corresponding mushroom locale IDs 17056–17063 resolve in the locale report. These checks establish that the referenced models/items are present in those resource tables. They do not prove the Legacy placement, drop behavior or resource association is native server behavior.

Every imported field has a `source_tag` or `source_tags` entry. The table values for type scale, effects, interaction range, cursor, vertical offset, pool capacity, respawn time, spawn positions/headings and drop weights are Legacy reference values. They remain reconstructed policy/data (R) pending client/native confirmation. Model IDs, item-definition IDs and locale strings are cross-checked as described above. No spawn placement or pool behavior was verified against a native packet or live client.

The loader is `HarvestNodesContent::load_from(path)` in `src/harvest_nodes.rs`. `tests/harvest_nodes.rs` includes the module by `#[path]`; `lib.rs` is unchanged. Lead integration needs `pub mod harvest_nodes;` in `src/lib.rs` before crate code can import it normally.

## Validation

| Command/check | Result |
| --- | --- |
| `python -I -` with an inline standard-library preflight over imported JSON and source tables | Passed: 2 model IDs and 8 loot IDs exist; 76 positions/headings are finite. |
| Private rustfmt from `$env:CARGO_HOME\bin\rustfmt.exe --edition 2024` on `src/progression_extras.rs`, `src/harvest_nodes.rs`, `tests/progression_extras.rs`, `tests/progression_extras_content.rs` and `tests/harvest_nodes.rs` | Passed; no crate-wide formatting run. |
| `$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W57'; ..\..\scripts\cargo.ps1 test --quiet --test progression_extras --test progression_extras_content --test harvest_nodes` from `prototype/osfr-tools` | Passed: 6 progression tests, 1 content cross-reference test and 1 harvest-node test; 0 failures. |

The tests check unique type/pool/spawn IDs, pool and type references, zone agreement, model/item existence, positive drop weights, and finite spawn positions/headings. They do not test runtime node spawning, collection interaction, loot roll semantics or native acceptance.

## Open gaps

- No reviewed native-map function or captured packet in this task established the collection-node placement/pool schema or verified the Legacy spawn coordinates.
- The four effect/cursor IDs and Legacy-only policy values are not independently checked against the client. Item/model table membership is not equivalent to runtime behavior verification.
- The Rust loader is not registered in `lib.rs` or connected to a world-spawn/interaction system.
- The node tables and spawn positions are retained as reference candidates, not historical completeness claims.
