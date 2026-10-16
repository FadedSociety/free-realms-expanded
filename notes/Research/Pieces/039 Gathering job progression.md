# 039 Gathering job progression

Status: closed for review, 2026-10-09 04:05 UTC. This piece recovers the **Farm/Ranch rank
progression** (the Farmer job's XP/rank/unlock table) and the **Miner rank gates** that the
original client actually ships, marks the Miner XP curve as an explicit gap, and adds the
duplicate-grant guard that items 001 (mining) and 002 (harvesting) must pass their outcomes
through. It is evidence, a checker and a small typed module; it is not a playable claim and it
grants no currency, items or XP by itself.

Owned paths: `prototype/osfr-tools/src/restoration/piece_039.rs`,
`prototype/osfr-tools/tests/piece_039.rs`, `scripts/pieces/piece_039.py`,
`tests/pieces/test_piece_039.py`, this note and `lab/reports/piece-039-20261009T040246Z/`. No
shared `lib.rs`, gameplay, gateway, Cargo manifest, launcher, content or state file was touched.

## Recovered original facts

**Farm/ranch rank table.** The original client ships `Resources/FactoryRanks.txt`
(sha256 `b1d26b59…c55b`), columns
`HOUSING_INSTANCE_ID^EXPERIENCE_POINTS_REQUIRED^NAME_STRING_ID^RANK^GAIN_RANK_COMPOSITE_EFFECT^FACTORY_TYPE^YIELD_MULTIPLIER^REWARD_SET_ID^ITEM_CAP_MODIFIER^MEMBERS_ONLY`.
Sixty rows cover exactly three instances, twenty ranks each:

| instance | type | ranks | xp required | item cap | reward sets | members only |
|---|---|---|---|---|---|---|
| 58 | FARMING | 1..20 | 0, 180, 540, 990, 1530, 2370, 3330, 4410, 5910, 7710, 10230, 13110, 16890, 21930, 28410, 35700, 45000, 57600, 72810, 90000 | 3, 4, 5, 6, 7, 8, 9, 10, 12, 14, 16, 18, 21, 24, 27, 31, 35, 39, 44, 50 | 4274 at rank 3, 4275 at 5, 4276..4289 at 7..20, none at 1, 2, 4, 6 | rank ≥ 5 |
| 63 | FARMING | 1..20 | identical ladder | identical caps | 4381 at rank 3, 4382 at 5, 4383..4396 at 7..20, none at 1, 2, 4, 6 | rank ≥ 5 |
| 60 | RANCHING | 1..20 | identical ladder | 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6 | none | rank ≥ 5 |

Every row carries `NAME_STRING_ID` 0 (the rank has no localized name),
`GAIN_RANK_COMPOSITE_EFFECT` 15263 and `YIELD_MULTIPLIER` 0.0. The client native loader reads this
file into `FactoryRankDefinition` maps (`lab/decompiled/client/FreeRealms/pseudocode.c`: the
`FUN_00402ff0(…, "FactoryRanks.txt", …)` load and the
`HashListMap<int,FactoryRankDefinition,1024,-1>` instantiation), and the client Lua raises
`FactoryHandler:OnFactoryRankUp(rank)` when a rank lands.

**Which instances are player plots.** `Resources/FactoryPlots.txt` (sha256 `eb6897c3…20cd`) binds
plot 3 to FARMING 58, plot 5 to FARMING 63 and plot 7 to RANCHING 60, all three with the 1360-1402
x 728-774 grid, `MISFORTUNE_CHECK_TIME_SECONDS` 240, `MAXIMUM_MISFORTUNES` 120 and
`MISFORTUNE_CHECK_PERCENT` 100 for the two farms (the ranch has misfortunes off). Plots 4 and 6
(FARMING 59 and 64) and plot 8 (RANCHING 62) have **no** `FactoryRanks.txt` rows, and plot 1/2
(instance 57) is a separate small 28 x 38 grid without rank rows, so only 58, 63 and 60 are
progression tracks.

The Farmer job itself is community-documented: the wiki `Farmer` page records the job as
available since May 2011 and "not in the rotation", with Farmer Chug quests levelling it. The
original locale carries the matching step text the job walks through — 434399 pull weeds, 434411
scare pesky birds, 434415 Farm-O-Matic, 434733 tiller/fallow topsoil, 434741 plant a Bumbleberry
seed, 434749 water the crop, 434759 first harvest, 436187/436196/436206/436211 Farmer Chug and
Farmer's Market — plus the "Farm Unlocks Are Here!" announcements (684, 685, 686, 709, 710). The
two farms are named at locale 435461 `Briarwood Farm` and 5409 `Wilds Farm` (community
`Houses.json` properties 33 and 34; wiki `Farms`: Wilds Farm free, Briarwood Farm paid).

**Miner vein gates.** The original locale rows 1663..1666 give the exact gate text and rank:
"You must be at least a level 4 Miner to mine this Tin Vein", 8 for Iron, 12 for Silver and 16 for
Gold. Copper carries no gate string at all, so copper stays ungated. These are original client
facts and they conflict with the wiki `Miner` page's 1/5/10/15/20 ore table; the locale rows win
for the captured build's client behaviour, and the wiki stays community documentation.

**Miner rank gates.** `Resources/Requirements.txt` (sha256 `1dc2f97f…2644`) rows 17..20 are
`CheckRankLastProfileByType`, operator 6, profile **type 9** at ranks 5, 9, 13 and 17. Miner
profile 14 has type 9 in the captured `Profiles.json` rows (item 031 catalogues the mapping), so
these are the recovered Miner-content gates. The client native requirement factory registers
`ClientRequirementCheckLastPlayedProfileOfTypeRank`, matching the operator semantics "last played
profile of that type has rank ≥ N". Type 9 is also carried by the two community-only staff rows
(Enforcer, Referee) in item 031's catalogue, so the rows identify the type, not the profile id.

**Rank wire contract the client reads.** `Profile_DS` (client Lua, `BaseClient.Profiles`) places
`Rank` at column 4, `StarsAvailable` at 5, `StarsEarned` at 11 and `RankPercent` at 12. The
captured server's `ClientPcProfile` (`lab/decompiled/server/Gateway/Gateway.Login/ClientPcData.cs`)
carries `JobLevel`, `JobLevelXP` and `JobLevelTotalXP`, and the native client defines
`ClientUpdatePacketUpdateProfileRank` with two fields (constructor `FUN_0089e350`, length 0x12)
plus the `RankExperience` / `RankMaxExperience` tooltip fields. Together these are the fields a
restored server must fill for a rank display; they do not ship any threshold table.

## Inferred, and labelled as such

`EXPERIENCE_POINTS_REQUIRED` is read as the cumulative total needed to hold that rank row
(rank 1 requires 0, and the column name is the only shipped semantics), so `rank_for_xp` returns
the last row whose requirement is `<= xp`. `RankProgress::percent` is the floored share of the
current rank's span, and an unknown number of xp beyond rank 20 keeps accumulating with
`rank_max_experience` 0. The instance-to-property join (which of instance 58/63 is Wilds Farm
versus Briarwood Farm) is **not** established: instance ids and property ids are separate
namespaces and no preserved row maps them.

## Reconstructed boundary

Nothing numeric was invented. The typed module carries the recovered tables, the recovered gates
and two derived helpers (the cumulative rank lookup and the percentage). It locks nothing, spawns
nothing, grants no money, items, stars or xp, and it does not replace `crate::profiles` as the
profile store. The duplicate-grant guard (`GrantLedger`) only records which activity outcomes
already authorised an award: mining (001) authorises one award per finished
`Type8Run::instance_id`, harvesting (002) one per finished harvesting instance, and a replay is
answered with `GrantDecision::Duplicate` so the caller applies xp exactly once.
The mining-to-Miner binding for that key comes from the wiki `Stars` page ("the stars from Miner
quests and Mining minigames go straight to the Miner job"); the harvesting key takes the rewarded
profile from the caller because that minigame's required profile type is server content.

## Explicit gaps

* **Miner per-rank xp thresholds**: unrecovered. The client loads `FactoryRanks.txt` and the other
  `Resources` tables only — there is no profile/job rank table in the shipped data or the packed
  asset manifest — and the client reads `RankExperience`/`RankMaxExperience` from the server. A
  capture of a Miner rank-up (or the server's own profile rank data) is the missing evidence.
* **Farm instance to housing property mapping**: 58/63/60 are not joined to Wilds/Briarwood farms.
* **Reward-set contents** (4274..4289, 4381..4396), **item-cap semantics** (what the modifier caps)
  and the **xp granted per mining/harvesting outcome** are server content.
* **Non-member rank cap 4** and the Sunstone Valley level-20 raise are wiki statements; no client
  or captured-server row carries them (`NON_MEMBER_RANK_CAP` is exported as documentation only).
* The requirement operator column value 6 and the ranking semantics of the type-9 rows are the
  client's own factory names; no enum table names them beyond
  `CheckRankLastProfileOfTypeRank`.

## Verification

```
& ..\..\scripts\cargo.ps1 test --test piece_039
  RED  before implementation: couldn't find file `tests\..\src\restoration\piece_039.rs`
  GREEN: 16 passed; 0 failed
& ..\..\scripts\cargo.ps1 build --lib
  GREEN: Finished `dev` profile
& '..\lab\tools\wiki-env\Scripts\python.exe' 'tests\pieces\test_piece_039.py'
  RED  11 tests: checker script absent
  GREEN: 11 passed (Ran 11 tests, OK)
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_039.py' --output <report>\gathering-job-progression.json
  piece 039 evidence ok: farm instances [58, 63], ranch 60, 20 ranks, miner gates [5, 9, 13, 17]
```

The checker also rejected the first draft of this piece's own reward-set table
(`FarmRewardSetMismatch`, the missing rank-6 gap), which is the guard working as intended. Input
hashes are in `lab/reports/piece-039-20261009T040246Z/source-provenance.txt`; the checker output
`gathering-job-progression.json` carries every source sha256 it read.

## Next gates

1. Lead integration: register `pub mod piece_039;` under `src/restoration/` and drop the `#[path]`
   include, keeping `crate::profiles` as the single profile store.
2. Wire integration: gameplay (001/002 result handling) calls `GrantLedger::authorize` with
   `GrantKey::mining`/`GrantKey::harvesting` and applies the returned amount once; the farm track
   feeds the farm rank table from `FactoryRanks.txt` rather than the reconstructed 100-xp curve.
3. Native acceptance: with the farm rank table wired, raise a farm to rank 3/5 and confirm the
   client rank-up notification and the item cap; the Miner rank display still needs the missing
   threshold source.
4. Evidence review: obtain a Miner rank-up capture to replace the Miner gap, and a farm instance
   join to resolve 58/63 versus Wilds/Briarwood.

Cross-links: [[Mining Restoration]], [[Harvesting Restoration]], [[001 Mining original board]] if
present, [[../Design/MMO Worker Contract]], [[../../MMO Work Queue]].
