---
type: content-dungeon
zone: Briarwood (Nettleseed)
combat: true
---

# Mushroom Cavern

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Mushroom Cavern" (`loc 501552057`, `813401468`, game-data name `loc 1040242605`). Many quest texts write the plural "Mushroom Caverns". | [client] |
| Panel description | "Fungal beasts and bats are causing havoc! Defeat their leader, Marasmius, to put a stop to their antics!" (`loc 3559035025`). The wiki quotes the same line but spells the boss "Maramius". | [client] [wiki] |
| Type | Cave dungeon, difficulty 3 | [wiki] |
| Entrance | Click the **barrier** in **Nettleseed**, in **Briarwood**. It has an Atlas icon. The client has a "Find the entrance to the Mushroom Cavern" objective (`loc 2053035202`) and a quest where the giver "can't make it past the entrance" (`loc 946440259`). | [wiki] [client] |
| Atlas | The world map lists "Mushroom Cavern" with "Press to Teleport..." as a warp target. | [video JMmHnklaj5Y/00002] |
| Story hook | Fungal creatures spill out of the cave and attack nearby animals (`loc 3685284269`). Their leader Marasmius is "the mushroom gigas" who "commands his minions" and holds the fourth shard (`loc 3203953768`, `1632976911`). | [client] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 5 combat job** (difficulty 3 rule: "Requires a level 5 combat job", `loc 2145075890`). The wiki infobox says "Levels 5 or higher". | [client] [wiki] |
| Members only | No. Members get an extra coin bonus on the results screen (see Rewards). | [wiki] |
| Unlocking quest | None found. Many quests send players in (see Related quests). | [client] |
| Group size | Unknown. A "Complete the Mushroom Cavern encounter with a full group." achievement exists (`loc 1940103154`), so a full group size exists but is not stated. | [client] |
| Video of play | None found. Video shows only the Atlas map entry. | [video JMmHnklaj5Y/00002] |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main | Client strings: **"Defeat Marasmius!"** (`loc 2259654589`), "Defeat Marasmius" (`292631595`), "Defeat the mad mushroom Marasmius!" (`2895589040`). The wiki writes "Defeat Maramius!". | [client] [wiki] |
| Bonus | Client: **"Collect all of the Sprouting Budlings!"** (`loc 75020374`). The wiki says "Bonus: Collect all of the Sprouting Budlings! (6)". Detail text: "Search for the Sprouting Budlings within the Mushroom Caverns, before they sprout into new creatures like Marasmius!" (`loc 1914220773`). | [client] [wiki] |
| Count | 6 Sprouting Budlings (wiki only; no video seen). | [wiki] |
| Internal bonus group | "Mushroom Caverns Bonus Objective" (`loc 861238372`, plural spelling) | [client] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Marasmius** (client), "Maramius" (wiki) | Called "the mushroom gigas" in quest text (`loc 3203953768`). Dev/test strings: "Marasmius kicks the wall and causes a cave in!" (`loc 4263912541`), "Marasmius walk around. If you close, Marasmius squish you!" (`loc 581490102`), "You leave Marasmius home now!" (`loc 1737938753`). The cave-in line is probably a boss ability; its effect is unknown. The separate battle [[The Mushroom Gigas]] is a different wiki entry. | [client] [wiki] |
| Enemies (wiki) | Bats, Vine Wolves, Giant Man Eating Fungus | Wiki list. | [wiki] |
| Enemies (client) | "Giant Man Eating Fungus" (`loc 3010509880`, game data `2192100189`), "Fungus Golem" (`loc 1195434887`, `3925227197`), "Fungal Soldier" (`loc 709881902`, `919627625`, `3829895799`), "Man Eating Fungus" (`loc 2885030490`) | Quests say "fungus golems", "fungus creatures" and "Fungal Soldiers" are in the cave. The client has no "Vine Wolf" enemy name; "Vine Wolf" appears only in card-game and food text, so that wiki name is unconfirmed. | [client] [wiki] |
| Bonus pickup | **Sprouting Budling** (`loc 896101975`, game data `178477681`) | Six to collect (wiki). | [client] [wiki] |
| Cave hazards in other content | Jumping a ravine to a "Sparkling Mushroom" and "fertile potting soils" appear in quest texts for the Mushroom Caverns. Whether they are inside the combat instance is unknown. | [client `loc 3945007233`, `3827180954`] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "372 Stars or less" (wiki wording). No video. | [wiki] |
| Coins on the prize wheel | The wiki lists **80 Coins** on the Archer, Brawler and Medic wheels and **162 Coins** on the Ninja, Warrior and Wizard wheels. Other pages give one coin value for every job, so this may be a wiki editing slip. Not confirmed. | [wiki] |
| Members-only bonus | **140 Coins** | [wiki] |
| Bonus-goal clothing | One piece of clothing from a set for the player's job, random colour and body slot, for everyone in the instance. | [wiki] |
| Achievements | "Complete the Mushroom Cavern encounter." (`loc 2935323594`), "Complete the Mushroom Cavern encounter with a full group." (`loc 1940103154`), "Win Combat Encounter Mushroom Cavern" (`loc 521572444`), "Group: Mushroom Cavern" (`loc 4208189005`) | [client] |
| Gear tier | Wheel gear carries "III", matching difficulty 3. | [wiki] [client names] |
| Speed boost | The locale has " Speed Boost for Mushroom Caverns." (`loc 1590531383`, `3988244158`). It is probably a promotion or item text; meaning unknown. | [client] |

### Prize wheel by job

Each wheel has five items and one Coins slot. Every item name below exists in the client locale (checked with `cq.py loc`). Three wiki spellings differ from the client and are listed in Conflicts. [wiki] [client names]

