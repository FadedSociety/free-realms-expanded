# 032 Brawler progression

Status: scoped definitions and boundary validation implemented and locally verified, 2026-10-09
03:52 UTC. Evidence label: **ORIGINAL definitions read from preserved client tables, with INFERRED
comparator semantics and COMMUNITY level lists kept separate**. This is not a native acceptance
claim; no client has been run against this module.

Owned paths:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_032.rs` | profile / class / equipment / gate definitions and boundary validation |
| `prototype/osfr-tools/tests/piece_032.rs` | 15 hermetic vectors plus one ignored private lab gate, `#[path]` include |
| `scripts/pieces/piece_032.py` | independent Python re-derivation of the same boundaries |
| `tests/pieces/test_piece_032.py` | 7 stdlib checker tests, including five fail-closed cases |
| `lab/reports/piece-032-20261009T032529Z/` | RED/GREEN logs, checker output, exported document |

No shared `lib.rs`, gameplay, gateway, Cargo, launcher, content, save or release path was touched.
`src/profiles.rs` was read, not modified.

## Why this item exists

Work item 032 asks for the original skills/XP/level/equipment definitions of the Brawler job and for
the unlock and reward boundaries to be validated. The lead's `src/profiles.rs` already carries the
wire side of job progression: it writes `Rank` and `RankPercent` into `ClientPcProfile` and gates the
rank notification ahead of the profile update, with the XP curve explicitly marked reconstructed
(`rank = (1 + xp / 100).min(20)`). What was missing is the recovered *data* those packets must agree
with: which item classes a Brawler may wield, what rank each weapon unlocks at, and which rank gates
the client itself enforces.

## Original facts used

Everything below is read from the supplied client or its preserved community conversion. Nothing here
is authored.

- **Profile.** The Brawler profile row is `Id` 43, `NameId` 2873, `DescriptionId` 3234, `Type` 2
  (the value every combat job carries), `Icon` 1337, `Hint` 384195, `MembersOnly` false, `Trial`
  false, with 25 declared item classes and the seven-item creation kit
  `DefaultItems` `7676 7536 7495 7939 7571 7781 75030`
  (`lab/references/FreeRealms-Legacy/src/Resources/Profiles.json`).
- **Independent identity check.** Client `Resources/ItemFilterOptions.txt` row 11 is
  `11^2873^0^43^0^0^`: the locale name id 2873 ("Brawler") maps to profile id 43. The module asserts
  that row rather than trusting the conversion alone.
- **Wieldable item classes.** `Resources/ItemClasses.txt` (`ID^NAME_ID^ICON_ID^WIELD_TYPE^STAT_ID^
  PROFILE_NAME_ID`) carries profile name id 2873 on exactly five classes: **5** "Brawler 2-Handed
  Hammers" (wield type 13), **25** "Brawler Gear", **42** "Brawler Jewelry", **52** "Brawler Shards"
  and **71** "Brawler 1-Handed Hammers" (wield type 13). The other 20 classes the profile lists are
  borrowed from other profile name ids (18 of them, for example class 50 "All Combat Job Item Job"
  and class 69 "All Job Shards") or carry no owning profile at all (183, 999). Class names come from
  the unmodified locale export of the supplied client.
- **Equipment boundaries.** `ClientItemDefinitions.json` gives every row a `Class` and a
  `MinProfileRank`: the rank the item unlocks at, `0` meaning no rank requirement. Slot 7 is the
  weapon slot (`Resources/EquipmentSlotDefinitions.txt` row 7 `SLOT_NAME` `Weapon`).
- **Rank ceiling.** The largest `MinProfileRank` in the whole shipped catalogue (58,795 rows) is 20,
  and `EquipmentSlotDefinitions.txt` row 15 requires `MIN_PROFILE_RANK` 20 for the MoonRune slot. The
  module takes 20 from those tables; the wiki's "level 20" agrees but is not the source.
- **Rank gates.** `Resources/Requirements.txt` carries 24 `CheckRankLastProfileByType` rows
  (`TYPE` 3, factory name recovered from the client binary's
  `ClientRequirementCheckLastPlayedProfileOfTypeRank`). Eight of them are combat gates, combined by
  `RequirementGroups.txt` and `RequirementSets.txt` into six sets. The other 16 gate the crafting and
  gathering job types (Postman 3, Miner 9, Chef 10, Blacksmith 12) at ranks 5, 9, 13 and 17.
