---
type: research
updated: 2026-10-22
task: T637
verdict: PREMISE PARTLY WRONG; PROPOSAL READY FOR LEAD
---

# Damage Scale

Note only; no code changed. The lead decides the scale from this note. Confidence words: **high** (client or many agreeing
sources), **medium** (one reviewed source or a fit), **low** (single frame, inference). `(R)` marks reconstructed values.

## Verdict in six lines

1. **The client builds no damage number itself.** Floating numbers print the signed amount the server sends (`%d`, `%d!!`,
   `+%d`). Weapon-ability tooltip rows ("Deals N damage.") print a float the server sends inside the ability definition
   (36/13 effect record). Nothing in the executable, its resources or its Lua scales a number by level or tier.
2. **The brief's "two disagreeing scales" does not exist today, but a different one does.** Our 36/13 definitions carry no
   effect rows, so no weapon tooltip shows any damage. The job-skill tooltips we do send (`description_id`) show numbers baked
   into the client locale (Zap 83, Kick 133, Shuriken Toss 50, First Aid 250). Those disagree with our floating numbers too
   (see Q2), and in the opposite direction for damage.
3. **The live scale is the tier ladder.** Tier bases 889 / 1554 / 2716 / 4750 / 8302 (x1.748 per tier). Video damage numbers
   and wiki tooltips sit on it (Q3). Our player damage is **3.7x** below it at every tier, because the ratio between tiers
   already matches.
4. **Player health is the opposite.** Video HUD frames fit `500 x 1.75^((L-1)/4)` (level 16 = 4077 vs 4069, level 20 = 7135 vs
   7116). Our baseline is 1000, so our health is **2x above** live. Heals already match at level 1 (254 / 250).
5. **The client has no level-to-tier table.** It gates each weapon on the item's own `MinProfileRank` and shows its own
   `PowerRating`. The vendor and level-scaling ladder is levels **1 / 4 / 8 / 12 / 16**; our `weapon_tier` uses 1 / 4 / 5 / 8 / 12.
6. **Proposal (Q5):** stage A (damage and enemy health x3.72, tier bands 1/4/8/12/16, heals follow the tier ladder) is a pure
   rescale with unchanged fight pacing. Stage B (player health 500) is optional. No enemy health exists in any source, so
   enemy numbers stay (R).

## Q1. Where do tooltip damage numbers come from?

| Fact | Evidence | Confidence |
|---|---|---|
| Floating number = packet amount, nothing else. `00ae2350` formats the event's signed delta: `+%d` for a gain, `%d!!` for a flagged negative, `%d` otherwise, then `008bb0b0` colours it. | IDA `decompile 00ae2350`; event built by `00a2ba40` (32/7) | high |
| A tooltip effect row is built from the effect record's named fields. `00a34e30` walks the ability's effect list and calls `00a30d20` per effect. `00a30d20` publishes `Param1`, `Param1TunedLow`, `Param1TunedHigh`... as `(int)(float)` of the record's own fields, and `DurationSeconds`, `MaxCharges`, `ChanceToOccur`. No level, stat or tier is read. | IDA `decompile 00a30d20` (`Param1TunedLow` = float at record `+100`, `TunedHigh` `+104`), `00a34e30` | high |
| Those fields come straight off the wire. The effect reader `00a30360` stores the dword after the counted blob at `+100`, then `+108`, then `+116/+104/+112/+120` through the NaN-checking float reader `008d53b0`. | IDA `decompile 00a30360` | high |
| The text template is locale, the number is the packet. Effect type 1 (direct damage, `AbilityEffectTypeSettings.txt` row 38) uses locale 2983 "Deals #number([*Param1TunedLow*]) damage."; type 3 rows use 92026 "Heals you for #number(TunedLow) - #number(TunedHigh) health." Level-band variants exist: 13095-13099 "Deals N damage at Level 1/4/8/12/16", 6816-6820 and 14827-14836 the 2x/3x/4x forms. | `lab/working/OSFR.Client/OSFR Client/Resources/AbilityEffectTypeSettings.txt` rows 3, 4, 38; `lab/reports/locale-id-to-text.json` ids above | high |
| The item tooltip passes the item's `PowerRating` (definition `+128`) and `MinProfileRank` (`+132`, "item_level_requirement") as plain numbers. Ability rows come from the item's `ItemAbilityEntries` through ability definitions fetched by id. | IDA `decompile 00a76a60` (`"PowerRating"` from `a2+128`), `00baec20` (`a2+132`) | high |
| The client ships no ability, damage or enemy table. | `notes/Build Guides/Gameplay/Combat and Abilities.md:146`; Lua `lab/decompiled/client-ui/ScriptsBase.lua` has no damage arithmetic (rg `TunedLow\|damage` finds only UI params) | high |
| **Exception: the job-skill description text is client-baked.** `description_id` resolves to locale text with the number inside: 3813 "Heals yourself and any ally near you for 250 health. Next rank: 281", 35505-35513 Zap 83/94/104/115/125, 24541 Kick 133 -> 150, 24491 Shuriken Toss 50 each, 24365 Multi-Arrow 50, 24134 Leg Sweep 50, 3815 Shock Paddles 75, 24695 Quake 83. | `lab/reports/locale-id-to-text.json`; ids from `data/content/job_abilities.json` | high |

