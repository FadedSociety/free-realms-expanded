# 024 Furniture catalogue

Status: mapping and checker implemented and locally verified, 2026-10-09 03:07 UTC. This is not a
native or gameplay acceptance claim. Evidence label: **ORIGINAL stock/asset/locale facts measured
against RECONSTRUCTED server content**.

Owned paths (work item 024): `prototype/osfr-tools/src/restoration/piece_024.rs`,
`prototype/osfr-tools/tests/piece_024.rs`, `scripts/pieces/piece_024.py`,
`tests/pieces/test_piece_024.py`, this note and `lab/reports/piece-024-<stamp>/`. No shared module,
manifest, content, save, launcher, native UI or release path was touched.

## What the catalogue is built from

| Input | Identity | Role |
| --- | --- | --- |
| `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json` | AGPL-3.0, `raisingkaines/FreeRealms-Legacy` revision `9e082025391177d5a33ee18c714ddd06b457a695`, sha256 `40a3cc79…` | 58,795 stock item rows; `Id`, `NameId`, `DescriptionId`, `Icon.Id`, `CategoryId`, `ModelName` |
| `lab/manifests/rust-asset-catalog.tsv` | Rust pack reader over the 24 preserved packs, sha256 `12bfd0f8…` | 108,543 distinct asset names |
| `lab/reports/locale-id-to-text.json` | unmodified FreeRealmsLocaleTools export of the supplied client `locale/en_us_data.dat`/`.dir`, sha256 `f1f22a30…` | 85,932 name ids |
| `lab/references/opening-anvil-placement-v1/content/inventory.json` | current reconstructed content, sha256 `1f8733ea…` | 1,647 reconstructed definitions |
| `lab/references/freerealms-wiki/wiki.sqlite` | read-only community corpus, sha256 `b9d7b9f1…` | corroboration only, 3,274 titles |

The furniture selector is the recovered `hsg_` model-file naming convention, not a server rule: no
recovered table states "this item is furniture". One row (15490 "Wolf") ships an `Icon` block with
only a `TintId`, so the icon id is optional in both implementations.

## Measured relationships

| Measure | Value |
| --- | --- |
| Furniture rows in stock (`hsg_` model) | 1,167 |
| Declared model file absent from the preserved packs | 5 |
| Name or description id absent from the locale export | 3 |
| No `icon_item_<model stem>*` asset (inferred link) | 68 |
| Housing-category rows with a non-`hsg_` model | 34 |
| Furniture ids defined by the reconstructed content | 31 (33 records) |
| Furniture ids present in stock but not reconstructed | 1,136 |
| Reconstructed furniture ids absent from stock | 0 |
| Reconstructed model mismatches | 0 |
| Exact wiki title matches for furniture names | 31 |

Housing categories are derived by membership (a category carrying at least 20 `hsg_` rows) and are
recorded, not hard-coded: 4 (Building Blocks), 52 (Wall Hangings), 53 (Furniture & Decorations), 54
(Decorations), 56 (Exterior Decorations), 57 (Outdoor Equipment), 147 (Blocks). The 34 rows in those
categories without an `hsg_` model are music blocks (`mkt_piano_block_*`, `mkt_fart_block_*`), the
Yeti Domino (`sasquatch_boss.adr`) and the Jackpot Plant (`mkt_coinplant_small_bare_water_full.agr`);
they are housing-category stock, not furniture models, and stay out of the 1,167.

### Missing entries

Models declared by stock furniture rows but absent from every preserved pack (exact name, case
insensitive):

| Item | Name | Model |
| --- | --- | --- |
| 9046 | (NameId `-1`) | `hsg_pirateship_ghost_01.adr` |
| 16872 | Basic Swimming Pool | `hsg_pool_basic_01_complete.agr` |
| 16874 | Ramp Scaffolding | `hsg_ramp_small_02.adr` |
| 18112 | Construction Barrier TNT | `hsg_constructionbarrier_01.adr` |
| 18136 | Giant Toilet Bowl | `hsg_pool_toilet_01_complete.agr` |

The two pool rows are the interesting ones: the packs carry `hsg_pool_basic_01.adr`,
`hsg_pool_basic_01.dds`, `hsg_pool_basic_01.cdt`, `hsg_pool_basic_01_lod0.dme` and the toilet-bowl
equivalents, but not the `_complete.agr` the item row names. Whether the client composes the pool
from those parts or the `_complete.agr` was shipped elsewhere is **not established**; the
relationship is recorded as missing rather than repaired.

