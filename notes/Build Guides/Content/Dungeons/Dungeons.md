---
type: index
---

# Dungeons

Part of [[Content]]. This is the index of every combat minigame in the fan wiki category "Combat Mini-game" (72 entries): Atlas dungeons, quest and node battles, and two stray pages. Each entry with a page links to it. Pages not written yet are linked anyway and marked **pending**.

Tags used on every dungeon page: **[client]** = the game's own locale text (`cq.py loc <text>`; ids are the first column of `en_us_data.dat`), **[video]** = player recordings (frame key), **[wiki]** = fan wiki. Precedence client > video > wiki.

## Page status

- **Written (all 70 combat entries):** the 10 detailed pages first written ([[Bigfoot's Lair]], [[Bixie Hive]], [[Bandit Hideout|Bandit Hideout!]], [[Tavern Cellar]], [[Briarheart Caverns]], [[Croaking Vale]], [[Hewey's Escape]], [[Cracked Claw Caverns]], [[Robgoblin Treasure Trove]], [[Bone Bog Cemetery]]), then dungeon batches 1–3 and the 27 battles (difficulty 1–5). Every row below links to its page.
- **Not pages:** Battle Item Mystery Pack and Cakenstein are listed for completeness but are not dungeons.
- **Page names:** use the wiki title without a trailing "!", so "Bandit Hideout!" is `Bandit Hideout.md`. Link it as `[[Bandit Hideout|Bandit Hideout!]]` when the shown title matters.
- **Layout of each page:** Overview, Requirements, Goals, Enemies and boss, Rewards (with a prize wheel table per job), Related quests and NPCs, Conflicts, Gaps. Later pages should copy it.

## The ten written pages at a glance

Figures are from the **wiki** page of each dungeon unless a source is named. Exact goal text, corrections and sources are on each page.