**What the server sends today.** `opening_abilities.rs:419-434` builds the 160-byte 36/13 with every optional field zero, so the
effect count is 0 and no row is drawn. `gameplay.rs:4456-4458` answers a 36/12 lookup only from definitions already sent. The
weapon-ability tooltip therefore shows no damage; the skill tooltips show the locale text numbers above.

**Consequence.** Whatever scale we choose, the weapon-tooltip rows can only disagree with the floating numbers if we send them
from a second formula. The fix is a rule, not a rescale: the effect row's `Param1TunedLow/High` must come from the same
function that the encounter uses to strike (Q5 item F).

## Q2. What does the server send today?

Formulas (all (R) except where noted): basic base by job `encounter.rs:2897-2910` (Medic 75, Archer 80, Ninja 100, Brawler 100,
Warrior 110, Wizard 130; slot 1 doubles it), times `1.75^(weapon_tier(level)-1)` (`encounter.rs:1194-1195`), times the ability
multiplier (`skills.rs:164-166`, `damage_multiplier` in `job_abilities.json`). Crit x1.5 (`encounter.rs:81-82`). The number sent is
the damage actually dealt, clamped to the target's remaining health (`encounter.rs:1905`).

**Basic attack by tier** (computed with the production rounding: `round(base x 1.75^(tier-1))`; the Job Basic Attacks note's
Ninja/Warrior/Wizard rows agree):

| Job | Tier 1 | Tier 2 | Tier 3 | Tier 4 | Tier 5 |
|---|---:|---:|---:|---:|---:|
| Medic | 75 | 131 | 230 | 402 | 703 |
| Archer | 80 | 140 | 245 | 429 | 750 |
| Ninja, Brawler | 100 | 175 | 306 | 536 | 938 |
| Warrior | 110 | 193 | 337 | 590 | 1032 |
| Wizard | 130 | 228 | 398 | 697 | 1219 |
| Live Medic Incision | 279 | 488 | 853 | 1492 | 2609 |

Server Medic / live Incision = 0.27 at every tier (3.7x low). Medic Triage damage on the server is `basic x 3.2`: 240 / 419 / 736
/ 1286 / 2250 against live 889 / 1554 / 2716 / 4750 / 8302 (also 3.7x low).

**Tier gate in use** (`encounter.rs:52-60`): level 1-3 tier 1, 4 tier 2, 5-7 tier 3, 8-11 tier 4, 12+ tier 5.

**Player health** (`level_health`, `encounter.rs:45-48`: `1000 x 1.75^((L-1)/4)`, clamp 1..20; job multipliers
`encounter.rs:68-79`: Brawler 1.15, Warrior/Archer 1.0, Ninja/Medic 0.9, Wizard 0.85):

