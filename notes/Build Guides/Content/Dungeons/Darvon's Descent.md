---
type: content-dungeon
zone: Sanctuary (Memorial Caverns)
combat: true
---

# Darvon's Descent

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki. Video has no gameplay footage of this dungeon, only world-map markers (see Overview), so goals, enemies and rewards rest on client text and the wiki.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Darvon's Descent" (`loc 96595886`, `373702072`; game-data name `4022539213`) | [client] |
| Panel description | "Pixies are getting kidnapped by Darvon's changeling followers! Free the pixies and defeat Darvon!" (`loc 876741041`) | [client] [wiki] |
| Type | Cave dungeon, difficulty 4 | [wiki] |
| Entrance | Click the **barrier** in the **Memorial Caverns**, in **Sanctuary**. It has an Atlas icon. The Memorial Caverns are "behind the Great Tree" (`loc 2067151334`). The wiki's exploration list puts the collectible "Darvon's Spot" in the water to the left of the dungeon entrance. The client has the name "Darvon's Spot" (`loc 3476177831`). | [wiki: Darvon's Descent, Exploration (Collections)] [client] |
| In video | Only as a red world-map marker "Darvon's Descent" in the Memorial Caverns, for example at 10 m beside a smelting furnace on a wooden walkway, and at 28 m and 33 m in blue-lit caves. Other frames show it as a "named location" at 40 m to 96 m. No frame enters the dungeon. A separate banner "Sanctuary Rare Exploration - Darvon's Spot" is the collectible, not the dungeon. | [video mu9Y82K62nk/00004, 00006, G0YPMcMAX4A/00006, 00008, qtN6Kohe7zc/00686, zIwAzf-mdqY/00308, KKYXlYPkv2o/00028] |
| Story hook | Darvon, a changeling, "has set up shop in the caverns" and uses changeling ninjas to defend his territory (`loc 813733078`). His followers kidnap pixies. He "has captured four Pixies for research" (`loc 3322798794`). | [client] [wiki] |
| Tooltip | None found. The Forgotten Caves have a "bonus rewards" promotion string, and no such string was found for this dungeon. | [client] |
| Wiki date | Page revision 2025-09-08. | [wiki] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 9 or higher.** Difficulty-4 rule. The client has the panel line "Requires a level 9 combat job" (`loc 1291047686`) and the refusal "You must have a combat job of at least level 9 to play this battle" (`loc 1882811692`). Neither names this dungeon. | [wiki: Darvon's Descent, Dungeons] [client, generic] |
| Seen playing at | Unknown. No video of a run. | [video] |
| Members only | No. Members get an extra coin bonus (see Rewards). | [wiki] |
| Unlocking quest | None. Many quests send players in (see Related quests). | [client] |
| Group size | Unknown. "Complete the Darvon's Descent encounter with a full group." (`loc 3095697270`) and "Group: Darvon's Descent" (`loc 2974367924`) exist, but no number is stated. | [client] [wiki: Achievements] |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main | **"Knock out Darvon!"** (`loc 2871702877`). Note "Knock out", not "Defeat". | [client] [wiki] |
| Bonus | **"Bonus: Free the captured Pixies (4)"** in the wiki. Client goal text "Free the captured Pixies" (`loc 1034991406`) and a separate "Bonus: " prefix string (`loc 897202295`). The wiki's "(4)" matches the client's "four Pixies". | [client] [wiki] |
| What the bonus involves | "Darvon has captured four Pixies for research... two of which are already in his lair for closer inspection. Save them by destroying their cages!" (`loc 3322798794`). "Deranged Changelings have captured innocent bystander Pixies. Destroy the cages to free them!" (`loc 1654884140`). Objects: "Pixie Cage" (`loc 4128488574`, game data `1719637053`). | [client] |
| Barrier clue | "Darvon is leading the way towards the kidnapped Pixies. Knock him out in order to break the barrier to Autumn and Flutter fly!" (`loc 3557360906`). This suggests two pixies sit behind a barrier that falls when Darvon is knocked out, and two are in cages. That reading is an inference. | [client] (inference) |
| Internal bonus group | "Darvon's Descent Bonus Objective" (`loc 3524006004`) | [client] |
| Win text | "Win Combat Encounter Darvon's Descent" (`loc 3012186552`); "Complete the Darvon's Descent encounter." (`loc 956092469`) | [client] |
| Unplaced goal-like text | "Locate the main audence chamber of Darvon himself!" (`loc 1443633791`) and "Defeat Darvon to retrieve the key" (`loc 4167002073`). Which quest or battle uses them is not known. | [client] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Darvon** (`loc 524741829`, game data `3574039490`) | Client ability names: "Darvon's Toe Crush" (`loc 3222700291`, `392343654`), "Darvon's Grip" (`loc 3025027839`, `4002673552`), "Darvon's Hook" (`loc 3996601953`), "Darvon Pin" (`loc 2658365915`, `2603790644`), "Darvon Teleport" (`loc 1253404844`, `3316293749`). The wiki Boss page ranks him a difficulty 4 boss. | [client] [wiki] |
| Enemies (wiki) | Changelings, Changeling Bruisers, Changeling Arcanus, Bats | Wiki list. | [wiki] |
| Enemies (client names) | **Changeling Bruiser** (`loc 2730526049`, `2930664756`), **Changeling Arcanus** (`loc 1435902026`, `1818611931`), Changeling Ninja (`loc 1830675970`, `1250680925`), Bat (`loc 1677961011`) and Cave Bat (`loc 1623002612`) | Quest text confirms changeling ninjas defend Darvon's territory (`loc 813733078`). The Arcanus carry transfiguring crystals; one quest names this dungeon with the Forgotten Caves (`loc 2268890788`). The bat names are generic and not tied to this dungeon by text. | [client] |
| Captive pixies (friendly) | **Autumn**, **Flutter Fly**, **Snappi**, **Eddi** | The wiki lists these four as the NPCs. The client has all four as NPC names (`loc 4044459935`, `3562406164`, `3536011623`, `3298403407`). The bonus text names Autumn and Flutter Fly (`loc 3557360906`). That they are the four captives is an inference from the count. | [wiki] [client names] |
| Developer strings | "darvons_descent 58:" followed by "Hostile Teleport" (`loc 1011376936`), "base pop attack + snare" (`2230235939`), "Summon Corrupted Spirit Gauntlet" (`2491442100`), "AE Root" (`2520657328`), "base pop ability" (`2716571198`), "boulder crush" (`2927491116`), "event ability" (`3332279266`), "Darvon Teleports himself for bladestorm" (`3805907101`), "Bladestorm" (`3819341792`). Also "darvons descent - snare boss for blade storm" (`3111965306`), "darvons descent - elite proc ability" (`733428181`), "Darvon's Descent - Rock block explode" (`2669545052`), "Darvon's Descent - Ninja Summon" (`3773789004`, `1631906282`). | Names only. Effects unknown. The "58" is not explained. An earlier "sampsons ability (deprecated)" (`loc 3610078684`) suggests the Brawler rival Sampson once had a role here; see Related quests. | [client] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "364 Stars or less" (wiki wording) | [wiki] |
| Coins on the prize wheel | **132 Coins** slot on every job's wheel. No client "Bonus: 132 Coins!" string exists. | [wiki] |
| Members-only bonus | **229 Coins**. No client "Bonus: 229 Coins!" string exists. | [wiki] |
| Bonus-goal clothing | Completing the bonus goal gives every player in the instance one piece of clothing from a set for their job. Colour and body slot are random. | [wiki] |
| Achievements | Bronze trophy (5 pts) "Complete the Darvon's Descent encounter."; silver trophy (10 pts) "Complete the Darvon's Descent encounter with a full group." | [client `loc 956092469`, `3095697270`] [wiki: Achievements] |
| Gear tier | Wheel gear carries "IV", matching difficulty 4. | [wiki] [client names] |

### Prize wheel by job

Item names use the client spelling. Every item was found in the client locale. One wiki typo is in Conflicts. Weapons are stored in the client as name templates (`{n}Name{pl=...}`). Every wheel also has the 132 Coins slot. [wiki]

| Job | Items | Sources |
|---|---|---|
| Archer | Archer's Necklace of Vitality IV, Archer's Power Shard of Vitality IV, Archer's Recurve Bow of Blizzards, Battle Item Mystery Pack, Bodkin Archer Cap | [wiki] [client names] |
| Brawler | Battle Item Mystery Pack, Brawler's Drill Hammer of Rumbling, Brawler's Necklace of Vitality IV, Brawler's Power Shard of Vitality IV, Stick and Move Brawler Helmet | [wiki] [client names] |
| Medic | Battle Item Mystery Pack, Medic's Megasaw of Vitals, Medic's Necklace of Vitality IV, Medic's Power Shard of Vitality IV, Pressure Wave Medic Helmet | [wiki] [client names] |
| Ninja | Battle Item Mystery Pack, Ninja's Jagged Scythe of 1000 Storms, Ninja's Necklace of Vitality IV, Ninja's Power Shard of Vitality IV, Sensei Ninja Cowl | [wiki] [client names] |
| Warrior | Battle Item Mystery Pack, Threat Range Warrior Helm, Warrior's Double Axe of Cleaving, Warrior's Necklace of Vitality IV, Warrior's Power Shard of Vitality IV | [wiki] [client names] |
| Wizard | Battle Item Mystery Pack, Disciple Wizard Fez, Wizard's Jewel Wand of Glaciers, Wizard's Necklace of Vitality IV, Wizard's Power Shard of Vitality IV | [wiki] [client names] |

Bonus clothing sets by job: Archer Bobtailed or Bodkin, Brawler Barnburner or Stick and Move, Medic Hydrostatic or Pressure Wave, Ninja Kanja or Sensei, Warrior High Initiative or Threat Range, Wizard Adherent or Disciple. [wiki] The wheel's head-slot piece is from the second-named set for all six jobs. Each set name was found in the client as an item name (for example "Bobtailed Archer Cap", "Barnburner Brawler Helmet", "Hydrostatic Medic Helmet", "Kanja Ninja Cowl", "High Initiative Warrior Helm", "Adherent Wizard Fez"). [client names]

## Related quests and NPCs

Quest titles are not stored beside descriptions in the locale, so these rows give description text and ids. Givers are only those the text names. Titles in the last column come from the job pages and are inferences there.

| Quest text (client) | Id | What it asks | Sources |
|---|---|---|---|
| "Another changeling, Darvon, has set up shop in the caverns, and also is using changeling ninjas to defend his territory! Kazuo must be held there! Head into Darvon's Descent and rescue Kazuo!" | `813733078` | Ninja line. Companions: `116212502`, "Search Darvon's Descent for the captive Kazuo." (`1920960461`), "Another group of changelings in Memorial Caverns might be holding Kazuo. Check Darvon's Descent for your captive master." (`3376299794`). See [[Ninja]]. | [client] [Ninja page] |
| "Tori was giving a tour of the memorial when a member of her tour group named Ceri went missing near Darvon's Descent. You must enter Darvon's Descent and find Ceri!" | `1259229194` | Find a missing tourist for Tori (named). Companions: `1323605662`, `2491030841`, objective "Find Ceri inside of Darvon's Descent." (`538977126`), reminder `920516691`. | [client] |
| "Changelings stole three of the memorial's precious artifacts! Can you please explore Darvon's Descent and see if you can find the artifacts?" | `2991605816` | Recover three artifacts. Companions: `3056620673`, `3048834171`, `808639705`, `1024874704`, `1426588532`, `2479790523`. Turn-in: "I knew you would come through! ... what Darvon would have done with those." (`1710406844`) | [client] |
| "The changeling, Darvon, sent some of his servants to steal all of our shock paddles! ... Head into Darvon's Descent and bring back our missing paddles!" | `3982142386` | Medic quest. Companions: `1882599266`, `2635429182`; completion news "[Player] ventured into Darvon's Descent and successfully recovered the stolen shock paddles!" (`705915133`). [[Medic]] places this as "Shocking Developments" (giver Clover there). | [client] [Medic page] |
| "Sampson has challenged you to a duel in Darvon's Descent. Show him who the real Brawler is!" | `3017559284` | Brawler rival duel. Companions: `4163237413` (Sampson's challenge text, "meet me in Darvon's Descent to settle this once and for all") and `1503117651`. Whether the duel is fought inside the normal dungeon battle is unknown. See [[Brawler]]. | [client] [Brawler page] |
| "Perhaps if you were to venture into the Forgotten Caves and Darvon's Descent and collect a transfiguring crystal from each Arcanus, I could study the crystals." | `2268890788` | Sandbur's Wizard research, shared with [[Forgotten Caves]] | [client] |
| "Complete the Darvon's Descent encounter." / "Win Combat Encounter Darvon's Descent" / "Complete the Darvon's Descent encounter with a full group." | `956092469`, `3012186552`, `3095697270` | Generic objectives used by quests and achievements | [client] |

NPCs: [[Darvon]], [[Autumn]], [[Flutter Fly]], [[Snappi]], [[Eddi]], [[Tori]], [[Sampson Idlefist]]. Zone: [[NPCs - Sanctuary]].

## Conflicts

1. **Wiki spelling.** The wiki Ninja wheel writes "Ninja's Neckace of Vitality IV"; the client item is "Ninja's Necklace of Vitality IV". Client spelling is used.
2. **Goal verbs.** The main goal says "Knock out Darvon!" (client and wiki agree). The panel description says "defeat Darvon". Both are kept. The goal text is the more specific one.
3. **Bonus line.** The wiki's "Bonus: Free the captured Pixies (4)" combines the client goal text, a separate "Bonus: " string and the count from the descriptions. The combined line was not seen in client or video.
4. **Name vs collectible.** "Darvon's Spot" (client and video) is a Sanctuary exploration collectible beside the entrance, not the dungeon. It is not an alternate name.
5. **Members coin figure.** The wiki's 229 and the wheel slot of 132 have no matching client "Bonus: N Coins!" string.
6. **Captive pixies.** The wiki NPC list has four pixies. The client bonus text names only two (Autumn and Flutter Fly) and says two others are "already in his lair". Which two sit in cages and which sit behind the barrier is inferred.

## Gaps

- Group size and the party size behind "full group".
- Star payout in a real run (only the wiki's "364 Stars or less") and the real coin payout.
- Wave structure, enemy counts and Darvon's health and phases; what "Bladestorm", "Hostile Teleport", "Corrupted Spirit Gauntlet" and "AE Root" do.
- Whether the bonus shows a 0/4 counter, where the four pixies are, and how the barrier works.
- Whether Sampson Idlefist appears in the dungeon fight or only in the quest.
- Which quest owns each description above (titles) and their rewards.
- No video of a run, so the lobby panel, the goal panel and the results screen have not been seen.
