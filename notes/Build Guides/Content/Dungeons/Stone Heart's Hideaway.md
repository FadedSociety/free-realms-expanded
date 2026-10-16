---
type: content-dungeon
zone: Merry Vale
combat: true
---

# Stone Heart's Hideaway

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

**What this is.** A seasonal combat instance tied to the Festival of Hearts in Merry Vale (February to March per the wiki): the player enters the hideout of the festival boss Stone Heart and defeats him and his changelings. It is a different thing from the open-world "Stone Heart's Rampage" in the town (love-potion arrows, no instance) and from the easy **Heartbreakers** battle (difficulty 1, Heartbreakers' camp). The same hideout also appears as a non-combat story visit in the quest Softening the Stone. The wiki has no page for the battle; it appears on the Festival of Hearts page, the Stone Heart page and the "Battles" index. [client] [wiki]

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Stone Heart's Hideaway" (`loc 1061411996`; game data `567675672`). The wiki's weapon pages misspell it "Hideway". | [client] [wiki] |
| Panel text (by wording) | **"Stop Stone Heart by entering his hideout and defeating him in combat!"** (`loc 1547462355`). The wiki's Stone Heart page says the same thing in its own words. No frame of the lobby panel was found. | [client] [wiki] |
| Type | Festival battle, **difficulty 5** ("Tier 5"). The Heartbreakers battle is "Tier 1". Not in the "Combat Mini-game" category; no wiki page of its own. | [wiki: Festival of Hearts, Battles] |
| Entrance | "Enter Stone Heart's hideaway on the western side of Merry Vale." (`loc 3155888320`). The NPC-side wording calls it a "hideout" (`loc 1251788045`, `2061189905`). Exact door or object is unknown. | [client] |
| When | Only during the Festival of Hearts event (wiki: February through March; 2010, 2011 and 2014 festivals are dated). The client promo: "Stoneheart and his Heart Breakers are back and wreaking havoc on the festival." (`loc 808006887`). | [wiki] [client] |
| Story hook | Stone Heart is the pixie Rylan, turned monster by dark magic and separated from Arianna; he "gathered the changelings around him" (wiki). Client lines: "Stone Heart is trying to ruin the holiday by breaking peoples hearts! You've found his hideout, go in there and take him out!" (`loc 2061189905`). Defeat cry: "Noooo! You have not seen the last of Stone Heart!" (`loc 647024719`). Arrival cry (wiki quote): "Invaders! Changelings! Stop them!" | [client] [wiki] |
| Story visit (non-combat) | In Softening the Stone the player finds the hideaway after Stone Heart retreats: "Find Stone Heart in his hideaway and speak to him." (`loc 2060386145`), "Enter Stone Heart's Hideout and Speak to Stone Heart." (`loc 1251788045`), "Speak to Stone Heart. Don't worry his minions will not attack you. This time..." (`loc 1476572121`). | [client] |
| Video | None of the instance. Related frames only: the open-world fight (`GR0ORW4cWts/00008`, "Shoot love arrows at Stone Heart - 61/100") and the Softening the Stone tracker (`Gk3v5EcwUBc/00014`). | [video] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 13 combat job** (wiki: "requires you to be level 13 combat job or above"; client difficulty 5 string `loc 1323029355`). No lobby frame was read. | [wiki] [client, generic] |
| Members only | **Yes, by the wiki**: "one must complete the appropriate members-only quests in the Festival of Hearts series". | [wiki] |
| Unlocking quest | **Softening the Stone** (Cudip). Then the daily repeatable quest **Stone Heart's Hideaway**, also from Cudip. | [wiki] [client] |
| Group size | Unknown. There are no "Complete the Stone Heart's Hideaway encounter" or "Group:" strings in the client. | [client: no match] |
| Repeat | "Stone Heart's Hideaway, given by Cudip (repeatable once a day)". | [wiki] |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main (probable) | **"Defeat Stone Heart"** (`loc 813734101`). The wiki and panel text agree the aim is to defeat him in combat. | [client] [wiki] |
| Minion goal (probable) | **"Defeat the minions of Stone Heart!"** (`loc 756429409`). Could be a sub-goal or a second panel line. | [client] |
| Quest-side objective | "Collect flowers that only grow in Stone Heart's Hideaway!" (`loc 1447454725`). Cudip's daily quest: "Help Cudip with his top secret solution by collecting flowers from within Stone Heart's Hideaway!" (`loc 2940859742`), "needs some rare flowers from within Stone Heart's Hideaway" (`loc 3517600679`, `3640360417`), "You need to enter Stone Heart's Hideaway!" (`loc 975160938`). The flowers are in the instance; whether they are the bonus goal is unknown. | [client] |
| Bonus | **Unknown.** There is no "Stone Heart's Hideaway Bonus Objective" string. | [client: no match] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Stone Heart** (`loc 419255199`; game data `945500272`) | Abilities (names only): **Stone Heart's Stone Slam** (`loc 2810703753`), **Stone Heart's Stonefist** (`loc 1289244022`), **Stone Heart's Stone Skin** (`loc 2008038419`), and the combat flags "Stone Heart *OnSpawn*" (`loc 1066560064`), "*OnHit*" (`loc 1732182403`), "*Invuln*" (`loc 219715362`). Effects are not described. | [client] [wiki] |
| Minions | **Changeling Heartbreaker** (`loc 3770695936`), "Stone Heart's Minion" (`loc 2454048414`), "Heart Breaker" (`loc 2517159261`) | "Heartbreaker Self Destruct" (`loc 2090488185`), "Heart Breaker Chain" (`loc 3934613293`). The wiki: "The Heart Breakers and Changelings attempt to attack you in the battle." | [client] [wiki] |
| Doves | Birds attack Stone Heart | Wiki quote "Blasted birds! Shoo! SHOOO!" when doves attack him. Dev note "vday combat 2 - stone fist shoos away birds (by beating them up)" (`loc 2364027310`) and ability "Stone Fist's Bird Shoo" (`loc 804037058`). | [wiki] [client] |
| Dev group | "vday combat 2 - ..." notes: knockback (`loc 535860728`), melee attack for non-player targets (`loc 1282729418`), basepop extra attack (`loc 2608938673`), mitigation buff (`loc 3176079948`). The "vday combat 2" tag is attributed to this fight because of the bird line; this is inference. A separate "vday combat" group (charm arrow `loc 3584507146`, charm attack `loc 4008040276`, basepop snare `loc 2224294864`, nuke `loc 863301142`) matches the Heartbreakers battle, whose dev note mentions charming NPCs (`loc 2553502269`). Rule for both: "npcs in vday combat encounters do not drop activatable power ups, only hp and mana" (`loc 687016359`). | [client] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Prize wheel | Wiki: the battle "rewards house, clothing, and fluff items, as well as a unique level 16 weapon for each job", and the weapon pages say "acquired from the prize wheel on the Stone Heart's Hideway encounter". The house, clothing and fluff items are not named. | [wiki] |
| Coins, stars, results | **Unknown.** No results screen or wiki figure. | |
| Treasure | The client has "Stone Heart's Treasure" (`loc 1980584385`) and "Stone Heart's Roses" (`loc 3597859952`). The wiki says the open-world Stone Heart drops a treasure chest of festival items; whether the Hideaway has one is not stated. | [client] [wiki] |
| Cudip's daily quest | Rewards unknown. | |

### Level 16 weapons by job

All six names exist in the client locale (checked with `cq.py loc`). Each weapon (wiki) is level 16, power 5, a "Prize Wheel Weapon" and a "Seasonal Weapon", with a standard attack plus the shared **Heart Breaker** ultimate ("Unleashes the ultimate attack of love", 6575 damage).

| Job | Weapon (client spelling) | Attack (wiki) | Sources |
|---|---|---|---|
| Archer | Beloved Bow (`loc 1953617640`) | Shot to the Heart, 2372 | [wiki] [client] |
| Brawler | Match-Maker Mallet (`loc 1337906201`) | Tough Love, 2372 | [wiki] [client] |
| Medic | Sweetheart Saw (`loc 2059619056`) | Heart Attack, 2372 | [wiki] [client] |
| Ninja | Everloving Edge (`loc 2793144955`) | Secret Admirer, 2609 | [wiki] [client] |
| Warrior | Heartthrob Hammer (`loc 2225765394`) | Love Struck, 2372 | [wiki] [client] |
| Wizard | Wooing Wand (`loc 547626363`) | Charm, 2372 | [wiki] [client] |

Client attack descriptions confirm the wording, for example "Beats up your opponents with the power of love!" (`loc 385185632`), "Strikes foes from the shadows with the power of love." (`loc 1397441263`), "Slashes foes with the power of love." (`loc 1412730518`), "Fires arrows charged with the power of love" (`loc 2145701978`), "Shoots spells enhanced by the power of love." (`loc 2612649001`). [client]

Not rewards of this fight: Stone Heart's Bear Costume, Stone Heart's Gummy Bear and the "Stone Hearted" furniture are Station Cash marketplace items (wiki), and Stone Heart's Pink Bear is the reward of the Bowheart achievement ("Hit Stone Heart with 1000 love arrows", `loc 3360433501`), which is the open-world fight. [wiki] [client]

## Related quests and NPCs

| Quest | Giver | What it does | Sources |
|---|---|---|---|
| **Softening the Stone** | Cudip | Hit Stone Heart with love-potion arrows during his rampage ("Shoot love arrows at Stone Heart", 100 needed per the video counter), then find his hideaway and talk. Opens this battle (wiki). Members-only. | [client] [video Gk3v5EcwUBc/00014, GR0ORW4cWts/00008] [wiki] |
| **Stone Heart's Hideaway** (Cudip's daily) | Cudip | Collect rare flowers inside the hideaway for his "top secret solution". | [client] [wiki] |
| **Sleeping Pixie** | Cudip | After the story visit: wake Arianna so she meets Rylan. | [client] [wiki] |
| **A Tale of Two Hearts** | Talespinner Yvelle | Opens the separate Heartbreakers battle, not this one. | [wiki] |
| **Mending Hearts** | Jecca Ruchcups | Drives off heartbreakers during the open-world rampage. Not an instance. | [client] [wiki] |

