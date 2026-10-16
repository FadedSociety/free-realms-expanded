---
type: content-dungeon
zone: Sunstone Valley
combat: true
---

# Precursor Ruins

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

**What this is.** A real combat dungeon in Sunstone Valley: a tarantula-infested precursor temple whose rocklar guardians were enslaved by a spider queen named **Venoxa**. The wiki has no battle page for it, only a generic page about precursor ruins as places, so almost everything below comes from the client's text and dev strings. Do not confuse it with the other "precursor ruins" scattered through Sacred Grove (the wiki's list: Vale of the Ancients, Great Wall, Isle of Tears, Ramirez's Dig Site), which are places, not this instance. [client] [wiki]

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Precursor Ruins" (`loc 1437391086`, `2108272728`; game-data name `loc 3169130279`). Dev tag for the encounter: "Precursor Ruins - 313". | [client] |
| Panel description (by wording) | "Spider infestation! These ancient precursor ruins have been overrun with tarantulas. Fight your way through the caverns, discover their ancient guardians, and uncover the heart of the ruins!" (`loc 2767276185`). It has the shape of the other lobby blurbs; no frame of the panel was found to confirm. | [client] |
| Type | Cavern and temple dungeon. The wiki's "Dungeons" index lists it at **difficulty 5** (so a level 13 combat job); the wiki's own Precursor Ruins page is about the place only. | [wiki: Dungeons, Precursor Ruins] |
| Entrance | A **ruin gate** in Sunstone Valley with the interaction header **"Precursor Ruins"** and "Press X or mouse". The Sunstone Valley region panel lists "Precursor Ruins" with The Rumbledome, Sandscale Oasis, Red's Haven, Whirly's Scrap Camp, Rough Rider Raceway, Sunstone Valley Approach and Glimmerdeep Hydropower Dam. The Dig is northeast of the ruins (wiki), and a Gateway quest sends the player "to the nearby Precursor Ruins" (`loc 1137255156`, `696992562`). | [video G2tF5uF5cBE/01452, b6QfVC_2YpQ/00028] [wiki: The Dig] [client] |
| Story hook | Sarkin Stonehearth's dig crew was driven from the dig by giant spiders; Assistant Bogse was dragged into the ruins ("The spiders must have grabbed him!", `loc 876180279`). The Gateway Freewheelers' boss Boril Greybeard "was poking around in the nearby Precursor Ruins when he disappeared" (`loc 1137255156`). Warning from a Sunstone NPC: "You're not planning on going into the Precursor Ruins are ya? There be horrible giant tarantulas in there." (`loc 4206829172`) and "this place is full of creepy, man-eating tarantulas" (`loc 3484067677`). | [client] [wiki: Quest:Rescue in the Ruins] |
| Reward-in-story | The ruins hold working "floating" precursor artifacts: "You saved Bogse AND discovered the greatest find I've ever heard of! Functioning Precursor artifacts?" (`loc 595687844`). | [client] [wiki] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 13 combat job** if the wiki's difficulty 5 is right (client tier string "Requires a level 13 combat job", `loc 1323029355`). No frame of the actual panel was found, so the number is unconfirmed for this dungeon. | [wiki] [client, generic] |
| Seen near it at | Archer 20 | [video G2tF5uF5cBE/01452] |
| Members only | Unknown for entry. At least one drop is members-only (see Rewards). | [wiki] |
| Unlocking quest | None. Quests send players in (see Related quests). | [client] |
| Group size | Unknown. There are **no** "Complete the Precursor Ruins encounter", "with a full group" or "Group: Precursor Ruins" strings in the client. | [client: no match] |

## Goals

The panel's exact goal text was not seen. Candidate strings from the client, with how sure the link is:

