---
type: content-dungeon
zone: Northern Wilds (near Shrouded Glade)
combat: true
---

# Mugwort's Hollow

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Mugwort's Hollow" (`loc 1339804009`, `3328472346`, game-data name `loc 871461304`) | [client] |
| Panel description | "The exiled druid Mugwort is endangering the Hollow! Head into the Hollow and confront him!" (`loc 2474280079`) | [client] [wiki] |
| Type | Forest dungeon, difficulty 4 | [wiki] |
| Entrance | Click the **barrier** in the northeast Wilds, "along the mountain face between Snowhill and Shrouded Glade". It has an Atlas icon. Client wording: "Find Mugwort's Hollow outside of Shrouded Glade" (`loc 2107188895`), "west of Shrouded Glade" (`loc 2867760776`, `4097395734`). | [wiki] [client] |
| Story hook | "Mugwort used to be a top student until he was accused of stealing power stones. When he ran off, most assumed he was guilty." (`loc 3924860970`). He is a "crazy druid" holding the Scroll of the Jade Tiger (`loc 1920299577`). | [client] |
| Video | None found. No frame names the dungeon. | [video] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 9 combat job** (difficulty 4 rule, `loc 1291047686`). The wiki infobox says "Levels 9 or higher". | [client] [wiki] |
| Members only | No. Members get an extra coin bonus on the results screen. | [wiki] |
| Unlocking quest | None found. Quests send players in (see Related quests). | [client] |
| Group size | Unknown. "Complete the Mugwort's Hollow encounter with a full group." exists (`loc 1339572619`). | [client] |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main | **"Defeat the druid, Mugwort!"** (`loc 2793325066`). Also quest text "Defeat Mugwort in Mugwort's Hollow outside of Shrouded Glade" (`loc 2216126723`). | [client] [wiki] |
| Bonus | **"Defeat all enemies!"** (wiki). The client has no exact string; the nearest is "All enemies are defeated" (`loc 733152254`). The bonus group exists: "Mugwort's Hollow Bonus Objective" (`loc 863942879`). | [wiki] [client] |
| Quest-side goals | "Pass the test of courage by exploring all of Mugwort's Hollow!" (`loc 1623844062`) and "See how far you can get through Mugwort's Hollow!" (`loc 1677061632`) belong to a scouting dare, not the battle. | [client] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Elder Mugwort** (`loc 3500909485`, game data `1294913866`) | Wiki boss name and client name agree. Quest text just says "Mugwort". | [client] [wiki] |
| Enemies (wiki) | Ravens, Battle Warthogs, Snakes, Giant Toads, Warthog Chargers, Toad Bouncers, Ghost Ravens, Ghost Snakes, Ghost Warthogs, Ghost Toads | Wiki list: living animals plus ghost versions. | [wiki] |
| Enemies (client) | "Warthog Charger" (`loc 2692431127`), "Toad Bouncer" (`loc 3060579901`), "Giant Toad" (`loc 128633574`), "Battle Warthogs" (`loc 1057237145`), "Forest Snakes" (quest text `loc 2646277932`), "Ghost Toad" (`loc 4168755251`: toad_ghost), "ghost raven" (`loc 3186883069`) | "Ghost Toad Immunity" (`loc 259364180`) and "Ghost Toad Belch" (`loc 444172207`) are ability names. The client has no "Ghost Warthog" or "Ghost Snake" string by those words. Quest hints: "They tend to be found in groups at Mugwort's Hollow." (`loc 2953544010`), "They're slithering around Mugwort's Hollow." (`loc 3708244266`). | [client] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "370 Stars or less" (wiki wording). No video. | [wiki] |
| Coins on the prize wheel | **97 Coins** slot on each job's wheel | [wiki] |
| Members-only bonus | **264 Coins** | [wiki] |
| Bonus-goal clothing | One piece of clothing from a set for the player's job, random colour and body slot, for everyone in the instance | [wiki] |
| Achievements | "Complete the Mugwort's Hollow encounter." (`loc 2950930613`), full-group version (`1339572619`), "Win Combat Encounter Mugwort's Hollow" (`939677309`), "Group: Mugwort's Hollow" (`118381270`) | [client] |
| Gear tier | Wheel gear carries "IV", matching difficulty 4. | [wiki] [client names] |

### Prize wheel by job

Each wheel has five items plus the 97 Coins slot. Item names follow the client locale (checked with `cq.py loc`). Two wiki names have no client match and are replaced by the client's closest item for that set and slot; see Conflicts. [wiki] [client names]

