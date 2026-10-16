---
type: content-dungeon
zone: Snowhill
combat: true
---

# Frostfang Caverns

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki. Video has no frame of the dungeon interior, goals or results, so goals, enemies and rewards rest on client text and the wiki.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Frostfang Caverns" (game data `loc 238419199`, UI names `loc 2736837919`, `2939927131`) | [client] |
| Panel description | "A small child is calling for help from within Frostfang Caverns! Head in and rescue the child, but beware of Chillblaine - a gigantic wolf!" (`loc 2126344996`) | [client] [wiki] |
| Type | Cave dungeon, difficulty 4 (infobox "Cave", "Levels 9 or higher") | [wiki] |
| Entrance | Click the **barrier** in **Snowhill**. It has an Atlas icon. The client has the objective "Find the entrance to Frostfang Caverns." (`loc 2984405344`) and puts NPCs "outside of Frostfang Caverns" (`loc 3672144686`, `2344779759`, `4099574863`). | [wiki] [client] |
| Story hook | An ancient evil, Chillblaine, lives in the depths of the caverns and lures adventurers in "with lies and illusions" (`loc 650608766`, `2218820701`: "Not everything in Frostfang will be as it seems!"). The panel's "small child" is probably a lure. Chillblaine says "Fools! There is no Teek. I am Chillblaine!" (`loc 1486574036`), and "Teek" exists as an NPC string (game data `loc 2979176060`, UI `loc 2717410956`). Linking Teek to the child is an inference. | [client] (Teek link inferred) |
| Lore | Legend says the warrior Fastvi suffered his first defeat here and lost his axe. His weapon was shattered, and whoever finds the pieces and defeats Chillblaine summons Fastvi's spirit (`loc 687730370`, `3914817276`, `114964436`). | [client] |
| Tooltip | "All battles at Frostfang Caverns reward bonus tickets!" (`loc 1019304106`), a promotion string. Its window is unknown. | [client] |
| Wiki source note | The wiki page lists as its source the "Free Realms Combat Design Portfolio" of a game designer, so its goals and rewards may come from a design document, not from play. | [wiki] |
| Video | No dungeon frames. Only overworld frames in Snowhill that show "Frostfang Snarler" enemy markers. Those are a separate overworld enemy, not proof of the dungeon roster. | [video _a9VF878icg/00010, YxvolFOnaJM/00016] |

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **Level 9 or higher.** Difficulty-4 rule. The client has the shared panel line "Requires a level 9 combat job" (`loc 1291047686`) and the refusal "You must have a combat job of at least level 9 to play this battle" (`loc 1882811692`). They are not tied to this dungeon by name. | [wiki: Dungeons, Frostfang Caverns] [client, generic] |
| Seen playing at | Unknown. No gameplay frame exists. | [video] none |
| Members only | No. Members get an extra coin bonus (see Rewards). | [wiki] |
| Unlocking quest | None found for the dungeon itself. | [client] |
| Campaign | None found. No campaign text names Frostfang Caverns. | [client] |
| Group size | Unknown. The silver achievement "Complete the Frostfang Caverns encounter with a full group." (`loc 4134765256`) and "Group: Frostfang Caverns" (`loc 1179850933`) exist, but no number. | [client] [wiki: Achievements] |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main (wiki) | **"Investigate the cavern!"** (`loc 1220971519`). The wiki lists this as the main goal. The same text exists in the client. | [wiki] [client] |
| Boss objective strings | "Defeat Chillblaine" (`loc 4293698538`), "Enter Frostfang Caverns and defeat Chillblaine." (`loc 2425989597`). The client has them as quest or objective text. Whether the in-dungeon panel shows one of them as a second step is unknown. | [client] |
| Bonus | **"Defeat the giant bat!"** (`loc 3433368763`). The wiki gives it as the bonus goal. The text is identical. Its detail line is probably "There is a rumor that within the cavern is a giant bat who hordes treasure. Knock out the bat and see what kind of treasure it was guarding!" (`loc 2727509360`); that text says "the cavern" and does not name Frostfang. | [client] [wiki] (detail link inferred) |
| Internal bonus group | "Frostfang Caverns Bonus Objective" (`loc 2710069578`) | [client] |
| Win text | "Win Combat Encounter Frostfang Caverns" (`loc 3125540002`); "Complete the Frostfang Caverns encounter." (`loc 773420663`) | [client] |

