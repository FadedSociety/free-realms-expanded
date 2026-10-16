# 097 Asset reference validation

Status: checker and reusable module implemented and locally verified, 2026-10-09 04:31 UTC. This is
a static catalogue audit, not a native or gameplay acceptance claim. Evidence label: **ORIGINAL
preserved catalogue rows measured against ORIGINAL pack names and locale export; two
RECONSTRUCTED rules labelled below.**

Owned paths (work item 097): `prototype/osfr-tools/src/restoration/piece_097.rs`,
`prototype/osfr-tools/tests/piece_097.rs`, `scripts/pieces/piece_097.py`,
`tests/pieces/test_piece_097.py`, this note and `../lab/reports/piece-097-20261009T042918Z/`. No
shared module, manifest, content, save, launcher, native UI or release path was touched; the Rust
module is a `#[path]` include until the lead registers it.

## What was validated

The recovered stock item catalogue makes seven kinds of reference. This piece resolves each of them
against the preserved target and reports every entry that does not resolve:

| Item cell | Target | Resolution rule |
| --- | --- | --- |
| `NameId` | locale export | id present in `locale-id-to-text.json` |
| `DescriptionId` | locale export | same, when the cell is present |
| `Class` | `ItemClasses.txt` | class id has a row |
| `CategoryId` | `ItemCategories.txt` | category id has a row |
| `ModelName` | pack catalogue | exact lowercased asset name, or a `<race>`/`<gender>` template expanded over the four PC variants |
| `Param1` | `Models.txt` | model id, only for item types whose other rows already resolve |
| (sibling tables) | locale export | `ItemClasses.txt` `NAME_ID` and `ItemCategories.txt` `NAME_ID` |

`Models.txt` rows are themselves validated: their declared model file must exist in the packs.

## Provenance (read-only inputs)

| Input | Bytes | SHA-256 |
| --- | --- | --- |
| `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json` (58,795 rows; AGPL-3.0, revision `9e082025391177d5a33ee18c714ddd06b457a695`) | 28,800,315 | `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025` |
| `.../ItemClasses.txt` (184 rows) | 4,347 | `73c2a9377477ff88e6f4966614930c3d375cd9d5904fddaa7544ec24f0c2f567` |
| `.../ItemCategories.txt` (71 rows) | 1,744 | `f26e76be792917e98286b906567403d0dff8ff1208b335c22751efbfaf809084` |
| `.../Models.txt` (3,743 rows) | 431,579 | `0bad5a154c3ef855744349eb03eb3ab02e7d8dfc3d29fd0f601257fce95076be` |
| `lab/manifests/rust-asset-catalog.tsv` (108,543 distinct names over the 24 preserved packs) | 11,218,192 | `12bfd0f81236ffc02e6a9a661e1ffb14890a2bd4ac2306c14b20e9cbe445b1c9` |
| `lab/reports/locale-id-to-text.json` (85,932 ids, unmodified community locale reader export) | 6,788,031 | `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844` |

The three reference tables ship inside the same pinned community package as
`ClientItemDefinitions.json`; they are the tables the community server itself loads. No community
tool validates the item-to-asset/locale/model references (`rg` over the pinned repositories found
only runtime consumers such as `BaseItemDefinition.cs`), so this checker fills a demonstrated gap
instead of duplicating an existing reader.

## Boundary: original versus reconstruction

ORIGINAL: every catalogue row, reference row, asset name and locale id read here; every count,
id list and file name below.

RECONSTRUCTED, and the only invented elements in this piece:

1. The four PC model variants `human_m`, `human_f`, `fairy_m`, `fairy_f` used to expand
   `<race>`/`<gender>` templates. No recovered table declares the set. It is the only expansion
   under which the catalogue nearly closes (31,957 of 31,962 templates resolve), and it matches the
   four character-creation rows in [[Character Creation Audit]].
2. The `Param1` conflict rule: a `Param1` value that does not resolve is only reported as a
   conflict for item types that resolve at least one other `Param1` value. Types that resolve none
   stay in the coverage report rather than being called missing.

Nothing was repaired, filled in or guessed: no missing model file, locale string, category row or
model row was added, and no catalogue file was written.

## Measured relationships