| Job | Items |
|---|---|
| Archer | Archer's Power Shard of Vitality IV, Archer's Recurve Bow of Splintering, Archer's Ring of Regeneration IV, Battle Item Mystery Pack, Bodkin Archer Leggings |
| Brawler | Battle Item Mystery Pack, Brawler's Drill Hammer of Roundhouse, Brawler's Power Shard of Vitality IV, Brawler's Ring of Regeneration IV, Stick and Move Brawler Pants |
| Medic | Battle Item Mystery Pack, Medic's Megasaw of Immunization, Medic's Power Shard of Vitality IV, Medic's Ring of Regeneration IV, Pressure Wave Medic Fatigues (wiki: "Pants") |
| Ninja | Battle Item Mystery Pack, Ninja's Jagged Scythe of Flame Wave, Ninja's Power Shard of Vitality IV, Ninja's Ring of Regeneration IV, Sensei Ninja Hakama |
| Warrior | Battle Item Mystery Pack, Threat Range Warrior Breeches (wiki: "Greaves"), Warrior's Double Axe of Warcry, Warrior's Power Shard of Vitality IV, Warrior's Ring of Regeneration IV |
| Wizard | Battle Item Mystery Pack, Disciple Wizard Slacks or Disciple Wizard Skirt, Wizard's Jewel Wand of Tsunami, Wizard's Power Shard of Vitality IV, Wizard's Ring of Regeneration IV |

Bonus clothing sets by job: Archer Bobtailed or Bodkin, Brawler Barnburner or Stick and Move, Medic Hydrostatic or Pressure Wave, Ninja Kanja or Sensei, Warrior High Initiative or Threat Range, Wizard Adherent or Disciple. [wiki]

## Related quests and NPCs

Quest titles are not stored beside descriptions in the locale, so each row gives the description id. Givers are only those named in the text.

| Quest text (client) | Id | What it asks | Sources |
|---|---|---|---|
| "Defeat Mugwort in Mugwort's Hollow outside of Shrouded Glade" | `2216126723` | Defeat Mugwort. Part of Chaz's three-boss quest (`loc 2431923117`) with [[Dark Spore Depths]] and [[Howling Hills]]. | [client] |
| "According to Kai, the Scroll of the Jade Tiger ended up in the possession of Mugwort..." | `1920299577` | Ninja line for Kazuo. Companions: `2037799473`, `3148214181`, `3417381922`, `1347689903`, `1801107415` (Rokuro). | [client] |
| "Mugwort used to be a top student until he was accused of stealing power stones." | `3924860970` | Find the power stones. Companions: `48902851`, `3679200986`, `1454045340` (Ceard Bauchlog), `281396575`. | [client] |
| "I almost have enough plant samples to start my science project, but I still need some from Mugwort's Hollow." | `252520526` | Plant samples. Companions: `1036495668`, `1985945785`, `2867760776`, `3437027651`. | [client] |
| "I want some life magic samples from Forest Snakes and Battle Warthogs at Mugwort's Hollow." | `2646277932` | Natali's samples (wizard line `2644582074`). Companion `2241882459`. | [client] |
| "Troy has dared you to scout out Mugwort's Hollow." | `1377414031` | Scouting dare. Companions: `3417579659`, `258775933`, `2328025099`, `1482063951`. | [client] |
| "Use your Leg Sweep ability to defeat ten opponents in Mugwort's Hollow." | `4128133192` | Brawler ability practice | [client] |
| "Complete the Mugwort's Hollow encounter." / "Win Combat Encounter Mugwort's Hollow" | `2950930613`, `939677309` | Generic objectives | [client] |

NPCs: [[Elder Mugwort]], [[Kai]], [[Rokuro]], [[Natali]], [[Ceard Bauchlog]], [[Chaz]]. Zone: [[NPCs - Northern Wilds]].

## Conflicts

1. **Item names without a client match.** Wiki "Pressure Wave Medic Pants": the client has Pressure Wave Medic Fatigues, Jacket, Boots, Gloves, Helmet and Backpack but no Pants (`loc 1852018411`, `2730753694`). Wiki "Threat Range Warrior Greaves": the client has Threat Range Warrior Breeches (`loc 2265806935`), Gauntlets, Hightops, Pauldrons, Helm and Tunic, but no Greaves. The legs slot is "Fatigues" for Medic and "Breeches" for Warrior in other client sets (for example "Stopping Power Medic Fatigues" and "Called Shot Warrior Breeches" in [[Dark Spore Depths]]). The client spellings are used above, on the assumption that the wiki named the slot loosely. Unconfirmed.
2. **Shard spelling.** Wiki "Wizard's Power shard of Vitality IV". Client "Shard" (`loc 567538116`). Client used.
3. **Zone.** The wiki sets it in "Northern Wilds" and also "northeast Wilds, along the mountain face between Snowhill and Shrouded Glade". The client says "outside of Shrouded Glade" and "west of Shrouded Glade". Both are kept in the page header.
4. **Ghost enemies.** The wiki's Ghost Snakes and Ghost Warthogs have no client string by those names. Ghost Toad and a ghost raven exist.
5. **Bonus wording.** "Defeat all enemies!" is wiki only.

## Gaps

- No video at all: goals panel, boss fight and results are unseen.
- Group size.
- Star payout (only the wiki's "370 Stars or less").
- Elder Mugwort's abilities and health, and the ghost-animal mechanic.
- Which quest owns each description above (titles).
