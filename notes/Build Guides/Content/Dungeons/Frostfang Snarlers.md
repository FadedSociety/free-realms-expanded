---
type: content-dungeon
zone: Snowhill (mountains south-east of Snowhill)
combat: true
---

# Frostfang Snarlers

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

> Frostfang Snarlers! is a **node battle**, not an Atlas dungeon. The wiki files it under *Battle* (nodes are "transient and appear as NPCs or objects", not on the Atlas). It is started by clicking a creature. [wiki: Battles, Frostfang Snarlers!]

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Frostfang Snarlers!" (game data `loc 2846538967`, UI `2082814897`) | [client] |
| Panel description | "A pack of hungry wolves are endangering travelers! Stop them before someone gets hurt!" (`loc 2246368084`). Matches the wiki quote word for word, but the client string does not name the battle. | [client] [wiki] |
| Type | Snowy forest battle, difficulty 1, level "Varied" | [wiki] |
| Entrance | Click a **frostfang snarler** in Snowhill. Quest text puts the Snarlers in the mountains **south-east** of Snowhill (`loc 937190059`, `1962735708`, `3693529699`). One line says the **western** side (`loc 2218766120`). Recordings show Frostfang Snarler as a red enemy marker with a "Press X or mouse" prompt in the Snowhill snow. | [wiki] [client] [video v1bVCKWiSvw/00000, DRoB1oX61Os/00100] |
| Story hook | Snowhill locals "are being terrorized by packs of angry wolves" and "the only thing worse than the snarlers are the yetis" (`loc 538859206`). Which quest or battle owns that line is not shown. | [client] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **None** (difficulty 1, "can be entered by anyone"). The client "Requires a level N combat job" strings exist only for levels 5, 9, 13 (`loc 2145075890`, `1291047686`, `1323029355`); they are generic and not tied to this battle. | [wiki: Battles] [client] |
| Seen near | Warrior 20, Archer 14, Adventurer 5 to 20 (open-world frames only, none inside the battle) | [video DRoB1oX61Os/00100, QHfkcL2_pfg/00194, 0KcQWIEfWhM/00082] |
| Members only | No (the wiki lists a members-only coin bonus, so non-members play) | [wiki] |
| Unlocking quest | None found. Quests send players here (see below). | [client] |
| Group size | Unknown | gap |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main (only goal) | **"Knock out all Snarlers!"** (`loc 2232341340`). A sibling line is "Defeat the Snarlers!" (`loc 1394128387`). No bonus goal on the wiki; no "Frostfang Snarlers Bonus" string in the client. | [client] [wiki] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Enemies | **Frostfang Wolf** (wiki "Frostfang Wolves"), **Frostfang Alpha** | Frostfang Wolf `loc 1464698360`, `1467111981`; Frostfang Alpha `loc 2559120146`, `3628885318`. No boss is named. Counts, health and abilities unknown. | [wiki] [client] |
| Node creature | **Frostfang Snarler** (`loc 1287937882`, `4197571244`) | The thing you click. The wiki enemy list does not name it. | [client] [video v1bVCKWiSvw/00000] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "86 Stars" (wiki wording) | [wiki] |
| Members Only Bonus | "70 Coins". The client has a generic "Bonus: 70 Coins!" (`loc 1931562791`) and "Members Only Bonus!" (`1925651180`), not tied to a battle. | [wiki] [client] |
| Prize wheel coins | A **2 Coins** slot on every wheel, recorded as written | [wiki] |
| Battle Item Mystery Pack | On every wheel (`loc 1225426704`, `1725853870`) | [wiki] [client] |
| Achievement | None found: not on the wiki list, and no "Win Combat Encounter Frostfang Snarlers" string (only Frostfang Caverns, `loc 3125540002`) | [wiki] [client] |

### Prize wheel by job

Each wheel is the five items shown plus the 2 Coins slot. [wiki]

