---
type: content-dungeon
zone: Blackspore Swamp
combat: true
---

# Cracked Claw Caverns

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Cracked Claw Caverns" (game data `loc 2805440117`, UI `loc 2730737157`, `3619527335`) | [client] |
| Panel description | "The cray in these caverns are extremely aggressive lately! Find their leader Cracked Claw and defeat him to lessen their aggressive nature!" (`loc 3604434679`) | [client] [wiki] |
| Type | Cave dungeon, difficulty 1 | [wiki] |
| Entrance | Click the barrier in **Blackspore Swamp**. Atlas icon exists. Client: "Vasher is located outside of Cracked Claw Caverns, in Blackspore" (`loc 3613074255`). In-dungeon teleports are named "Cracked Claw Teleport" (`loc 881450501`, `3038927058`). | [wiki] [client] |
| Story hook | The swamp cray (crayfish) in the caverns have become aggressive. Many explorers never returned from the depths. Their bones and spirits lie inside, and the ancient cray Cracked Claw rules the cray. | [client `loc 611115531`, `2866249235`] [wiki: Cracked Claw] |
| Related | Cracked Claw also hoards treasure: "Search for Cracked Claw's hoard" (`loc 2104817905`), "a giant pearl ... a prized trophy of old Cracked Claw himself" (`loc 2932483582`). | [client] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **None.** Difficulty 1 "can be entered by anyone". | [wiki: Dungeons] |
| Seen playing at | Brawler 10, Ninja (level not shown), Ninja 2 | [video NtXTmB-wPW0/00028, 0egOV5hzmjI/00014, -5UUuwaVXnM/00012] |
| Members only | No. Members get an extra coin bonus. | [wiki] |
| Unlocking quest | None. Several job quests send players in (see Related quests). | [client] |
| Group size | Unknown. A "full group" achievement exists (`loc 2428865771`, "Group: Cracked Claw Caverns" `loc 3474526595`). | [client] [wiki: Achievements] |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main | **"Defeat the ancient cray, Cracked Claw!"** (`loc 1849027452`). An alternative "Find and defeat the ancient cray, Cracked Claw!" exists (`loc 1902746988`). | [client] [video NtXTmB-wPW0/00028, -5UUuwaVXnM/00012] [wiki] |
| Bonus | **"Bonus: Release the trapped spirits of the explorers lost within Cracked Claw Caverns!"** with a counter shown as **0/5** in the recording, rising to 5/5. Client string: `loc 2018071426`. | [client] [video NtXTmB-wPW0/00028 to 00116] |
| Panel shows | "Goals (2)" | [video NtXTmB-wPW0/00028] |
| Internal bonus group | "Cracked Claw Caverns Bonus Objective" (`loc 348805156`) | [client] |

How the bonus works: the player interacts with a **Lost Explorer Bones** object ("Press X or mouse"). The log then reads "The bones crumble and a tormented spirit materializes!" and the counter goes up by one. The spirit is a "Trapped Spirit" (`loc 1499775690`) or "Lost Explorer" (`loc 3037496321`) entity. A recording shows a hostile red nameplate "Lost Explorer" after a spirit appears. Whether the spirit must be defeated, or only the bones touched, is not shown. [video NtXTmB-wPW0/00036, 00046, 00078, 00082, 00084]

Quest versions of the same props: "Collect lost explorer bones" (`loc 2177515076`), "Lost Explorer Bones" (`loc 1202637199`, game data `1451080438`).

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Cracked Claw**, the ancient cray (`loc 4053599865`, game data `3157544754`) | "Cracked Claw looks at you with rage." (`loc 1211155113`). Dev string: "cracked claw cavern - boss ability" (`loc 2406144006`). A boss health bar labelled "Cracked Claw" shows. | [client] [video NtXTmB-wPW0/00140] |
| Enemies | Giant Toads, Cray, Lost Explorers, Elder Swamp Cray | Client names: "Swamp Cray" (`loc 1734500867`), "Swamp Cray Broodling" (`loc 3122193249`), "Elder Swamp Cray" (`loc 3403234900`, game data `2781098192`). A large red caption "...p Cray" was also seen. | [wiki] [client] [video NtXTmB-wPW0/00042] |
| Spawner | A **tunnel** object. Hint text: "Attack the tunnel to stop the monsters from coming!" | Destroying it stops spawns, as with "Wolf House" in [[Tavern Cellar]]. | [video NtXTmB-wPW0/00078, 00082] [client `loc 3907530502`] |

### Scoring seen on the results screen

One run. [video 0egOV5hzmjI/00056]

| Line | Value |
|---|---|
| Enemies Defeated | 33, worth 66,500 points |
| Time Bonus | 1:29.80, worth 88,533 points |
| Goals: Knockouts Remaining | 2/5, worth 91,666 points |