The number of goals shown in the panel is unknown. The wiki lists two (one main, one bonus). [wiki]

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Chillblaine** (game data `loc 959328268`, UI `loc 3805681920`), also "Chillblaine the Deceiver" (game data `loc 837056531`, UI `loc 3711380827`) | A gigantic wolf (panel text). Lies and illusions; quest text says "Not everything in Frostfang will be as it seems!". Quest strings imply he can be hit by job abilities: "Use Kick on Chillblaine" (`loc 1159694499`), "Use Commanding Shout on Chillblaine" (`loc 2192945840`), "Use Spinning Blade on Chillblaine" (`loc 3322368188`), "Use Quake on Chillblaine" (`loc 3452480767`). Dev line "frostfang caverns boss ability" (`loc 935329041`). Ability names and effects unknown. | [client] [wiki] |
| Enemies (wiki) | Wolves, Bats, Ice Crawlers, Mammoth Crawlers | Client spells the last two "Icy Crawler" (game data `loc 924164145`, UI `loc 2402395289`) and "Mammoth Crawler" (game data `loc 1291206330`, UI `loc 2474514970`). Quest text confirms wolves: "There are wolves in there" (`loc 1404846525`), "Defeat wolves in Frostfang Caverns" (`loc 1074594800`). | [wiki] [client] |
| Bonus target | **Giant Bat** (game data `loc 1792121157`, UI `loc 1041065255`) | Matches the bonus goal text. Starts as a hoarder of treasure per the rumor line. Health and behaviour unknown. | [client] [wiki] |
| Client names that may belong here | "Frostfang Bat" (`loc 4133118994`), "Frostfang Wolf" (`loc 1464698360`), "Elder Frostfang Wolf" (`loc 3208817585`), "Frostfang Alpha" (`loc 3628885318`), "Frostfang Prowler" (`loc 2226443022`), "Frostfang Howler" (`loc 63695428`), "Frostfang Growler" (`loc 1390519959`) | These are game-data names with the dungeon's prefix. Whether any is a dungeon spawn, or an overworld or other-battle enemy (the battle [[Frostfang Snarlers|Frostfang Snarlers!]]), is not shown. | [client] (placement unknown) |
| Named abilities (client dev strings) | "frostfang caverns - cave in" (`loc 950652070`), "frostfang caverns - elite aoe damage" (`loc 4147078979`), "Frostfang - Arctic Blast" (`loc 1983701938`), "Frostfang - Icy Claws" (`loc 2368825867`) | Names only. Effects unknown. | [client] |
| Friendly NPC | Possibly "Teek", the child | See Overview. A trap, not a friendly, if the inference holds. | [client] (inferred) |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "242 Stars or less" (wiki wording). The same figure is printed on the [[Sweetwater Climb]] wiki page, so treat it with care. | [wiki] |
| Coins on the prize wheel | **97 Coins** slot on every job's wheel | [wiki] |
| Members-only bonus | **220 Coins**. No client string "Bonus: 220 Coins!" exists. | [wiki] |
| Bonus-goal clothing | Completing the bonus goal gives every player in the instance one piece of clothing from a set for their current job. Colour and body slot are random. | [wiki] |
| Achievements | Bronze trophy (5 pts) "Complete the Frostfang Caverns encounter." (`loc 773420663`); silver trophy (10 pts) "Complete the Frostfang Caverns encounter with a full group." (`loc 4134765256`) | [client] [wiki: Achievements] |
| Gear tier | Wheel gear carries "IV", matching difficulty 4. | [wiki] [client names] |
| Video | None. No result screen from this dungeon was found. | [video] none |

