---
type: content-dungeon
zone: Wilds (south of the Bumbleberry Patch)
combat: true
---

# Thugawug Sneak

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

> Thugawug Sneak! is a **node battle**, not an Atlas dungeon. The wiki files it under *Battle* (nodes are "transient and appear as NPCs or objects", not on the Atlas), started by clicking a firepit. [wiki: Battles, Thugawug Sneak!]

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Thugawug Sneak!" (game data `loc 3809365958`, UI `1708117603`, `3993953571`) | [client] |
| Panel description | "Thugawug Sneaks kidnapped a semi-famous rock star! Defeat them to free him!" (`loc 3002640394`) | [client] [wiki] |
| Type | Forest battle, difficulty 2, level "Varied" | [wiki] |
| Entrance | Click a **firepit** south of the **Bumbleberry Patch** in the Wilds. (The client says the Bumbleberry Patch is "southwest ... past the Bixie Farms", `loc 427913259`.) Outside the battle, "Thugawug Sneak" is an ordinary enemy marker (red, 83 m away). | [wiki] [client] [video UFsVcZ3X5iA/00050] |
| Story hook | The rock star is **Bruce**, kidnapped by Thugawug fans and forced to play. Client: "Bruce is being forced to sing awful songs that the Thugawugs like. Help him out by knocking out all the Thugawug female Fans!" (`loc 535805016`). | [client] [wiki: Bruce] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **None** (difficulty 2: "recommended levels 3 and higher, but can be entered by anyone"). No level-3 string exists in the client. | [wiki: Battles] [client] |
| Members only | No (the wiki lists a members-only coin bonus) | [wiki] |
| Unlocking quest | None found | [client] |
| Seen playing at | No frames of the battle | gap |
| Group size | Unknown | gap |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main (only goal) | **"Help the guest star!"** (`loc 2667119631`). A related client line is "Help Bruce!" (`loc 1260723979`). No bonus goal on the wiki; no "Bonus" string in the client. | [client] [wiki] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Enemies | **Thugawug Fan** (`loc 792844711`, `2934940997`), **Thugawugs** (`1452792709`, `1579172513`) | Quest text says "female Fans" (`loc 535805016`), so the Fans are the targets. A dev string "thugawug sneaks flavor ability" (`loc 4185549290`). Counts unknown. | [wiki] [client] |
| Friendly NPC | **Bruce** (`loc 255196100`, `533439775`) | Rescue target. | [client] [wiki] |
| Boss | None named | | gap |

### Scripted lines (client, speakers from the wiki)

The wiki's Bruce page records this exchange as the in-battle script, and each line exists in the client. [client] [wiki: Bruce]

1. Bruce: "It's your world... You can do what you want to do...  Please help me... They won't let me stop playing this song!" (`loc 439016396`); "Rock is the word!  Rock is the power! Yeah yeah!" (`2711032681`)
2. Thugawug Fan: "Go, Bruce! I never get tired of this song!" (`loc 3671610195`); "Play the Free Realms song! Or else!" (`2871200140`)
3. Bruce, after the rescue: "Thank you for rescuing me from those crazed fans! Let's get out of here!" (`loc 391828984`)

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "62 Stars" (wiki wording) | [wiki] |
| Members Only Bonus | "18 Coins". Generic client "Bonus: 18 Coins!" (`loc 3364924573`), not tied to a battle. | [wiki] [client] |
| Prize wheel coins | A bare **"Coins"** slot on the Medic wheel; no figure given | [wiki] |
| Achievement | None on the wiki list; no "Win Combat Encounter" string | [wiki] [client] |

### Prize wheel by job

The wiki page lists **only the Medic wheel**. For the other jobs only the weapon is known, from weapon item pages that name Thugawug Sneak! on their prize-wheel lists (they also name Pixie Hunters!, Danger Peaks, Ice Troll Scout!, Sweetwater Climb). [wiki, item pages]

| Job | Items (client spelling) |
|---|---|
| Medic | Battle Item Mystery Pack, Medic's Power Shard of Vitality II, Medic's Ring of Energy II, Medic's Scalpel of Vitals, Urgent Care Medic Pants, Coins |
| Archer | Archer's Horse Bow of Blizzards, rest unknown |
| Brawler | Brawler's Hammer of Rumbling, rest unknown |
| Ninja | Ninja's Blade of 1000 Storms, rest unknown |
| Warrior | Warrior's Axe of Cleaving, rest unknown |
| Wizard | Wizard's Wand of Glaciers, rest unknown |

Every name above is an exact client string (Medic's Scalpel of Vitals `loc 1616934325`, `4262915773`; Urgent Care Medic Pants `3630931021`, `3873436516`). [client]

## Related quests and NPCs

Titles are not stored beside descriptions in the locale. No source ties a named quest to this battle.

| Quest | Giver | Text and ids | Sources |
|---|---|---|---|
| **Warrior: Knocking Some Sense In** | Aster, road south of Merry Vale | "Just knock some sense into some of those Thugawug Sneaks and a Thugawug Bumbler." (`loc 3906118291`); "You need to defeat some Thugawug Sneaks and a Bumbler so I can pass." (`3815545278`); title `2111744232`/`3588410217`. Open-world Sneaks, not the click-node. | [client] (see [[Warrior]]) |
| **Contract: Bowman for Hire** (repeatable) | [[Eddie Eagle Eyes]] | "As an archer, defeat Thugawug Sneaks that wander the wilderness, and some Shadow Talon Apprentices in Danger Peaks." (`loc 2704426997`; intro `2054828921`); title match is tentative on the [[Archer]] page. | [client] |
| "Defeat Thugawug Sneaks" | not shown | Objective, `loc 3542187094` | [client] |

NPCs: [[Bruce]]. Zone: [[NPCs - Wilds]].

## Conflicts

1. **Bruce listed as a mob.** The wiki puts "Bruce" under "Mobs: NPCs"; the client treats him as a rescue NPC. Both are consistent.
2. **Wheel completeness.** Only the Medic wheel is on the battle page.
3. **Entrance.** The wiki says a firepit; the title and the red "Thugawug Sneak" marker suggest a creature. Unresolved.

## Gaps

- Five of six prize wheels, the real wheel coin figure, and the real star and coin payout (62 and 18 are wiki figures).
- Group size, Fan counts and waves, whether Sneaks (not only Fans) fight.
- The firepit's exact position and what it looks like.
- Which quest starts it.
