# 015 Minigame launch catalogue

Status: mapping rebuilt and verified 2026-10-09 03:00 UTC, repaired 06:22 UTC so
row-level `native-accepted` requires a captured receipt bound to the source
activity id, and repaired again 10:50 UTC for the independent review findings in
`notes/Reviews/Activity Coverage Receipt Review.md` (Smelting77 reward, TYPE 8
slot wording, `strongest_row_status`, leftover piece-018 wording). Static data
audit plus a runtime lookup module; no native client run was made by this worker
and nothing here is a playability claim.

## What this piece delivers

| Path | What |
| --- | --- |
| `scripts/pieces/piece_015.py` | Builds the 783-row mapping (source line, declared client mode, shipped-mode evidence, slot, implementation coverage, native receipts) and validates it |
| `tests/pieces/test_piece_015.py` | 20 stdlib tests: caret grammar, module id binding, token evidence, fixture mapping, receipt rule incl. the Smelting77 turn-in, real-source counts, determinism, outputs |
| `prototype/osfr-tools/src/restoration/piece_015.rs` | Runtime parser + launch-mode lookup for the four shipped tables (not yet registered in `lib.rs`) |
| `prototype/osfr-tools/tests/piece_015.rs` | 6 owned tests including the real-table run (783/22/20/610) |
| `../lab/reports/piece-015-20261009T030000Z/` | Evidence bundle: `catalogue.json`, `catalogue-rows.tsv` (783 rows), `coverage.json`, `findings.json`, `provenance.json`, `run.log`, RED/GREEN logs, cargo log |

The other catalogue of this queue item is
`notes/Research/Original Activity Coverage.md` + `notes/Research/original-activity-coverage.json`
(sha256 `00ce0a0e28af8b4939dd44af7134238a67a00025d71e5357d279ed2a92e7453b`).
It was repaired twice the same day: its first revision marked 44 rows
`native-accepted` by family, which the receipt rule forbids, and the second pass
fixed the Smelting77 reward record, the TYPE 8 forging slot wording and the
type-level `strongest_row_status` name. This piece consumes it as a cross-check
input and now reports zero disagreements.

## Provenance (read-only inputs)

| Input | Bytes | SHA-256 |
| --- | --- | --- |
| `lab/working/OSFR.Client/OSFR Client/Resources/MiniGameData.txt` | 30,734 | `8ce3cc61a2ad324912465e8d1785d246ccd680a01712e256fddf702ce856c252` |
| `.../MiniGameTypeData.txt` | 805 | `2d200849d113c0f4b071b869774843b2fe9d07095235e7a6e907be83156e27d0` |
| `.../MiniGameGroupData.txt` | 678 | `3b288fa6e665c5ea41122195273ae846251835c8fc323183407e0d48ca969048` |
| `.../MiniGameGroupLinks.txt` | 13,582 | `bff472fb91e88ebf6077f541012701cc774dcfb1d7b89261b2ad92d95ffcb623` |
| `lab/reports/locale-id-to-text.json` (community reader output, 85,932 entries) | 6,788,031 | `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844` |
| `lab/manifests/rust-asset-catalog.tsv` (24-pack catalogue) | 11,218,192 | not hash-pinned by this piece; asset names verified by substring |
| `lab/decompiled/client/FreeRealms/pseudocode.c` (pinned Ghidra export) | 95,176,801 | `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3` |

Every one of the four table hashes matches the recovery intake, and the CLI
fails if any of them changes.

## Counts

783 game rows; 22 declared type ids (20 with game rows, 2 declared-but-empty:
4 and 18); 24 referenced by game rows (20 declared + 4 undeclared: 0, 17, 1009,
1011); 26 distinct ids across both tables. 20 groups; 610 group links; 739 named
rows and 44 unnamed; 173 rows have no group link.

Declared mode (`MiniGameTypeData.IS_FLASH`): 360 flash-declared, 417
native-declared, 6 rows whose type is undeclared and therefore unlabeled.

## Client mode

`client_reading` is a reconstruction from recovered artifacts, kept separate
from the declared flag:

| Type | Rows | Declared | Reading | Evidence |
| --- | --- | --- | --- | --- |
| 0 | 1 | undeclared | unresolved | NOT LIVE Sacred Glade tutorial row |
| 1 | 152 | native | unresolved | IWMG/quest activities, no single movie or binding recovered |
| 2 | 91 | native | unresolved | postman delivery rows, 60 NOT LIVE |
| 3 | 4 | flash | mixed-unresolved | TCG lobby; native `tradingcard` identifiers |
| 5 | 1 | native | unresolved | Robgoblin Camp battle |
| 6 | 12 | native | mixed-unresolved | `game_chess.gfx` tutorial + 1,579 native `chess` tokens |
| 7 | 6 | native | mixed-unresolved | `game_checkers.gfx` only; no native checkers tokens |
| 8 | 131 | flash | flash-movie | mining/farming/mailsorting/forging/artifact/halloween movies; 0 `matchthree`/`mailsort` tokens |
| 9 | 54 | flash | flash-movie | cooking/smelting/forging/music/sword movies; 0 `musicconduct`/`swordtest` tokens |
| 10 | 68 | native | native-client | 190 `vehiclerace` tokens; `kart_racing.rs` lobby ids 3052/3053 |
| 11 | 48 | native | native-client | 120 `demolitionderby` tokens |
| 12 | 22 | native | native-client | 225 `soccer` tokens; `soccer.rs` id 42 |
| 13 | 5 | native | native-client | same `vehiclerace` family as type 10 |
| 14 | 46 | flash | flash-movie | `tutorial_towerdefense.gfx`; 0 `towerdefense` tokens |
| 15 | 1 | native | flash-movie | `game_mahjongmatch.lst`; 0 `mahjong` tokens (declared flag contradicts the shipped movie) |
| 16 | 124 | flash | mixed-unresolved | 656 native `tradingcard` tokens; an earlier note called the decks Flash, so the split is unresolved |
| 17 | 1 | undeclared | unresolved | single unnamed row |
| 19 | 1 | native | unresolved | "Wildwood Kart Racing" id 582; racing-family inference only, no row-level symbol |
| 20 | 1 | native | unresolved | "Wildwood Kart Racing" id 583; same |
| 21 | 8 | native | native-client | 3 `controllerfishing` tokens; `fishing.rs` |
| 22 | 1 | native | flash-movie | `game_wheel.gfx` (declared flag contradicts the shipped movie) |
| 23 | 1 | flash | unresolved | Treasure War |
| 1009 | 3 | undeclared | flash-movie | Spot the Difference; `game_hidden.lst`; 0 `hiddenobject`/`spotdiff` tokens |
| 1011 | 1 | undeclared | unresolved | Cannonball Chatty |

Token counts are case-insensitive **substring** counts over the pinned Ghidra
export (`chess` counts `ChessLib` and `TradingCard`-style identifiers too). That
definition is deliberate and pinned: word-boundary counting reports `chess` 809
and `soccer` 3, which hides the identifier evidence the reading rests on. The
checker re-runs the counts and fails if they change.

## Implementation coverage and native receipts

Status vocabulary: `native-accepted` (a captured receipt binds the source
activity id), `code/test-candidate` (module + tests exist, review/native gate
pending), `missing-implementation`, `unknown`. Current totals:

| Status | Rows |
| --- | --- |
| native-accepted | 3 |
| code/test-candidate | 195 |
| missing-implementation | 511 |
| unknown | 74 |

Receipt rule: `native-accepted` requires the source activity id to be bound to a
captured native entry and completion receipt (reward recorded when captured);
type/family/movie/name inference is not acceptance.

| Row | Name | Receipt kinds | Captures |
| --- | --- | --- | --- |
| 46 | Cooking: Strong Arm Stew | entry, completion, reward | 164218, 171251, 054421 (completion 6298; entry 1560, recipe 3866-3870, close 3927, Mae turn-in 4068-4072) |
| 77 | Smelting: Copper Bar | entry, completion, reward | 173814 (entry 2614, recipe 8850-8852, completion 8855, close 8935, turn-in 9086→9090-9093) |
| 139 | Forging: Spunky Scrapper Hammer | entry, completion, reward | 173814 (partial, score 6600), 181715 (entry 1455, recipe 2901-2905, completion 2943, close 3010, turn-in 3203-3206) |

Rows whose MiniGameData id is bound by a module constant keep that module's
coverage (20 rows), but the three receipt-bound rows take the receipt instead:

| Module | Bound game ids | Status |
| --- | --- | --- |
| `src/mining.rs` | 18, 91 | code/test-candidate |
| `src/harvesting.rs` | 58, 109-120, 217 | code/test-candidate |
| `src/cooking.rs` | 46 | native-accepted (receipt) |
| `src/smelting.rs` | 77 | native-accepted (receipt) |
| `src/forging.rs` | 139 | native-accepted (receipt) |
| `src/soccer.rs` | 42 | code/test-candidate |
| `src/kart_racing.rs` | none (type/lobby constants only) | code/test-candidate for types 10/19/20 |

