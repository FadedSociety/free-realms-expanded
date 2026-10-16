# 042 Soccer and postman progression

## Scope

Recovered the Soccer Star and Postman job progression: job identity, the level milestone quest
families, the Postman training gates, both unlock routes, the Soccer tutorial definitions, the
Postman mail tutorials and the activity coverage. This is a recovery and validation deliverable. It
grants no job, item, currency or experience, and it does not claim a playable native flow.

Two helpers carry real behaviour, both test-first:

- `soccer_tutorial_advance(page, action_column)` reproduces the client tutorial advance rule.
- `unlocked_level_gates(level)` / `level_gate(level)` / `milestone_reached(level, row)` expose the
  recovered level gates.

## Evidence

| Source | Path | sha256 |
| --- | --- | --- |
| Original locale export (85,932 rows) | `lab/reports/job-catalogue-20261009T022023Z/locale-en_us.json` | see sources block of `piece-042-evidence.json` |
| Client string mapping | `lab/working/OSFR.Client/OSFR Client/Resources/CodeStringMappings.txt` | see evidence JSON |
| Decompiled client Lua | `lab/decompiled/client-ui/ScriptsBase.lua` | see evidence JSON |
| Community profile rows | `lab/references/Sanctuary/src/Resources/Profiles.json` | see evidence JSON |
| Community item table | `lab/references/Sanctuary/src/Resources/ClientItemDefinitions.json` | see evidence JSON |
| Community item classes | `lab/references/Sanctuary/src/Resources/ItemClasses.txt` | see evidence JSON |
| Extracted minigame table | `lab/reports/minigame-content-inventory.json` | see evidence JSON |
| Community POI table | `lab/references/Sanctuary/src/Resources/PointOfInterests.json` | see evidence JSON |
| Read-only wiki corpus | `lab/references/freerealms-wiki/wiki.sqlite` | see evidence JSON |

Every hash above is recomputed by `scripts/pieces/piece_042.py` into
`lab/reports/piece-042-20261009T0352Z/piece-042-evidence.json`.

## Original facts

Locale rows quoted verbatim by the module and re-checked on every run:

- `Postman` at 3237 and its description placeholder at 3238; `Soccer Star` at 4650.
- Hints: 384184 "Felipe needs to train more postmen to help out at Pixiewood." and 416471 "South of
  Snowhill, Manager Landon is helping teams recruit new soccer stars."
- Level milestones: `Postman: Level 4/8/12/16/20` (34115-34119) and `Soccer Star: Level
  4/8/12/16/20` (34120-34124) with the "Reach Level N as a ..." objectives at 34195-34199 and
  34200-34204.
- Postman training gates: 387931/387935/387939/387943 (Achieve Level 5/10/15/20) with objectives
  387932/387936/387940/387944 that name the trainers Felipe, Enve LeStamp, Postchug Runalug and
  Brin Galeglow, plus the gate and success dialogs 387930/387933, 387934/387937, 387938/387941 and
  387942/387945.
- Soccer tutorial: `CodeStringMappings.txt` resolves `SoccerTut0001..0007` to 406669-406674 and
  416748 and `SoccerTutIcon0001/0002/0004` to 406675/406676/406678. The recovered text covers the
  basics, the kick, the super kick and charge meter, the Alt/right-mouse pass, the Control dodge,
  the slide tackle and steal, and the power-up page that ends the tutorial.
- Postman mail sorting tutorial: `MailSortingTutorialOneTop/Bottom` resolve to 19323/19302.
- Client tutorial flow: `DS_SOCCER_TUTORIAL = 25` selects
  `BaseClient.SoccerProcessor.SoccerPlayerActions`, `pageCount = 7`, and
  `SoccerTutorial.OnDataChanged` advances only on these column/value pairs: (page 1, column 0) -> 2,
  (2, 2) -> 3, (3, 3) -> 4, (4, 6 or 7) -> 5, (5, 1) -> 6, (6, 4 or 5) -> 7. The tutorial loads
  images 20999/20998/20248 and spawns the pickups 11542 (Charge Rate), 17353 (Speed Boost) and
  18855 (Toughness).
