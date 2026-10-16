---
type: content-dungeon
zone: Seaside (north end, near the Tide Pools)
combat: true
---

# Snatching Snappers

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

> Snatching Snappers is a **night-only special-spawn battle**, not an Atlas dungeon. The wiki files it under *Battle*, *Special Spawn* and *Seaside*. The player clicks a roaming Snatching Snapper. [wiki]

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Snatching Snappers!" (game data `loc 1259532086`, UI `loc 3091471312`) | [client] |
| Panel description | "Bothersome crays are stealing from beachgoers! Stop them before tourism declines!" (`loc 3024485759`) | [client] [wiki] |
| Type | Sand-area battle, difficulty 3 | [wiki] |
| Entrance | Click a Snatching Snapper (red enemy marker) in Seaside. The wiki says they are in the north end of the region, near the Tide Pools. The recordings show the marker on the beach path, 43 to 62 m away. | [wiki] [video s_4bpO1xPDs/00732, 00806] |
| Spawn rule | **Night only.** Client: "at night Snatching Snappers are always attacking and stealing things from tourists" (`loc 1658540213`, `3111281418`); "They only come out at night, so be ready!"; "Remember they only come out at night!" (`loc 2492132900`). The wiki agrees ("only found during night"). The day-night cycle length and spawn counts are unknown. | [client] [video nDX88pgHwSg/00068] [wiki] |
| Story hook | Snatching Snappers (crays) raid the Seaside beaches each night and steal from tourists. | [client `loc 3225763451`, `1397454576`] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 5 or higher** (difficulty 3). Client panel line "Requires a level 5 combat job" (`loc 2145075890`) is generic, not tied to this battle by name. The wiki infobox says "Levels 5 and higher". | [wiki] [client, generic] |
| Members only | Not tagged on the wiki. Wiki shows a members coin bonus. | [wiki] |
| Unlocking quest | None needed to click the node as far as any source says. The Seaside quest **Finding the Raiders** sends players here. | [client] [video] |
| Group size | Unknown. | gap |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main | **"Defeat the Cray Snappers."** (`loc 2923271887`; the wiki has the same text). Hint: "To stop the snatching you will need to defeat the Cray Snappers" (`loc 1017778735`). | [client] [wiki] |
| Bonus | **"Bonus: Pop all of the bubbles"** (`loc 3583207144`). The wiki adds a count of 5, "(5)". Hint: "Find a way to pop the bubble!" (`loc 31303853`). Internal groups: "Snatching Snappers bonus objectives" (`loc 2372301333`) and "Snatching Snappers - Bubble Pop" (`loc 2637611448`). What the bubbles are and how they are popped is unknown. | [client] [wiki] |
| Quest-side objectives | "Defeat the Snatching Snappers" (`loc 110118614`), "Defeat Snatching Snappers" (`loc 4142578463`) | [client] |

## Enemies and boss

| Role | Who | Sources |
|---|---|---|
| Enemy | **Cray Snapper** (`loc 1585721449`, game data `3120660947`). Seen with a "Cray Snapper" nameplate in the quest run. | [client] [video nDX88pgHwSg/00068] |
| Boss | None named. | gap |

Counts and waves are unknown. One results screen counts "Enemies Defeated 10". [video nDX88pgHwSg/00080]

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "74 Stars" | [wiki] |
| Prize wheel | One wheel per job: 5 items plus a **2 Coins** slot (as the wiki writes it) | [wiki] |
| Members bonus | "Members Only Bonus: 31 Coins" | [wiki] |
| Results screen (video) | "YOU WIN!", "Enemies Defeated 10: 3000", "Time Bonus 1:07: 67000", "Knockouts Remaining 3/3: 25000", "Goals 1/1: 5000", "TOTAL SCORE 100000". The run shows one goal done, so it did not clear the bonus. Note **3** knockouts here, not 5. | [video nDX88pgHwSg/00080] |
| Quest-run chat line | "You receive 10 stars and 7 coins" appears with Finding the Raiders in a low-confidence frame; which reward it belongs to is not clear. | [video nDX88pgHwSg/00068] |

### Prize wheel by job

Every item name exists in the client locale exactly as written. [client, checked by exact name] Every wheel also holds Battle Item Mystery Pack and 2 Coins. [wiki]

| Job | Items | Source |
|---|---|---|
| Archer | Archer's Composite Bow of Stunning, Archer's Power Shard of Vitality III, Archer's Ring of Regeneration III, Drifting Archer Boots | [wiki] |
| Brawler | Brawler's Anvil Hammer of Cheapshot, Brawler's Power Shard of Vitality III, Brawler's Ring of Regeneration III, Catchweight Brawler Boots | [wiki] |
| Medic | Emergency Medic Boots, Medic's Bonesaw of Vitamins, Medic's Power Shard of Vitality III, Medic's Ring of Regeneration III | [wiki] |
| Ninja | Ninja's Power Shard of Vitality III, Ninja's Ring of Regeneration III, Ninja's Scythe of Shadow Armies, Nusumi Ninja Tabi Boots | [wiki] |
| Warrior | Combat Bonus Warrior Hightops, Warrior's Battle Hammer of Whirlwind, Warrior's Power Shard of Vitality III, Warrior's Ring of Regeneration III | [wiki] |
| Wizard | Scholar Wizard Shoes, Wizard's Bone Wand of Vortex, Wizard's Power Shard of Vitality III, Wizard's Ring of Regeneration III | [wiki] |

## Related quests and NPCs

| Quest | Giver | What it asks | Sources |
|---|---|---|---|
| **Finding the Raiders** | [[Duane]], Seaside | Title `loc 3524069531` (game data `922726292`). Duane: "Every night, I see Snatching Snappers raiding my town, stealing things. If you think you can do the job, I'll hire you to defeat them. They only come out at night, so be ready!" (`loc 3225763451`). Summaries: `loc 1835685632`, `1361980182`, `3246968964`, `3585979347`. Return step "Return to Duane and let him know you have defeated a few Snatching Snappers." (`loc 2932791523`). Video chat lines: "Finding the Raiders Quest Started!", "Goal Update: Defeat Snatching Snappers", then "New Goal: Return to Duane". Duane's repeat line: "Come back again tomorrow night and beat up some more of them!" so it is likely repeatable nightly (not confirmed). Rewards unknown (see Rewards). | [client] [video nDX88pgHwSg/00068, Yoxys3l0-vg/00000] |
| Seaside fight hint | Not stated | "If you're looking for a fight, at night Snatching Snappers are always attacking and stealing things from tourists. Feel free to take a few out. Also there are hostile craymen along the beach to the north in the Cray Caves." (`loc 1658540213`) | [client] |

NPCs: [[Duane]]. Zone: [[NPCs - Seaside]]. Related: [[Cray Caves]], [[Seaside Swoopers]].

## Conflicts

1. **Bonus count.** The wiki says "Pop all of the bubbles. (5)". The client text has no count: "Bonus: Pop all of the bubbles" (`loc 3583207144`). The count 5 is wiki-only.
2. **Knockouts.** Other pages in this guide infer 5 knockouts per player from "Knock outs Remaining N/5". This results screen reads "3/3". The number may depend on the battle or party.

## Gaps

- Cray Snapper count, waves and abilities; what the bubbles are.
- Real star and coin payout (the wiki's "2 Coins" slot and the video's "Enemies Defeated 10" are the only figures).
- Group size and whether the node is shared or per player.
- Quest rewards for Finding the Raiders.
- The exact night window.
