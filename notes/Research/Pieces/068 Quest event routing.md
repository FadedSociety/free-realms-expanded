# 068 Quest event routing

2026-10-09 UTC (2026-10-08 Pacific). MMO work queue item 068. **Acceptance:**
typed talk/visit/kill/gather/craft/game events; matching original identifiers
and no unrelated kill credit.

Status: implementation and focused tests pass. The module is **not** registered
in `src/lib.rs` and not wired into `gameplay.rs`; no native client ran for this
item. The lead owns integration and the native gate.

## What landed

| Artifact | Path |
| --- | --- |
| Router module | `prototype/osfr-tools/src/restoration/piece_068.rs` |
| Rust tests (RED/GREEN) | `prototype/osfr-tools/tests/piece_068.rs` |
| Evidence checker | `scripts/pieces/piece_068.py` |
| Checker tests | `tests/pieces/test_piece_068.py` |
| Private evidence | `../lab/reports/piece-068-20261009T0521Z/` (`route-table.json`, `check.txt`, `red-rust.txt`, `red-python.txt`, `rust-tests.txt`, `rust-cross-check.txt`, `python-tests.txt`, `full-suite.txt`, `related-suite.txt`, `commands.log`) |

The router turns one recovered objective trigger table into typed routing:
an incoming `QuestEvent` credits exactly the objectives whose recovered
identifier matches, and credits nothing when the identifier was never
recovered. `route()` returns `ObjectiveCredit { quest_id, objective_key,
required, credit }`; `objective_key` is the objective definition id, which is
the key the native client looks up for live journal updates (see
[[Native Journal Live Objective Keys]]).

## Evidence classes

| Class | Source | Gives | Cannot give |
| --- | --- | --- | --- |
| Original (locale) | `lab/reports/locale-id-to-text.json`, 85,932 rows | `5100200 Hooligan Wolf`, `5100532 Encounter 411 Darkthorne Teleport`, and every objective name text below | Binding of text to a server behaviour |
| Original (supplied record) | `content/quest-inventory.json` sliced from the supplied `PacketSendSelfToClient.json` | 25 quest records / 42 objectives with objective ids, locale ids and required counts | Target identifiers: no enemy/item/activity/NPC id is present, so none of these objectives is routable |
| Community (pinned) | `Sanctuary` rev `4f028dcc274f628b65ef665fd59b184f5f3d3e3e`, AGPL-3.0, `src/Resources/Npcs.json` | Row 3156 `Hooligan Wolf`, model 3284 | Runtime quest rules; Sanctuary ships no quest content |
| Reconstructed | `content/opening.json`, `content/quests.json`, `content/encounters.json` | The 17-stage `910xxx` chain with typed triggers, the local engine definitions, enemy/dungeon rows | Any original binding: every `original_quest_id` stays null |

## The route table

36 routes: 35 from the reconstructed opening chain plus the engine definition
`900001` kill goal.

| Typed kind | Routes | Content trigger | Identifier domain |
| --- | --- | --- | --- |
| talk | 10 | `talk` | `actor_guid` |
| interact | 8 | `interact` | `actor_guid` |
| visit | 1 | `visit` | `zone_id` + position + radius |
| kill | 4 (3 opening + `900001`) | `defeat` | `definition_id` |
| gather | 6 | `gather` | `resource_id` |
| craft | 6 | `craft` | `recipe_id` |
| game | 1 | `dungeon_completed` | `encounter_id` |

13 `class_selected` and 3 `cosmetic_equipped` objectives stay outside the typed
vocabulary and are reported as unsupported rather than silently merged.
`interact` is kept as its own kind because the recovered chain uses it for
object use (journal, well, fires, artifacts); merging it into `talk` would
credit two different objective kinds from one event.

## Identifier matching, and what is original

* kill `definition_id` 3156 resolves to the pinned NPC table row `Hooligan Wolf`
  model 3284 and to locale 5100200; the reconstructed encounter row spawns the
  same model, so this route matches a recovered enemy definition. Definition
  1294 (engine `900001`) also resolves in the pinned NPC table (name absent) and
  is the `combat.json` sandbox enemy.
* kill `definition_id` 920001 / 920002 are local reserved ids (>= 920000) with
  encounters.json rows; they are not original definitions.
* `game` encounter 411 is the cave story link locale 5100532 names. The zone id
  411 is a **local assignment** (recorded in `encounters.json` provenance), not
  a recovered zone.
* `talk` / `interact` actor GUIDs (5715144…) and the objective keys (910xxx),
  resource ids 1001-1003, recipe ids 2001-2003 and the visit `zone_id` 0 are
  local reserved values. The visit trigger's zone 0 is recorded as a gap: no
  original zone id is recovered for that position.
* Actor definitions behind those GUIDs include original NPC ids (Mae Farnum
  2538, Pappy Farnum 3211, Bartle 2087, Queen Valerian 31331 …), so the local
  acting set is built from recovered NPC data even though the GUIDs are local.

## No unrelated kill credit

The supplied originals name typed actions in their objective **text** —
`1593` "Defeat some Robgoblin Pondblasters/Amazing Asps/Killer Cobras",
`2287` "Defeat the Pumpkin Prince", `1625` "Cook up some Gummyfruit Gumbo",
`2248` "Gather honeytwig/bumbleberry", `92` "Beat 10 stages of a Briarwood
Defense game" — but carry **no target identifier**. The checker therefore holds
every one of the 25 records / 42 objectives as `routable: false` and refuses any
route that claims an original quest id or objective key.