| Level | 1 | 4 | 8 | 12 | 16 | 20 |
|---|---:|---:|---:|---:|---:|---:|
| Base health | 1000 | 1522 | 2663 | 4660 | 8154 | 14270 |
| Medic | 900 | 1370 | 2397 | 4194 | 7339 | 12843 |
| Brawler | 1150 | 1750 | 3062 | 5359 | 9377 | 16411 |

Cap for saved health `PLAYER_HEALTH_CAP` 20,000 (`encounter.rs:39`). Existing tests pin level 10 = 3522 and level 20 = 14270
(`w64_regional_quest_tests.rs:2514-2552`).

**Heals** (`skills.rs:236-241`): `ability.heal x level_health(level) / 1000`. First Aid 250, Triage 254 at level 1 (data
`job_abilities.json`): level 4 380 / 386, level 8 665 / 676, level 12 1165 / 1183, level 16 2038 / 2071, level 20 3567 / 3624.
Potions heal a percent of maximum health (`item_bar.rs:56-61`), so they are scale-free.

**Enemy health and damage** (all authored level-1 baselines (R); no live source, Q3). Field enemies (`field_content.rs:160-163`):
`max_health x level_health(level)/1000`, `damage x 1.75^(weapon_tier(level)-1)`. Field battles (`field_content.rs:289-292`):
both multiplied by `level_health(difficulty_level)/1000` (difficulty 1-5 maps to level 1/3/5/9/13, `encounter.rs:86-95`). Examples:
Hooligan Wolf 180 hp / 25 dmg at level 5 becomes 315 / 77; Hooligan 180 / 15 at level 10 becomes 634 / 80; level 19 2233 / 141.
Authored baselines span 100-1020 hp and 4-62 damage across `encounters.json` (12 rows), `enemies.json` (85),
`enemies_unresolved.json` (55), `field_enemies.json` (34), `field_battles.json` (53), `dungeons/opening_cave.json` (10).
Validation bounds: hp <= 100,000 and damage <= 1,000 (`encounter.rs:2759-2761`).

**Client-baked skill numbers against our level-1 floats** (slot-1 skills, basic x multiplier): Zap tooltip 83, server 130 x 1.8 = 234;
Kick 133 vs 198; Shuriken Toss 50 vs 200; Multi-Arrow 50 vs 144; Leg Sweep 50 vs 200; First Aid heal 250 vs 250 (match).

## Q3. What did the live game use?

### Damage (high)

The wiki harvest (`lab/references/freerealms-wiki/harvest/lists/weapons.json`, `attacks.json`) and the tooltip images agree on
one ladder: tier base **889 / 1554 / 2716 / 4750 / 8302** (ratios 1.748, 1.7477, 1.7489, 1.7478), attack = base x a seventh-based
multiplier. Every job's spammable first attack is the same fraction: Zap, Incision, Sweeping Slash, Barrage, Wallop all 0.314
(`attacks.json` `damage_x`), Glancing Blow, Burn, Smack 0.286. The 0.314 form is 0.286 x 1.1 (Incision 279 / 488 / 853 / 1492 / 2609
= floor(base x 11/35), checked on all five). Triage: damage 1.0 x base, **heal 2/7 x base = 254 / 444 / 776 / 1357 / 2372**
(`weapons.json`: Triage rows). Level-scaling "New School" and Player Studio weapons print the ladder against player level:
"Deals 254 / 444 / 776 / 1357 / 2372 damage at Level 1 / 4 / 8 / 12 / 16" and "889 / 1554 / 2716 / 4750 / 8302" (harvest FTS
`search` table, title "New School Exploding Hammer item.jpg" and others). The harvest README documents the 91% text-vs-image
agreement.

**Video floating numbers sit on the same ladder** (`lab/video-archive/review/haiku/B*.jsonl`, reviewed frame records, medium):

