---
type: content-job
job: Fisherman
category: freestyle
members_only: false
max_level: 20
---

# Fisherman

Part of [[Content]] > Jobs. Engine guide for the minigame code: [[Fishing]] (mapped client functions) and [[Minigame Framework]]. Related: [[Chef]] (fish dishes for Merri De'Flor), [[Robgoblin Creek]], [[Cray Caves]], [[Mushroom Cavern]], [[Snowy Canyon]], [[NPCs - Stillwater Crossing]], [[NPCs - Lakeshore]], [[NPCs - Briarwood]], [[NPCs - Snowhill]], [[NPCs - Seaside]], [[NPCs - Sanctuary]].

Source tags: **[client]** locale text (`en_us_data.dat`) and our mapped client code, **[video]** frame records (frame key `video/frame`), **[wiki]** fan-wiki archive. Precedence is client, then video, then wiki. Numbers in parentheses, for example `(2663118808)`, are client locale ids. Each quest title has a `ucdt` entry and, usually, a `ugdt` twin with the same text.

**Job name.** The client spells it "Fisherman" everywhere (job name ids 1300085378, 3979380738, 4175188632; ugdt 3000768155). No variant spelling exists. The video job bars read "Fisherman" too (`ns0rlvCjXtU/00138`, `0IaICGnTWWQ/00010`). [client] [video]

**How the quest data was matched.** The client stores quest titles, offer text, objectives and reminders as separate entries with no id link. The 42 "Fisherman:" titles have no giver, level, reward or order field that we can read. Every string below was matched to a title by wording: **direct** means the string carries the title's own words or its NPC and place; **†** means the string fits but nothing proves it. Anything without a string is "unknown". Order inside a group is inferred only where a string says "after", "next" or "now that"; otherwise it is not an order. No reward table was found, so every reward cell is "unknown" unless a source names one.

## Overview

- Fisherman is a **freestyle** job whose gameplay is the fishing minigame at six fishing spots, plus a quest line that sends the player between Stillwater Crossing, Sacred Grove Shallows, Briarwood, Lakeshore and Snowhill. [client] [wiki Fisherman] [video: "Fishing" tile in the Games menu, `UFsVcZ3X5iA/00014`, `4h2eVoje3Eg/00050`]
- **Max level 20.** The client has "Reach Level 20 as a Fisherman." (140915077) and the title "Fisherman: Level 20" (2323556482). The video job window shows Fisherman 20 (`0IaICGnTWWQ/00010`, `0LCKMQz3zMs/00048`) and Fisherman 1 on a fresh character (`ns0rlvCjXtU/00138`). Level achievements are at 4, 8, 12, 16 and 20; level 20 gives the title "The Angler". [client] [video] [wiki Achievements, Titles]
- **How to unlock.** Talk to **Reed Stillwater** in Stillwater Crossing: "Learn how to be a fisherman by finding Reed Stillwater." (1498860325), "Speak to Reed Stillwater in Stillwater Crossing for more information on how to become a Fisherman." (2889323239). The job-trainer message board points players at trainers for "go fishing" (2435323721); the wiki says the unlock quest starts from the **Message Board in Sanctuary**. Obtaining the job is an achievement: "Obtain the Fisherman job" (3939100553), trophy text "Obtain the Fisherman job to earn this trophy" (708401564). [client] [wiki Fisherman]
- **Members status.** The wiki Jobs list marks Warrior, Wizard, Chef, Kart Driver, Pet Trainer and Postman as members only and does **not** mark Fisherman, so the frontmatter says `members_only: false`. [wiki Jobs] The client rule for every job is "Anyone can play up to level 4, but every job goes up to level 20! Use the Marketplace to purchase any job you want, or pay for a membership to unlock all of them!" (348625404; Station Cash wording 1319279804). A non-member at the level cap sees "Oh, no! Want to keep playing as a Fisherman? Upgrade to unlock more fun!" (1192929349). Fisherman items exist for the sale: "Members Only Fisherman Pants" (407235252), "Fisherman Membership Upgrade Bundle" (3062922474), "If you join now, you'll receive this special new member pack of fisherman items!" (2703082541). Which individual quests, spots or fish are members only is unknown. [client] [wiki]
- **Quest line shape.** 42 "Fisherman:" titles plus 5 level titles, and 9 unprefixed titles that fit the line (see Quest line), with a Related table for repeatable and event quests. The client names no first-to-last order. The text forms nine clusters by NPC and place: Reed's tutorial, the Shallows beginners (Anna Mudskipper, Muggup, Guppie), Jonah Relicreel's bait quests, Vick Sapshot and Callie Kelpcake at the Shallows, Gunther at Brambleback's Bayou, Dooba and Estelle at Darklit Lagoon, Merri De'Flor and the McScuttle family around Lakeshore, Big J's crew at Wintery Basin, and Lenore's family at Frostbitten Banks. Completing the line earns an achievement, "Complete the Fisherman Questline." (67279265, 4076386985). [client]
- **Rank names.** The client has generic rank names "Novice Fisherman" (1514869630), "Intermediate Fisherman" (2027811164), "Advanced Fisherman" (1833403675) and "Master Fisherman" (3060178052), the same pattern as other jobs. Which level each applies to is unknown. [client]

## Trainers

| NPC | Location | Role | Sources |
|---|---|---|---|
| **Reed Stillwater** | Stillwater Crossing, "the fishing job center", with the boat to Sacred Grove Shallows behind him | Starts the line and unlocks the job. "Pleasure to meet ya'. The name's Reed Stillwater. Are you lookin' to catch a few fish? I have some extra gear if you're interested." (2663118808). "Interact with the boat behind Reed Stillwater to enter Sacred Grove Shallows and start fishing!" (2989176545). Identifies odd catches and fungus (845805624, 1094700659, 985992331) and shows the way to Brambleback's Bayou (2651529106, 3405576332). Name ids 992146572, 1393689979. Not seen in video | [client] [wiki Fisherman] |
| **Message Board** | Sanctuary | Wiki: starts "Becoming a Better/Fisherman". Client text only says the board points to job trainers (2435323721) | [wiki] [client] |
| **Anna Mudskipper** | Sacred Grove Shallows | Beginner quest giver: scouting spots (623486525, 1276310580, 1769596398), best equipment lost with Muggup (1534748619), "reeling in a fat butter flyfish" (4003707613), "Report your fishing success to Anna Mudskipper in Sacred Grove Shallows." (1471236277). Name id 3469699336 | [client] |
| **Muggup** | Sacred Grove Shallows; Chef quests also send the player to "Muggup, south of Lakeshore" (1238069234, 31022572) | Beginner quest giver and teacher: scouting variants (1894107280, 2745178719, 2965626921), "purchase a lure from Guppie" (2184478761), "Report your fishing success to Muggup in Sacred Grove Shallows." (4069402652), "Report your casting success to Muggup at Sacred Grove Shallows." (4263989381). Name id 3644769639. Whether these are one NPC in two places is unknown | [client] |
| **Jonah Relicreel** | Sacred Grove Shallows ("Take the tasty baconfish back to Jonah Relicreel at Sacred Grove Shallows", 2091401093) | Bait quests: small fish, butter flyfish, baconfish. Name id 629785534. The wiki puts Jonah in Rainbow Lake (see Conflicts) | [client] [wiki Fisherman] |
| **Guppie** | Sacred Grove Shallows, "set up shop not too far from here" (2277428605) | Lure vendor: "Buy a lure (name) from Guppie in Sacred Grove Shallows." (1920656359); "He carries all the basic supplies a fisherman might need." (641677831). Name id 3661659780. Stock and prices not seen | [client] |
| **Big J** | Snowhill, "Ice Skating and Bait Shop" (352146976, 1941898979) | Owner of the bait shop, runs the ice fishing crew at Wintery Basin: "Big J wants you to join his crew. Head to the Wintery Basin for further instructions." (379035497). A speaker who calls Big J "my brother" says "You should go see him if you need some fishing gear." (1781442419). Placeholder shop lines: "Someday I will be selling ice skating and fishing supplies" (1816866529), "I will be setting up my fishing supplies shop soon" (1579896036), speaker not named. Name id 3579792565 | [client] |
| **Chip Numbwing** | Snowhill per the wiki text, Seaside per the wiki infobox; client: "Chip Numbwing can show you the way" to Lenore's cabin at Frostbitten Banks (216811999, 4011916228) | Wiki: pixie merchant in the Fisherman category. Stock unknown | [wiki Chip Numbwing] [client] |
| **Bluerose** | Lakeshore | Bait seller and lure maker: "Bluerose is running out of bait" (471732220), shiny lures from silver and onyx (2223724148), Merri sends players to her for bait before Rainbow Lake (1333262692). Name id 3424745770 | [client] |
| **Selvi Slowreel** | Lakeshore docks | Boat guide to Rainbow Lake: "Selvi Slowreel on the docks in Lakeshore can show you the way to Rainbow Lake." (3685316205) | [client] |
| **Lila Darklure** | Darklit Lagoon, beside a boat | Guide: "Only Lila Darklure knows the way." (1194232508); video nameplate near the Darklit Lagoon marker (`ag1LtV5-oho/00006`, `piWfe7lXZ9I/00132`) | [client] [video] |

Spot guides without a named NPC in the strings: "Looking to catch some fish? ... I'd be happy to take you there. Are you ready to head out?" is repeated for Sacred Grove Shallows (3033894121, 3436173164), Brambleback's Bayou (372656995, 556094923), Darklit Lagoon (2155301212), Rainbow Lake (879913977, 1794459848, 2952230529), Frostbitten Banks (863439235, 2235241872) and Wintery Basin (401493682). The speakers are not named. [client]

## Quest line

Rewards are unknown unless a cell says otherwise. Locale ids are `ucdt / ugdt`. Level quests are in the last table of this section.

### Entry and tutorial (titles without the "Fisherman:" prefix)

| # | Quest (client title) | Giver | Location | Prerequisite / level | Objectives and text (client string ids) | Rewards | Locale id(s) | Match | Sources |
|---|---|---|---|---|---|---|---|---|---|
| E1 | **Become a Fisherman** | Message Board in Sanctuary [wiki] | Sanctuary to Stillwater Crossing | none seen | "Speak to Reed Stillwater in Stillwater Crossing for more information on how to become a Fisherman." (2889323239); "Speak to Reed Stillwater in Stillwater Crossing" (2058615822) | unknown | 1758278349 / 3334211415 | † (title words) | [client] [wiki] |
| E2 | **Being a Better Fisherman** (wiki: "Becoming a Better/Fisherman") | Message Board in Sanctuary [wiki] | Stillwater Crossing | After E1 (inferred) | "Learn how to advance your skill as a Fisherman by speaking with Reed Stillwater in Stillwater Crossing" (334285285) | unknown | 1976010527 / 3582337918 | direct | [client] [wiki] |
| E3 | **Learn to Fish!** | Reed Stillwater | Stillwater Crossing | none seen | "Learn all you need to know about fishing!" (676088078); "Grab your fishing gear and get ready to reel 'em in!" (61954176); "You won't find a better place to cast your line than at Sacred Grove Shallows ... Just click on the boat over there" (532385540) | unknown (Reed offers "some extra gear", 2663118808; the item is unknown) | 3359670549 / 1197071914 | † | [client] |
| E4 | **Testing the Waters** (wiki: "Testing the waters") | Reed Stillwater [wiki] | Stillwater Crossing to Sacred Grove Shallows | After E3 (inferred) | "It's time to test out your new fishing gear. Take a short boat ride to Sacred Grove Shallows to catch your first fish!" (465495327); "Head to Sacred Grove Shallows to catch your first fish!" (1609335359) | unknown | 3481931046 / 2465040477 | direct (test, first fish) | [client] [wiki] |

### Cluster A - Sacred Grove Shallows beginners (Anna Mudskipper, Muggup, Guppie)

| # | Quest (client title) | Giver | Location | Prerequisite / level | Objectives and text (client string ids) | Rewards | Locale id(s) | Match | Sources |
|---|---|---|---|---|---|---|---|---|---|
| 1 | **Fisherman: Scouting a Spot** | Anna Mudskipper or Muggup (variants) | Sacred Grove Shallows | After E4 (inferred) | "Anna Mudskipper wants you to scout out the docks / the area by the falls / the area near the pond to see if it's a good spot for fishing." (623486525 / 1276310580 / 1769596398; Muggup versions 2965626921 / 1894107280 / 2745178719); "Have you found a good fishing spot yet? Make sure to scout out all my favorite places!" (3034199041, 1897306937); "Muggup is willing to share his favorite fishing spots with you!" (209159216) | unknown | 2289344674 / 1439337978 | direct | [client] |
| 2 | **Fisherman: So A-Luring** | Muggup | Sacred Grove Shallows | After 1 (inferred) | "Muggup wants you to purchase a lure from Guppie and attach it to your line to help you increase your fishing skill." (2184478761); "Buy a lure from Guppie" (1683285341); "Different lures do different things. Some increase your casting range while others attack a specific kind of fish." (641677831); "Since I gave you some lures, would you mind helping me scout out a fishing spot?" (2334549144) | unknown (2334549144 says lures were given) | 2224144896 / 2711371647 | direct | [client] |
| 3 | **Fisherman: Find Your Own Spot!** | unknown | Sacred Grove Shallows † (Darklit Lagoon is the other candidate) | unknown | † "Find a good fishing spot in Sacred Grove Shallows." (4023541801); "Now that you've found a good spot for fishing, reel in some slugmud skipper." (188116452) / "chipsen fish" (3208103027, 1612800194); "I've found the perfect spot! ... Where you cast your line determines what kinds of fish you can catch." (3130568525). Rival reading †: Ernest "isn't pleased with you showing up in his fishing spot" (97962021) | unknown | 277440424 / 2174454942 | † | [client] |
| 4 | **Fisherman: Stronger Equipment** | Anna Mudskipper † | Sacred Grove Shallows | unknown | † "Anna Mudskipper would like you to retrieve her best fishing equipment from Muggup." (1534748619); "I'm going to need a strong pole ... I let Muggup borrow my best equipment." (626514487); "Find Anna's fishing supplies somewhere near the shore." (3128385732); "Take the fishing equipment back to Anna Mudskipper." (2257928300) | unknown | 1766915747 / 161844165 | † | [client] |

Unprefixed Shallows title, probably this cluster: **Casting Out For Fishing** (2967656271, no ugdt). † Muggup: "Get some perfect casts" (1324197664), "Get a perfect cast" (2484361493), "Did you get a perfect cast yet? Don't give up!" (2502209390), "Report your casting success to Muggup at Sacred Grove Shallows." (4263989381). Giver, level and reward unknown. [client]

### Cluster B - Jonah Relicreel's bait (Sacred Grove Shallows)

The wiki lists **Itty Bitty Bait** as a Fisherman quest from Jonah. [wiki] All five titles below fit Jonah's three bait steps (small fish, butter flyfish, baconfish); which title is which is not proven.

| # | Quest (client title) | Giver | Location | Prerequisite / level | Objectives and text (client string ids) | Rewards | Locale id(s) | Match | Sources |
|---|---|---|---|---|---|---|---|---|---|
| 5 | **Itty Bitty Bait** (no prefix) | Jonah Relicreel [wiki] | Sacred Grove Shallows | none seen | "If opposites really do attract, then Jonah should be able to use some small fish to lure in larger ones. Slugmud skippers would make the perfect tiny bait!" (2954385707); "Reel him in a few itty bitty fish to use as bait." (3354165782); "Take the slugmud skippers back to Jonah Relicreel." (1282516933); "Did you catch any slugmud skippers yet?" (1143322939) | unknown | 2512349121 / 255813548 | direct | [client] [wiki] |
| 6 | **Much Bigger Bait** (no prefix) | Jonah Relicreel † | Sacred Grove Shallows | After 5 (inferred) | "Those tiny slugmud skippers swam right off my hook! ... I'm going to have to use bigger bait ... Nothing's fatter than a butter flyfish." (3185937380) | unknown | 1761706035 / 2345891884 | † | [client] |
| 7 | **Fisherman: Bigger is Better** | Jonah Relicreel | Sacred Grove Shallows | After 5 (inferred) | "Jonah can't keep bait on his hook! Something bigger might be better. He thinks a few butter flyfish should do the trick." (1138155636); "Catch some butter flyfish for Jonah to use as bait." (2534315375); "Take the butter flyfish back to Jonah Relicreel." (2331909717); "Be sure you have at least six of them" (2683818190, extra large) | unknown | 3611566187 / 1268993754 | direct | [client] |
| 8 | **Better Bacon Bait** (no prefix) | Jonah Relicreel | Sacred Grove Shallows | After 7 (inferred) | "Catch some large baconfish to help Jonah make better bait." (614938190); "Reel in large Baconfish" (817893594); "Take the big baconfish back to Jonah Relicreel at Sacred Grove Shallows." (2855761643); "at least ten" (2468067908) | unknown | 2332317412 / 190453112 | direct | [client] |
| 9 | **Bacon Bait** (no prefix) | Jonah Relicreel | Sacred Grove Shallows | After 7 (inferred) | "Help Jonah make irresistible bait by reeling in some baconfish." (513506823); "Jonah has tried nearly every type of bait out there, but he still cannot catch the big one ... baconfish is the answer!" (1122892346); "Take the tasty baconfish back to Jonah Relicreel" (2091401093); "extra large baconfish ... at least eight" (1705496909) | unknown | 3452590829 / 1277114033 | direct | [client] |

### Cluster C - Vick Sapshot and Callie Kelpcake (Sacred Grove Shallows)

| # | Quest (client title) | Giver | Location | Prerequisite / level | Objectives and text (client string ids) | Rewards | Locale id(s) | Match | Sources |
|---|---|---|---|---|---|---|---|---|---|
| 10 | **Fisherman: Picky Palettes** | Vick Sapshot | Sacred Grove Shallows campground | unknown | "Campers can be such picky eaters! They refuse to eat anything but cheery salmon, so Vick Sapshot has asked you to fish a few up." (392766723); "Reel in some cheery salmon in Sacred Grove Shallows for Vick to feed to his campers." (2694070175) | unknown | 2050871502 / 1330900611 | direct | [client] |
| 11 | **Fisherman: Beware of Sharks!** | Vick Sapshot | Sacred Grove Shallows | unknown | "Vick Sapshot is worried that rumors of a shark are scaring away all of his campers. He's asked you to reel in the fish that's causing all of the trouble." (4073302592); "Use your fishing skills to reel in the shark that's scaring away Vick's campers." (3973639438); "Take the shark to Vick Sapshot." (4234514393). Video shows a Tangletooth Shark catch in the Shallows (`0LCKMQz3zMs/00110`) | unknown | 3329223754 / 4259329610 | direct | [client] [video] |
| 12 | **Fisherman: What Is It?!** | Vick Sapshot / Reed † (other candidates: Melvin, Karl) | Sacred Grove Shallows to Stillwater Crossing † | After 11 † | † "Vick isn't sure what you reeled in, but he thinks that Reed Stillwater might have a clue." (3343038694); "I told you, I have no idea what kind of shark you caught, but Reed Stillwater might." (1510873938); "See if someone can identify your catch." (163608836). Rival: Melvin "wants you to catch something he's never seen before." (2362502917) | unknown | 1563901977 / 1445326379 | † | [client] |
| 13 | **Fisherman: Good Enough to Eat!** | Reed Stillwater / Merri De'Flor † | Sacred Grove Shallows campground | unknown | † "Reed Stillwater wouldn't want you to waste such a delicious catch! He's asked you to deliver the butter flyfish to Vick Sapshot at the camping grounds" (786112746); "Take the fish to Vick Sapshot" (1019628643, 3380660720); "I bet my buddy Vick Sapshot would love to grill that" (620657836). Rival: "show your fish to Merri De'Flor in Lakeshore. She would know if it would taste good or not." (628974987) | unknown | 1446607819 / 319747346 | † | [client] |
| 14 | **Fisherman: Fresh Fish Special** | Callie Kelpcake † | Sacred Grove Shallows | unknown | † "Callie may be quite the chef, but she doesn't excel when it comes to fishing. She'd love to serve a fresh catch of the day, but needs your help reeling in some local fresh fish." (3475541213); "Take your catches back to Callie Kelpcake at Sacred Grove Shallows." (3124768930) | unknown | 3284614772 / 647619036 | † | [client] |

### Cluster D - Gunther at Brambleback's Bayou (Briarwood)

Reed shows the way: "Can you take this crate to Brambleback's Bayou? If you tell Reed Stillwater it's for Gunther, he should show you the way." (2651529106); there is also a boat in the stream by Brambleback's farm (3890617641). Gunther's strings give an order through "now that" lines, shown in the prerequisite column. [client]

| # | Quest (client title) | Giver | Location | Prerequisite / level | Objectives and text (client string ids) | Rewards | Locale id(s) | Match | Sources |
|---|---|---|---|---|---|---|---|---|---|
| 15 | **Fisherman: Hungry Chugs** | Gunther | Brambleback's Bayou | Entry to the Bayou via Reed | "Gunther can't keep up with the appetites of chugawug soldiers. He's going to need some help catching fish if he hopes to get them all fed." (2312632048); "Help Gunther reel in enough fish to feed the chugawug soldiers for the day." (1192989159); "Catch some large Fanged Grouper so you can feed the hungry Chugawugs!" (220778717); "Take the large fish back to Gunther." (2240696687) | unknown | 31742645 / 1875532559 | direct | [client] |
| 16 | **Fisherman: Fishy Fungus** | Gunther | Brambleback's Bayou | After Ernest's rumors (4062211766 mentions Ernest "ruining your reputation") | "Prove to Gunther that you're an accomplished fisherman by reeling in some clumps of funky fungus from the Bayou's waters." (803603798); "Reel in Clumps of Fungus" (485814472); "Take the funky fungus clumps to Gunther at Brambleback's Bayou." (1220599649); "Take the fungus to Reed Stillwater to see if he can help identify the plant growth." (985992331) | unknown | 2361775528 / 3837772952 | direct | [client] |
| 17 | **Fisherman: Ink Cod Ink** | Gunther, then Reed Stillwater | Darklit Lagoon (catch) and Reed (turn in) | After 16 | "Reed Stillwater claims that if you bring him some large ink cod found at Darklit Lagoon, that he can safely extract their ink." (2941977733); "Reel in some large Ink Cods so that you can use their stinky ink to get rid of the fungus in Brambleback's Bayou." (3494475770); "Take the large ink cod to Reed Stillwater at the fishing job center." (690281943) | unknown | 1551424341 / 2552104441 | direct | [client] |
| 18 | **Fisherman: Restocking the Stream** | Gunther | Brambleback's Bayou, fish from Darklit Lagoon | After 17 ("Even with the fungus gone ...", 1893407786) | "Gunther thinks that stocking the water with small fish will lure the big ones back." (1893407786); "I still need those small fish to help restock the Bayou. If you cast your line in Darklit Lagoon, you should find plenty." (1238489015); "Take the small fish back to Gunther at Brambleback's Bayou." (2010241866) | unknown | 1958796589 / 4007798221 | direct | [client] |
| 19 | **Fisherman: Undercover** | Gunther | Brambleback's Bayou | unknown | "Gunther needs you to be sneaky in order to find out what those other fisherment are up to. He can create you a nice disguise if you bring him some Bitter Beta." (2756979809); "Reel in some Bitter Beta so that Gunther can make you a disguise from their scales." (1206772153); "Take some leftover fish to Dixie and see if she'll tell you what those other fishermen are up to." (37912213); "Inform Gunther that Dixie and the other fishermen won't talk." (2923357406) | unknown | 3950823086 / 2104310213 | direct | [client] |
| 20 | **Fisherman: VIP Fish** | Gunther † | Brambleback's Bayou | unknown | † "Gunther wants you to try and catch a Golden Scaled Nettler in Brambleback's Bayou." (4225580413); "The Golden Scaled Nettler is a fabled fish! ... The fisherman who catches it will be showered in luck and riches" (2938007331); "Reel in a Golden Scaled Nettler" (1132288440) | unknown | 3818943794 / 3523108011 | † | [client] |
| 21 | **Fisherman: Say Cheese!** | Gunther, then Feldman | Sanctuary (Feldman) | After 20 † | "Gunther wants you to show off your catch to Feldman in Sanctuary. You might even get your picture on the cover of Sacred Grove Fish Pro magazine!" (3453890318); "Take your catch to Feldman in Sanctuary so he can take a picture with you and your Golden Scaled Nettler!" (3769055955) | unknown | 2902383387 / 1381408625 | direct | [client] |

### Cluster E - Dooba, Estelle and Darklit Lagoon (Briarwood)

| # | Quest (client title) | Giver | Location | Prerequisite / level | Objectives and text (client string ids) | Rewards | Locale id(s) | Match | Sources |
|---|---|---|---|---|---|---|---|---|---|
| 22 | **Fisherman: Dooba's Secret** | Karl Kippersun † points the player to Dooba | Nettleseed (Briarwood) | unknown | "Karl claims that Ernest only catches big fish because he uses Dooba's special bait. If you ask nicely, Dooba might give you some too." (3027758188); "See if Dooba will let you have some of his special bait." (3231918655); "Visit Dooba in Nettleseed and see if he'll let you have some of his bait." (3570244979) | unknown | 32713170 / 3523351710 | direct | [client] |
| 23 | **Fisherman: Live Bait!** | Dooba † | Nettleseed † | After 22 † | † "You want to know the secret to my success? I use a mixture of moldy cheese, grubs, and live Nettletrout! If you catch a few, I can make you some bait." (730127507); "Catch enough live Nettletrout before the timer runs out." (3449403952); minigame "Live Bait!" (2203871487). Rival: Ernest rescue, "can only be lured in with live bait" (1500179458) | unknown | 3169546093 / 4245194083 | † | [client] |
| 24 | **Fisherman: Taking the Bait** | Dooba † | Darklit Lagoon | After 23 † | † "Dooba is convinced that with his new batch of bait you'll be able to reel in an even bigger fish than before." (3251360113); "See what you can do with Dooba's bait at Darklit Lagoon." (3960575324); "Reel in the biggest purplenosed shark you can at Darklit Lagoon." (940256353) | unknown | 1181072204 / 3198130757 | † | [client] |
| 25 | **Fisherman: A Gift For Estelle** | Dooba | Darklit Lagoon (Estelle LaFrell) | After 24 (Estelle complains about the smell, 1008211882) | "Would you give Estelle this gift for me as a way of saying sorry?" (557589592); "Take Dooba's gift to Estelle LaFrell at Darklit Lagoon." (53509434); "Deliver a gift to Estelle" (2210684110) | unknown | 308216041 / 5027847 | direct | [client] |

### Cluster F - Lakeshore, Rainbow Lake and the McScuttle family

Merri De'Flor's fish-dish quests are Chef content, see [[Chef]]. [client]

| # | Quest (client title) | Giver | Location | Prerequisite / level | Objectives and text (client string ids) | Rewards | Locale id(s) | Match | Sources |
|---|---|---|---|---|---|---|---|---|---|
| 26 | **Fisherman: Bluerose's Best Bait** | Merri De'Flor † | Lakeshore | before 27 | "You need to be properly prepared to fish at Rainbow Lake! Merri De'Flor suggests getting some bait from Bluerose before heading out." (1333262692); "Talk to Bluerose in Lakeshore and see if she'd be willing to part with some of her special bait." (2930920091); "Get some bait from Bluerose in Lakeshore before you head out to Rainbow Lake." (3803238666) | unknown | 3645450289 / 2335275176 | direct | [client] |
| 27 | **Fisherman: Fish Fry** | Merri De'Flor | Lakeshore, fish at Rainbow Lake | unknown | "Merri wants you to catch a bunch of fish for the fish fry." (1687374735); "It's too bad the local fishermen can't catch anything from Rainbow Lake. I'm hosting a fish fry big enough to feed all the tourists in Lakeshore" (1771171212); "Catch some peachy perch at Rainbow Lake." (2399471352); mayor's extra large Peachy Perch (4278415298, 1032719374) | unknown | 1096441424 / 2519471726 | direct | [client] |
| 28 | **Fisherman: Fish Market** | Merri De'Flor | Lakeshore | After 27 ("we managed to have a few Peachy Perch left over", 147880831) | "Merri has a bunch of left over fish and doesn't want it to go to waste. See if anyone around Lakeshore is willing to buy it." (388540569); "See if Yoko would like to buy some fresh fish." (2803296949); "See if Mister Nooklings would like to buy some fresh fish." (3341966803); "Let Merri De'Flor know that no one wants to buy your fish, thanks to Ernest's lies." (1160414925) | unknown | 200616204 / 3821202652 | direct | [client] |
| 29 | **Fisherman: An Offering** | Waterwings † | Lakeshore (Lakeshore Fishing Village) | unknown | † "Waterwings is a renowned fisherman ... Bring him an amythest to replace it, and maybe he will let you rub his lucky stone for a reward!" (560949661); "Bring Waterwings an amythest." (3789798120) | unknown ("a reward" is unspecified) | 3719837526 / 1187024573 | † | [client] |
| 30 | **Fisherman: McScuttle Family Farm** | Merri / the buyers † | McScuttle Family Farm at Rainbow Lake | After 28 † | "It's time to put a stop to Ernest's badmouthing. Head to the McScuttle family farm at Rainbow Lake to find out where Ernest is." (1527356230); "I'm not telling you where Ernest is until you fish up my socks and boots!" (396930357); "Fish up Katie McScuttle's old boots." (1523245831), "soggy socks" (376306261) | unknown | 2241805448 / 1384302499 | direct | [client] |
| 31 | **Fisherman: Setting Ernest Straight** | Kati McScuttle | Sacred Grove Shallows tournament | After 30 | "Kati McScuttle says you can probably find her brother Ernest at the Sacred Grove Shallows fishing tournament. Head in there and set Ernest straight!" (2610015718); "See if you can find Ernest at the fishing tournament at Sacred Grove Shallows." (1926341177) | unknown | 3317169556 / 3713184071 | direct | [client] |
| 32 | **Fisherman: One Spot Left** | tournament host † | Sacred Grove Shallows | After 31 † | † "Catch the Tickled Trout to win a spot in the fishing tournament!" (1149157092); tournament rules "The first fisherman to reel in all the fish wins!" (3721820016, 2741502183); "Reel in a large Finless Fish, a large Calico Catfish, and a medium Toothy Tetra" (2901056583) | Winner prize: Snowhill getaway (see 33) | 222852317 / 2530249675 | † | [client] |
| 33 | **Fisherman: Snowhill Getaway** | tournament host, then Murphy | Snowhill Hot Springs | After 32 | "You've won a relaxing Snowhill getaway for taking first place in the fishing tournament! Meet up with Murphy in Snowhill to start your vacation." (277601712); "We have a winner, and for once it's not Ernest! You're going to love your prize." (3678253789) | A Snowhill getaway: "an all expenses paid snowy retreat" at the Hot Springs, offered by Murphy (3660244810); items unknown | 2747966825 / 1748570026 | direct | [client] |

### Cluster G - Big J's crew at Wintery Basin (Snowhill)

| # | Quest (client title) | Giver | Location | Prerequisite / level | Objectives and text (client string ids) | Rewards | Locale id(s) | Match | Sources |
|---|---|---|---|---|---|---|---|---|---|
| 34 | **Fisherman: Joining Big J's Crew** | Big J | Snowhill, supplies from Smitty | After Snowhill Getaway † | "Big J could really use an expert fisherman on his team. Before you can join the crew up at the Wintery Basin, you'll need to pick up some supplies from Smitty." (2434367310); "Let Big J know you have all the supplies you need." (666380660); "Big J thinks you're ready to join his fishing crew! head to the Wintery Basin and get started." (1801317899) | unknown | 3204709185 / 2519333855 | direct | [client] |
| 35 | **Fisherman: Partner Up** | Ernest McScuttle | Wintery Basin | After 34 | "Ernest says you need to partner up with Hilbert. Two fishermen are better than one, apparently!" (1786706482); "Sometimes the fish out here are stronger than the fisherman. A novice like you needs a partner ..." (2458992729); "Talk to Hilbert" (3829846926) | unknown | 672407544 / 2964345589 | direct | [client] |
| 36 | **Fisherman: Plenty of Fish** | Helena | Wintery Basin | After 35 † | "Helena doesn't like working with beginners. Hilbert has put the fishing crew behind on their quota, and she hopes an experienced fisherman like you will help pick up his slack." (2737356113); "Help Helena meet her quota by reeling in some local fish." (3076124323); "Take your catches back to Helena in the Wintery Basin." (1942846786) | unknown | 3182666508 / 1419650360 | † | [client] |
| 37 | **Fisherman: Fish Food** | Hilbert † | Wintery Basin | unknown | † "Catch some blubracuda and feed it to the brown bears around the Wintery Basin." (2639689036); "The only thing that will keep them away is a blubracuda." (1794152498); "Time to let Hilbert know you've reeled in all the Blubracuda you could" (3335357992) | unknown | 1605025653 / 1396542932 | † | [client] |
| 38 | **Fisherman: Fishing Frenzy** | Trina Turtledove † | Wintery Basin | unknown | † "Travel to Wintery Basin here in Snowhill ... catch enough fish so that you smash the season's current fishing record." (2210309464); "Ice fishing in Wintery Basin is a time-honored tradition ... The most anyone has caught so far this season is 5 fish" (3391216071). These strings may belong to the Snow Days quest "Day 5: Wintery Fishing" (397680377) instead | unknown | 3824034041 / 2858840631 | † | [client] |
| 39 | **Fisherman: To the Rescue!** | Helena | Wintery Basin | After the quota quests † | "Ernest has been swallowed by an icy sea creature! Helena hopes a large Spineless Stickleback will lure the creature to the surface" (69065426); "Cast your line in the water until you catch the icy beast that swallowed Ernest!" (1966701847); "You caught the Ferocious Fangler that swallowed Ernest McScuttle!" (3472420756); "Take the Spineless Stickleback to Helena." (712795590) | Ernest's thanks; "the other half of the treasure map" (4267294910) | 1050206127 / 20799686 | direct | [client] |
| 40 | **Fisherman: Reel Riches** | Hilbert / Helena † | Wintery Basin | After 39 † | † "It turns out the fishermen here are trying to reel in a little more than fish. There is treasure in these waters! Hilbert claims Helena knows all the details." (3857941608); "Talk to Helena in the Wintery Basin to learn more about the sunken treasure." (1858645937) | unknown | 117871878 / 3546430374 | † | [client] |
| 41 | **Fisherman: Sunken Treasure** | Helena | Wintery Basin | After 40 † | "Supposedly Helena knows the lore and legends behind some sunken treasure." (2337545016); "Reel in the sunken treasure" (2345226722); "Talk to Helena in the Wintery Basin and let her know you've reeled in the sunken treasure!" (2430272201) | unknown | 2827127973 / 3897195439 | direct | [client] |
| 42 | **Fisherman: Buried Treasure** | Ernest † | Wintery Basin, chest opened by Smitty | After 41 † | † "Remember what Ernest said and see if you can find the treasure!" (399581790); "Return to Ernest in the Wintery Basin so he can open the treasure chest for you." (2698778678); "You should take that treasure chest to Smitty in Snowhill and see if he can open it." (1714672652); "Fisherman Treasure Chest" item (3927392722) | Treasure chest contents unknown | 3896600965 / 1642535845 | † | [client] |

### Cluster H - Lenore's family at Frostbitten Banks (Snowhill)

| # | Quest (client title) | Giver | Location | Prerequisite / level | Objectives and text (client string ids) | Rewards | Locale id(s) | Match | Sources |
|---|---|---|---|---|---|---|---|---|---|
| 43 | **Fisherman: My Hut is Your Hut** | Murphy †, then Lenore | Snowhill to Frostbitten Banks | After 33 and 34 (inferred; the suites are "booked" for Big J's crew, 2309992678, 1601373522) | "I hate to leave you out in the cold, but I'm afraid all the rooms in town are full. I guess there's always old lady Lenore. She has a cabin out near Frostbitten Banks and loves visitors. Chip Numbwing can show you the way there" (216811999); "Hopefully Lenore at Frostbitten Banks has a place for you to stay!" (2418676270) | A place to stay; items unknown | 4291056686 / 3983317068 | † (title) | [client] |
| 44 | **Fisherman: Feed the Penguins** | Lenore | Frostbitten Banks to the Penguin Training Institute, Snowhill | After 43 † | "Lenore ... completely forgot about the Blue Thornfin fish she needed to deliver to Valinda! You'll help an old lady out and reel some in" (2937628843); "Catch a bunch of Blue Thornfin from the Frostbitten Banks for Valinda's penguins." (1569613026); "Feed Valinda's penguins for her!" (1012420051) | unknown | 1693119961 / 729858016 | † | [client] |
| 45 | **Fisherman: Showing Off Your Skills** | Melvin † | Frostbitten Banks | After 43 † | † "Melvin wants you to show him how to get perfect casts!" (4012210695); "Show Melvin a thing or two about fishing at the Frostbitten Banks." (1314863705); "Let Lenore know that Melvin caught his first fish." (3464306105) | unknown | 1531773669 / 3633172799 | † | [client] |
| 46 | **Fisherman: The "Big" One** (client text `&quot;Big&quot;`) | Melvin † | Frostbitten Banks | After 43 † | † "Melvin wants you to catch a fish that's bigger than his Aunt Mildred's head!" (1949907475). Rival: Ernest's biggest creeping cod contest, "The true test of skill is in how big of a fish you can catch!" (3381448989, Karl judges) | unknown | 2246774640 / 3510437043 | † | [client] |
Rows 1 to 46 hold the 42 "Fisherman:" titles; rows 5, 6, 8 and 9 are unprefixed Jonah titles and E1 to E4 are unprefixed tutorial titles. "What Is It?!" (row 12) has two plausible homes, the Shallows shark and Melvin's request (2362502917, "Melvin wants you to catch something he's never seen before."); the client has one entry for it.

### Level quests and achievements

| Title | Objective text | Locale id(s) | Wiki reward | Sources |
|---|---|---|---|---|
| Fisherman: Level 4 | "Reach Level 4 as a Fisherman." (25725524) | 2589480019 ucdt | Bronze Trophy, 5 points | [client] [wiki Achievements] |
| Fisherman: Level 8 | "Reach Level 8 as a Fisherman." (1811147096) | 4183064544 ucdt | Bronze Trophy, 5 points | [client] [wiki Achievements] |
| Fisherman: Level 12 | "Reach Level 12 as a Fisherman." (3107436134) | 2934868057 ucdt | Silver Trophy, 10 points | [client] [wiki Achievements] |
| Fisherman: Level 16 | "Reach Level 16 as a Fisherman." (1470096382) | 2080787357 ucdt | Silver Trophy, 10 points | [client] [wiki Achievements] |
| Fisherman: Level 20 | "Reach Level 20 as a Fisherman." (140915077; 1242406917 without the period) | 2323556482 ucdt | Gold Trophy, 15 points; Title: The Angler | [client] [wiki Achievements, Titles] |

The five level titles are bare `ucdt` entries with no `ugdt` twin, like the other jobs. The client also holds "Achieve Level 5 as a Fisherman" (547562084, 1009520043), "Achieve Level 10 as a Fisherman" (3905333529) and "Achieve Level 15 as a Fisherman" (2727861305), which no title names. The achievement titles are "Title: Angler" (3966962646) and "Title:  The Angler" (4253955498); "Aspiring Angler" (3357259481), "Fishing Fan" (1300861805), "Fishing Expert" (1075832935) and "Title: Flyfisher" (2544675408) exist as titles of unknown origin. [client]

### Related fishing quests outside the "Fisherman:" prefix

| Quest | Giver | Where | What is known | Sources |
|---|---|---|---|---|
| **Keep it Clean** (321518181 / 1923184725) | Melanie | Sacred Grove Shallows | Offer: "The fish of Sacred Grove Shallows would thank you if they could! Those who help keep the waters clean don't go unrewarded." (851187120). Goals: "Reel in Plastic Bags" (3896657549), "Reel in Rusty Cans - 2/3" (852965110), "Return to Melanie". Reward not shown; completion text "Please keep fishing out any rusty cans or plastic bags" (2108110159) | [client] [video `0LCKMQz3zMs/00292`, `0LCKMQz3zMs/00246`, `0LCKMQz3zMs/00262`] |
| **Shallows: Catch of the Day** (3795167698 / 3363703546) | Melanie is the nameplate beside the completion frame; Callie Kelpcake † | Sacred Grove Shallows | Goals "Catch various fish from Sacred Grove Shallows." (3418008221): extra large Flutterfish, large Cheery Salmon, extra large Butter Flyfish, large Chipsen Fish (video). **Reward on screen: "You receive 30 stars and 16 coins"** (the Power Hour 2x coins banner is up in neighbouring frames, so the coin figure may be doubled) | [client] [video `0LCKMQz3zMs/00298`, `0LCKMQz3zMs/00304`] |
| **Bayou: Catch of the Day** (3284062000 / 3806006381), **Basin: Catch of the Day** (1450129131 / 536569294) | unknown | Brambleback's Bayou; Wintery Basin | Same pattern as the Shallows one, titles only. "Deliver the catch of the day!" (2370875296) | [client] |
| **Fester's Fishing Challenge** (1055628265; "NOT LIVE" copy 538396331 / 3155925240) | Fester | unknown | Timed fish-off: "reel in six butter flyfish and return to Fester before time runs out to earn a reward" (163816406) against "Catching four butter flyfish in less than five minutes" (4221070183) | [client] |
| **Sacred Grove Fishing Derby** (3092160053), **Community Event: Fishing Derby** (457693766) | event | Sacred Grove Shallows | "Make your way to the fishing tournament area in Sacred Grove Shallows." (489796872). The wiki lists a Snow Days fishing party at Wintery Basin | [client] [wiki Community Events] |
| **Day 5: Wintery Fishing** (397680377), **Snow Day: Ice Fishing** (3725735621) | Trina Turtledove | Wintery Basin | "Trina Turtledove of Snowhill wants you to participate in one of the most popular Snowhill tourist activities - fishing!" (887132585). Possibly the Snow Days home of row 38's strings | [client] [wiki Snow Days] |
| **Brugo's Fishing Spot** (409840810 / 3465147268) | n/a | Forest Troll area, Brawler content | A duel instance, not fishing: "Selfish Brugo won't share this prime fishing area with anyone! Go knock some sense into Brugo!" (3093646501). See [[Brawler]] | [client] [video `bLxqx_aCizY/00112`, `bLxqx_aCizY/00178`] |
| **Robgoblin Creek!** battle | Robgoblin Fishmonger, northwest Wilds | Wilds | "Robgoblins are running amok at this popular local fishing spot! Kick them out!" (801423612). See [[Robgoblin Creek]] | [client] [wiki Robgoblin Creek!] |
| **Selvi Slowreel's Prized Fish** (2909264631) | Selvi Slowreel | Lakeshore, Mushroom Cavern | Wants a Spotted Trufflefish; toxic truffles as bait from Mushroom Cavern (358456730, 2029586513). See [[Mushroom Cavern]] | [client] |
| **Poco Pogo's nets** | Poco Pogo | Seaside, Cray Caves | "Find Poco Pogo's strong fishing net" (1754162425); "Enter the Cray Caves near Seaside and recover Poco Pogo's fishing net." (589517572). Whether the "Strong Fishing Net" item (3309065077) is a reward is unknown. See [[Cray Caves]] | [client] |
| **Fetch quests whose titles were not found** | Archie Tingletoes (calico catfish, Rainbow Lake, 1713567318); Calvin Codscale (pacu pacu, Frostbitten Banks, 314427879); Lenny (roach and coach loach, Darklit Lagoon, 2900883901); Kim McScuttle (toothy tetra for cats, Rainbow Lake, 3501614680); Derek (cheery salmon bouquet for Kim, 1847394871); Estelle LaFrell (thorny trout, 1129551213); Anna Mudskipper (large butter flyfish, 458972245); Emil Jadefish (release Lakeshore fish in Shrouded Glade, 1558693511); Laggosh (trade shiny metal for fish, 1384681019); Waterwings (feed ducks spicy fish bait, 281790141) | various | They may share generic titles such as Fresh Fish Special or Plenty of Fish; unproven | [client] |

## Fishing spots

Six spots. The client lists the fish for each in the "Grab your fishing gear and get ready to reel 'em in!" strings; the wiki agrees on every list except Brambleback's Bayou (see Conflicts). The wiki calls each "a Difficulty 1 fishing game". The number after each fish is the wiki "Catchable at level" gate; the client confirms gating with "You need to level up before you can catch this fish!" (1580530816). [client] [wiki]

| Spot | Zone and access | Fish (client list) with wiki level gate | Sources |
|---|---|---|---|
| **Sacred Grove Shallows** | Stillwater Crossing; boat behind Reed Stillwater. Also the tournament venue and the camp of Vick Sapshot | Flutterfish 1, Slugmud Skipper 1, Tickled Trout 1, Butter Flyfish 5, Cheery Salmon 10, Chipsen Fish 14, Feral Catfish 17, Baconfish 20, Tangletooth Shark 20 (489842996) | [client] [wiki Sacred Grove Shallows] [video: Cheery Salmon, Feral Catfish and Tangletooth Shark catches `0LCKMQz3zMs/00110`] |
| **Rainbow Lake** | Lakeshore; boat from Selvi Slowreel on the docks | Flutterfish 1, Calico Catfish 1, Tickled Trout 1, Toothy Tetra 5, Peachy Perch 10, Finless Fish 14, Lady Tetra 17, Tangletooth Shark 20 (2642629420) | [client] [wiki Rainbow Lake] |
| **Brambleback's Bayou** | Briarwood; via Reed with a crate for Gunther, or the boat by Brambleback's farm | Creeping Cod 1, Bitter Betta 1, Golden Scaled Nettler (client; wiki lists no level), Globfish 1, Blind Swurglefish 5, Changed Salmon 10, Fanged Grouper 14, Briar Nibbler 17 (2008084678). Wiki repeats Bitter Betta at level 20 | [client] [wiki Brambleback's Bayou] |
| **Darklit Lagoon** | Briarwood; Lila Darklure's boat, Nettleseed area | Ink Cod 1, Creeping Cod 1, Golden Scaled Nettler 1, Thorny Trout 5, Old Sole 10, Purplenosed Shark 14, Roach Loach 17 (2213265004) | [client] [wiki Darklit Lagoon] [video: Lila Darklure `ag1LtV5-oho/00006`] |
| **Frostbitten Banks** | Snowhill; boat, "the Queen's favorite fishing spot" (863439235); Lenore's cabin is near | Chilly Grouper 1, Winter Piranha 1, Frostgill Smelt 5, Pacu Pacu 5, Blue Thornfin 10, Spineless Stickleback 10, Goofy Grouper 14, Talking Bass 17, Ferocious Fangler 20, Plattypus 20 (1772163183) | [client] [wiki Frostbitten Banks] |
| **Wintery Basin** | Snowhill; ice holes, brown bears, "has the most bites!" (401493682); Big J's crew | Frozen Char 1, Winter Piranha 1, Frostgill Smelt 5, Spineless Stickleback 10, Blubracuda 14, Coach Loach 17, Ferocious Fangler 20 (4128541831) | [client] [wiki Wintery Basin] |

The gates follow one ladder on every spot: 1, 5, 10, 14, 17, 20. [wiki] Collections tied to fishing: "Briarwood Pond Fish" (2607923565), "Briarwood Stream Fish" (2707821362), "Snowhill Pond Fish" (3143710812), "Snowhill Stream Fish" (2975698759), "Wilds Pond Fish" (1195446770), "Wilds Stream Fish" (2499355938), "Extra Large Catch of the Day" (3192545933, "A collection of extra large fish.", 1498725963), "Extra Large and in Charge" (147008632). Which fish belongs to which collection is unknown. [client] [wiki]

## Fish caught

40 fish. "Lure" is the lure that gives a 10% better chance, from the client lure descriptions (by fish triple) and from the client "The X likes the Y!" lines where they exist; the lure's name comes from the client line where one exists and otherwise from the wiki. Size classes Small, Medium, Large and Extra Large exist as separate client item names (`sg_fish_catch_small_bbe` 3633727465, `_medium_bbe` 3645144087, `_large_bbe` 4001028658, `_xlarge_bbe` 3500845140). The "Sizes" column lists the sizes found as client item names; an absent size is not proof the size cannot occur. [client]

| Fish | Spots (wiki gate) | Lure | Sizes in client | Sources |
|---|---|---|---|---|
| Baconfish | Shallows (20) | French Fry | M L XL | [client 4019347670] |
| Bitter Betta | Bayou (1) | Sleek Clicker | S M | [client] [wiki] |
| Blind Swurglefish | Bayou (5) | 16oz Steak | S M L | [client 794966760] |
| Blubracuda | Basin (14) | French Fry | S M L | [client] [wiki] |
| Blue Thornfin | Banks (10) | Thorn Jig | M L XL | [client 3983215875] |
| Briar Nibbler | Bayou (17) | Flyfisher 3000 | L XL | [client 41214024] |
| Butter Flyfish | Shallows (5) | Flyfisher 3000 | M L XL | [client 2103224641] |
| Calico Catfish | Rainbow Lake (1) | Tiny Rib | M L | [client 3275375471] |
| Changed Salmon | Bayou (10) | French Fry | M L | [client] [wiki] |
| Cheery Salmon | Shallows (10) | Sleek Clicker | S M L | [client] [wiki] [video catch card `0LCKMQz3zMs/00268`] |
| Chilly Grouper | Banks (1) | Flyfisher 3000 | S M L XL | [client 834526786] |
| Chipsen Fish | Shallows (14) | Thorn Jig | M L | [client 429680893] |
| Coach Loach | Basin (17) | Shiny Crankbait | S M L | [client] [wiki] |
| Creeping Cod | Lagoon (1), Bayou (1) | Shiny Crankbait | M L | [client] [wiki] |
| Fanged Grouper | Bayou (14) | Mega Slider | M L XL | [client] [wiki] |
| Feral Catfish | Shallows (17) | Shiny Crankbait | L XL | [client 1277837023] [video: Large 13.60 and Extra Large 20.39 catch cards `0LCKMQz3zMs/00048`, `0LCKMQz3zMs/00036`] |
| Ferocious Fangler | Banks (20), Basin (20) | none ("does not find any lure appetizing", 2013843572) | none found | [client] [wiki] |
| Finless Fish | Rainbow Lake (14) | Mega Slider | S M L | [client 1103744427] |
| Flutterfish | Shallows (1), Rainbow Lake (1) | Frostflies | S M L XL | [client 230034789] |
| Frostgill Smelt | Banks (5), Basin (5) | Frostflies | S M | [client] [wiki] |
| Frozen Char | Basin (1) | Skipper Seeker | S M | [client] [wiki] |
| Globfish | Bayou (1) | Perch Pinpointer | M L XL | [client 3488328578] |
| Golden Scaled Nettler | Lagoon (1), Bayou (client only) | none (3651030718) | none found | [client] [wiki] |
| Goofy Grouper | Banks (14) | Mega Slider | M L XL | [client] [wiki] |
| Ink Cod | Lagoon (1) | Frostflies | S M L | [client] [wiki] |
| Lady Tetra | Rainbow Lake (17) | Nightcrawlers | S M | [client] [wiki] |
| Old Sole | Lagoon (10) | Skipper Seeker | S M L | [client] [wiki] |
| Pacu Pacu | Banks (5) | Perch Pinpointer | M L | [client] [wiki] |
| Peachy Perch | Rainbow Lake (10) | Perch Pinpointer | S M L | [client] [wiki] |
| Plattypus | Banks (20) | Nightcrawlers | L XL | [client] [wiki] |
| Purplenosed Shark | Lagoon (14) | Tiny Rib | M L XL | [client] [wiki] |
| Roach Loach | Lagoon (17) | Nightcrawlers | S M L | [client] [wiki] |
| Slugmud Skipper | Shallows (1) | Skipper Seeker | S M | [client] [wiki] |
| Spineless Stickleback | Banks (10), Basin (10) | Tiny Rib | S M L | [client] [wiki] |
| Talking Bass | Banks (17) | Sleek Clicker | L XL | [client] [wiki] |
| Tangletooth Shark | Shallows (20), Rainbow Lake (20) | none (364583144) | none found | [client] [wiki] [video `0LCKMQz3zMs/00110`] |
| Thorny Trout | Lagoon (5) | Thorn Jig | S M | [client] [wiki] |
| Tickled Trout | Shallows (1), Rainbow Lake (1) | none (3097252158) | none found | [client] [wiki] |
| Toothy Tetra | Rainbow Lake (5) | 16oz Steak | S M | [client] [wiki] |
| Winter Piranha | Banks (1), Basin (1) | 16oz Steak | S M | [client] [wiki] |

Other catches: junk "Fishing Junk" (1387835333): Plastic Bag (1697736224), Rusty Can (4210741307), Old Boot (3018182641, "Reel in an old boot" 3365847474), Soggy Socks (3003591811), sticks of dynamite at Wintery Basin (1920246094); "Fisherman Treasure Chest" (3927392722) and the "Treasure" a Treasure Magnet lure attracts. The video shows Plastic Bag and Rusty Can catch messages (`0LCKMQz3zMs/00272`, `0LCKMQz3zMs/00246`). Contents of the chest, junk rates and any weight or rarity numbers are unknown. [client] [video]

## Gear

### Rods

| Item | Description (client) | Tier per wiki | Sources |
|---|---|---|---|
| Simple Bamboo Fishing Rod (2011292012) | "This fishing rod allows you to cast a short distance." (2926168180) | cast short | [client] [wiki Fisherman] |
| Golden Reel Fishing Rod (3465747698) | same short-cast text | cast short | [client] [wiki Fisherman] |
| Metal Fishing Rod (1514584696; also "Metal Rod" 1924101778) | "This fishing rod allows you to cast a greater distance" (380809544) | cast greater | [client] [wiki Fisherman] |
| Red Scoped Fishing Rod (1161288116) | same greater-distance text | cast greater | [client] [wiki Fisherman] |
| Golden Scoped Fishing Rod (2684056351) | "This fishing rod allows you to fish in the deepest of waters!" (2726421479) | deepest waters | [client] [wiki Fisherman] |

The client names the five rods and the three description strings, but no string pairs a rod with its description; the pairing is the wiki's. Also in the client: "Fishing Pole" (3904006406), "Shiny Bobber" (693034143), "A Bobber" (3453659592), "Strong Fishing Net" (3309065077). Level requirements, prices and the starter rod are unknown. The wiki says tools come from the Coin Shop, quests and treasure chests. [client] [wiki]

### Lures

The wiki says lures are bought from the Coin Shop under supplies; Guppie sells lures in the Shallows. [wiki] [client] Each gives "a 10% higher chance of catching" three fish (client wording varies between "grants" and "gives").

| Lure | Fish | Client description ids | Sources |
|---|---|---|---|
| 16oz Steak (893565369, 2773926980) | Winter Piranha, Toothy Tetra, Blind Swurglefish | 2381670504, 3820692972 | [client] [wiki] |
| Flyfisher 3000 (1244002952, 2252922992) | Chilly Grouper, Butter Flyfish, Briar Nibbler | 1184708990, 3918747329 | [client] [wiki] |
| French Fry (3083264) | Blubracuda, Baconfish, Changed Salmon | 4011784697, 4285509887 | [client] [wiki] |
| Frostflies (832643680) | Frostgill Smelt, Flutterfish, Ink Cod | 2151391554, 3583441429 | [client] [wiki] |
| Mega Slider (1339554551) | Goofy Grouper, Finless Fish, Fanged Grouper | 667549452, 3475855459 | [client] [wiki] |
| Nightcrawlers (3636673450) | Plattypus, Lady Tetra, Roach Loach | 525725934, 3585523660 | [client] [wiki] |
| Perch Pinpointer (96454568; lure 3294545566) | Pacu Pacu, Peachy Perch, Globfish | 817732383, 3901661591 | [client] [wiki] |
| Shiny Crankbait (2430425532; lure 3303689553) | Coach Loach, Feral Catfish, Creeping Cod | 2635579793, 3480897785 | [client] [wiki] |
| Skipper Seeker (3947634520) | Frozen Char, Slugmud Skipper, Old Sole | 2798377239, 3655159277 | [client] [wiki] |
| Sleek Clicker (3254128423) | Talking Bass, Cheery Salmon, Bitter Betta | 83366119, 3800966576 | [client] [wiki] |
| Thorn Jig (3468361485) | Blue Thornfin, Chipsen Fish, Thorny Trout | 750999162, 1687881692 | [client] [wiki] |
| Tiny Rib (2245382931) | Spineless Stickleback, Calico Catfish, Purplenosed Shark | 631467910, 3927898942 | [client] [wiki] |
| Treasure Magnet (321260520) | Treasure | "This magnet attracts treasure 10% more often than other lures." (4006283740) | [client] [wiki] |

Lures are used from the tackle box: "This is your tackle box. Select which kind of bait you'd like to use." (2103509077); "Lures help you catch a specific fish. Interact with a lure to use it." (540422074). Lure stock, prices and unlock levels are unknown. The client also holds "Bucket of Bait" (1627504736) and a "Fishing Lures" name entry (318877188, use unknown). [client]

### Outfits, shards and consumables

| Item | What is known | Sources |
|---|---|---|
| **Fearless Fisherman** set: Hat (2947683184), Vest (600639802), Waders (3180924634), Boots (3972257609) | Names only | [client] |
| **Aggressive Angler** set: Hat (3543660510), Shirt (310512607), Waders (1900117187), Boots (3536260279) | Names only | [client] |
| **Tough Troller** set: Hat (2216062781), Shirt (3607919492), Waders (3139204433), Boots (3486551378) | Names only | [client] |
| Members Only Fisherman Pants (407235252) | Members only | [client] |
| Captain's Fishing Coat (1142530463) | Chest, all jobs, SC Marketplace, 149 SC or 134 SC for members; "Show those fish who's the boss." (2589938451) | [client] [wiki Captain's Fishing Coat] |
| **Shards and accessories** | Slot families: Algae, Backwater, Livebox and Rollcast each as Necklace, Ring and Square Shard; Biomass and Clearwater as Triangle Shard. Tier prefixes seen: Amatuer (sic), Fisher's, Angler's, Troller's, Pro, All-Star. Examples: Fisher's Livebox Ring (105034181), Angler's Rollcast Necklace (3565605294), Troller's Clearwater Triangle Shard (2678289237), Pro Livebox Ring (1482983089), All-Star Biomass Triangle Shard (2703990515). Tier order, stats and levels unknown. Collections "Fisherman Shards" (4157167645), "Fisherman Accessories" (4239768659), "Fisherman Gear" (3388920877) | [client] |
| Fisherman Sticker (260419612, 3740872000) | Name and description entries only | [client] |
| **Stat buffs** | Stat names: Casting Skill (1956734809), Casting Strength (2461281922), Line Strength (451523175), Reeling Speed (3985729543), plus "Fishing Perfect Cast Skill" (3852363318). Consumables: "Increases Casting Skill for 30 mins." (1336769804), "Increases Casting Strength for 30 mins." (3430923859), "Increases Line Strength for 30 mins." (4152924649), "Increases Reeling Speed for 30 mins." (4543442). Chef's Baconfish Bits is "a delicious snack that will make you feel a bit stronger while fishing" (4009066548) | [client] |

## Fishing minigame flow

What the client text and our mapped code tell us. [client] The detailed function list is in [[Fishing]].

1. **Enter.** Interact with a boat or guide at a spot (see Fishing spots). The client loads a zone definition and a player definition from `bbe_FishingZoneConfig.txt` (keyed by an `EncounterDefId`) and `bbe_FishingPlayerConfig.txt` (mapped as "likely"). Turning fishing on saves audio, switches to an underwater mix, registers the local player with a **RegisterPlayerRequest** (sub-type 2) and loads the fishing music (`FishingProcessor::SetActive`, `RegisterLocalPlayer`). All fishing traffic is one packet family, opcode 0x8a with a sub-type in the header. [client code]
2. **Pick bait or a target.** "This is your tackle box. Select which kind of bait you'd like to use." (2103509077); "This is the fish finder. Select what you would like to try to catch." (3562503964). The data tables the client binds are `BaseClient.FishingProcessor.FishTypes` (columns FishType, FishName, FishIcon, FishSpecial, FishCatchable, FishLureRequirement), `FishingLures` and `FishScored` (columns FishName, FishIconId, FishDifficulty, FishRarity, FishSize, FishScore, TimeToCatch). [client code]
3. **Cast.** "Press the left mouse button again to cast your line. Try to match your aim with the target for a perfect cast." (1357510210). "The more accurate your cast is, the better your chances are. If your target cast matches up with the target ring in the water, you'll have a perfect cast!" (1313092581); "Line up your casting ring within the target ring to get a perfect cast!" (3340416373); banner "Perfect Cast!" (1422899335). The client sends a cast-animation request (sub-type 5) and then a **CastRequest** (sub-type 6: handle, four floats for the target, a flag). [client] [client code]
4. **Wait and attract.** UI events "Fishing:StartCasting" (posted by the state machine), "FishingDistanceMeter:FishHooked", "Fishing:EndCasting" and "FishingStatusText:Show" (posted by the receive dispatcher) drive the on-screen meter and prompts. Tips: "Press the left mouse button to tug on your line. It works best if the fish has a question mark over its head." (360414060); "Press and release the left mouse button to attact the fish" (3810042332); "Hold down the left mouse button when the fish is looking at your hook." (3371589180). States shown in text: "Fish Nibbling" (4242821167), "Fish Hooked" (2705304767). Fish approach in "schools"; the packet classes for school spawn, despawn, update and fish run are sub-types 11, 12, 13 and 18, and for bobber spawn and update 8 and 10 (names from our mapped classes, direction inferred as server to client). [client] [client code]
5. **Reel.** "If you get the fish to bite, you'll see the reel meter. Press the left mouse button to reel in the fish, but don't press it too much or the line will snap. If you don't press it enough, the fish will escape!" (1084468075); "PRO TIP: Don't reel so hard! Release the left mouse button when your bobber goes in the red." (993914867); "Hold down the left mouse button to reel in your fish." (1492704980). The client sends a **ReelInRequest** (sub-type 7: handle and a flag) while reeling. Failure text: "Fish Escaped" (2867629310), "You let that fish get away!" (1724785125). [client] [client code]
6. **Result.** The server answers with a **FishingResult** (sub-type 14, three strings) and fish info updates (sub-type 4). The catch card shows fish name, size and a number ("Feral Catfish, Extra Large, 20.39", "Cheery Salmon, Medium, 3.94"; the unit is not shown), and a message "You receive 1 Medium Cheery Salmon" (`0LCKMQz3zMs/00036`, `0LCKMQz3zMs/00268`, `0LCKMQz3zMs/00260`). A fish above the player's level cannot be caught: "You need to level up before you can catch this fish!" (1580530816). [client] [video]
7. **Special.** A SpecialRequest/SpecialResponse pair (sub-types 15 and 16) exists; its purpose is unknown. [client code]

The lure model uses attachment names `sg_fishing_lure_bbe`, `mouth` and `hinge_2`. [client code]

## Abilities and traits by level

- **Abilities by level: none found.** Fisherman is a freestyle job; the wiki Jobs page does not list ability unlocks for it. Progression is gear, fish gates (see the ladder 1, 5, 10, 14, 17, 20 above), lures and the stats Casting Skill, Casting Strength, Line Strength and Reeling Speed. [wiki Jobs] [client]
- **Titles.** "The Angler" at level 20 [wiki Titles] [client 2919484740, 4253955498]. Rank names Novice, Intermediate, Advanced and Master Fisherman exist with no level mapping [client].
- **Trophies.** Levels 4 and 8 bronze (5 points), 12 and 16 silver (10), 20 gold (15). [wiki Achievements]

## Where the job levels

- The fishing minigame at the six spots (see Fishing spots). Video: Fisherman 1 at the start, 20 on a veteran account. [video]
- The quest line above and the repeatable "Catch of the Day" and "Keep it Clean" quests; the video reward for one Catch of the Day is 30 stars and 16 coins. [video]
- Events: the Sacred Grove Shallows tournament and Snow Days fishing at Wintery Basin. [client] [wiki]
- Mixed activity: Chef fish dishes and the Merri De'Flor contests, see [[Chef]]; Brawler duel instance "Brugo's Fishing Spot" is not fishing, see [[Brawler]].
- Whether quests or the minigame give job XP, and how much, is unknown.

## Conflicts

1. **Trainer.** The wiki gives one trainer, Reed Stillwater, and a Message Board in Sanctuary as the unlock. The client confirms Reed and the message board text but also has Anna Mudskipper, Muggup, Jonah Relicreel and Guppie inside Sacred Grove Shallows. Client used. [wiki] [client]
2. **Jonah Relicreel's location.** The wiki lists "Itty Bitty Bait: Jonah Relicreel in Rainbow Lake"; the client places Jonah in Sacred Grove Shallows (2091401093, 1353143859). Client used. [wiki] [client]
3. **Quest names.** Wiki "Becoming a Better/Fisherman" against client "Being a Better Fisherman"; wiki "Testing the waters" against client "Testing the Waters". Client spelling used. [wiki] [client]
4. **Members status.** The wiki Jobs list does not mark Fisherman as members only, while the client says every job is free only to level 4 (348625404) and sells Fisherman membership bundles. Resolved here as `members_only: false` with the level-4 free cap; per-feature status unknown. [wiki] [client]
5. **Brambleback's Bayou fish.** The client lists eight fish including Golden Scaled Nettler. The wiki lists seven and has Bitter Betta twice (levels 1 and 20) without the Nettler. Client used. [client] [wiki]
6. **Wiki spelling.** The wiki lure text says "Blubbercuda" and "Butter, Flyfish" (its Wintery Basin page says Blubracuda); the client spells Blubracuda and Butter Flyfish, and has its own typos "Pirahna" (3820692972), "amythest" and "Katie" against "Kati" McScuttle (1523245831, 376306261 against 2610015718). Client spelling used per string. [client] [wiki]
7. **Chip Numbwing.** The wiki text says Snowhill, the infobox says Seaside. [wiki]
8. **Catch of the Day giver.** Callie Kelpcake's offer reads like a catch-of-the-day request (3475541213) but the video frame shows Melanie beside the completion. Unresolved. [client] [video]
9. **Fester's challenge.** Six butter flyfish (163816406) against four in five minutes (4221070183). Unresolved; one is probably the "NOT LIVE" version. [client]
10. **Level quests.** The wiki and client have "Level 4/8/12/16/20" titles; the client also has "Achieve Level 5/10/15" strings with no title. [client] [wiki]
11. **Muggup.** The client puts Muggup in Sacred Grove Shallows and also "south of Lakeshore, before you get to Blackspore" (31022572). One NPC or two is unknown. [client]
12. **Rod pairing.** The wiki pairs rods with the three description strings; the client has no link between them. [wiki] [client]

## Gaps

- **Rewards.** Coins, XP and items for every quest except the one video datapoint (30 stars and 16 coins for a Shallows Catch of the Day). The "Fisherman:" titles also give no reward in any string.
- **Order and prerequisites.** No quest order, level gate or prerequisite is decoded. Within clusters the order is read from "now that" and "after" lines and is marked inferred. The order between clusters, and whether each cluster has an entry level, is unknown. The fish ladder suggests the line is gated by job level, but this is not proven.
- **Title-to-text links.** Rows marked † are guesses from wording. Rows with the weakest links: Find Your Own Spot!, Stronger Equipment, What Is It?!, Good Enough to Eat!, Fresh Fish Special, VIP Fish, Live Bait!, Taking the Bait, An Offering, Fish Food, Fishing Frenzy, Reel Riches, Buried Treasure, My Hut is Your Hut, Feed the Penguins, Showing Off Your Skills, The "Big" One, Casting Out For Fishing, and One Spot Left.
- **Givers and locations** for the rows where the giver is shown as "unknown" or "†".
- **Titles of the many fetch quests** (Archie, Calvin, Lenny, Kim, Derek, Estelle, Emil, Laggosh, Waterwings) and of the Reed shark and Shallows follow-ups.
- **Gear numbers.** Rod, lure and outfit prices, level requirements, stats and shard tier order; vendor stock for Guppie, Big J, Chip Numbwing and Bluerose.
- **Minigame numbers.** Fish weights, size thresholds, rarity, difficulty, catch timers, bite chance, line-tension rates, how a lure's 10% applies, junk and treasure tables, and what the "special" action does. The fishing data tables are bound by name at runtime and are not in the locale file.
- **Video.** Almost none: one player recording of the Shallows at Fisherman 20 (`0LCKMQz3zMs`), plus job windows. No frames show Reed, the trainers, Wintery Basin, Rainbow Lake, the Bayou or any "Fisherman:" quest panel.
- **Members.** Which Fisherman content is members only.
- **Rank names.** Which level each of Novice, Intermediate, Advanced and Master Fisherman applies to.
