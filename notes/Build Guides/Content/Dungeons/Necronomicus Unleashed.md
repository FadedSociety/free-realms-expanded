---
type: content-dungeon
zone: Merry Vale
combat: true
---

# Necronomicus Unleashed

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

**What this is.** A real combat instance: a group boss fight against the Gloam Lord of Necromancy in caverns beneath Merry Vale. It has two versions in the client: the normal fight and a later **Awakened** fight for level 20 players (the wiki trivia agrees there is a "Level 20 Awakened fight"). The wiki has no page for the battle itself (only a section on the boss page and a line on the "Battles" index), so most facts come from the client. [client] [wiki]

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Necronomicus Unleashed" (quest or battle text `loc 858689397`, `3401326779`; game data `2648973931`). Dev names: "Encounter 432 - Necronomicus' Lair" (`loc 3625684659`), "Necromancer's Lair" (`loc 2691893784`; game data `2593879256`) and "Necromancer's Lair Blocker" (`loc 2002625004`). | [client] |
| Panel text | **"WARNING: You are about to enter the lair of a Gloam Lord! The Royal Guard recommends that this only be attempted by a group of 5 very powerful battlers. Only the brave or the foolish should enter. You have been warned!"** (`loc 866380874`). The wiki prints the same text as the "Battle description". | [client] [wiki] |
| Type | Group battle, **difficulty 5**. The wiki's "Battles" index puts it at difficulty 5; it is not in the "Combat Mini-game" category and has no wiki page of its own. | [wiki] |
| Entrance | "Use the secret passage behind the mushrooms near the Den of Secrets" (`loc 2156346441`). Wiki: "within the caverns leading to the Den of Secrets, beyond a swirling portal of Gloam magic". The Den of Secrets itself is "the cavern north of Merry Vale" (`loc 1876015264`), so the passage is in that cave system. | [client] [wiki] |
| Setting | "beneath Merry Vale" (`loc 2004825146`); the rewards text says the fight frees "Merry Vale and the Southern Wilds from the necromancer's corrupting presence" (`loc 4100297378`). | [client] |
| Story hook | Necronomicus first appears as a "Mysterious Stranger" near the missing caravan south of Queensfields during Bixie-B-Gone (wiki). Fight text: "Necronomicus may have been defeated, but he has not been stopped! Defeat him at his full power in Necronomicus Unleashed!" (`loc 3448748083`). Return version: "Necronomicus, Gloam Lord of Necromancy, has been spotted again beneath Merry Vale!" (`loc 2156346441`). On defeat: "Necronomicus wasn't just a nasty guy, he was a Gloam Lord!" (`loc 2463334192`). | [client] [wiki] |
| Video | No frame of the instance itself. A level 20 Archer's tracker shows "Necronomicus Unleashed" as the active quest, with the marker 218 m away, while in Sunstone Valley (a quest tracker, not a location). | [video G2tF5uF5cBE/00000, 00262] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 13 combat job** for the normal fight (wiki: "level 13 combat job or above"; client difficulty 5 tier string `loc 1323029355`). **Level 20** for the Awakened fight: "This battle is recommended for a full group of level 20 players." (`loc 2156346441`); the Awakened weapons are level 20. See Conflicts. | [wiki] [client] |
| Group size | **5 recommended** by the panel warning ("a group of 5 very powerful battlers", `loc 866380874`). This is a recommendation, not an entry rule. The generic "full group" count is not defined anywhere else; no "Complete the Necronomicus Unleashed encounter" achievement strings exist. | [client] |
| Members only | Unknown for entry. The wiki marks the Wild weapons and the Of Light boots as Members only. | [wiki] |
| Unlocking quest | The wiki: the battle "can only be unlocked by getting the quest 'Necronomicus Unleashed' from the Fighter's Guild Message Board in Sanctuary", and it follows the Southern Wilds story (Heroes Journal Merry Vale part, around Unmasking the Villain). The client text of the quest (`loc 3448748083`) says it is the sequel to an earlier defeat. | [wiki] [client] |
| Job switch | Not seen. The battle is a combat area, so the usual switch is expected but unconfirmed. | |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main | **"Defeat Necronomicus!"** (`loc 577662353`); also "Defeat Necronomicus" (`loc 4229669973`). Quest-level objectives: "Complete Necronomicus Unleashed" (`loc 258526631`) and, for the return version, "Defeat the Awakened version of Necronomicus beneath Merry Vale in Necronomicus Unleashed!" (`loc 2004825146`). | [client] |
| Bonus | **None found.** There is no "Necronomicus Bonus Objective" string in the client. | [client: no match] |
| Possible sub-goal | "Destroy the Zombie Portals" (`loc 1233963050`, `1793505741`) with eight numbered dev strings "Destroy Zombie Portal 1" to "Destroy Zombie Portal 8" (for example `loc 333870958`, `3730548396`, `255810573`). The wiki says a zombie portal appears in this fight; the client does not say which encounter owns the numbered portals. Unassigned. | [client] [wiki] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Necronomicus**, "Gloam Lord of Necromancy" (`loc 3006978437`, `3935840341`; title `loc 3444961058`; game data `932079975`) | The wiki calls him the only Gloam Lord in the game. Dialog that matches wiki quotes: "Back for more?" (`loc 3571622297`), "Have you met my friends?" (`loc 661282997`), "My power is eternal!" (`loc 1459764415`), and (as the Mysterious Stranger) "Do not meddle in my affairs, 'hero.' ..." (`loc 2021704970`). | [client] [wiki] |
| Phases (wiki) | (1) Necronomicus fights with magic. (2) When defeated "the skeleton pieces will appear on the ground" and a **zombie portal** spawns Necrowart Zombies; destroying it makes him assemble a powered **skeletal form**. (3) In that form he smacks the ground to shake it and harm players; defeating it ends the fight. | [wiki] |
| Phases (client) | "[dev] Necronomicus Phase 3 Basic Attack" (`loc 264350541`) alongside "Necronomicus Basic Attack" (`loc 1963478812`, `2182182348`), which supports at least three phases. "Buff Package 1/2/3" (`loc 3180343573`, `1721405563`, `1592780761`). | [client] |
| Abilities | **Necrotic Gloamfire** (`loc 3074822869`): "Burns an opponent with Gloamfire. Nearby opponents also catch fire. Necronomicus' signature magic blast." (`loc 201406982`), with **Splash** (`loc 2523024004`): "Splash damage inflicted if a nearby ally is struck by Necrotic Gloamfire." (`loc 1144200077`). **Cave-in** (`loc 1576730357`): "Necronomicus strikes the ground, raining stactites down upon the ground." (`loc 3095627209`). "The great skeleton's chest opens and Necronomicus rains necrotic gloamfire down upon opponents in front of him." (`loc 3772051160`). "Necronomicus removes his head, which bounces around him while biting at unlucky opponents." (`loc 3601968300`). The wiki trivia calls it the first boss whose abilities can be dodged. | [client] [wiki] |
| Adds | **Necrowart Zombie** (wiki appearance list; client names `loc 1923750772`, `4143459490`, "Ancient Necrowart Zombie" `103459866`) spawned by the **Zombie Portal** (`loc 825559812`) | The Necrowart Zombie wiki page lists this fight among its appearances. Exact add counts unknown. | [wiki] [client names] |
| Aftermath | A guard-style line: "Necronomicus will be placed in a holding cell for as long as he can be held. Thank you for your help, citizen, and enjoy the spoils of your victory!" (`loc 4059893473`). | [client] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars, coins, bonus clothing | **Unknown.** No battle page, no results screen in video. | |
| Boots | "After returning to Sanctuary, an ancient pair of boots was awarded. They look like they are part of a larger set." (`loc 4100297378`). Wiki: one **"<Job> of Light Boots"** per job, Members only, level 2, "acquired by defeating Necronomicus". The client has the whole Of Light set (helmet, shoulderpads, breastplate, gauntlets, leggings, boots) for each job, but only the boots are awarded here. The queen's thanks: "The queen has asked me to bestow these rewards upon you" (`loc 3450879884`). | [client] [wiki] |
| Awakened weapon (level 20) | The wiki weapon pages: each is "randomly acquired by completion of the quest 'Necronomicus Unleashed'", level 20, power 5. The client slot label is "Level 20 Awakened Weapon" (`loc 369384098`). Items below. | [wiki] [client] |
| Wild weapon (level 2, Members only) | The wiki weapon pages say "acquired through defeating the Overworld Boss Necronomicus" and the boss page lists them under "Upon defeat". Items below. | [wiki] [client] |

