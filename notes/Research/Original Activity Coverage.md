# Original Activity Coverage

Status: catalogue rebuilt and verified from preserved sources, 2026-10-09
10:50 UTC. Row-level native acceptance now requires a captured receipt bound to
the source activity id; the previous revision granted `native-accepted` to 44
rows by family, which this revision repairs. A second repair pass (see
`notes/Reviews/Activity Coverage Receipt Review.md`) adds the captured Smelting77
turn-in, retires the last "TYPE 9 Mimic anvil" slot wording, and renames the
type-level line to `strongest_row_status`. Static data audit plus a runtime
lookup module; no playability claim and no native run was made by this worker.

Owned artifacts (queue item 015):

| Path | What |
| --- | --- |
| `scripts/audit-minigame-coverage.py` | Read-only audit over the four shipped tables, the locale export, the pack catalogue, the earlier inventory and the receipt table |
| `tests/test_minigame_coverage.py` | 24 stdlib tests: parsing, findings, hash provenance, determinism, output, the native-receipt overclaim regression, and the evidence-text scan |
| `notes/Research/original-activity-coverage.json` | Committed catalogue (783 games, 24 referenced type ids, 20 groups, receipts, family gates, findings) |
| `../lab/reports/activity-catalogue-20261009T105054Z/` | Current lab evidence bundle: `catalogue.json` (byte-identical to the committed copy), `findings.json`, `provenance.json`, `run.log` |
| `../lab/reports/activity-coverage-receipt-review-20261009T103810Z/` | Independent review bundle: `row-verification.json`, `reward-packets.txt`, `receipt-packets.txt`, regenerated catalogue |
| `../lab/reports/receipt-repair-*.log` | RED/GREEN logs for both repair passes (the 06:22 and 10:50 bundles are preserved side by side) |

Earlier runs `activity-catalogue-20261009T022610Z`, the three before it and
`activity-catalogue-20261009T062252Z` are preserved but superseded; only the
current bundle should be cited.

## What this revision corrected

| Previous claim | Corrected position |
| --- | --- |
| 44 rows `native-accepted` for Cooking/Smelting by family/slot | 3 rows `native-accepted`, each bound to a captured receipt: 46, 77, 139 |
| TYPE 9 "Mimic movies (cooking, smelting, forging anvil, ...)" | TYPE 9 is cooking/smelting/music/sword; the forging row is TYPE 8 and is not a Mimic row |
| "22 declared / 24 referenced" stated loosely | 22 declared = 20 with game rows + 2 empty (4, 18); 24 referenced = 20 declared + 4 undeclared (0, 17, 1009, 1011); 26 distinct ids across both tables |
| Smelting77 receipt said "no stage5 reward/turn-in captured" | 173814 frames 8850-8852 grant the recipe output, 9086→9090-9093 turn it in (coins 120→130, Miner XP 25, stage 910005), so 77 is entry+completion+reward |
| TYPE 8 forging slot evidence still said "distinct from the native-accepted TYPE 9 Mimic anvil" | replacement text states the TYPE 8 receipt-bound row 139, the local MatchThree model and the unproven original engine class; a catalogue-wide text scan guards it |
| `types[*].restoration_status` read as family acceptance | renamed to `strongest_row_status`; per-row acceptance stays in `native_accepted_rows`/`row_status_counts` and the movie claim stays in `family_gate` |

## Provenance (read-only inputs)

| Input | Bytes | SHA-256 |
| --- | --- | --- |
| `lab/working/OSFR.Client/OSFR Client/Resources/MiniGameData.txt` | 30,734 | `8ce3cc61a2ad324912465e8d1785d246ccd680a01712e256fddf702ce856c252` |
| `.../MiniGameTypeData.txt` | 805 | `2d200849d113c0f4b071b869774843b2fe9d07095235e7a6e907be83156e27d0` |
| `.../MiniGameGroupData.txt` | 678 | `3b288fa6e665c5ea41122195273ae846251835c8fc323183407e0d48ca969048` |
| `.../MiniGameGroupLinks.txt` | 13,582 | `bff472fb91e88ebf6077f541012701cc774dcfb1d7b89261b2ad92d95ffcb623` |
| `lab/reports/locale-id-to-text.json` (community reader output, 85,932 entries) | 6,788,031 | `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844` |

All four table hashes match the recovery intake; the receipt evidence paths are
re-checked on every run and `native_receipt_evidence_missing` is part of the
findings.

## Counts

