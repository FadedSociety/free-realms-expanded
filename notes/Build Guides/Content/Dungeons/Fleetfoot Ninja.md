---
type: content-dungeon
zone: Greenwood Forest
combat: true
---

# Fleetfoot Ninja

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

> Fleetfoot Ninja! is a **node battle**, not an Atlas dungeon. The wiki files it under *Battle* (nodes are "transient and appear as NPCs or objects", not on the Atlas), started by clicking a Fleetfoot Ninja. No video of it was found. [wiki: Battles, Fleetfoot Ninja!]

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Fleetfoot Ninja!" (game data `loc 2337280707`, UI `2402342185`, `4081902086`) | [client] |
| Panel description | "Ninjas without fashion sense are terrorizing travelers! Defeat their leader, Niobe, to run them off!" (`loc 2011785054`) | [client] [wiki] |
| Type | Forest battle, difficulty 2, level "Varied" | [wiki] |
| Entrance | Click a **Fleetfoot Ninja** in **Greenwood Forest**. Client objectives say "Defeat Fleetfoot Ninjas in Sacred Grove." (`loc 2350636737`), a client place name; how it relates to Greenwood Forest is not stated. | [wiki] [client] |
| Story hook | Panel text above. Also: "Defeating Niobe will end the trouble the Fleetfoot ninja have been causing in this area." (`loc 1463485584`). In a Ninja scroll quest, a first-person line says "Niobe and her ninjas jumped me" while he sought the Scroll of the Jade Tiger (`loc 3615443681`; speaker not shown, probably Rokuro). | [client] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **None** (difficulty 2: "recommended levels 3 and higher, but can be entered by anyone"). No level-3 string exists in the client. | [wiki: Battles] [client] |
| Members only | No (the wiki lists a members-only coin bonus) | [wiki] |
| Unlocking quest | None found | [client] |
| Seen playing at | No video | gap |
| Group size | Unknown | gap |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main (only goal) | **"Defeat Niobe, leader of the Fleetfoot Ninja"** (`loc 2271095538`; the wiki adds a trailing "!"). No bonus goal on the wiki; no "Bonus" string in the client. | [client] [wiki] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Niobe** (`loc 805454138`, `2950520869`) | Leader of the Fleetfoot Ninja. Dev ability strings: "Fleetfoot Ninja - Niobe punch 1", "2", "3" (`loc 1347606910`, `537161854`, `4004951505`), "Niobe kick" (`3325841182`), "Niobe teleport" (`3009609841`), "Niobe Punch Combo!" (`3596168535`, `1837391786`). Effects unknown. | [client] [wiki] |
| Enemies | **Fleetfoot Ninjas** (wiki plural) | The client has no stand-alone enemy string "Fleetfoot Ninja", only the battle name. Dev strings: "Fleetfoot Ninja - black teleport" (`loc 391472723`), "- Teleport to player" (`1074303002`), "- basepop extra attack" (`1553259670`); abilities "Fleetfoot Teleport" (`2351261440`, `3577956202`) and "Fleetfoot Shuriken" (`1099716074`, `49160249`). A note reads "defeated the 3 fleetfoot ninja" (`loc 1487857482`), which hints at a count of 3 but is not tied to this fight. | [wiki] [client] |
| Barks | "Fleetfoot Ninja, assemble!" (`loc 3861175590`); "You think you're ready to challenge the Fleetfoot Ninja?" (`59559918`). Speakers not shown. | [client] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "41 Stars" (wiki wording) | [wiki] |
| Members Only Bonus | "22 Coins". Generic client "Bonus: 22 Coins!" (`loc 343739712`), not tied to a battle. | [wiki] [client] |
| Prize wheel coins | **8 Coins** slot on every wheel, as written | [wiki] |
| Achievement | None on the wiki list; no "Win Combat Encounter" string | [wiki] [client] |

### Prize wheel by job

Each wheel is the five items shown plus the 8 Coins slot. [wiki] The wiki item pages for the six weapons also name Fleetfoot Ninja! on their prize-wheel lists. [wiki]

| Job | Items (client spelling) |
|---|---|
| Archer | Archer's Horse Bow of Splintering, Archer's Power Shard of Regeneration II, Archer's Ring of Regeneration II, Battle Item Mystery Pack, Point Blank Archer Cap |
| Brawler | Brawler's Hammer of Roundhouse, Brawler's Power Shard of Regeneration II, Brawler's Ring of Regeneration II, Battle Item Mystery Pack, Peek-a-Boo Brawler Beanie |
| Medic | Medic's Scalpel of Immunization, Medic's Power Shard of Regeneration II, Medic's Ring of Regeneration II, Battle Item Mystery Pack, Urgent Care Medic Cap |
| Ninja | Ninja's Blade of Flame Wave, Ninja's Power Shard of Regeneration II, Ninja's Ring of Regeneration II, Battle Item Mystery Pack, **Nokizaru Ninja Tabi Boots** |
| Warrior | Warrior's Axe of Warcry, Warrior's Power Shard of Regeneration II, Warrior's Ring of Regeneration II, Battle Item Mystery Pack, Basic Attack Warrior Helm |
| Wizard | Wizard's Wand of Tsunami, Wizard's Power Shard of Regeneration II, Wizard's Ring of Regeneration II, Battle Item Mystery Pack, Hedge Wizard Cap |

Other names are exact client strings. [client]

## Related quests and NPCs

Titles are not stored beside descriptions in the locale.

| Quest | Giver | Text and ids | Sources |
|---|---|---|---|
| **Contract: Sharp Claws and Sharper Swords** (repeatable) | [[Ree Peatpants]], Snowhill | "Ree Peatpants has asked you to defeat Cray Marauders and Fleetfoot Ninjas." (`loc 565942739`); "As a ninja, defeat Cray Marauders and Fleetfoot Ninjas." (`3639882617`); title `loc 1061584715`/`1720151834`. Objectives "Defeat Fleetfoot Ninjas" (`220384013`), "Defeat the Fleetfoot Ninjas" (`3410197255`). The text names the ninjas; it does not say "start the battle". | [client] [wiki: Ree Peatpants] (see [[Ninja]]) |
| Scroll of the Jade Tiger thread | Master Kai and Kazuo's story | `loc 3615443681` blames Niobe and her ninjas for an ambush; quest text `1920299577`, `3148214181`. Not part of the battle. | [client] |

NPCs: [[Ree Peatpants]]. Zone: [[NPCs - Greenwood Forest]].

## Conflicts

1. **Nokizaru boots.** Wiki "Nokizaru Ninja Boots"; no such client item. The client has "Nokizaru Ninja Tabi Boots" (`loc 2773740876`, `3484397153`).
2. **Goal punctuation.** Wiki "Defeat Niobe, leader of the Fleetfoot Ninja!"; client has no "!" (`loc 2271095538`).
3. **Enemy name.** Wiki "Fleetfoot Ninjas"; the client has only the battle names.
4. **Zone.** Wiki Greenwood Forest; the client objective says Sacred Grove (relation not stated).

## Gaps

- Group size, number of ninjas, waves, Niobe's health and ability effects.
- Real star and coin payout; no results frame in video.
- Which quest starts it, and who speaks the barks.
