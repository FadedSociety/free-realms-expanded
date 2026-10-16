---
type: content-dungeon
zone: Snowhill
combat: true
---

# Howling Hills

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki. Video has no gameplay footage of this dungeon, so goals, enemies and rewards rest on client text and the wiki.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Howling Hills" (game data `loc 2369298095`, UI names `loc 1215986265`, `4292880689`). Dev-only duplicates "NOT LIVE - Howling Hills" (`loc 2177583776`, `1030469818`) also exist. | [client] |
| Panel description | "Trained snow wolves and their ice troll masters threaten Snowhill! Defeat their leader, Snognosh!" (`loc 97935654`) | [client] [wiki] |
| Journal description | "Snognosh is the leader of this roving band of Ice Trolls. They have raided and taken over a small snowy village. Run them out of town!" (`loc 2732465452`). A shorter line: "Ice Trolls have taken over this small snowy village. Teach them not to take what isn't theirs!" (`loc 157347705`) | [client] |
| Type | Mountain dungeon, difficulty 3 (infobox "Mountain", "Levels 5 or higher") | [wiki] |
| Entrance | Click the **barrier** in **Snowhill**. It has an Atlas icon. The client calls it "the Howling Hills, north of Snowhill" (`loc 3394134736`), "the pass through Howling Hills" (`loc 797046977`) and says an NPC stands "near the entrance to Howling Hills, not far from here" (`loc 3882321755`). | [wiki] [client] |
| Story hook | A huge ice troll, Snognosh, leads a force of ice trolls and trained wolves. They must be stopped before they reach Snowhill (`loc 797046977`, `1920647107`, `1731996392`). | [client] |
| Video | Only incidental: a loading-screen title card "Howling Hills" (no gameplay) and a world-map marker "Howling Hills 62m". | [video kXeF2h7Xsoo/00132, YxvolFOnaJM/00084] |
| Event names | "Snow Day: Howling Hills" (`loc 1085146205`) and the dev line "Form groups to take on the Howling Hills battle! SERVER 1" (`loc 1109136936`). Their meaning and dates are unknown. | [client] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 5 or higher.** Difficulty-3 rule. The client has the shared panel line "Requires a level 5 combat job" (`loc 2145075890`) and the refusal "You must have a combat job of at least level 5 to play this battle" (`loc 3173661191`). They are not tied to this dungeon by name. | [wiki: Dungeons, Howling Hills] [client, generic] |
| Seen playing at | Unknown. No gameplay frame exists. | [video] none |
| Members only | No. Members get an extra coin bonus (see Rewards). | [wiki] |
| Unlocking quest | None found for the dungeon itself. | [client] |
| Campaign | None found. No campaign text names Howling Hills. | [client] |
| Group size | Unknown. The silver achievement "Complete the Howling Hills encounter with a full group." (`loc 3116031680`) and "Group: Howling Hills" (`loc 2067422787`) exist, but no number. | [client] [wiki: Achievements] |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main | **"Find and defeat Snognosh!"** (`loc 3825908130`). Related client objective strings: "Defeat Snognosh" (`loc 1940525597`), "Enter Howling Hills and defeat Snognosh." (`loc 3289387601`), "Defeat Snognosh, the Ice Troll King!" (`loc 3694925383`). | [client] [wiki] |
| Bonus | **"Return all three Lost Ice Bear Cubs to their mother!"** (`loc 4049526188`). The wiki gives it as "Bonus: Return all three Lost Ice Bear Cubs to their mother!". The "Bonus: " prefix was not found in the client text for this string. | [client] [wiki] |
| Bonus detail | "A few ice bear cubs have gotten separated from their mother! Find them and return them to their worried mother!" (`loc 1448652886`). A cub NPC says "The lost bear cub seems nervous, but begins to follow you." (`loc 3000050148`). On delivery: "The bear cub seems happy to be with its mother once again." (`loc 1226425784`). | [client] |
| Internal bonus group | "Howling Hills Bonus Objective" (`loc 1396700682`) | [client] |
| Win text | "Win Combat Encounter Howling Hills" (`loc 93383990`); "Complete the Howling Hills encounter." (`loc 773375349`) | [client] |

