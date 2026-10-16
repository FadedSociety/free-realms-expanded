# 074 Collections and collectibles

## Scope

Recovered the collection category table, the client's collection row contract and the community
set/entry/area/spawn/reward catalogue, plus collection progress with exactly-once completion and
restart persistence behind it. The category table, the column contracts and the entry-view limit
are first-party client facts; the set names, entry names, areas, spawn types and rewards are
community wiki facts; the catalogue/ledger split and the durable document are reconstructed. This
piece grants no item, currency or experience, and it does not claim native acceptance.

## Evidence

| Source | Path (analysis lab) | Pinned by |
| --- | --- | --- |
| First-party category table | `working/OSFR.Client/OSFR Client/Resources/CollectionCategories.txt` | sha256 `c3a06887…fbd37`, 182 bytes, CRC32 `88c0f5c2`, from `manifests/OSFR.Client-zip-inventory.json` entry `OSFR Client/Resources/CollectionCategories.txt` |
| Decompiled client Lua | `decompiled/client-ui/ScriptsBase.lua` | sha256 recomputed by the checker |
| Decompiled client export | `decompiled/client/FreeRealms/pseudocode.c` | sha256 recomputed by the checker, markers scanned in chunks |
| Shipped locale export (85,932 rows) | `reports/locale-id-to-text.json` | sha256 recomputed by the checker |
| Read-only wiki corpus | `references/freerealms-wiki/wiki.sqlite` | per-page `revision_id`, `revision_timestamp` and `sha256` recorded per record |

Every hash and revision id above is recomputed into
`lab/reports/piece-074-20261009T041951Z/collections.json` by
`scripts/pieces/piece_074.py`; the run log is
`lab/reports/piece-074-20261009T041951Z/extractor-run.log` and the compact result is
`lab/reports/piece-074-20261009T041951Z/summary.txt`.

## Original facts

- `CollectionCategories.txt` is a three-column table, header `#ID^NAME_ID^ICON_ID^`, ten rows with
  ids 1..10 in file order: `1^93636^9535^`, `2^392292^1865^`, `3^401015^7220^`, `4^392293^5088^`,
  `5^392294^1433^`, `6^392295^5166^`, `7^392296^7536^`, `8^392297^9293^`, `9^390416^4566^`,
  `10^392298^1319^`.
- The shipped locale resolves those ten `NAME_ID`s to Exploration, Job Specific, Flora, Toys,
  Insects, Robgoblin Junk, Archaic, Aquatic, Quests, Minigames, in that order.
- `ScriptsBase.lua` defines the client data source `BaseClient.CollectionCategory` with three
  columns (`CollectionCategoryID` 0, `CollectionCategoryNAME` 1, `CollectionCategoryICON_ID` 2) and
  `BaseClient.CollectionData` with seventeen columns: `CollectionID` 0, `Name` 1, `Description` 2,
  `ImageID` 3, two unused columns 4 and 5, `EntryCount` 6, `ImageTintValue` 7,
  `CollectionCategoryID` 8, then exactly eight entry flags `EntryCollected1..8` at 9..16.
- The same script carries `DS_COLLECTIONS` 30, `DS_COLLECTION_ENTRIES` 31, `DS_COLLECTION_REWARD`
  32 and `DS_COLLECTION_REWARD_ENTRIES` 33, which index (1-based) the data sources
  `BaseClient.CollectionData`, `BaseClient.CollectionEntriesData`,
  `BaseClient.CollectionData.Reward` and `BaseClient.CollectionData.Reward.Entries`; the tree data
  source `DataSourceTrees[2]` is `BaseClient.CollectionCategory`.
- `pseudocode.c`: `FUN_009fce20` returns 3 for the category row width, `FUN_009fce30` names the
  three category columns, and `FUN_009fd050` buckets a category with `param_2 % 10`.
- The same export constructs `ClientUpdatePacketCollectionStart`,
  `ClientUpdatePacketCollectionAddEntry`, `ClientUpdatePacketCollectionRemoveEntry` and
  `ClientUpdatePacketCollectionRemove`, resolves `BaseClient.CollectionData.Reward`, and drives the
  `CollectionStarted` / `UpdateCollection` / `CollectionCompleted` notifications.

