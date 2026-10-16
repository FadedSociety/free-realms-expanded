---
type: content-dungeon
zone: Merry Vale
combat: true
---

# Den of Secrets

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Den of Secrets" (game data `loc 1242062165`, UI `loc 1903380661`, `2107689987`), "The Den of Secrets" (`loc 1324221923`, `4230705426`). Two "NOT USED" entries exist (`loc 2614537499`, `2906953785`). | [client] |
| Panel description | "Something dark and sinister is afoot in the Den of Secrets. Many have gone missing and are suspected to be inside! Free them and defeat whoever is responsible for this!" (`loc 3571165291`) | [client] [wiki] |
| Type | Cave dungeon, difficulty 2 | [wiki] |
| Entrance | Click the **barrier** in **Merry Vale**. The client places it "in the back of the cave in northern Merry Vale" (`loc 1819244747`) and "the cavern north of Merry Vale" (`loc 1876015264`). A video HUD banner reads "Den of Secr..." (cut off by the timer). Atlas icon exists. | [wiki] [client] [video Sy-W_AHbwcw/00018] |
| Story hook | Fairy cultists have been kidnapping children and pixies from Merry Vale. Their hideout is a cave "den". Captives are turned into shamblers ("You won't turn me into a shambler!"). The client text: "Deep in the cultists' cave is an area that is more protected than the rest. It's a den - A DEN OF SECRETS! Sneak into the cultist Den of Secrets and find out who's behind all of these kidnappings!" (`loc 3052313041`). "Evil magic and mushroom monsters are creating shamblers! Venture in and stop them!" (`loc 2380049548`). | [client] [video Sy-W_AHbwcw/00010, 00060, 00102] |
| Tooltip | "Stop picking on the music! It's all the rage in Den of Secrets!" (`loc 24395095`), a jokey line. Not a rule. | [client] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **None** (difficulty 2: "recommended levels 3 and higher, but can be entered by anyone"). No level-3 string exists in the client. | [wiki: Dungeons] [client] |
| Seen playing at | Warrior 7 | [video Sy-W_AHbwcw/00010 to 00398] |
| Members only | No. The wiki lists a members-only coin bonus of 73. | [wiki] |
| Unlocking quest | None. A Merry Vale quest chain with Mayor McFairy sends players in (see Related quests). | [client] |
| Group size | Unknown. "Complete the Den of Secrets encounter with a full group." (`loc 817888718`) and "Group: Den of Secrets" (`loc 897743126`) exist. | [client] |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main | **"Defeat Illusionist Liana!"** (`loc 856066364`). Forms: "Defeat Liana" (`1938450632`), "Defeat Illusionist Liana" (`3724491489`), "Defeat Liana within the Den of Secrets!" (`495320710`), "Find the mysterious Illusionist Liana and defeat her!" (`2256745373`). | [client] [video Sy-W_AHbwcw/00010 to 00398] [wiki] |
| Bonus | **"Bonus: Destroy the mysterious pillars within the caverns!"** with a counter 0/10 to 10/10. Client strings: "Destroy the mysterious pillars within the caverns!" (`loc 3070263815`), "Find the strange runed pillars throughout the Den of Secrets and destroy them!" (`loc 2435444316`). Counter values seen: 0, 1, 2, 3, 5, 8, 9. The goal is ticked before the boss. The first frame of the run reads "cavern" in the singular. | [client] [video Sy-W_AHbwcw/00010, 00018, 00060, 00098, 00142, 00312, 00342, 00366] [wiki: "(10)"] |
| Panel shows | "Goals (2)" | [video Sy-W_AHbwcw/00018] |
| Internal groups | "Den of Secrets - Bonus Objective" (`loc 3236142750`) and "Den of Secrets Bonus Objective" (`loc 4290138172`) | [client] |
| Win text | "Win Combat Encounter Den of Secrets" (`loc 1323605988`) | [client] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Illusionist Liana** (`loc 1718583424`, `773551638`) | Shows a boss health bar. Lines in order: "Someone's here! Get rid of them quickly!", "You'll serve me as a shambler!" (`loc 1438074561`), "Shadows! Protect me!", "Liana calls on magical guardians!" (`loc 2847304587`), "Liana vanishes! Her illusions remain!" (`loc 1867480605`). Also "Liana reappears!" (`1947804719`). Ability and object names: "Liana's Illusion" (`573399177`, `1118846632`), "Liana's Mana Orb" (`1313784871`), "Liana's Mana/Health Orb" (`2817704573`), "Liana's Crystal" (`1466546315`, `4073202367`), "den of secrets 117: Liana Teleport back / out" (`1373470542`, `2462338955`), "den of secrets - boss primary attack" (`1736350820`), "elite attack" (`2507190634`), "basepop nuke" (`2098768805`), "npc spawner ability spawns dudes" (`1313792123`), "den_of_secrets 117: Shambler Spore Cloud" (`1350530913`). Mechanics (illusions, orbs, guardians, teleporting) are inferred from these names only. | [client] [video Sy-W_AHbwcw/00366 to 00398] [wiki] |
| Enemies | **Transformed Shamblers** (`loc 1411463193`, `341228863`), **Fairy Cultists** (`695744272`, `2180850741`), Fairy Guards (`1971096666`), Fairy Acolytes (`2249696127`), Necrowart Zombies (`loc 4143459490`, `1923750772`, `1557261858`) and Sporelings | Video confirms Transformed Shambler and Fairy Acolyte nameplates and the Fairy Cultist line "Right away, mistress! Sporelings, with me!" (`loc 1432569317`). Enemy speech: "You won't leave with our secrets!", "Once you enter this place, you can't leave!", "Get a cage ready for the intruder!" Transformed Shambler: "Save us... please! Defeat Liana!" (`loc 535476607`) and "She is... behind... this fake wall! I can't...control my actions!". The necrowart zombie enrage ability: "becomes enraged, increasing its crit chance and reducing the amount of damage received for a short time." (`loc 3607324380`) | [client] [video Sy-W_AHbwcw/00076, 00152, 00154, 00290, 00306, 00308, 00312] [wiki] |
| Friendly NPCs | **Disoriented Prisoners** (`loc 3560676045`, `1025118610`), kept in **Cages** | A caged prisoner says "You won't turn me into a shambler!" (`loc 3792730861`). Cages have a "Press X or mouse" prompt. The wiki also lists Jazlin, Sammy and Lessi as NPCs. The client has "Locate all of Meliee's friends! Find Jazlin!" (`loc 295733676`) and "Find Lessi!" (`loc 1377626183`) for the first two. Sammy is only a wiki listing here. | [video Sy-W_AHbwcw/00060, 00098, 00102, 00142] [client] [wiki] |
| Props and pickups | "Treasure Chest" (press X), "Coin Drop" pickups, a fake wall hiding the boss route, pillars (10) | The fake wall is from the Shambler's line, not seen as an object name. | [video Sy-W_AHbwcw/00080, 00316, 00308] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "256 Stars or less" (wiki wording) | [wiki] |
| Coins on the prize wheel | **26 Coins** slot on every job's wheel | [wiki] |
| Members-only bonus | **73 Coins** | [wiki] |
| Bonus-goal clothing | Completing the bonus goal gives every player in the instance one piece of clothing from a set for their job. Colour and body slot are random. | [wiki] |
| In-run purchases | "You purchased 16 Medium Replenishment Potions." appears at 5:42 in the Warrior 7 run. The vendor is not visible. | [video Sy-W_AHbwcw/00342] |
| Achievements | "Complete the Den of Secrets encounter." (`loc 2813753258`), full-group version (`817888718`) | [client] |
| Gear tier | Wheel gear carries "II", matching difficulty 2. | [wiki] [client names] |