783 game rows, 739 named and 44 unnamed, 173 rows with no group link. Type ids:
22 declared (20 with game rows, 2 declared-but-empty: 4 and 18); 24 referenced by
game rows (20 declared + 4 undeclared: 0, 17, 1009, 1011); 26 distinct ids across
both tables. Declared engine flag across the 783 rows: 360 flash-declared, 417
native-declared, 6 rows on undeclared types. Status totals: 3 native-accepted,
195 code/test-candidate, 511 missing-implementation, 74 unknown.

## Native acceptance rule and receipts

`native-accepted` requires the source activity id to be bound to a captured
native entry **and** completion receipt (a reward receipt is recorded when
captured). Inference from type, family, movie, name or queue coverage is not
acceptance; an unreceipted row stays `code/test-candidate` when a module exists
and explicit `unknown` otherwise.

| Row | Name | Receipt kinds | Captures | Caveat |
| --- | --- | --- | --- | --- |
| 46 | Cooking: Strong Arm Stew | entry, completion, reward | 164218, 171251, 054421 | frames: completion 6298 (164218); entry 1560, recipe 3866-3870, completion 3873, close 3927, Mae turn-in 4068-4072 (171251); stage13 burst 6368-6372, turn-in 6448-6459 (054421, local recipe) |
| 77 | Smelting: Copper Bar | entry, completion, reward | 173814 | frames: entry 2614, recipe 8850-8852, completion 8855, close 8935, turn-in 9086→9090-9093 (coins 120→130, Miner XP 25, stage 910005); the reconstructed part is the rule VALUES (5 ore + 3 coal → 10 bars) and the reward mapping, since MiniGameData.txt has no reward column |
| 139 | Forging: Spunky Scrapper Hammer | entry, completion, reward | 173814, 181715 | frames: entry 139000001/score 6600 then IO failure (173814); entry 1455, recipe 2901-2905, completion 2943, close 3010, Cleetus turn-in 3203-3206 (181715); pre-Close state is a log projection |

Evidence: `notes/Reviews/Native Cooking Six Rounds 164218.md`,
`notes/Reviews/Native Cooking Finish and Mae 171251.md`,
`notes/Reviews/Native Gear Up and Smelting 173814.md`,
`notes/Reviews/Native Forging Victory 181715.md`,
`lab/reports/native-cooking-mae-054421-review/receipt.json`,
`prototype/osfr-tools/src/forging.rs`.

## Family and movie gates (separate from row acceptance)

Family gates record which original movie actually completed natively. They never
promote every row of a type:

| Type | Movie gate | Verified rows | Scope |
| --- | --- | --- | --- |
| 8 | true | 139 | forging board movie completed for row 139 only; mining/harvesting are module-tested, artifact/mailsorting/halloween and the other 60 forging rows have no module or receipt |
| 9 | true | 46, 77 | original Cooking46 and Smelting77 movies completed; all other TYPE 9 rows (other recipes, music conducting, sword testing, unnamed) are unreceipted |

Everything else is `movie_accepted: false` with the family's own evidence. The
type record carries `row_status_counts` and `native_accepted_rows` for per-row
acceptance, and the family line itself is named `strongest_row_status` (not
`restoration_status`) so it cannot be read as family-wide acceptance.

## Leftover wording outside this piece's ownership

`prototype/osfr-tools/src/restoration/piece_018.rs:156` still calls the forging
entry "TYPE 9 mimic anvil movie" (its own records already carry
`original_type: 8`). Piece 015 does not edit that file; the record is listed in
`findings.leftover_wording_outside_ownership` for the owning piece to retire.

## Forging 139: TYPE 8 row, local model, no Mimic binding

Shipped row 139 (`Forging: Spunky Scrapper Hammer`) is TYPE 8 (`IS_FLASH=1`).
`prototype/osfr-tools/src/forging.rs` binds `const GAME: i32 = 139` and drives a
server-owned MatchThree board; the 181715 results page titled itself from row
139's name, so the id binding itself is captured evidence. The original service
engine class for row 139 stays unproven and no TYPE 9/Mimic binding is invented.
The other 60 TYPE 8 `Forging:` rows remain unreceipted and unimplemented.

## Type map

Restoration is the per-row mix; the native column lists receipt-bound rows only.

