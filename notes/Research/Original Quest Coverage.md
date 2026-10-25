# Original quest coverage

2026-10-09 UTC (2026-10-08 Pacific). MMO work queue item 066: bounded catalogue
of original quest evidence for the selected supplied 2014 opening arc
(Briarwood Caverns -> Farnum Farmstead -> Cobblestone Village -> Royal Palace),
compared against the 17 reconstructed local stages. **Historical starting-area
completeness remains unaccepted.** This note records source evidence; it does
not promote reconstructed content to original content.

Machine-readable catalogue: [[original-quest-coverage.json]] (`notes/Research/original-quest-coverage.json`).
Run evidence: `../lab/reports/quest-catalogue-20261009T0223Z/` (`audit.json`,
`check.txt`, `tests.txt`, `red.txt`, `commands.log`; `red.txt` is the retained
test-first failure before the audit script existed).

## Evidence classes

| Class | Source | Gives | Cannot give |
| --- | --- | --- | --- |
| Original (supplied) | `content/quest-inventory.json` sliced from the supplied `PacketSendSelfToClient.json` | 25 original numeric quest records with locale title/description ids, objectives, required counts and reward item references | A complete or internally consistent database: the set is canned character data, and 1029/2270 already show mismatched text |
| Original (locale) | `lab/reports/locale-id-to-text.json`, 85,932 named ids | Title, description, objective, hub, sticker and NPC text for referenced ids | Binding of text to server behavior |
| Community reconstruction | `FreeRealms-Legacy` `StartingZone.cs` (AGPL-3.0, rev `9e082025391177d5a33ee18c714ddd06b457a695`) | 10 hub records, 58 hub quest registrations, 16 stickers with locale ids | Titles/objectives/rewards for hub registrations; runtime rules |
| Wiki-only | `freerealms-wiki` read-only corpus, 52 `Quest:` pages with revision pins | 2013-era prerequisites, start NPC, goals and coin amounts for two opening pages | Exact 2014 server behavior; the corpus spans 2013-2026 revisions |
| Local reconstruction | `content/opening.json` | 17 reserved `910xxx` stages, 51 objectives, 27 actors, 4 discoveries | Any original numeric binding: every `original_quest_id` is null |

## Corpus totals

`supplied-journal` 25 records / 42 objectives; `community-journal` 10 hubs /
58 hub registrations (13 in the three opening hubs) / 16 stickers (6 region 1);
`wiki` 52 quest pages, 2 in the opening series (`Farnum's Farm`);
`local-opening` 17 stages / 51 objectives; `locale` 85,932 entries;
`engine-definitions` 2 local definitions.

## Recovered original records

Supplied journal ids and titles (original ids, canned data):

```
6 Stolen Turnips | 92 History Comes Alive! | 218 Flag Dancin' | 435 Noise Permit
439 Snowhill Secret Society | 462 Grubby Hands | 624 One Man's Trash
1019 Blacksmith: Smithy Merchandise | 1029 Smoothie Operator | 1077 No More Norv?
1304 Card Duelist: Poe's Best Deck | 1346 Free the Balloons?!
1593 Contract: Hello Fernando | 1617 Harassing the Harassers | 1625 Something to Snack On
1648 Burial Service | 1673 Smelly Stockpile | 1680 Mushroom Guard
1875 Card Duelist: Esther, the Brains | 2061 Trick-or-Treat | 2248 Feeding the Crowd
2270 Defensive Strategy | 2287 The Great Scare | 2347 Welcome Home | 2513 Hero's Journal
```

Card Duelist: supplied records 1304 and 1875 are bound to the content quests 3040011 and 3040034 (objectives, rewards, Dominic as giver of 1875); both stay disabled until a card duel can produce a result.

Thirteen journal hub registrations cover the three opening hubs:
2513, 2514 (Briarwood Caverns); 2521-2526 (Farnum Farmstead);
2527-2530, 2566 (Cobblestone Village). None carries a recovered title,
objective, prerequisite or reward record. A prior extraction
(`starting-area-audit.json`) lists the same 13 ids; the audit re-derives them
from `StartingZone.cs` and cross-checks the two lists (match).

Region-1 stickers with recovered titles (locale): 2563 Second Helping of
Strongarm Stew, 2564 Cleetus' Copper Collection, 2565 Forging for Reinforcements,
2572 Fighting for Funds, 2573 Margin of Victory, 2587 Spreading the Cheer.
Regions 2-4 add 10 more stickers (2568-2575, 2602, 2605, 2606, 2592).
Sticker 2592 has `NameId = 0`: its title is unrecovered, not empty.

Wiki opening pages (revision-pinned):

| Page | Revision | Prerequisite | Next | Start NPC | Goals | Coin |
| --- | --- | --- | --- | --- | --- | --- |
| Quest:Your First Weapon | 27815 (2013-07-12) | Learn to Mine & Smelt Ore (page absent) | Fighting off the Pack | Cleetus Farnum, Farnum's Farm | Forge a Funky Scrapper Hammer, return | 16 |
| Quest:Fighting off the Pack | 28955 (2013-07-21) | Your First Weapon | Eviction Notice (page absent) | Cleetus Farnum, Farnum's Farm | Switch to Brawler, knock out 5 Hooligan Wolves, return | 18 |

The wiki's next quest after Fighting off the Pack is **Eviction Notice**, while
the supplied locale recovers **One More Gloam** as the following local stage.
That version/identity gap stays open; the two are not merged.

## Comparison with the local 17 stages