| Page | Zone | Diff | Boss | Bonus goal | Wheel coins | Stars cap | Members bonus |
|---|---|---|---|---|---|---|---|
| [[Hewey's Escape]] (removed) | Highroad Junction | 1 | Hewey the Bear | none | 36 | 10 | none listed |
| [[Cracked Claw Caverns]] | Blackspore Swamp | 1 | Cracked Claw | release trapped spirits (wiki 6, video 5) | 26 | 58 | 77 |
| [[Bixie Hive]] | Sanctuary (Queensfields) | 1 | Drone Fauzz | save 4 frightened workers | 68 | 228 | 126 |
| [[Tavern Cellar]] | Wilds (Roadhouse) | 1 | Cal | destroy 4 brawl club rules | 49 | 216 | 88 |
| [[Robgoblin Treasure Trove]] | Wilds (Robgoblin Camp) | 2 | Wizard Hildegarde | find the spellbook | 49 | 98 (video 103) | 105 (video agrees) |
| [[Bone Bog Cemetery]] | Blackspore Swamp | 3 | Banshee (Pumpkin Prince in October) | defeat 3 skeletons | 112 | 306 | 180 |
| [[Bigfoot's Lair]] (battle) | Wilds (Camp Chatty) | 3 | Bigfoot, really Robbie | none | 990 | 102 | none listed |
| [[Bandit Hideout\|Bandit Hideout!]] | Wilds (Greenwood) | 4 | Muggenstomp | defeat 5 Big Bandits | 60 | 210 | 209 |
| [[Croaking Vale]] | Briarwood (Nettleseed) | 4 | Smuggler Han and Smuggler Hannah | free 5 captured frogs | 202 | 304 | 308 (video 583) |
| [[Briarheart Caverns]] | Briarwood (Briarheart) | 5 | Geomancer Hailstorm | "Defeat all enemies!" (wiki only) | 117 | 311 | 281 |

Patterns seen across them, which later pages should check against:

- **Gear tier follows difficulty.** Prize-wheel gear carries a numeral I to V that matches the difficulty: Bixie Hive, Tavern Cellar and Cracked Claw "I"; Robgoblin Treasure Trove "II"; Bone Bog Cemetery "III"; Bandit Hideout and Croaking Vale "IV"; Briarheart Caverns "V". The bonus-clothing set steps up too (Hen Feather and similar at 1, Roving at 2, Cross Wind at 3, Bobtailed or Bodkin at 4, Bull's-Eye or Pin Hole at 5). [wiki] [client names verified]
- **Goal panels.** Among these ten, eight show "Goals (2)": one main goal and one bonus goal. [[Hewey's Escape]] and [[Bigfoot's Lair]] show "Goals (1)". [video]
- **Prize wheel.** One wheel per combat job: 5 items (6 at difficulty 5) plus a Coins slot. "Battle Item Mystery Pack" is on every wheel except Hewey's Escape and Bigfoot's Lair. [wiki]

## Shared rules (apply to every entry)

| Rule | Detail | Sources |
|---|---|---|
| Difficulty tiers | 1 to 5, easiest to hardest. The tier sets the entry requirement: 1 anyone; 2 "recommended levels 3 and higher, but can be entered by anyone"; 3 combat job level 5+; 4 level 9+; 5 level 13+. | [wiki: Dungeons, Battles] |
| Client requirement strings | One panel line per tier: "Requires a level 5 combat job" (`loc 2145075890`), "Requires a level 9 combat job" (`loc 1291047686`), "Requires a level 13 combat job" (`loc 1323029355`). Matching refusal messages: "You must have a combat job of at least level 5 / 9 / 13 to play this battle" (`loc 3173661191`, `1882811692`, `225515096`) and invitee versions "[*invited_player*] must have a combat job of at least level N to play this battle" (`loc 66524078`, `3240997533`, `2162810712`). There is no string for level 3, so tier 2 has no hard gate. | [client] |
| Panel in video | Battle-select window: entry name, "Difficulty:" row, optional "Requires a level N combat job", description, "Prizes / Win the game and you could win one of these prizes!", a store offer ("Daily Grab Bag", "Check back daily for a new Daily Grab Bag!", price 450) and a Go! or PLAY! button. | [video s4q4D_xmPoU/00002, pfgmx-YRZD0/00016, TjpDl22Lwpk/00042] |
| Dungeon vs battle | Dungeons have fixed Atlas icons and entrances (barriers, gates, doors). Battles are transient nodes, NPCs or special spawns and are "not represented on the Atlas". The Atlas overlay offers "Press to Teleport..." to dungeons. | [wiki: Dungeons, Battles] [video GvF2-OD8sY0/00090] |
| Job switch | On entry: "Your job has been switched because you are entering a combat area." | [video 0LoQe5VZ0PY/00034, EP2-qVs1mR8/00004] |
| Goals panel | "Goals (N)". A bonus goal is shown as "Bonus: ..." with a counter "x/N" when it has one. A "Goal Complete!" banner shows as each ticks. After the main goal, an exit marker appears. | [video 5_U-uAzDuR4/00002, 9gfkEIFMV4s/00022, s4q4D_xmPoU/00194] |
| Spawner objects | Some waves come from a destroyable structure with a health bar and the hint "Attack the ... to stop the monsters from coming!": Wolf House ([[Tavern Cellar]]), tunnel ([[Cracked Claw Caverns]]), Bixie Tunnel ([[Bixie Hive]]). | [video fktAs_GRrjM/00634, NtXTmB-wPW0/00078, 5_U-uAzDuR4/00064] [client `loc 2920003869`, `3907530502`] |
| Results screen | "YOU WIN!", then "Score Info" (Opponents or Enemies Defeated, Time Bonus, Knock outs Remaining N/5, Goals, Bonus Goals, Total score), "Your Rewards" (Coins, Stars, items), a prize wheel with a Coins slot, and for members "Members Only Bonus! This special reward is only available to members!" Non-members see "If you were a member, you'd receive this special bonus!" and an Upgrade button. | [video kJpAm8IAtjs/00424, qCG_K2piQgE/00178, CnUr3X24d_M/00170, l-KJnbZShSk/00180] |
| Score weights seen | [[Briarheart Caverns]] and Forest Troll Fort: 3,000 per opponent, 50,000 per remaining knockout, 120,000 for the goal, 25,000 for the bonus goal. [[Cracked Claw Caverns]] uses different weights. "Knock outs Remaining N/5" suggests five knockouts per player. | [video kJpAm8IAtjs/00424, -bXM3-p0eCk/00502, 0egOV5hzmjI/00056] |
| Bonus-goal clothing | Completing the bonus goal grants every player in the instance one piece of clothing from the set for their current job. Colour and body slot are random. Seen in video for a Warrior and a Medic ([[Robgoblin Treasure Trove]]) and a Brawler ([[Bixie Hive]], [[Bandit Hideout]]). | [wiki] [video WuTrZmFvTmI/00032, qCG_K2piQgE/00070, 5_U-uAzDuR4/00166, 9gfkEIFMV4s/00236] |
| Battle Item Mystery Pack | "This prize grants the winner 3 battle items!" | [video qCG_K2piQgE/00176] |
| Trophies | The wiki says the achievement system launched on 14 February 2013. Most dungeons have a bronze trophy (5 pts) "Complete the X encounter." and a silver trophy (10 pts) "Complete the X encounter with a full group." Client strings: "Complete the X encounter.", "Complete the X encounter with a full group.", "Win Combat Encounter X", "Group: X". The full-group size is not stated anywhere found. | [wiki: Achievements] [client] [video TjpDl22Lwpk/00378] |
| Bonus-objective groups | The client has an "X Bonus Objective" string for 37 of the 42 dungeons, and for the battle Snatching Snappers. Dungeons with none: Cursed Graveyard, Hot Springs Haven, Sandscale Oasis, King of the Ring!, Hewey's Escape. None was found for Bigfoot's Lair. Search with `cq.py loc "Bonus Objective"`. | [client] |
| Job collections | Combat-job collections are found inside dungeons, for example Medic Shock Rod Materials in the [[Robgoblin Treasure Trove]]. | [wiki: Dungeons] [video qCG_K2piQgE/00092] |
| "Bonus: N Coins!" strings | The client holds about 100 generic strings "Bonus: N Coins!" (for example 88, 77, 281). None is named for a dungeon, so they do not confirm a dungeon's members-only coin figure. | [client] |

## All 72 entries

The level column is the tier rule above (from the wiki), not a per-page infobox value. Locations are the wiki's.

### Dungeons (Atlas entries, category Dungeon)

| Entry | Diff | Min combat level | Wiki location | Wiki notes | Page |
|---|---|---|---|---|---|
| [[Bixie Hive]] | 1 | none | Queensfields, Sanctuary |  | **written** |
| [[Cracked Claw Caverns]] | 1 | none | Blackspore Swamp |  | **written** |
| [[Cursed Graveyard\|Cursed Graveyard!]] | 1 | none | Stillwater Crossing |  | **written** |
| [[Hewey's Escape]] | 1 | none | Highroad Junction | No longer in-game | **written** |
| [[Hot Springs Haven]] | 1 | none | Snowhill |  | **written** |
| [[King of the Ring\|King of the Ring!]] | 1 | none | Crossroads | No longer in-game | **written** |
| [[Sheep Watch]] | 1 | none | Southern Wilds |  | **written** |
| [[Tavern Cellar]] | 1 | none | Northern Wilds |  | **written** |
| [[Arachnia's Lair]] | 2 | none (3+ recommended) | Bristlewood, Briarwood |  | **written** |
| [[Danger Peaks]] | 2 | none (3+ recommended) | Southern Wilds |  | **written** |
| [[Den of Secrets]] | 2 | none (3+ recommended) | Merry Vale |  | **written** |
| [[Highroad Hijinx]] | 2 | none (3+ recommended) | Southern Wilds |  | **written** |
| [[Robgoblin Treasure Trove]] | 2 | none (3+ recommended) | Northern Wilds |  | **written** |
| [[Sandscale Oasis]] | 2 | none (3+ recommended) | Sunstone Valley |  | **written** |
| [[Sweetwater Climb]] | 2 | none (3+ recommended) | Wugachug |  | **written** |
| [[The Bat Cave\|The Bat Cave!]] | 2 | none (3+ recommended) | Singing Crystal Mines, Snowhill |  | **written** |
| [[Bone Bog Cemetery]] | 3 | 5 | Blackspore Swamp |  | **written** |
| [[Bristlewood Glade]] | 3 | 5 | Bristlewood, Briarwood |  | **written** |
| [[Deep Mines]] | 3 | 5 | Singing Crystal Mines, Snowhill |  | **written** |
| [[Floren Forest]] | 3 | 5 | Northern Wilds |  | **written** |
| [[Forgotten Caves]] | 3 | 5 | Memorial Caverns, Sanctuary |  | **written** |
| [[Howling Hills]] | 3 | 5 | Snowhill |  | **written** |
| [[Misty Mountain]] | 3 | 5 | Shrouded Glade |  | **written** |
| [[Mushroom Cavern]] | 3 | 5 | Nettleseed, Briarwood |  | **written** |
| [[The Snarling Hedges]] | 3 | 5 | Thistlerow, Briarwood |  | **written** |
| [[Bandit Hideout\|Bandit Hideout!]] | 4 | 9 | Southern Wilds |  | **written** |
| [[Briar Patch]] | 4 | 9 | Bristlewood, Briarwood |  | **written** |
| [[Croaking Vale]] | 4 | 9 | Nettleseed, Briarwood |  | **written** |
| [[Darvon's Descent]] | 4 | 9 | Memorial Caverns, Sanctuary |  | **written** |
| [[Frostfang Caverns]] | 4 | 9 | Snowhill |  | **written** |
| [[Grexan's Camp]] | 4 | 9 | Northern Wilds |  | **written** |
| [[Mugwort's Hollow]] | 4 | 9 | Northern Wilds |  | **written** |
| [[Treeleaf's Retreat]] | 4 | 9 | Briarwood |  | **written** |
| [[Vale of the Ancients]] | 4 | 9 | Bristlewood, Briarwood |  | **written** |
| [[Briarheart Caverns]] | 5 | 13 | Briarheart, Briarwood |  | **written** |
| [[Cray Caves]] | 5 | 13 | Seaside |  | **written** |
| [[Dark Spore Depths]] | 5 | 13 | Nettleseed, Briarwood |  | **written** |
| [[Forest Troll Fort]] | 5 | 13 | Northern Wilds |  | **written** |
| [[Haunted Mines]] | 5 | 13 | Blackspore Swamp |  | **written** |
| [[Precursor Ruins]] | 5 | 13 | Sunstone Valley | Not in the Combat Mini-game category; listed on the wiki Dungeons page | **written** |
| [[Snowy Canyon]] | 5 | 13 | Snowhill |  | **written** |
| [[Tanglewood Fort]] | 5 | 13 | Nettleseed, Briarwood |  | **written** |
| [[Trail of Betrayal]] | 5 | 13 | Nettleseed, Briarwood |  | **written** |

### Battles (category Battle: nodes, special spawns and quest battles)

| Entry | Diff | Min combat level | Wiki location | Wiki notes | Page |
|---|---|---|---|---|---|
| [[Frostfang Snarlers\|Frostfang Snarlers!]] | 1 | none | Snowhill |  | **written** |
| [[Snakes in a Maze\|Snakes in a Maze!]] | 1 | none | Briarwood |  | **written** |
| [[Treasure of the Bone Shaman\|Treasure of the Bone Shaman!]] | 1 | none | Blackspore Swamp |  | **written** |
| [[The Rumbledome]] | 1 | none | Rumble Ridge, Sunstone Valley | Arena; Battle category, not Combat Mini-game | **written** |
| [[Feisty Floren\|Feisty Floren!]] | 2 | none (3+ recommended) | Wugachug |  | **written** |
| [[Fleetfoot Ninja\|Fleetfoot Ninja!]] | 2 | none (3+ recommended) | Greenwood Forest |  | **written** |
| [[Ice Troll Scout\|Ice Troll Scout!]] | 2 | none (3+ recommended) | Snowhill |  | **written** |
| [[Petty Yetis\|Petty Yetis!]] | 2 | none (3+ recommended) | Snowhill |  | **written** |
| [[Pixie Hunters\|Pixie Hunters!]] | 2 | none (3+ recommended) | Merry Vale |  | **written** |
| [[Robgoblin Creek\|Robgoblin Creek!]] | 2 | none (3+ recommended) | Wilds |  | **written** |
| [[Thugawug Sneak\|Thugawug Sneak!]] | 2 | none (3+ recommended) | Wilds |  | **written** |
| [[Vampire Dance Practice]] | 2 | none (3+ recommended) | Blackspore Swamp | Special Spawn, Super Spooktacular, Seasonal | **written** |
| [[Werewolf Dance Practice]] | 2 | none (3+ recommended) | Blackspore Swamp | Special Spawn, Super Spooktacular, Seasonal | **written** |
| [[Bigfoot Defender]] | 3 | 5 | Southern Wilds | Special Spawn | **written** |
| [[Bigfoot's Lair]] | 3 | 5 | Southern Wilds | Special Spawn | **written** |
| [[Nettleseed Nibblers\|Nettleseed Nibblers!]] | 3 | 5 | Briarwood |  | **written** |
| [[Return to Sender\|Return to Sender!]] | 3 | 5 | Northern Wilds |  | **written** |
| [[Seaside Swoopers]] | 3 | 5 | Seaside |  | **written** |
| [[Snatching Snappers\|Snatching Snappers!]] | 3 | 5 | Seaside | Special Spawn | **written** |
| [[The Mushroom Gigas\|The Mushroom Gigas!]] | 3 | 5 | Nettleseed, Briarwood | Members Only, Special Spawn | **written** |
| [[Moldy Shamblers\|Moldy Shamblers!]] | 4 | 9 | Nettleseed, Briarwood |  | **written** |
| [[Oasis of Peril\|Oasis of Peril!]] | 4 | 9 | Seaside | Special Spawn | **written** |
| [[Prince of Low Tide\|Prince of Low Tide!]] | 4 | 9 | Seaside | Special Spawn | **written** |
| [[Shady Smugglers\|Shady Smugglers!]] | 4 | 9 | Nettleseed, Briarwood |  | **written** |
| [[Stealthy Despoilers]] | 4 | 9 | Seaside | BrawlWiki | **written** |
| [[Bergram Stumpfinger's Ghost]] | 5 | 13 | Blackspore Swamp | Special Spawn | **written** |
| [[Chugawump\|Chugawump!]] | 5 | 13 | Briarwood |  | **written** |
| [[Mudshell]] | 5 | 13 | Blackspore | Special Spawn | **written** |
| [[Mushroom Mania\|Mushroom Mania!]] | 5 | 13 | Nettleseed, Briarwood |  | **written** |
| [[Necronomicus Unleashed]] | 5 | 13 | Merry Vale | Listed on the wiki Battles page only; Awakened version level 20 | **written** |
| [[Stone Heart's Hideaway]] | 5 | 13 | Merry Vale | Seasonal (Festival of Hearts), members-only quests; listed on the wiki Battles page only | **written** |

### Other entries in the category (not battles)

| Entry | What it is | Page |
|---|---|---|
| Battle Item Mystery Pack | An item page, filed under the category by mistake [wiki] | none needed |
| Cakenstein | An NPC page for the Birthday Bash boss [wiki]. The fight is "Cakenstein Attacks!" (difficulty 5, requires a level 13 combat job) [video bCg4kMUbF0/00002] | pending, as `Cakenstein Attacks` |



## Not in the category

- **Precursor Ruins** is listed by the wiki's "Dungeons" page as difficulty 5, but it is not in the "Combat Mini-game" category and its wiki page describes the ruins as a place. It needs a page of its own. [wiki: Dungeons, Precursor Ruins]
- The wiki "Battles" page lists many more battles than this category holds (for example Bixies Gone Wild!, Hooligan Brawling Club!, Cray Marauders, Brutus the Brute!). They are out of scope for this index unless a later agent adds them. [wiki: Battles]

## Conflicts

1. **Type labels.** The wiki files [[Bigfoot's Lair]] under Battle, not Dungeon, and marks [[Hewey's Escape]] removed. Both still work in the client text and recordings. The ten pages treat them as dungeon-style pages because the brief assigned them.
2. **Category membership.** Two entries in the category are not dungeons (see the "Other entries" table) and one wiki-listed dungeon is outside it (see above).
3. **Wiki infobox level rows.** The wiki shows "Varied" for tiers 1 and 2 and "Levels N or higher" (or "and higher") for tiers 3 to 5. The tier rule above is what the client strings confirm.
4. **Achievement coverage.** The wiki's achievements list has encounter trophies for most dungeons but not for Cursed Graveyard, Sandscale Oasis, Hewey's Escape, Tanglewood Fort, Trail of Betrayal or the battles.

## Gaps

- Group size (how many players make a "full group") is unknown for every entry.
- Star payouts: the wiki writes "N Stars or less", which no video confirms. One recording ([[Robgoblin Treasure Trove]]) shows 103 stars against a wiki value of 98.
- Wave layouts, boss health and ability effects are not known for any entry.
- Every entry now has a page; the gaps above apply across them.
