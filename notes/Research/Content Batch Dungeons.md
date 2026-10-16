# Content Batch: Dungeons

Date: 2026-10-13  
Scope: W28, 74 dungeon guide pages plus the opening cave definition used by the local encounter implementation.

## Output

- `prototype/osfr-tools/data/content/dungeons/`: 75 versioned JSON files. The 74 guide-page IDs are filename slugs, not recovered client dungeon IDs; `opening_cave.json` is the additional accepted local encounter.
- `prototype/osfr-tools/data/content/dungeons_unresolved.json`: the exact current set of external references that do not resolve in the content catalogues available to this batch.
- `prototype/osfr-tools/tests/content_dungeons.rs`: per-file parsing/source/null checks, `ContentDb` validation of the typed dungeon projections, catalogue-reference parity, and opening-cave parity.

Each file uses a version-1 envelope with a strict `record` projection that `src/content.rs::Dungeon` can load, plus a `dungeon_data` detail object. The detail object retains entry requirements, entrance, scene name, goals, phases, enemy and boss rows, reward claims, exit behavior, source tags, and named external references. Guide claims retain their `[client]`, `[video]`, and `[wiki]` provenance. Inferred values are tagged `(R)`; missing values use `value: null` with a reason in the detail object.

## Extraction rules and provenance

- Requirements come from each page's `Requirements` table and the `Dungeons.md` index. Index minimums and membership/job statements remain guide claims; they are not asserted to be historical server authorization rules.
- The entrance region comes from page frontmatter. A named clicked NPC or object is recorded when the `Overview` entrance text identifies one; otherwise its value is null with a reason.
- Goal rows are preserved as written. A phase or wave is included only where a page labels one. Enemy names and roles come from `Enemies and boss`; HP and damage are populated only when a numeric value is stated in that row. Model IDs remain unknown unless explicitly mapped.
- Prize-wheel rows are reported as conditional item names. A missing item-definition ID or grant count stays null. Explicit numeric coin outcomes use the `RewardGrant` shape with their stated condition; conditional wheel outcomes are not promoted to a guaranteed dungeon payout.
- Exit behavior is included only with page evidence or the shared index's cited exit marker. No destination zone is inferred.
- The actual index is `notes/Build Guides/Content/Dungeons/Dungeons.md`; the task's sibling path `notes/Build Guides/Content/Dungeons.md` does not exist. It indexes 74 written pages in this batch. Index-only pending entries and the two explicitly non-dungeon entries are not emitted as dungeon pages.

## Counts

| Measure | Count |
|---|---:|
| Guide pages emitted | 74 |
| Opening cave definitions emitted | 1 |
| JSON files | 75 |
| External references in dungeon detail records | 2,095 |
| Resolved against current content catalogues | 87 |
| Listed in `dungeons_unresolved.json` | 2,008 |
| Null detail wrappers checked | 5,281 |
| Null detail wrappers missing a reason | 0 |

Reference totals and unresolved counts by kind:

| Kind | References | Unresolved |
|---|---:|---:|
| Item | 1,667 | 1,667 |
| NPC | 303 | 239 |
| Zone | 75 | 52 |
| Object | 41 | 41 |
| Quest | 9 | 9 |

The 64 NPC matches are unique, exact-name matches to reconstructed regional `npc_id` content keys. Those keys are not original client actor GUIDs. The 23 resolved zones are exact matches to the current zone catalogue. No item, object, or quest reference resolved in this batch.

Repeated source-tag occurrences across nested fields total 19,470: 3,053 `client`, 988 `video`, 9,204 `wiki`, and 6,225 `reconstructed`. These are tag occurrences, not unique documents or independent evidence counts. The unresolved list carries 1,975 wiki, 498 client, 127 video, and 7 reconstructed source-tag occurrences; references can carry more than one source.

## Opening cave parity and limits

`opening_cave.json` copies encounter 411 from `lab/references/opening-journal-text-v2/content/encounters.json`, the local content package used by `src/encounter.rs`. It records zone/scene `411` / `sg_npx_cavern_01`; guide `Darkthorne` (GUID `5715144128442986497`, model `353`, name string `441292`); three Gloamlings (model `4201`, 150 HP, 20 damage each); Gloam Overlord (model `3956`, 500 HP, 40 damage); and a configured grant of 50 coins plus item definition `1195` x1. The source tag explicitly calls this a local reconstructed config, not an original client or server record. `EncounterEngine::exit` in `src/encounter.rs` restores the saved return location; the exit destination is therefore left null per player.

