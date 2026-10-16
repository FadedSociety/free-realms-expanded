---
type: content-job
job: Brawler
category: combat
members_only: false
max_level: 20
---

# Brawler

Part of [[Content]]. Sibling job pages: [[Archer]], [[Medic]], [[Ninja]], [[Warrior]], [[Wizard]].

## How to read this page

- Tags: **[client]** is the game's locale text (`en_us_data.dat`), **[video]** is player recordings, **[wiki]** is the fan wiki. Precedence is client, then video, then wiki. Where they disagree, the higher source is in the table and the clash is in **Conflicts**.
- Numbers in parentheses, for example `(1844179881)`, are client locale ids. The "Title ids" column gives both title entries for a quest as `ucdt` / `ugdt`; the two tags carry the same text.
- Video frame keys look like `(aveqYa4AVk8/00000)`.
- **Matching caveat.** The locale file stores quest titles, descriptions, objectives and dialog as separate entries with no link between them. Every description or objective below was matched to its quest by wording (names, places, enemies), not by a data link. Matches that rest on wording alone are marked with a dagger (†). A dagger means "fits this quest, not proven". Strings that fit no listed quest are in the **Unmatched client text** section.
- Rewards (coins, XP, items) do not appear in the locale file and no video frame shows a Brawler quest reward. Every reward cell reads "unknown" unless a source names a reward.
- The client spells some titles differently from the wiki. The client spelling is used, with the wiki spelling noted in the quest cell where it differs.

## Overview

Brawler is a combat job that fights up close with hammers and kicks. [wiki] The client's own description of the job is the job-trainer blurb "Carlos Brazenfist in Cobblestone Village is a great person to talk to if you want to learn the ways of the Brawler" (2653185694) [client]. Experience comes from Brawler quests, battles in the Wilds and dungeons. [wiki] The client tip says combat jobs level by playing "battles" (4024725827). [client]

**Unlocking.** Three trainers can start the job, one per starting area. [client][wiki] The client's "Art of Brawling" text lists all three: "Find someone who can train you in the art of Brawling. Carlos Brazenfist in Cobblestone Village, Harold at Highroad Junction, or Caitlyn Gravefog at Stillwater Crossing." (296956814) [client]. A fourth route, Flanders in the Sacred Glade tutorial, no longer exists. [wiki]

