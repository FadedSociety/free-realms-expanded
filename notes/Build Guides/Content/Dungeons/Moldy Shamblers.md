---
type: content-dungeon
zone: Briarwood (Nettleseed, eastern)
combat: true
---

# Moldy Shamblers

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

> Moldy Shamblers! is a **node battle**, not an Atlas dungeon. The wiki files it under *Battle* and *Combat Mini-game*, difficulty 4, "started by clicking on a Moldering Fungaloid in Nettleseed, Briarwood". [wiki: Moldy Shamblers!]

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Moldy Shamblers!" (UI `loc 4263847977`, game data `loc 2329978711`); plain form "Moldy Shamblers" (`loc 2660771564`) | [client] [video liVAgboeCLg/00004] |
| Panel description | "Evil magic and mushroom monsters are creating shamblers! Venture in and stop them!" (`loc 2380049548`). Seen on the battle-select panel with "Difficulty:" shown as 4 of 5 stars, the prize list and a Daily Grab Bag offer. | [client] [video liVAgboeCLg/00004] [wiki] |
| Type | Forest battle, difficulty 4 | [wiki] [video] |
| Entrance | Click a **Moldering Fungaloid** (client `loc 1367475438`, game data `4265826352`) in Nettleseed, Briarwood. Quest text places the shamblers in "eastern Nettleseed". | [wiki] [client `loc 160007391`, `3829646810`] |
| Story hook | Whigguma, a sasparilla seller, is refused entry to Nettleseed; the guards say shamblers make it unsafe. Clearing them is the fix. | [client `loc 1761622797`, `307449156`, `160007391`] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 9 or higher.** Panel shows "Requires a level 9 combat job" (`loc 1291047686`). Related messages: "You must have a combat job of at least level 9 to play this battle" (`loc 1882811692`), invitee form (`loc 3240997533`). These strings are generic per tier, not tied to this battle by name. | [client] [video liVAgboeCLg/00004] [wiki: "Levels 9 and higher"] |
| Members only | No. Members get "Members Only Bonus: 40 Coins" (client label `loc 1925651180`); a string "Bonus: 40 Coins!" exists (`loc 3342032968`), one of many "Bonus: N Coins!" strings, so the match is weak evidence. | [wiki] [client] |
| Unlocking quest | None needed per source; the quest **Safety First** sends players here. | [video ATNDJ0I3sf0/00046, 00138] |
| Group size | Unknown. | gap |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main (only goal) | **"Defeat all Shamblers!"** (`loc 312921545`). The in-battle goal panel read "Goals (1)". No bonus goal. | [client] [wiki] [video ATNDJ0I3sf0/00046] |
| Quest-side objectives | "Knock out Moldy Fungaloids" (`loc 2831060905`) and "Knock out Rotten Fungaloids" (`loc 3278187382`); video counters "1/6" Moldy and "1/3" Rotten. Also "Defeal all Moldy Shamblers" (`loc 1334606463`, sic) and "Defeat Moldering Fungaloids in Briarwood." (`loc 1811046308`). | [client] [video ATNDJ0I3sf0/00046] |

No "Moldy Shamblers Bonus Objective" or "Win Combat Encounter Moldy Shamblers" string exists in the client.

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Enemy | **Moldy Shambler** (`loc 731873455`, game data `391771898`) and **Rotten Shambler** (`loc 2864343722`, game data `2031409020`) | Quest text: "Tackle a group of Fungaloids, knocking out several of the Moldy ones." (`loc 3221866086`) and "...the Rotten ones." (`loc 1680043675`). The quest counters (6 Moldy, 3 Rotten) are quest-level; the battle's actual count is not shown. | [client] [video ATNDJ0I3sf0/00046] |
| Boss | None listed. | The wiki page has no Bosses line. | [wiki] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "153 Stars" | [wiki] |
| Members Only Bonus | "40 Coins" | [wiki] |
| Prize wheel | One wheel per combat job, each with a **68 Coins** slot (as the wiki writes it). | [wiki] |
| Video wheel (Archer) | Centre prize "Archer's Power Shard of Regeneration IV"; other slots read "Small Energy Potion", "Coins" and a partly legible "Archer's Natural ... of Blessing". The first matches the wiki. The potion is not on the wiki. The partial name has no client match (see Conflicts). | [video liVAgboeCLg/00054] |
| Quest reward | "You receive 5 coins" shown in a Safety First frame during the fight. Unclear if it is the quest or battle reward. | [video ATNDJ0I3sf0/00046] |