### Prize wheel by job

Item names are checked against the client locale. Every name below exists there (weapons and some clothing are stored as `{n}Name{pl=...}` entries). Every wheel also has the 97 Coins slot. [wiki]

| Job | Items | Sources |
|---|---|---|
| Archer | Archer's Power Shard of Regeneration IV, Archer's Recurve Bow of Explosions, Archer's Ring of Energy IV, Battle Item Mystery Pack, Bodkin Archer Quiver | [wiki] [client names] |
| Brawler | Battle Item Mystery Pack, Brawler's Drill Hammer of Pummeling, Brawler's Power Shard of Regeneration IV, Brawler's Ring of Energy IV, Stick and Move Brawler Shoulderpads | [wiki] [client names] |
| Medic | Battle Item Mystery Pack, Medic's Megasaw of Alarm, Medic's Power Shard of Regeneration IV, Medic's Ring of Energy IV, Pressure Wave Medic Backpack | [wiki] [client names] |
| Ninja | Battle Item Mystery Pack, Ninja's Jagged Scythe of Shuriken Storm, Ninja's Power Shard of Regeneration IV, Ninja's Ring of Energy IV, Sensei Ninja Shoulderguard | [wiki] [client names] |
| Warrior | Battle Item Mystery Pack, Threat Range Warrior Pauldrons, Warrior's Double Axe of Earthquake, Warrior's Power Shard of Regeneration IV, Warrior's Ring of Energy IV | [wiki] [client names] |
| Wizard | Battle Item Mystery Pack, Disciple Wizard Talisman, Wizard's Jewel Wand of Firestorm, Wizard's Power Shard of Regeneration IV, Wizard's Ring of Energy IV | [wiki] [client names] |

Bonus clothing sets by job: Archer Bobtailed and Bodkin; Brawler Barnburner and Stick and Move; Medic Hydrostatic and Pressure Wave; Ninja Kanja and Sensei; Warrior High Initiative and Threat Range; Wizard Adherent and Disciple. [wiki] [client names: each set has 6 or 7 pieces in the locale]

## Related quests and NPCs

Quest titles are not stored beside descriptions in the locale, so each row gives the description text and its id. Givers are named only when the text names them.

