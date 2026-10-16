# 033 Warrior progression

Status: bounded recovery plus checker returned. Item 033 acceptance is "recover
job/skill definitions; ownership and level restrictions, not copied Brawler
assumptions". This piece recovers the Warrior profile definition, the four
Warrior-owned item classes, the creation kit, the recovered level gates, the
skill definitions and the reconstructed equip/skill policy over them. Native
equip, ability-bar and combat acceptance remain separate gates.

## Recovered original facts

Profile 32 in the pinned Legacy `Profiles.json` (revision
`9e082025391177d5a33ee18c714ddd06b457a695`, AGPL-3.0, inspected as facts; no
upstream source copied into this repository) is `Warrior`, `NameId 4664`,
`DescriptionId 4665`, `Type 2`, icon `21594`, `BadgeImageSet 10`,
`ButtonImageSet 1`, `AbilityBgImageSet 31`, `Hint 384191`, `MembersOnly false`,
`Trial true`. The preserved locale map resolves 4664 to "Warrior", 4665 to
"Warriors are hardy Brawlers that can take and dish out damage." and 384191 to
"A true warrior does not fear harsh weather.".

The prepared profile packet `references/rust-opening/profiles.json` id 32 is
643 bytes, sha256
`537adb2a123f07673ddf994f507f6e34654212e68ce91223276c691f9a9a3408`. Its first
eight int32 values are exactly the header above. Its equipment section at
offset 339, length 112, decodes as a count of 9 followed by (slot,
unidentified, slot) triples for slots `[1,2,3,4,5,6,7,8,12]`, matching the
prepared spool in `references/rust-opening/inventory.json`. The triple middle
value and the rank section at offset 319 are recorded but not interpreted.

Class ownership from the pinned `ItemClasses.txt`
`ID^NAME_ID^ICON_ID^WIELD_TYPE^STAT_ID^PROFILE_NAME_ID`:

| class | locale name | owner name id | wield type | stat id |
|---|---|---|---|---|
| 15 | Warrior Gear | 4664 | 0 | 16 |
| 17 | Warrior Axes & Hammers | 4664 | 12 | 19 |
| 45 | Warrior Jewelry | 4664 | 12 | 0 |
| 55 | Warrior Shards | 4664 | 12 | 0 |

The Brawler rows are distinct: class 25 `Brawler Gear` and class 71 `Brawler
1-Handed Hammers` are owned by name id 2873 (profile 43). Warrior's recovered
class allowlist is `[15,17,45,50,55,69,144,145,146,150,151,153,154,155,156,
158,161,162,168,173,183,999]`; it contains no Brawler class, and the Brawler
default weapon `75030` "Brawler's Mallet of Sweeps" (`Class 25`) is not part of
the Warrior slice.

The recovered `DefaultItems` of profile 32 resolve to the seven-piece creation
kit: `46644` Student Warrior Helm (slot 1), `46389` Gauntlets (2), `46137`
Tunic (3), `46732` Breeches (4), `46224` Hightops (5), `47019` Pauldrons (6)
and `29963` Student Warrior Cudgel (class 17, slot 7). All carry
`MinProfileRank 1`.

Level restrictions are recovered from `MinProfileRank`:

- The `Level N Warrior Test Chest` family `70054,70060,70066,70072,70078,
  70084,70090,70096,70102,70108,70114` carries the level it tests: 2, 5, 8,
  10, 13, 15, 16, 17, 18, 19, 20.
- Distinct gates across `Warrior Axes & Hammers` records are
  `[1,2,3,4,5,8,10,11,12,13,15,16,17,18,19,20]`; across `Warrior Gear`
  records `[1,2,3,4,5,6,8,10,11,12,13,14,15,16,17,18,19,20]`. For example
  `7074` "Pro Warrior Hammer" carries `MinProfileRank 8` and `4216`
  "All-Star Warrior Axe" carries 12.
- The supplied client's `Resources/EquipmentSlotDefinitions.txt` names the
  spool slots `Head, Hands, Chest, Legs, Feet, Shoulders, Weapon, Accessory1,
  SquareRune` and defines `DiamondRune` slot 13, which is not in the recovered
  Warrior spool.

Skills: names and rank ladders are original locale text from
`reports/locale-id-to-text.json`.

| skill | name id | rank text ids | recovered rank damage |
|---|---|---|---|
| Kick | 24540 | 24541, 24544, 24546, 24548, 24550 | 133, 150, 167, 183, 200 |
| Spinning Blade | 4908 | 6163, 47414, 47415, 47416, 47417 | 50, 56, 63, 69, 75 |
| Quake | 24694 | 24695, 24697, 24699, 24701, 24705 | 83, 94, 104, 115, 125 |
| Commanding Shout | 24706 | 24708, 24710, 24712, 24714, 24716 | 50, 56, 63, 69, 75 |

Weapon abilities: locale 420252 "Sweeping Slash" with tier text 420253
" Warrior Tier 1 - Spammable Ability - Sweeping Slash", and 442458 "Spinning
Attack" with tier text 420255 " Warrior Tier - Ultra Ability - Spinning
Attack". These are name/tier facts only; no ability ids, cooldown, energy cost
or numeric damage is claimed here.

## Community or derived material, kept separate

The unlock *levels* 1, 5, 10 and 15 come from the read-only wiki Warrior
article (revision 49733, 2025-12-20). Original locale corroborates the level
milestones through "Achieve Level 5/10/15 as a Warrior" and quest titles
"Warrior: Achieve Level 5/10/15/20"; it does not itself state which skill
unlocks at which level. The wiki's effect ranges ("50-68", "83-126") do not
match the recovered locale rank ladders exactly; the locale ladders are
treated as authoritative and the difference is reported, not resolved.

The community Sanctuary `CombatJobs.json`/`CombatAbilities.json` files were
inspected and deliberately **not** used: they carry reconstructed ability ids
(5010/5011/5012), damage and radius values. This piece ships no such numbers.

## Reconstructed policy (not original)

`check_warrior_equip(job_level, class_id, slot, item_min_profile_rank)` in
`src/restoration/piece_033.rs` is reconstructed policy over the recovered
restrictions: the class must be Warrior-owned, the slot must be in the
recovered spool, and both the item and slot level gates must be at or below the
job level. `skill_unlocked(job_level, name_id)` applies the wiki levels to the
recovered skill name ids. Neither function claims native client behavior.

## Checks and outcomes

Run from the project directory:

```powershell
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_033.py" -v
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_033.py'
& '.\scripts\cargo.ps1' test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_033
```

- Python checker tests: 4 passed, 0 failed (fixture pass, wiki-level drift
  fails, Rust-module drift fails, real corpus passes).
- Real corpus run: status `pass`, 15 checks, report
  `../lab/reports/piece-033-20261009T035724Z/{warrior-progression.json,summary.md}`.
- Rust `tests/piece_033.rs` (module included with `#[path]` until the lead
  registers it): 9 passed, 0 failed.

## Gaps and remaining gates

- The unlock levels are community wiki content; a server-side or client-side
  Warrior ability table was not recovered. The exact per-level ability list
  therefore stays a documented gap.
- No ability ids, cooldowns, energy costs, AoE radii or damage values are
  recovered from original data. The community Sanctuary values were left out
  on purpose.
- The profile packet's rank section and the equipment triple middle value are
  not interpreted.
- Native ability-bar, equip and combat acceptance are not claimed; this is a
  data and policy piece only. The lead owns registration of the module, any
  ability-bar integration and the native gate.
- Item 031 already owns the job catalogue profile rows; this piece reuses
  read-only evidence rather than re-implementing that catalogue.
