---
type: content-dungeon
zone: Snowhill
combat: true
---

# Ice Troll Scout

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

> Ice Troll Scout is a **wandering-encounter battle**, not an Atlas dungeon. The wiki files it under *Battle* and *Snowhill*. The player clicks a roaming Ice Troll Scout. [wiki]

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Ice Troll Scout!" (game data `loc 2863547858`, UI `loc 3378281663`). The enemy itself is "Ice Troll Scout" (`loc 305101810`, `1421032326`). | [client] |
| Panel description | "An ice troll is stealing from tourists! Find him and knock him out!" (`loc 1143438941`) | [client] [wiki] |
| Type | Snowy battle, difficulty 2 | [wiki] |
| Entrance | A roaming Ice Troll Scout on the fringes of Snowhill. The recordings show it as a red enemy marker with the "Ice Troll Scout" name, about 33 to 62 m away, near the Hot Springs Haven marker and copper-mining markers. Client: "Ice Troll Scouts can be found near Snowhill." (`loc 3590488412`). The dev strings call it a "wandering encounter" (`loc 3813654513`). | [client] [video YxvolFOnaJM/00096, 00098, VO5EE-1VdkA/00008, gEi7O6pbnQQ/00022] [wiki] |
| Story hook | An ice troll scout is snooping around the edges of Snowhill and "may try to ambush you". One quest suspects he is "the first wave of an attack" from the Howling Hills trolls. | [client `loc 900680513`, `4058148193`, `937113415`] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | None. Difficulty 2: "Recommended levels 3 and higher, but can be entered by anyone". The wiki infobox says "Level 1, Any Combat Job". No client panel line for tier 2 was found. | [wiki: Ice Troll Scout!, Battles] [client] |
| Members only | Not tagged on the wiki. | [wiki] |
| Unlocking quest | None needed to click the node as far as any source says. Several quests point at it (below). | [wiki] [client] |
| Group size | Unknown. | gap |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main (only goal listed) | **"Defeat the Ice Troll Scout!"** No bonus goal is listed. No "Bonus Objective" or "Win Combat Encounter" string was found for this battle. | [client `loc 1647936596`] [wiki] |
| Quest-side objectives | "Defeat the Ice Troll Scout" (`loc 338662044`), "Defeat an Ice Troll Scout" (`loc 2060370786`), "Find the Ice Troll Scout in Snowhill and defeat him." (`loc 3722703003`) | [client] |

## Enemies and boss

| Role | Who | Sources |
|---|---|---|
| Boss | **Ice Troll Scout**, a single troll. Dev notes give it an "Ice Clap" ability: "Weaker version of Howling Hills Ice Troll ability Ice Clap for wandering Encounter - Ice Troll Scout" (`loc 2675402140`). Effect and numbers unknown. | [client] [wiki] |
| Other enemies | None named. | gap |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "25 Stars" | [wiki] |
| Prize wheel | One wheel per job: 5 items plus a **4 Coins** slot | [wiki] |
| Members bonus | "Members Only Bonus: 24 Coins" | [wiki] |

### Prize wheel by job

Every item name exists in the client locale as written, except the Medic garment noted in Conflicts. Every wheel also holds Battle Item Mystery Pack and 4 Coins. [wiki] [client, checked by exact name]

| Job | Items | Source |
|---|---|---|
| Archer | Archer's Horse Bow of Blizzards, Archer's Power Shard of Vitality II, Archer's Ring of Energy II, Point Blank Archer Leggings | [wiki] |
| Brawler | Brawler's Hammer of Rumbling, Brawler's Power Shard of Vitality II, Brawler's Ring of Energy II, Peek-a-Boo Brawler Pants | [wiki] |
| Medic | Medic's Power Shard of Vitality II, Medic's Ring of Energy II, Medic's Scalpel of Vitals, "Urgent Care Medic Fatigues" (no client match) | [wiki] |
| Ninja | Ninja's Blade of 1000 Storms, Ninja's Power Shard of Vitality II, Ninja's Ring of Energy II, Nokizaru Ninja Hakama | [wiki] |
| Warrior | Basic Attack Warrior Breeches, Warrior's Axe of Cleaving, Warrior's Power Shard of Vitality II, Warrior's Ring of Energy II | [wiki] |
| Wizard | Hedge Wizard Slacks or Skirt, Wizard's Power Shard of Vitality II, Wizard's Ring of Energy II, Wizard's Wand of Glaciers | [wiki] |

## Related quests and NPCs

Quest titles are not stored beside descriptions in the locale, so each row gives a description id. Givers are not stated unless noted.

| Quest text (client) | Id | What it asks | Sources |
|---|---|---|---|
| "Sightings of an Ice Troll Scout poking around on the fringes of Snowhill have been a cause for concern. Track down the scout and defeat him before he can complete his mission." | `937113415` | Kill the scout | [client] |
| "Beyond the yetis, the biggest threat to Snowhill is probably the ice trolls ... an Ice Troll Scout lurking just outside of Snowhill. ... Defeat him before he gets whatever it is he came for." | `1434545011` | Kill the scout (a Snowhill quest-giver speaks; name not in the string) | [client] |
| "Find and defeat the Ice Troll Scout! Careful... He may try to ambush you!" | `900680513` | Objective text | [client] |
| "Defeat an Ice Troll Scout snooping around the edges of Snowhill." | `2063594847` | Kill quest | [client] |
| "Keep Snowhill safe from the wandering Ice Troll Scout!" | `850594696` | Summary text | [client] |
| "Chase off some Ice Troll Scouts" | `2200866489` | Short text | [client] |
| "Jonathon Forkpath thinks that the Ice Troll Scout might be the first wave of an attack. Go into the Howling Hills and let the ice trolls in there know that an attack on Snowhill would be a bad idea." | `4058148193` | Follow-up that leads to [[Howling Hills]] | [client] |

Rewards of all these quests are unknown. NPCs: [[Jonathon Forkpath]]. Zone: [[NPCs - Snowhill]]. Related battles in the same area: [[Frostfang Snarlers]], [[Petty Yetis]].

## Conflicts

1. **Medic garment.** The wiki writes "Urgent Care Medic Fatigues". The client has Urgent Care Medic Pants (`loc 3873436516`), Gloves, Boots, Shirt, Cap and Backpack, and no "Fatigues" item. The Medic wheel garment is probably the Pants, but this is unproven, so the wiki name is kept and flagged. The client does use "Fatigues" for other Medic sets (for example "Energy Transfer Medic Fatigues", `loc 176942616`).
2. **Level.** The wiki infobox says "Level 1, Any Combat Job" while the difficulty-2 text says level 3 is recommended. Both can be true (no hard requirement). Client has no matching line.

## Gaps

- Boss health, wave layout, ability effects, whether any minions appear.
- Real star and coin payout (wiki figures only), results screen.
- Which quest titles own the description ids above, their givers and rewards.
- Group size, respawn timing of the wandering scout.
