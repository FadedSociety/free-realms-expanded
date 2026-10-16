# Recipe content (W51b)

## Result

W51b expands `data/content/recipes.json` to **31 content rows**: 25 Chef dishes, five Miner bars, and the Spunky Scrapper Hammer local Blacksmith sample. The remaining candidate catalogue has **83 rows**: 19 Chef and 64 Blacksmith. Together the files preserve the 114-candidate inventory. The Blacksmith sample is restoration configuration, not a recovered original recipe row.

Recipe and unresolved records now carry `field_sources` for every populated data field. `source` remains as the row-level summary. The loader rejects absent provenance, missing item definitions, repeated ingredient ids, duplicate keys/recipe ids, unsupported schema versions, and invalid counts/ranks.

## Native recipe wire path

The recipe data is server-sent through a `PlayClient/BaseClient` custom message. Native evidence, checked against bytes with `nmq dis`:

- `00c8cfc0` registers `BaseClient.Recipes`; `00c8bd10` registers `BaseClient.RecipeComponents`.
- `009471a0` dispatches message `0x3f` to thunk `00a12740`; the thunk loads the recipe handler and transfers to `00c8d170`.
- `00c8d170` reads the operation byte and dispatches add (1), component update (2), remove (3), and full replacement (5). Add reads a row through `0090a720`; update reads recipe id, component index and value; remove reads a recipe id; replacement reads a count then keyed rows through `00c8beb0`.
- `00c8a5b0`/`00c8b360` expose recipe columns ID, Name, Icon ID, Description, Members Only, Component Count, Filter Type, Tint Value, Item Definition Id and Passes Requirement. `00c8a500`/`00c8b1d0` expose component Name, Icon ID, Description, Required Count, Count in Inventory, Tint Value and Item Definition Id. The getter offsets were checked in the disassembly; unknown row fields stay named by offset in Rust.

`src/recipes.rs` encodes the inner message as `u16 0x003f` little-endian, then an operation byte. Operation 1 carries a recipe row; 2 carries three `u32`s (recipe id, component index, updated value); 3 carries the recipe id; 5 carries a `u32` row count followed by repeated `u32` key plus row. A recipe row follows the bounded reader at `0090a720`: id, name, icon, tint, description, unknown `+0x1c`, member flag, filter, counted component list, item definition id, requirement flag, unknown `+0x2c`. Each component follows `008db930` and carries name, icon, tint, description, required count, inventory key, unknown `+0x20`, and item definition id. This is the inner client message, not an SOE/gateway frame.

No recipe row was found in the preserved baseline traffic. The native self-data audit at `lab/reports/selfdata-native-20261014T011718Z/character.json` found no known-recipe entries. The seven raw `3f 00` occurrences in baseline application frames were not identified as recipe packets and are not treated as fixtures. `tests/recipes_wire.rs` therefore uses synthetic byte fixtures; no captured original frame is claimed.

## Recipe index and `Param1`

The `Param1` values are now treated as recipe indices. Client evidence for the field semantics is `RewardBundleEntryRecipe::ReadBody` at `0103cc80` (reads one `u32` into `+0x44`), getter `0103ace0` (returns it), and formatter `0103bbf0` (prints `RecipeID=%d`). `notes/Research/Pieces/040 Crafting job progression.md` verifies that Type 5, Class 58/59 recipe patterns store their own unique recipe index in `Param1`. The pattern item descriptions tie the Strong Arm Stew and Spunky Scrapper Hammer items to those products, so their indices are 70 and 11 respectively.

This resolves the field meaning and those pattern indices; it does not recover a recipe row payload. The original `MimicMiniGameData`/`MimicMiniGamePhaseData` index-to-minigame bindings remain unavailable. New Chef rows retain `minigame_id: null` when that binding is not known. Copper's game id 77 and the two local samples' ids 46/139 are individually source-tagged; the local sample selections are marked reconstructed where applicable.

## Content and provenance

Source precedence is client, video, then wiki. The wiki was used where the client has no recipe ingredients. Wiki lookup used the local pinned corpus through `cq.py`; the item reference is FreeRealms-Legacy revision `9e082025391177d5a33ee18c714ddd06b457a695`, `ClientItemDefinitions.json` SHA-256 `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`. No Legacy source code was copied.