| Job | Items (client spelling) |
|---|---|
| Archer | Archer's Bow of Blizzards, Archer's Power Shard of Vitality I, Archer's Ring of Energy I, Battle Item Mystery Pack, Hen Feather Archer Gloves |
| Brawler | Brawler's Mallet of Rumbling, Brawler's Power Shard of Vitality I, Brawler's Ring of Energy I, Battle Item Mystery Pack, **Saved by the Bell Brawler Gloves** |
| Medic | Medic's Reflex Hammer of Vitals, Medic's Power Shard of Vitality I, Medic's Ring of Energy I, Battle Item Mystery Pack, Stable Condition Medic Gloves |
| Ninja | Ninja's Training Sword of 1000 Storms, Ninja's Power Shard of Vitality I, Ninja's Ring of Energy I, Battle Item Mystery Pack, Kusa Ninja Armwraps |
| Warrior | Warrior's Cudgel of Cleaving, Warrior's Power Shard of Vitality I, Warrior's Ring of Energy I, Battle Item Mystery Pack, Standard Action Warrior Gauntlets |
| Wizard | Wizard's Sparkle Twig of Glaciers, Wizard's Power Shard of Vitality I, Wizard's Ring of Energy I, Battle Item Mystery Pack, Novice Wizard Gloves |

Every other name is an exact client string. [client]

## Related quests and NPCs

Quest titles are not stored beside descriptions in the locale, so rows give description ids.

| Quest | Giver | Text and ids | Sources |
|---|---|---|---|
| **Warrior: Warrior Warm-up** (probable; title `loc 1777367688`/`1775539410`) | [[Drill Sergeant Dewey]], Snowhill | "Drill Sergeant Dewey wants you to practice on the Frostfang Snarlers!" (`loc 2218766120`); "As a Warrior, defeat the Frostfang Snarlers wandering the mountains south-east of Snowhill." (`1962735708`); Dewey: "Put a hurtin' on those Frostfang Snarlers" (`937190059`); hint "Having trouble finding those Frostfang Snarlers?" (`3693529699`); objectives "Defeat Frostfang Snarlers in Snowhill." (`2857140979`), "Defeat Frostfang Snarlers" (`2505559730`), "Defeat the Frostfang Snarlers" (`3912700565`). The Warrior page notes the Snarler lines may sit in this quest or the one after. The text says "defeat", not "start the battle". | [client] (see [[Warrior]]) |
| **Contract: Trouble in Snowhill** (repeatable) | [[Ree Peatpants]] | "Ree Peatpants has asked you to defeat wolves and yetis in Snowhill." (`loc 2853626813`; title `172639783`). Names wolves, not Snarlers. | [client] [wiki: Ree Peatpants] |

NPCs: [[Drill Sergeant Dewey]], [[Ree Peatpants]]. Zone: [[NPCs - Snowhill]]. Jobs: [[Warrior]], [[Ninja]].

## Conflicts

1. **Enemy name.** Wiki "Frostfang Wolves"; client "Frostfang Wolf" (`loc 1464698360`, `1467111981`). The client also names "Frostfang Snarler", absent from the wiki enemy list.
2. **Brawler gloves.** Wiki "Bell Brawler Gloves"; client "Saved by the Bell Brawler Gloves" (`loc 3683936112`, `3699014784`).
3. **Snarler direction.** Client text says south-east (three lines) and west (one line). South-east is used above.
4. **Goal.** Wiki and client agree on "Knock out all Snarlers!".

## Gaps

- Group size, enemy counts and waves, health and abilities, and whether the Alpha is a boss.
- Real star and coin payout (the wiki's 86 and 70 are unverified; no results frame seen).
- Which quest, if any, starts the battle (the quest texts say "defeat", not "click").
- Owner of `loc 538859206`, and whether the client title "Hungry Wolves" (`loc 1946868430`, `101972945`) is an old name of this or a neighbouring battle.