- **Traits.** The client locale export holds the Brawler trait name block 420938..420941 and the text
  block 420962..420965: "Bruising Strikes", "Savvy", (420940), "Resilience", with the recovered
  descriptions "All opponents you hit are bruised…", "Your critical strikes cause double damage.",
  "You are able to withstand more damage before you are knocked out." and "You show great resilience
  and gain health after being hit.". Id 420940 is **absent** from the export; the wiki names it
  "Toughness" and the sibling text 420964 covers it, so the gap is recorded, not filled.
- **Wire carrier.** `ClientPcProfile` (`Sanctuary.Packet.Common`, community model of the original
  packet) serialises `Rank`, `RankPercent`, `StarsAvailable`, `StarsEarned` and the ability set per
  profile; the supplied client's field-name table returns `"RankPercent"`, `"StarsAvailable"` and
  `"JobLevelsGained"` (`lab/decompiled/client/FreeRealms/pseudocode.c`).

## Inferred and community material

- **INFERRED:** requirement operator code 6 means `rank >= N` and code 4 means `rank <= N`. The
  shipping AND groups are the evidence: group 3 is `{req 3: >= 5, req 4: code 4 rank 12}` and group 4
  is `{req 5: >= 9, req 6: code 4 rank 16}`, which only describe a rank window if code 4 is an upper
  bound. The comparator implementation itself is not recovered from the binary, so the module keeps
  the operator codes as well as the reading, and the note does not claim the codes are proven.
- **COMMUNITY:** the read-only wiki (revision 48172) lists Brawler weapon levels
  `1, 3, 4, 5, 8, 11, 12, 13, 15, 16, 17, 19, 20`, clothing levels
  `1, 3, 5, 8, 11, 13, 15, 17, 19, 20`, equipment tiers up to level 20 and trait ranks
  `5, 10, 15, 20`. These arrive as caller input and are cross-checked, never trusted.

## What is validated

- **Equipment ranks are the original rows, not the wiki list.** Live weapon rows (slot 7, Brawler
  class, not a development row) occupy ranks
  `1, 3, 4, 5, 8, 11, 12, 13, 15, 16, 17, 19, 20` with 907 rows in total. That set is exactly the
  wiki's weapon level list, which is the cross-check this item ships.
- **Development rows are excluded from live boundaries.** 17 shipped weapon rows are development rows
  ("Level 2 Brawler Test Weapon", "Chris' Test Weapon", "Encounter 375 test weapon"). They occupy
  ranks 2, 10 and 18, where no live weapon exists, so those three ranks are reported as
  development-only instead of being attributed to live progression.
- **Rank 0 means no requirement.** Six weapon rows carry `MinProfileRank` 0 (for example 78197
  "Brawler's Power Fist") and are reported separately as always available rather than as a rank-0
  boundary.
- **The unlock curve is monotone.** `unlock_curve()` returns the cumulative count for ranks
  1..=20 and the vectors assert it never shrinks.
- **The creation kit is inside its own boundary.** All seven `DefaultItems` are class 25, rank 1,
  slots 1..7, and the module reports each entry with `class_wieldable` so a kit item outside the
  profile's classes would surface instead of shipping silently.
- **Out-of-range rows are refused, not clamped.** A Brawler weapon row above rank 20 becomes a
  reported violation and is kept out of the live buckets.
- **Gate windows are assembled, not averaged.** Each requirement set with combat gates becomes one
  `RankWindow` carrying the requirement ids of its bounds. A group that mixes profile types, an
  empty window (lower above upper) and a rank outside 1..=20 are all refused.
- **Another job's class is not this job's boundary.** Rows whose `Class` is not in the profile's
  class set are ignored rather than mis-attributed, which keeps the same table reusable for items
  033–037.

Recovered gate windows for combat (type 2), by requirement set:

| Set | Bounds | Requirements |
| --- | --- | --- |
| 1 | `rank >= 10` | 1 |
| 2 | `rank <= 8` | 2 |
| 3 | `5 <= rank <= 12` | 3, 4 |
| 4 | `9 <= rank <= 16` | 5, 6 |
| 5 | `rank >= 13` | 7 |
| 6 | `rank >= 17` | 8 |

Sets 3 and 4 overlap. That is what the shipped groups say, so it is reported rather than smoothed;
the content that consumes those sets is server-side and is not recovered here.

The wiki's Brawler trainer quests are named "Achieve Level 5", "Achieve Level 10", "Achieve Level 15"
and "Achieve Level 20". Only the first two line up with a recovered lower bound (5 in set 3, 10 in
set 1); 15 and 20 do not match the recovered bounds 13 and 17. Since the quest definitions themselves
are server content, that mismatch is recorded as an open question for the lead instead of being
resolved by guessing which side is wrong.