- Each starting line has a first quest that grants the job and opens a one-fight instance: [[King of the Ring!]] (Carlos), [[Hewey's Escape]] (Harold or Archy), [[Cursed Graveyard!]] (Caitlyn). Client lines say the trainer will "teach you the brawler job" before the fight (1378400831, 3003135002, 3584573774, 753054202). [client]
- Hidden objectives named "Hidden Quest Start objective - Harold" (1340394691), "Hidden Start Quest objective - Caitlyn Gravefog" (896003297), "Hidden Dialogue objective - Harold" (3103391636) and "Hidden Dialogue objective - Caitlyn Gravefog" (2235365195) exist in the client. They suggest the start of these lines is driven by a hidden talk objective. [client]
- The wiki says one starting line is enough to continue from Blackspore, and that doing all three gives extra experience. [wiki] The client agrees that all three lines end the same way, by sending the player to Petra with a job offer or referral (see the series tables). [client]

**Levels.** The job runs from level 1 to 20 (**max_level 20**). [wiki] The client Sunstone Valley blurb says players can "Play for FREE as a Brawler, Miner, and Blacksmith all the way to level 20!" (2640708938). [client] The wiki says non-members could max the job from 16 March 2012, yet it also marks three main-series quests (Fight for Your Right, Achieve Level 20, Paying your Dues) and one contract (Troll Brawling) as members only. See **Conflicts**.

**Level gates.** The main series stops at four named trainers until the player reaches a level. Each gate has its own client lines (see **Trainers**): Petra at 5, Mayor Crystalline at 10, Jeni Shortfuse at 15, Rourke Ironfist at 20. [client][wiki]

## Trainers

Trainers and level gates. "Starts the line" means the first quest in a starting line. All of these are Brawler-line NPCs; the zone is where the source places them.

| NPC | Location | Role | Sources |
|---|---|---|---|
| Ashley Lightwings | Sacred Glade tutorial | Gives "Brawling 101"; tells the player to find Flanders (1151662855). No longer in game. | [client][wiki] |
| Flanders | Sacred Glade tutorial practice yard, south west of Ashley | Starts the line in the tutorial and teaches the job ("Learn some fighting moves from Flanders!" 2805330495). No longer in game. | [client][wiki] |
| Carlos Brazenfist | Cobblestone Village, at the Fighter Pit / Brawling Ring (3636618686, 1534164790) | Starts the Cobblestone line and teaches the job. Trainer in [[NPCs - Cobblestone Village]]. | [client][wiki] |
| Harold | Highroad Junction, at his wagon (3410136663) | Starts the Highroad line per the wiki. In the client and video he is the one being rescued. A "bear problem" speaker who teaches the job first (3003135002) is probably him. Coaches the player inside the instance. Trainer in [[NPCs - Highroad Junction]]. | [client][video][wiki] |
| Assistant Archy | Highroad Junction, near the wagons (1624210645) | Gives "Brawler: Hewey's Escape!" in the client and video. | [client][video] |
| Caitlyn Gravefog | Stillwater Crossing, to the north (3927973780) | Starts the Stillwater line and teaches the job. Trainer in [[NPCs - Stillwater Crossing]]. | [client][wiki] |
| Tevin (Assistant Mayor) | Snowhill, in front of the town hall (1035129215) | Ends the Highroad line with "Brawler: Way of the Brawler" and refers the player to Petra. See [[NPCs - Snowhill]]. | [client][wiki] |
| Petra | Blackspore, in the swamps. Client places her "west of Mire's Edge" (298629165) and "north of Mire's Edge" (1778688354). | Main series start. **Blocks until level 5** ("Reach level 5 as a brawler, then return to Petra." 344566168). See [[NPCs - Blackspore]]. | [client][wiki] |
| Mayor Crystalline | Snowhill, by the clocktower / town hall (2539266454) | **Blocks until level 10** (3925636851). | [client][wiki] |
| Jeni Shortfuse | Client: Howling Hills on ice-troll duty (3882321755), later near Sanctuary at the Queensfields (2893356683). Wiki: Snowhill. | **Blocks until level 15** (3940651359). | [client][video][wiki] |
| Rourke Ironfist | Lakeshore | **Blocks until level 20** (2901628704). Members only per the wiki. See [[NPCs - Lakeshore]]. | [client][wiki] |
| Fernando Flexsteel | Stillwater Crossing per the wiki. The client gives no location. | Bounty-hunter giver of the four repeatable "Contract" quests. In the video, "Talk to Fernando Flexsteel" is a listed objective of "Brawler: Achieve Level 15". | [client][video][wiki] |
| Chaz Strongarm | Lakeshore Fishing Village (2087382628) | Runs the Brawler Club after level 20 ("Check in with Chaz and he should have some work for you." 4024416966). His own quests are not mapped here. | [client] |
| Merigold, Samantha | Starting areas | Hand-off lines only: "Merigold has pointed you towards Harold" (2362498448), "Samantha has pointed you towards Carlos Brazenfist" (3965578265). | [client] |

### Other NPCs in the quest line

| NPC | Where | Used in | Sources |
|---|---|---|---|
| Kibler, Mongo | Cobblestone Village Brawling Ring. Mongo is the troll; Kibler is his owner. | Carlos's quests ("shamed ... by Kibler and his pet troll Mongo" 608807105) | [client][wiki] |
| Jones | Makeshift camp on the road east from Cobblestone Village toward Blackspore (18137765) | Delinquent Dilemma, Bad Business | [client][wiki] |
| Sabastian Cabet | Blackspore (Jones's son) | Given to Guard | [client][wiki] |
| Gerold | Camp "north along the road to Snowhill" (180775131), "south of Snowhill" (1744098469). Harold's twin brother (379975581). | Growler Encroachment, The Growler Report | [client][wiki] |
| Seth McGrobie | Camp outside Stillwater Crossing, last seen heading south east (3422963869) | Get McGrobie!, Stolen Merchandise, Rightful Owner | [client][wiki] |
| Virgil Veilmont | Mire's Edge cemetery, Blackspore (3434446051) | Rightful Owner, Cemetery Security | [client][wiki] |
| Andrew | Briarwood, near the warpstone before the maze (2783126347) | Missing Wallet, Missing Money, IOU! | [client][wiki] |
| Nurble (junkmaster) | Robgoblin Camp (1649648269) | IOU!, One Man's Junk..., Curious Key | [client][wiki] |
| Sorin | Singing Crystal Mines, Snowhill (3234034289) | Alternate silver-dust buyer (see Unmatched client text) | [client] |
| Sampson Idlefist | Wiki: south east of Snowhill. Client: "south of Snowhill" (3842204484) and "in Sanctuary" (656570057). | I Want My Money!, Providing Proof, Confronting the Crook | [client][video][wiki] |

## Quest line

Order is the wiki's order, since neither the client nor the video gives a quest sequence. Level and prerequisite are only filled in where a source states them. Quest tables use the client spelling.

### Entry quests (optional, outside the three starting lines)

| # | Quest (client title) | Giver, location | Prerequisite / level | Objectives and text | Rewards | Title ids | Sources |
|---|---|---|---|---|---|---|---|
| E1 | Brawling 101 (wiki: "Tutorial: Brawling 101") | Ashley Lightwings, Sacred Glade tutorial | Tutorial start. Wiki: optional, unlocks the job. No longer in game. | "Ashley wants you to find her friend Flanders so that you can learn how to be a Brawler!" (1151662855); "Find and speak with Flanders. He should be near the practice yard to the south west of Ashley Lightwings." (2683813173) † | unknown | ucdt 3192267549 / ugdt 155837148 | [client][wiki] |
| E2 | The "Hands On" Method (wiki: "Tutorial: The "Hands On" Method") | Flanders, tutorial practice yard | After E1. Optional. No longer in game. | "Talk to Flanders after changing into a Brawler" (1736789071); "Show Flanders you can follow directions by talking to him after changing into your new Brawler job." (1308513445); "Learn some fighting moves from Flanders!" (2805330495); "Find Flanders and tell him about your first victory as a Brawler!" (1204985290) † | unknown | ucdt 1616251712 / ugdt 872075585 | [client][wiki] |
| E3 | Art of Brawling | None. Wiki: added by clicking any dungeon while the player has no combat job. Optional. | No combat job | "Find someone who can train you in the art of Brawling. Carlos Brazenfist in Cobblestone Village, Harold at Highroad Junction, or Caitlyn Gravefog at Stillwater Crossing." (296956814). Related objectives: "Get a combat job!  See Carlos Brazenfist in Cobblestone Village." (2054056858), "Become a Brawler.  Find the trainer, Caitlyn." (543967656), "Obtain the Brawler job" (258397747) † | unknown | ucdt 1703411187 / ugdt 889379665 | [client][wiki] |

The client titles for E1 and E2 have no "Tutorial:" prefix. Strings about the Sacred Glade picnic ("Use your Brawler skills to get rid of the robgoblins ruining Ashley's party!" 901191017) may belong to a tutorial quest, but no title could be matched, so they are in the unmatched section.

### Starting line 1: Cobblestone Village (Carlos Brazenfist)

| # | Quest (client title) | Giver, location | Prerequisite / level | Objectives and text | Rewards | Title ids | Sources |
|---|---|---|---|---|---|---|---|
| C1 | Brawler: Bringing Back Brazenfist | Carlos Brazenfist, Cobblestone Village | Start of line. **Unlocks the job.** | "As a brawler, partner with Carlos Brazenfist in Cobblestone Village, to help him defeat Mongo and restore his honor." (3456933436); "Talk to Carlos Brazenfist, and see if you can help him restore his honor." (1246729765); "Interact with the gate to the Brawling Ring behind Carlos to fight Mongo." (623292891); "Defeat Mongo in the brawling ring" (1844179881). Carlos teaches the job and hands over "some gear" and "a few potions" (2441571970) † | Basic gear and potions from Carlos (items unnamed). Coins/XP unknown. | ucdt 4039116213 / ugdt 3597755459 | [client][wiki] |
| C2 | Brawler: Honor Lost | Carlos Brazenfist | After C1 | "Carlos Brazenfist in Cobblestone Village has been shamed for the last time by Kibler and his pet troll Mongo. Use your brawling skills to defeat Mongo once and for all, and restore Carlos's honor!" (608807105) †; a variant for players who already have the job: "...teach Mongo a lesson, especially since you already know how to brawl." (3525770216) † | unknown | ucdt 833784102 / ugdt 249251189 | [client][wiki] |
| C3 | Brawler: Building your Rep (wiki: "Building Your Rep") | Carlos Brazenfist | After C2 | "You should build your reputation as a Brawler. I heard some local hooligans have been causing trouble. See if Jones needs your help. He's at the camp to the east." (2520950946) †; "Find Jones at his ramshackle camp, east along the road to Blackspore, and help him with his Hooligan problem." (3748962211) † | unknown | ucdt 1157026386 / ugdt 2140058686 | [client][wiki] |
| C4 | Brawler: Delinquent Dilemma | Jones, camp east of Cobblestone Village | After C3 | "As a brawler, defeat the Hooligan Delinquents south of Jones to recover the artifacts they stole from him." (1138712921); "Recover Pilfered Artifacts from Hooligan Delinquents" (2318463689); "Return the artifacts to Jones" (710022440) | unknown | ucdt 4051044198 / ugdt 3177811994 | [client][wiki] |
| C5 | Brawler: Bad Business | Jones | After C4 | "Jones has asked you to look for his son, Sabastian Cabet, in Blackspore. Jones has also warned you that it can be a dangerous place, so make sure to be careful." (2093587189); "As a brawler, search for Sabastian Cabet in Blackspore Swamp. The entrance to Blackspore is a large cave directly east of Cobblestone Village." (4283728578) | unknown | ucdt 3547801737 / ugdt 1266448623 | [client][wiki] |
| C6 | Brawler: Given to Guard | Sabastian Cabet, Blackspore | After C5. Leads to Petra. | "Sabastian Cabet wants you to deliver a job offer to Petra, a famous brawler located in Blackspore." (3838971935); "Petra is somewhere to the west of Mire's Edge in Blackspore. Find her and deliver Sabastian's job offer." (298629165) | unknown | ucdt 2513078725 / ugdt 1114540062 | [client][wiki] |

### Starting line 2: Highroad Junction (Harold / Archy)

| # | Quest (client title) | Giver, location | Prerequisite / level | Objectives and text | Rewards | Title ids | Sources |
|---|---|---|---|---|---|---|---|
| H1 | Brawler: Hewey's Escape! | Client and video: Assistant Archy, Highroad Junction. Wiki: Harold. | Start of line. **Unlocks the job.** | "As a brawler, help Archy save Harold from Hewey the escaped bear in Highroad Junction." (1605633744). Tracker, in order: "Talk to Archy" (300747673), "Save Harold from Hewey the Bear" (1107073768) (aveqYa4AVk8/00040, aveqYa4AVk8/00000: "Talk to Archy" ticked, giver Archy). Then "Click on the gate to Hewey's Escape" (1714966968) and "Knock out Hewey the Bear!" (2247616509). Hand-in: "Now that Hewey is all worn out, meet Harold back at his wagon in Highroad Junction to collect your reward." (3410136663) | Quest reward unknown. The dungeon wheel (wiki) is on [[Hewey's Escape]]. | ucdt 4204217995 / ugdt 3490633876 | [client][video][wiki] |
| H2 | Brawler: Restless Rumors | Harold | After H1 | "Harold's twin brother Gerold is having some trouble with wolves south of Snowhill. Find him and see what you can do to help!" (379975581) †; "As a brawler, find Gerold along the road leading north into Snowhill." (3128659717) † | unknown | ucdt 730283070 / ugdt 2872459359 | [client][wiki] |
| H3 | Brawler: Growler Encroachment | Gerold, south of Snowhill | After H2 | "As a brawler, knock out Frostfang Growlers to the east of Gerold." (3027406492); "Defeat all of the Frostfang Growlers." (658160985) †; "Return to Gerold, south of Snowhill, and tell him of your success against the Frostfang Growlers!" (1744098469) | unknown | ucdt 3978607958 / ugdt 956204014 | [client][wiki] |
| H4 | Brawler: The Growler Report | Gerold | After H3 | "Gerold has asked you to deliver his report on the Growler situation to Assistant Mayor Tevin in Snowhill." (1470657465) | unknown | ucdt 1220364157 / ugdt 2443760478 | [client][wiki] |
| H5 | Brawler: Way of the Brawler | Tevin, Snowhill | After H4. Leads to Petra. | "Gerold spoke highly of your skills as a brawler in his report, and Tevin has suggested you continue improving those skills. His old partner, Petra, can help you with that. Find her in the swamps of Blackspore." (1014135925); "As a brawler, find Petra in the swamps of Blackspore." (3031934489) †. Tevin's dialog mentions Vaelen Warpwatcher, who teleports players to Blackspore (998605592). | unknown | ucdt 1258558122 / ugdt 736817341 | [client][wiki] |

### Starting line 3: Stillwater Crossing (Caitlyn Gravefog)

| # | Quest (client title) | Giver, location | Prerequisite / level | Objectives and text | Rewards | Title ids | Sources |
|---|---|---|---|---|---|---|---|
| S1 | Brawler: Graveyard Grumble | Caitlyn Gravefog, Stillwater Crossing | Start of line. **Unlocks the job.** | "As a brawler, help Caitlyn Gravefog in Stillwater Crossing by defeating the wraith in the Cursed Graveyard." (4207940928; variant with a capital W 3451178374); "Click on the gate to the Cursed Graveyard north west of Caitlyn Gravefog to start the battle!" (357349257); "Defeat the graveyard wraith!" (362231453); "You've defeated the wraith! Head back to Caitlyn and let her know it's safe to enter the graveyard again." (3052502896) | Quest reward unknown. See [[Cursed Graveyard!]]. | ucdt 1833939976 / ugdt 3822555601 | [client][wiki] |
| S2 | Brawler: Get McGrobie! | Caitlyn Gravefog | After S1 | "Caitlyn Gravefog knows why her graveyard was cursed. It was because of that headstone Seth McGrobie sold her! Find Seth and make him give Caitlyn her money back." (1640143829); "As a brawler, find Seth McGrobie who was last seen traveling southeast out of Stillwater Crossing." (3422963869) | unknown | ucdt 3469974208 / ugdt 3744151610 | [client][wiki] |
| S3 | Brawler: Stolen Merchandise | Seth McGrobie, camp outside Stillwater Crossing | After S2 | "As a brawler, defeat Robgoblin Pondblasters to the east of Seth McGrobie's camp, outside of Stillwater Crossing." (1546609659). **Count 30**: tracker "Defeat Robgoblin Pondblasters - 1/30" through "27/30" (jt1vMCwS8h4/00018, /00028, /00038, /00048). A full-screen card "Robgoblin Pondblasters!" appears at 27/30 (jt1vMCwS8h4/00048). Hand-in text: "Those robgoblins won't give up their ill gotten gains without a fight. Knock a few of them out to recover all of Seth McGrobie's merchandise." (3371705788) † | unknown | ucdt 1071817985 / ugdt 2590049011 | [client][video][wiki] |
| S4 | Brawler: Rightful Owner | Seth McGrobie | After S3 | "Bring the merchandise you found back to Seth McGrobie so that he can pay back Caitlyn." (2530199340) †; "Seth McGrobie has asked you to help get rid of his curse by telling Virgil Veilmont in Blackspore where he can find the headstones that McGrobie "borrowed" from him." (3491657000); "As a brawler, find Virgil Veilmont at the Mire's Edge cemetery, far to the south in Blackspore." (3434446051) | unknown | ucdt 4277306924 / ugdt 3481172471 | [client][wiki] |
| S5 | Brawler: Cemetery Security | Virgil Veilmont, Mire's Edge cemetery, Blackspore | After S4. Leads to Petra. | "Virgil Veilmont wants you to deliver a job offer to Petra, a veteran brawler who resides in Blackspore." (296481797); "Petra is north of Mire's Edge in Blackspore. Find her and deliver Virgil's job offer." (1778688354) | unknown | ucdt 3750868535 / ugdt 2768213677 | [client][wiki] |

### Main series, band 1: level 1 to 5 (Petra, Blackspore)

Entry from any of C6, H5 or S5. All Petra gates are enforced by level, not by quest.

| # | Quest (client title) | Giver, location | Prerequisite / level | Objectives and text | Rewards | Title ids | Sources |
|---|---|---|---|---|---|---|---|
| M1 | Brawler: Prey on the Cray | Petra, Blackspore | Arrive via C6, H5 or S5 | "Not interested in training under Carlos any longer, Petra has decided to try to make some money by taking gems from the Cracked Claw Caverns. She has asked you to retrieve some of them. Be careful of the swamp cray!" (1105828342) †; "As a brawler, retrieve radiant gems from inside the Cracked Claw Caverns of Blackspore." (2977396604) †; "Collect radiant gems" (139578799) †. Petra's first words to a newcomer: "Sure, I can teach you more about being a brawler, but first I need to know you can handle yourself in a fight." (3927787240). See [[Cracked Claw Caverns]]. | unknown | ucdt 856863320 / ugdt 1205612188 | [client][wiki] |
| M2 | Brawler: Achieve Level 5 | Petra | Level 5 | "Reach level 5 as a brawler, then return to Petra." (344566168); wait text "Your brawling training will continue once you have reached level 5 as a brawler. Return to Petra at that time." (3317336969). Journal strings "Achieve Level 5 as a Brawler" (3664401017) and goal "Brawler: Level 5" (2505749063). Map legend shows it as the active quest (juuNtGDgDjc/00014). | unknown | ucdt 2041706193 / ugdt 3577001779 | [client][video][wiki] |
| M3 | Contract: Hello Fernando | Fernando Flexsteel, Stillwater Crossing. **Repeatable.** | Wiki order: after M2. Level not stated. | "You are standing before legendary bounty hunter, Fernando Flexsteel. No need to bow. Petra has sent you to help me complete some bounties, no? Well let's get started!" (3566246703); "Oh my. You have returned but not yet defeated the horrible creatures you were tasked to defeat." (2932475783) †. Objective text not matched; a candidate is listed under Unmatched client text. | unknown | ucdt 495972426 / ugdt 2898916836 | [client][wiki] |

### Main series, band 2: level 5 to 10 (silver dust and the Snowhill key)

| # | Quest (client title) | Giver, location | Prerequisite / level | Objectives and text | Rewards | Title ids | Sources |
|---|---|---|---|---|---|---|---|
| M4 | Brawler: Splendid Silver! | Petra | After M2 (level 5) | "Petra would like for you to collect some silver dust from tormented spirits that wander around Blackspore." (616697640) †; "Defeat the tormented souls wandering the southern reaches of Blackspore as a brawler to collect pouches of silver dust." (3481194386) †; "Obtain pouches of silver dust" (24784006) †. The amount is "a dozen silver dust samples" (2245156433) †. | unknown | ucdt 843192539 / ugdt 543033624 | [client][wiki] |
| M5 | Brawler: Finding a Buyer | Petra | After M4 | "Petra has asked you to find a buyer in Briarwood for the silver dust you obtained." (3595505996); "Look for a treasure hunter in Briarwood that may be willing to buy that silver dust you looted." (3146752245) † | unknown | ucdt 1472669897 / ugdt 3056676963 | [client][wiki] |
| M6 | Brawler: Missing Wallet | Andrew, Briarwood | After M5 | "As a brawler, find Andrew's wallet amongst the Bristlewood Arachnids." (319709127); "Return to Andrew in Briarwood" (811949141) †. Andrew: "I'd love to buy these, but I lost my wallet and all my money." (1975092436) † | unknown | ucdt 2435809608 / ugdt 1087803489 | [client][wiki] |
| M7 | Brawler: Missing Money | Andrew | After M6 | "As a brawler, find Andrew's money in Arachnia's Lair, at the edge of Briarwood Forest." (1970455788); "Recover Andrew's lost money" (2717877565); "Return Andrew's lost money." (2822053296) †. Dungeon: [[Arachnia's Lair]]. | unknown | ucdt 732384495 / ugdt 465037923 | [client][wiki] |
| M8 | Brawler: IOU! (wiki: "IOU") | Andrew | After M7 | "Andrew still doesn't have enough money to pay you for the silver dust. Nurble at the Robgoblin Camp owes him some money though. Go collect it for Andrew." (207172090); "Talk to Nurble at the Robgoblin Camp." (2816772335) | unknown | ucdt 1312438298 / ugdt 620477359 | [client][wiki] |
| M9 | Brawler: One Man's Junk... | Nurble, Robgoblin Camp | After M8 | "As a brawler, collect the stolen goods from the Robgoblin Troublemakers that lair to the north of the Robgoblin Camp and return to Nurble when done." (3047239161); "Recover the junk from them and return it to Nurble so he can pay Andrew's IOU." (1076069151) | unknown | ucdt 2036192541 / ugdt 964430896 | [client][wiki] |
| M10 | Brawler: Curious Key | Nurble | After M9 | "Nurble discovered a key to the city of Snowhill within the junk you recovered for him. It apparently belongs to Mayor Crystalline. Return it to her in Snowhill." (212523396); "Return the key to Mayor Crystalline in Snowhill." (2299574401). Mayor's reaction: "Thank you for being so honest and returning it!" (751375033) †. | unknown | ucdt 747938953 / ugdt 1207251011 | [client][wiki] |
| M11 | Brawler: Achieve Level 10 | Mayor Crystalline, Snowhill | Level 10 | "Reach level 10 as a brawler, then return to Mayor Crystalline." (3925636851); wait text (4140533352); "Come back when you reach level 10 as a brawler. We have much to do, my friend." (4144770568) †. Journal "Achieve Level 10 as a Brawler" (1538923985); goal "Brawler: Level 10" (863884951). | unknown | ucdt 4284196844 / ugdt 2671412968 | [client][wiki] |

### Main series, band 3: level 10 to 15 (Snowhill and Sampson)

| # | Quest (client title) | Giver, location | Prerequisite / level | Objectives and text | Rewards | Title ids | Sources |
|---|---|---|---|---|---|---|---|
| M12 | Contract: Colder Weather | Fernando Flexsteel. **Repeatable.** | Wiki order: after M11. | "This bounty has you going to Snowhill to defeat some nasty ice creatures. Get some warm brawler gear on and prepare for adventure!" (105952643) †; "As a brawler, defeat Frosty Yetis, and Ice Trolls that wander the icy slopes Snowhill." (3823327910) † (client typo "Snowhill" without "of"); reminder "Why have you not defeated those snowy beasts for the bounty I gave you? Are you afraid of the cold?" (4003539695) † | unknown | ucdt 3946995887 / ugdt 3549991489 | [client][wiki] |
| M13 | Brawler: An Honest Mistake | Mayor Crystalline | After M11 | Sorin refuses a so-called brawler's gems and sends the player back: "Sorin has recently been scammed by a so-called brawler who tried to sell gems to him. He's not buying your story and has asked you to talk to Mayor Crystalline in Snowhill." (3669888178) †; "You brawlers should really watch your reputation. Some problems you can't just fight your way out of... Mayor Crystalline has heard of the shenanigans going on, and she's asked to speak with you." (1218945572) †; a buyer (probably Sorin): "So it's all a misunderstanding you say?" (2645956707) † | unknown | ucdt 3585750460 / ugdt 2332315243 | [client][wiki] |
| M14 | Brawler: Fraudulent Brawler (wiki: "Fradulent Buyer") | Jeni Shortfuse (client: on ice-troll duty at Howling Hills; wiki: Snowhill) | After M13 | "Mayor Crystalline wants you to talk to a troubled Brawler named Jeni Shortfuse, who is stationed near Howling Hills on Ice Troll duty." (284991750) †; "Help the troubled Brawler, Jeni Shortfuse, who's on Ice Troll duty outside of Howling HIlls." (755445428) † (client typo "HIlls"); "Jeni claims she was setup by Sampson Idlefist. Now you and Jeni need to clear your names and restore your good reputation with the Mayor of Snowhill. Perhaps defeating some ice trolls in Snowy Canyon would do the trick." (669548179) †; "Pummel some Ice Trolls in the Howling Hills and then return to me." (2209636019) † | unknown | ucdt 4188888238 / ugdt 1406997752 | [client][wiki] |
| M15 | Brawler: I Want My Money! (wiki: "I Want My Money") | Sampson Idlefist (client: Sanctuary / south of Snowhill; wiki: south east of Snowhill) | After M14 | "Sampson Idlefist in Sanctuary wants you to collect his "protection" money." (3086409554) †; "Sampson wants his money back for offering his protection to several people. He doesn't care how you get it, just do it!" (864025712) †; "Gain Sampson's trust by collecting his money from Tori." (57669918), "...from Hartley Harrison." (984637270), "...from Bluerose." (4044833138) † | unknown | ucdt 3807785477 / ugdt 477733253 | [client][wiki] |
| M16 | Brawler: Achieve Level 15 | Jeni Shortfuse | Level 15 | "Reach level 15 as a brawler, then return to Jeni Shortfuse." (3940651359). Wait text (2852056939). Seen in the video as: description "Brawler: Achieve Level 15 - Reach level 15 as a brawler, then return to Jeni Shortfuse.", objectives "Brawler: Level 15" and "Talk to Fernando Flexsteel", giver Jeni Shortfuse (fMOGHMuA2Hc/00000 and 60+ other frames from that video). Journal "Achieve Level 15 as a Brawler" (1152313014); goal "Brawler: Level 15" (1107938647). | unknown | ucdt 3618962003 / ugdt 2146334259 | [client][video][wiki] |

### Main series, band 4: level 15 to 20 (the Sampson showdown and Brawler Club)

Order is the wiki's. The wiki marks M21 to M24 as members only.

| # | Quest (client title) | Giver, location | Prerequisite / level | Objectives and text | Rewards | Title ids | Sources |
|---|---|---|---|---|---|---|---|
| M17 | Contract: Local Bounty | Fernando Flexsteel. **Repeatable.** | Wiki order: after M16. | Objective text not matched (candidate in Unmatched client text). | unknown | ucdt 542119772 / ugdt 2689972243 | [client][wiki] |
| M18 | Brawler: Providing Proof | Jeni Shortfuse | After M16 | "Jeni wants you to talk to Sampson Idlefist in Sanctuary and expose him for the fraud he really is." (4154230293) †; "We have to prove Sampson Idlefist is a fraud so that we can reclaim our brawler pride! We'll need to gain his trust first... use this recorder to try and capture some evidence!" (3612327809) †; "Now that you appear to have gained Sampson's trust, you must get a recording of something he says that may be proof of his shady dealings." (4110480696) †; "Take the recording of Sampson to Mayor Crystalline in Snowhill!" (574491294) †. Chat lines "Brawler Providing Proof Quest Started!" and "...Completed!" appear around the Sampson fight (vokALCeGZqY/00018, /00034, /00064). | unknown | ucdt 1376350525 / ugdt 831993972 | [client][video][wiki] |
| M19 | Brawler: Confronting the Crook | Mayor Crystalline, Snowhill | After M18 | Video tracker: "Talk to Sampson" (complete), "Defeat Sampson Idlefist in battle!", "Return to Mayor Crystalline in Snowhill!" (vokALCeGZqY/00064); description "Confront Sampson Idlefist outside of Snowhill." (video, vokALCeGZqY/00064; client 2150507159). Client: "Defeat Sampson Idlefist in battle!" (3531274289); "Mayor Crystalline wants you to rid Sacred Grove of Sampson Idlefist once and for all. Confront him outside of Snowhill and defeat him in battle!" (2826198928). The fight is a boss challenge: text "Sampson Idlefist thinks no one can defeat him! Knock some humility into him!" (3534259923); panel shows "Requires a level 9 combat job", three Brawler item prizes and a "Daily Grab Bag" for 450 coins (vokALCeGZqY/00010). Boss line: "I can't believe you had the courage to show up! Let's finish this!" (vokALCeGZqY/00020). | Quest reward unknown. Fight prizes: three Brawler items (names not readable). | ucdt 1147393094 / ugdt 536674650 | [client][video][wiki] |
| M20 | Brawler: Reputation Regained | Mayor Crystalline | After M19 | "Mayor Crystalline wants you to let Jeni Shortfuse know that Sacred Grove is indebted to you both and that she is now officially free of ice troll duty!" (2579397611) †; "Let Jeni Shortfuse in Snowhill know that she was released from ice troll duty." (3351838503) †. Jeni then points the player to the Brawler Club: "Thanks to you, our names have been cleared..." (1385074921); "You should talk to Rourke Ironfist in Lakeshore about joining the Brawler Club." (434419887) † | unknown | ucdt 2442530618 / ugdt 3692767672 | [client][wiki] |
| M21 | Brawler: Fight for Your Right **(members only, wiki)** | Rourke Ironfist, Lakeshore | After M20 | "Defeat Troll Roughguards in the Forest Troll Fort as a brawler." (771540081). Count **5**: video tracker "Defeat Troll Roughguards - 0/5" with a "Take Me There" marker to Forest Troll Fort (FsCYVYzjnpg/00018, /00104, /00138). Description: "Now that you know the rules of Brawler Club, you will need to prove you can fight before Rourke will let you join. Defeat Troll Roughguards in the Forest Troll fort." (2126173344) †. Dungeon: [[Forest Troll Fort]]. | unknown | ucdt 584305414 / ugdt 547667738 | [client][video][wiki] |
| M22 | Brawler: Achieve Level 20 **(members only, wiki)** | Rourke Ironfist | Level 20 | "Reach level 20 as a brawler, then return to Rourke Ironfist." (2901628704); wait text (2042311147). Goals "Reach Level 20 as a Brawler" (822010770 / 489514764); "Brawler: Level 20" has two entries (2418471390, 3617216321). | unknown | ucdt 2768426379 / ugdt 1407922908 | [client][wiki] |
| M23 | Contract: Troll Brawling **(members only, wiki)** | Fernando Flexsteel. **Repeatable.** | Wiki order: after M22. | "As a brawler, defeat trolls in Forest Troll Fort and find and defeat Brugo." (2369808622) †; "Hey amigo! It is time to bash some trolls." (2288957617) †; "Did you take care of those trolls yet, amigo?" (3560931082) † | unknown | ucdt 2272272295 / ugdt 422963565 | [client][wiki] |
| M24 | Brawler: Paying your Dues (wiki: "Paying Your Dues") **(members only, wiki)** | Rourke Ironfist | After M22 | "You certainly have the brawn to join Brawler Club, but even skilled brawlers like yourself have to pay their dues. All members get a weapon. Of course, you'll have to make a donation to the club and find the materials yourself, but it's worth it! Interested in earning one?" (1732652847) †; "[Player] can now proudly wield their Brawler Club weapon!" (1071514084) †. A crafting material says "Requires quest: "Brawler: Paying your Dues."" (181883626). | Weapon **The Rumbler** (level 20, Power 5, Glancing Blow and Leg Sweep) [wiki]. Coins/XP unknown. | ucdt 4000587074 / ugdt 1582345642 | [client][wiki] |

After M24 the client continues into the Brawler Club with Chaz Strongarm in Lakeshore ("Welcome to Brawler Club! ... Check in with Chaz and he should have some work for you." 4024416966). Those quests are not part of the wiki's Brawler list. One has enough text to list as a row (BC1); the rest are not mapped here.

### Brawler Club follow-up (Chaz Strongarm, client only)

Order is by context only. None of this is in the wiki's Brawler quest list, and no video frame shows it. The video's "Hooligan Bullies" frames (3jLPsxADZ1w) belong to a different quest, "Local Hero", run on a Medic in South Sacred Grove, and are not used here.

| # | Quest (client title) | Giver, location | Prerequisite / level | Objectives and text | Rewards | Title ids | Sources |
|---|---|---|---|---|---|---|---|
| BC1 | Title not proven. "Hooligan Bullies!" is the candidate (see Conflicts 12). | Chaz Strongarm, Lakeshore ("Chaz wants you to defeat the hooligan bullies" 2825469905; Chaz is in Lakeshore, wiki and client 459411207). Targets roam outside Farnum Farmstead ([[NPCs - Farnum Farmstead]]). | Brawler Club member, so after the M21 to M24 gate by context. Level unknown. | "Hooligans have been spreading rumors about Brawler Club in the dark of night. I want you to stop this. Defeat Hooligan Bullies near Farnum Farmstead. They are the ones that are causing the most trouble." (1297081741); "Chaz wants you to defeat the hooligan bullies that have been spreading rumors about the Brawler Club in the dark of the night. The hooligans can be found wandering around outside of Farnum Farmstead and the wilderness." (2825469905). Objectives: "Defeat Hooligan Bullies wandering near Farnum Farmstead as a brawler." (4036179728); "Defeat hooligan bullies" (3331761840); "Stop the hooligan bullies!" (15774823). Hints: "Are you having trouble finding the hooligan bullies? They should be wandering outside of Farnum Farmstead and throughout the wilderness causing trouble." (1929508390; variant 3126773827). The kill count is not stated. | unknown | candidate: ucdt 694203918 / ugdt 982150142 "Hooligan Bullies!" (a third ucdt 3875666612 has the same text; the enemy name entry is 485224451 "Hooligan Bullies") | [client] (title weak) |


### Client-only quest titles (not in the wiki list)

These five titles exist as `Brawler:` quests in the client. No giver, location or objective could be tied to any of them with confidence.

| Quest (client title) | Title ids | Best guess from wording | Sources |
|---|---|---|---|
| Brawler: Prized Brawlers | ucdt 1664531761 / ugdt 4156705342 | Possibly the Carlos quest that sends the player after "one of Kibler's unstoppable recruits" trained by Sampson (1714012446, 220085769), ending with "Report to Petra in Blackspore for further instruction." (3556251182). Unproven. | [client] |
| Brawler: Bear Safety | ucdt 527059134 / ugdt 2971801818 | Possibly an older Hewey line. Only a loose thematic match (73042736: "...you might end up in the middle of a bear cave one day"). | [client] |
| Brawler: Grave Danger | ucdt 1116641955 / ugdt 3700471152 | Possibly an older Caitlyn line. No text matched. Both this and "Graveyard Grumble" exist, so it is not a rename. | [client] |
| Brawler: Fit to Fight | ucdt 1351986487 / ugdt 2141609189 | No text matched. | [client] |
| Brawler: Night Fights | ucdt 2687540202 / ugdt 2157827498 | No text matched. | [client] |

## Abilities and traits by level

Traits are the four level-gated passives. [wiki] Names are confirmed in the client (Bruising Strikes 307664311 / 247678943, Savvy 2727244432 / 721657749, Toughness 3400671368 / 90104885, Resilience 679549300 / 2597081709). [client]

| Level | Trait | Effect (wiki) | Client text | Sources |
|---|---|---|---|---|
| 5 | Bruising Strikes | All opponents you hit are weakened and are more likely to be struck by a critical attack. | Name only (proc entry "Bruising Strikes Proc" 405779129). | [wiki][client] |
| 10 | Savvy | Your critical strikes cause additional damage. | Name only. A possibly related line is "Inflict additional damage when you score a critical hit." (2715545685) †. | [wiki][client] |
| 15 | Toughness | You are able to withstand more damage before being knocked out. | Name only. | [wiki][client] |
| 20 | Resilience | You show great resilience and gain health and energy after being hit. | "You show great resilience and gain health after being hit." (738432971) (no energy). Proc entry "Resilience Proc" 2114223425. | [wiki][client] |

**Abilities.** Brawler abilities are tied to the weapon in hand, not to a job level. The client says "Abilities are tied to the weapon you're using. Equip a different weapon to use different abilities." (819381112) [client]. Tier comments in the client list these abilities, by tier and type [client]:

| Tier | Spammable ability | Ultra ability | Source ids |
|---|---|---|---|
| 1 | Glancing Blow, Wallop | Leg Sweep, Ready to Rumble | 1184992873, 1226665271, 2246428902, 717451594 |
| 2 | Smack, Pound | Pummel, Roundhouse Kick | 2037788749, 3635733609, 468413955, 4079096629 |
| 3 | Wild Swing | Sucker Punch | 1709514678, 430732220 |

- "Enrage" and "Leg Sweep" appear as "Max out Brawler ability" achievements (1614990398, 3607998123). [client]
- Pummel is a learned ability. A wait text says "you don't know how to pummel your opponents yet" (2232233194, 4200601785) and a follow-up says "You learned how to pummel? Excellent!" (2757299385). The level at which it is learned is unknown. [client]
- The Rumbler's wiki entry lists Glancing Blow (2372 damage) and Leg Sweep (8302 damage, knocks down). [wiki]
- Weapon tiers by item level (wiki): level 1 starter hammers; level 3 Log Hammer; level 4 Street Scrapper; level 5 Pro Brawler Hammer and the Kendama line; level 8 Anvil enchantments; level 11 Anvil Hammer; level 12 Drill hammers; level 15 Driller; level 16 Atlas hammers; level 19 Frenzy Fighter Hammer; level 20 Crystal Hammer and quest weapon The Rumbler. Clothing sets run Student Brawler (1) up to Berserk Bruiser (20). [wiki]
- Vendors (wiki): Trent Gingertuft in Blackspore (level 1 weapons, level 20 clothing); Lacy Trembletoes in Lakeshore (level 19 weapons, level 20 clothing). [wiki]
- Collections: Brawler Hammer Materials (550930469), Brawler Magical Hammer Essences (520328136), Brawler Stories (1075446716). [client][wiki]

## Where the job levels

Combat jobs level through battles, which are the dungeons and the wandering fights in the Wilds. [client][wiki] Dungeon entry needs a minimum combat level that grows with difficulty: difficulty 2 recommended level 3, difficulty 3 needs level 5, difficulty 4 needs level 9, difficulty 5 needs level 13. [wiki] The Sampson challenge panel in the video says "Requires a level 9 combat job" (vokALCeGZqY/00010). [video]

Dungeons tied to Brawler quests:

| Dungeon | Difficulty (wiki) | Quest link | Sources |
|---|---|---|---|
| [[King of the Ring!]] | 1, removed | C1, C2. Boss Mongo. | [client][wiki] |
| [[Hewey's Escape]] | 1, removed | H1. Boss Hewey the Bear. In the video the instance runs a combat tutorial: Harold coaches ("Hit the [2] key on your keyboard to use your super!") and a power-up tooltip panel opens (fktAs_GRrjM/00404, /00426, DheErd-aDpQ/00068); the reward wheel appears at the end (fktAs_GRrjM/00486). | [client][video][wiki] |
| [[Cursed Graveyard!]] | 1 | S1. Boss Graveyard Wraith. | [client][wiki] |
| [[Cracked Claw Caverns]] | 1 | M1. | [client][wiki] |
| [[Arachnia's Lair]] | 2 | M7. | [client][wiki] |
| [[Forest Troll Fort]] | 5 | M21, M23. | [client][video][wiki] |
| [[Robgoblin Treasure Trove]] | 2 | Candidate for M17 (see Unmatched client text). | [client][wiki] |
| [[Bandit Hideout!]] | 4 | Candidate for Sampson's coin-purse job (see Unmatched client text). A Brawler at level 9 is seen fighting here (9gfkEIFMV4s/00040). | [client][video][wiki] |
| [[Tavern Cellar]] | 1 | Possible link to a Hooligan Delinquent hand-off (3825082558). Unproven. | [client][wiki] |
| [[Trail of Betrayal]] | 5 | Candidate for a Sampson follow-up (see Unmatched client text). | [client][wiki] |

Open-world fights named in quests: Hooligan Delinquents (C4), Frostfang Growlers (H3), Robgoblin Pondblasters (S3), tormented spirits (M4), Bristlewood Arachnids (M6), robgoblin troublemakers (M9), Frosty Yetis and Ice Trolls (M12, M14). [client]

## Conflicts

1. **Hewey's Escape giver.** Wiki: Harold starts the line. Client and video: the quest is given by Assistant Archy, and Harold is the person being rescued ("help Archy save Harold" 1605633744; giver Archy in aveqYa4AVk8/00000). The client also has Harold-as-trainer variants ("Here to help with my bear problem? Great! But first let me teach you the brawler job." 3003135002), so more than one version of the opener existed. The page uses the client/video giver.
2. **Title spelling.** Client: "Building your Rep", "Paying your Dues", "IOU!", "I Want My Money!", "Fraudulent Brawler". Wiki: "Building Your Rep", "Paying Your Dues", "IOU", "I Want My Money", "Fradulent Buyer". Client titles are used. "Fradulent Buyer" looks like a misspelt wiki name for the client's "Fraudulent Brawler" (the client has no "Fraudulent Buyer"); the match is by position and giver, not proven.
3. **Tutorial titles.** Wiki: "Tutorial: Brawling 101", "Tutorial: The "Hands On" Method". Client: "Brawling 101", "The "Hands On" Method" (no prefix).
4. **Fight for Your Right title.** The video tracker on screen reads "Brawlers Fight for Your Right" in most frames of FsCYVYzjnpg; the client title is "Brawler: Fight for Your Right" (the recorder noted the same in FsCYVYzjnpg/00104). Client wording is used.
5. **Jeni Shortfuse location.** Wiki: Snowhill. Client: on ice-troll duty near Howling Hills (284991750, 3882321755), later near Sanctuary at the Queensfields (2893356683, 490127261), and "Return to Jeni Shortfuse in Snowhill" (1780200458). Probably two stages of the story, not one fixed spot.
6. **Sampson's name and place.** Wiki: Sampson south east of Snowhill. Client: "Sampson Idlefist" in Sanctuary (656570057) or south of Snowhill (3842204484), and a separate "Sampson Stonefist" in some strings (804036497, 3972238692). The video nameplate says Sampson Idlefist. Idlefist is used.
7. **Petra's position.** Client has "west of Mire's Edge" (298629165) from Sabastian's offer and "north of Mire's Edge" (1778688354) from Virgil's. Wiki says only "Blackspore Swamp".
8. **Resilience.** Wiki: health and energy. Client: health only (738432971).
9. **Members only.** Wiki says non-members can max the job and still marks M21 to M24 as members only. The client Sunstone blurb says Brawler is free to 20 (2640708938). The members flag per quest has no client or video evidence either way.
10. **Missing Wallet in the video.** The video's "Missing Wallet" frames show Andrew asking "Can you search Grexan's Camp for my wallet?" and a "Defeat General Grexan" boss fight (1qqVwVCxjGM/00006, /00074). The quest title there has no "Brawler:" prefix, and the client has both "Missing Wallet" (ucdt 1637048359 / ugdt 2642975313) and "Brawler: Missing Wallet" (M6, spiders, Bristlewood Arachnids). This page treats the video as a different quest and does not use it for M6.
11. **Silver-dust buyer.** Wiki and client follow the Andrew, Nurble, Mayor chain. The client also has a Sorin branch (Sorin buys the dust, 1307497720, 1596159037, 3234034289) that does not fit the wiki order. It is placed under M13 only because Sorin's complaint ends with "talk to Mayor Crystalline".
12. **"Hooligan Bullies!" is probably not a Brawler quest title.** The client has the exact string "Hooligan Bullies!" three times (ucdt 694203918, ugdt 982150142, ucdt 3875666612) with no `Brawler:` prefix. The wiki lists "Hooligan Bullies!" as a difficulty 5 battle (its `Battles` page) whose prize wheels give level 16 weapons such as Brawler's Atlas Hammer of Rage, and it has no page of its own. So the string is as likely a battle name as a quest title. [[NPCs - Farnum Farmstead]] treated it as the Brawler Club quest title by wording; that pairing is unproven, so BC1 keeps the text and lists the title as a candidate only. [client] [wiki]

## Gaps

- **Rewards.** No coin, XP or item rewards for any quest are known except The Rumbler for M24. Per-quest rewards likely live in server quest data not present in the locale or videos.
- **Quest-to-text link.** All description, objective and dialog matches are by wording. A real quest table would settle every dagger-marked entry.
- **Giver and location** for Fernando Flexsteel (client has none) and exact places for Tevin, Mayor Crystalline, Rourke Ironfist and Chaz Strongarm inside their towns need the NPC zone pages.
- **Contract objectives.** The objectives of Hello Fernando, Local Bounty and Troll Brawling are not established. Candidate texts: "As a brawler, defeat Robgoblin Pondblasters, Amazing Asps, and Killer Cobras." (2810740797) and "As a brawler, defeat PCA Operatives and robgoblins in the Robgoblin Treasure Trove." (24632303). Neither can be tied to a contract.
- **Level for the first quest of each band.** Only the four Achieve Level gates state a level.
- **Quest order inside a band** is the wiki's order. The client and video show no sequence flags.
- **Client-only titles.** Bear Safety, Fit to Fight, Grave Danger, Night Fights and Prized Brawlers are not placed (see above). Goals "Brawler: Level 4 / 8 / 12 / 16" (295055365, 673500362, 1020569915, 3577711677, also "Reach Level 4 / 8 / 12 / 16 as a Brawler" 4186288298, 2860374067, 1123074533, 2215096391) exist but no quest uses them in any source.
- **Ability unlock levels** and tier-to-weapon mapping are unknown. Wiki gives weapon levels only.
- **Video coverage.** No frames for Carlos Brazenfist, Caitlyn Gravefog, Petra, Tevin, Rourke Ironfist, Seth McGrobie, Jones or most quests in the middle of the series. Quest completion rewards are not shown in any frame.
- **Chaz Strongarm / Brawler Club** quests after level 20 are not covered.

## Unmatched client text

Brawler-related strings that fit no listed quest, kept so they are not lost. Matching is by wording only.

- **Sacred Glade picnic (tutorial).** "Use your Brawler skills to get rid of the robgoblins ruining Ashley's party!" (901191017); "Head back to the picnic area and use your Brawler skills to get rid of the robgoblins ruining Ashley's party." (211233107); "Looks like Ashley could use a hand. Here's some brawler gear. Now teach those robgoblins a lesson!" (3681561598).
- **Carlos prompts from other NPCs.** "I just heard that Carlos Brazenfist is looking for someone to beat that troll Mongo in the Brawling Ring!" (1021160902); "Carlos Brazenfist, the retired Brawler, is looking for some brave souls to help out with the Hooligan issue here in Cobblestone Village. You can find him at the Fighter Pit." (3636618686); "Get a combat job!  See Carlos Brazenfist in Cobblestone Village." (2054056858).
- **Highroad prompts.** "Harold from the Durango Zoo is looking for some help! He needs to knock out an escaped bear!" (3137596165); "A bear recently escaped from the Durango Zoo, and now it's trapped someone inside a cave! If you want to help, talk to Assistant Archy" (1624210645); "Hewey the Bear escaped the local zoo! Knock some sense into Hewey so he'll return to the zoo." (3003893450, also on the wiki dungeon page); "Hewey the Bear has escaped from the zoo and refuses to leave this cave. Knock him out so Harold can take him home." (374052479).
- **Prey on the Cray alternates.** "The swamp cray in Cracked Claw Caverns hoard all sorts of things, even gems!" (3312881182) may be Petra's dialog.
- **Sorin branch of the silver-dust sale.** (1307497720, 1596159037, 3234034289, 3749445082). Likely Sorin: "Oh no, you fighters aren't going to fool me twice!" (2761152484).
- **Sampson follow-up near Sanctuary.** "Why haven't you fought Sampson Idlefist yet? He is at the Trail of Betrayal. All of Sacred Grove is depending on you to end this threat." (843313741); "Jeni has asked you to travel to the Trail of Betrayal and defeat Sampson and his thug bodyguards." (877479789); "Defeat Sampson in the Trail of Betrayal." (3663462466); "Sampson has challenged you to a duel in Darvon's Descent. Show him who the real Brawler is!" (3017559284); "Beat the Bandit Champion, Muggenstomp in the Bandit Hideout to prove that you're tougher than Sampson." (3135837105); "Jeni wants you to enter the Bandit Hideout and collect the money that the bandits stole from Sampson's Clients." (1546849457); "Sampson wants you to head into the Bandit Hideout southwest of Sanctuary and collect some coin purses from the bandits there." (1676208470).
- **Brawler Club follow-up (Chaz Strongarm).** "Chaz here has the final say on who gets to join Brawler Club... Defeat the scum on Chaz's list" (282491522); "Chaz wants you to defeat the hooligan bullies that have been spreading rumors about the Brawler Club" (2825469905); "Defeat Hooligan Bullies wandering near Farnum Farmstead as a brawler." (4036179728); "Chaz wants you to take care of some trouble makers around Sacred Grove. Defeat Mugwort in Mugwort's Hollow, Necrolaceous in Dark Spore Depths, and Snognosh in the Howling Hills." (2431923117); "Chaz Strongarm in Lakeshore wants you to defeat several villains as a brawler." (3720229457).
- **Blutin.** "Defeat Blutin, the Roughguard Captain!" (3462897026) appears with the Forest Troll Fort strings; no quest tie.

Scratch notes for this page: `%USERPROFILE%\Documents\Projects\CartoonMMO\lab\content\scratch\Brawler\`.