### Prize wheel by job

Every item name exists in the client locale as written. [client, checked by exact name] [wiki]

| Job | Items (plus 68 Coins) |
|---|---|
| Archer | Archer's Power Shard of Regeneration IV, Archer's Recurve Bow of Booming, Archer's Ring of Regeneration IV, Battle Item Mystery Pack, Bodkin Archer Hoodie |
| Brawler | Battle Item Mystery Pack, Brawler's Drill Hammer of Chucking, Brawler's Power Shard of Regeneration IV, Brawler's Ring of Regeneration IV, Stick and Move Brawler Jacket |
| Medic | Battle Item Mystery Pack, Medic's Megasaw of Vitae, Medic's Power Shard of Regeneration IV, Medic's Ring of Regeneration IV, Pressure Wave Medic Jacket |
| Ninja | Battle Item Mystery Pack, Ninja's Jagged Scythe of Mysticism, Ninja's Power Shard of Regeneration IV, Ninja's Ring of Regeneration IV, Sensei Ninja Gi |
| Warrior | Battle Item Mystery Pack, Warrior's Double Axe of Frenzy, Warrior's Power Shard of Regeneration IV, Warrior's Ring of Regeneration IV, Threat Range Warrior Tunic |
| Wizard | Battle Item Mystery Pack, Disciple Wizard Long Cloak, Wizard's Jewel Wand of Energy, Wizard's Power Shard of Regeneration IV, Wizard's Ring of Regeneration IV |

Example loc ids: Bodkin Archer Hoodie `216849473`, Sensei Ninja Gi `310684847`, Small Energy Potion `2967150320`. [client]

## Related quests and NPCs

| Quest | Giver | Text and ids | Sources |
|---|---|---|---|
| **Safety First** | Whigguma (sasparilla seller at Nettleseed's gate) | Title `loc 3236921623` (game data `2667764867`). Steps seen in client text: "Whigguma can't get into Nettleseed to deliver her sasparilla. Speak to the guards and find out why." (`loc 2942987325`); "Speak to Chief Bilgragum in Nettleseed for an explanation." (`loc 1840694640`); Bilgragum: "It's too dangerous in Nettleseed for travelers. Between the plants, the trolls, and the shamblers, something bad is likely to happen." (`loc 307449156`); "Inform Whigguma of your findings." (`loc 1465931498`); Whigguma asks you to "knock out the Moldering Fungaloid shamblers in eastern Nettleseed so that the guards consider it safe enough to let me in?" (`loc 160007391`); "Return to Whigguma" (`loc 1178136541`), "Inform Whigguma of your success." (`loc 1886680868`). The recorded run shows the win banner then the objective "Return to Whigguma". The order shown is a reading of the text, not a verified journal sequence. | [client] [video ATNDJ0I3sf0/00046, 00138] |

Another line, "Tackle a group of Fungaloids in eastern Nettleseed to make it a safer place for travelers." (`loc 3829646810`), reads like the Safety First journal description, but the Adventurer page lists it as a candidate for the Chapter 1 quest "There's Fungus Among Us!" instead; the owner is unconfirmed. [client] [[Adventurer]]

NPCs: [[Whigguma]] (seen on the video overworld, `CR7BfAesNLM/00000`), [[Chief Bilgragum]]. Zone page: [[NPCs - Briarwood]]. The wiki has no quest page for this battle.

## Conflicts

1. **Enemy spelling in video.** The video quest tracker reads "Moldy Fungoids" and "Rotten Fungoids"; the client text is "Moldy Fungaloids" (`loc 2831060905`) and "Rotten Fungaloids" (`loc 3278187382`). Client spelling used.
2. **Client typo.** "Defeal all Moldy Shamblers" (`loc 1334606463`) is a misspelling in the locale; the goal text seen is "Defeat all Shamblers!".
3. **Archer wheel.** The video shows "Small Energy Potion" and "Archer's Natural ... of Blessing" which the wiki list lacks; the wiki shows a 68-coin slot and "Archer's Recurve Bow of Booming". The partial name matches nothing in the client (`Archer's Natural` and `Bow of Blessing` found no entries), so it is probably a misread or a different prize table. Unresolved.
4. No wiki item name lacked a client match, so no item corrections apply.

## Gaps

- Wave layout, shambler counts, boss or spawner, health.
- Real coin and star payouts; the quest reward of Safety First.
- Group size; respawn of the Fungaloid node.
- Whether Safety First is the only quest that starts it.
- Contents of the full prize wheel (video shows slots the wiki does not).
