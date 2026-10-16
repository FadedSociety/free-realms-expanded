---
type: content-dungeon
zone: Briarwood (Nettleseed)
combat: true
---

# Nettleseed Nibblers

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

> Nettleseed Nibblers is a **node battle**, not an Atlas dungeon. The wiki files it under *Battle* and *Combat Mini-game*, difficulty 3, and says it is "started by clicking on a Nettleseed Nibbler in Nettleseed, Briarwood". [wiki: Nettleseed Nibblers!]

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Nettleseed Nibblers!" (UI `loc 3406377541`, game data `loc 1957758763`); the quest and monster-group name has no "!" (`loc 1015374158`, `3434772332`) | [client] |
| Panel description | "Mushroom monsters are eating everything in sight! Stop them!" (`loc 3143853963`) | [client] [wiki] |
| Entrance | Click a **Nettleseed Nibbler** (red-marker enemy nameplate in the open world) in Nettleseed, Briarwood. | [wiki] [video piWfe7lXZ9I/00360, 00414] |
| Quest text on the same enemy | The giver calls them weeds, not mushrooms: "thin out those ferocious weeds" (`loc 3982023502`). The wiki's [[Man-Eating Plant]] page also lists this battle. See Conflicts. | [client] [wiki: Man-Eating Plant] |
| Time frame | Unknown. | gap |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 5 or higher, any combat job.** Panel line "Requires a level 5 combat job" (`loc 2145075890`); entry message "You must have a combat job of at least level 5 to play this battle" (`loc 3173661191`); invitee form (`loc 66524078`). These strings are generic per tier, not tied to this battle by name. | [client] [wiki: "Level 5, Any Combat Job", difficulty 3] |
| Members only | No. Members get a coin bonus: "Members Only Bonus: 34 Coins". Client label "Members Only Bonus!" (`loc 1925651180`); a string "Bonus: 34 Coins!" also exists (`loc 2268039290`), one of many "Bonus: N Coins!" strings, so the match is weak evidence. | [wiki] [client] |
| Unlocking quest | None stated for the battle. The recorded run was started during the quest **Nettleseed Nibblers**. | [video KLx739bZjNs/00030] |
| Group size | Unknown. | gap |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main (only goal) | **"Defeat the Nettleseed Nibblers!"** No bonus goal is listed. | [client `loc 2871998889`] [wiki] [video KLx739bZjNs/00030] |
| Quest-side objective | "Knock out Nettleseed Nibblers" (`loc 3375186727`), counted in play as "Knock out Nettleseed Nibblers - 2/9" toasts. Whether the battle holds 9 nibblers is unknown (frame confidence low). | [client] [video KLx739bZjNs/00030] |

No "Nettleseed Nibblers Bonus Objective" or "Win Combat Encounter Nettleseed Nibblers" string exists in the client.

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Enemy | **Nettleseed Nibbler** (`loc 3459805504`, game data `2034647647`) | The only enemy named. Count and behaviour unknown. | [client] [wiki] |
| Boss | None listed. | The wiki page has no Bosses line. | [wiki] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "107 Stars" | [wiki] |
| Members Only Bonus | "34 Coins" | [wiki] |
| Prize wheel | One wheel per combat job, each with a **54 Coins** slot (as the wiki writes it). | [wiki] |
| Results screen | "YOU WIN!" then "Score Info / Enemies Defeated / Rewards"; the count read as 0 and the text behind the title is blurred. | [video KLx739bZjNs/00108, 00110] |
| Quest reward | Not shown by any source. | gap |

### Prize wheel by job

Every item name below exists in the client locale as written (checked by exact name). [client] [wiki]

| Job | Items (plus 54 Coins) |
|---|---|
| Archer | Archer's Composite Bow of Blizzards, Archer's Power Shard of Regeneration III, Archer's Ring of Regeneration III, Battle Item Mystery Pack, Drifting Archer Hoodie |
| Brawler | Battle Item Mystery Pack, Brawler's Anvil Hammer of Rumbling, Brawler's Power Shard of Regeneration III, Brawler's Ring of Regeneration III, Catchweight Brawler Jacket |
| Medic | Battle Item Mystery Pack, Emergency Medic Shirt, Medic's Bonesaw of Vitals, Medic's Power Shard of Regeneration III, Medic's Ring of Regeneration III |
| Ninja | Battle Item Mystery Pack, Ninja's Power Shard of Regeneration III, Ninja's Ring of Regeneration III, Ninja's Scythe of 1000 Storms, Nusumi Ninja Gi |
| Warrior | Battle Item Mystery Pack, Combat Bonus Warrior Tunic, Warrior's Battle Hammer of Cleaving, Warrior's Power Shard of Regeneration III, Warrior's Ring of Regeneration III |
| Wizard | Battle Item Mystery Pack, Scholar Wizard Medium Cloak, Wizard's Bone Wand of Glaciers, Wizard's Power Shard of Regeneration III, Wizard's Ring of Regeneration III |

Example loc ids: Battle Item Mystery Pack `1225426704`, Scholar Wizard Medium Cloak `244013527`, Nusumi Ninja Gi `2246318202`. [client]

## Related quests and NPCs

| Quest | Giver | Text and ids | Sources |
|---|---|---|---|
| **Nettleseed Nibblers** | Private Chaggis | Description: "I bumped into a Nettleseed Nibbler on my last shift and the thing tried to take off my hand. Don't let the name fool you. They'll do more than nibble! ... Can you thin out those ferocious weeds?" (`loc 3982023502`; title `loc 1015374158`). Objectives seen: "Defeat the Nettleseed Nibblers!", "Knock out Nettleseed Nibblers", then "Return to Private Chaggis" (`loc 1368741264`). Other client lines: "Knock out those Nettleseed Nibblers!" (`401597647`), "Knock out a pack of Nettleseed Nibblers." (`1188283273`), "Defeat Nettleseed Nibblers in Briarwood." (`2251818499`). Which line belongs to which quest field is not confirmed. | [client] [video KLx739bZjNs/00030, 00108] |

NPCs: [[Private Chaggis]] (client NPC `loc 2375647043`; no wiki page found). Zone page: [[NPCs - Briarwood]]. The wiki has no quest page for this battle.

## Conflicts

1. **What the enemies are.** The battle panel (`loc 3143853963`) and the wiki call them "mushroom monsters". The quest text (`loc 3982023502`) calls the targets "ferocious weeds", and the wiki lists the battle on its Man-Eating Plant page. Both are client or wiki text; no source says which the model is.
2. **Coins.** The wiki shows "54 Coins" as a wheel slot and "34 Coins" for the members bonus. Neither is confirmed by video.

No item-name corrections were needed: every prize item matches the client exactly.

## Gaps

- Group size, wave layout, nibbler count and health.
- Exact coin/star payout (video results screen was blurred) and quest reward.
- Whether the battle can be entered with no quest.
- Respawn behaviour of the Nibbler nodes after the battle.
- Which locale line is the quest title versus short objective.
