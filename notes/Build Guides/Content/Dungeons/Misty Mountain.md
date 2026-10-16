---
type: content-dungeon
zone: Shrouded Glade
combat: true
---

# Misty Mountain

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Misty Mountain" (`loc 440183008`, game-data name `loc 1193653572`). Quest text also writes "Misty Mountains". Dev-tag spellings "Misty Mountains 159", "misty_mountains 159". | [client] |
| Panel description | "The normally peaceful and benign elemental guardians are attacking anyone seeking the oracle! Find out what's behind this and defeat it!" (`loc 545996883`). The lobby panel in video shows the same text. | [client] [video PqWJjsgvi_s/00014] [wiki] |
| Type | Mountain dungeon, difficulty 3 | [wiki] |
| Entrance | Click the **gate** in **Shrouded Glade**. It has an Atlas icon. On the world map and as a distance marker it shows as a red marker named "Misty Mountain" (47 to 130 m away) with a crossed-swords battle icon. | [wiki] [video KYKRXeDqYDc/00018, EkNbw2nCVQc/00012, NemdyF6pHTs/00242] |
| Setting | A snowy ridge and ice cave. Quest text: "Most of our ancient texts are buried beneath rubble on Misty Mountain, which is all that remains of the once expansive Athenaeum" (`loc 2768709272`) and "the old druidic library in the Misty Mountains" (`loc 3867652092`). | [client] |
| Story hook | The elemental guardians turned hostile toward anyone who seeks the oracle, The Rose that Knows. The boss is the Elemental Lord, Snowface. | [client] [video UgBonM1p0Z0/00278, 00292] |
| Promo strings | "Battle the evil at Misty Mountains to earn bonus rewards!" (`loc 2348854149`). Window unknown. | [client] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 5 combat job.** The lobby panel reads "Difficulty: Requires a level 5 combat job" (`loc 2145075890`). The wiki infobox says "Levels 5 or higher". | [client] [video PqWJjsgvi_s/00014] [wiki] |
| Seen playing at | Ninja 20 | [video UgBonM1p0Z0/00140, NbP36KcX8N8/00210] |
| Members only | No. Members get an extra coin bonus on the results screen. | [wiki] |
| Unlocking quest | None found. Quests send players in (see Related quests). | [client] |
| Group size | Unknown. "Complete the Misty Mountain encounter with a full group." exists (`loc 3971860013`). | [client] |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main | **"Investigate the Shrine of Knowledge!"** (`loc 1705950638`). Ticked after the Snowface fight. Client dev group: "Misty Mountain - Primary Objective" (`loc 1991555935`). | [client] [video UgBonM1p0Z0/00140, 00314] [wiki] |
| Bonus | **"Bonus: Find the pieces of the Oracle's lost tablet of teachings! - 1/4"** (counter seen at 1/4 and 2/4). Client detail: "Search throughout the Misty Mountains to find the 4 pieces of the lost teachings of the Oracle!" (`loc 2142119447`). Dev group: "Misty Mountain - Bonus Objective" (`loc 1957291602`). | [client] [video UgBonM1p0Z0/00140, 00292] [wiki] |
| Panel shows | "Goals (2)". The pickup is a **Tablet Fragment** (`loc 45447572`) with a "Press X or ..." prompt. | [video UgBonM1p0Z0/00198] [client] |
| End | After the fight, an **Exit** door appears ("Press X or mouse" over a glowing green door). | [video UgBonM1p0Z0/00328, 00332] |

Other client goal strings, usage unknown here: "Defeat Snowface!" (`loc 2354532842`), "Find and defeat the Elemental Lord, Snowface!" (`loc 2914021299`), "Seek out the wisdom of the Rose that Knows and deal with any problems that may arise!" (`loc 3669990794`). [client]

### Scripted scene (video)