| Job | Populated rows | Evidence and remaining candidates |
|---|---:|---|
| Chef | 25 | Ingredient lists from `List of Recipes (Chef)` oldid 45318; product/item ids and recipe pattern indices from the pinned Legacy definitions; Strong Arm ingredient counts also match video `3caQP-As968/00654`. Nineteen candidates remain incomplete, including Seared Stainberry, which has no ingredient row. |
| Blacksmith | 1 | Spunky Scrapper Hammer retains the local forging sample with reconstructed consumption/output selection; pattern item 2506 supplies Param1 11 and MinProfileRank 1. Sixty-four candidates remain incomplete. The wiki Manguana Statue item page gives five ingredient quantities, but its output definition/recipe-row binding is not established in the pinned item table. |
| Miner | 5 | Ore/flux rows from Miner oldid 46068; ingredient/output item ids from Legacy; MiniGameData rows 77 and 134–137 confirm type 9. Copper output quantity is five from client tutorial locale 1772250646. |

Plant ingredients shown without “Bundle of” on the wiki map to standard Type 1, Category 16 bundle definitions in Legacy. Each resolved ingredient definition id and count has its own provenance tag. The De'Flor recipe's Stainberry count is absent from the wiki, so its count defaults to one with an `(R)` tag. Chef output quantity defaults to one with `(R)` where sources are silent. XP defaults to zero with `(R)` for each resolved row; XP is not a field in the native recipe row and no recipe-specific XP evidence was found. The other four Miner rows are one bar per wiki recipe; Copper uses the client tutorial's five-bar quantity.

At the W51b checkpoint, the catalog's Copper Bar output was **5**, based on the client tutorial, while the local smelting sample still said 10. W60 below records the module wiring and resolves that checkpoint discrepancy.

The recipe index fields and `MinProfileRank` values come from the pinned recipe item definitions. Profile ids come from the job catalogue: Chef 45, Blacksmith 16, Miner 14. Every non-null item id is checked against the pinned Legacy id set. Output variants are not guessed where multiple tint-specific definitions exist.

## Checks

- `cargo test --manifest-path Cargo.toml --test recipes`: **5 passed**. Checks both catalogues, all item references, total candidate coverage and per-job counts, duplicate-key/id validation, current local samples, the Copper conflict, and recipe minigame types against the preserved `MiniGameData.txt`.
- `cargo test --manifest-path Cargo.toml --test recipes_wire`: **4 passed**. Synthetic add, update, remove and replacement bytes match the native reader field order.
- Both commands used `project/scripts/cargo.ps1` with `CARGO_TARGET_DIR=%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W51b`. Existing warnings in `chess.rs` and `housing.rs` are unrelated; the test targets also report expected unused items because they include `recipes.rs` through separate `#[path]` modules.
- Scoped formatting used the private Rust toolchain's `rustfmt.exe` on `src/recipes.rs`, `tests/recipes.rs` and `tests/recipes_wire.rs`; crate-wide formatting was not run.

## Open gaps

- No original server recipe rows, confirmed recipe packet capture, or known-recipes self-data field was recovered. Wire tests are synthetic.
- The original recipe-index-to-minigame bindings and most Blacksmith recipe components remain unknown; no client minigame id is inferred from `Param1`.
- Seared Stainberry and 18 other unresolved Chef candidates lack complete ingredients. Sixty-four Blacksmith candidates lack complete recipe content; output tint variants remain ambiguous.
- Blacksmith consumed counts, historical output quantities where the wiki/client is silent, and recipe XP remain reconstructed or unknown.

## W60 crafting recipe wiring (2026-10-13)

`cooking.rs`, `smelting.rs`, and `forging.rs` now load `data/content/recipes.json` through `RecipeCatalog` from `src/recipes.rs`. Each minigame selects by the configured stable `key`, preserves the selected key in its service state, and takes ingredient counts, output definition/count, job-rank requirement, minigame id, and XP from that row. The modules use the package catalog when a campaign content directory has no local `recipes.json`. Runtime catalog validation receives the ids present in that table; `tests/recipes.rs` independently checks all references against the pinned Legacy item definitions. Their legacy `begin(profile)` entry remains for the current gateway path; the new `begin_recipe(profile, key, rank, learned, inventory)` rejects unknown, unlearned, rank-locked, or under-stocked starts before changing state. Rejection tests compare the complete state before and after.