| Job level | Number | Ladder reading |
|---|---|---|
| Brawler 20 | -2372, -2609, -5218 | 8302 x 2/7, x 0.314, 2x that |
| Brawler 9 | -776, -1552 | 2716 x 2/7, x 4/7 |
| Medic 10 | -2987 | 2716 x 1.1 (Triage, x1.1 variant) |
| Archer 16-20 | -2490, -1245, -4981!! | 8302 x 0.30, 0.15, 0.60 |
| Medic 8-19 | -127 (several) | 889 / 7 (a tier-1 weapon at level 19) |
| Medic 19 / unnamed | +371, +375 (green heals) | not on the ladder (maybe a percent heal) |

### Player health (medium)

Reviewed frame records and OCR (`lab/video-archive/out/*/frames.jsonl`, job and level read from the HUD): Brawler 1 **500**; Adventurer 1
400; Brawler 7 1487; Brawler 9 1568; Warrior 9 1896; Medic 10 2081; Medic 14 3639; Archer 14 3483; Ninja 14 3309; Archer 16
4069 (and 4476, 4883); level 20: Ninja/Warrior 7116, Archer 7349 and 8540, Brawler 9821. Fit `500 x 1.75^((L-1)/4)`: level 9 1531, 14 3082, 16
4077, 20 7135. Gear raises it (Power Shards below), so single frames scatter by +-30%; level 1, 16 and 20 are the clean points.
Non-combat jobs differ (Adventurer 20 2500, Card Duelist 6 800). The `level_health` comment already names these videos; it states
that level-1 health is unsourced, and the video gives 500, not 1000.
Wiki, high: Power Shard of Vitality adds **133 / 233 / 407 / 712** health in combat at levels 4 / 8 / 12 / 16 (x1.75 per tier; Honey
Roasted Moonbean 712), Regeneration 23 -> 122 -> 213 (x1.75). Those absolute bonuses read as 17% of a 4069-health level-16 player and 9% of ours.

### Enemy health and heal size

- **Enemy health: none found.** The harvest (FTS `health`, `HP`, `hit points`: only a Game Update note "increased hit points" and
  unrelated game screenshots) and the 4,184 article facts hold no creature health or damage figure. Video target bars show no
  number. Enemy numbers stay (R).
- **Heal size:** First Aid 250 rank 1, 281 rank 2 (client locale 3813, high). Triage heal ladder above (high). Medium Health Potion
  heals 16% per second for 3 s plus 2% instant (percent of maximum, wiki tooltip, high).
- **Era caveat:** the 83 / 94 / 104 / 115 / 125 skill-rank text, the 133 / 50 skill numbers and the Medic wiki page (revision
  2025-12-20) describe a launch-era skill system; the 889 ladder and every video number above belong to the later weapon-power system.
  Our level-1 numbers (75-130) are launch-era sized with a final-era x1.75 ladder bolted on, which is why they match neither.

## Q4. Which tier gate does the client use?

**None as a level table.** The client compares per item: `00a142c0` returns "usable now" only when `profile rank (profile+20) >=
item MinProfileRank (def+132)` and the slot and requirement sets validate; the tooltip shows the same two numbers ("Level N",
"Power: N"). Tier is data on the item (`PowerRating`), not a client function. The gate for our weapons is therefore the item
catalogue plus the server's own damage rule.

Data (community capture decoded from `lab/reports/item-catalogue-20261015/catalogue.json` with the schema of
`scripts/build_item_catalogue.py`; wiki cross-check, medium):

- Vendor line for all six combat jobs: Power 1/2/3/4/5 at `MinProfileRank` **1 / 4 / 8 / 12 / 16** (wiki vendor weapons: 17 P1 at 1, 24
  P2 at 4, 38 P3 at 8, 55 P4 at 12, 64 P5 at 16; prices 750 / 850 / 1055 / 1504 / 2152). The Ninja named set 75090-75119 is the
  same: P1 rank 1, P2 rank 4 (rank 5 cosmetics), P3 rank 8, P4 rank 12, P5 rank 16. `piece_034.rs:16` says 1/5/8/12/16, which the
  catalogue contradicts (rank 5 only on cosmetic P2 rows).
- Reward weapons run one band ahead (P3 at rank 4-5 or 11, P4 at 8-15, P5 at 12-20). That is where our 1/4/5/8/12 comes from; it
  picks the best weapon obtainable, not the player's tier.
