---
type: content-dungeon
zone: Highroad Junction (Wilds)
combat: true
---

# Hewey's Escape

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

> **Removed content.** The wiki marks this dungeon "No longer in-game" and "removed" (page revision 2025-02-05). The client still holds its text and the recordings show it working, so it existed in the build the project targets. It is also effectively a **Brawler job tutorial battle**, tied to the Brawler starter quests.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Hewey's Escape" (game data `loc 241400652`, UI `loc 2001235230`) | [client] |
| Panel description | "Hewey the Bear escaped the local zoo! Knock some sense into Hewey so he'll return to the zoo." (`loc 3003893450`) | [client] [wiki] |
| Journal description | "Hewey the Bear has escaped from the zoo and refuses to leave this cave. Knock him out so Harold can take him home." (`loc 374052479`) | [client] |
| Type | Cave battle, difficulty 1 | [wiki] |
| Entrance | A gate in **Highroad Junction**, "the Bear Cave", east of Harold. Client prompts: "Click on the gate to Hewey's Escape that's east of Harold to start the battle!" (`loc 3403732640`) and "Interact with the gate to the Bear Cave east of Harold to start the battle!" (`loc 3216721027`). The wiki says it had an Atlas icon. | [client] [wiki: Hewey's Escape, Highroad Junction] [video fktAs_GRrjM/00388, 00414] |
| Story hook | Hewey the Bear escaped from the Durango Zoo and is angry. He hurt himself and will not let anyone near. Harold, an animal-control agent from the zoo, went after him and is trapped or cornered in the cave. | [client `loc 3137596165`, `1666998872`, `973814746`] |
| Placement | Highroad Junction is where new players are guided (Merigold) in the older starter flow; the wiki notes players now start in Cobblestone Village. | [client `loc 260624254`] [wiki: Highroad Junction] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **None.** Difficulty 1. In the Brawler starter quest the player is on the Adventurer job (level 1 to 2) when sent here, and the game switches the job on entry (message "Your job has been switched because you are entering a combat area.", seen on another dungeon). | [wiki: Dungeons] [video aveqYa4AVk8/00020, 00082, 0LoQe5VZ0PY/00034] |
| Seen playing at | Brawler 5 | [video DheErd-aDpQ/00058, 00068] |
| Members only | No | [wiki] |
| Unlocking quest | The battle is started from quests: "Hewey's Escape" (objective "Click on the gate to Hewey's Escape") and "Brawler: Hewey's Escape!". See Related quests. | [client] [video fktAs_GRrjM/00388, aveqYa4AVk8/00020] |
| Group size | Unknown. The recordings show a solo run. | gap |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main (only goal) | **"Knock out Hewey the Bear!"** (the recordings show it without the "!", possibly cut off). Panel shows "Goals (1)". There is no bonus goal. | [client `loc 2247616509`] [video fktAs_GRrjM/00418, DheErd-aDpQ/00058] [wiki] |
| Quest-side objective | "Save Harold from Hewey the Bear" (`loc 1107073768`) | [client] [video aveqYa4AVk8/00020] |
| Client alt | "Wear out Hewey" (`loc 3355293881`) | [client] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Hewey the Bear** (`loc 2845865015`, game data `3066097249`) | The only enemy named. "Hewey the Bear roars angrily!" (`loc 1011460884`). A dev string reads "Small damage and knockback for encounter - Hewey's Escape." (`loc 1926348349`). | [video fktAs_GRrjM/00416, 00418] [client] [wiki] |
| Friendly NPC | **Harold** | Coaches the player through chat lines (below). | [video fktAs_GRrjM/00418 to 00486] [wiki] |

### Harold's scripted lines (video, in order)

1. "Don't just stand there! HELP ME!!!" / "AHHHHHHHHHH! HELP ME!!!" / "HELLLLLLLLLLLLLP MEEEEEEEEEEE!!!"
2. "Phew! You got here just in time!" / "Attack the bear!!!"
3. "Pick this up for a special attack!"
4. "Don't Stop! Keep attacking!" / "Are you okay!? Try using a potion!"
5. "You have 100 Energy!" / "Hit the [2] key on your keyboard!" (later variant: "...to use your super!")
6. "Pick up the power up and press [3] on your keyboard to use it."
7. "Looks like you have full energy!" / "Use a big attack!"
8. "Woo-hoo! You knocked him out!" / "...does it feel drafty in here to you?" / "Umm... I'll stay here and take care of Hewey..." (`loc 2785103551`)

Sources: [video fktAs_GRrjM/00414 to 00486, DheErd-aDpQ/00058, 00068]

### Combat tutorial overlay

During this battle a "How to Rule in Combat!" panel shows Attack, Super Attack and Power-up, with the hints "Use arrow keys to move around." and "Use potions when your health is getting low." A second panel explains drops: "Power-ups are items that drop off of enemies during combat. The Health and Energy power-ups are used automatically. Use the others by clicking on the third slot where the arrow is pointing." The five power-ups named are Health (gives you health), Energy (replenish energy), Flame Wave (damages nearby enemies), Earth Shard (damages and stuns enemies) and Super Shield (makes you invulnerable). [video fktAs_GRrjM/00404, 00426]

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "10 Stars" (wiki wording, the smallest in this guide) | [wiki] |
| Coins on the prize wheel | **36 Coins** slot on every wheel | [wiki] |
| Results | "YOU WIN!" then a prize wheel with a Coins slot | [video fktAs_GRrjM/00484, 00486] |
| Prize won in video | "Adherent Wizard Gloves", tooltip "Required: Level 10, Wizard" (small print, may be misread), won by a player who was doing the Brawler quest. See Conflicts. | [video fktAs_GRrjM/00504] |
| Quest reward text | "Now that Hewey is all worn out, meet Harold back at his wagon in Highroad Junction to collect your reward." (`loc 3410136663`). Item "Hewey's Bear Claw" (`loc 859367774`, `4174843352`) exists, link unconfirmed. | [client] |
| Achievements | None. This dungeon is absent from the wiki's encounter achievements list. The client has "Win Combat Encounter Hewey's Escape" (`loc 1456920798`), a quest objective. | [client] [wiki: Achievements] |

### Prize wheel by job

Each wheel has the 36 Coins slot, a Small Health Potion and one glove. [wiki]

| Job | Glove item (wiki) | Client name |
|---|---|---|
| Archer | Hen Feather Archer Gloves | same |
| Brawler | Bell Brawler Gloves | "Saved by the Bell Brawler Gloves" |
| Medic | Stable Condition Medic Gloves | same |
| Ninja | Kusa Ninja Armwraps | same |
| Warrior | Standard Action Warrior Gauntlets | same |
| Wizard | Novice Wizard Gloves | same |

## Related quests and NPCs

| Quest text (client) | Id | What it asks | Sources |
|---|---|---|---|
| **Brawler: Hewey's Escape!** | title `loc 3490633876`/`4204217995`; text `1605633744` | "As a brawler, help Archy save Harold from Hewey the escaped bear in Highroad Junction." Objectives: "Talk to Archy" (`loc 300747673`), "Save Harold from Hewey the Bear". Archy says "My boss, Harold, went after Hewey the bear, but now he's trapped in that cave behind me. Can you save him?" (`loc 973814746`). | [client] [video aveqYa4AVk8/00000 to 00090] |
| **Hewey's Escape** (quest) | title `loc 2001235230` | "Harold from the Durango Zoo is looking for some help! He needs to knock out an escaped bear! ... go find Harold, he's to the east!" (`loc 3137596165`). Objective "Click on the gate to Hewey's Escape" (`loc 1714966968`). Goal-complete banner "Brawler: Hewey's Escape". | [client] [video fktAs_GRrjM/00308, 00388] |
| Harold as trainer | `loc 1378400831`, `3003135002` | "I've helped raise Hewey from nothing but a little cub ... Will you help me knock him out ... I can even teach you the brawler job." and "Here to help with my bear problem? Great! But first let me teach you the brawler job." | [client] |
| Merigold's pointer | `loc 2362498448` | "Merigold has pointed you towards Harold, an animal control agent who's looking for some help with a recently escaped bear!" | [client] |
| Zoo errand | `loc 1032780066` | "Hewey needs to be taken back to the Durango Zoo, but the only way to do that is if he's really worn out. Go to Hewey's Escape at Highroad Junction ..." | [client] |
| Follow-on | `loc 23059866`, `1289334862` | Hewey thanks; Harold's twin brother Gerold has a wolf problem near Snowhill. | [client] |

NPCs: [[Harold]] (Brawler Trainer, Highroad Junction), [[Assistant Archy]], [[Merigold]], [[Gerold]]. Zone: [[NPCs - Highroad Junction]]. Job: [[Brawler]].

## Conflicts

1. **Prize item.** Video shows the won prize as "Adherent Wizard Gloves" while the wiki lists Novice Wizard Gloves for the Wizard wheel. The Adherent Wizard set is listed on the wiki for the [[Bandit Hideout]] and [[Croaking Vale]] bonus rewards, not for this dungeon. The tooltip read "Required: Level 10". Either the live prize table was different when the video was recorded, or the frame is misread. The player was doing the Brawler quest, so a Wizard glove is also odd. Unresolved; video recorded, wiki kept as the list.
2. **Brawler glove name.** The wiki writes "Bell Brawler Gloves". The client item is "Saved by the Bell Brawler Gloves".
3. **Availability.** The wiki says removed. The client and video show it live.

## Gaps

- Exact star and coin payout; the whole prize table (the wiki list may be partial).
- Group size and whether it can be entered without the quest.
- Hewey's health and abilities (none are named in the client strings found).
- When it was removed and what replaced it in the Brawler starter flow.
- Whether "Hewey's Bear Claw" is the quest reward.