| Goal | Exact text | Sources |
|---|---|---|
| Main (probable) | **"Find and defeat the queen spider deep within the Precursor Ruins."** (`loc 540067880`). The only goal-style string that names this place. A shorter "Defeat the spider queen!" (`loc 57473757`, `1753746483`) also exists but is not tied to this place (Arachnia's Lair also has a spider queen). | [client] |
| Bonus (unknown) | No "Precursor Ruins Bonus Objective" string exists in the client. Tunnel-themed strings that may belong here or to the Gateway "Tarantula Wandering Encounter" (`loc 93885984`): "Destroy All tarantula tunnels and knock out all tarantulas!" (`loc 3757841580`), "Collapse all the tarantula tunnels!" (`loc 2853979623`), "Destroy all the tarantula tunnels so the tarantulas will have a more difficult time getting in and out of the temple." (`loc 2772490446`, mentions "the temple"), "Destroy tarantula tunnels to stop the tarantulas from coming!" (`loc 697129597`). Not assigned. | [client] |
| Mechanic strings | "Click on the artifact to add your power to it!" (`loc 152515907`), "Click on the large artifact to add your power to it!" (`loc 676783675`), "You must defeat nearby opponents before you can turn on the artifact." (`loc 95077356`), "Quickly, use the artifact and takes its power! That is the only way you can defeat the spider queen!" (`loc 2771610245`). The last one names the spider queen and the artifact together, so it fits this place; the others are by wording only. | [client] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Venoxa** (`loc 1551834791`; game data `1308810956`), the spider queen | Dev notes: "Venoxa takes reduced damage from player attacks and abilities." (`loc 3003020211`); "Venoxa Artifact Buff" (`loc 1868828043`) and "Venoxa Artifact Debuff" (`loc 785733795`); "Ancient Artifact Power Beam - Venoxa" (`loc 1593092145`); "Precursor Ruins - 313 - Beam effect that hits Venoxa" (`loc 949244314`), "Beam for the player" (`loc 1606950941`), "healing beam from rocklar power source" (`loc 3024238250`); "Venoxa Invisible Controler Cube" (`loc 2656446186`); "Venoxa Blurb#6" (`loc 2922512771`). Speech by wording: "You're stronger than you look, but not strong enough! Rocklar slaves - ATTACK!" (`loc 3330216712`) and "My power fades? The artifact! You will pay for this!" (`loc 699106935`). The wiki lists Venoxa only on its boss list (no page). | [client] [wiki: Boss] |
| Minion | **Spawn of Venoxa** (`loc 3855117949`; game data `3499292762`) | "Spawn of Venoxa Knockback" (`loc 2315809752`); "Spawn of Venoxa ability played on players who get too close" (`loc 3718238087`). | [client] |
| Spiders | **Tarantula** (`loc 3650906579`), **Leaping Tarantula** (`loc 2725141712`) | "Leaping Tarantula teleport" (`loc 37962547`), "leaping spider leap" (`loc 1733358504`), "Tarantula ability" (`loc 3778053066`). The client also has Agitated, Skittering and Ambusher tarantulas, "Leaping Widow" and baby tarantulas; which are in this dungeon is not shown. Tarantula Tunnel spawner exists (`loc 512923587`). | [client] |
| Enslaved guardians | **Rocklar Guardian** (`loc 281206154`, `2691726200`), **Rocklar Shaper** (`loc 2775596314`) | Guard speech: "The temple must not be breached by outsiders. Rocklar." (`loc 705456043`), "You will not lure me away from guard post! Rocklar!" (`loc 346746067`), "Intruder. Rocklar. Eliminate Intruder. Rocklar." (`loc 733697614`). Shaper dev notes: "buff other" (`loc 3386915224`), "crow control" (`loc 213926295`), "primary attack weld attack" (`loc 3762070435`). "Spawner on death of enslaved guardians" (`loc 2374182993`). The wiki says rocklar guardians often serve as monster spawners here. | [client] [wiki: Rocklar] |
| Freed ally | **Freed Rocklar Guardian** (`loc 3155082900`; game data `597607856`) | "Finally free! We need to draw the queen away from the artifact, it is the source of her power! Rocklar!" (`loc 13364284`, same text as the wiki quote), "The queen must be stopped. You cannot survive her attacks if she comes after you. I will distract her while you get to the artifact. Rocklar!" (`loc 189249263`), "Thank you, hero. We used to guard this temple, but the spider queen took control of us. Now that I free I will help you defeat her." (`loc 3209546092`). On victory: "Thank you hero, for freeing us and defeating the spider queen. We must now return to our duties. You should leave now." (`loc 3762587092`). Dev: "Summon the freed guard" (`loc 3229706364`), "Spawn Freed Guardian" (`loc 895734560`). | [client] [wiki] |
| Mini-boss | Unnamed | Dev: "mini boss stun attack" (`loc 796696552`), "mini boss heal" (`loc 1261820228`). | [client] |
| Objects | **Rocklar Power Artifact** (`loc 177936578`), **Enormous Rocklar Artifact** (`loc 22907098`) | Probably the artifact the freed guard says is the queen's source of power. By wording only. | [client] |

Flow suggested by the strings (unproven order): fight through tarantulas and enslaved rocklar guardians, free guardians (who later help), lure Venoxa off the artifact, use the artifact to hurt her (she "takes reduced damage" otherwise), then defeat her. [client]

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Buckled Skull Jacket | "A Members only Freestyle Jobs Clothing item drop" acquired "through the Dungeon Precursor Ruins or through a Sunstone Valley Boss Chest". Client name matches (`loc 413478426`). | [wiki] [client name] |
| Boss chests | Wiki: Sunstone Valley boss chests give items such as Bones Flair Shard (`loc 3286740144`). Whether Venoxa's chest is one is not stated. | [wiki] |
| Prize wheel, coins, stars, bonus clothing | **Unknown.** The wiki has no battle page, no video shows a results screen, and the client holds no per-dungeon figures. | |
| Quest rewards | **Rescue in the Ruins**: the Cactoid (wiki, `loc 4029078262`). Other chain rewards unknown. These belong to the quests, not the dungeon. | [wiki] [client name] |
| Achievements | None (see Requirements). | [client: no match] |

## Related quests and NPCs

| Quest | Giver | What it asks | Sources |
|---|---|---|---|
| **Rescue in the Ruins** | Sarkin Stonehearth (The Dig) | "Rescue Assistant Bogse" by completing the dungeon; needs It's Too Quiet... first (wiki). Return text `595687844`. | [client: `876180279`, `878390704`, `2012736738`] [wiki] |
| **Finding the Missing Greybeard** (title by wording) | Griff (Sunstone Gateway) | "Scour the Precursor Ruins to find and rescue Boril Greybeard." (`loc 2641088394`); "Rescue Boril Greybeard" (`loc 2569427213`); description `345497198`. Completion `3021968095`. | [client] |
| Precursor technology research (title not found) | Mac Gravelfist | "research the precursor technology in the Precursor Ruins" (`loc 2181649697`); "Examine the strange precursor artifacts found in the Precursor Ruins." (`loc 1593747459`); "Examine artifacts in the Precursor Ruins" (`loc 3367435613`). A weak title candidate is "Precursor Prospecting" (`loc 1927835068`). | [client] |
| Oasis source (title not found) | Mac Gravelfist | "maybe there's a clue inside the nearby Precursor Ruins ... strange floating artifacts" (`loc 696992562`); "There may be evidence in the Precursor Ruins that helps explain the source of the oasis water." (`loc 4259954262`). | [client] |
| A flying machine for Garrick (title not found) | Garrick Highflyer, then 'Tip Top' Tommy | "searched the Precursor Ruins" for a floating object (`loc 3712154655`, `2957347056`). | [client] |

NPCs: [[Sarkin Stonehearth]], [[Assistant Bogse]], [[Boril Greybeard]], [[Griff]], [[Mac Gravelfist]], [[Garrick Highflyer]], [[Tip Top Tommy]], [[Venoxa]]. Zone and quest rows: [[NPCs - Sunstone Valley]]. Neighbours: [[The Rumbledome]], [[Sandscale Oasis]].

## Conflicts

1. **Page type.** The wiki lists Precursor Ruins as a difficulty 5 dungeon but it is not in the "Combat Mini-game" category, and its page describes ruins in general. The existence of a real combat instance is confirmed by client dev strings ("Precursor Ruins - 313 ..."), the lobby-style blurb, the gate prompt, and the wiki's Rescue in the Ruins page, which says Bogse is rescued by "completing Precursor Ruins Sunstone Valley Dungeon". [wiki] [client] [video]
2. **Level.** Difficulty 5 and level 13 are the wiki's tier rule; no panel line was read. [wiki]
3. **Venoxa's name and rank.** Client "Venoxa", wiki Freed Rocklar Guardian "Queen Venoxa", and "spider queen" or "queen spider" in quest text. One entity is assumed. [client] [wiki]
4. **Spider queens.** "Defeat the spider queen!" strings are shared with Arachnia's Lair and the Haunted Mines (Necrosis); only strings naming the Precursor Ruins or the artifact are used here. [client]

## Gaps

- The lobby panel (exact description, goals, difficulty, level line).
- Bonus goal, and which tunnel strings are this dungeon's.
- Group size, members-only status, results screen, prize wheel, coins, stars, bonus clothing.
- Layout and order of the fight, how many guardians are freed, and Venoxa's abilities beyond the dev tags.
- Mini-boss identity.
- Whether Garros (a rocklar boss with a crystal "Heal Aura", `loc 1025754866`, `769209083`) belongs to this dungeon; the client does not say.