How the bonus works, from the text only: each of three cubs follows the player, and the player brings it to the mother bear. The follow-then-deliver flow is inferred from the three strings above. No video shows it. [client, inferred]

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Snognosh** (game data `loc 1860779061`, UI `loc 1531420550`, `3243532495`) | Titled "the Ice Troll King" in quest text (`loc 3694925383`). Client ability strings (names only): "Snognosh Chilling Breath" (`loc 1471056918`, `4092135597`), "Snognosh Brain Freeze" (`loc 2756599452`, `3529926034`), "Snognosh Frost Rain" (`loc 4018652854`, `4141448947`). The dev line "howling hills - boss attack" (`loc 4191662328`) also exists. Effects unknown. | [client] [wiki] |
| Enemies | **Ice Trolls**, **Wolves**, **Big Mean Ice Trolls** (wiki list) | Client confirms "Big Mean Ice Troll" (game data `loc 3048214012`, UI `loc 3483836035`). Client quest text names "Ice Wolf Trainers" in Howling Hills (`loc 2289362402`), plus the enemy strings "Ice Wolf Trainer" (`loc 3750191317`) and "Trained Ice Wolf" (`loc 2520071568`, `3401191480`). The panel text calls the wolves "trained snow wolves". Quest text also sends players to "defeat Ice Trolls in the Howling Hills" (`loc 3864767315`). | [wiki] [client] |
| Friendly NPCs | **Lost Ice Bear Cub** (x3) and **Wild Mother Ice Bear** | Client names: "Lost Ice Bear Cub" (game data `loc 2773688057`, UI `loc 4293000492`), "Wild Mother Ice Bear" (game data `loc 14612718`, UI `loc 3779482736`). The wiki writes the cubs as "Lost Ice Bear Cubs". | [wiki] [client] |
| Other client names, not tied to this dungeon | "Ice Troll Berserker" (`loc 935317783`), "Ice Troll Warchief" (`loc 984835494`), "Ice Troll Scout" (`loc 1421032326`) | They may be overworld or other-battle enemies. The wiki lists the Scout under a separate battle, "Ice Troll Scout!". Not confirmed here. | [client] [wiki] |

Wave structure, number of enemies and the boss's health are unknown.

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "178 Stars or less" (wiki wording) | [wiki] |
| Coins on the prize wheel | **343 Coins** slot on every job's wheel. This is far above the other difficulty-3 entries ([[Bone Bog Cemetery]] 112). It may be a wiki error. See Conflicts. | [wiki] |
| Members-only bonus | **98 Coins**. The client holds a generic string "Bonus: 98 Coins!" (`loc 4007016911`) that is not tied to any dungeon. | [wiki] [client, generic] |
| Bonus-goal clothing | Completing the bonus goal gives every player in the instance one piece of clothing from a set for their current job. Colour and body slot are random. | [wiki] |
| Achievements | Bronze trophy (5 pts) "Complete the Howling Hills encounter." (`loc 773375349`); silver trophy (10 pts) "Complete the Howling Hills encounter with a full group." (`loc 3116031680`) | [client] [wiki: Achievements] |
| Gear tier | Wheel gear carries "III", matching difficulty 3. | [wiki] [client names] |
| Video | None. No result screen from this dungeon was found. | [video] none |

### Prize wheel by job

Item names are checked against the client locale. Every name below exists there (weapons are stored as `{n}Name{pl=...}` entries). Every wheel also has the 343 Coins slot. [wiki]

