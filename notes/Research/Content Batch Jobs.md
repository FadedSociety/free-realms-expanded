# Content Batch Jobs (W22)

Status: the first job batch is serialized in the unchanged schema v1 and passes `ContentDb` validation through the batch test adapter. This records guide evidence and reconstructed package keys; it does not establish original server quest IDs, payout policy, or complete game behavior.

## Inputs and provenance

Used `notes/Build Guides/Build Guides.md`, `notes/Build Guides/Content/Content.md`, all 17 pages in `notes/Build Guides/Content/Jobs/`, `notes/Research/Content Data Model.md`, `prototype/osfr-tools/src/content.rs`, its content tests/fixtures, and `lab/content/content-brief.md`. Source precedence is client > video > wiki. Each job, quest, and unresolved-NPC record retains page source tags with the relevant guide section/row as evidence. A client/video/wiki row count below means the source kind is present on that record; counts overlap.

Client profile IDs in `jobs.json` come from `notes/Build Guides/Gameplay/Items and Inventory.md:143` (client job option values). Job name, category, level cap, and member status come from the job-page frontmatter and overview; source tags preserve which sources support each page value. Quest title IDs come from the row's Title IDs cell. Quest giver, prerequisite, objective text, reward, and ordering evidence comes from each page's quest table and source tags. The 17 pages do not provide server quest numeric IDs; the stable quest IDs and NPC reference IDs are reconstructed package keys marked `(R)`. Quest array order follows the page order and is not represented by a separate schema field.

Targeted `cq.py` checks were read-only: `loc 'Path of the Archer'` returned client title IDs UCDT 346916944 / UGDT 956709502 and the client objective; `wiki Archer` returned the archived Archer page (revision 2022-03-20); `vid-quest 'Path of the Archer'` returned no video match. These checks were a sample cross-check, not a substitute for the source annotations on every row.

## Batch counts

“Objective text” counts quest rows with usable objective text in the guide, not individual objective sentences. The current schema could not serialize these free-text objectives as typed goals, so every JSON `objectives` array is empty. “Known-value rewards” counts quest rows with a numeric coin grant serialized; item/XP/qualitative rewards are not counted as known numeric grants. Coin grants were supported by video evidence on the Adventurer page. C/V/W = rows tagged client/video/wiki, respectively.

| Job | Quest rows | Objective text rows | Known-value reward rows | C/V/W rows |
|---|---:|---:|---:|---:|
| Adventurer | 58 | 55 | 6 | 58 / 18 / 45 |
| Archer | 34 | 31 | 0 | 34 / 0 / 32 |
| Blacksmith | 42 | 35 | 0 | 42 / 2 / 36 |
| Brawler | 43 | 41 | 0 | 43 / 7 / 43 |
| Card Duelist | 63 | 62 | 0 | 58 / 12 / 55 |
| Chef | 33 | 32 | 0 | 33 / 5 / 31 |
| Demo Derby Driver | 46 | 45 | 0 | 46 / 3 / 0 |
| Fisherman | 50 | 50 | 0 | 50 / 1 / 4 |
| Kart Driver | 49 | 47 | 0 | 49 / 11 / 5 |
| Medic | 33 | 32 | 0 | 32 / 3 / 33 |
| Miner | 29 | 23 | 0 | 29 / 3 / 29 |
| Ninja | 25 | 24 | 0 | 25 / 4 / 25 |
| Pet Trainer | 26 | 26 | 0 | 26 / 0 / 25 |
| Postman | 33 | 33 | 0 | 33 / 7 / 31 |
| Soccer Star | 30 | 29 | 0 | 30 / 1 / 1 |
| Warrior | 24 | 23 | 0 | 24 / 1 / 24 |
| Wizard | 33 | 33 | 0 | 33 / 1 / 32 |
| **Total** | **651** | **621** | **6** | **645 / 79 / 451** |

The six serialized numeric grants are 3, 4, 16, 18, 13, and 1 coins (55 total), each in an Adventurer quest row. Quest source counts are not exclusive; for example a row can cite both client text and wiki quest order.

The unresolved reference list has 465 unique package references. 51 have no resolved NPC name and 236 have no canonical zone. Candidate names and guide place labels are retained where available; no NPC spawn is asserted by this list. The batch test confirms every job trainer and quest giver reference appears in this explicit list.

## Schema limits and nulls

`ContentDb` v1 strictly rejects extra fields. `Job` has no job name string ID, unlock-rule, or starting-gear fields. `Quest` has no display-name, explicit order, or free-text objective field. Its typed goals require numeric NPC/item/POI/zone/defeat targets, but this batch lacks keyed definitions adequate to map guide prose without invention. The job ability slot accepts numeric ability definition IDs; the guide's ability names, locale IDs, and effects do not establish such IDs, so the arrays are empty. These omissions are recorded as `(R)` source evidence in the affected records. Quest display names remain in each record's evidence string.

Six quest `title_string_id` fields remain null with a reason in the record evidence: Adventurer "Another Day, Another Coin" (title not found), Medic "A Strong Supplement" (no title ID found), Miner "Speed Smelting" (no live title ID), and three Pet Trainer level rows (wiki-only titles). Adventurer C2/C3 repeated title IDs, Adventurer Royally Thirsty Mushrooms, both bare Card Duelist titles, and Postman "Can You Deliver?" are populated from explicit guide ID cells; the Postman field uses its documented UGDT-only ID. Fisherman locale IDs written without `ucdt`/`ugdt` prefixes were retained as documented. Miner bold "members only" annotations were excluded as quest names, and the Wizard page's Advanced Wizard board advertisement was excluded from quests.

The source package is per-job `quests/<job>.json` as requested; the unchanged loader accepts one root `quests.json` and requires all eight root datasets. The test aggregates the 17 job files into a temporary root package, uses existing fixture zones and empty unrelated datasets, and creates temporary `(R)` NPC rows for the unresolved refs so `ContentDb::load` can validate the real jobs/quests and cross-references. It does not write synthetic NPC rows back into the content batch. This adapter and the explicit unresolved list are test accommodations, not new schema behavior.

## Validation

- `python -I %USERPROFILE%\Documents\Projects\CartoonMMO\lab\content\tools\cq.py loc 'Path of the Archer'` — returned the client title IDs and objective noted above.
- `python -I %USERPROFILE%\Documents\Projects\CartoonMMO\lab\content\tools\cq.py wiki Archer` — returned archived wiki page, revision 2022-03-20.
- `python -I %USERPROFILE%\Documents\Projects\CartoonMMO\lab\content\tools\cq.py vid-quest 'Path of the Archer'` — no matching video result.
- `$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W22'; ../../scripts/cargo.ps1 test --test content_jobs` from `prototype/osfr-tools` — **passed** (1 test, 0 failures); pre-existing `housing.rs::instance_index` dead-code warning.
- `%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W22\debug\deps\content_jobs-996a88ac74f6373e.exe --nocapture` - passed and printed `unresolved NPC references explicitly listed: 465`.
- `rustfmt --config skip_children=true tests/content_jobs.rs` using the private toolchain `rustfmt.exe` — completed; only the owned test file was formatted.

## Artifacts

- `prototype/osfr-tools/data/content/jobs.json`
- `prototype/osfr-tools/data/content/quests/*.json` (17 files)
- `prototype/osfr-tools/data/content/npcs_unresolved.json`
- `prototype/osfr-tools/tests/content_jobs.rs`
