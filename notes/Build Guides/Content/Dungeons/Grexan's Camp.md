---
type: content-dungeon
zone: Northern Wilds (client text says Briarwood)
combat: true
---

# Grexan's Camp

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Grexan's Camp" (`loc 610001547`, `3584067474`, game-data name `loc 2377828641`) | [client] |
| Panel description | "Trolls are training spiders to strengthen their army. Defeat General Grexan before the trolls become unstoppable!" (`loc 808194982`) | [client] [wiki] |
| Type | Forest dungeon, difficulty 4 | [wiki] |
| Entrance | Click the **spider web** in the northeast of the Wilds. It has an Atlas icon. Client quest text places it "in Briarwood" (`loc 1062088120`, `2364232962`, `3593632403`), which disagrees with the wiki zone (see Conflicts). | [wiki] [client] |
| Story hook | Forest trolls in the camp train spiders (and capture pet spiders and animals) for their army, led by General Grexan. "General Grexan is the mastermind behind training the spiders of the Grexan's Camp. Defeating him should put an end to the clan's activities here." (`loc 1984988926`) | [client] |
| Older internal name | Dev strings say "Hidden Valley - Grexan Fist Slam", "Hidden Valley - Grexan spider summon" and "Spiders summoned by General Grexan in Hidden Valley" (`loc 438575859`, `661654159`, `3539038389`). The dungeon was probably named Hidden Valley at some stage; unconfirmed. | [client] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 9 combat job** (difficulty 4 rule, `loc 1291047686`). The wiki infobox says "Levels 9 or higher". | [client] [wiki] |
| Seen playing at | Job and level not readable in the recording | [video 1qqVwVCxjGM/00006] |
| Members only | No. Members get an extra coin bonus on the results screen. | [wiki] |
| Unlocking quest | None found. Quests send players in (see Related quests). | [client] |
| Group size | Unknown. "Complete the Grexan's Camp encounter with a full group." exists (`loc 1163863200`). | [client] |

## Goals

The panel shows **"Goals (2)"**. [video 1qqVwVCxjGM/00006, 00074]

| Goal | Exact text | Sources |
|---|---|---|
| Main | **"Defeat General Grexan."** (`loc 4161890033`; also "Defeat General Grexan" `1121303603`). Video panel line "Defeat General Grexan". | [client] [video 1qqVwVCxjGM/00006, 00074] [wiki] |
| Bonus | Video reads **"Rescue: Free all the people trapped in cocoons?"** (the last word blurred). The client string is **"Free all the people trapped in cocoons."** (`loc 1499850166`). The wiki writes "Bonus: Free all the people trapped in cocoons. (3)". | [client] [video 1qqVwVCxjGM/00074] [wiki] |
| Bonus detail | "The spiders here have recently captured several people and wrapped them up in cocoons to snack on later. Open the cocoons to rescue them!" (`loc 2065138959`). Related pickup names: "Web Cocoon" (`loc 667889186`), "Cocoon" (`799699984`), "Solid / Striped / Spotted / Checkered Cocoon". | [client] |
| Panel state | Both goals ticked in a later frame. | [video 1qqVwVCxjGM/00074] |
| Result | A black screen with "YOU WIN! Grexan's Camp". | [video 1qqVwVCxjGM/00082] |
| Internal bonus group | "Grexan's Camp Bonus Objective" (`loc 3800147333`) | [client] |

Count: three people in cocoons (wiki only). [wiki]

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **General Grexan** (`loc 3257185804`, game data `2410609210`) | Boss health bar at the top. Taunts: "When we get saddles and ride into town on our spiders, you'll be sorry!" and "Spiders, attack!". Dev notes: "Grexan Fist Slam", "Grexan spider summon" (`loc 438575859`, `661654159`), "Grexan adds" (`loc 2386173666`, `3812335450`). | [client] [video 1qqVwVCxjGM/00006] |
| Boss adds | "Grexan's Wood Spider" (`loc 1634651122`, `1987949534`), "Grexan's Noxious Spider" (`loc 65908768`, `2499994905`) | "Grexan's. plays an effect then changes to one of 6 random models." (`loc 1835754033`), a dev note on the spawn effect. | [client] |
| Enemies | Troll Spider Trainer (`loc 3792279912`), Troll Spider Handler (`loc 3555192058`), Troll Naturalist (`loc 3596602690`), Forest Trolls | Quest text names them in the camp. The wiki page lists no enemies. Dev note "Spider Trainer Shout" (`loc 2516040236`). | [client] |
| Captives | People in cocoons | Free three (wiki). | [client] [wiki] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "264 Stars or less" (wiki wording). Not read from video (black result frame). | [wiki] |
| Coins on the prize wheel | **162 Coins** slot on each job's wheel | [wiki] |
| Members-only bonus | **176 Coins** | [wiki] |
| Bonus-goal clothing | One piece of clothing from a set for the player's job, random colour and body slot, for everyone in the instance | [wiki] |
| Achievements | "Complete the Grexan's Camp encounter." (`loc 616972832`), full-group version (`1163863200`), "Win Combat Encounter Grexan's Camp" (`2151712976`), "Group: Grexan's Camp" (`2010608854`) | [client] |
| Gear tier | Wheel gear carries "IV", matching difficulty 4. | [wiki] [client names] |

### Prize wheel by job

Each wheel has five items plus the 162 Coins slot. Every item name below exists in the client locale (checked with `cq.py loc`). [wiki] [client names]

