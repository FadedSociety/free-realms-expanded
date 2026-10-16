---
type: content-dungeon
zone: Briarwood (Bristlewood)
combat: true
---

# Briar Patch

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Briar Patch" (`loc 429991411`, `2877402501`, game-data name `loc 3558511664`) | [client] |
| Panel description | "The evil wizard Azure performs sinister experiments in the Briar Patch. Put a stop to him!" (`loc 2304631555`) | [client] [wiki] |
| Type | Forest dungeon, difficulty 4 | [wiki] |
| Entrance | Click the **thorn plants** at the north end of **Bristlewood**, in **Briarwood**. It has an Atlas icon. The world map lists "Briar Patch" as a warp target ("Press to Teleport..."). Client: quests send you to "Sven's location near the Briar Patch" and "outside the Briar Patch" (`loc 163410406`, `370494468`). | [wiki] [client] [video d0g8BkcGNiw/00032] |
| Story hook | "An evil wizard has taken over the Briar Patch, corrupting it and its inhabitants with his dark experiments! His name is Azure, and we have been too afraid to face him and his spells..." (`loc 3933872900`). Also "He is corrupting everything in the Patch, and must be stopped!" (`loc 1462520443`). | [client] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 9 combat job** (difficulty 4 rule, `loc 1291047686`). The wiki infobox says "Level 9 or higher". | [client] [wiki] |
| Members only | No. Members get an extra coin bonus on the results screen. | [wiki] |
| Unlocking quest | None found. Quests send players in (see Related quests). | [client] |
| Group size | Unknown. "Complete the Briar Patch encounter with a full group." exists (`loc 1942616588`). | [client] |
| Video of play | None found. Video shows only the map warp entry. | [video d0g8BkcGNiw/00032] |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main | **"Defeat Azure."** (`loc 1367235519`; also "Defeat Azure" `3001054205`) | [client] [wiki] |
| Bonus | **"Defeat the Mutated Shepards before Azure is defeated!"** (`loc 33914393`). The wiki adds a count of (10). Related text: "Azure has mutated several creatures in Briar Patch. If you see the mutant wolf, defeat it quickly! It may not be around for a long time!" (`loc 4004867960`) and "Defeat the Briar Patch bears before they are mutated by Azure!" (`loc 2175577612`). | [client] [wiki] |
| Sub-fight | "Defeat Nettalie, servant of Azure." (`loc 1690803369`). On her defeat: "You defeated Nettalie and forced the magical barrier to fade away!" (`loc 1118524840`). A barrier blocks the way until she falls. | [client] |
| Internal bonus group | "Briar Patch Bonus Objective" (`loc 1571131770`) | [client] |

The bonus is time-limited by the boss: the Shepards must fall before Azure does. How they spawn and whether they come in waves is unknown. [client] [wiki]

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Azure** (`loc 747079180`, game data `1317891221`), an evil wizard | Dev strings: "Briar Patch - Azure Zap" (`loc 361091755`), "briar patch - boss heal" (`loc 825455871`), "briar patch - boss extra attack" (`loc 3523542519`), "Briar Patch: Azure Filling Cauldron visual" (`loc 1380729011`), "Azure Cube Target" (`loc 309267311`). Taunts: "Argh! Face me in the Briar Patch if you dare!" (`loc 3081093241`). | [client] [wiki] |
| Mini-boss | **Nettalie** (`loc 1777995598`, game data `1143684331`), "servant of Azure" | Her defeat drops the magical barrier. Line: "They have walked right into our trap Nettalie! Forest creatures, attack!" (`loc 2672565646`). | [client] [wiki] |
| Bonus targets | **Mutated Shepard** (`loc 1937894311`, game data `1600198396`) | Ten per the wiki. | [client] [wiki] |
| Enemies (wiki) | Man-Eating Plants, Briar Patch Wolves, Bears, Mutated Shepards, Nettalie | Wiki list. | [wiki] |
| Enemies (client) | "Briar Patch Bear" (`loc 2622719429`, game data `49018959`), "Briar Patch Wolf" (`loc 2399670370`, game data `3399538305`), "Wolf Vine Golem" (`loc 4131421719`), "Forest Giant" (`loc 986043600`), "Forest Giant Boss" (`loc 1600493580`), "Mini Forest Giant" (`loc 1569117458`) | Dev notes: "Briar Patch - Forest Giant Kick" (`loc 2597947267`), "Briar Patch - Barrier Knockback" (`loc 917485720`), "briar patch - bear basepop ability" (`loc 2340634996`), "Briar Patch 23: bonus mini boss aoe attack" (`loc 533858157`). Whether the Forest Giants are in this instance is unknown. | [client] |
| Allies / NPC cries | "It is time! Come fellow creatures of Briar Patch!" (`loc 1355831470`) | A boss or enemy call. Speaker unknown. | [client] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "182 Stars or less" (wiki wording). No video. | [wiki] |
| Coins on the prize wheel | **176 Coins** slot on each job's wheel | [wiki] |
| Members-only bonus | **255 Coins** | [wiki] |
| Bonus-goal clothing | One piece of clothing from a set for the player's job, random colour and body slot, for everyone in the instance | [wiki] |
| Quest-complete line | "[player] has defeated the wizard Azure inside of the Briar Patch and claimed his summoning crystal for Sven!" (`loc 216645771`) | [client] |
| Achievements | "Complete the Briar Patch encounter." (`loc 3739169015`), full-group version (`1942616588`), "Win Combat Encounter Briar Patch" (`859862734`), "Group: Briar Patch" (`2144076015`) | [client] |
| Gear tier | Wheel gear carries "IV", matching difficulty 4. | [wiki] [client names] |

