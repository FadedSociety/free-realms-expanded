---
type: content-dungeon
zone: Snowhill
combat: true
---

# Hot Springs Haven

Part of [[Content]] › [[Dungeons]]. Tags: **[client]** = game locale text (`loc <id>`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

## Overview

| Fact | Value | Sources |
|---|---|---|
| Client name | "Hot Springs Haven" (game data `loc 626948345`, UI `loc 1022968102`, `3124272662`) | [client] |
| Panel description | "Umari, the yeti chief, took over Hot Springs Haven! Residents can no longer enjoy their favorite place of leisure! Head in and defeat Umari!" (`loc 1785262435`) | [client] [wiki] |
| Type | Cave (ice cave with hot pools), difficulty 1 | [wiki] [video QHfkcL2_pfg/00036 to 00068] |
| Entrance | Click the barrier in **Snowhill**, up in the hills north west of the town. On the Snowhill map the Atlas pin is next to a "Crystal Barrier" label with "Press to Teleport..." and a "Teleport Now" button. In the world the interact prompt reads "Hot Springs Haven" with "Press X or mouse". The client says "Head to the Hot Springs Haven, up in Snowhill" and "Just head north west until you reach the Hot Springs Haven". Quest text: "Find the entrance to the Hot Springs Haven." (`loc 1979671850`). | [wiki] [client `loc 484739003`, `2945581549`] [video gEi7O6pbnQQ/00002, _a9VF878icg/00002, v1bVCKWiSvw/00004, QHfkcL2_pfg/00018, 2fhn8kePbKQ/00774] |
| Story hook | Yetis moved into the springs and play loud music from jam boxes. The client's quest blurb: "The Yetis are making WAY too much noise. Destroy all of their Jam Boxes and do your part to stop noise pollution." (`loc 3603151827`). Another sign text: "Umari's tribe now owns these cavern springs!" (`loc 723923456`). | [client] |
| Promo string | "Battle in Hot Springs Haven for a bonus!" (`loc 843663247`), a promotion string. Its window is unknown. | [client] |
| Campaign text (unproven as a gate) | The client has a string "Campaign 5: Hot Springs Haven" (`loc 1116188729`, `3270524944`) with story text that ends here. A video campaign-select screen lists it fifth: Taverns and Troves, Haunted Mines, Change of Heart, Lakeshore Shenanigans, Hot Springs Haven, Tanglewood Fort, The Queen's Last Hope, Boss Battles. Campaigns 3 to 7 show a lock and "SC 500". That screen sits inside the Treasure War card game (a campaign map of the card game with ten numbered nodes), and the client says the card game has a "campaign mode" (`loc 3567208383`). So this is most likely a Treasure War card campaign, **not** a gate or mode of this Atlas dungeon. Unproven either way. Do not build it as a gate. | [client] [video JYoSoRzyLrI/00042, 00156, 00618] (inference) |

Campaign story text (unproven as dungeon content; may be Treasure War card-campaign flavour), in order: a penguin postman gives directions (`loc 1100172034`, `1503021259`), the postman attacks (`812100600`), a Frostfang wolf (`347552360`), the alpha wolf on the path up (`3323126154`), then "You stand before the entrance to Hot Springs Haven. There's a Yeti inside who keeps pelting you with snowballs! You enter the haven to teach him a lesson!" (`3518130846`). [client]

## Requirements

| Requirement | Value | Sources |
|---|---|---|
| Combat job level | **None.** Difficulty 1. | [wiki: Dungeons] [client: no level string tied to it] |
| Seen playing at | Archer 14 | [video QHfkcL2_pfg/00018 to 00172] |
| Members only | No. Members get a coin bonus on the results screen (wiki: 91 Coins). | [wiki] |
| Unlocking quest | None. Many quests send players in (see Related quests). | [client] |
| Group size | Unknown. Achievement text "Complete the Hot Springs Haven encounter with a full group." (`loc 2130344966`) and "Group: Hot Springs Haven" (`loc 2960801722`) exist. | [client] |

## Goals

| Goal | Exact text | Sources |
|---|---|---|
| Main | **"Defeat Umari, the Yeti Chief."** (`loc 3528262119`). Shorter forms: "Defeat Umari." (`1608279822`), "Defeat Umari in the Hot Springs Haven" (`2060027127`). | [client] [video QHfkcL2_pfg/00054 to 00160] [wiki] |
| Bonus | **"Bonus: Destroy all of the yeti jam boxes!"** with a counter 0/9 to 9/9. Client strings: "Destroy all of the yeti jam boxes!" (`loc 2906785817`, `519092191`). Counter values seen: 0/9, 1/9, 2/9, 4/9. | [client] [video QHfkcL2_pfg/00054, 00114, 00156, 00160] [wiki: "(9)"] |
| Panel shows | "Goals (2)" | [video QHfkcL2_pfg/00036, 00054] |
| Internal groups | The client has **no** "Hot Springs Haven Bonus Objective" string, although the bonus goal clearly exists in video. | [client: no match] [video] |

## Enemies and boss

| Role | Who | Notes | Sources |
|---|---|---|---|
| Boss | **Umari**, the Yeti Chief (`loc 1749732162`, `2861189576`) | Large yeti with a red health bar and name plate. Speech in chat: "I do so love to dance!" and "Hey! You're the one ruining all our fun! Get out!". Archer 14 saw damage numbers of -1,566 and -4,166 in the fight. | [client] [video QHfkcL2_pfg/00152, 00154, 00156, 00160, 00062] [wiki] |
| Second boss | **Champion Pengubowler** (`loc 3022145963`, `3127401949`) | The wiki lists him beside Umari under Bosses. The client has separate quest text about him "in Hot Springs Haven" that uses penguins as bowling pins: "Defeat the Champion Pengubowler" (`1047427673`), "Enter Hot Springs Haven and defeat the Champion Pengubowler." (`3825132825`). He is not named in the dungeon's goal text. It is not shown whether he is a mini-boss in the same arena or a quest-only fight. | [wiki] [client] |
| Enemies | **Yeti Bouncer** (`loc 3369220996`), Yeti Chucker, Yeti Lounger, Ice Crawler | Client objective strings: "Defeat Yeti Bouncers" (`3420115971`), "Defeat Yeti Chuckers" (`2965023077`), "Defeat Yeti Loungers" (`1199735562`). Video confirms the Yeti Bouncer nameplate. Ice Crawlers are wiki only. | [client] [video QHfkcL2_pfg/00054] [wiki] |
| Friendly NPCs | Penguin Pins | Wiki lists "Penguin Pins" under NPCs. The client has an item/object "Penguin Pin" (`loc 723016342`, `3997534045`) and the quest "Rescue the trapped penguins from within Hot Springs Haven!" (`loc 3310085444`). | [wiki] [client] |
| Bonus targets | **Yeti Jam Boxes** (9) | Objects named "Jam Box" (`loc 2086531761`, `4223365191`), "Yeti BoomBox" (`1338933260`), "Umari's Boom Box" (`457516918`, `4157815384`). | [client] [video QHfkcL2_pfg/00156] |
| Hazards and abilities (client dev strings) | "hot springs haven - umaris boombox" (`659148470`), "Hot Spring Caverns - Umari Ice Nova" (`2338333699`), "hot springs haven - elite flavor attack" (`1995934666`), "Hot Spring Caverns - Boombox icicles" (`98493172`, `767173397`), "Umari Trigger Cube" (`3867910541`, `4009180098`) | Names only. Effects unknown. A fire burst hazard appears in video with the warning "You're heating up! Get out of ..." (cut off). The "Sp?der Tunnel?" nameplate in one frame is unclear and not used. | [client] [video QHfkcL2_pfg/00044, 00062, 00064] |

## Rewards

| Reward | Detail | Sources |
|---|---|---|
| Stars | "220 Stars or less" (wiki wording) | [wiki] |
| Coins on the prize wheel | **49 Coins** slot on every job's wheel | [wiki] |
| Members-only bonus | **91 Coins** | [wiki] |
| Results screen | "YOU WIN!" with the dungeon name and a six-slot reward wheel, "Click here!". Timer at result 02:52:59. The Archer 14 wheel shows "Archer's Ring of Energy I" in the centre, which matches the wiki list. | [video QHfkcL2_pfg/00172] |
| Bonus-goal clothing | Completing the bonus goal gives every player in the instance one piece of clothing from the set for their job, random colour and body slot, matched to the current job. | [wiki] |
| Achievements | "Complete the Hot Springs Haven encounter." (`loc 2235295654`), the full-group version (`2130344966`), "Win Combat Encounter Hot Springs Haven" (`3508421084`). Points are not in the locale. | [client] |

### Prize wheel by job

Every wheel also has the 49 Coins slot. Every item name exists in the client locale except the Brawler gloves, whose client name is "Saved by the Bell Brawler Gloves". [wiki] [client names]

| Job | Items | Sources |
|---|---|---|
| Archer | Archer's Bow of Blizzards, Archer's Power Shard of Vitality I, Archer's Ring of Energy I, Battle Item Mystery Pack, Hen Feather Archer Gloves | [wiki] [client names] [video QHfkcL2_pfg/00172 (ring)] |
| Brawler | Battle Item Mystery Pack, Brawler's Mallet of Rumbling, Brawler's Power Shard of Vitality I, Brawler's Ring of Energy I, Saved by the Bell Brawler Gloves | [wiki] [client names] |
| Medic | Battle Item Mystery Pack, Medic's Power Shard of Vitality I, Medic's Reflex Hammer of Vitals, Medic's Ring of Energy I, Stable Condition Medic Gloves | [wiki] [client names] |
| Ninja | Battle Item Mystery Pack, Kusa Ninja Armwraps, Ninja's Training Sword of 1000 Storms, Ninja's Power Shard of Vitality I, Ninja's Ring of Energy I | [wiki] [client names] |
| Warrior | Battle Item Mystery Pack, Standard Action Warrior Gauntlets, Warrior's Cudgel of Cleaving, Warrior's Power Shard of Vitality I, Warrior's Ring of Energy I | [wiki] [client names] |
| Wizard | Battle Item Mystery Pack, Novice Wizard Gloves, Wizard's Power Shard of Vitality I, Wizard's Ring of Energy I, Wizard's Sparkle Twig of Glaciers | [wiki] [client names] |

Bonus clothing sets by job: Archer Hen Feather, Brawler Saved by the Bell, Medic Stable Condition, Ninja Kusa, Warrior Standard Action, Wizard Novice. [wiki]

## Related quests and NPCs

Quest titles are not stored beside descriptions in the locale, so each row gives the description id. Givers are only those named in the text.

| Quest text (client) | Id | What it asks | Sources |
|---|---|---|---|
| "Glad to see you could join us for our warrior initiation rites. The plan is to start things off in the Hot Springs Haven. Those dirty Yeti Bouncers are really stinking up the hot springs here ..." | `682434084` | Warrior job initiation by **Morninglory**. Companions: "Find Morninglory near the Hot Springs Haven" (`3712772048`), "Equip your warrior gear and complete the warrior initiation rites at the Hot Springs Haven in Snowhill." (`3713487538`), "Now that you have some experience, you're ready for initiation! Go see Morninglory." (`3491114034`), "You should really find Morninglory ... head north west until you reach the Hot Springs Haven" (`2945581549`), reminder "You're a bit forgetful ... put a stop to the unruly activities of those Yeti Bouncers" (`16743865`), objective "Defeat Yeti Bouncers in the Hot Springs Haven" (`4058861570`) | [client] |
| "Murphy says the yetis in Hot Springs Haven are making a lot of noise ..." | `472821565` | Defeat yetis. Objective "Defeat Yetis in Hot Springs Haven" (`891345621`, `681350824`). Related: "Defeat noisy yetis inside of the Hot Springs Haven in Snowhill." (`3041353926`). | [client] |
| "Could you sneak into the Hot Springs Haven and turn down that radio? You will want to defeat the yetis' boss, Umari, as well so he doesn't turn the music back up." | `189369242` | Defeat Umari. Also "Go to Hot Springs Haven, in Snowhill and beat up Umari." (`3516239961`), "Rid the Hot Springs of unwanted yeti by defeating the Chief. Try appeasing him with groovy BoomBox tunes!" (`2107374452`) | [client] |
| "That yeti rock music is scaring away customers! Go up to the Hot Springs Haven and break up the party!" | `4023897901` | Defeat yetis, then return | [client] |
| "There's a yeti in Hot Springs Haven that has kidnapped a bunch of little penguins ... bring down this Champion Pengubowler" | `3765548120` | Penguin rescue. Companions: `758097602`, `4269087891`, `3825132825`, `1047427673`, `1099743950`. | [client] |
| "Escort Marcus through the Hot Springs Haven so he can turn off their stolen cable television service." | `2818707766` | Escort quest. Objective `3110674979`. | [client] |
| "You look strong enough to unfreeze Yaren! ... He's somewhere in the Hot Springs Haven." | `2364570686` | Free Yaren from his icy prison (`2982543704`). Related: `570670804`, `1596215678`. | [client] |
| "Oh my! I seem to be out of mineral water ... go into the Hot Springs Haven and fetch me some? If you melt a few of the ice blocks ..." | `1698456989` | Retrieve mineral water. Companions: `3742073126`, `3674101205`, `4135848408`. Giver not named. | [client] |
| "Can you take these extra-strength cleaning tablets and drop them in the yeti pools?" | `2742856147` | Drop tablets in yeti pools. Companions: `272372227`, `2888200292`. Giver not named. | [client] |
| "I believe the Shadow Talon ninjas are behind these kidnappings ... Go to Hot Springs Haven and defeat the denizens within." | `3857679096` | Ninja training quest. Also "Defeat your foes at the Hot Springs Haven in Snowhill as a Ninja." (`3397682061`). | [client] |
| "Complete any of the Snowhill battles by defeating the bosses of Hot Springs Haven, The Bat Cave, Deep Mines, Howling Hills, Frostfang Caverns, or Snowy Canyon." | `3121919529` | Generic Snowhill battle quest. Objective `2926552261`, blurb `3611018622`. This dungeon is one of six that count. See [[The Bat Cave]], [[Deep Mines]], [[Howling Hills]], [[Frostfang Caverns]], [[Snowy Canyon]]. | [client] |
| "Defeat your foes at the Hot Springs Haven in Snowhill." / "Win Combat Encounter Hot Springs Haven" | `2486182583`, `3508421084` | Generic objectives | [client] |
| Tyrone's trick deck | `3657285132` | "Be the first to score 12 points ... and a Hot Springs Haven trick card from his deck." Card game reward, not a dungeon drop. | [client] |

Video also shows **Bruno Battlerock** standing at the cave entrance as a "Blacksmith: Smithy Merchandise" step (buy Thorny Fasteners and Wooden Planks) with Adventurer 6. [video 2fhn8kePbKQ/00774] A quest line mentions Victor "near Hot Springs Haven, in a path behind the Hot Springs" (`loc 2484432254`). [client]

NPCs: [[Umari]], [[Champion Pengubowler]], [[Morninglory]], [[Bruno Battlerock]]. Zone: [[NPCs - Snowhill]]. Job: [[Warrior]].

## Conflicts

1. **Brawler Gloves name.** The wiki wheel says "Bell Brawler Gloves". The client item is "Saved by the Bell Brawler Gloves" (`loc 3699014784`). The client name is used above.
2. **Bonus objective record.** The wiki and video show a bonus goal with a 0/9 counter, but the client has no "Hot Springs Haven Bonus Objective" string. The index lists this dungeon among those with none. The video wins.
3. **Campaign meaning.** The client's "Campaign 5: Hot Springs Haven" was first read as a dungeon campaign. The only video of the campaign list is a Treasure War card-game campaign map, so the page now treats it as unproven and not a dungeon gate (see Overview). Unresolved without a client mapping of the campaign feature.
4. **Second boss.** The wiki lists Champion Pengubowler as a boss, but the goal panel in video names only Umari. Not resolved.

## Gaps

- The star payout in video (only the wiki's "220 Stars or less") and the real coin figure on the results screen.
- Where the 9 jam boxes stand and whether they are required to open the boss.
- Whether Champion Pengubowler is inside this arena, and how the penguin rescue works.
- What the Ice Nova, trigger cube and "elite flavor attack" do.
- Group size, and the full result screen of a members run.
