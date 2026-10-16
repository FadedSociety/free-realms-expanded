# 031 Job catalogue

Acceptance: recover every job/profile/localization/unlock mapping and state the historical coverage
explicitly. Work item [[notes/WorkItems/031.md]] (Job catalogue) under the
[[notes/Design/MMO Worker Contract.md]] classification rules.

Artifacts

- `prototype/osfr-tools/src/restoration/piece_031.rs` - runtime catalogue and locale validator.
- `prototype/osfr-tools/tests/piece_031.rs` - 8 acceptance tests (`#[path]` include until the lead
  registers the module).
- `scripts/pieces/piece_031.py` - evidence checker that builds and validates the catalogue.
- `tests/pieces/test_piece_031.py` - 7 checker tests, including locale drift and payload rejection.
- `../lab/reports/piece-031-20261009T035045Z/` - `job-catalogue.json`, test logs, RED evidence and
  provenance hashes.

## Original facts

The client job datasource is `BaseClient.Profiles`. `Profile_DS` and `Profile_Filtered_DS` in
`lab/decompiled/client-ui/ScriptsBase.lua` (lines 2844 and 2862) declare the same 16 columns in
order: `Id, Name, Description, Type, Rank, StarsAvailable, Icon, BadgeImageSet, ButtonImageSet,
AbilityBgImageSet, MembersOnly, StarsEarned, RankPercent, Hint, Trial, TrialExpired`. Every job row
the client renders comes from the server; the client never defines a job itself.

Captured private payloads (`lab/references/rust-opening/profiles.json`) hold exactly 16 profiles
with per-profile sha256 proofs and byte offsets:

| id | name | payload bytes | payload sha256 prefix |
| --- | --- | --- | --- |
| 1 | Adventurer | 579 | `43de6f628102` |
| 2 | Ninja | 655 | `3bb89626da1d` |
| 4 | Postman | 435 | `f87dd9a8a7d5` |
| 11 | Medic | 655 | `06f6d15a8e3f` |
| 12 | Wizard | 631 | `453b52ed5c04` |
| 14 | Miner | 495 | `a77bc3b5f7e3` |
| 16 | Blacksmith | 471 | `e110977c58b7` |
| 32 | Warrior | 643 | `537adb2a123f` |
| 35 | Archer | 619 | `cde6cffe080b` |
| 43 | Brawler | 667 | `95e0f8d00719` |
| 45 | Chef | 483 | `731969b3b07b` |
| 48 | Kart Driver | 363 | `7836ed8c1edd` |
| 49 | Demo Derby Driver | 363 | `183f1217da23` |
| 52 | Soccer Star | 423 | `0f1e76d76c21` |
| 120 | Card Duelist | 519 | `87a78f357f34` |
| 137 | Fisherman | 411 | `87d6da43f4e0` |

The same capture records the account state `active` 1 (Adventurer) with `unlocked` `[1, 43]`: the
captured character had Adventurer and Brawler only.

Activation is already implemented by the lead-owned production `src/profiles.rs`. The client
readers `FUN_008d86e0` (38/14 rank) and `FUN_008d8910` (38/18 rank) are noted there, and profile
activation uses opcode 38/21 with the captured payload. `src/profiles.rs` line 2 labels unlock
timing and the opening XP curve as reconstructed local rules, so this catalogue deliberately stops
short of XP and unlock policy.

## Localization mapping

Every profile `NameId`/`DescriptionId` in the community table resolves in the preserved original
locale export (85,932 rows, `lab/reports/job-catalogue-20261009T022023Z/locale-en_us.json`), and the
resolved text equals the catalogue name. Examples: 2836 Adventurer, 2278 Medic, 2873 Brawler,
2410 Card Duelist, 20178 Fisherman, 23268 Referee, 2301 Enforcer. A drifted name is rejected by
`validate_locale` and by the checker (`LocalizedNameDiffers`).

The locale also carries two original job rosters:

- ids 4419971-4420001 under `Reach Level 20 On all Jobs.` (4419970): 16 jobs - Adventurer, Archer,
  Blacksmith, Brawler, Card Duelist, Chef, Demo Driver, Fisherman, Kart Racer, Medic, Miner, Ninja,
  Postman, Soccer Star, Warrior, Wizard.
- ids 4420005-4420031 under `Complete the following job questlines.` (4420004): the same roster
  without Adventurer and Miner (14 entries).

