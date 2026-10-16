---
type: content-dungeon
zone: Briarwood (Nettleseed)
combat: true
---

# The Mushroom Gigas

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

> The Mushroom Gigas! is a **node battle**, not an Atlas dungeon. The wiki files it under *Battle*, *Special Spawn* and *Members Only*, difficulty 3. The client calls it a "Wandering Combat Encounter" (`loc 3348947269`). No video frame of it was found.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "The Mushroom Gigas!" (UI `loc 1456495012`, game data `loc 2135275246`). The plain form "The Mushroom Gigas" (`loc 1101342148`, game data `2901551770`) is probably the quest title, by the same naming pattern as Nettleseed Nibblers. | [client] |
| Panel description | "Azure is causing nothing but trouble! Go in and stop him!" (`loc 2009158808`) | [client] [wiki] |
| Type | Forest battle, difficulty 3 | [wiki] |
| Entrance | Started by clicking **the Mushroom Gigas** in Nettleseed, Briarwood. | [wiki] |
| Spawn rule | "Special Spawn". After you accept a battle with the Mushroom Gigas, it disappears for an unknown duration. | [wiki] |
| Story hook | Quest text: "A Mushroom Gigas roams the lands of Nettleseed. Cargug has tasked you with defeating it." (`loc 1791384205`) and "We have a full on, free roaming, class six Mushroom Gigas on the loose in Nettleseed! ... If you can drop that thing, I'll be impressed!" (`loc 3182660409`) | [client] |
| Dev strings | "Mushroom Gigas Awaken" (`loc 3993718488`, game data `2054651957`), "Mushroom Gigas! Boulder Throw Ability" (`loc 2242314618`), "mushroom gigas! 132" and "mushroomg gigas! 132: AoE" (`loc 4207525121`, `2688768337`). Effects unknown. | [client] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 5 or higher** (infobox "Levels 5 and higher"; difficulty 3). Generic tier strings: "Requires a level 5 combat job" (`loc 2145075890`), "You must have a combat job of at least level 5 to play this battle" (`loc 3173661191`), invitee form (`loc 66524078`). They are not tied to this battle by name. | [wiki] [client] |
| Members only | **Yes (wiki).** "You must have membership or be grouped with someone who has membership to battle the Mushroom Gigas." No client members-only line for this battle was found. Members also get "Members Only Bonus: 74 Coins" (client label `loc 1925651180`); a string "Bonus: 74 Coins!" exists (`loc 1579014258`), one of many "Bonus: N Coins!" strings, so the match is weak evidence. | [wiki] [client] |
| Unlocking quest | None needed per the wiki; the quest below also sends players here. | [wiki] [client] |
| Group size | Unknown. | gap |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main (only goal) | **"Foil Azure's scheme!"** (`loc 397947052`). No bonus goal listed. | [client] [wiki] |
| Quest-side wording | "Defeat the Mushroom Gigas" (`loc 3404839574`), "Defeat all of the Mushroom Gigas!" (`loc 197990145`), "Defeat the Mushroom Gigas in the wilds of Nettleseed." (`loc 4116430349`) | [client] |
| Also in client | "Win Wandering Combat Encounter The Mushroom Gigas!" (`loc 3348947269`), a quest objective. | [client] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Azure** (`loc 747079180`, game data `1317891221`) | The wiki lists Azure as boss and uses a "Gigas Azure" image on his page. He is also the Briar Patch boss. How the two are linked in play is unknown. | [wiki: The Mushroom Gigas!, Azure] [client] |
| Enemy | **Mushroom Gigas** (`loc 794104169`, `1390168761`; game data `2112386074`) | The wiki writes "Mushroom Giants". Count and behaviour unknown. | [client] [wiki] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "77 Stars" | [wiki] |
| Members Only Bonus | "74 Coins" | [wiki] |
| Prize wheel | One wheel per combat job. The coin slot is written **" Coins" with no number** on the wiki (blank figure). | [wiki] |
| Quest reward | Not found. | gap |

### Prize wheel by job

Every item name exists in the client locale as written. [client, checked by exact name] [wiki] Same item set as [[Seaside Swoopers]] except the Power Shards are tier II and the rings tier III.