### Weapons by job

All twelve names exist in the client locale (checked with `cq.py loc`).

| Job | Wild weapon (level 2) | Awakened weapon (level 20) | Boots |
|---|---|---|---|
| Archer | Archer's Wild Bow of Forbidden Magic (`loc 3985640422`) | Archer's Awakened Bow of Forbidden Magic (`loc 3443801602`) | Archer of Light Boots (`loc 179207758`) |
| Brawler | Brawler's Wild Hammer of Forbidden Magic (`loc 2494776816`) | Brawler's Awakened Hammer of Forbidden Magic (`loc 3270322538`) | Brawler of Light Boots (`loc 2091427569`) |
| Medic | Medic's Wild Bonesaw of Forbidden Magic (`loc 4209126640`) | Medic's Awakened Bonesaw of Forbidden Magic (`loc 2486274137`) | Medic of Light Boots (`loc 3035400151`) |
| Ninja | Ninja's Wild Sword of Forbidden Magic (`loc 4023287926`) | Ninja's Awakened Sword of Forbidden Magic (`loc 218281465`) | Ninja of Light Boots (`loc 1000231786`) |
| Warrior | Warrior's Wild Axe of Forbidden Magic (`loc 1929377381`) | Warrior's Awakened Axe of Forbidden Magic (`loc 1154275319`) | Warrior of Light Boots (`loc 930409689`) |
| Wizard | Wizard's Wild Wand of Forbidden Magic (`loc 1607964994`) | Wizard's Awakened Wand of Forbidden Magic (`loc 3270259384`) | Wizard of Light Boots (`loc 182694960`, `2310270687`) |