| TYPE | Games | IS_FLASH | Family | Row statuses | Native rows |
| --- | ---: | ---: | --- | --- | --- |
| 0 | 1 | - | undeclared, NOT LIVE Sacred Glade tutorial | missing 1 | - |
| 1 | 152 | 0 | instanced world minigames (IWMG) and quest activities | missing 152 | - |
| 2 | 91 | 0 | postman delivery minigames (mostly NOT LIVE) | missing 91 | - |
| 3 | 4 | 1 | trading card game lobby and tables | missing 4 | - |
| 4 | 0 | 0 | declared type with no rows | - | - |
| 5 | 1 | 0 | Robgoblin Camp battle activity | missing 1 | - |
| 6 | 12 | 0 | Chess | missing 12 | - |
| 7 | 6 | 0 | Checkers | missing 6 | - |
| 8 | 131 | 1 | MatchThree boards incl. the receipted forging board | native 1, code 51, missing 12, unknown 67 | 139 |
| 9 | 54 | 1 | Mimic movies (cooking, smelting, music, sword) | native 2, code 42, missing 8, unknown 2 | 46, 77 |
| 10 | 68 | 0 | kart racing sessions | code 68 | - |
| 11 | 48 | 0 | demolition derby arenas | missing 48 | - |
| 12 | 22 | 0 | soccer matches | code 22 | - |
| 13 | 5 | 0 | vehicle race in the wilds | code 2, unknown 3 | - |
| 14 | 46 | 1 | tower defense maps | missing 46 | - |
| 15 | 1 | 0 | Herbalist Mahjong (DNT) | missing 1 | - |
| 16 | 124 | 1 | trading card game rounds and decks | missing 124 | - |
| 17 | 1 | - | undeclared single unnamed row | unknown 1 | - |
| 18 | 0 | 0 | declared type with no rows | - | - |
| 19 | 1 | 0 | kart racing (second session slot) | code 1 | - |
| 20 | 1 | 0 | kart racing (third session slot) | code 1 | - |
| 21 | 8 | 0 | fishing lakes and tutorials | code 8 | - |
| 22 | 1 | 0 | daily wheel | missing 1 | - |
| 23 | 1 | 1 | Treasure War | missing 1 | - |
| 1009 | 3 | - | undeclared, Spot the Difference | missing 3 | - |
| 1011 | 1 | - | undeclared, Cannonball Chatty | unknown 1 | - |

The largest unreceipted families remain TYPE 1 (152), TYPE 2 (91), TYPE 16
(124), TYPE 14 (46) and the 60 unbound TYPE 8 forging rows.

## Data quality

Zero malformed rows, zero duplicate ids in the three keyed tables, zero dangling
group links, zero groups without links, zero source-hash mismatches and zero
missing receipt evidence. Semantic gaps stay explicit: 9 locale name ids and 3
description ids are absent from the community export (`-1` for rows 700-703;
440646/440650/440654/440701/440702/440703 for 1076-1081; 4420212/4420221 for
802/803); every group row has `NAME_ID=0` and every link row has empty
`LINK_NAME`/`SHORT_NAME`, so group identity is only inferable from members.

## Verification (commands and results)

RED before the repair (from the project directory):

```
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests -p "test_minigame_coverage.py" -v
```

First repair RED: `Ran 20 tests ... FAILED (failures=1, errors=5)` - the receipt
tests found `AttributeError: ... has no attribute 'NATIVE_ROW_RECEIPTS'`, the
missing `type_id_math` field and the old "forging anvil" TYPE 9 label; then
`Ran 20 tests ... OK`. Second repair (this pass) RED: `Ran 24 tests ...
FAILED (failures=3, errors=1)` - Smelting77 lacked `reward`, the TYPE 8 slot
evidence still contained "distinct from the native-accepted TYPE 9 Mimic anvil",
type records lacked `strongest_row_status`, and the leftover-wording finding was
absent; the same command now reports `Ran 24 tests ... OK`.

Regeneration:

```
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\audit-minigame-coverage.py' --stamp 20261009T105054Z
```

`games=783 flash=360 native=417 unknown=6`, committed catalogue written to
`notes/Research/original-activity-coverage.json` (sha256
`00ce0a0e28af8b4939dd44af7134238a67a00025d71e5357d279ed2a92e7453b`) with
evidence in `../lab/reports/activity-catalogue-20261009T105054Z/`.

## Boundary and remaining gates

Original facts: the four shipped tables and their hashes, the locale export, the
pack catalogue and the native captures referenced by the receipt table.
Reconstruction: status vocabulary, slot grouping, family scopes, caveats and the
local forging model. Not established: any original engine-class binding for row
139 beyond id and name, per-row acceptance for the other 780 rows, and any
native run by this worker. Items 016/017 should build on these ids; items
010/042 consume the TCG and postman rows mapped here. Known leftover outside
this piece: `src/restoration/piece_018.rs:156` still says "TYPE 9 mimic anvil
movie" (listed in `findings.leftover_wording_outside_ownership`, not edited).