1. Elemental Guardians appear and say "Please.", "Come no further.", "We do not wish to harm you." [video UgBonM1p0Z0/00140, 00144]
2. The Rose that Knows: "At last, you have come. I have been waiting for you." [video UgBonM1p0Z0/00278]
3. The Rose that Knows: "But beware! The enemy approaches! Prepare yourself..." Snowface (red text): "The oracle is mine! You will never leave here!" (client `loc 3649534776`) [video UgBonM1p0Z0/00292, 00294]
4. After Snowface falls, the Rose says: "Finally, I will be a prisoner no longer, thanks to you." / "You are the one that will be a light shining amidst the gathering darkness." / "I have looked into the future and have seen it come to pass." / "We must part for now. We may speak again... in the future." [video UgBonM1p0Z0/00314 to 00332]

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Snowface** (`loc 4091691195`, game data `3459945755`) | "the Elemental Lord". Boss health bar at the top centre during the fight; the player is seen stunned ("You can't use abilities when you are stunned!"). The client even holds a stuttering line "Snowface... s-s-s-snow f-f-f-face!" (`loc 3454018831`). | [client] [video UgBonM1p0Z0/00300, 00302] [wiki] |
| Friendly NPC | **The Rose that Knows** (`loc 172818260`) | The imprisoned oracle. She speaks the scene above. | [client] [video UgBonM1p0Z0/00278 to 00332] [wiki] |
| Enemies | **Elemental Guardian** (`loc 1079243857`, `3019122702`), "Elemental Guardian Patrol" (`loc 2106945594`) | The first enemies. Quest text says "ice elementals" and names Earth, Fire, Air and Wind Elemental dev tags. | [client] [video UgBonM1p0Z0/00144, 00314] |
| Enemies (wiki) | Wolves, Cray, Broodmother, Floren, Misty Mountain Hermit | Wiki list. The client has "Broodmother" (`loc 1861603980`), "Misty Mountain Hermit" (`loc 1779674673`, `2355546867`) and "snow cray" (`loc 2755439790`). No video of these in the instance. | [wiki] [client] |
| Dev strings | "Misty Mountains 159: Wind Elemental Aura" (`112469131`), "air elemental attack" (`348605666`), "Earth Elemental Ability" (`855457753`), "Earth Elemental Buff" (`3971832887`), "fire elemental ability" (`3212542574`), "Boss Confusion" (`408402569`), "teleport to location 1" (`4246246838`), "boss ability" (`902543252`), "monster spawnwer" (`3818797804`), "Misty mountains base pop ability" (`2887480072`) | Names only. Effects unknown. | [client] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "334 Stars or less" (wiki wording). No video result screen. | [wiki] |
| Coins on the prize wheel | **54 Coins** slot on each job's wheel | [wiki] |
| Members-only bonus | **125 Coins** | [wiki] |
| Bonus-goal clothing | One piece of clothing from a set for the player's job, random colour and body slot, for everyone in the instance | [wiki] |
| Achievements | "Complete the Misty Mountain encounter." (`loc 3365634525`), full-group version (`3971860013`), "Win Combat Encounter Misty Mountain" (`1447018070`), "Group: Misty Mountain" (`3019289650`) | [client] |
| Gear tier | Wheel gear carries "III", matching difficulty 3. | [wiki] [client names] |
| Collection | Quest text mentions elemental samples and ice shards from the guardians ("Elemental Samples in Misty Mountains" `loc 1865843540`, `3951530435`, `3567537879`). Whether these drop in the instance is unknown. | [client] |

### Prize wheel by job

Each wheel has five items plus the 54 Coins slot. Every item name below exists in the client locale (checked with `cq.py loc`) except where flagged. [wiki] [client names]

| Job | Items |
|---|---|
| Archer | Archer's Composite Bow of Stunning, Archer's Necklace of Bark Skin III, Archer's Power Shard of Regeneration III, Battle Item Mystery Pack, Cross Wind Archer Gloves |
| Brawler | Battle Item Mystery Pack, Brawler's Anvil Hammer of Cheapshot, Brawler's Necklace of Bark Skin III, Brawler's Power Shard of Regeneration III, Rubber Match Brawler Gloves |
| Medic | Battle Item Mystery Pack, Medic's Bonesaw of Vitamins, Medic's Necklace of Bark Skin III, Medic's Power Shard of Regeneration III, Trauma Care Medic Gloves |
| Ninja | Battle Item Mystery Pack, Ninja's Necklace of Bark Skin III, Ninja's Power Shard of Regeneration III, Ninja's Scythe of Shadow Armies, Shinobi Ninja Gloves (wiki: "Shinobi Ninja Wristguard", not in the client; see Conflicts) |
| Warrior | Battle Item Mystery Pack, Charge Attack Warrior Gauntlets, Warrior's Battle Hammer of Whirlwind, Warrior's Necklace of Bark Skin III, Warrior's Power Shard of Regeneration III |
| Wizard | Battle Item Mystery Pack, Practitioner Wizard Gloves, Wizard's Bone Wand of Vortex, Wizard's Necklace of Bark Skin III, Wizard's Power Shard of Regeneration III |

Bonus clothing sets by job: Archer Cross Wind or Drifting, Brawler Catchweight or Rubber Match, Medic Emergency or Trauma Care, Ninja Nusumi or Shinobi, Warrior Charge Attack or Combat Bonus, Wizard Practitioner or Scholar. [wiki]