Family/movie gates stay separate from row acceptance and are carried in the
ledger (`family_gate`) and by the other catalogue: the Cooking46 and Smelting77
movies completed natively (only rows 46/77 receipted) and the forging board
movie completed for row 139 only. Rows in a module-covered slot without an id
binding rely on that slot's module evidence: 68 kart rows, 21 soccer rows, 2
wilds-race rows, 8 fishing rows, plus the unbound regional rows of the bound
families (27 mining, 8 harvesting, 38 cooking, 4 smelting).

## Forging 139: TYPE 8 row, local model, no Mimic binding

Shipped row 139 is TYPE 8 (`IS_FLASH=1`). `forging.rs` binds
`const GAME: i32 = 139` and drives a server-owned MatchThree board; the 181715
results page titled itself from row 139's name, so the id binding is captured.
The original service engine class for row 139 stays unproven; no TYPE 9/Mimic
binding is invented, and the other 60 TYPE 8 forging rows are unreceipted.

## Findings and gaps

Clean at the format level: 0 malformed rows, 0 duplicate ids, 0 dangling group
links, 0 groups without links, 0 missing evidence paths, 0 missing receipt
evidence, 0 native-token drift, 0 disagreements with the repaired catalogue.
Gaps: 9 locale name ids and 3 description ids are absent from the community
export (`-1` for rows 700-703; 440646/440650/440654/440701/440702/440703 for
1076-1081; 4420212/4420221 for 802/803); all 20 groups have `NAME_ID=0` and
every link row has empty `LINK_NAME`/`SHORT_NAME`, so group identity is only
inferable from members; 173 rows have no group link; 44 rows have no name.
Largest uncovered families: 152 TYPE 1 instanced-world rows, 91 TYPE 2 postman
rows (60 NOT LIVE), 124 TYPE 16 TCG rows, 61 TYPE 8 forging rows (one bound),
46 tower defense maps.

## Verification (commands and results)

Python arm, from the project directory:

```
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests\pieces -p "test_piece_015.py" -v
```

RED before implementation (2026-10-09 03:00 run): `FileNotFoundError:
scripts\pieces\piece_015.py`. After implementation: `Ran 17 tests ... OK`. After
the first receipt repair: `Ran 19 tests ... OK`; after the review repair:
`Ran 20 tests ... OK`. The receipt regression itself went RED first (`unknown`
for an unreceipted cooking sibling) and is green now.

```
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_015.py'
```

`games=783 types=22 groups=20 links=610`, engine `{flash 360, native 417,
undeclared 6}`, status `{native-accepted 3, code/test 195, missing 511,
unknown 74}`, `prior_disagreements=0`, exit 0.

The other catalogue's arm (same receipt rule):

```
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests -p "test_minigame_coverage.py" -v
```

First RED with `FAILED (failures=1, errors=5)` (missing `NATIVE_ROW_RECEIPTS`,
missing `type_id_math`, old "forging anvil" TYPE 9 label) → `Ran 20 tests ...
OK`. Review-repair RED with `FAILED (failures=3, errors=1)` (Smelting77 lacked
`reward`, TYPE 8 slot evidence still said "TYPE 9 Mimic anvil", type records
lacked `strongest_row_status`, leftover-wording finding absent) → `Ran 24 tests
... OK`, and regeneration with
`scripts/audit-minigame-coverage.py --stamp 20261009T105054Z`.

Rust arm, from `prototype\osfr-tools`:

```
& '..\..\scripts\cargo.ps1' test --test piece_015
```

RED before implementation: `couldn't find file tests\..\src\restoration\piece_015.rs`.
After implementation: `6 passed; 0 failed`, including the real-table test that
parses the preserved 783/22/20/610 rows.

## Boundary and remaining gates

Original facts: the four shipped tables, their hashes, the locale export, the
pack catalogue entries, the Ghidra export, the id constants in the existing Rust
modules and the native captures referenced by the receipt table. Reconstructed
policy: coverage statuses, slot vocabulary, client readings, family gates and
the module acceptance table. Not verified: any original engine-class binding for
row 139 beyond id and name, per-row acceptance for the other 780 rows, and
native launch behaviour.

Remaining integration: `src/restoration/piece_015.rs` is not registered in
`lib.rs` and no gateway launch path reads the catalogue yet; the lead owns that
registration and any routing. Item 016/017 (loss/retry, disconnect recovery)
should build on these ids, and item 024/042/010 consume the group and TCG rows
mapped here. Known leftover outside this piece:
`src/restoration/piece_018.rs:156` still says "TYPE 9 mimic anvil movie"
(listed in `findings.leftover_wording_outside_ownership`, not edited here).