| Job | Items |
|---|---|
| Archer | Archer's Necklace of Vitality IV, Archer's Power Shard of Regeneration IV, Archer's Recurve Bow of Flame, Battle Item Mystery Pack, Bodkin Archer Boots |
| Brawler | Battle Item Mystery Pack, Brawler's Drill Hammer of Dirt Kick, Brawler's Necklace of Vitality IV, Brawler's Power Shard of Regeneration IV, Stick and Move Brawler Boots |
| Medic | Battle Item Mystery Pack, Medic's Megasaw of Shock, Medic's Necklace of Vitality IV, Medic's Power Shard of Regeneration IV, Pressure Wave Medic Boots |
| Ninja | Battle Item Mystery Pack, Ninja's Jagged Scythe of Solar Flare, Ninja's Necklace of Vitality IV, Ninja's Power Shard of Regeneration IV, Sensei Ninja Tabi Boots |
| Warrior | Battle Item Mystery Pack, Threat Range Warrior Hightops, Warrior's Double Axe of Hurling, Warrior's Necklace of Vitality IV, Warrior's Power Shard of Regeneration IV |
| Wizard | Battle Item Mystery Pack, Disciple Wizard Shoes, Wizard's Jewel Wand of Lightning, Wizard's Necklace of Vitality IV, Wizard's Power Shard of Regeneration IV |

Bonus clothing sets by job: Archer Bobtailed or Bodkin, Brawler Barnburner or Stick and Move, Medic Hydrostatic or Pressure Wave, Ninja Kanja or Sensei, Warrior High Initiative or Threat Range, Wizard Adherent or Disciple. [wiki]

## Related quests and NPCs

Quest titles are not stored beside descriptions in the locale, so each row gives the description id. Givers are only those named in the text.

| Quest text (client) | Id | What it asks | Sources |
|---|---|---|---|
| "Enter Grexan's Camp and defeat General Grexan." | `709983517` | Defeat the general. Companions: `169948351`, `991240852`, `3767511030`, `1984988926`. | [client] |
| "Spinnerette knows that the trolls are training her children ... Go to Grexan's Camp and free him from his cage." | `183411461` | Free a spider. Companions: `2143329485`, `2752915335`, `3553035426`, `205510042`, `308201061`. | [client] |
| "Help! Every time I try to explore Briarwood a troll captures my pet spider and runs into Grexan's Camp." | `90007050` | Buttercup Webbywand's spider. Companions: `1729218102`, `3129805985`. | [client] |
| "You found an animal cage outside of Grexan's Camp in Briarwood? They must be smuggling animals!" | `2364232962` | Spider Trainer defeat quest (Bowman Bilter, archer line). Companions: `2614225169`, `1062088120`, `894549317`, `868520107`, `2643286811`, `3593632403`. | [client] |
| "Ree Peatpants has asked you to defeat the Stealthy Despoilers in Seaside as well as Spider Trainers and Naturalists in Grexan's Camp." | `4119265144` | Ninja job quest. Companions: `1607738573`, `65660571`, `2034268346`. | [client] |
| "Head to Grexan's Camp and defeat some Forest Trolls." / "defeat six Troll Spider Handlers" | `838885338`, `4205071207` | Warrior job quests. Completion `4435439`. | [client] |
| "Defeat Spider Trainers in Grexan's Camp." | `3225825575` | Defeat quest. Companion `471422368`. | [client] |
| Missing wallet: "Can you search Grexan's Camp for my wallet?" | not found | A character who says he would buy something but "lost my wallet traveling through the hedge maze" asks you to search the camp. The quest shows in a recording as "Missing Wallet", with the goal "Find ... wallet" and "Return to ... in Briarwood". Giver's name in video is read two ways. | [video 1qqVwVCxjGM/00006, 00074] |
| "Complete the Grexan's Camp encounter." / "Win Combat Encounter Grexan's Camp" | `616972832`, `2151712976` | Generic objectives | [client] |

NPCs: [[General Grexan]], [[Spinnerette]], [[Buttercup Webbywand]], [[Bowman Bilter]], [[Ree Peatpants]]. Zone: [[NPCs - Northern Wilds]] or [[NPCs - Briarwood]] (zone disputed).

## Conflicts

1. **Zone.** The wiki places Grexan's Camp in the northeast of the Wilds (category Wilds, infobox "Northern Wilds"). Client quest text says "enter Grexan's Camp in Briarwood" (`loc 1062088120`) and "outside of Grexan's Camp in Briarwood" (`loc 2364232962`). The video wallet quest also returns the player "in Briarwood". Client and video are used for the frontmatter hint; the Dungeons index still shows the wiki's Northern Wilds.
2. **Bonus wording.** Video "Rescue: Free all the people trapped in cocoons?" against client "Free all the people trapped in cocoons." and wiki "Bonus: ... (3)". The recording's "Rescue:" prefix is not in either other source and may be a bonus-goal label.
3. **Shard spelling.** Wiki "Ninja's Power shard of Regeneration IV" (lower-case s). Client "Shard" (`loc 3847042839`). Client used.
4. **Wiki enemy list.** The wiki page has no Mobs or Enemies section. The enemy list above comes from client quest text.

## Gaps

- Which zone is correct in the shipped game. Settle by finding a zone banner in video.
- Group size.
- Star payout read from the results screen (the recording's result frame is black).
- Where the three cocoons are and whether Grexan's spider adds come in waves.
- Boss health and ability effects (names only).
- Which quest owns each description above (titles).
