# 040 Crafting job progression

Status: bounded recovery plus checker returned for lead review. Item 040
acceptance is "Chef and Blacksmith recipe/rank unlocks; preserve current
accepted crafting behavior". This piece recovers the two crafting jobs, their
shard classes, the 49 recovered recipe records with their `MinProfileRank`
rank gates, the job level title rows, the level 5/10/15/20 gate quests and the
two recovered wiki acquisition rows, and ships a fail-closed rank access
validator. Native rank gating remains a separate gate; no accepted crafting
service was modified.

## Recovered original facts

Jobs and shard classes are recovered from the pinned Legacy `ItemClasses.txt`
(revision `9e082025391177d5a33ee18c714ddd06b457a695`, AGPL-3.0, inspected as
facts; no upstream source copied into this repository) and the preserved
locale table:

| Profile | Name id | Recipe shard class | Class owner |
|---|---|---|---|
| 16 Blacksmith | 2614 | 58 `Blacksmith Shards` (name id 2561) | Blacksmith |
| 45 Chef | 2856 | 59 `Chef Shards` (name id 2562) | Chef |

`ItemClasses.txt` stores the owner's profile *name* id; the recovered job rows
are the name-id to profile-id mapping used by the checker.

Recipe records are the `Class 58`/`Class 59`, `CategoryId 17`, `Type 5`
records of the pinned `ClientItemDefinitions.json` (same revision). All 49
carry a `Recipe` name, single stack, `NoSale`, `NoTrade`, `Cost 540`, a 10%
member discount and `Param1` as their own recipe index; indices are unique
within each shard class. `MinProfileRank` is the recovered rank gate:

| Job | rank ladder (rank, item id) |
|---|---|
| Blacksmith (class 58, 27 rows) | (4, 23882) (5, 2514) (10, 49385) (15, 2519) (15, 49382) (20, 2523) (20, 2525) (20, 49383) (20, 49384) |
| Chef (class 59, 22 rows) | (5, 849) (5, 851) (10, 854) (10, 857) (15, 852) (15, 859) (15, 863) (20, 865) (20, 867) |

Every other recovered recipe row carries `MinProfileRank` 1. Record 23872 is
the single row whose JSON field is absent.

Two recovered records sit under the other job's shard class: `869
Chocolate Bumbleberry Mousse Recipe` (Chef name, class 58) and `2518
Chivalrous Guardian Hammer Recipe` (Blacksmith name, class 59). They are
reported, not resolved; the validator follows the recovered class owner.

Job level titles and gate quests are recovered locale rows
(`report/job-catalogue-20261009T022023Z/locale-en_us.json`, 85,932 rows):

| Job | title rows (level: title / requirement ids) | gate quests (level: quest ids) |
|---|---|---|
| Blacksmith | 4: 34060/34140, 8: 34061/34141, 12: 34062/34142, 16: 34063/34143, 20: 34064/34144 | 5: 387551/387615, 10: 387555/387625, 15: 387559/387633, 20: 387563/387641 |
| Chef | 4: 34075/34155, 8: 34076/34156, 12: 34077/34157, 16: 34078/34158, 20: 34079/34159 | 5: 387247/387267, 10: 387255/387273, 15: 387259/387279, 20: 387263/387285 |

The two recovered recipe acquisition rows come from the community wiki
(`wiki.sqlite`): "List of Recipes (Chef)" revision 45318 records
`Strong Arm Stew` as `Where to find: Tutorial` and `Starnut Sundae` as
`Where to find: Gain 10 levels in Chef`. The supplied client's recovered
`MiniGameData.txt` binds those recipes to minigames: rows 46 and 560
`Cooking: Strong Arm Stew` difficulty 1 and row 186 `Cooking: Starnut Sundae`
difficulty 3. The Chef revision 46136 and Blacksmith revision 46062 pages
list the level gate quests.

## Boundary: original, inferred, reconstructed

Original (recovered) facts: the two jobs and their name ids, the shard class
owners, all 49 recipe records with `Param1`/`NameId`/shape flags and
`MinProfileRank`, the 10 level title rows, the 8 gate quest rows, the two
wiki acquisition rows, and the minigame row bindings above.

Inferred: the absent `MinProfileRank` field on 23872 serializes as the
default 0; the field is not written in the recovered JSON.

Reconstructed: the access rule enforced by `recipe_access`/`wiki_gate_access`
(the recipe's recovered shard class owner must be the requesting job, rank 0
is below the recovered rank floor of 1, requests above the recovered maximum
rank 20 clamp to 20, rows with no recovered entry are refused as
`UnknownRecipe`). The accepted cooking service requires profile 45 and the
forging service profile 16; the module mirrors those constants and the
checker fails if either production source drifts.

Explicit gaps, reported rather than invented:

- `job_rank_curve_missing`: the star/experience curve that raises a job rank
  was not recovered; `src/profiles.rs` still uses its reconstructed local
  curve.
- `recipe_rank_table_partial`: only the item `MinProfileRank` fields and the
  single Starnut Sundae wiki row were recovered; 23 of 25 wiki Chef rows have
  an empty `Where to find` cell, and no Blacksmith recipe acquisition rows
  exist in the preserved pages.
- `wiki_blacksmith_gate_lines_missing`: the Blacksmith wiki page spells out
  only levels 10 and 20; levels 5 and 15 are reported as a warning.
- `recipe_index_rows_missing`: the `MimicMiniGameData`/`MimicMiniGamePhaseData`
  rows that bind a `Param1` recipe index to a minigame were not recovered.
- `minigame_difficulty_semantics_unknown`: difficulty 1/3/5 is a recovered
  file field with no recovered relation to job ranks; it is deliberately not
  used as a rule.
- `cross_class_records_unresolved`: 869 and 2518 are reported by id.
- `native_rank_gating_unverified`: no client input was sent.

## Artifacts

- `prototype/osfr-tools/src/restoration/piece_040.rs` - recovered tables and
  the `recipe_access`/`wiki_gate_access` validator. Not registered in
  `lib.rs`; the owned test `#[path]`-includes it until the lead registers it.
