---
type: content-dungeon
zone: Merry Vale
combat: true
---

# Pixie Hunters

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

> Pixie Hunters! is a **node battle**, not an Atlas dungeon. The wiki files it under *Battle* (nodes are "transient and appear as NPCs or objects", not on the Atlas), started by clicking an object. It is a different encounter from the Atlas dungeon [[Den of Secrets]]. [wiki: Battles, Pixie Hunters!]

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Pixie Hunters!" (game data `loc 3465291836`, UI `3041464716`, `3611613118`; plain `469484404`) | [client] |
| Panel description | "Fairy Cultists have come out of the Den of Secrets to kidnap pixies!  Defeat them and their leader!" (`loc 2561540889`). The wiki quote is the same without the final "!". The string also sits in the Den of Secrets quest chain (see that page). | [client] [wiki] |
| Type | Difficulty 2 battle | [wiki] |
| Entrance | Click a **Mysterious Pillar** in Merry Vale. A recording shows "Pixie Hunters!" as a pink node marker 22 to 25 m away in an area full of Cakeling enemies (a birthday event, Archer 16). | [wiki] [video a1p2wZhNao0/00000, 00006] |
| Story hook | Panel text above. The boss is Macannis, "the Cruelest Fairy" (see Enemies). | [client] [wiki] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **None** (difficulty 2: "recommended levels 3 and higher, but can be entered by anyone"). No level-3 string exists in the client. | [wiki: Battles] [client] |
| Seen near | Archer 16 (node marker only) | [video a1p2wZhNao0/00000] |
| Members only | No (the wiki lists a members-only coin bonus) | [wiki] |
| Unlocking quest | None found | [client] |
| Group size | Unknown | gap |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main (only goal) | **"Find a way to defeat Macannis, the Cruelest Fairy!"** (`loc 1666491557`; the wiki prefixes "Primary:"). "Find a way" hints at a puzzle or mechanic rather than a plain boss kill. No bonus goal on the wiki; no "Bonus" string in the client. | [client] [wiki] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Macannis, the Cruelest Fairy** (`loc 3444465588`, `129771388`) | The wiki calls him "Macannis". Dev strings: "Pixie Hunters - Macannis repel ability" (`loc 1163175640`), ability "Macannis Repel" (`3368843201`, `3919267900`), and "pixie hunters - effect pixies use on Macannis" (`1922873446`). So pixies apparently apply an effect to him; how is unknown. | [client] [wiki] |
| Enemies | **Fairy Cultist** (wiki plural; `loc 695744272`, `2180850741`), **Cruel Fairy Cultist** (`loc 1479963776`, `4064549042`) | The "Cruel" variant is in the client only; whether it is used here is not shown. A dev string "Pixie Hunters - Ambient Fireball for fairy event" (`loc 3705311519`). | [wiki] [client] |
| NPC line | "Thanks for the help!  Now I'm going to go take care of that Macannis guy!" (`loc 1025611698`); speaker not shown. | [client] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "86 Stars" (wiki wording) | [wiki] |
| Coins | "6 Coins" in the wiki rewards list, recorded as written | [wiki] |
| Members Only Bonus | "37 Coins". Generic client "Bonus: 37 Coins!" (`loc 2664870581`), not tied to a battle. | [wiki] [client] |
| Prize wheel coins | A bare **"Coins"** slot on the Archer wheel; no figure given | [wiki] |
| Achievement | None on the wiki list; no "Win Combat Encounter" string | [wiki] [client] |

### Prize wheel by job

The wiki page lists **only the Archer wheel**; the other five headings are empty. For the other jobs only the weapon is known, from the weapon item pages that name Pixie Hunters! on their prize-wheel lists (they also name Thugawug Sneak!, Danger Peaks, Ice Troll Scout!, Sweetwater Climb). [wiki, item pages]

| Job | Items (client spelling) |
|---|---|
| Archer | Archer's Horse Bow of Blizzards, Archer's Necklace of Bark Skin II, Archer's Power Shard of Vitality II, Battle Item Mystery Pack, Roving Archer Boots, Coins |
| Brawler | Brawler's Hammer of Rumbling, rest unknown |
| Medic | Medic's Scalpel of Vitals, rest unknown |
| Ninja | Ninja's Blade of 1000 Storms, rest unknown |
| Warrior | Warrior's Axe of Cleaving, rest unknown |
| Wizard | Wizard's Wand of Glaciers, rest unknown |

Every name above is an exact client string (e.g. Archer's Horse Bow of Blizzards `loc 2173006625`, `2240935617`). [client]

## Related quests and NPCs

Titles are not stored beside descriptions in the locale. No source ties a named quest to this battle.

| Quest text | Id | Note | Sources |
|---|---|---|---|
| Merry Vale kidnapping chain | `loc 2561540889` (also `1028588083`, `3250915691`) | Mayor McFairy and the Den of Secrets pixie abductions; see [[Den of Secrets]]. The panel text is part of this story. | [client] |
| Fairy Cultist objectives | `loc 204699500` "Handle some Fairy Cultists", `903513980`, `631483049` | Open-world cultist hunts; owning quests unknown. | [client] |
| Pixie Nursery quest | `loc 3307692916` | "Handle the fairy cultists and robgoblin fishmongers nearby." Not shown to use this battle. | [client] |

NPCs: [[Mayor McFairy]] (story context only). Zone: [[NPCs - Merry Vale]].

## Conflicts

1. **Entrance object.** The wiki says "Mysterious Pillar". The client string "Destroy the mysterious pillars within the caverns!" (`loc 3070263815`) is the Den of Secrets bonus. The pillar names may be mixed up, or the same prop is reused. Unresolved.
2. **Boss name.** Wiki "Macannis"; client "Macannis, the Cruelest Fairy".
3. **Enemy name.** Wiki "Fairy Cultists"; client "Fairy Cultist".
4. **Goal prefix.** Wiki "Primary:"; client text has none.

## Gaps

- Five of six prize wheels, the real wheel coin figure, and the real star and coin payout (86 stars and 6 coins are wiki figures).
- Group size, cultist counts, how "find a way" works, Macannis's health and the "repel" and pixie effects.
- Which quest starts it, and where the Mysterious Pillar stands in Merry Vale.