The parity test compares the generated native-config mirror and the exposed scene, guide, enemy/stat, and typed `RewardGrant` values against encounter 411. Its test fixture inserts temporary zone 411 and item 1195 rows so `ContentDb` can validate the typed projection. Those fixture rows do not modify the production catalogues; the actual dungeon references remain unresolved in the output list where the catalogue lacks them.

## Schema gaps

`src/content.rs::Dungeon` does not model entrance initiators, instance scene names, authored phases, enemy/boss definitions or stats, conditional wheel rewards, object references, or exit destinations. Its typed encounter/reward records cannot faithfully represent the pages' full claims. The strict typed `record` therefore holds only the supported projection; `dungeon_data` keeps the rest without editing shared schema files. `ContentDb::load` validates the projected records, while the batch test checks the detail envelope, source arrays, null reasons, and reference-list parity. A typed detail schema and an integration decision for this extension remain open.

Guide pages do not provide an exact instance scene-name field, so their scene values remain null. The opening cave scene name comes from the reconstructed local encounter config. Most guide enemy names, prize items, quest titles, and entrance targets still lack numeric content crosswalks. The unresolved list is a tracking surface for those gaps, not proof that the references do not exist in the game.

## Commands and results

The temporary batch builder was run from the project root and removed after generating the owned JSON outputs:

```powershell
python -I prototype/osfr-tools/data/content/dungeons/_generate.py
```

Result: 74 guide pages, 75 files, 2,008 unresolved references; breakdown: item 1,667, NPC 239, object 41, quest 9, zone 52.

Only the owned test file was formatted:

```powershell
%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\rustfmt.exe tests/content_dungeons.rs
```

Focused validation used the requested private Rust toolchain and target directory:

```powershell
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W28'
..\..\scripts\cargo.ps1 test --test content_dungeons
```

Result: 3 passed, 0 failed. The build emitted one existing `src/housing.rs::Lot::instance_index` dead-code warning.

## W58 enemy crosswalk (2026-10-13)

All 75 files in `prototype/osfr-tools/data/content/dungeons/` now carry an `enemy_refs` array inside `records[0].dungeon_data`. The strict projected `record` was left unchanged. Each entry preserves the source field (`enemies` or `bosses`), row index, exact name, status, definition ID when resolved, source evidence, and a reason when unresolved. Bosses duplicated in both source fields remain two references to the same definition where applicable.

| Coverage | Result |
|---|---:|
| Dungeon files | 75 |
| Named enemy/boss occurrences | 367 |
| Unique names | 250 |
| Resolved occurrences | 45 |
| Unresolved occurrences with reason | 322 |
| Unique names resolved | 32 |
| Unique names unresolved | 218 |
| Added definitions | 26 (`930000..930025`) |

The resolved subset uses the Legacy NPC model/name table as a starting crosswalk, checks the selected ADR filename against the client catalog, and resolves the numeric name string against the client locale map. Exact or singularized name matching is used only where the recorded candidate is unique. A same-row alias such as Bigfoot/Robbie can point to one definition. None of the new numeric model mappings is independently verified against a native dungeon `AddNpc`. Every other occurrence remains explicitly unresolved; `enemies_unresolved.json` retains its guide/client evidence and the reason a Legacy candidate was not accepted.

World-zone leads are recorded separately: 82 `enemy_kind` rows from 74 regional NPC files (70 unique description texts) remain unresolved because the regional content supplies descriptive prose rather than a hostile actor/model/stat record. This is an incomplete world crosswalk, not a claim that these NPCs do not exist.

Ability strings and combat-related dev strings documented in the dungeon guides are preserved in `enemies_unresolved.json` for seven matched definitions (28 evidence strings; 26 cited client `loc` references). Their wording does not establish that each is a live ability or reveal its behavior. No numeric loot-table references were identified; the added rows use zero reconstructed reward values. Combat values, their formulas, the unknown loot status, and the strict-loader schema gap are recorded in [[World Enemies]].

### W58 commands and results

The test target used the private Rust toolchain and an isolated build directory:

```powershell
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W58'
..\..\scripts\cargo.ps1 test --test content_enemies '--' '--nocapture'
..\..\scripts\cargo.ps1 test --test content_dungeons
..\..\scripts\cargo.ps1 test --test enemies
```

`content_enemies`: 2 passed; printed `45/367` resolved dungeon name occurrences and `0/82` world `enemy_kind` rows resolved. `content_dungeons`: 3 passed. Existing `enemies`: 6 passed. Builds emitted two existing dead-code warnings in `src/chess.rs` and `src/housing.rs`; no source or Cargo manifest was changed for W58.