- `prototype/osfr-tools/tests/piece_040.rs` - 11 focused tests.
- `scripts/pieces/piece_040.py` - read-only recovery and drift checker; writes
  only its report directory.
- `tests/pieces/test_piece_040.py` - 5 stdlib tests, including one over the
  real preserved corpus.
- `../lab/reports/piece-040-20261009T035326Z/` - RED/GREEN logs,
  `piece-040-progression.json`, `summary.md`, `run.log`, the discovery probes
  and the sibling regression log.

Pinned source hashes of the evidence run: Legacy
`ClientItemDefinitions.json`
`40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`,
`ItemClasses.txt`
`73c2a9377477ff88e6f4966614930c3d375cd9d5904fddaa7544ec24f0c2f567`,
client `MiniGameData.txt`
`8ce3cc61a2ad324912465e8d1785d246ccd680a01712e256fddf702ce856c252`,
locale `locale-en_us.json`
`f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844`,
`wiki.sqlite`
`b9d7b9f1adabb42db54c3253bf43fddc249d6d70e3f2075b134a9441e2fcd911`.

## Commands and results

Run from `%USERPROFILE%\Documents\Projects\CartoonMMO\project`.

```powershell
# RED before the module existed (exit 101, missing piece_040.rs)
& scripts/cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','piece_040')

# GREEN: 11 passed; 0 failed
& scripts/cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','piece_040')

# Accepted crafting preservation: cooking/gameplay, forging/gameplay, smelting/gameplay and pieces 012/013/014
& scripts/cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','cooking','--test','cooking_gameplay','--test','forging','--test','forging_gameplay','--test','smelting','--test','smelting_gameplay','--test','piece_012','--test','piece_013','--test','piece_014')
# 59 passed; 0 failed; 10 ignored across the nine accepted suites

# Checker tests: RED before the script existed, then 5 passed
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_040.py" -v

# Evidence build (status=pass; 14 pass, 2 warn, 0 fail)
& '..\lab\tools\wiki-env\Scripts\python.exe' scripts/pieces/piece_040.py --output '..\lab\reports\piece-040-20261009T035326Z'
```

Checker warnings: the Blacksmith wiki page does not spell out levels 5 and
15, and the wiki Chef list covers 25 rows against 22 recovered Chef recipe
items (6 items without a wiki row, 9 wiki rows without a recovered item).

## Remaining gates

- Lead integration: register `piece_040` in `lib.rs` only when a consumer
  exists, or use the checker as the content regression gate. Wiring the
  validator into the cooking/forging flow is lead-owned and was not modified
  here.
- Native: the original client's rank gating, recipe learning and any
  level-up presentation for Chef/Blacksmith remain unexercised.
- Original: the star curve, the `Param1` recipe rows and the Blacksmith
  acquisition rows stay explicit gaps.

## Skills applied

RTK (normal commands first; only the report directory was written), Ponytail
(stdlib-only checker, existing pinned community corpora reused, no new
dependency, no speculative abstraction), Caveman (concise reporting; persisted
notes stay complete), community-first recovery (pinned Legacy tables, wiki
corpus and supplied client resources before any custom reversing) and
test-first RED/GREEN for both the Rust module and the checker.