| Measure | Value |
| --- | --- |
| Stock item rows | 58,795 |
| Duplicate item ids (conflicting rows) | 0 |
| Rows with no `NameId` cell / sentinel `NameId` (<= 0) / dangling `NameId` | 250 / 4 / 113 |
| Rows with no `DescriptionId` cell / sentinel / dangling | 3,533 / 7 / 16 |
| Literal `ModelName` rows / absent from every preserved pack | 9,209 / 21 |
| `<race>`/`<gender>` template rows / no resolved PC variant | 31,962 / 5 |
| Rows with an unrecovered template token (`<descriptor>`, `<hairtype>`) | 8,617 (`<descriptor>` 8,586, `<hairtype>` 31) |
| `Class` references with no `ItemClasses.txt` row | 0 (positive control) |
| `CategoryId` references with no `ItemCategories.txt` row | 29 |
| Class rows with an unrecovered `NAME_ID` | 1 (class 999) |
| Category rows with an unrecovered `NAME_ID` | 0 |
| `Models.txt` rows / declaring a file absent from the packs | 3,743 / 187 |
| `Models.txt` rows flagged `IS_VALID_FOR_PC` | 4 |
| `Param1` positive rows / resolving to a `Models.txt` id | 2,867 / 2,525 |
| `Param1` conflicts (unresolved inside a type that resolves another value) | 340 over item types 1, 2, 6, 14, 19 |
| Item types that resolve every `Param1` value they carry | 3, 4, 5, 9, 10, 11, 12, 13, 16, 17, 18, 27, 30 |

## Missing and conflicting entries

Literal model files declared by item rows but absent from all 24 preserved packs (21):

| Item | `ModelName` |
| --- | --- |
| 9046 | `hsg_pirateship_ghost_01.adr` |
| 11039 | `archer_{Error with ID}` (upstream literal placeholder, not a file name) |
| 16872 | `hsg_pool_basic_01_complete.agr` |
| 16874 | `hsg_ramp_small_02.adr` |
| 17523 | `vulture_m_undead.adr` |
| 17543 | `firefly.adr` |
| 17559 | `toad_black.adr` |
| 17562 | `treesoldier_ice.adr` |
| 17563 | `treesoldier_mushroom.adr` |
| 18112 | `hsg_constructionbarrier_01.adr` |
| 18136 | `hsg_pool_toilet_01_complete.agr` |
| 30589 | `vanitypet_candycorn_critter.adr` |
| 70428 / 70430 / 70431 | `penguin.adr` / `turtle.adr` / `frog.adr` |
| 77484-77489 | `SkinTone 1` ... `SkinTone 6` (upstream literal cells that are not file names) |

The five `hsg_` entries and the two `_complete.agr` pool rows match the furniture findings of
[[024 Furniture catalogue]] exactly; the other sixteen are outside that piece's furniture selector.
Whether the packs outside the preserved 24 ever carried these files is **not established**.

`<race>`/`<gender>` templates with no PC variant present (5): item 79182
`freestyle_<race>_f_handss_geniebracelets.adr`, 79274 `freestyle_<race>_<gender>_misc_clockworkcowboy.adr`,
and 79343/79344/79345 `freestyle_<race>_<gender>_shoulders_dwarfbiker.adr`.

Dangling `NameId` values (113, full list in `findings.json`): 15978, 18167, 29062, 29099, 55018,
57007, 70324, 76493, 76628, plus the ranges 52418-52487 (70 rows) and 55605-55638 (34 rows).
Dangling `DescriptionId` values (16): 57007, 78105, 79229 and 79231-79243.

`CategoryId` values referenced by items but undefined in `ItemCategories.txt` (29): 42, 60, 120,
122, 123, 125, 127, 135, 137, 138, 148, 149, 214, 247 and 6026-6040. Class 999 in
`ItemClasses.txt` carries a `NAME_ID` the locale export does not contain.

`Models.txt` rows whose declared file is absent from the packs (187, full list in `findings.json`):
the first entries are retired or QA actor models such as `dwarf_m_miner_01.adr` (id 37),
`youngling01.adr`/`yoda01.adr`/`watto01.adr`/`droid01.adr` (55, 56, 66, 67),
`human_m_qa_test.agr` (225). Only 4 of the 3,743 rows are flagged `IS_VALID_FOR_PC`.

`Param1` conflicts (340, full list in `findings.json`) cluster in item types 1 (13), 2 (13), 6
(57), 14 (12) and 19 (245). The type 14 misses include values such as 5,000/10,000/500,000, which
read as amounts rather than model ids, so `Param1`'s meaning is **not** one recovered field for
every type; the checker reports the mixed coverage rather than asserting a rule.

## Cross-check against piece 024

[[024 Furniture catalogue]] measured the same three inputs with a narrower selector. Its three
furniture locale gaps appear here as 9046 (`NameId -1`, classified here as a sentinel rather than a
dangling id), 15978 and 18167, and its five missing furniture models are a subset of the 21 literal
misses above. The two implementations agree on every overlapping entry.

## Verification

RED before implementation:

