# 072 Starting area historical coverage

2026-10-09 UTC (2026-10-08 Pacific). MMO work queue item 072: compare the working
17-stage local opening chain with the original tutorials/quests, identify the
evidenced omissions, and restore the one that is implementable from recovered
evidence. **Historical starting-area completeness remains unaccepted.** This note
records original facts, inferred meaning, reconstructed policy and the native
gate separately.

Machine-readable run evidence: `../lab/reports/piece-072-20261009T045430Z/`
(`coverage.json`, `omissions.json`, `provenance.json`, `check.txt`,
`red-rust.txt`, `red-python.txt`; the two `red-*` files retain the test-first
failure before either artifact existed).

## Evidence classes

| Class | Source | Gives | Cannot give |
| --- | --- | --- | --- |
| Original (client Lua) | `decompiled/client-ui/ScriptsBase.lua`, SHA256 `c7c71c7c…a8f6` | The journal-close objective emit path, the `FirstTimeEvent` directed-play callbacks, their prompt code strings and sound definition ids | Server packet ids, trigger volumes, ordering |
| Original (client resource) | `working/OSFR.Client/OSFR Client/Resources/CodeStringMappings.txt`, SHA256 `1da9690f…3e5a` | `code string -> locale id`: `FtesNpxBookExit^5100079^`, `FtesNpxDppCamera{1,2,3}` -> 10258/10259/10260, `FtesNpxDppMovement{1,2,3}` -> 10261/10262/10263 | Any runtime behaviour; it is a name table |
| Original (locale) | `reports/locale-id-to-text.json`, SHA256 `f1f22a30…2844` | The exact player-facing prompt text for those ids | Which prompt the server fires, or when |
| Original (native readers) | [[Original Opening Microtutorial Recovery]] (45/10, 45/7, 45/8, 45/9; 45/6 LookAt; 45/11 FirstMovement) | Message shapes and directions | Per-objective values, globe/barrier bindings |
| Local reconstruction | `content/opening.json`, SHA256 `4e6ac15c…0ad1` | 17 reserved `910xxx` stages, 51 objectives, 27 actors, 4 discoveries | Any original binding: 0 of 17 stages carry `original_quest_id` |

## Verified original chain

The journal-close gate is a real client objective, not a guess. `StoryBook:Hide`
at `ScriptsBase.lua:14689` hides the book and restores movement and keyboard
(`Ui.SetMovementEnabled(1)`, `Ui.SetKeyboardEnabled(1)`); only inside
`Ui.IsInNPXTutorial()` (line 14716) does it kill the four `FtesNpxBookTab/
Sticker/Quest/Open` timers (14718-14721) and emit
`Ui.OnObjectiveEvent("onStoryBookClose")` (14722). The event string occurs
exactly once in the whole file, so a stray match cannot satisfy the checker.

Each recovered prompt verifies as a three-file chain: the `FirstTimeEvent`
callback reads its own code string (`FtesNpxDppCamera1` at 40598 … 
`FtesNpxDppMovement3` at 40663, `FtesNpxBookExit` at 40819), the resource table
maps that code string to a locale id (lines 1438, 1444-1446, 1451-1453), and the
locale export carries the text. Camera prompts play sound definitions
18264/18265/18266, movement prompts 18267/18268/18269, and `FtesNpxBookExit`
plays no sound and sets `StoryBook.shouldAllowHide = true`. The 19-value chain is
re-derived by `scripts/pieces/piece_072.py`, not copied from prose.

## Omissions found

The bound 17-stage chain has no client objective at all: `objectives` use the
trigger kinds `class_selected`, `cosmetic_equipped`, `craft`, `defeat`,
`dungeon_completed`, `gather`, `interact`, `talk` and `visit`, and **zero**
`ui_event`/`objective_event` triggers. Book Close therefore cannot advance any
local stage, and none of the seven recovered prompts is bound. `omissions.json`
records four omissions with their smallest demonstrable implementation and the
exact missing evidence: the unbound journal-close objective, the unrecovered
camera globe bindings, the unrecovered movement barrier bindings and the
unrecovered directed-play server ordering. Broader quest/sticker gaps stay owned
by [[Original Quest Coverage]] and [[Opening Historical Coverage Ledger]]; this
piece does not re-derive them.