### Prize wheel by job

Every wheel also has the 26 Coins slot. Every item name exists in the client locale. [wiki] [client names]

| Job | Items |
|---|---|
| Archer | Archer's Horse Bow of Splintering, Archer's Power Shard of Vitality II, Archer's Ring of Energy II, Battle Item Mystery Pack, Roving Archer Leggings |
| Brawler | Battle Item Mystery Pack, Brawler's Hammer of Roundhouse, Brawler's Power Shard of Vitality II, Brawler's Ring of Energy II, Bum Rush Brawler Pants |
| Medic | Antibiotic Medic Pants, Battle Item Mystery Pack, Medic's Power Shard of Vitality II, Medic's Ring of Energy II, Medic's Scalpel of Immunization |
| Ninja | Battle Item Mystery Pack, Ninja's Blade of Flame Wave, Ninja's Power Shard of Vitality II, Ninja's Ring of Energy II, Sekkou Ninja Hakama |
| Warrior | Battle Item Mystery Pack, Second Wind Warrior Breeches, Warrior's Axe of Warcry, Warrior's Power Shard of Vitality II, Warrior's Ring of Energy II |
| Wizard | Battle Item Mystery Pack, Journeyman Wizard Slacks or Journeyman Wizard Skirt, Wizard's Power Shard of Vitality II, Wizard's Ring of Energy II, Wizard's Wand of Tsunami |