| Job | Items | Sources |
|---|---|---|
| Archer | Archer's Composite Bow of Blizzards, Archer's Necklace of Bark Skin III, Archer's Power Shard of Vitality III, Battle Item Mystery Pack, Cross Wind Archer Cap | [wiki] [client names] |
| Brawler | Battle Item Mystery Pack, Brawler's Anvil Hammer of Rumbling, Brawler's Necklace of Bark Skin III, Brawler's Power Shard of Vitality III, Rubber Match Brawler Helmet | [wiki] [client names] |
| Medic | Battle Item Mystery Pack, Medic's Bonesaw of Vitals, Medic's Necklace of Bark Skin III, Medic's Power Shard of Vitality III, Trauma Care Medic Cap | [wiki] [client names] |
| Ninja | Battle Item Mystery Pack, Ninja's Necklace of Bark Skin III, Ninja's Power Shard of Vitality III, Ninja's Scythe of 1000 Storms, Shinobi Ninja Cowl | [wiki] [client names] |
| Warrior | Battle Item Mystery Pack, Charge Attack Warrior Helm, Warrior's Battle Hammer of Cleaving, Warrior's Necklace of Bark Skin III, Warrior's Power Shard of Vitality III | [wiki] [client names] |
| Wizard | Battle Item Mystery Pack, Practitioner Wizard Fez, Wizard's Bone Wand of Glaciers, Wizard's Necklace of Bark Skin III, Wizard's Power Shard of Vitality III | [wiki] [client names] |

Bonus clothing sets by job: Archer Cross Wind and Drifting; Brawler Catchweight and Rubber Match; Medic Emergency and Trauma Care; Ninja Nusumi and Shinobi; Warrior Charge Attack and Combat Bonus; Wizard Practitioner and Scholar. [wiki] [client names: each set has 6 or 7 pieces in the locale]

## Related quests and NPCs

Quest titles are not stored beside descriptions in the locale, so each row gives the description text and its id. Givers are named only when the text names them.

| Quest text (client) | Id | What it asks | Sources |
|---|---|---|---|
| "A huge ice troll leads the attack! You must stop them before they can reach Snowhill! Use the pass through Howling Hills to find their force and defeat their leader, Snognosh, before it's too late!" | `797046977` | Main line: defeat Snognosh. Companions: `1920647107`, `3289387601`, `3893102942` ("Defeat Snognosh in the Howling Hills of Snowhill"), `3825908130`, `3694925383`, `1940525597`. Another objective line has the misspelling "Snogosh" (`loc 1670545450`). Giver not named. | [client] |
| "A troll scout could mean that more are coming soon! You must go into the Howling Hills and knock some sense into the trolls before they attack Snowhill!" | `1731996392` | Kill ice trolls. Companion: "Jonathon Forkpath thinks that the Ice Troll Scout might be the first wave of an attack. Go into the Howling Hills ..." (`4058148193`). | [client] |
| "Chaz wants you to take care of some trouble makers around Sacred Grove. Defeat Mugwort in Mugwort's Hollow, Necrolaceous in Dark Spore Depths, and Snognosh in the Howling Hills." | `2431923117` | Three-dungeon boss quest. Turn-in: "Return to me once you have taken care of Mugwort, Necrolaceous, and Snognosh." (`loc 3264721046`). | [client] |
| "Mayor Crystalline wants you to talk to a troubled Brawler named Jeni Shortfuse, who is stationed near Howling Hills on Ice Troll duty." | `284991750` | Start of the Jeni line. Companions: `755445428`, `3882321755`, `1692630527` ("We put her on Ice Troll duty at Howling Hills to repay her debt."), `2090355927` ("Help me pummel some Ice Trolls and I'll tell you my story."), `1269992516` and `2209636019` ("Pummel some Ice Trolls in the Howling Hills and then return to me."), `2085371497` (tell Jeni she is no longer on ice troll duty). Jeni's later quests move to [[Snowy Canyon]]. | [client] |
| "Little Plecky has lost her doll ... Search within the Howling Hills and see if you can find her doll!" | `227931480` | Find a doll. Companions: `61788199`, `2260527412`. | [client] |
| "Perfect timing! My friend, Patterwings, had to flee from his home. Unfortunately, he didn't grab his medicine. Can you go to the Howling Hills and find his medication?" | `3249672503` | Recover medicine from Patterwings' hut. Companions: `223971465`, `2411072262`, `979442635`, `2775634085`, `3466977529` (names Michi). Ninja job version: `1191913618`. | [client] |
| "You are to become the stuff of legends as you take over Fastvi's role as protector of the Free Realms! Travel to the Howling Hills and rescue three villagers from the vile ice trolls." | `627054189` | Warrior line: rescue three villagers. Companions: `962396221`, `3638736052` ("Take your new axe and rescue three trapped villagers"), `3195882668` (Warrior job quest), `4099574863` (return to Fastvi Frostflutter outside [[Frostfang Caverns]]), `810638163` (reward text). | [client] |
| "Valinda has asked you to rescue her missing penguins from the Howling Hills." | `2955920251` | Find baby penguins. Companions: `3606335981`, `2979343873`, `1254409301`, `2227870499`. | [client] |
| "Defeat the Ice Wolf Trainers in the Howling Hills." | `2289362402` | Kill wolf trainers. Companion `3037791910`. | [client] |
| "Defeat Ice Trolls in the Howling Hills, north of Snowhill." / "Defeat opponents in the Howling Hills." | `3394134736`, `1129900404` | Kill quests. Related: "Defeat ten opponents in Briarheart Caverns and Howling Hills" (`loc 356359373`), `3864767315`. | [client] |
| "Complete any of the Snowhill battles by defeating the bosses of Hot Springs Haven, The Bat Cave, Deep Mines, Howling Hills, Frostfang Caverns, or Snowy Canyon." | `3121919529` | Any-of-six Snowhill battle quest. Companions: `3611018622`, `2926552261`. | [client] |
| "Complete the Howling Hills encounter." / "Win Combat Encounter Howling Hills" | `773375349`, `93383990` | Generic objectives used by quests and achievements | [client] |

