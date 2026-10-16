---
type: content-dungeon
zone: Snowhill
combat: true
---

# Snowy Canyon

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki. Video gives one results screen and a few map markers. It has no interior gameplay.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Snowy Canyon" (game data `loc 1429180355`, UI names `loc 407600028`, `3207905701`). A placeholder "Snowy Canyon Entrance Placeholder" (`loc 493598014`, `1965244404`) also exists. | [client] |
| Panel description | "The ice trolls and yetis are at war! Stop them before their fight reaches Snowhill!" (`loc 3946935837`) | [client] [wiki] |
| Journal description | "Ice trolls and yetis are fighting over the snowy canyon. They are causing trouble for the citizens of Snowhill and need to be run out." (`loc 1033072735`) | [client] |
| Type | Snow-area dungeon, difficulty 5 (the hardest tier). Infobox "Snow area", "Levels 13 or higher". | [wiki] |
| Entrance | Click the **barrier** in **Snowhill**. It has an Atlas icon. The client says "the gate to the Snowy Canyon" (`loc 1578499778`), "Snowy Canyon, north of Snowhill" (`loc 1644634710`, `1019302529`) and "The yetis up the path" (`loc 2755895875`). Video shows a "Snowy Canyon" signpost with a red X on a rope bridge over an icy gorge. | [wiki] [client] [video Dsk5NVq5YMU/00420] |
| Story hook | Ice trolls and yetis fight each other in the canyon. The yeti war leader, Unga Frostclaw, attacks the ice trolls, and the fighting could spill into Snowhill (`loc 2060332552`, `2403629791`, `3586250099`). | [client] |
| Tooltip | "Battles in Snowy Canyon give bonus rewards!" (`loc 1788284246`), a promotion string. Its window is unknown. | [client] |
| Event string | "Enter the Snowy Canyon with a battle group and defeat the dangerous yeti! SERVER 1" (`loc 3623792193`), a dev-style event line. Its meaning and dates are unknown. | [client] |
| Video, map markers | Quest markers with the destination "Snowy Canyon" at 1600 to 1663 m and a zone label "Snowy Canyon 95m" appear on the world HUD. They show where the area is relative to the player, not gameplay. | [video 1ZK3DH0t2YE/00004, 00050, 00108, 00116, 00162, CW9hNMMNV8w/00074] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 13 or higher.** Difficulty-5 rule. The client has the shared panel line "Requires a level 13 combat job" (`loc 1323029355`) and the refusal "You must have a combat job of at least level 13 to play this battle" (`loc 225515096`). They are not tied to this dungeon by name. | [wiki: Dungeons, Snowy Canyon] [client, generic] |
| Seen playing at | Unknown. The one results frame shows no job or level. A video title mentions a Medic, but the frame does not show it. | [video fMSzCfrLzc8/00062] |
| Members only | No. Members get an extra coin bonus (see Rewards). | [wiki] |
| Unlocking quest | None found for the dungeon itself. | [client] |
| Campaign | None found for this dungeon. A different campaign, set in [[Hot Springs Haven]], features its boss: "Unga Frostclaw is visiting the Hot Springs ... Earn the Clockwork Cowboy Bowler when you complete this campaign. Make sure you have already purchased and completed "Campaign 4: Lakeshore Shenanigans"." (`loc 865153698`) | [client] |
| Group size | Unknown. The silver achievement "Complete the Snowy Canyon encounter with a full group." (`loc 428128665`) and "Group: Snowy Canyon" (`loc 3967167436`) exist, but no number. | [client] [wiki: Achievements] |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main | **"Defeat the yeti's leader!"** (`loc 2397646746`). Related client objectives: "Defeat Unga Frostclaw" (`loc 350689074`), "Enter Snowy Canyon and defeat Unga Frostclaw." (`loc 3794133524`), "Defeat the yeti boss in Snowy Canyon, north of Snowhill." (`loc 1644634710`). | [client] [wiki] |
| Bonus | **"Find the Elusive Tranquil Sasquatch!"** (`loc 1067763583`). The wiki gives it as "Bonus: Find the Elusive Tranquil Sasquatch!". The "Bonus: " prefix was not found in the client text. | [client] [wiki] |
| Bonus detail | "There is a rumor that a tranquil sasquatch lives in this canyon but nobody can ever take a picture of it! If you happen to see him, chase him until you find his home!" (`loc 2485363970`) | [client] |
| Internal bonus group | "Snowy Canyon Bonus Objective" (`loc 1435702301`) | [client] |
| Win text | "Win Combat Encounter Snowy Canyon" (`loc 3596935201`); "Complete the Snowy Canyon encounter." (`loc 522775849`) | [client] |
| Goal count | The results screen shows "Goals 1/1" and "Bonus Goals 1/1" for a completed run, so one main goal and one bonus goal. | [video fMSzCfrLzc8/00062] |