### Changed values and provenance

- The local Copper Bar transaction changed from 10 bars to **5** (`definition_id=21785`). The source is the Miner client tutorial locale `1772250646`, recorded above and in the Copper record's `field_sources`; the old 10-bar value was only the smelting module's local sample.
- All 31 resolved recipe ingredient definitions/counts and output definitions/counts now come from their `recipes.json` rows: 25 Chef, five Miner, and one Blacksmith sample. Row-level field provenance is stored beside every value. Chef and Miner rows use the pinned wiki/client and Legacy item-definition evidence described in W51b; Blacksmith sample consumption/output selection remains `(R)` reconstructed policy.
- Recipe minigame ids now come from each row when present: Miner 77 and 134–137 and Blacksmith 139. Those ids are corroborated by `MiniGameData.txt` as described above. Chef records with `minigame_id: null` reuse the configured game 46 as `(R)`; original Chef recipe-to-minigame bindings remain unknown.
- The transaction carries the row's `experience_points`. All current rows specify zero as `(R)` because native recipe rows have no XP field and no recipe-specific XP was recovered. The current shared gameplay transaction application does not yet apply this field; all present rewards remain zero.
- The minigame message layouts and accepted control messages retain their existing encodings. Selected recipe ids now supply the corresponding game-id words in minigame-info/start/phase/result packets. The only sample gameplay reward value change is Copper's catalog-backed output count, 10 to 5. Native protocol evidence remains in `Original Cooking.md`, `Native Crafting Results Lifecycle.md`, `Ordinary Crafting Quit Acceptance.md`, and the W51b native recipe-wire section above.

### W60 checks

Commands ran from `project/prototype/osfr-tools` with `CARGO_TARGET_DIR=%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W60` and `../../scripts/cargo.ps1`:

- `test --test cooking`: **11 passed, 1 ignored**; table-driven coverage starts all 25 Chef rows, checks exact consumes/output/XP, rejects locked/unknown/under-stocked starts without state mutation, and verifies no duplicate grant.
- `test --test smelting`: **11 passed, 1 ignored**; covers all five Miner rows and the five-bar Copper output.
- `test --test forging`: **13 passed, 1 ignored**; checks the Blacksmith sample's exact transaction and duplicate completion behavior.
- `test --test cooking_gameplay`, `smelting_gameplay`, `forging_gameplay`: each **1 passed**; private/native-content cases were ignored. The Copper gateway path now ends with five bars.
- `test --test type8_gateway`, `type8_harvesting_gateway`, `type8_native_repairs`: **10**, **11**, and **8 passed**, respectively.
- `test --test recipes`, `recipes_wire`: **5** and **4 passed**, respectively. The sample test now checks the module's recipe key instead of removed inline constants; Copper's count assertion cites tutorial locale `1772250646`.
- `test --lib cooking::`, `smelting::`, `forging::`: **4 passed** in each filtered module run.
- Scoped formatting used the private toolchain's `rustfmt.exe` on the three minigame modules and the directly related recipe/gameplay test files. No crate-wide formatter ran.

### Gateway hook for the W59 successor

After login/self-data is ready, the gateway should send known learned recipes in client message `0x003f` using operation 5 (replace-all). When a recipe is learned, send operation 1 (add) for that row; inventory changes should refresh its component values with operation 2. The wire encoders and native field order are in `src/recipes.rs`, with client dispatch/readers listed in the native recipe-wire section. The current gateway does not yet track learned recipes or map every content key to a recovered native recipe id, so those rows cannot all be emitted as verified client recipe rows yet. The native request itself stays inside the existing `PlayClient/BaseClient` channel; it is not a standalone SOE frame.

### W60 open gaps

- The current gateway still uses the compatibility `begin(profile)` path, which assumes the default learned recipe and does not receive player-specific rank/inventory. Its successor must call `begin_recipe` with those facts and must send the recipe-list/add messages above.
- Shared gameplay must apply nonzero `experience_points` atomically with inventory rewards if later evidence/content assigns any; current rows are zero and no XP change is claimed.
- Most Chef minigame bindings, many native recipe ids, and the historical Blacksmith recipe contents remain unresolved. `(R)` sample movies, recipe values, and XP remain explicitly reconstructed.