Engine definition `1617` objective `14096` "Defeat Petty Yetis" (required 20,
reward item 1843 x1) is the explicit blocked example: no verified Petty Yeti
enemy definition exists in the pinned resources, so the route table contains no
row for it, `QuestEventRouter::new` refuses to build a route with
`definition_id` 0, and the Rust test drives Robgoblin 1294, wolf 3156, Gloam
920001, Mac 920002 and Gloamling 920100 plus an unknown id through the router
and asserts that none of them credits 14096.

## Router rules implemented

* Exact identifier equality per kind; `visit` also requires the same zone and a
  position inside the recovered radius.
* `gather` / `craft` credit `min(quantity, required)` (one event can complete an
  objective but never overshoot it); other kinds credit 1.
* A well-formed event with no matching route returns an empty list — no credit.
* A malformed event is refused with an error instead of being credited: zero
  GUID / definition / resource / recipe / encounter ids, zero quantity and
  non-finite positions.
* Table validation refuses an empty table, zero quest/objective/required values,
  identifiers `<= 0` (including a kill target of 0), duplicate objective keys,
  non-finite visit positions and radii outside `0.1..=1000`.
* `load_json` accepts the checker document (extra `sources` / `blocked` / `gaps`
  keys are ignored) and rejects a foreign `schema_version`.

## Commands and results

```powershell
$lab = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab'
$dir = "$lab\reports\piece-068-20261009T0521Z"
$py  = "$lab\tools\universal-modder-env\Scripts\python.exe"

# RED (before the module/script existed)
& scripts\cargo.ps1 test --test piece_068            # cannot find src\restoration\piece_068.rs
& $py -B -m unittest discover -s tests/pieces -p "test_piece_068.py" -v
#   FileNotFoundError: ...\scripts\pieces\piece_068.py

# GREEN
& $py -B scripts\pieces\piece_068.py --check --output $dir
& $py -B -m unittest discover -s tests/pieces -p "test_piece_068.py" -v
& scripts\cargo.ps1 -CargoArgs @('test','--test','piece_068')
$env:OSFR_PIECE_068_ROUTES = "$dir\route-table.json"
& scripts\cargo.ps1 -CargoArgs @('test','--test','piece_068','--','--include-ignored')
```

Results:

* Checker: 36 routes (35 opening + 1 engine), 0 errors, 6 documented warnings
  (1 blocked objective, 13 + 3 out-of-vocabulary objectives, 3 gaps).
* Python: 18 tests pass. Rust: 9 tests pass + 1 ignored by default; the ignored
  cross-check passes (`10 passed; 0 failed`) by comparing the embedded table
  with the checker's independent `route-table.json`.
* Focused regression run: `--lib` 24 passed, `opening` 7, `opening_journal` 6,
  `piece_066` 11, `piece_069` 9 — all pass. A full `cargo test` currently fails
  to compile another worker's `src/restoration/piece_097.rs` (two `E0308` type
  mismatches at lines 463/474); that file is outside this item's paths.

## Original vs reconstruction boundary

ORIGINAL: locale rows 5100200 / 5100532, community NPC row 3156 (pinned mirror),
the 25 supplied quest records with their objective ids/locale ids/required
counts, and the engine packet family that carries quest state (unchanged).

RECONSTRUCTED: the seven-kind typed vocabulary and its `defeat` → kill /
`dungeon_completed` → game renaming, the exact-match rule, quantity clamping,
the resource/recipe sentinels, and the entire 910xxx route table.

GAPS: no original objective target identifier is recovered for any supplied
record; the visit trigger keeps sentinel zone 0; encounter 411's zone id is a
local assignment; `class_selected` / `cosmetic_equipped` objectives have no
typed event here; original minigame/activity ids are not bound to any recovered
quest objective.

## Remaining integration and native gates

1. Lead integration: register the module and decide whether `gameplay.rs`
   replaces its hard-coded per-id checks (`recipe_id == 2001`, `resource_id ==
   1003`, …) with `QuestEventRouter::route`. No shared file was touched here.
2. Journal wiring: emit subtype 9/10 updates keyed by `objective_key` on the
   existing `OpeningJournal` path; repeated or regressive credit handling stays
   with the engine that owns objective state.
3. Native acceptance: an actual original-client run that completes a typed
   objective through this router and shows the journal counter. Nothing in this
   item was exercised natively; the lead owns the shared client window.
4. Binding original objectives (items 066/067 follow-up): each needs its
   recovered target identifier before it may enter the route table.

## Skills applied

RTK ran the commands normally with host-condensed output. Ponytail reused the
existing `serde`/`serde_json` dependencies and the pinned lab sources, added no
new dependency, and kept the change to one module, one test, one stdlib checker
and one test file. Caveman kept reporting short; this note stays complete.
game-recon was not re-run because the engine and formats are already
established. game-research-websearch principles applied to source selection:
pinned revisions, community-mirror provenance, no new scraping. reverse-
engineering was read-only inspection of the pinned decompilation (quest packet
family, objective definition id semantics). asset-pipeline was not applicable
(no assets produced). game-automation was not used: the assignment does not own
native testing.

## Limits

The route table proves identifier matching over the reconstructed chain, not
original server behaviour. The supplied records are canned mixed-era emulator
data. The checker reads local `../lab` paths and the module embeds a checked
copy; if the pinned content changes, the checker's drift test and the ignored
Rust cross-check fail instead of the router silently following stale data.
No currency, item, quest or experience grant was added or changed.