How the bonus works, from the text: the sasquatch runs and the player chases it to its home. Client strings for the sasquatch in different states exist: "Roaming Sasquatch" (game data `loc 4227098462`), "Tired Sasquatch" (`loc 3364235990`), "Sleeping Sasquatch" (`loc 87743135`) and "Cold Sasquatch" (`loc 1710698413`). They may be stages of the chase. That is a guess from the names. [client, inferred]

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Unga Frostclaw** (game data `loc 1755324008`, UI `loc 1289647903`) | Yeti war leader. He also appears as the last fight in [[Hot Springs Haven]]: "He must have been visiting the haven from his home in Snowy Canyon" (`loc 3292573459`). Dev lines "snowy canyon - elite ability" (`loc 3877761380`) and "snowy canyon - yeti basepop extra attack" (`loc 2570447616`) exist. The first may be the boss or elites. Effects unknown. | [client] [wiki] |
| Enemies (wiki) | Ice Crawlers, Ice Trolls, Icetroll Champion, Yetis, Yeti Pounders | Client names: "Icy Crawler" (game data `loc 924164145`), "Ice Troll" (game data `loc 1440855783`), "Icetroll Champion" (game data `loc 2806251612`, UI `loc 4281410542`), "Yeti Pounder" (game data `loc 1318614414`, UI `loc 251233426`). Many other yeti names exist ("Frosty Yeti", "Icy Yeti", "Cave Yeti", "Yeti Mystic", "Yeti Frostlord"). Which ones spawn in the canyon is not shown. Quest text confirms both sides: "Defeat some of the yeti within Snowy Canyon!" (`loc 1091593956`) and "Find and knock out Ice Trolls at Snowy Canyon." (`loc 557938627`). | [wiki] [client] |
| Bonus target | **Elusive Tranquil Sasquatch** (game data `loc 720135146`, UI `loc 3842035294`) | See Goals. | [client] [wiki] |
| Score data | One run defeated 47 enemies (141,000 points, so 3,000 each). | [video fMSzCfrLzc8/00062] |

Wave layout, the route through the canyon, and the boss's health are unknown.

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "186 Stars or less" (wiki wording). The results frame has a "Rewards" panel, but the star and coin values are not recorded. | [wiki] [video fMSzCfrLzc8/00062] |
| Coins on the prize wheel | **117 Coins** slot on every job's wheel. This equals the [[Briarheart Caverns]] figure. | [wiki] |
| Members-only bonus | **304 Coins**. No client string "Bonus: 304 Coins!" exists. | [wiki] |
| Bonus-goal clothing | Completing the bonus goal gives every player in the instance one piece of clothing from a set for their current job. Colour and body slot are random. | [wiki] |
| Achievements | Bronze trophy (5 pts) "Complete the Snowy Canyon encounter." (`loc 522775849`); silver trophy (10 pts) "Complete the Snowy Canyon encounter with a full group." (`loc 428128665`) | [client] [wiki: Achievements] |
| Gear tier | Wheel gear carries "V", matching difficulty 5. | [wiki] [client names] |

### Scoring seen on the results screen

One completed run. [video fMSzCfrLzc8/00062]

| Line | Value |
|---|---|
| Enemies Defeated | 47, worth 141,000 points (3,000 each) |
| Time Bonus | 18:13, worth 181,000 points |
| Knock outs Remaining | 5/5, worth 250,000 points (50,000 each) |
| Goals | 1/1, worth 120,000 |
| Bonus Goals | 1/1, worth **75,000** |
| Total score | 767,000 |

The per-enemy, per-knockout and goal weights match [[Briarheart Caverns]]. The bonus goal is worth 75,000 here and 25,000 there. See Conflicts. [video fMSzCfrLzc8/00062, kJpAm8IAtjs/00424]

### Prize wheel by job