- The client's own locale prints damage at **Level 1 / 4 / 8 / 12 / 16** (ids 6816-6820, 13095-13099, 14827-14836) and defence at
  the same levels (6737-6741). The level-scaling weapons use that table.
- Video (low, two frames): Medic 10 hits 2716 x 1.1 (tier 3) where ours would be tier 4 at level 10; Warrior 7 hits 1554 x 0.314 =
  488 and 1554 / 7 = 222 (tier 2, frame -221) where ours is tier 3. Both fit the 1/4/8/12/16 bands.

**Answer:** use the level bands 1 / 4 / 8 / 12 / 16 (high for level-scaling weapons, medium for vendor weapons). Better still once
an equipped weapon is tracked: tier = the equipped item's `PowerRating`, because a level-19 Medic with a power-1 weapon hits tier 1.

## Q5. Proposal

Every ratio the server already matches stays: x1.75 per tier (live 1.748), health x1.75 per four levels (video fit), heal 250 / 254 at
level 1, Triage = 3.2 x basic (live 889 / 279 = 3.19), crit x1.5 (client "50 Crit Mult %"), potions as percent, job identity (R).
Tune everything from one new file, `data/content/combat_scale.json`, with sanity bounds (tier levels strictly increasing from 1,
at most 20; basic damage 1..100,000; base health 100..100,000), not pinned tests.

### Stage A: damage scale (recommended; fight pacing unchanged)