- Minigame tutorial contract: `Ui.GameTutorialGetInfo(tutorialId)`, `Ui.GameTutorialShowAgain`,
  `MiniGameFlash:TutorialClosed(id, showAgain)` and `Pet.TutorialEnd(id, showAgain)` - the tutorial
  id plus a persisted "show again" flag.
- Activities: the extracted table holds the type-12 rows 42 `Sacred Grove Soccer`, 43 `Snowhill
  Soccer` and 44 `Briarwood Soccer`, and the Postman mail rows 3/5/6/12/13/14/16/24/238.
  `Can You Deliver?` is row 238 (name 93381, description 93382).

## Community and wiki facts

Community tables (pinned `Sanctuary` mirrors, not original assets): profile 4 (type 3, icon 1339,
trial, hint 384184) with the `Student Postman *` kit 13137/12827/13197/13317/12957 in owned class 3;
profile 52 (type 8, icon 20743, trial, hint 416471) with the `Fast Forward *` kit
61907/61353/62169/61609/62473 in owned class 134. Class rows: 3 -> name 2923 `Postman Gear` owned by
3237; 134 -> name 3922 `Soccer Star Gear` owned by 4650. Point of interest 117 `Snowhill Soccer
Field` (name 407110, location 207771, position -145.95377/-6.12704/259.3836, heading -3.01945)
carries community breadcrumb quest id 1881.