Item names are checked against the client locale. Every name exists there, with one spelling difference (Archer headwear, see Conflicts). Weapons and clothing are stored as `{n}Name{pl=...}` entries. Every wheel also has the 117 Coins slot. [wiki]

| Job | Items | Sources |
|---|---|---|
| Archer | Archer's Necklace of Vitality V, Archer's Power Shard of Regeneration V, Archer's Raptor Bow of Blizzards, Archer's Ring of Precision, Battle Item Mystery Pack, Pin Hole Archer Shades | [wiki] [client names] |
| Brawler | Battle Item Mystery Pack, Brawler's Atlas Hammer of Rumbling, Brawler's Necklace of Vitality V, Brawler's Power Shard of Regeneration V, Brawler's Ring of Precision, Power Punch Brawler Helmet | [wiki] [client names] |
| Medic | Battle Item Mystery Pack, Medic's Necklace of Vitality V, Medic's Power Shard of Regeneration V, Medic's Ring of Precision, Medic's Shockrod of Vitals, Wound Ballistic Medic Helmet | [wiki] [client names] |
| Ninja | Battle Item Mystery Pack, Ninja's Necklace of Vitality V, Ninja's Power Shard of Regeneration V, Ninja's Ring of Precision, Ninja's Shadow Blade of 1000 Storms, Soke Ninja Cowl | [wiki] [client names] |
| Warrior | Battle Item Mystery Pack, Called Shot Warrior Helm, Warrior's Necklace of Vitality V, Warrior's Power Shard of Regeneration V, Warrior's Ring of Precision, Warrior's Warlord Axe of Cleaving | [wiki] [client names] |
| Wizard | Archmagus Wizard Hat, Battle Item Mystery Pack, Wizard's Necklace of Vitality V, Wizard's Ornate Wand of Glaciers, Wizard's Power Shard of Regeneration V, Wizard's Ring of Precision | [wiki] [client names] |

This wheel has 6 items per job, like [[Briarheart Caverns]], but it uses a Necklace of Vitality V and a Ring of Precision where Briarheart has a Ring of Energy V. [wiki] [client names]

Bonus clothing sets by job: Archer Bull's-Eye, Pin Hole, Upshot; Brawler Bare-Knuckle, Knockdown, Power Punch; Medic Energy Transfer, Stopping Power, Wound Ballistic; Ninja Chunin, Jonin, Soke; Warrior Action Power, Called Shot, Critical Hit; Wizard Adept, Arcane Master, Archmagus. [wiki] (client spellings; each set has 6 or 7 pieces in the locale)

## Related quests and NPCs

Quest titles are not stored beside descriptions in the locale, so each row gives the description text and its id. Givers are named only when the text names them.

