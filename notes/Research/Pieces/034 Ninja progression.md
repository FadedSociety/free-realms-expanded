# 034 Ninja progression

Status: recovered job/equipment/cooldown data plus a bounded server validation, locally verified
2026-10-09 03:41 UTC. Evidence labels used below: **ORIGINAL** (read out of the preserved client,
the pinned extracted resources or the revision-pinned wiki), **COMMUNITY-RECOVERED** (pinned
community reconstruction), **RECONSTRUCTED** (local policy). No native client has accepted any
packet this item produces.

Owned paths:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_034.rs` | catalogue validation, equipment gate, toolbar, recast/energy validation, denial payloads |
| `prototype/osfr-tools/tests/piece_034.rs` | 15 owned vectors, `#[path]` include, RED/GREEN recast evidence |
| `scripts/pieces/piece_034.py` | extractor and independent checker over the pinned corpus |
| `tests/pieces/test_piece_034.py` | 15 stdlib parser/checker tests |
| `lab/reports/piece-034-20261009T034148Z/` | emitted catalogue, RED/GREEN logs, probes |

No shared `lib.rs`, gameplay, gateway, Cargo, launcher, content, save or release path was touched.

## Why this item exists

The local sandbox (`src/combat.rs`, `src/encounter.rs`) answers ability opcode 36/10 only for a
single hard-coded basic slot and drops everything else, and the pinned community gateway answers
every `AbilityPacketClientRequestStartAbility` with one blanket
`AbilityPacketFailed { StringId = 3079 }` (`Sanctuary.Gateway/Handlers/BaseAbilityPacket/
AbilityPacketClientRequestStartAbilityHandler.cs`). Ninja had no recovered job row, no equipment
gate, no toolbar mapping and no recast rule in the project. This item supplies those, bounded to
what the preserved sources actually state.

## ORIGINAL facts used

Sources are hashed by the extractor; the hashes are in the emitted document and repeated in
`lab/reports/piece-034-20261009T034148Z/ninja-progression.json` under `sources`.

**Job row** (`lab/references/Sanctuary/src/Resources/Profiles.json`, sha256 `83c7af56…`; the same
row exists in the older pinned `FreeRealms-Legacy` copy):

* profile **2** = "Ninja", `NameId` 3100, `DescriptionId` 3101, `Type` 2, `Icon` 1342,
  `AbilityBgImageSet` 3, `BadgeImageSet` 10, `ButtonImageSet` 1, `Hint` 384179.
* the row carries `MembersOnly: false`, `Trial: true`. The wiki revision 48173 states the stricter
  combat rule: "this job is locked for MEMBERS ONLY", free to sample "through level 4" as of
  2009-12-05. The document keeps both flags (`members_only`, `trial`, `combat_members_only`) rather
  than folding them.
* `DefaultItems` = `[19522, 19431, 19197, 19787, 19253, 75091]` — the rank 1 creation kit:
  Student Ninja Cowl / Armwraps / Gi / Hakama, Beginner Ninja Tabi, and Ninja's Training Sword of
  1000 Storms. Every row is `Class` 2 (Ninja) with `MinProfileRank` 1.

**Item classes** (`ItemClasses.txt`, sha256 `73c2a937…`; rows are
`ID^NAME_ID^ICON_ID^WIELD_TYPE^STAT_ID^PROFILE_NAME_ID`):

* Ninja-owned classes: 2 "Ninja Gear" (wield 0), 6 "Ninja Swords" (wield 9), 29 "Ninja Run Test"
  (9), 30 "Ninja Kamas" (9), 44 "Ninja Jewelry" (9), 54 "Ninja Shards" (9).