Wiki (`wiki.sqlite`, read-only): `Postman` is a freestyle delivery job unlocked by talking to Felipe
in Sanctuary, optionally introduced by interacting with town postboxes ("Postman: It's all in the
Training"); its trainers are Felipe, Flitter, Postmaster Vixie, Postchug Runalug, Loryn and Brin
Galeglow; its abilities are Mail Throw and Toss Bone; the page lists 30 quest entries. `Soccer Star`
is an athlete job whose stars come from job quests and soccer minigames; it is unlocked by
interacting with the icon at the Snowhill Soccer Field entrance and finishing the soccer tutorial;
it was introduced in the Goal Time update on 2009-10-02; the page lists Kick Power, Tackle Power and
Toughness as its stats. `Jobs` documents the level-20 cap, the level-4 non-member cap, the
object-driven unlock route and the matcher stats shared by Blacksmith, Miner and Postman.

## Experience contract and the XP gap

The client never derives progress: `Profile_DS.Rank`, `StarsAvailable`, `StarsEarned` and
`RankPercent` are server-reported columns that the job window renders directly, and the client
contains no experience table. The minigame table carries no reward amounts and the wiki records no
XP thresholds. The numeric XP curve is therefore an explicit gap
(`xp.numeric_curve_recovered = false` in the evidence JSON, `XP_CURVE_RECOVERED = false` in the
module). The checker fails if a future source or a module edit claims a curve without new evidence.

## Unlocks

- Postman: Felipe in Sanctuary; the optional introduction is triggered by town postboxes. The
  unlock minigame is `Can You Deliver?` (row 238) with the objectives "Deliver the mail and earn the
  postman job!", "Complete the Can You Deliver? minigame", "Click the mailboxes to deliver the
  mail." and "Return to Felipe in Sanctuary", and the grant lines 9737/103830/103831 ("Welcome,
  postman! Here's a starting uniform.").
- Soccer Star: the Snowhill Soccer Field entrance icon leads to the tutorial and the job; the
  community POI records breadcrumb quest id 1881.
- Gap: neither job has a recovered unlock**quest id** (`unlock_quest_ids_recovered = false`), and the
  community breadcrumb id has no matching row in any recovered quest corpus. Quest ids for the
  recovered Soccer Star questline names (407407-407418) are missing as well.

## Implementation

`prototype/osfr-tools/src/restoration/piece_042.rs` (test-first, `#[path]` included by
`prototype/osfr-tools/tests/piece_042.rs` until the lead registers the module):

- Recovered tables: `TEXT_CLAIMS`, `REFERENCE_IDS`, `POSTMAN_LEVEL_MILESTONES`,
  `SOCCER_STAR_LEVEL_MILESTONES`, `POSTMAN_LEVEL_GATES`, `POSTMAN_SKILLS`, both creation kits,
  `SOCCER_STAR_JOB_QUESTLINE`, `SOCCER_TUTORIAL_PAGES`, `SOCCER_TUTORIAL_PICKUP_ICONS`,
  `POSTMAN_MAIL_SORTING_TUTORIAL`, `SOCCER_TUTORIAL_ADVANCE`, the POI and activity ids.
- Behaviour: `soccer_tutorial_advance`, `level_gate`, `unlocked_level_gates`, `milestone_reached`,
  `job_progression`, `validate`, `validate_locale`.
- Labels in `TutorialPage::label` are recovery labels for readability; the original text is the
  locale row behind `string_id`.

`scripts/pieces/piece_042.py` re-reads every pinned source, cross-validates the Rust module against
it, and fails closed with a named check. It verifies the locale text claims and references, the
client string mapping, the client tutorial flow, the `Profile_DS` columns, the profile rows, the
creation kits, the item classes, the minigame rows and the POI row.

## Checks

| Check | Command | Result |
| --- | --- | --- |
| Rust RED | `scripts/cargo.ps1 test --test piece_042 --manifest-path prototype/osfr-tools/Cargo.toml` before the module existed | failed to compile: `couldn't find file tests\..\src\restoration\piece_042.rs` - `lab/reports/piece-042-20261009T0352Z/red-test-rust.log` |
| Rust GREEN | same command | `8 passed; 0 failed; 0 ignored`, exit 0 - `green-test-rust.log`, repeated after `rustfmt` as `green-test-rust-postfmt.log` |
| Python RED | `lab/tools/wiki-env/Scripts/python.exe tests/pieces/test_piece_042.py -v` before the checker existed | `Ran 9 tests ... FAILED (failures=9)` - `red-test-python.log` |
| Python GREEN | same command | `Ran 9 tests ... OK` - `green-test-python.log`, repeated after cleanup as `green-test-python-postfmt.log` |
| Evidence document | `lab/tools/wiki-env/Scripts/python.exe scripts/pieces/piece_042.py --output lab/reports/piece-042-20261009T0352Z/piece-042-evidence.json` | `piece 042 evidence ok: 2 jobs, 10 milestones, 7 tutorial pages, 7 gaps` - `checker-run.log` |
| Format | `rustfmt --edition 2024 src/restoration/piece_042.rs tests/piece_042.rs` | clean |

Python tamper cases in the suite: a drifted locale name (4650), a rewritten client tutorial
transition, a removed `SoccerTut0005` mapping, a module that claims a numeric XP curve, and a
missing wiki corpus. Each must exit 2 with its named check.

## Artifacts

| Artifact | sha256 |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_042.rs` | `69795dec79dbcc1b607f85f532887ac35f00fe40846e8646b9c1866a1c8ad1fa` |
| `prototype/osfr-tools/tests/piece_042.rs` | `10c1cd7fcd7049dcda8d3b0ea8c331364c45efdfd117d051d9791f658011e694` |
| `scripts/pieces/piece_042.py` | `a3868e1657b2affaee2c8c7740ccdd53b5eb90741e031f6c412f89eb5c43494b` |
| `tests/pieces/test_piece_042.py` | `e1b31cf604a2d66cd683776807ec5e4c4719aa2ed5365e47f606972c37235c01` |
| `lab/reports/piece-042-20261009T0352Z/piece-042-evidence.json` | `8d2650f336cfbd15dae6686a77990ccb4057955d48c786b545873d358ae3fece` |

Hashes are of the final revision of each file; `lab/reports/piece-042-20261009T0352Z/provenance.txt`
records the same list plus this note. Recompute with `Get-FileHash` before relying on them.

## Limitations and next acceptance

- The XP curve, the unlock quest ids and the Soccer tutorial's action column meaning remain
  missing; nothing here invents them. A future capture of the server's job rows or the job quest
  rows would close the unlock and XP gaps.
- Profile 52 reuses the Demo Derby description id 2920, so the Soccer Star blurb is not recovered;
  the wiki blurb is documented separately.
- The wiki's stat list (Kick Power, Tackle Power, Toughness) conflicts with the client tutorial
  pickups (Charge Rate, Speed Boost, Tackle Power, Toughness); both are recorded unresolved.
- The tutorial advance rule is wired into no client or gateway path, and no client run was made for
  either job. Native acceptance stays with the lead.
- No shared file, gateway, save, content or release path was touched.