Original bindings: **0 of 17**. Local stages keep `original_quest_id: null`,
and the catalogue refuses name-similarity bindings. Text-level only:

- Local stage titles "Your First Weapon" (910006) and "Fighting off the Pack"
  (910007) match the two wiki page titles exactly, and the wiki prerequisites
  and five-wolf/18-coin detail agree with the local text. This is
  corroboration of the story step, not proof that the local stage is the
  original quest record.
- Local stages 910013-910015 ("Second Helping of Strongarm Stew",
  "Cleetus' Copper Collection", "Forging for Reinforcements") match region-1
  sticker titles 1-3 exactly. Stickers 4-6 are absent from the chain.
- Supplied record 2513 "Hero's Journal" shares a title with local stage 910001
  and id 2513 also appears in the hub-1 registration list. The shared id is
  reported, not silently merged.
- Local stages 910001-910003, 910008-910012 and 910016-910017 have no direct
  wiki or supplied-record title match; their objectives remain reconstructed.

Implementation states stay distinct (2026-10-08 snapshot, `Current Progress`):
12 local stages native-accepted (`910001`-`910012`), 5 code-only
(`910013`-`910017`); **0 original quests** have any native acceptance.
The original cave encounter (411, `sg_npx_cavern_01`) has 6 unrecovered rules
and no complete-dungeon acceptance; the original micro-tutorial actions are
locale text only.

## Disabled claim: Petty Yeti 1617

Supplied record 1617 is **Harassing the Harassers** (title 389427, description
389428): objective 14096 "Defeat Petty Yetis", required 20, reward item 1843
quantity 1. The local engine definition for 1617 keeps `enemy_definition = 0`,
so generic Robgoblins (1294), Hooligan Wolves (3156), the local Gloam construct
(920001) or local Mac (920002) cannot advance it, and no verified Petty Yeti
enemy definition exists in the pinned resources. The catalogue carries an
explicit `blocked_claims` entry with status `disabled`; any report that quest
1617 is covered by unrelated kills is rejected by the audit.

## Missing originals (explicit)

13 opening hub quest records with no recovered title/objective/prerequisite/
reward; wiki chain pages Quest:Learn to Mine & Smelt Ore and Quest:Eviction
Notice absent from the corpus; region-1 stickers 2572 Fighting for Funds,
2573 Margin of Victory and 2587 Spreading the Cheer absent from the local
chain; original micro-tutorial gates, cave scripting, gathering games and
optional farm dungeons (Sheep Watch, Highroad Hijinx) as recorded in
[[Opening Historical Coverage Ledger]]; a verified Petty Yeti enemy definition;
and a complete item-definition corpus for reward references.

## Integrity checks

Duplicate ids: none within any corpus (25 supplied records, 16 stickers, 58 hub
registrations, 52 wiki pages, 17 stages). Dangling references: none, all 25
supplied record title/description/objective locale ids, all sticker locale ids,
all 17 stage locale ids, all stage actor GUIDs and all unlock profiles resolve.
Cross-checks: `quest-pages.json` equals the read-only `wiki.sqlite` page list
(52/52) and the re-derived opening hub ids equal the prior extraction (13/13).
Audit result: **0 errors**, 20 documented warnings only (13 missing hub
records, 2 missing wiki chain pages, 3 missing stickers, 1 disabled claim,
1 shared id).

## Commands

```powershell
$lab = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab'
$py = "$lab\tools\universal-modder-env\Scripts\python.exe"
$dir = "$lab\reports\quest-catalogue-20261009T0223Z"
& $py -B scripts\audit-quest-coverage.py --check --output "$dir\audit.json"
& $py -B -m unittest discover -s tests -p "test_quest_coverage.py" -v
```

Result: 10 tests pass; audit exits 0 with 0 errors. Sources are referenced by
SHA-256 inside `audit.json`; key hashes: `quest-inventory.json`
`17a94fe8ebbe49a1fa0d08fceacdb6cdcb547e126a1636b80d3f4f4166c7fc54`,
`opening.json` `4e6ac15cd19b5785a04ac7eb944bf0235f4a0592f4c42df755f89c47896b0ad1`,
`StartingZone.cs` `0e26fd20ad103e6d78461e150dbbcc7e80c175b1061cab7ee3785c14053cae37`,
`quest-pages.json` `db4266bed954da6aab394cb514de2c54da1181966cd393fc122cdaae90c66c05`,
`wiki.sqlite` `b9d7b9f1adabb42db54c3253bf43fddc249d6d70e3f2075b134a9441e2fcd911`,
`locale-id-to-text.json` `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844`.

## Skills applied and limits

RTK ran commands normally (host condenses output). Ponytail kept this to one
stdlib-only read-only script plus one test file, reusing existing pinned
sources and prior audits instead of new extractors or dependencies. Caveman
kept reporting concise. game-research-websearch principles: community-first,
pinned revisions and archive-friendly inputs, no new scraping or network calls.
reverse-engineering: read-only inspection of pinned community/recovered sources;
no new binary analysis (the journal byte oracle from prior work is referenced,
not re-derived). game-recon, asset-pipeline and game-automation were not
applicable: the engine and formats are already established, this audit produces
no assets, and the lead owns the shared native window.

Limits: supplied records are canned mixed-era data; hub registrations and most
stickers carry no objective/reward fields; wiki text spans 2013-2026; reward
item ids are references without a complete item corpus; native acceptance
states are a snapshot; the selected arc boundary excludes other surviving
introductions (Merigold/Highroad, Sacred Glade) that are reference-only here.
