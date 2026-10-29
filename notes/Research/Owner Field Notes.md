# Owner Field Notes

Observations the owner gathers from original-game videos and screenshots. They are first-hand descriptions of the live game and serve as `[video]` evidence. Add new notes at the end, dated. Where a note names an NPC or place, check it against the regional NPC pages and Legacy spawns before turning it into content.

## 2026-10-14

### Dialogue and camera
- Talking to an NPC zooms the camera onto that NPC, who fills the view. The dialogue or quest window sits to the left of the NPC's body (reference: MMODEN screenshot of Merigold's "Introduce Yourself" offer).
  - Implemented: the dialog camera target is the NPC GUID in 26/3 dwords 2–3, after the reward bundle in 49/1, and at offset 6 in 49/13. Verified natively in run 233813. See [[Native Round5 050310]].

### Overhead markers
- Arrows at the top of the screen point to enemies, NPCs and resources, with a name and a distance in metres (e.g. "Queensfields Bushel 20m").
- Colour by type:
  - red: enemy encounters
  - green: NPCs (specifics unknown)
  - pink: mining deposits and probably other resources
- Active quests show a green arrow reading "Active Quest (Quest Name)".
  - Client: these are `ScreenDirectionIndicatorDataSource` rows (Text, Subtext, IconId, ScreenAngle, Distance, Color, CharacterGuid, IsPc). They point at spawned actors, so they hold no location data of their own.

### NPC quest icons
- An NPC with a quest for you has an exclamation-mark icon over their head.
- A winged exclamation mark probably marks an important quest or a class (job) quest.

### Enemies and encounters
- Overworld enemies carry a red circular label with two crossed swords. Clicking one opens a battle menu where you choose the difficulty (example: Frostfang Snarler).
  - Research (2026-10-14): each battle has one fixed difficulty (wiki "Battles": 1–5; 3/4/5 need a level 5/9/13 combat job, locale 4199–4201). Separate battles are separate spawners, e.g. Legacy "Hooligan Brawling Club!" 4156 and "Hooligan Bullies!" 4162 (model `sg_spawner_human_01`). The start screen comes from 41/114 details; Play sends 41/108.
- There are also ordinary overworld enemies. Specific ones, such as the Frostfang Snarler, push you into an instance (probably a dungeon) to fight them.
- The Troll Summoner looks like an instance enemy.

### Lakeshore and the goblin and troll route
- **Len Tyril** stands in front of a bridge at Lakeshore.
- **Grosh** stands in front of the hut at the goblin camp after the bridge.
- The second encampment after the first has another NPC (name unreadable) whose quest has the winged exclamation icon, probably a class quest.
- Along the trail are overworld trolls that do not seem hostile: static enemies walking the trail.
- At the end, by the Forest Troll Fort:
  - **Vergosh**, a static troll NPC walking back and forth.
  - **Laggosh**, an NPC in front of the dungeon.

### Combat numbers (video)
- A level 3 Medic fighting **Hooligan Bullies** took 83 damage per hit (124 on a critical) and dealt 127 per hit.
  - Use as the first balance reference for overworld hooligans. Our reconstructed Hooligans (`field_enemies.json`) currently use 220 HP / 8 damage, and the opening wolves 180/25; player HP is 1000 in our reconstruction. Check the original player HP scale before matching the absolute numbers.

### Enemy speech
- Enemies show a black text box (speech bubble) over their heads. Example: a **Rowdy Hooligan** says "Hey guys, look over here! We have guests!"
  - Locale: line hash 1899870574; name "Rowdy Hooligan" hashes 833150515 (ucdt) and 2034994574 (ugdt). Probably an aggro bark when they notice the player.
- A **Hooligan Bully** (the model we show in game) says "What are you looking at? Get out of here!" in a **white** text box, and also "Let's see what they brought to the party!"
  - Locale: 3270315144 "What are you looking at? Get out of here!" and 3640848319 "Lets see what they brought to the party!" (numeric ids in the 2026-10-15 entry below).