| # | Change | Number and source |
|---|---|---|
| A1 | `profile_stats` basic bases x3.72 | Medic 75 -> **279**, Archer 80 -> 298, Ninja and Brawler 100 -> 372, Warrior 110 -> 409, Wizard 130 -> 484. Medic 279 = wiki Incision tier 1; others keep the existing ratio to Medic (the owner's job identity, (R)). Keeping the `1.75^(tier-1)` formula gives 279 / 488 / 854 / 1495 / 2617 against live 279 / 488 / 853 / 1492 / 2609 (max +0.3%). Per-job live-faithful alternative: every job 279, since live first attacks are identical across jobs. |
| A2 | Enemy `max_health` x3.72 | Keeps every time-to-kill. Rewrite the baselines in the six data files above (249 rows) with a script that records the factor in each file's provenance, or apply one multiplier at the three load points `field_content.rs:160`, `:289` and the `encounters.json` loader `encounter.rs:913`. Worst case 1020 x level factor 5.36 x 3.72 = 20.3k, inside the 100,000 bound. |
| A3 | Triage multiplier | Keep 3.2 (live 3.186, result 893 vs 889). |
| B-free | Heals | Keep 250 / 254 (already live-scale at level 1). |

### Stage A2: tier gate and heal ladder (independent)

| # | Change | Source |
|---|---|---|
| D | `weapon_tier(level)` reads `tier_min_levels` = **[1, 4, 8, 12, 16]** from data (was 1/4/5/8/12 in code, `encounter.rs:52`). Levels 5-7 drop a tier, 8-11 drop a tier, 12-15 drop a tier. Enemies use it too (`field_content.rs:162`), so field enemy damage drops with it. | Q4: client locale 13095-13099, wiki New School ladder, vendor line |
| C | Heal follows the tier ladder: `heal = ability.heal x damage_scale` instead of `x level_health/PLAYER_MAX` (`skills.rs:240-241`). Result 254 / 444 / 777 / 1361 / 2382 vs live 254 / 444 / 776 / 1357 / 2372; today 386 / 676 / 1183 / 2071 at levels 4 / 8 / 12 / 16 is 13% low. | wiki Triage rows |
| F | Tooltips: when 36/13 grows effect rows, write `Param1TunedLow/High` from the strike function (`base x damage_scale x damage_multiplier`, same rounding), type 1 row for damage, type 3 for heals (`00a30d20` fields, wire offsets `+100/+104` after the counted blob read by `00a30360`). Never a second formula. | Q1 |
| G | Skill description text: pick description ids without a baked number, or accept 83 / 50 / 133 as the client's launch-era text. Open (R). | Q1 exception |

### Stage B: player health (optional)

`PLAYER_MAX` 1000 -> **500** (`encounter.rs:37`; video level 1, 16, 20). `level_health`, the field-content ratios
(`field_content.rs:160`, `:289`) and heal scaling divide by `PLAYER_MAX`, so they need no edit. To keep pacing, enemy `damage` x0.5. Level-1
enemy damages are 4-25, so halving rounds badly (7 -> 4 is +14%, 4 -> 2 is 0%); do it only if you accept that drift or store
damage as a float. Saved players above the new maximum are clamped by `set_player_job` (`encounter.rs:1198-1203`). Benefit: the
absolute shard and food bonuses (133-712) and live heal sizes land at their live share of health (First Aid 250 is 50% of a 500
player). Reject a presentation-only multiplier on the wire: two number systems and saves in internal units.

### Tests and content each change touches

| Change | Content | Tests that pin the old numbers |
|---|---|---|
| A1 | new `combat_scale.json` (or `profile_stats` constants) | `encounter_job_skills.rs:18-23` (basic 75, crit 112); `w64_regional_quest_tests.rs:2560` (Medic 402 or 603 at level 10, tier 4); bot gates `scripts/bot/w85-*.bot` ("dealt 150" assumes level-4 hits clamp the Gloamling's 150) |
| A2 | `encounters.json`, `enemies.json`, `enemies_unresolved.json`, `field_enemies.json`, `field_battles.json`, `dungeons/opening_cave.json` | `field_enemies_runtime.rs` (hp 100 / 175, damage 21); `bandit_hideout_dungeon.rs:94-100` (boss 980, bandit 490 live health); `encounter_job_skills.rs:360-385` (`job_wins` fixture hp 240 x curve); `battle_catalogue_audit.rs` bounds; opening wolf and Mac fixtures |
| D | `combat_scale.json` `tier_min_levels` | `w64_regional_quest_tests.rs:2514-2560`; `field_enemies_runtime.rs:274` (7 x 1.75^2 at level 5); `piece_034.rs` / `tests/piece_034.rs` are unaffected (catalogue ranks) |
| C | none | `w64_regional_quest_tests` medic heal test (`medic_job_abilities_strike_heal_and_refuse`) |
| B | enemy damage baselines | `w64_regional_quest_tests.rs:2522-2552` (3522, 14270, 14270 cap), `combat_jobs_differ_in_health_and_crits`, `kart`/`startup_health` health sync |

Per the tunable-over-locked rule, replace the pinned literals in those tests with bounds against the data values (ratio between
tiers, health within 10% of `base x 1.75^((L-1)/4)`) while moving the numbers.

## Open questions (R)

- Enemy health and damage on the live game: no source; scaled by the same factors (R).
- Whether the live job-skill numbers (83 / 133 / 250) stayed in force beside the weapon ladder in the final era; client text says
  they were still shown (locale), the videos show only ladder numbers.
- Whether job health multipliers (Brawler 1.15, Wizard 0.85) existed live: video Brawler 20 = 9821 is 1.38x the 7116 Ninja/Warrior
  frames, Brawler 1 = 500 is 1.25x Adventurer 1 = 400; the server's 1.15 is in range but unproven.
- The `%` heal numbers (+371 / +375 green) are unexplained.

## Method

Searches ran in `harvest.sqlite` FTS and `lists/*.json`, `articles.sqlite` (4,184 facts), `lab/video-archive/review/haiku` (5,760 reviewed
frames) and `lab/video-archive/out/*/frames.jsonl` (OCR), the decoded item catalogue and the locale export
`lab/reports/locale-id-to-text.json`. IDA queries: `decompile 00ae2350`, `00a2ba40`, `00a30d20`, `00a30360`, `00a34e30`, `00c20420`,
`00a76a60`, `00baec20`, `00a142c0`, `00a15690`; `strings PowerRating`, `MinProfileRank` (no match). Scratch scripts stayed in
`%TEMP%\op-T637\`.