| Quest text (client) | Id | What it asks | Sources |
|---|---|---|---|
| "The ice trolls and the yeti are fighting each other in Snowy Canyon ... Defeat the leader of the yeti aggressors, Unga Frostclaw, and settle this conflict before it gets out of hand." | `2060332552` | Main line: defeat Unga. Companions: `2403629791`, `3586250099`, `3794133524`, `350689074`, `1644634710`. Giver not named. | [client] |
| "The yetis up the path have been stomping around, awaking Milo from his sleep. Head into Snowy Canyon and give their leader, Unga, a little nap of his own!" | `2755895875` | Defeat Unga for Milo. Companion `3730737698` ("Would you head into Snowy Canyon and knock that yeti out?"). A follow-up line says "So you managed to teach Unga a thing or two, eh?" (`loc 329593019`). | [client] |
| "Search within the Snowy Canyon to see if there are any clues to why the trolls and yeti are fighting!" | `628117216` | Clue hunt. Companions: `2978358759`, `1256399600` ("It strikes me as a little odd that the yetis were the aggressors here."). | [client] |
| "You are both brawlers and you probably trained under that rotten Sampson as well. She's serving out her punishment by putting her fighting skills to good use, ridding Snowy Canyon of ice trolls." | `567297379` | Brawler [[Jeni Shortfuse]] line, continued from [[Howling Hills]]. Companions: `669548179` (names Sampson Idlefist and the Mayor of Snowhill), `3213191293` ("use your pummel ability to defeat ten ice trolls"), `3839812555`, `557938627`. | [client] |
| "Hello, there! I was delivering my turnips to town, but it appears that the yetis got into my stock again. Can you please go to the Snowy Canyon and retrieve my missing turnips?" | `636790647` | Find stolen turnips. Companions: `2633133937`, `1019302529`, `1830881788`, `2015585983`, `208091165`, `190207954` (names Farmer Cinn). | [client] |
| "Chilly the snowman has been trapped by the Yetis! Head into snowy canyon and find Chilly!" | `833136493` | Free a snowman. Companions: `2271589192`, `95554079` (names Jimmy). | [client] |
| "Several young penguins have gone missing when they went exploring in the Snowy Canyon on a dare! Find them and bring them home!" | `1683492102` | Find penguins. Companion: `504793312`. | [client] |
| "Go collect samples of ice lotus from Snowy Canyon" (Boiling Fever in Seaside) | `2324529950` | Collect ice lotus. Companions: `525045854`, `732580517`. | [client] |
| "Some of Snowhill's supplies have gone missing, and it appears that the trolls may have taken them! Search the Snowy Canyon for the supply crates!" | `934702729` | Find crates. Companion: `3822154834`. | [client] |
| "The buckets of fish you found led towards Snowy Canyon?! ... The creatures there must be waiting until no one is watching and stealing the fish!" | `622948619` | Fish-thief chain. Giver not named. | [client] |
| "Defeat some of the yeti within Snowy Canyon!" / "Find the gate to the Snowy Canyon" | `1091593956`, `1578499778` | Kill yeti; reach the entrance. | [client] |
| "Complete any of the Snowhill battles by defeating the bosses of Hot Springs Haven, The Bat Cave, Deep Mines, Howling Hills, Frostfang Caverns, or Snowy Canyon." | `3121919529` | Any-of-six Snowhill battle quest. Companions: `3611018622`, `2926552261`. | [client] |
| "Complete the Snowy Canyon encounter." / "Win Combat Encounter Snowy Canyon" | `522775849`, `3596935201` | Generic objectives used by quests and achievements | [client] |
| "Unga Frostclaw is visiting the Hot Springs ... Earn the Clockwork Cowboy Bowler when you complete this campaign." | `865153698` | A campaign in [[Hot Springs Haven]], not this dungeon. Listed because it features the same boss. | [client] |

NPCs: [[Unga Frostclaw]], [[Elusive Tranquil Sasquatch]], [[Milo]], [[Jeni Shortfuse]], [[Sampson Idlefist]], [[Farmer Cinn]], [[Jimmy]], [[Chilly]]. Zone: [[NPCs - Snowhill]].

## Conflicts

1. **Bonus-goal score.** The video results frame for Snowy Canyon shows the bonus goal worth 75,000 points. [[Briarheart Caverns]] and Forest Troll Fort show 25,000. The enemy, knockout and goal weights are identical. One frame each, so a per-dungeon difference is likely, not proven.
2. **Head-slot spelling.** The wiki writes "Pinhole Archer Shades". The client has "Pin Hole Archer Shades" (game data `loc 3432799264`, UI `loc 1763295644`). Client wins.
3. **Brawler set spelling.** The wiki writes "Bare Knuckle". The client set is "Bare-Knuckle" (hyphen, for example `loc 1342456888`). The [[Briarheart Caverns]] page uses the unhyphenated form, so its "client spellings" note is slightly off for this set. The wiki also writes "Enery Transfer" for "Energy Transfer".
4. **Entrance word.** The wiki says "barrier". The client objective says "gate" (`loc 1578499778`). Both may describe the same object.
5. **Bonus text.** The wiki shows "Bonus: Find the Elusive Tranquil Sasquatch!". The client string has no "Bonus: " prefix. The prefix is probably added by the goal panel, as video shows for other dungeons, but it is not seen here.
6. **Members bonus.** Wiki 304 Coins. No matching "Bonus: 304 Coins!" client string, so it is wiki-only.
7. **Wiki enemy list.** It is a short list. The client names many more yeti types than "Yeti" and "Yeti Pounder". The real spawn table is unknown.

## Gaps

- Group size and the party size behind "full group".
- Star payout, coin payout and members bonus in a real run. The one results frame does not record the rewards.
- Level and job of the player in the single results frame.
- The route, wave layout and enemy counts, and whether trolls and yetis fight each other in the instance.
- Where the sasquatch is, how it flees, and whether the Roaming, Tired, Sleeping and Cold states are chase stages.
- Unga's abilities and health.
- Which quest owns each description above (titles), and their rewards.
- The meaning of the "SERVER 1" event string.