* shared classes the profile also lists: 50 "All Combat Job Item Job" (profile scope 17372 "Any
  Combat Job"), 69 "All Job Shards" (scope 17373 "Any Job"), plus cosmetic/event scopes
  (144 Werewolf, 145/146/150/162 Freestyle, 153 Crackshot, 154 Magetech, 155 Wildclaw, 156
  Ghostshadow, 158 Parachute, 161/173 "All Jobs", 168 Consumable, 179 "Ninja and Freestyle Jobs").
* class 999 is declared by every profile row but its `ItemClasses.txt` row is `999^0^0^0^0^0` with
  no name id; the document records it as an unresolved class instead of inventing a name.
* Ninja's wield type from `CombatJobs.json` is 9, matching the Ninja weapon classes.

**Equipment ladder** (`ClientItemDefinitions.json`, sha256 `ffe9cd2d…`): 71 named Ninja weapons
(excluding a TEST row and one unnamed row 55018). The named family 75090-75119 is 30 weapons that
gate on `MinProfileRank` **1 / 5 / 8 / 12 / 16** with `PowerRating` **1 / 2 / 3 / 4 / 5**; the coin
costs 750 / 850 / 1055 / 1504 / 2152 match the wiki pages. Promotion and event weapons (for example
the rank 1 power 5 "Dragon Blade" rows) break a strict rank/power ordering, so the extractor keeps
the full tier table and asserts only the canonical five-step ladder.

**Ability damage ladder** (`lab/references/freerealms-wiki/wiki.sqlite`, sha256 `b9d7b9f1…`): the
1000 Storms weapon family pages state, per tier:

| Tier | Weapon (item id) | Lightning Strike | 1000 Storms | Page revision |
| --- | --- | --- | --- | --- |
| Power 1, level 1 | Ninja's Training Sword of 1000 Storms (75091) | 254 | 889 | 32898 |
| Power 2 | Ninja's Blade of 1000 Storms (75093) | 488 | 1554 | 47120 |
| Power 3, level 8 | Ninja's Scythe of 1000 Storms (75097) | 853 | 2716 | 34379 |
| Power 4, level 12 | Ninja's Jagged Scythe of 1000 Storms (75103) | 1357 | 4750 | 32873 |
| Power 5, level 16 | Ninja's Shadow Blade of 1000 Storms (75111) | 2372 | 8302 | 32888 |

The Ninja page (revision 48173) adds the trait ladder (level 5 Ultimate Power, 10 Shrouded Armor,
15 Ninja's Grace, 20 Dragon's Boon) and the equipment tier lists; the "Ninja Power Attacks" page
(revision 47692) carries the Power 5 comparison table with 21 power attacks.

**One recorded disagreement:** the wiki page for 75093 says "level 4" while the shipped item
definition carries `MinProfileRank` 5. The module gates on the item definition and the document
records the page verbatim in `level_discrepancies`; the checker fails if such a disagreement ever
loses its record.

**Ability wire and denial phrases** (all ORIGINAL except the last):

* `lab/decompiled/server/LibSOE/SOE/BaseAbilityPackets.cs` — opcode 36 with `AbilityPacketFailed`
  1, `AbilityPacketStartCasting` 3, `AbilityPacketClientRequestStartAbility` 10.
* `lab/reports/locale-id-to-text.json` (sha256 `f1f22a30…`) — 285 "You don't have enough energy for
  that ability!", 287 "That ability isn't ready yet...", 288 "You need to choose a target!".
* `lab/decompiled/client/FreeRealms/pseudocode.c` — the ordered cast-precondition ladder
  (`NoCastKnockedOut`, `NoCastAsleep`, `NoCastSilenced`, `NoCastAfraid`, `NoCastStunned`,
  `NoCastFrozen`, `NoCastKnockedBack`, `NoCastPoppedUp`, `NoCastDefault`, `NoCastInsufficientMana`,
  `NoCastNotEnabled`, `NoCastRecastTimer`) and the ability-definition listener fields
  (`ManaCost`, `CastSeconds`, `AuraDuration`, `Description`, `ManaCostPerSecond`, `MaxAoeTargets`).
  There is no recast field in the definition and no per-ability recast value anywhere in the
  preserved client, which is why the cooldown duration stays an explicit gap.
* `lab/extracted/client-assets/catalog.tsv` — 25 `icon_abil_ninja_*` names, including `shuriken`,
  `shuriken_storm`, `smokebomb`, `counter`, `dragon_strike`, `shadow_minion`, `thousand_storms`.
* COMMUNITY: the pinned gateway's blanket refusal string id 3079 "You can't use that ability right
  now." is what the extracted client prints when the server refuses an ability request.

## COMMUNITY-RECOVERED rows

`CombatJobs.json` (sha256 `2acec676…`) profile 2 and `CombatAbilities.json` (sha256 `ecf3d2d6…`):
reaches **4** (basic) / **7** (special), basic recast **600 ms**, energy pool **100** regenerating
**4 per second**, bare-hand fallback basic **2010**, and the level-1 weapon 75091 mapping to basic
**2154** Lightning Strike (254) and special **2050** 1000 Storms (889, cost 100, animation 1033,
cast effect 16088). The 254/889 pair is independently corroborated by the wiki weapon page, and
2154/2050 are corroborated by the wallet of ability ids in the same community table; the slot
assignment (basic first, special second) follows the community toolbar model
(`Sanctuary.Game/Entities/Player.cs`) and is labelled as reconstruction in the module.

Only this one weapon mapping is recovered. Every other Ninja weapon — including the higher tiers
of the same 1000 Storms family — resolves to the bare-hand fallback and an empty special slot
rather than to guessed ability ids.

## What is reconstructed

* The validation order and the denial mapping. The module follows the recovered client ladder for
  the *last* two checks (energy, then recast) but the ladder itself is the client's, not a
  recovered server implementation.
* Recast bookkeeping: the last accepted cast timestamp per ability, compared against the recorded
  cooldown for that ability only. An ability without a recovered cooldown is never given an
  invented timer; its cast reports `cooldown_ms: None` and `cooldown_source: "unrecovered"`.
* Energy bookkeeping (whole-second regeneration to the recovered maximum, saturating) and the
  charge-once rule.
* Packet writers: `AbilityPacketFailed` = 36/1 + string id, `AbilityPacketStartCasting` = 36/3 +
  caster guid, target guid, cast effect, animation, ability id, action time seconds, progress flag.
  The `StartCasting` field order is the pinned community reconstruction; `action_time_ms` is a
  caller input because cast time is not recovered.
* Refusals mutate nothing: every check runs before the first write, which the vectors assert.

## Verification

Commands run from `prototype/osfr-tools` with the project `scripts/cargo.ps1` (private toolchain)
and from the project root with `../lab/tools/wiki-env/Scripts/python.exe`.

| Command | Result | Log |
| --- | --- | --- |
| `cargo.ps1 test --test piece_034` (first revision, recast recorded but not enforced) | 12 passed, 3 failed - the recast window returned `Ok` for a second cast | `red-recast.log` |
| `cargo.ps1 test --test piece_034` | 15 passed, 0 failed | `green-recast.log` |
| `cargo.ps1 test --test piece_034` with `OSFR_PIECE_034_CATALOGUE` = the emitted document | 15 passed, 0 failed; this run caught that only 75091 has a recovered mapping | `green-final.log` |
| `cargo.ps1 test --lib` | 24 passed, 0 failed | `lib-suite.log` |
| `cargo.ps1 test --test combat --test encounter --test opening_abilities` | 5 (+1 ignored) / 15 (+2 ignored) / 4 (+1 ignored) passed | `neighbour-suite.log` |
| `cargo.ps1 test` (crate wide) | did not complete: the concurrently written `tests/piece_045.rs` (another worker's path) fails to compile with "`Inventory` doesn't implement `Debug`", so cargo stops before this target; unrelated to this item | `full-suite.log` |
| `python scripts/pieces/piece_034.py --out lab/reports/piece-034-20261009T034148Z` | exit 0; 71 weapons, 6 tiers, 3 abilities, 10 refusals, 25 icons, 7 gaps, 1 recorded discrepancy | `extract.log` |
| `python scripts/pieces/piece_034.py --check .../ninja-progression.json` | exit 0, no problems | `extract.log` |
| `python -m unittest discover -s tests/pieces -p "test_piece_034.py" -v` | 15 passed | `python-tests.log` |

## Deliberate gaps and remaining gates

1. **Lead integration.** Register `pub mod piece_034;` under `src/restoration/` (or `lib.rs`) and
   replace the `#[path]` include in the owned test. The controller needs a caller: today
   `src/combat.rs` and `src/encounter.rs` still answer 36/10 themselves, so the validator is only
   exercised by its own vectors until the lead routes the request.
2. **Cooldown durations.** No original recast value is recovered for any Ninja ability. Only the
   community 600 ms basic recast is applied, and the special is deliberately left without one.
   The exact missing evidence is an original ability definition payload (36/13) or a captured
   server cast answer; until then, per-ability recasts stay unrecovered.
3. **Energy numbers.** The 100/4 pool is community data. The client phrase id exists, so the
   refusal path is original, but the pool size is not.
4. **Rank progression.** The equipment gates use `MinProfileRank`; no rank experience curve,
   trait effect or reward boundary is recovered. `set_rank` is the seam for item 039/040 style
   progression work.
5. **Ability ids above tier 1.** Only 75091's pair is recovered. The wiki names the higher-tier
   pairs (Dragonstrike, Shuriken Storm, Flame Wave, Shadow Armies, Solar Flare, Mysticism, Soul
   Power, Deception) but no ability ids; the module therefore falls back instead of guessing.
6. **Targeting and geometry.** "In front" and "nearby" are recovered text, not geometry. Reach,
   cone, line of sight, hit timing and damage application stay with the caller; item 036's Wizard
   slice and item 054's melee rule are the adjacent owners.
7. **Native acceptance.** No client has accepted the denial or start-casting payloads. The lead's
   native gate needs a real client request, which requires the routing work in gate 1.
8. **Cross-item finding.** `src/restoration/piece_036.rs` (Wizard) states that no denial string id
   is recovered; this item recovered ids 280-288 plus 3079 from the locale export and the client
   ladder. The lead can hand that set to the Wizard slice rather than leaving its
   `AbilityPacketFailed` payloads unfabricated for lack of a phrase id.

Cross-links: [[../../MMO Work Queue]], [[../../Design/MMO Worker Contract]],
[[../Free Realms Recovery]].