Sample stats (wiki, Archer's Awakened Bow): Archer's Shot 2609 damage, Focused Volley 9132 damage and 50% slow, Sniper Shot 11741 damage. Weapon descriptions in the client: Wild "crafted in the Southern Wilds with a hint of Necronomicus' dark magic" (for example `loc 1275115682`), Awakened "imbued with Necronomicus' dark magic" (for example `loc 3190220875`). [wiki] [client]

## Related quests and NPCs

| Quest | Giver | What it does | Sources |
|---|---|---|---|
| **Necronomicus Unleashed** | Fighter's Guild message board, Sanctuary (wiki) | The battle quest. Two client text sets: the first (`loc 3448748083`, `858689397`) and the return or Awakened one (`loc 2156346441`, `2004825146`). | [client] [wiki] |
| **Unmasking the Villain** (Southern Wilds 4-5) | Mayor McFairy line, then Captain Sunspark | Client goal "Locate and defeat the one responsible for the kidnappings in the Southern Wilds." (`loc 3723509814`); the wiki says the overworld Necronomicus fight becomes available during it. The client text itself does not name Necronomicus, so the link is wiki-only. | [client] [wiki] |
| **No More Necro(nomicus)** (title only; `loc 1792157612`) | Unknown | A quest title that names him. Its text was not found; a thanks line "A Gloam necromancer! That is some serious business!" (`loc 146575251`) may belong to it. By wording only. | [client] |
| **Bixie-B-Gone** | Sergeant Larkspur (client) | Where the Mysterious Stranger is met (wiki). | [wiki] [client] |
| Queen's reward text | Sanctuary NPC | `loc 3450879884`, `2463334192`, `4100297378` | [client] |

NPCs: [[Necronomicus]], [[Captain Sunspark]], [[Sergeant Larkspur]], [[Mayor McFairy]] (Southern Wilds story only). Zone pages: [[NPCs - Merry Vale]], [[NPCs - Sanctuary]]. Sibling instance: [[Den of Secrets]] (the passage is beside its cave). Weapon rows also appear on the job pages, for example [[Warrior]].

## Conflicts

1. **Level 13 against level 20.** Wiki and the generic difficulty 5 rule: level 13 combat job. Client: "a full group of level 20 players" (`loc 2156346441`). The wiki trivia separately mentions a "Level 20 Awakened fight", and the Awakened weapons are level 20, so the two readings are most likely **two versions of the fight**, not one disputed number. This is inference. Level 13 is used for entry, level 20 for the Awakened version. [wiki] [client]
2. **Wild or Awakened rewards.** The boss page lists the Wild weapons and boots as drops of "Necronomicus" and the Awakened weapons as the quest reward of "Necronomicus Unleashed"; the Wild weapon pages say "Overworld Boss". Which version of the fight gives which set is not proven. [wiki]
3. **Group size.** Client "5 very powerful battlers" (a recommendation); no entry rule or trophy defines a full group. [client]
4. **Overworld or instance.** The wiki describes three Necronomicus forms (a temporary Mysterious Stranger, an overworld enemy reached by a secret passage, and the battle boss). The client's "secret passage behind the mushrooms" text is attached to the return version of the battle quest. [wiki] [client]
5. **Wiki wording.** The boss page says the Wild items drop at "level 2" and members-only; the Of Light boots are also "level 2". Taken as the wiki's values; no client level was read. [wiki]

## Gaps

- Stars, coins, results screen, bonus goal and bonus clothing.
- Panel difficulty line and exact entry level in the client (only the generic tier string).
- Group size rule.
- Which version of the fight gives the Wild, Awakened and boots rewards.
- Phase triggers, health thresholds, number of Zombie Portals and Necrowart adds, and the effect of the "Buff Package" dev items.
- The quest text and giver of "No More Necro(nomicus)".
- Whether the Southern Wilds quest "Unmasking the Villain" really unlocks this fight.