Total is not visible. The weights differ from [[Briarheart Caverns]] (3,000 per opponent, 50,000 per remaining knockout), so scoring is probably tuned per dungeon or per difficulty.

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "58 Stars or less" (wiki wording) | [wiki] |
| Coins on the prize wheel | **26 Coins** slot on every wheel | [wiki] |
| Members-only bonus | **77 Coins** (the client does hold a generic string "Bonus: 77 Coins!", `loc 2278433839`, not tied to this dungeon) | [wiki] [client] |
| Coin drops | "You receive 15 coins" | [video NtXTmB-wPW0/00042] |
| Bonus-goal clothing | One piece of the job's set, random slot and colour, for everyone in the instance | [wiki] |
| Achievements | Bronze (5 pts) "Complete the Cracked Claw Caverns encounter." (`loc 999015670`), silver (10 pts) with a full group (`loc 2428865771`); "Win Combat Encounter Cracked Claw Caverns" (`loc 3388071600`) | [client] [wiki: Achievements] |

### Prize wheel by job

Each wheel also has the 26 Coins slot. Every name exists in the client. [wiki] [client names]

| Job | Items |
|---|---|
| Archer | Archer's Bow of Blizzards, Archer's Power Shard of Vitality I, Archer's Ring of Regeneration I, Battle Item Mystery Pack, Hen Feather Archer Tunic |
| Brawler | Battle Item Mystery Pack, Brawler's Mallet of Rumbling, Brawler's Power Shard of Vitality I, Brawler's Ring of Regeneration I, Saved by the Bell Brawler Shirt |
| Medic | Battle Item Mystery Pack, Medic's Power Shard of Vitality I, Medic's Reflex Hammer of Vitals, Medic's Ring of Regeneration I, Stable Condition Medic Shirt |
| Ninja | Battle Item Mystery Pack, Kusa Ninja Gi, Ninja's Training Sword of 1000 Storms, Ninja's Power Shard of Vitality I, Ninja's Ring of Regeneration I |
| Warrior | Battle Item Mystery Pack, Standard Action Warrior Tunic, Warrior's Cudgel of Cleaving, Warrior's Power Shard of Vitality I, Warrior's Ring of Regeneration I |
| Wizard | Battle Item Mystery Pack, Novice Wizard Short Cloak, Wizard's Power Shard of Vitality I, Wizard's Ring of Regeneration I, Wizard's Sparkle Twig of Glaciers |

Bonus clothing sets by job: Archer Hen Feather, Brawler Saved by the Bell, Medic Stable Condition, Ninja Kusa, Warrior Standard Action, Wizard Novice. [wiki]

## Related quests and NPCs

Quest titles are not stored beside descriptions in the locale except where noted, so each row gives the description id.

| Quest text (client) | Id | What it asks | Sources |
|---|---|---|---|
| **Burial Service** | title `loc 2725790777`/`165850984` | Vasher wants someone to recover the bones of fallen explorers (`loc 1310442148`, `1640485516`, `4055360652`). "You up for a little excitement? Cracked Claw Caverns is a dangerous place ..." (`loc 611115531`). Objective in video: "Collect last explorer bones". | [client] [video 0egOV5hzmjI/00014] |
| **Ninja: Strike from the Shadows** | title `loc 2580872460`/`1535781811`; text `1599642277` | Chloe's ninja training: "Head to Cracked Claw Caverns and defeat the inhabitants within." (also `3658895198`, `3000775896`). New goal in video: "Return to Chloe in Blackspore". | [client] [video 0egOV5hzmjI/00014] |
| "Bullseye has asked you to defeat some cray in the Cracked Claw Caverns and bring back proof of your archer skills." | `1800358254` | Archer quest (also `106181628`, `704225054`) | [client] |
| "...retrieve radiant gems from inside the Cracked Claw Caverns of Blackspore" | `2977396604` | Brawler quest (also `3312881182`, `2596294375` completion, Petra `1105828342`) | [client] |
| "Search the Cracked Claw Caverns in Blackspore for a giant pearl." | `789473883` | Pearl quest (also `2932483582`, `1107631518`, `2354008992`) | [client] |
| "Look for Bart's "lost" badge in the Cracked Claw Caverns." | `2459692326` | Bart's Badge (`loc 3217266699`) | [client] |
| "Find the Elder Swamp Cray in the Cracked Claw Caverns and defeat him." | `3676395674` | Kill the elder cray (also `3246284446`, `3790848322`, `1519774645`) | [client] |
| Crystal samples | `809438111`, `1250819232`, `1939959567` | Bring crystals back (Oswald in Mire's Edge values them) | [client] |
| "Donovan says that people being trapped in Cracked Claw Caverns is bad for business. Clear out these crabs..." | `991685528` | Kill crabs | [client] |

NPCs: [[Vasher]], [[Chloe]], [[Bullseye]], [[Petra]]. Zone: [[NPCs - Blackspore Swamp]].

## Conflicts

1. **Bonus count.** The wiki says the bonus has 6 spirits. The recording shows the counter ending at 5 (0/5 up to 5/5). The video number (5) is used.
2. **Wiki foe list.** The wiki lists "Cray" and "Elder Swamp Cray". The client also names "Swamp Cray" and "Swamp Cray Broodling". Treat them as the same cray family.

## Gaps

- Group size.
- Star payout in video.
- Whether the spirits must be fought after the bones are touched.
- Boss health, boss ability effects and how often the tunnel spawns.
- The unexplained "Hammer Essence of Water" text seen on screen in one frame (probably a collection or loot item, role unclear). [video NtXTmB-wPW0/00028]
- Which quest owns each description above (titles) and their rewards.