| Job | Items | Coins slot (wiki) |
|---|---|---|
| Archer | Archer's Composite Bow of Volleys, Archer's Power Shard of Vitality III, Archer's Ring of Energy III, Battle Item Mystery Pack, Drifting Archer Cap | 80 |
| Brawler | Battle Item Mystery Pack, Brawler's Anvil Hammer of Sweeps, Brawler's Power Shard of Vitality III, Brawler's Ring of Energy III, Catchweight Brawler Helmet | 80 |
| Medic | Battle Item Mystery Pack, Emergency Medic Cap, Medic's Bonesaw of Triage, Medic's Power Shard of Vitality III, Medic's Ring of Energy III | 80 |
| Ninja | Battle Item Mystery Pack, Ninja's Power Shard of Vitality III, Ninja's Ring of Energy III, Ninja's Scythe of Dragonstrike, Nusumi Ninja Cowl | 162 |
| Warrior | Battle Item Mystery Pack, Combat Bonus Warrior Helm, Warrior's Battle Hammer of Spinning, Warrior's Power Shard of Vitality III, Warrior's Ring of Energy III | 162 |
| Wizard | Battle Item Mystery Pack, Scholar Wizard Fez, Wizard's Bone Wand of Shock, Wizard's Power Shard of Vitality III, Wizard's Ring of Energy III | 162 |

Bonus clothing sets by job: Archer Cross Wind or Drifting, Brawler Catchweight or Rubber Match, Medic Emergency or Trauma Care, Ninja Nusumi or Shinobi, Warrior Charge Attack or Combat Bonus, Wizard Practitioner or Scholar. [wiki]

## Related quests and NPCs

Quest titles are not stored beside descriptions in the locale, so each row gives the description id. Givers are only those named in the text.

| Quest text (client) | Id | What it asks | Sources |
|---|---|---|---|
| "Enter Mushroom Cavern and defeat Marasmius to obtain the fourth shard." | `2710911522` | Shard quest line. Companions: `1632976911`, `2615830165`, `3203953768`. Boss target for the main-line shard hunt. | [client] |
| "Rumor has it that Fungal Soldiers are stirring up trouble in the Mushroom Cavern... see what the fuss is all about." | `432495480` | Scout and fight quest, giver not named | [client] |
| "Defeat Fungus Golems in the Mushroom Cavern." / "Defeat the fungus creatures within Mushroom Caverns." | `2677238513`, `1031884174`, `3653542620`, `3685284269` | Defeat quests. Companions: `186021886` (Briarwood clearing), `1601020896` and `2540808216` (the chugs, earning their trust, with "Defeat Fungus Golems" `4217539244`). | [client] |
| "Stone heads like my brother, Mekaletaliki, and sister, Lana, use rumblings to communicate. ... find the source of the rumbling" | `518335411` | A stone head sends you to investigate. Companions: `2394618004`, `4086426221`, `2610390602`, `3681690919` (Malarkilanihani). | [client] |
| "My apprentice, Chicory, is in the Mushroom Cavern. Find him..." | `922734498` | Find Sperry's apprentice Chicory. Companions: `2928313003`, `3117632465`, `1561243387`, `2925900217`, `4027367938`. | [client] |
| "Arg! I need to plant more of these seeds, but I can't make it past the entrance of the Mushroom Cavern." | `946440259` | Johnny's seeds. Plant seeds in good soil. Companions: `1063100854`, `3844599297`, `3851795684`, `3827180954` (six potting soils), `3451970127` ("Return to Johnny outside of the Mushroom Cavern"). | [client] |
| Toxic truffle bait from golems | `358456730` | Fishing-vacation quest, giver not named. Companion `3001522676`. | [client] |
| "Yattes has bet that no one can cross the pit in Mushroom Caverns and retrieve the Sparkling Mushroom." | `188376167` | Ravine jump. Companions: `490740120`, `3945007233`. | [client] |
| "Go into the Mushroom Caverns and find me a mushroom." | `301159145` | Cure for a zombified character | [client] |
| "Complete the Mushroom Cavern encounter." / "Win Combat Encounter Mushroom Cavern" | `2935323594`, `521572444` | Generic objectives | [client] |

NPCs: Marasmius (boss, no NPC page yet), [[Sperry]], [[Chicory]], [[Johnny]]. Zone: [[NPCs - Briarwood]]. Related battle: [[The Mushroom Gigas]].

## Conflicts

1. **Boss spelling.** Wiki "Maramius". The client spells "Marasmius" in many strings (`loc 3559035025` and others). Client used.
2. **Head-slot item.** Wiki Wizard wheel "Scholar Wizard Medium Fez". The client has "Scholar Wizard Fez" (`loc 39413729`) and "Scholar Wizard Medium Cloak" (`loc 244013527`). Client "Fez" used.
3. **Shard spelling.** Wiki "Wizard's Power shard of Vitality III" (lower-case s). Client "Wizard's Power Shard of Vitality III". Client used.
4. **Coin slots.** The wiki gives 80 on three wheels and 162 on three. Not checked against any other source.
5. **Enemy names.** The wiki says "Vine Wolves". The client has no such enemy name, and the cave's client enemies are Giant Man Eating Fungus, Fungus Golem and Fungal Soldier.
6. **Bonus group name.** "Mushroom Caverns Bonus Objective" (plural) against the singular dungeon name. Both exist in the client.

## Gaps

- Any gameplay video: no frame shows the fight, the goals panel or the result screen.
- Group size.
- Star payout (only the wiki's "372 Stars or less").
- The bonus goal's exact on-screen text (the wiki adds a "(6)" count that is not seen in the client).
- Wave layout, boss health and abilities, and whether the "cave in" line is an attack.
- Which quest owns each description above (titles), and the givers of the unnamed ones.