### Prize wheel by job

Each wheel has five items plus the 176 Coins slot. Every item name below exists in the client locale (checked with `cq.py loc`). [wiki] [client names]

| Job | Items |
|---|---|
| Archer | Archer's Necklace of Bark Skin IV, Archer's Power Shard of Vitality IV, Archer's Recurve Bow of Stunning, Battle Item Mystery Pack, Bodkin Archer Gloves |
| Brawler | Battle Item Mystery Pack, Brawler's Drill Hammer of Cheapshot, Brawler's Necklace of Bark Skin IV, Brawler's Power Shard of Vitality IV, Stick and Move Brawler Gloves |
| Medic | Battle Item Mystery Pack, Medic's Megasaw of Vitamins, Medic's Necklace of Bark Skin IV, Medic's Power Shard of Vitality IV, Pressure Wave Medic Gloves |
| Ninja | Battle Item Mystery Pack, Ninja's Jagged Scythe of Shadow Armies, Ninja's Necklace of Bark Skin IV, Ninja's Power Shard of Vitality IV, Sensei Ninja Wristguards |
| Warrior | Battle Item Mystery Pack, Threat Range Warrior Gauntlets, Warrior's Double Axe of Whirlwind, Warrior's Necklace of Bark Skin IV, Warrior's Power Shard of Vitality IV |
| Wizard | Battle Item Mystery Pack, Disciple Wizard Gloves, Wizard's Jewel Wand of Vortex, Wizard's Necklace of Bark Skin IV, Wizard's Power Shard of Vitality IV |

Bonus clothing sets by job: Archer Bobtailed or Bodkin, Brawler Barnburner or Stick and Move, Medic Hydrostatic or Pressure Wave, Ninja Kanja or Sensei, Warrior High Initiative or Threat Range, Wizard Adherent or Disciple. [wiki]

## Related quests and NPCs

Quest titles are not stored beside descriptions in the locale, so each row gives the description id. Givers are only those named in the text.

| Quest text (client) | Id | What it asks | Sources |
|---|---|---|---|
| "Enter the Briar Patch and defeat the wizard Azure." | `2526422319` | Defeat Azure. Companions: `4282093033`, `3631129144`, `3933872900`. | [client] |
| "I heard that there is a powerful wizard inside of the Briar Patch. He must have some sort of summoning crystal and I want it." | `1050588066` | Sven's summoning crystal. Companions: `1305393074` (Wizard job quest), `1616829095`, `216645771`, `972505537`, `3700611734`, `163410406`, `370494468`, `2697230849`, `3743532866`. | [client] |
| "Please head into the Briar Patch and find my sister, Dewybloom!" | `1625790078` | Escort Dewybloom to the wizard. Companions: `2811260645`, `3601729293`, `319328099`, `2069765461`. | [client] |
| Lost animals: "Several small animals have been lost within the Briar Patch, and a girl named Meddri is concerned" | `343664516` | Rescue animals. Companion `86115058`. | [client] |
| "Susy is worried about the bears from Briar Patch getting too close to her farm." | `3359815202` | Defeat five bears. Companions: `4198001793`, `653729316`, `347835809`, `2136934869`. | [client] |
| "Chugilda is running low on meat. ... collect haunches of meat from the wolf vine golems" | `369763845` | Meat fetch. Companions: `693851549`, `3370537470`, `3373027538`, `374443422`, `3079093362`. | [client] |
| "There's a shiny stone hidden in the Briar Patch" (Corflugg, Zargoomi) | `582349080` | Fetch the stone. Companions: `822650344`, `1464879126`, `3329050432`, `4240543234`. | [client] |
| "Some peculiar vine creatures have come out of the Briar Patch recently. Barwen has asked me..." | `574469026` | Knock out vine creatures | [client] |
| Lead Champs through the Briar Patch to search for the locket. | `2187069506` | Escort. Companions: `1535548360`, `2128677304`. The "Lost in the Briar Patch" titles are marked NOT LIVE (`loc 3154811741`, `3918781666`). | [client] |
| "Complete the Briar Patch encounter." / "Win Combat Encounter Briar Patch" | `3739169015`, `859862734` | Generic objectives | [client] |

NPCs: [[Azure]], [[Nettalie]], [[Sven]], [[Dewybloom]], [[Susy]], [[Chugilda]], [[Zargoomi]]. Zone: [[NPCs - Briarwood]].

## Conflicts

1. **Warrior axe name.** The wiki text is garbled ("Warrior�s_Double_Axe_of_Whirlwind", a broken apostrophe and underscores). The client has "Warrior's Double Axe of Whirlwind" (`loc 2780901882`). Client used.
2. **Shard spelling.** Wiki "Wizard's Power shard of Vitality IV" (lower-case s) against the client "Shard". Client used.
3. **Wiki location label.** The wiki puts Briar Patch in both "Bristlewood" and "Briarwood" categories. The index says "Bristlewood, Briarwood". Kept as "Briarwood (Bristlewood)".
4. **Dev flags.** Two strings label a related quest "NOT LIVE". The dungeon strings are not flagged.

## Gaps

- No gameplay video: goals panel, Azure fight and result screen are unseen.
- Group size.
- Star payout (only the wiki's "182 Stars or less").
- How the Mutated Shepards spawn, and what Azure's cauldron and zap do.
- Whether Forest Giants appear in this instance.
- Which quest owns each description above (titles).
