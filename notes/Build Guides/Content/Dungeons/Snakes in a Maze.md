---
type: content-dungeon
zone: Briarwood (Thistlerow hedge maze)
combat: true
---

# Snakes in a Maze

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

> Snakes in a Maze! is a **node battle**, not an Atlas dungeon. The wiki files it under *Battle* (nodes are "transient and appear as NPCs or objects", not on the Atlas), started by clicking a creature. It is separate from the Atlas dungeon [[The Snarling Hedges]], which is in the same maze. [wiki: Battles, Snakes in a Maze!]

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Snakes in a Maze!" (game data `loc 2702431099`, UI `3583000408`; plain form `1748558207`) | [client] |
| Panel description | "Venomous snakes are trying to bite anyone that passes! Head into the maze and defeat them!" (`loc 4050579400`) | [client] [wiki] |
| Type | Maze battle, difficulty 1, level "Varied" | [wiki] |
| Entrance | Click an **Amazing Asp** in **Thistlerow, Briarwood**. Client hint: "Amazing Asps can be found slithering through Thistlerow." (`loc 911492415`). Video shows Amazing Asp markers 46 to 62 m away beside the Snarling Hedges label. | [wiki] [client] [video piWfe7lXZ9I/00057, 00096] |
| "NOT LIVE" copies | The client also holds "NOT LIVE - Snakes In a Maze" (game data `loc 3779511659`, UI `350402694`) next to the live names. Meaning unclear: either an older copy of the encounter or a disabled variant. | [client] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **None.** The wiki page shows "Required: Level 1, Any Combat Job" (`Required` is `loc 3137861621`; the rest is wiki text). Difficulty 1 means anyone. Client level strings exist only for 5, 9, 13 (`loc 2145075890`, `1291047686`, `1323029355`) and are generic. | [wiki] [client] |
| Members only | No (members get a coin bonus) | [wiki] |
| Unlocking quest | None found | [client] |
| Seen playing at | No battle frames in video | gap |
| Group size | Unknown | gap |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main (only goal) | **"Defeat all of the snakes!"** (`loc 4283649743`). Maze variants "Defeat all of the snakes in the hedge maze." (`580361275`) and a typo copy "...hegde maze!" (`3342657809`) may belong to this battle or to [[The Snarling Hedges]] quests; not tied. No bonus goal on the wiki or in the client. | [client] [wiki] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Enemies | **Amazing Asp** (`loc 1935444348`, `4167186094`), **Killer Cobra** (`105919611`, `3862166462`) | The wiki lists no enemies. Both names appear in the same open-world quest text ("Robgoblin Pondblasters, Amazing Asps, and Killer Cobras", `loc 2810740797`), so they are likely the maze snakes. | [client] |
| Dev ability strings | "Snakes in a Maze - Snake Dance - Roots target" (`loc 629344352`); "- Constrict - Stuns target" (`3073611076`); "- Killer Cobra - Deadly Poison - DoT" (`3916944133`); "Venom Bite - Single target poison attack - Snakes in a Maze" (`4188969632`) | Which snake uses which move is not stated. | [client] |
| Boss | None named | | gap |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "74 Stars" (wiki wording) | [wiki] |
| Members Only Bonus | "37 Coins". Generic client "Bonus: 37 Coins!" (`loc 2664870581`), not tied to a battle. | [wiki] [client] |
| Prize wheel coins | A **2 Coins** slot on every wheel, as written | [wiki] |
| Achievement | None on the wiki list; no "Win Combat Encounter Snakes in a Maze" string | [wiki] [client] |

### Prize wheel by job

Each wheel is the five items shown plus the 2 Coins slot. [wiki] The garment slot is the last item.

| Job | Items (client spelling) |
|---|---|
| Archer | Archer's Bow of Blizzards, Archer's Power Shard of Vitality I, Archer's Ring of Energy I, Battle Item Mystery Pack, **Hen Feather Archer Leggings** |
| Brawler | Brawler's Mallet of Rumbling, Brawler's Power Shard of Vitality I, Brawler's Ring of Energy I, Battle Item Mystery Pack, Saved by the Bell Brawler Pants |
| Medic | Medic's Reflex Hammer of Vitals, Medic's Power Shard of Vitality I, Medic's Ring of Energy I, Battle Item Mystery Pack, **Stable Condition Medic Pants** |
| Ninja | Ninja's Training Sword of 1000 Storms, Ninja's Power Shard of Vitality I, Ninja's Ring of Energy I, Battle Item Mystery Pack, Kusa Ninja Hakama |
| Warrior | Warrior's Cudgel of Cleaving, Warrior's Power Shard of Vitality I, Warrior's Ring of Energy I, Battle Item Mystery Pack, Standard Action Warrior Breeches |
| Wizard | Wizard's Sparkle Twig of Glaciers, Wizard's Power Shard of Vitality I, Wizard's Ring of Energy I, Battle Item Mystery Pack, Novice Wizard Slacks (or Novice Wizard Skirt) |

Other names are exact client strings. [client]

## Related quests and NPCs

Titles are not stored beside descriptions in the locale; rows give description ids. No source ties a named quest to this battle.

| Quest text | Id | Note | Sources |
|---|---|---|---|
| "As a brawler, defeat Robgoblin Pondblasters, Amazing Asps, and Killer Cobras." | `loc 2810740797` | A Brawler contract objective; the [[Brawler]] page cannot tie it to a contract title. "Defeat", not "start the battle". | [client] |
| "Defeat some Amazing Asps" / "Defeat some Killer Cobras" | `loc 204880572`, `1645346549` | Objectives; owning quest unknown. Hints: `911492415`, `4023818094`. | [client] |

NPCs: none confirmed. Zone: [[NPCs - Briarwood]].

## Conflicts

1. **Garment slot (Archer).** Wiki "Hen Feather Archer Pants"; no such client item. The client has "Hen Feather Archer Leggings" (`loc 493943759`, `2986377167`).
2. **Garment name (Medic).** Wiki "Stable Condition Medic Fatigues"; no client match. The client has "Stable Condition Medic Pants" (`loc 784223931`, `1980737962`).
3. **Zone.** Wiki says Thistlerow, Briarwood. The client names Thistlerow for the Asps and "the maze", and calls the Atlas dungeon "The Snarling Hedges". Same maze area assumed.

## Gaps

- Group size, snake counts, waves, health and ability effects, and the meaning of the two "NOT LIVE" strings.
- Whether the battle was ever live as a click-node, and what the Asp interaction prompt looks like.
- Real star and coin payout; no results frame in video.
- Which quests start it.