## Community facts

- The wiki `Collections` page lists ten categories whose order matches the first-party table. Its
  only label difference is category 6, which the wiki calls "Robgoblin Junkpile" after the spawn
  type page while the client table resolves to "Robgoblin Junk"; the difference is asserted, not
  normalised.
- `List of Collections` names 240 collections across Exploration, Job Specific, Flora, Toys,
  Insects, Robgoblin Junk, Archaic and Aquatic. The corpus has a per-set page only for Exploration,
  Job Specific and Toys (100 sets with entry lists); `Aquatic (Collections)` is prose only, and
  Quests and Minigames have no set list at all.
- Per-set records carry a type, quality, spawn type, area, reward lines and entry names. 44 entry
  notes are cascade links ("This is the reward for completing …") and every one resolves to a set
  in the parsed corpus.
- Every one of the 100 parsed sets has at most eight distinct entries, which matches the eight
  entry flags the client view exposes. The single set with nine bullets,
  `Blacksmith's Forge Adventures`, repeats `Forge Story pg.5`; the duplicate is recorded as a
  finding rather than silently deduplicated.
- One parsed set, `Ninja Dolls`, is absent from `List of Collections` although the Toys page defines
  it and `Human Dolls` cites it as the source of its `Elite Ninja Doll` entry. The exception is
  asserted.

## Reconstruction boundary

- `prototype/osfr-tools/src/restoration/piece_074.rs` owns the catalogue/ledger split, the durable
  document, every bound and every error. It carries the recovered category table verbatim and
  refuses a definition with an unknown category, no entries, a duplicate entry or a duplicate
  collection id.
- Entry identifiers are caller-supplied. No first-party collection id, entry id, node coordinate,
  radius, spawn table, reward item id or currency amount was recovered, so none is invented: each
  emitted set record lists those four fields as gaps instead of carrying a value.
- Completion is exactly once. The ledger records a completed collection when the last missing entry
  arrives and persists that marker, so replaying an entry (before or after a restart) returns
  `newly_collected = false` and no completion. The first revision of the module recomputed the
  completion on every call and dropped the marker from the document; that revision is the retained
  RED evidence.

## Checks

| Command | Result |
| --- | --- |
| `scripts/cargo.ps1 test --test piece_074` (first revision) | RED: `a replay must not re-emit the completion, or the reward is granted twice`; `restored.completed_count()` left 0, right 1 — `lab/reports/piece-074-20261009T041951Z/red-p1.log` |
| `scripts/cargo.ps1 test --test piece_074` (final) | GREEN: 7 passed, 0 failed — `lab/reports/piece-074-20261009T041951Z/green-final.log` |
| `python -m unittest discover -s tests/pieces -p "test_piece_074.py" -v` | GREEN: 20 passed — `lab/reports/piece-074-20261009T041951Z/python-tests.log` |
| `python scripts/pieces/piece_074.py` | 10 categories, 100 sets, 240 set-list names, 11 spawn types, 14/14 checks ok — `lab/reports/piece-074-20261009T041951Z/summary.txt` |

## Remaining gates

- Lead integration: register `piece_074` in `prototype/osfr-tools/src/lib.rs` and drop the `#[path]`
  include from `tests/piece_074.rs`. Nothing else in the shared crate needs to change.
- Interface: collection progress has no owner yet for the character save document. The ledger
  document is self-contained and bounded, but the lead decides where a character's bytes live.
- Native: the client constructs the collection update packets, but no server-side sender and no
  capture of a live collection row were recovered, so neither progress, completion nor the category
  list has been exercised in the native client. The community server's decoded row shape
  (`Index`, `Collected`, reward bundle) is a secondary source only.
- Missing evidence, exact: first-party collection/entry identifiers and their parent category
  mapping, the node or spawn placements with trigger radii, and the reward table behind
  `BaseClient.CollectionData.Reward`. Until one of those is recovered, the client's collection
  window cannot be populated from this catalogue.
