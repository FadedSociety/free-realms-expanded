# 067: Quest prerequisite engine

2026-10-09 UTC (2026-10-08 Pacific). MMO work queue item 067. Acceptance:
source-defined requirements, branch availability and invalid acceptance
rejection. This note separates recovered original facts, inferred meaning,
reconstructed policy and native acceptance. **No native run is claimed.**

## Artifacts

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_067.rs` | Prerequisite engine: requirement kinds, validated definition table, availability and accept gate |
| `prototype/osfr-tools/tests/piece_067.rs` | 11 focused tests (owned `#[path]` include; module not yet registered) |
| `scripts/pieces/piece_067.py` | Corpus checker: re-derives every requirement row from the pinned pages and cross-checks the Rust constants |
| `tests/pieces/test_piece_067.py` | 21 tests over the real corpus plus rejection fixtures |
| `../lab/reports/piece-067-20261009T050538Z/` | `prerequisites.json`, `provenance.json`, `check.txt`, RED/GREEN logs, `run.txt` |

## Evidence classes

| Class | Source | Gives | Cannot give |
| --- | --- | --- | --- |
| Original (documented, revision-pinned) | `lab/references/freerealms-wiki/quest-pages.json`, 52 quest pages | Verbatim requirement sentences, `Questprog` previous/next/series chain, coin amounts, page revision ids | Server behaviour, numeric quest ids, exclusions |
| Original (first-party client) | `lab/decompiled/client-ui/ScriptsBase.lua` (`QuestData_DS.JobLevel = 7`, `.MembersOnly = 9`); locale 401549 "In order to accept more quests, you must first become a member."; locale 71311 "Sorry, you don't meet the requirements for that action!" | That an accept-time requirement gate with membership and job-level inputs and a rejection message exists | The requirement table itself |
| Original (supplied journal) | `content/quest-inventory.json` (25 records) | Quest/objective/reward rows for the canned character | Any prerequisite field: the journal carries none |
| Community reconstruction | FreeRealms-Legacy `Sanctuary.Game/Resources/Definitions/QuestDefinition.cs` (AGPL-3.0) | `PrerequisiteQuestId` / `ExcludesQuestIds`, "exclusions are checked before the prerequisite" | First-party exclusion lists or ids |
| Local reconstruction | `content/opening.json` | 17 ordered stages | Explicit prerequisite fields (ordering is implicit at stage advance) |

## Recovered requirements (original, verbatim)

The corpus checker reads the `Requirements` section of each page and classifies
all 36 non-empty sections; nothing is skipped silently. Counts:

| Kind | Rows | Verbatim example |
| --- | --- | --- |
| Quest completion | 29 | "Player must complete Quest:Your First Weapon in order to do Quest:Fighting off the Pack" (rev 28955) |
| Membership | 2 | "Player must complete Quest:Horrible Hiccups and have an active Membership in order to do Quest:Scaredy Pants." (rev 28848) |
| Glide-trainable ride | 5 | "Player must own a usable glide-trainable ride to complete this quest." (rev 27755) |
| One-day completion cooldown | 4 | "Player must own a usable glide-trainable ride along with waiting at least one day after completion of Quest:Glide Training: Getting Started in order to do Quest:Glide Training: Basic Training" (rev 27858) |
| Combat job level | 1 | "Player must achieve at least one Level 8 or higher Combat Job" (rev 23970) |
| Event window | 4 | "Quest:Winter Party is only available during Winter Party Community Events." (rev 35916) |

Opening-arc rows (the two pages the restored arc names, no numeric ids recovered):

| Page | Revision | Prerequisite | Next |
| --- | --- | --- | --- |
| Quest:Your First Weapon | 27815 (2013-07-12) | Quest:Learn to Mine & Smelt Ore (page absent) | Fighting off the Pack |
| Quest:Fighting off the Pack | 28955 (2013-07-21) | Quest:Your First Weapon | Eviction Notice |

Cross-checks the checker enforces on every run: the sentence's own clause names
the page (`self-clauses match: True`, 29/29), a prerequisite equal to the page's
`previous` field, the prerequisite page's `next` field pointing back
(`chain consistent: True`), no duplicate pages, and the pinned SHA-256 of the
corpus. Six prerequisite pages are absent from the 52-page corpus and are
reported as explicit gaps, never dropped: Quest:A Brewing Darkness,
Quest:Committed to the Plan, Quest:Gathering Evidence, Quest:It's Too Quiet...,
Quest:Learn to Mine & Smelt Ore, Quest:The Search for Golmie Greasebeard.