## Restored here

`prototype/osfr-tools/src/restoration/piece_072.rs` implements the recovered
client-objective contract for the journal-close gate:

* ORIGINAL: `45/10` registration framing
  (`u16(45), u8(10), i32 objectiveID, i32 string byte length, string bytes`),
  strict `45/7` client-completion acceptance (exactly 7 bytes, opcode and message
  type checked, positive id, no trimming), the `onStoryBookClose` event name, the
  four timers and locale 5100079, and the seven recovered prompt rows as
  `ORIGINAL_TUTORIAL_GATES` (callback, locale id, sound id, input hint, binding
  status).
* RECONSTRUCTED, labelled: the session-scoped registry, the zone/phase guard,
  the once-only completion ledger, the event-name length bound, and the objective
  id itself (`STORY_BOOK_CLOSE_OBJECTIVE_ID = None`).
* NOT IMPLEMENTED: the `45/8` and `45/9` byte layouts (only their state effect -
  clear, and retry-after-failure - is modelled), the camera globe and barrier
  triggers, any directed-play ordering, and every reward. The module grants no
  items, coins, XP or quest progress.

## Gates

Integration (lead-owned): register the module in `lib.rs`, register the
objective in a fresh private tutorial campaign branch, and bind the event to the
reconstructed cave stage. Native acceptance is **pending** and cannot be claimed
from this artefact: it needs the real client to open the original journal, show
the close instruction/arrow, emit a real `45/7` only on normal Close while NPX
tutorial mode is active, refuse hidden/disallowed/outside-mode closes, restore
controls, and preserve the active 17-stage save. No packet injection, memory
modification or script replacement belongs to that gate. The original objective
id is unrecovered, so any id used in the campaign is an explicit reconstruction.

## Commands

```powershell
# project root
& '..\lab\tools\wiki-env\Scripts\python.exe' -B -m unittest discover -s tests/pieces -p "test_piece_072.py" -v
& '..\lab\tools\wiki-env\Scripts\python.exe' -B scripts\pieces\piece_072.py --reports-dir ..\lab\reports\piece-072-20261009T045430Z --stamp 20261009T045430Z
# prototype/osfr-tools
& ..\..\scripts\cargo.ps1 test --lib --test piece_072 --test piece_043 --test opening --test opening_journal --test opening_world
```

Results: python 19 passed; checker exit 0 with `gates=7`, `gates_uncovered=7`,
`rust_mismatches=0`; scoped cargo targets green (lib 24, piece_072 12, piece_043
13, `opening` 7+2 ignored, `opening_journal` 6+2 ignored, `opening_world` 1).
A plain `cargo test` currently stops on the concurrently edited
`tests/piece_047.rs` (missing `PartialEq` for `DecodedItemStat`), which belongs
to another worker; no file outside this item's paths was touched.

## Skills applied and limits

RTK ran commands normally. Ponytail reused the preserved Lua/resource/locale
files and the existing 066/ledger work instead of new extractors or
dependencies; the checker is one stdlib script plus one test file. Caveman kept
reporting concise while these notes stay complete. game-research-websearch
principles: community/pinned sources, no new scraping or network calls.
reverse-engineering: read-only inspection of the recovered Lua and native
reader notes; no new binary analysis. game-recon and asset-pipeline did not
apply (engine and formats are already established; this piece produces no
assets). game-automation did not apply: the lead owns the shared native window.

Limits: the seven prompts are recovered text/audio, not a recovered server
sequence; the wiki and community journal remain separate evidence classes; the
`45/7` packet carries no event string, so a completion can never prove which UI
action produced it; and no native tutorial-mode entry/exit path is established.