The short forms `Demo Driver` and `Kart Racer` are achievement strings, not profile names; the
profile table uses `Demo Derby Driver` and `Kart Driver`.

## Unlock mapping

Recovered from the original client:

- `FRJobBrowser:PopulateJobList` renders `BaseClient.Profiles` rows, so the datasource itself is
  the unlock list the server sends.
- `JobBrowser.BrowserOnUserEvent` with `SelectJob` calls `JobScreen.SelectJob(id)` only when the id
  is not already active.
- `JobBrowser:StartJobQuest(profileID)` (ScriptsBase.lua line 8303) scans the ActivityFinder
  datasource for a row whose column 0 is the profile id and starts column 4 as a breadcrumb quest.
  That is the shape of the profile-id to unlock-quest mapping; the data is server-side.
- `GroupMemberData_DS.PlayerJobId`, `QuestData_DS.JobLevel` and `PlayerInspectDS.JobsUnlocked`
  confirm other systems consume the same profile ids.

Community documentation (wiki `Jobs` and per-job pages) records how jobs unlocked historically:
most jobs unlock by talking to a basic trainer, some (for example Postman) start from an
introduction object, and mini-game objects can start the unlock quest for an unowned job. The
checker stores the trainer lines it finds (Brawler: Ashley Lightwings in the retired Sacred Glade
tutorial, Carlos Brazenfist in Cobblestone Village, Harold in Highroad Junction, Caitlyn Gravefog
in Stillwater Crossing).

Not recovered, and named as the missing evidence: the original server's `BaseClient.Profiles` rows
and its `BaseClient.ActivityFinder` rows. No per-job unlock quest id is recovered, and none is
fabricated.

## Historical coverage

Wiki classification of the historic roster: Combat - Archer, Brawler, Medic, Ninja, Warrior,
Wizard; Crafting - Blacksmith, Chef, Miner; Freestyle - Adventurer, Card Duelist, Demolition Derby
Driver, Fisherman, Kart Driver, Pet Trainer, Postman, Soccer Star.

- 16 jobs have captured original payloads and community definitions, and are classed Combat (6),
Crafting (3) or Freestyle (7).
- 2 rows exist only in the community profile table: 58 Referee and 20 Enforcer (Staff, type 9 with
Miner). They have no captured payload and cannot be activated from captured bytes.
- 1 job is historical only: Pet Trainer (locale 17419, freestyle, removed 2009-12-14 per the wiki
page; still visible to pre-removal accounts). No profile id or payload survives.

## Contradictions kept open

- Membership: the wiki marks Chef, Kart Driver, Pet Trainer, Postman, Warrior and Wizard as members
  only; the community table sets `MembersOnly` false on every row and `Trial` true on nine. The
  client only reads `Trial`/`TrialExpired` in `ValidateJobForStartScreen`, so the exact original
  policy is not recovered. Both claims stay recorded.
- Naming: the locale achievement rosters use short job names that do not equal the profile names.
- Description reuse: ids 49 and 52 share description 2920, and Referee/Enforcer share 2302 in the
  community table. Recorded verbatim, not repaired.

## Verification

```
python scripts/pieces/piece_031.py --output ../lab/reports/piece-031-20261009T035045Z/job-catalogue.json
  piece 031 evidence ok: 18 jobs, 16 captured payloads, 1 removed rows
python tests/pieces/test_piece_031.py -v
  Ran 7 tests in 2.571s / OK
scripts/cargo.ps1 test --test piece_031
  test result: ok. 8 passed; 0 failed; 0 ignored
```

Test-first evidence: both suites were run before the implementation existed and failed for the
missing module/checker only (see `red-evidence.txt` in the report directory).

## Limits and next gates

- This is recovery metadata plus validation. It grants no job, defines no XP curve and claims no
  native activation.
- Profile item classes, starter kits and default items stay with the existing production
  `src/profiles.rs` (`starter_items` covers six kits) and the community `Profiles.json`
  `ItemClasses`/`DefaultItems` fields; this catalogue does not duplicate or change them.
- Independent review of the cross-check logic and a lead integration decision on registering
  `piece_031` and on which catalogue rows the gateway may reply with are still open.
- The catalogue can only become activation policy once the server-side profile rows or an explicit
  lead decision replace the community table.