## Related quests and NPCs

Quest titles are not stored beside descriptions in the locale, so each row gives the description id. Givers are only those named in the text.

| Quest text (client) | Id | What it asks | Sources |
|---|---|---|---|
| "Tald Daybreaker dared his friend, Milwind, to travel up Misty Mountain. He hasn't been back..." | `2416549744` | Rescue Milwind from snow cray. Companions: `3478305251`, `1073835134`, `2755439790`. | [client] |
| "Arestol will reward you if you can find some buried tomes up on Misty Mountain" | `794019279` | Buried-tome fetch. Companions: `2768709272`, `3936911343`, `1308263417` (book on Fastvi Frostflutter from the highest peak). | [client] |
| "Some nasty critters are blocking the path through Misty Mountain that the druids of Shrouded Glade use..." | `3536529370` | Ceard: clear the path. Companions: `1107906352`, `3710481989`, `2002964206`. | [client] |
| "Assistant Headmaster Tov wants you to collect bags of incense that were dropped on the icy paths of Misty Mountain" | `2553051681` | Incense fetch. Companions: `2296958080`, `1197697940`, `3144507374`, `3279305315`, `4269811908`. | [client] |
| "Embir needs you to head up Misty Mountain and get a sample of water from the lake there." | `2895189191` | Water sample. Companions: `42203743`, `357935139`, `1784862177`, `2192944852`. | [client] |
| "Thieves took Lavender's chest of treasured things up into the Misty Mountain." | `228598730` | Daphne: recover Lavender's Locker. Companions: `1729069872`, `868302540`, `3404283059`, `3718547770`, `3935684290`. | [client] |
| "Puck Petalplucker ... needs some samples from the elementals in Misty Mountain." | `3752511862` | Elemental samples. Companions: `3567537879`, `2432717976`, `1865843540`. | [client] |
| "Search for the Scroll of Six Moons in the Misty Mountains." | `2449167576` | Kazuo's last hope (Ninja line). Companions: `990535413`, `1846113949`, `3867652092`. | [client] |
| "Michi in Snowhill ... get a copy of 'Ancient Evils: A User-Friendly Guide to Evil Artifacts' from a crazy hermit in the Misty Mountains" | `3982906952` | Ninja errand to the hermit. Companions: `2312819135`, `2275677711`, `460925844`. | [client] |
| "Escort little Aedon to the top of Misty Mountain" / the lost penguin | `16034566`, `1841897735` | Escort quests. Return text `3793430938`. | [client] |
| "Misty Mountain Fizz" | `1483953870`, `2518614605`, `3935063848` | Perci in Shrouded Glade sells it. Not a battle quest. | [client] |
| "Complete the Misty Mountain encounter." / "Win Combat Encounter Misty Mountain" | `3365634525`, `1447018070` | Generic objectives | [client] |

NPCs: [[The Rose that Knows]], [[Snowface]], [[Tald Daybreaker]], [[Arestol]], [[Ceard]], [[Assistant Headmaster Tov]], [[Embir]], [[Perci]]. Zone: [[NPCs - Shrouded Glade]]. Video also shows Shrouded Glade NPCs Tald Daybreaker and Justin Roderic near the warp stone. [video KYKRXeDqYDc/00000]

## Conflicts

1. **Ninja hand item.** Wiki "Shinobi Ninja Wristguard". The client has no such item; it has Shinobi Ninja Gloves (`loc 1336193860`, `2063608243`) but wristguards only for Soke, Kanja and Chunin. Client "Gloves" used, which assumes the wiki slot is the hands. Unconfirmed.
2. **Shard spelling.** Wiki "Wizard's Power shard of Regeneration III". The client capitalises Shard (`loc 1168914962`). Client used.
3. **Goal wording.** The wiki bonus text has no counter; video shows "- 1/4" and "- 2/4", matching the wiki's "(4)".
4. **Wiki enemy list.** "Wolves", "Cray", "Floren" and "Broodmother" are wiki only for this instance. Only the Elemental Guardian is seen in video.
5. **Dev tags.** "Misty Mountain - NOT USED" (`loc 1873970541`, `4168250283`) are old dev leftovers beside the live strings.

## Gaps

- The results screen, so stars, coins and the bonus piece are unconfirmed in video.
- Group size.
- Which non-guardian enemies are inside the instance.
- Snowface health and abilities (names only), and the "Boss Confusion" effect.
- Difficulty rating number on the lobby panel (the line was captured as a level requirement only).
- Which quest owns each description above (titles).