Locale gaps: 9046 (`NameId -1`), 15978 (`hsg_block_01_ice.adr`, `NameId 21674`) and 18167
(`hsg_well_01.adr`, `NameId 147996`) have no name entry in the current export even though two of the
three models exist. A second shipped locale table (`tcg/locale`) and retired strings were not
searched; the gap stays open.

Icon gap: 68 rows have no asset whose name starts `icon_item_<model stem>`. Some are cash crops
(3067 Cashberry Shrub), clouds (14303/14304), holiday sets (16723–16754) and the removed ghost ship.
The `icon_item_<model stem>` link is an **INFERRED naming convention**; the recovered item row
carries a numeric `Icon.Id` and no recovered table maps that id to an asset, so this list is a
relationship report, not proof that those items have no icon in the native client.

## Stock versus reconstructed inventory

The reconstructed content defines 33 furniture records across 31 unique ids. All 33 carry the stock
model and texture (0 model mismatches) and every id exists in the stock catalogue. Two ids appear
twice with byte-identical records (17750 `hsg_halloween_doordeco_01.adr`, 27943
`hsg_chair_lodge_01.adr`); the Rust inventory loader tolerates identical duplicates, so this is
content-file noise rather than a load failure, but a regenerated content file should not repeat
them. The remaining 1,136 furniture definitions are stock-only: they exist in the original
catalogue and are not served by the reconstructed inventory.

## What is reconstructed here

Nothing about the catalogue. The module and script read preserved rows and measure them; the only
invented numbers are the reporting threshold (a category is called housing at 20 `hsg_` rows) and
the icon prefix convention, both labelled. No price, stack size, reward, placement rule or physics
value is produced.

## Verification

| Check | Command | Result |
| --- | --- | --- |
| Rust RED | `cargo test --test piece_024` (first revision) | 3 failed on locale/icon/delta behaviour |
| Rust GREEN | `cargo test --test piece_024` | 6 passed, 1 ignored |
| Private Rust gate | `OSFR_PIECE_024_*` env + `--ignored --exact private_furniture_catalogue_matches_preserved_lab_inputs` | 1 passed; recomputes 1,167 / 5 / 3 / 31 / [17750,27943] / 1,136 from the real files |
| Python RED | `python -m unittest discover -s tests\pieces -p "test_piece_024.py"` (first revision) | 5 failed on missing locale/icon/delta |
| Python GREEN | same | 8 passed, including the real-source count check |
| Regression sweep | `cargo test --lib` and `--test housing --test inventory --test item_categories` | 24, 26, 12 (2 ignored) and 3 (1 ignored) passed; 0 failed |
| Format | `rustfmt --check` on the two owned Rust files | clean |
| Artefacts | `scripts/pieces/piece_024.py --output lab/reports/piece-024-20261009T030716Z` | `furniture-catalogue.json` sha256 `c7ef0a9c…`, `findings.json` sha256 `30c9c946…` |

The whole-repository `cargo fmt --check` and the complete ordinary suite currently fail on other
workers' in-flight files (`tests/piece_008.rs` does not compile; the `piece_009`, `011`–`021`, `023`
and `026` files are unformatted). Neither list contains `piece_024` after formatting. RED/GREEN logs
and the blocked full-suite attempt are in `lab/reports/piece-024-20261009T025508Z/`.

Reproduction from the project directory:

```powershell
& '..\lab\tools\universal-modder-env\Scripts\python.exe' -m unittest discover -s tests\pieces -p "test_piece_024.py" -v
& '..\lab\tools\universal-modder-env\Scripts\python.exe' 'scripts\pieces\piece_024.py' --output '..\lab\reports\piece-024-<stamp>'
.\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','piece_024','--','--ignored','--exact','private_furniture_catalogue_matches_preserved_lab_inputs')
```

## Remaining gates

1. Lead integration: the module is a `#[path]` include; register or move it and drop the include.
2. Decide the served furniture set. The current reconstructed inventory ships 31 furniture ids out
   of 1,167; item 025/026 wiring and any content regeneration should use this catalogue instead of
   adding rows by hand.
3. Wire integration (item 025) and inventory conservation (item 026) still own placement and stack
   accounting; this piece only measures catalogue relationships.
4. Native acceptance is untouched: no client run, no housing lot, no placement, and no inventory
   screen was exercised here. A native gate would have to show a furniture item in the real client's
   inventory/catalogue.
5. Open evidence gaps: the five declared model files, the three locale ids and the icon-id mapping
   remain unresolved and are listed above rather than filled in.

Cross-links: [[Housing Restoration]], [[Inventory Catalog Framing Compatibility]], [[../Design/MMO
Worker Contract]], [[../../Design/Rust Revival Roadmap]].