| Quest text (client) | Id | What it asks | Sources |
|---|---|---|---|
| "An ancient evil known as Chillblaine has revealed itself in the depths of Frostfang Caverns ... You must enter Frostfang and defeat Chillblaine before his deceptions can lure in more victims." | `650608766` | Main line: defeat Chillblaine. Companions: `630106188`, `2218820701`, `2425989597`, `4293698538`. Giver not named. | [client] |
| "Use Kick / Commanding Shout / Spinning Blade / Quake on Chillblaine" | `1159694499`, `2192945840`, `3322368188`, `3452480767` | Job-ability practice objectives on the boss. The quests that own them are unknown. | [client] |
| "Please wait! Can you help me? My dog, Lailu, ran into the Frostfang Caverns behind me. There are wolves in there ... Can you please go save her?" | `1404846525` | Rescue the dog Lailu. Companions: `2465659416`, `159291288`, `3805713460`, `2514079322`, thanks `146061716` and `2466414509`. Turn-in "Return to Wintara outside of Frostfang Caverns" (`loc 3672144686`). | [client] |
| "Can you please help us? Our dog, Rocko wandered off into the Frostfang Caverns and we are very worried." | `1221974686` | Find Rocko. Companions: `3716534023`, `2029530530`, `3791502302` ("Return to Wintara with any news about Rocko."). | [client] |
| "Please hurry! You must find all the pieces of Fastvi's broken weapon in Frostfang Caverns, and then defeat Chillblaine to summon Fastvi's spirit." | `114964436` | Warrior line: weapon pieces, then Chillblaine. Companions: `3914817276`, `1252491346`, `2344779759` ("Fastvi has been summoned, find him outside of Frostfang Caverns"). Lore chain: `687730370`, `201027568`, `367056625`, `681222927`, `3284801864`. | [client] |
| "This tome says that a new warrior will wield Fastvi's weapon ... You must retrieve Fastvi's weapon from Frostfang Caverns and take it to Fastvi's Retreat!" | `3573835553` | Iris' variant. Companion: `2711042187`. Also Sir Whittington's hand-off, `561000951`. | [client] |
| "I heard a beastly wolf lives in Frostfang Caverns. If you could get me his majestic pelt, I would never be cold again!" | `3189947780` | Get Chillblaine's pelt. Companions: `3182230301`, `2993673504`, `2931360` ("Knock out Chillblaine so you can get his furry coat."), turn-in `4006962732`. | [client] |
| "Would you mind traveling into Frostfang Caverns and retrieving my lost supplies?" | `1226500124` | Fetch quest. The text mentions "my brother". | [client] |
| "Go to the Frostfang Caverns and defeat 12 Snowfoot Ninjas and maybe I will help you out with your training." | `1089531341` | Ninja training quest. | [client] |
| "Head to Snowhill and defeat Wild Fang, a nasty wolf that prowls outside of Frostfang Caverns!" | `1538599502` | Overworld wolf quest. Companions: `1632165060`, `974912794`, `1040480777`, `3269322479`, `3883191566`, `1475846110`. Not a dungeon quest. | [client] |
| "Rescue the trapped villagers from the Howling Hills and return to Fastvi Frostflutter outside of Frostfang Caverns when you are done." | `4099574863` | Return point for the [[Howling Hills]] warrior line. | [client] |
| "Complete any of the Snowhill battles by defeating the bosses of Hot Springs Haven, The Bat Cave, Deep Mines, Howling Hills, Frostfang Caverns, or Snowy Canyon." | `3121919529` | Any-of-six Snowhill battle quest. Companions: `3611018622`, `2926552261`. | [client] |
| "Complete the Frostfang Caverns encounter." / "Win Combat Encounter Frostfang Caverns" | `773420663`, `3125540002` | Generic objectives used by quests and achievements | [client] |

NPCs: [[Chillblaine]], [[Teek]], [[Wintara]], [[Fastvi Frostflutter]], [[Iris]], [[Sir Whittington]]. Zone: [[NPCs - Snowhill]].

## Conflicts

1. **Main goal.** The wiki gives "Investigate the cavern!" as the main goal. The client has that text, and also "Defeat Chillblaine" and "Enter Frostfang Caverns and defeat Chillblaine." as objective text. The real panel wording is unknown. The wiki's is used because it is the only one tied to the dungeon.
2. **Crawler names.** The wiki says "Ice Crawlers". The client says "Icy Crawler". Client wins.
3. **Bat detail text.** The bonus goal text matches client and wiki. The longer rumor line is only linked by the word "cavern". It could belong to a different cave such as [[The Bat Cave|The Bat Cave!]].
4. **Members bonus.** Wiki 220 Coins. There is no client "Bonus: 220 Coins!" string, so it is wiki-only. The client does hold "Bonus: 97 Coins!" (`loc 840806084`), but 97 is this dungeon's wheel coin slot, so the match is probably a coincidence. Not used as evidence.
5. **Star cap.** The wiki gives 242 here and also 242 on [[Sweetwater Climb]]. It may be a copy error. Not verified.
6. **Item spelling.** The wiki has stray trailing spaces on several names (for example "Archer's Power Shard of Regeneration IV "). The client spellings match once trimmed. No other differences.

## Gaps

- Group size and the party size behind "full group".
- The exact goal wording and goal count on the dungeon panel.
- Star payout, coin payout and members bonus in a real run. No result screen was found in video.
- The enemy roster. The wiki's four types are a list, and the "Frostfang" client names are not placed.
- Chillblaine's abilities and health, the cave-in event, and how the "child" lure works in play.
- Where the giant bat is and whether it can be found in any order.
- Which quest owns each description above (titles), and their rewards.
- Whether Teek is the child in the panel text.