## Reconstructed policy (labelled, not original)

* **Branch availability.** `ExcludesQuestIds` and the ordering rule come from the
  pinned community `QuestDefinition`; the comment there names the starter-job
  choice as the rationale. No first-party exclusion list was recovered, so the
  engine implements the rule and the tests exercise it with synthetic ids.
* **Gate order.** `accept` rejects in this order: already active/completed,
  branch taken, prerequisite not completed, then the remaining requirements in
  source order (membership, job level, ride, cooldown, event window). Availability
  reports every unmet condition in the same order instead of one at a time.
* **Definition validation.** A prerequisite must exist in the table or in the
  caller's explicit `external_prerequisites` list; zero-level job requirements,
  zero-day cooldowns and empty event names are rejected. Self-prerequisites and
  self-exclusions are rejected. Longer cycles are not scanned: they keep both
  quests blocked forever, which is a visible data bug rather than an unsafe
  acceptance.
* **Cooldown arithmetic.** "waiting at least one day after completion" is
  evaluated from the recorded completion timestamp and the caller's clock; the
  unit is milliseconds per day as in the rest of the local engine.
* **Award-free.** The engine never grants coins, items or XP and never reads
  reward fields. Reward transactions stay with items 066/069.

## Cross-check against item 066

`notes/Research/original-quest-coverage.json` stores the two opening wiki
prerequisites as `["Quest:Your"]` and `["Quest:Learn"]` — a word-split artifact
of its extractor. The page text says `Quest:Your First Weapon` and `Quest:Learn
to Mine & Smelt Ore`. This item's catalogue carries the full titles; the
truncated 066 values should not be used as prerequisite ids. (066 is another
worker's path and was not edited here.)

## Implementation and acceptance status

* Implemented acceptance: source-defined requirements as typed rows and validated
  tables; branch availability with the recovered check order; rejection of
  unknown quests, unmet requirements, taken branches, re-accept and re-complete
  without mutating player state.
* Original acceptance gate (native): **not exercised**. The engine is a library;
  wiring it into `quests.rs`/`opening.rs` is the lead's integration step, and no
  native accept/turn-in was run by this worker.
* Explicit gaps: numeric quest ids for the two wiki pages; the six missing
  prerequisite pages; a first-party exclusion table; the full historical quest
  requirement corpus (52 pages against the original quest count).

Integration seam for the lead: build `QuestRow<u32>` values from the local
quest definitions (or from `prerequisites.json` once numeric ids are bound),
call `PrerequisiteTable::new(rows, external)` once at load, then ask
`availability` before offering a quest and `accept` before flipping
`QuestProgress.accepted` in `quests.rs`. External ids are the ids whose
`original_quest_id` is still unknown: recording their completion is allowed,
inventing their definition is not.

## Commands (actual)

```powershell
$py = '..\lab\tools\universal-modder-env\Scripts\python.exe'
& $py -B -m unittest discover -s tests/pieces -p "test_piece_067.py" -v
& $py -B scripts\pieces\piece_067.py --reports-dir '..\lab\reports\piece-067-20261009T050538Z' --stamp 20261009T050538Z --check
cd prototype\osfr-tools
& ..\..\scripts\cargo.ps1 test --test piece_067
```

Results: Python 21 tests pass; checker exits 0 with `errors: 0` (52 pages, 36
requirement rows, 29 prerequisite rows, 6 unresolved); Rust 11 tests pass.
RED was established first and is retained: missing `piece_067.py`
(`red-python.txt`) and missing `piece_067.rs` (`red-rust.txt`).

## Skills applied and limits

RTK ran the commands normally. Ponytail kept the engine to stdlib Rust (`BTreeMap`
plus one enum per concept), reused the existing pinned corpus and locale export,
and added no dependency; the Python checker is stdlib-only. Caveman kept the
report concise while the persisted note stays complete. reverse-engineering read
the pinned community definition and the existing native extracts instead of new
binary analysis; game-research-websearch principles were applied by reusing the
already-crawled revision-pinned corpus and citing its snapshot ids, with no new
network calls. game-recon, asset-pipeline and game-automation were not
applicable: the engine and formats are established, no assets are produced and
the lead owns the shared native window.

Limits: the wiki is community documentation of the original game and spans
2013-2026 revisions; requirements describe availability, not the server's exact
implementation; the corpus is a small sample of the historical quest set and
skews to the Sunstone Valley, Pony Vale, Glide Training and holiday chains.