NPCs: [[Stone Heart]], [[Cudip]], [[Arianna]], [[Rylan]], [[Changeling Heartbreaker]], [[Talespinner Yvelle]], [[Jecca Ruchcups]]. Zone page: [[NPCs - Merry Vale]]. Sibling battle: Heartbreakers (in the same festival; no page here).

## Conflicts

1. **Hideaway or hideout.** The panel and rewards text say "Hideaway"; story quest strings say "hideout". Client title "Hideaway" used. [client]
2. **Wiki spelling.** The wiki weapon pages write "Stone Heart's Hideway". Client spelling used. [wiki]
3. **Stone Heart or Stoneheart.** A promo string writes "Stoneheart" (`loc 808006887`); every other string uses two words. Client name `loc 419255199`. [client]
4. **Level.** Wiki level 13; only the generic difficulty 5 client string supports it. [wiki] [client]
5. **Combat or story.** Quest strings treat the hideaway as a safe talk ("his minions will not attack you. This time..."); the battle quest treats it as a fight. Two uses of the same place are assumed. [client]

## Gaps

- Lobby panel (goals, difficulty, level line) and any video of the fight.
- Bonus goal and whether the flowers are it.
- Stone Heart's ability effects, health stages and how the Invuln flag is used.
- Group size, coins, stars, and the named house, clothing and fluff prizes.
- Which dev-note ability groups belong to this battle.
- Exact entrance object on the western side of Merry Vale.