NPCs: [[Snognosh]], [[Jeni Shortfuse]], [[Mayor Crystalline]], [[Jonathon Forkpath]], [[Patterwings]], [[Valinda]], [[Fastvi Frostflutter]], [[Chaz]]. Zone: [[NPCs - Snowhill]].

## Conflicts

1. **Coin slot.** The wiki gives 343 Coins on every wheel. The other difficulty-3 page, [[Bone Bog Cemetery]], gives 112, and difficulty 5 gives 117. The client has no string "343 Coins". The figure is wiki-only and may be a typo. Kept as written, flagged.
2. **Item spelling.** The wiki writes "Wizard's Power shard of Vitality III" (lower-case "shard"). The client is "Wizard's Power Shard of Vitality III". Client wins. The wiki's Warrior link text uses a curly apostrophe ("Warrior’s Battle Hammer of Cleaving"); the client uses a straight one.
3. **Boss name.** One client quest objective reads "Snogosh" (`loc 1670545450`). Every other client string, and the wiki, read "Snognosh". Snognosh is used.
4. **Cubs.** The wiki lists "Lost Ice Bear Cubs" as the NPC and as one entry. The client NPC string is singular, "Lost Ice Bear Cub". Singular is used per NPC.
5. **Bonus text.** The wiki shows "Bonus: Return all three ...". The client string has no "Bonus: " prefix. The prefix is probably added by the goal panel, as video shows for other dungeons, but it is not seen here.
6. **Members bonus.** Wiki 98 Coins. The generic client string "Bonus: 98 Coins!" matches the number but is not named for this dungeon, so it does not confirm it.

## Gaps

- Group size and the party size behind "full group".
- Star payout, coin payout and members bonus in a real run. No result screen was found in video.
- Whether the 343 Coins wheel slot is real.
- Wave layout, enemy counts, boss health and the effects of Snognosh's three named abilities.
- Where the three bear cubs are and how the follow behaviour works.
- Which quest owns each description above (titles), and their rewards.
- The meaning of "Snow Day: Howling Hills" and the "SERVER 1" event string.
- Whether the wiki's "Wolves" are the client's "Trained Ice Wolf" or "Ice Wolf Trainer" or both.
