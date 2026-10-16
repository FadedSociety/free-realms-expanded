# 066 Historical quest catalogue

2026-10-09 UTC (2026-10-08 Pacific). MMO work queue item 066. Acceptance:
recover quest/objective/prerequisite/NPC/reward sources; missing original
bindings remain explicit. **Historical starting-area completeness remains
unaccepted.**

This piece turns the existing catalogue evidence into two owned, runnable
checkers and one typed reader, and it re-derives the acceptance fields from the
pinned raw sources instead of trusting the catalogue's own totals.

## Deliverables

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_066.rs` | Typed reader/validator for the catalogue: source-scoped accessors (objectives, rewards, prerequisites, start NPCs), integrity errors vs documented warnings, `original_quest_id` boundary enforcement |
| `prototype/osfr-tools/tests/piece_066.rs` | 11 tests against the real catalogue plus synthetic duplicate/fabricated-binding rejection |
| `scripts/pieces/piece_066.py` | Stdlib-only re-derivation checker: reads the pinned raw sources, compares the acceptance fields to `notes/Research/original-quest-coverage.json`, writes a lab report |
| `tests/pieces/test_piece_066.py` | 11 unittest cases (real corpus + synthetic mismatches) |
| `../lab/reports/piece-066-20261009T043320Z/` | `report.json`, `verify.txt`, `check.txt`, RED/GREEN logs, `commands.log` |

Machine catalogue (owned by an earlier 066 pass, read-only here):
`notes/Research/original-quest-coverage.json`.

## Recovered sources (original vs community vs reconstruction)

| Class | Source (hash pinned in `report.json`) | What it gives | What it cannot give |
| --- | --- | --- | --- |
| Original (supplied) | `content/quest-inventory.json`, the 25-record slice of the recorded `PacketSendSelfToClient.json` | numeric ids, title/description locale ids, 42 objectives with required counts, reward bundle item references | a complete or consistent database: the records are canned character data |
| Original (engine, bounded) | `content/quests.json` | 2 local definitions, enemy bindings, coins, item references | any original quest beyond those ids |
| Community reconstruction | `FreeRealms-Legacy` `StartingZone.cs` (AGPL-3.0, rev `9e082025391177d5a33ee18c714ddd06b457a695`) | 10 hubs, 58 hub registrations (13 in region 1), 16 stickers (6 region 1) | titles/objectives/prerequisites/rewards for hub registrations |
| Wiki (2013-era) | `quest-pages.json` + read-only `wiki.sqlite` (52 `Quest:` pages, revision pinned) | prerequisites, start NPC, goals, coin amounts for the two `Farnum's Farm` opening pages | exact 2014 server behavior |
| Local reconstruction | `content/opening.json` | 17 reserved `910xxx` stages, 51 objectives | any original numeric binding: every `original_quest_id` is null |

## Verified acceptance

Re-derived by `scripts/pieces/piece_066.py` and asserted by both test suites:

- Supplied journal: 25 records, 42 objectives, reward item references
  (1617 → objective 14096 "Defeat Petty Yetis", required 20, item 1843 ×1).
- Opening chain: hub registrations `2513, 2514, 2521-2530, 2566` (13, region 1),
  region-1 stickers `2563-2565, 2572, 2573, 2587` (6, titles from locale).
- Wiki opening pages: `Quest:Your First Weapon` rev 27815, prerequisite
  `Quest:Learn to Mine & Smelt Ore`, NPC Cleetus Farnum, coin 16; `Quest:Fighting
  off the Pack` rev 28955, prerequisite `Quest:Your First Weapon`, coin 18.
- Local binding boundary: **0 of 17** local stages carry an original binding; 12
  native-accepted / 5 code-only are a dated snapshot claim, not re-derivable.
- Blocked claim: quest 1617 stays `disabled`; engine `enemy_definition = 0`, so
  unrelated kills cannot advance it.
- Corpus totals and the wiki sqlite page count re-derive exactly (0 errors).

## Defect found by the checker

The catalogue stores truncated wiki prerequisite tokens: `Quest:Learn` and
`Quest:Your`, produced by a whitespace-ending token match. The raw wiki text
contains the full titles (`Quest:Learn to Mine & Smelt Ore`,
`Quest:Your First Weapon`), and the same pages' `previous` fields already record
them. The checker reports this as the single `wiki-opening-chain` mismatch on
every run, and the Rust validator emits `unresolved wiki prerequisite` warnings
for both tokens. Repair belongs to the catalogue owner: the file is outside
item 066's exclusive paths, so it was not edited here.

## Original vs reconstruction boundary

Original numeric quest records, wiki text and community registration ids are
recovered facts. The 17 `910xxx` stages, their counts, rewards and actor
bindings are local reconstruction. Missing originals stay listed (13 hub
records with no title/objective/prerequisite/reward, 2 wiki chain pages absent
from the corpus, 3 region-1 stickers absent from the local chain, a verified
Petty Yeti enemy definition, a complete item corpus for reward references).
Nothing in this piece grants money or items.

## Commands and results

```powershell
$py = "$lab\tools\universal-modder-env\Scripts\python.exe"
# RED (module absent): exit 5, FileNotFoundError scripts\pieces\piece_066.py
& $py -B -m unittest discover -s tests/pieces -p "test_piece_066.py" -v
# RED (module absent): exit 101, couldn't find file tests\..\src\restoration\piece_066.rs
#   (cwd: prototype\osfr-tools)
& powershell -NoProfile -ExecutionPolicy Bypass -File scripts\cargo.ps1 test --test piece_066
# GREEN: exit 0, 11 passed; 0 failed
& powershell -NoProfile -ExecutionPolicy Bypass -File scripts\cargo.ps1 test --test piece_066
# GREEN: exit 0, Ran 11 tests, OK
& $py -B -m unittest discover -s tests/pieces -p "test_piece_066.py" -v
# Evidence: exit 0, errors 0, defects 1, checks 6
& $py -B scripts\pieces\piece_066.py --output "$lab\reports\piece-066-20261009T043320Z"
```

Logs: `../lab/reports/piece-066-20261009T043320Z/`.

## Skills applied

RTK: commands run normally, key output retained under `../lab`. Ponytail: no new
dependency, one stdlib script plus one typed reader; the existing catalogue is
reused instead of re-generating it. Caveman: concise reporting. game-research-websearch:
community-first sourcing, revision pins, no new scraping. reverse-engineering:
read-only inspection of the pinned C# literal tables and wiki text; no new
binary analysis. game-recon, asset-pipeline and game-automation were not
applicable (engine and formats already established; no assets; the lead owns
the shared native window).

## Remaining gates

1. Lead integration: the module is not registered in `src/lib.rs`; `tests/piece_066.rs`
   uses a `#[path]` include until the lead registers it.
2. Catalogue repair: full wiki prerequisite titles (defect above), outside this
   item's paths.
3. Native: **0 original quests** have native acceptance; the 12/17 local stage
   figure is a 2026-10-08 snapshot. No claim of native historical fidelity.
4. Crate-wide `cargo check --tests` currently fails on other workers' in-flight
   targets (`piece_044`, `piece_062`, `piece_065`, `piece_068`, `piece_092`
   reference modules that do not exist yet). The `piece_066` target itself
   compiles and passes; log: `rust-check-tests.txt`.