## What is not recovered

1. **Per-rank experience thresholds.** No preserved table carries them. The client receives `Rank`
   and `RankPercent`; `src/profiles.rs` reconstructs a 100-XP curve and marks it reconstructed. This
   item does not add, change or endorse a curve, and `experience_gaps()` names the missing evidence.
2. **Reward amounts.** Brawler quest XP and coin rewards live in server quest/reward content that is
   not among the preserved inputs. The only original trace is the reward notification string family
   ("You receive #count([*experience*]) and #count([*coins*])", locale 3).
3. **Comparator proof.** Operator codes 6 and 4 are read as `>=` and `<=` by inference; the binary's
   comparison routine is not recovered.
4. **Trait rank mapping.** The trait *ids and text* are original; the rank each trait unlocks at is
   only the community list. The module keeps `BRAWLER_TRAIT_RANKS` in a constant that documents its
   community provenance.
5. **Locale text for 420940.** Absent from the export; recorded as a gap.

## Verification

Commands run from `prototype/osfr-tools` with `scripts/cargo.ps1` (private toolchain under
`../lab/tools`) and from the project root with `../lab/tools/wiki-env/Scripts/python.exe`.

| Command | Result | Log |
| --- | --- | --- |
| `cargo.ps1 test --test piece_032` (first revision: every row treated as a live boundary) | 11 passed, 4 failed - development rows counted as live ranks, rank-21 row accepted, cross-check blind to development-only ranks | `red-hermetic.log` |
| `cargo.ps1 test --test piece_032 -- --ignored` (same revision, preserved tables) | failed at `item_classes.len()` 25 vs 24 - a real recovery error the lab gate caught | `private-gate-run1.log`, `private-gate-run2.log` |
| `cargo.ps1 test --test piece_032` | 15 passed, 0 failed, 1 ignored, no warnings | `green-hermetic.log` |
| `cargo.ps1 test --test piece_032 -- --ignored --nocapture` | 1 passed, 0 failed against the preserved tables | `private-gate-green.log` |
| `cargo.ps1 test --lib` | 24 passed, 0 failed | `lib-suite.log` |
| `cargo.ps1 test --test profiles --test inventory` | 6 (+1 ignored) and 12 (+2 ignored) passed | `neighbour-suite.log` |
| `python scripts/pieces/piece_032.py …` | exit 0, `live ranks 13, gates 24, declared missing [], unexpected []` | `independent-checker.log`, `brawler-progression.json` |
| `python -m unittest discover -s tests/pieces -p "test_piece_032.py" -v` | 7 passed | `python-tests.log` |

The ignored private gate asserts the concrete recovered numbers: 25 wieldable classes with 5 owned,
13 live weapon ranks and 907 live rows, 17 development rows, 6 always-available rows, 447 rank-1 and
11 rank-20 live weapons, the seven-slot creation kit, 24 gate rows, six combat windows, and the
absence of any `MinProfileRank` above 20 in the shipped catalogue.

The independent checker is a second implementation in another language. It re-reads the raw tables
(including `ItemFilterOptions.txt` and the locale export), re-derives the same boundaries, records a
sha256 for every input, and exits 2 with a named check when a claim is unsupported — the python tests
exercise five of those refusals (changed profile field, missing class row, rank-21 item,
uninterpretable operator, mixed gate group).

## Deliberate gaps and remaining gates

1. **Lead integration.** Register `pub mod restoration::piece_032;` (or move the module) in
   `src/lib.rs` and drop the `#[path]` include from the owned test file. The module has no
   dependencies beyond `serde` and `std`.
2. **Wire use.** Nothing here answers a client packet. `src/profiles.rs` owns `Rank` / `RankPercent`
   and the 38/14 + 38/18 packets; the intended seam is that its curve stays reconstructed while this
   module supplies the ceiling, the equip boundaries and the gate windows.
3. **Server-side gates.** Which quest or item consumes requirement sets 1–6 is server content and is
   not recovered, so the module reports the windows without binding them to content.
4. **Native acceptance.** Equipping a rank-gated weapon and watching the job window in the supplied
   client is the only gate that proves the reconstruction end to end; it belongs to the lead's native
   session, not to this module.

Cross-links: [[../../MMO Work Queue]], [[../../Design/MMO Worker Contract]].