| Job | Items (plus a blank "Coins" slot) |
|---|---|
| Archer | Archer's Composite Bow of Stunning, Archer's Power Shard of Vitality II, Archer's Ring of Regeneration III, Battle Item Mystery Pack, Drifting Archer Boots |
| Brawler | Battle Item Mystery Pack, Brawler's Anvil Hammer of Cheapshot, Brawler's Power Shard of Vitality II, Brawler's Ring of Regeneration III, Catchweight Brawler Boots |
| Medic | Battle Item Mystery Pack, Emergency Medic Boots, Medic's Bonesaw of Vitamins, Medic's Power Shard of Vitality II, Medic's Ring of Regeneration III |
| Ninja | Battle Item Mystery Pack, Ninja's Power Shard of Vitality II, Ninja's Ring of Regeneration III, Ninja's Scythe of Shadow Armies, Nusumi Ninja Tabi Boots |
| Warrior | Battle Item Mystery Pack, Combat Bonus Warrior Hightops, Warrior's Battle Hammer of Whirlwind, Warrior's Power Shard of Vitality II, Warrior's Ring of Regeneration III |
| Wizard | Battle Item Mystery Pack, Scholar Wizard Shoes, Wizard's Bone Wand of Vortex, Wizard's Power Shard of Vitality II, Wizard's Ring of Regeneration III |

Example loc ids: Archer's Power Shard of Vitality II `3832684316`, Brawler's Power Shard of Vitality II `1656067803`. [client]

## Related quests and NPCs

| Quest | Giver | Text and ids | Sources |
|---|---|---|---|
| **The Mushroom Gigas** (title inferred from `loc 1101342148`/`2901551770`) | Sergeant Cargug, near the entrance to Nettleseed | "A Mushroom Gigas roams the lands of Nettleseed. Cargug has tasked you with defeating it." (`loc 1791384205`). Objectives: "Defeat the Mushroom Gigas in the wilds of Nettleseed." (`loc 4116430349`), "Win Wandering Combat Encounter The Mushroom Gigas!". Cargug's location: "near the entrance to Nettleseed" (`loc 1789311537`). Which of the Gigas lines `3182660409` and `17368034` ("Keep your eye out for a towering shamble of fungal destruction...") is the briefing is unconfirmed. | [client] [wiki: Sergeant Cargug] |
| Follow-on hint | unknown | "If you can take out a gigas, I think it's time to up the ante! In Grexan's Camp, trolls have been training spiders..." (`loc 991240852`). Implies a [[Grexan's Camp]] quest follows the Gigas. | [client] |

The Adventurer page lists the Cargug line (`loc 1791384205`) as an unplaced candidate for "Chapter 1: There's Fungus Among Us!" ([[Adventurer]]). So the quest title above is uncertain: it may be that Chapter 1 quest or a separate quest. [client]

Not this battle: the Mushroom Cavern boss Marasmius is also called "the mushroom gigas" (`loc 3203953768`); see [[Mushroom Cavern]]. NPCs: [[Sergeant Cargug]], [[Azure]]. Zone page: [[NPCs - Briarwood]].

## Conflicts

1. **Enemy name.** The wiki writes "Mushroom Giants"; the client enemy is "Mushroom Gigas" (`loc 794104169`). "Mushroom Giant Boss" (`loc 751644705`) is a trading card name and was not assumed to be this enemy.
2. **Boss versus quest text.** The panel and goal are about Azure. Quest text only mentions a roaming Mushroom Gigas, with no Azure. Both are client text, so both are kept; the link is unknown.
3. **Coin slot.** The wiki leaves the wheel coin amount blank. No source gives it.
4. **Members only.** Stated by the wiki only; not confirmed in the client.
5. No wiki item name lacked a client match, so no item corrections apply.

## Gaps

- The coin amount, quest reward and exact star/coin payout.
- Fight layout: whether Azure and the Gigas are separate phases, health, the effect of "Boulder Throw" and "Awaken".
- Group size and how members-only is enforced; respawn time after "disappears for an unknown duration".
- The exact title and giver wording of the Gigas quest.
- Time frame of the content.
