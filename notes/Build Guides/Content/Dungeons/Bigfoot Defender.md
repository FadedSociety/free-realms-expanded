---
type: content-dungeon
zone: Wilds (Greenwood Forest, southwest, near Camp Chatty and Camp Robbie)
combat: true
---

# Bigfoot Defender

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

> Bigfoot Defender is a **quest-triggered special-spawn battle**, not an Atlas dungeon. The wiki files it under *Battle*, *Special Spawn* and *Wilds*. It is the Camp Robbie mirror of [[Bigfoot's Lair]]: there you hunt Bigfoot, here you **protect** him from the Camp Chatty counselors. [wiki] [client]

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Bigfoot Defender" (game data `loc 2068880740`, UI `loc 3712650989`) | [client] |
| Panel description | Not found in the client. The wiki infobox gives none for this page. | gap |
| Type | Forest battle, difficulty 3 | [wiki] |
| Entrance | Started by clicking **Bigfoot** in the southwest Wilds, near Camp Chatty. Bigfoot only spawns while the player holds the quest **Bigfoot's Endangered** (wiki). | [wiki: Bigfoot Defender] |
| Story hook | Camp Robbie's "guardian" Bigfoot is being hunted by the Camp Chatty counselors. The player, as a Camp Robbie minion, chases the counselors off. Bigfoot is really Robbie in costume, which the camp insiders hide. | [client `loc 3316236307`, `1243160555`, `1148190599`] [wiki] |
| Time frame | Camp Robbie came into the game on 4 June 2010 (wiki). A "summer combat event - bigfoot rock throw" string exists (`loc 1801534029`). | [wiki: Robbie] [client] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 5 or higher** (difficulty 3). Client panel line "Requires a level 5 combat job" (`loc 2145075890`) and "You must have a combat job of at least level 5 to play this battle" (`loc 3173661191`) are generic strings, not tied to this battle by name. | [wiki: Battles, Bigfoot Defender] [client, generic] |
| Members only | Not tagged on the wiki. Unconfirmed. | [wiki] |
| Unlocking quest | **Bigfoot's Endangered** (wiki). Client title `loc 54624951` (game data) and `3740470622` (UI). Related quest **Bigfooted Friend** (title `loc 1628471361`). See Related quests. | [wiki] [client] |
| Group size | Unknown. | gap |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main | **Four knock-outs**, one per counselor. The wiki lists the goals as "Knock out Counselor Kaela", "Knock out Counselor Edmond", "Knock out Counselor Tyra" and "Knock out Counselor Jake" (each exact in the client: `loc 314220268`, `1141526814`, `1564263106`, `22760277`). The goals panel count was not seen. | [client] [wiki] |
| Related client goal | "Don't let Bigfoot get knocked out" (`loc 817448663`). This looks like a fail condition or a goal of this battle, but the client does not say so. | [client] |
| Bonus goal | None listed. No "Bonus Objective" string was found. | [wiki] [client] |

## Enemies and boss

| Role | Who | Sources |
|---|---|---|
| Player-side NPC | **Bigfoot** (`loc 524303829`, `1698506345`) | [wiki] [client] |
| Enemies (wiki "Mobs") | **Counselor Tyra**, **Counselor Edmond**, **Counselor Jake**, **Counselor Kaela**, plus **bears**. Kaela is a pixie and Jake a human, per their wiki pages. | [wiki] [client for the names] |
| Dialogue strings that may belong here | "There he is! It's Bigfoot! Let's take him down!" (`loc 2404845728`); "I'm not scared of you, Bigfoot! Come out!" (`loc 3421589477`). Speaker and battle not stated. | [client] (weak) |
| Bigfoot's abilities | "Bigfoot Thrash", "Bigfoot Rock Throw", "Bigfoot Bash", "Bigfoot Rage Resist" (`loc 130524752`, `328597092`, `2135435772`, `2289119840`). Names only; use in this battle unproven. | [client] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "130 Stars or less" (as written on the wiki, meaning unclear) | [wiki] |
| Prize wheel | Each job has a **990 Coins** slot. The items and the 990 figure are identical to [[Bigfoot's Lair]]. | [wiki] |
| Members bonus | None listed. | [wiki] |
| Quest rewards | Unknown for Bigfoot's Endangered and Bigfooted Friend. | gap |

### Prize wheel by job

Every item name exists in the client locale exactly as written. [client, checked by exact name] The wiki lists the same items for [[Bigfoot's Lair]]; the two pages may be sharing one table (unproven).

| Job | Items (plus 990 Coins) | Source |
|---|---|---|
| Archer | Camp Chatty Banner, Camp Chatty Tent, Camp Robbie Flair Shard, Chugawug Totem, Mushroom Spitfire, Tasty Marshmallow | [wiki] |
| Brawler | Camp Robbie Banner, Camp Robbie Flair Shard, Camp Robbie Tent, Caterpillar Spitfire, Treeble Treat, Troll Totem | [wiki] |
| Medic | Camp Chatty Tent, Camp Robbie Flair Shard, Owl Totem | [wiki] |
| Ninja | Camp Chatty Flair Shard, Camp Robbie Tent, Chatty Totem | [wiki] |
| Warrior | Camp Chatty Flair Shard, Camp Chatty Tent, Robbie Totem | [wiki] |
| Wizard | Camp Chatty Banner, Camp Chatty Flair Shard, Camp Robbie Tent, Man-Eating Plant Totem, Nutty Spitfire, Tasty Marshmallow | [wiki] |

## Related quests and NPCs

Quest bodies are in the locale without titles beside them, so descriptions are listed by id. The wiki quest pages for these titles are not in the archive.

| Quest | Giver | Text | Sources |
|---|---|---|---|
| **Bigfooted Friend** | [[Robbie]] sends the player to [[Counselor Jacen]] | Robbie: "Well, now that you're my official minion, there's a secret I can share with you ... Talk to Counselor Jacen, and have him tell you all about "Bigfoot."" (`loc 860443517`). Objective "Ask Counselor Jacen about "Bigfoot"" (`loc 788218943`). Jacen: "...those violent haters from Camp Chatty are harassing him. If you really want to help, chase off Chatty's camp counselors!" (`loc 3316236307`). The wiki lists the quest on Robbie's page. | [client] [wiki: Robbie] |
| **Bigfoot's Endangered** | Camp Robbie side (giver not stated; likely Jacen) | "Counselor Jacen is worried about his friend, Bigfoot. It seems that the camp counselors from Camp Chatty just won't leave him be! Help him out." (`loc 1515101801`); "Those mean ol' camp counselors from Camp Chatty just won't quit! ... Find where they've cornered Bigfoot, and chase off those camp counselors!" (`loc 1148190599`); "Bigfoot's just misunderstood. Save him from those violent camp counselors from Camp Chatty!" (`loc 2268053794`); "Help Bigfoot!" (`loc 1798556775`). Repeat text: "The counselors from Camp Chatty are pestering Bigfoot. Chase them off again." (`loc 1526180155`). Turn-in: "Hopefully, Bigfoot will be OK for today. If those counselors from Camp Chatty come back tomorrow, I'll let you know!" (`loc 820703868`), so likely a daily repeat. | [client] [wiki] |
| Camp Chatty side | [[Counselor Edmond]] | **Confronting Bigfoot** and **Putting Your Foot Down** lead to [[Bigfoot's Lair]], the opposite battle. | [client] |

NPCs: [[Robbie]], [[Counselor Jacen]], [[Counselor Edmond]], [[Counselor Tyra]], [[Counselor Jake]], [[Counselor Kaela]]. Zone: [[NPCs - Greenwood Forest]].

## Conflicts

1. **Shared prize table.** The wiki prints the same 990 Coins wheel here and on Bigfoot's Lair, and the same figure is not seen on any other level-5 entry (26 to 117 coins). It may be a copy of one table. No video confirms either.
2. **Quest link.** Only the wiki says Bigfoot's Endangered is the spawn quest. The client strings fit it (counselors cornering Bigfoot) but name no battle.
3. **Which counselors fight.** Edmond and Tyra appear as allies in Bigfoot's Lair (video) and as enemies here (wiki). Both can be true; neither is confirmed by video for this battle.

## Gaps

- Panel description, goals panel count, any bonus goal.
- Whether "Don't let Bigfoot get knocked out" is a fail condition.
- Real star and coin payout; group size; bear count; counselor abilities.
- Quest rewards and cooldown for Bigfoot's Endangered.
- No video of this battle was found.