```text
scripts/cargo.ps1 test ... --test piece_097
  error: couldn't find file `tests\..\src\restoration\piece_097.rs`      (exit 101, red-rust.log)
python -m unittest discover -s tests/pieces -p "test_piece_097.py"
  FileNotFoundError: ... scripts\pieces\piece_097.py                     (exit 1, red-python.log)
```

GREEN after implementation (exact commands run from the project directory):

```powershell
scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','piece_097')
# test result: ok. 8 passed; 0 failed; 1 ignored  (the ignored test is the private gate)

# private gate over the preserved lab inputs
$env:OSFR_PIECE_097_STOCK=...; $env:OSFR_PIECE_097_ASSETS=...; $env:OSFR_PIECE_097_LOCALE=...
$env:OSFR_PIECE_097_CLASSES=...; $env:OSFR_PIECE_097_CATEGORIES=...; $env:OSFR_PIECE_097_MODELS=...
scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','piece_097','--','--ignored','--exact','private_catalogue_references_match_preserved_lab_inputs')
# test result: ok. 1 passed; 0 failed  (asserts 58,795 / 113 / 16 / 21 / 5 / 8,617 / 29 / 187 / 340)

& '..\lab\tools\universal-modder-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_097.py"
# Ran 11 tests ... OK

& '..\lab\tools\universal-modder-env\Scripts\python.exe' 'scripts\pieces\piece_097.py' --output '..\lab\reports\piece-097-20261009T042918Z'
# items=58795 duplicate_item_ids=0 name_id_absent=250 name_id_unset=4 missing_name_ids=113
# description_id_absent=3533 missing_description_ids=16 literal_model_rows=9209
# missing_literal_models=21 race_gender_model_rows=31962 missing_race_gender_models=5
# unmapped_template_rows=8617 missing_item_classes=0 missing_item_categories=29
# model_asset_gaps=187 param1_conflicts=340 asset_names=108543 locale_ids=85932

# formatting of the two owned Rust files
rustfmt --check --edition 2024 prototype\osfr-tools\src\restoration\piece_097.rs prototype\osfr-tools\tests\piece_097.rs
# exit 0
```

Evidence bundle `../lab/reports/piece-097-20261009T042918Z/`: `findings.json` (every missing and
conflicting entry, sha256 `a56f43fe44c0b248c2bdc7d9c40c3bfe31a96a4e4ce42c733c37cf25ad1004d9`),
`provenance.json` (input paths, digests, counts, the two reconstructed rules), `run.log`, the RED
and GREEN logs, the private-gate log, the rustfmt logs and the reconnaissance scripts.

What the tests actually assert: unknown `<...>` tokens and `{...}` braces classify apart; expansion
dedupes survivors and never invents a variant; malformed reference rows raise with table and line;
duplicate ids in the reference tables are rejected; every finding bucket reports the right entry for
synthetic fixtures; anchors, locale, class, category and model lookups are case-insensitive; the
real inputs match their pinned hashes and re-derive the 58,795 / 113 / 16 / 21 / 5 / 8,617 / 29 /
187 / 340 counts.

## Remaining integration and native gates

1. Lead integration: register `pub mod piece_097;` under `src/restoration/` and drop the `#[path]`
   include from the test. The module is not referenced by `src/lib.rs` yet.
2. No runtime consumer: nothing in the gateway enforces these findings. A follow-up could gate
   served inventory definitions (the reconstructed `inventory.json` ids) against this report before
   they are served; that is item 081/083/096 territory, not this piece.
3. Native acceptance is untouched: no client run, no inventory screen and no rendered model was
   exercised here. A native gate would have to show an affected item in the real client.
4. Open evidence gaps, left explicit rather than repaired: the icon-id-to-asset mapping, the
   `<descriptor>`/`<hairtype>` expansion values, the second shipped locale table
   (`tcg/locale`) and any pack content outside the preserved 24 archives were not searched; the
   340 `Param1` conflicts need per-type semantics before any code treats `Param1` as a model id.

## Skills applied

`game-recon` for identifying the catalogue and its sibling reference tables as the recovered data
surface; `game-research-websearch`/community-first practice for checking the pinned AGPL package
and confirming no community validator covers item-to-asset references; `reverse-engineering`
discipline for separating recovered field names from the two reconstructed rules; RTK (normal
commands first), Ponytail (one pass per table, sets and vectors, no index files) and Caveman
(concise reporting). `asset-pipeline` and `game-automation` were not needed: no asset conversion
and the shared native window is lead-controlled.

Cross-links: [[../MMO Work Queue]], [[024 Furniture catalogue]], [[Character Creation Audit]],
[[Inventory Catalog Framing Compatibility]], [[../Design/MMO Worker Contract]].
