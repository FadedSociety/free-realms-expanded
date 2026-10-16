# Content Batch: Regional NPCs

Status: W27 regional NPC pages converted to page-oriented content data, reference links reconciled, and `content_npcs` loader validation passed. Updated 2026-10-13.

## Inputs and provenance

- The 17 reviewed pages under `notes/Build Guides/Content/NPCs/` are the evidence source for NPC names, client name-string IDs, roles, quest text, vendor stock, services, and textual locations. Their row-level source tags are copied into each NPC's `source` and `field_sources`. The guide's precedence is client, then video, then wiki; conflicts are retained in the guides rather than silently collapsed.
- Existing `data/content/jobs.json`, `data/content/quests/*.json`, and `data/content/npcs_unresolved.json` supplied the reference keys and contexts. Only `giver_npc_id` changed in quest records; the existing quest content fields were left intact. There are no numeric turn-in NPC fields in these quest records.
- `lab/content/startup-snapshot/startup.json` contains 23 captured NPC spawn rows with actor/model/name-string IDs and transforms. The capture is in `FabledRealms`; its name-string IDs do not exactly join the regional page names. None of its transforms was assigned to a regional NPC.
- `lab/content/content-brief.md` sets client > video > wiki precedence. None of the 17 regional guides or reviewed video notes supplies a source-backed XYZ transform for these rows, so positions remain null. Nearby landmarks and distance text are preserved as location evidence, not converted into coordinates.
- Reconstructed package keys, regional `zone_id` values, role normalization, and aliases are marked `(R)`. The 17 zone IDs in `zones.json` follow page order and are not native zone IDs. NPCs without a resolved reference key receive a synthetic package key.

The guides contain 275 conflict-ledger entries across the 17 pages. Examples retained for review include Andrea Motorspark/Motorspeak in [[NPCs - Briarwood]], the competing 10/25 Robgoblin objectives in [[NPCs - Durango Zoo]], and Roadhouse placement in [[NPCs - Wilds]]. Guide decisions use the client when it resolves a discrepancy; unresolved competing claims remain recorded in the guide and source evidence.

## Batch counts

Counts are by source guide page. “Stock rows” means an NPC row has non-null vendor stock. Every position is unknown.

| Region page | NPC rows | Stock rows | Conflicts | Position known / unknown |
|---|---:|---:|---:|---:|
| Blackspore Swamp | 93 | 18 | 16 | 0 / 93 |
| Briarwood | 181 | 16 | 21 | 0 / 181 |
| Cobblestone Village | 20 | 4 | 14 | 0 / 20 |
| Durango Zoo | 10 | 0 | 11 | 0 / 10 |
| Farnum Farmstead | 24 | 5 | 17 | 0 / 24 |
| Greenwood Forest | 51 | 13 | 15 | 0 / 51 |
| Highroad Junction | 23 | 4 | 14 | 0 / 23 |
| Lakeshore | 39 | 9 | 15 | 0 / 39 |
| Merry Vale | 72 | 11 | 16 | 0 / 72 |
| Sanctuary | 133 | 17 | 15 | 0 / 133 |
| Seaside | 108 | 17 | 18 | 0 / 108 |
| Shrouded Glade | 81 | 9 | 16 | 0 / 81 |
| Snowhill | 95 | 23 | 19 | 0 / 95 |
| Stillwater Crossing | 28 | 5 | 15 | 0 / 28 |
| Sunstone Valley | 78 | 7 | 18 | 0 / 78 |
| Wilds | 118 | 12 | 19 | 0 / 118 |
| Wugachug | 45 | 7 | 16 | 0 / 45 |
| **Total** | **1,199** | **177** | **275** | **0 / 1,199** |

Of 1,199 rows, 104 have no resolved canonical zone; they remain in the regional guide file where their evidence was documented, with a reason for the null zone. Role counts overlap: 782 quest-giver, 166 trainer, 192 vendor, 234 ambient, and 82 enemy classifications. Roles are normalized content categories, not recovered runtime behavior. Five NPC rows preserve explicit quoted hand-in text under `quests_accepted`; those have no stable numeric turn-in quest/NPC link in this batch.

## Reference reconciliation

The original unresolved list had 465 reference keys. Name and region evidence resolved 277 keys to NPC records; 188 remain in `npcs_unresolved.json`:

| Remaining kind | References |
|---|---:|
| NPC, identity/placement still unresolved | 136 |
| Unknown/non-NPC target | 48 |
| Ambiguous target | 3 |
| Non-NPC target | 1 |
| **Total** | **188** |

Fifty-one remaining references have no single name; 113 have no canonical zone. `jobs.json` has 172 unique trainer references: 137 now resolve through NPC `reference_ids`, and 35 remain unresolved. Across the 651 quest records, 403 giver occurrences now point to resolved NPC IDs and 248 still point to unresolved references. The NPC batch test checks that every job and quest reference is either resolved or explicitly unresolved.

## Validation and integration limits

`src/content.rs` has a strict v1 runtime NPC-spawn schema; it does not represent these page records' names, vendor stock, or field-level evidence. To avoid changing shared schema files, `tests/content_npcs.rs` validates every regional record and every field source, then projects all 1,199 page rows plus the package references into a temporary v1 adapter before calling `ContentDb::load`. Adapter-only zero transforms and fallback zones are marked in the test and are not written as NPC positions. This validates the full existing content package and its cross-references, but does not register the regional page format as a production runtime consumer.

The focused `content_npcs` test passes and prints 277 resolved / 188 unresolved references, 403 resolved / 248 unresolved quest-giver occurrences, and 0 known / 1,199 unknown positions. The requested combined run reaches the existing `content_jobs` test, which fails at `tests/content_jobs.rs:62`: it expects all job and quest NPC references to remain in `npcs_unresolved.json`. That assertion reflects the old all-unresolved contract and conflicts with this batch's requirement to remove resolved references; the test file is outside W27 ownership and was not changed.

Commands run:

```powershell
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W27'
& '..\..\scripts\cargo.ps1' test --test content_npcs --test content_jobs
& '..\..\scripts\cargo.ps1' test --test content_npcs '--' '--nocapture'
```

The combined command fails on the stale `content_jobs` assertion before `content_npcs` runs. The separate `content_npcs` command passes and displays the counts above. The test file was formatted with the private toolchain's scoped `rustfmt.exe --edition 2024 tests\content_npcs.rs`; no crate-wide formatting was run. No native client, new reverse-engineering, or asset-pipeline work was needed for this content conversion.