### Encounter start screen ("Hooligan Bullies!")
(Reference frame from a gameplay video, kept locally; not redistributed.)
- Title "Hooligan Bullies!" (the spawner's NameId 74753). **Difficulty: 5 red dots**, "Requires a level 13 combat job" (locale 4201).
- Description: "Hooligans are bullying everyone in sight! Go in and knock some sense into them!" (hash 1428427418).
- **Prizes**: "Win the game and you could win one of these prizes!" Three items: Ninja's Power Shard of Second Wind, Ninja's Power Shard of Regeneration V, Ninja's Shadow Blade of Soul Power. The prizes seem to follow the player's job (a Ninja here).
- A **Go!** button bottom right and a close X top right. A side panel advertises a marketplace item (Prospector's Scroll, 50 SC, BUY!).
- An encounter timer (00:20:03) shows at the top of the HUD.
  - Client: this is `MinigameStartScreen`, opened by 41/114 details; Go sends 41/108 (research 2026-10-14).

## 2026-10-15
- Numeric locale ids for the Hooligan Bullies battle, from `lab/reports/locale-id-map.json`:
  - 74752: "Rowdy Hooligan" (enemy name)
  - 74753: "Hooligan Bullies!" (spawner and screen title)
  - 74755: the description
  - barks: 141234 "Hey guys, look over here! We have guests!", 141235 "Lets see what they brought to the party!", 76405 "What are you looking at? Get out of here!"
- Hooligan Bullies goal text: "Stand up to the bullies!" = locale **74758**, completing the 74752–74758 cluster (enemy name, title, description, goal).
- More bully flavor text: "I can bench 10 of you."
- **The owner's model of combat (2026-10-15):**
  - Combat instances: walk up to the enemy or spawner, a menu opens, you must be on a combat job (only combat jobs are allowed), pick the difficulty, then jump in and fight.
  - Free overworld enemies probably exist **only for quests**: they spawn when the quest is active and only while it is.
  - The opening already works this way: its wolves are stage-gated. Plan: field enemies get quest gating (spawn while one of their quests is active).
- **Correction (The owner, 2026-10-15):** not all enemies are quest-gated. Spawner battles (e.g. the Troll Summoner) stay in the world: a level-20 player passed one and it was still available.
  - Entry is level-gated: some battles need a minimum combat-job level before you can attempt them ("Requires a level 13 combat job" on Hooligan Bullies). The wiki tiers match: 1 open, 2 recommended level 3+, 3/4/5 require a level 5/9/13 combat job.
  - Plan: spawners are always visible; the start screen shows the requirement; the server refuses Go (41/108) unless the active job is a combat job at or above the level. Only quest-specific enemies (e.g. the Hooligan Delinquents for 3030007) use `quest_ids` gating.

### Ninja quest line (video, level 1 onwards)
- **Ty** is the Ninja trainer, next to a lake and a dock (NPC 9000000223; name 5455).
- **Ninja: The Ninja Path** (quest 3110001; title 20960), from Ty.
  - Offer: "Part of me wants to help, but... it's best if I stay uninvolved. Are you willing to fight for the good of others?" (24829).
  - Description: "Trolls are kidnapping pixies from Lakeshore. Travel to the Forest Troll Village, northeast of the lake, and see if you can learn more." (20961).
  - Helper step: "Ask around Forest Troll Village for information" (93206). A "Take me There" button is shown.
  - Turn-in at **Grosh** (9000000335; name 20955), in front of the hut at the first goblin/troll camp after the bridge: "You look for kidnap pixies? Yes, Queen be kidnapping pixies. Me not know why. Maybe she want eat them? Me not know." (9731), then "Thanks!". The end card showed a "6" (probably a reward amount).
- **Ninja: Stolen Away** (quest 3110002; title 78925), from Grosh. Goal: "Defeat the Troll Summoner".
  - It is fought as the battle **"Troll Summoner Madness!"** (42056). Instance goal "Defeat the troll summoner!" (42071; also 42081 "...to complete the encounter"; journal 115023 / 140307). Enemy name "Troll Summoner" (15623).
- **Quest icons**: "!" over a giver; a winged "!" for important or class quests; **"?" over the NPC you hand a quest in to**, when another NPC sent you.
- **Battle prizes**: on winning, the prize items spin in a cycle like a loot box (CS:GO style) and land on a random item; each probably has a rarity weight.
- Other names along the route: Len Tyril (76038) at the Lakeshore bridge, Laggosh (20959) in front of the troll dungeon, Vergosh (21081) pacing by the Forest Troll Fort.
- **Area entry banner:** entering a new area shows that area's emblem picture with its name. The owner can send a screenshot of the UI.
  - Screenshot (FreeRealmsPro "Brawler and Ninja training guide part 2", 0:38): "Now Entering..." above a round emblem (Snowhill: a purple mountain) and the large area name "Snowhill", top centre, fading over the world.
  (Reference frame kept locally; not redistributed.)

### Battle flow, corrected (The owner, 2026-10-15)
- **No tent.** The battle marker is the enemy itself: the Troll Summoner model walks around the overworld with the crossed-swords icon over it. Walk up and click it to open the start screen.
- **Battles are instanced.** After Go!, the player is teleported to a separate, closed-off battle location (probably a forest-type area for trolls; dungeon-like), and the fight starts there. It is not fought in the overworld.
- **Troll Summoner behaviour:**
  1. It runs away, then turns around.
  2. After the player attacks it for a while, it swirls its arms (summon animation). A puff of smoke appears and 2–3 small ("midget") troll monsters form, which attack the player.
  3. The player kills those, keeps attacking the summoner, and the cycle repeats until the summoner is beaten.

### Multiplayer: party consent before instancing (The owner, 2026-10-15, design idea)
- This is a design request. It is not confirmed in the original game (R); check the client for a party/group instance prompt (e.g. a group "ready check" or encounter invite) before building our own.
- When party members are grouped and one member starts a dungeon/battle, the others must **not** be pulled in automatically.
- Each other member gets a request: "<member> wants to enter <dungeon>. Join?" with **Accept / Decline** (a queue-up / ready-check style).
- Accepting sends that member into the instance. Declining leaves them where they are, **still in the party**.
- Test this when the multiplayer work starts (two clients seeing each other, partied).

### Troll Summoner AI and combat abilities (The owner, 2026-10-15, design + reminders)
- **Summoner behaviour (design, R):** while its summoned trolls are alive, the summoner gangs up on the player alongside them. Once they are dead it runs away, stops to catch its breath (a rest cycle after running too long), then runs again.
  - Lead's refinements (agreed in chat): the rest is the damage window (com_stunned, no attacks); its run speed is slower than the player's; summons still trigger at its health thresholds.
- **Play-test feedback (run 040443):** the troll looped one animation and never ran; the other trolls were invisible or "phasing in and out of reality" while still hitting; the hit reach was huge (about four troll lengths); clicking the summoner never attacked it.
  - Causes found: a missing animation id, an invented ranged attack, and a fixed height that sank runners under slopes.
- **The summoned trolls are small ("midget") trolls,** not full-size ones.
- **Jobs have combat abilities.** A Medic heals itself on cooldown between abilities and survives a fight like this by playing smart and strafing. Balance must assume abilities, not only the basic attack. Our server implements only the basic slot so far (Build Ledger: combat authority, one ability family at a time).

### Job skills are level-locked (The owner, 2026-10-15)
- Each job skill unlocks at a job level. Medic: **First Aid at level 1** (the core heal), **Target Vitals at 5**, **Vitamins at 10**, **Shock Paddles at 15**. This matches the Medic job guide and the client's ability text.
- Weapon abilities (Incision, Triage on the Student Medic Hammer) come with the weapon, not the job level.
- Build rule: every job skill in `data/content/job_abilities.json` carries a minimum job level, enforced against the player's level for that job before the skill goes on the bar.

### Original launcher look (The owner, 2026-10-16)
- **Animated title:** the original Free Realms launcher's "Free Realms" wordmark was animated, with vines flowing out of it.
- **Frames with raised edges:** UI parts were "3D but not completely 3D": frames wrapping text had a raised, bumpy rim ("braille bump style") around their edges.
- Not in the client files: the client's own `Free_Realms_Logo.dds` is the plain orange wordmark with a glow, and the launcher (SOE LaunchPad) was a separate download. No screenshot found online yet (2026-10-16 search), so this note is the evidence. Lead's reading of "braille bump" is (R) until the owner confirms it.

### Combat power-ups (The owner, 2026-10-16)
- **Enemies drop power-ups during combat**, by chance per kill. The owner's screenshot of the official in-game help (via the wiki): you use one with the **number 3 key** and can **hold only one at a time**, so you use it to make room for the next.
- The five kinds:
  - **Health:** the big heart heals everyone in your group; the little heart restores a small amount of your own health.
  - **Energy:** restores energy.
  - **Flame Wave:** instantly damages all enemies around you.
  - **Earth Shard:** damages all enemies in your area and stuns them briefly.
  - **Super Shield:** raises your speed and makes you invulnerable for a short time.
- Client evidence so far (lead, 2026-10-16): the locale has "Flame Wave Power Up", "earth shards power up buff - tier 4" and "spawn healt powerup npc", which suggests a power-up spawns as a pickup actor that grants a buff. Drop chances were not found yet. If they are server-side only, we reconstruct a chance per kind and mark it (R).
- **Drop model (Owner decision, 2026-10-16):** every player who took part in a kill gets their own drop, which only they can see. Nobody sees or can take another player's drops, so pickups are never shared or contested. The server still tracks each player's pickups (the client is never trusted to create one).
  - Lead refinements: "took part" means dealt damage or was engaged in that fight, not bystanders.
  - Chances stay tunable for group play.
  - The big heart's group heal must not stack absurdly.
  - The original game's behaviour is unrecorded, so this is (R).