Bonus clothing sets by job: Archer Point Blank or Roving, Brawler Bum Rush or Peek-a-Boo, Medic Antibiotic or Urgent Care, Ninja Nokizaru or Sekkou, Warrior Basic Attack or Second Wind, Wizard Hedge or Journeyman. [wiki]

## Related quests and NPCs

Quest titles are not stored beside descriptions in the locale, so each row gives the description id. Givers are only those named in the text.

| Quest text (client) | Id | What it asks | Sources |
|---|---|---|---|
| "You must hurry and go to the Den of Secrets and see what is going on to cause the children of Merry Vale to go missing. You are our only hope!" | `1028588083` | Merry Vale kidnapping chain. Companions: `3250915691` ("Go down there and put a stop to the abductions!"), `1333139774`, `950521189`, `1157028388`, `2693250383`, `4143502912`, `2561540889` ("Fairy Cultists have come out of the Den of Secrets to kidnap pixies! Defeat them and their leader!"). | [client] |
| "Mayor McFairy, mayor of Merry Vale, has given me all of the tools I need to enter the cavern north of Merry Vale and defeat the source of the kidnappings in southern wilds." | `1876015264` | Mayor McFairy line. Objectives "Speak with Mayor McFairy and collect information on the kidnappings." (`2736543039`), "Confront Mayor McFairy here in Merry Vale and uncover his secrets!" (`2466823051`), "Defeat Mayor McFairy within the Den of Secrets!" (`3349678837`), "Mayor McFairy and his assistant Liana were involved? ..." (`3448615816`). See Conflicts. | [client] |
| "Meliee's friends are missing. Search the Den of Secrets to locate them." | `3974853380` | Find Jazlin and Lessi (`295733676`, `1377626183`), "Return to Meliee!" (`742799014`) | [client] |
| "Tumbolina's weapons crate has been stolen and hauled off to the Den of Secrets. Retrieve the weapons crate and return it to Tumbolina in Merry Vale." | `2811794577` | Recover crate. Companions: `2857447987`, `2903750529`, `645885654`, `20462587`, `328117154`. | [client] |
| "Shaggy Mysterychaser, an amateur detective in Merry Vale, needs me to explore the depths of the Den of Secrets in the back of the cave in northern Merry Vale." | `1819244747` | Explore. Companions: `1794630937` ("Search the Den of Secrets for something scary!"), `2795530967`, `3679298455`. | [client] |
| "Destroy the prism within the Den of Secrets!" | `2195643916` | Destroy a prism | [client] |
| "Knock out some Shamblers within the Den of Secrets!" / "Defeat Shamblers within the Den of Secrets." | `2204502353`, `3828495262` | Defeat shamblers | [client] |
| "Defeat Yuriah in the Den of Secrets!" | `426775865` | Yuriah, a necrowart zombie (the son in `loc 1040002658`) | [client] |
| "Search the Den of Secrets for the bucket that Clyss mentioned, and return it to him." / "Get the tools back from the Den of Secrets" | `4227542947`, `3235021407` | Retrieval tasks | [client] |
| "Complete the Den of Secrets encounter." / "Win Combat Encounter Den of Secrets" | `2813753258`, `1323605988` | Generic objectives | [client] |

NPCs: [[Illusionist Liana]], [[Mayor McFairy]], [[Meliee]], [[Tumbolina]], [[Jazlin]], [[Lessi]], [[Shaggy Mysterychaser]]. Zone: [[NPCs - Merry Vale]].

## Conflicts

1. **Boss.** The wiki and the video have Illusionist Liana. The client also has "Defeat Mayor McFairy within the Den of Secrets!" (`loc 3349678837`) and "Defeat Yuriah in the Den of Secrets!" (`loc 426775865`). These look like older versions of the story or other quest variants. Liana is used here because the goal panel in video names her.
2. **Pillar singular.** The first video frame reads "within the cavern!" and later frames "within the caverns!". The client string is plural ("caverns").
3. **Sammy.** The wiki lists Sammy as an NPC of this dungeon, but no client or video text ties any Sammy to it. The client only has a "Sammy" (`loc 1740525025`) and "Sammy "The Shark"" without a dungeon link.

## Gaps

- Star payout in video (only the wiki's "256 Stars or less"), and the results screen. The Warrior 7 run ends before the win.
- Where the 10 pillars are and the order of the route; what the fake wall is.
- What the cages do (free the prisoner? an objective?). Prisoners are neither a goal nor a counter in the panel.
- Liana's health, phases, and how the illusions and orbs work.
- Group size.
