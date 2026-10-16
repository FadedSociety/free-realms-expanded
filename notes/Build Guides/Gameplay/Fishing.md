---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Fishing

Part of [[Build Guides]] › Gameplay. Related: [[Minigame Framework]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`, `unknown`) are the native-map labels for a function; every label used here is accepted. Several fishing labels are `unknown` or had part of their wording rejected by the reviewer. Where this page leans on one of those, it says "read from pseudocode in this pass", which means the behaviour was taken from the Ghidra pseudocode in `map.sqlite` by this write-up and is not a labelled fact. Source tags: *client* = the shipped executable (shard findings, labels, pseudocode), *client Lua* = the decompiled UI script `ScriptsBase.lua`, *client resource* = a file in the shipped `Resources` folder or the extracted asset catalog, *client locale* = the shipped text table, *video* = a reviewed frame record (frame key given; no player names or chat are recorded), *wiki* = the fan wiki. Precedence is client, then video, then wiki. **(R)** marks a reconstruction: our own server policy, or an inference the client alone cannot settle. No fishing datagram has ever been captured and no original server code exists, so every statement about what the server sent or decided is a hypothesis built from what the client requires. This page does not use OpenSourceFreeRealms or FreeRealms-Legacy code or data, the community packet catalogue, or vault notes that depend on them. Related: [[Minigame Framework]], [[Fisherman]] (the job and its quests), [[Items and Inventory]], [[Quests and Objectives]], [[Rewards and Collections]], [[Pets]], [[Chef]].

### Overview

Fishing is a native client activity: a packet family, a processor object and a set of Lua and Flash panels. It is not one of the Flash-movie games. In the minigame tables it is TYPE 21 and plugs into the shared launch path described in [[Minigame Framework]]. The round itself runs on its own message family, id 138 (`0x8a`).

- **Where.** Six fishing areas plus a tutorial. The client text names Sacred Grove Shallows, Brambleback's Bayou, Darklit Lagoon, Rainbow Lake, Wintery Basin and Frostbitten Banks. The player reaches each by talking to a guide or using a boat (client locale: "Interact with the boat ... and you'll be fishing in no time!"). Which town each area belongs to comes from the wiki: Stillwater Crossing (Shallows), Briarwood (Bayou and Lagoon), Lakeshore (Rainbow Lake), Snowhill (Wintery Basin and Frostbitten Banks). (client locale, wiki)
- **Casting.** The first tutorial line says to go to the shore and click the water to cast. Another says to press the left mouse button again to cast and to "match your aim with the target for a perfect cast"; a third says to "line up your casting ring within the target ring". The asset manifest has inner and outer reticle rings in "cast" and "no cast" versions. The client text names four cast grades: Perfect, Great, Good and "Bad Cast Location". (client locale, client manifest; the grade names are paired with the message keys by wording only, see Data and formats)
- **Underwater view.** After the cast the client switches to an underwater scene. The Lua hides the main HUD, mini-map and game dock on `Fishing:StartUnderwater` and restores them on `Fishing:EndUnderwater`. Schools of fish and other fishermen (as proxied players) are drawn in the scene. (client Lua `Fishing`, `ScriptsBase.lua` line 44185 ff.; client `00b6cd80`)
- **Bites and reeling.** Fish drift in schools. A fish that is interested shows a question mark over its head. The client text then says: press the left mouse button to tug on your line. If the fish bites, a reel meter appears; hold the left button to reel, but "don't press it too much or the line will snap. If you don't press it enough, the fish will escape!" A tip says to release the button when the bobber goes in the red. Status words the text table has for this: Fish Nibbling, Fish Hooked, Item Hooked, Fish Combo, Fish Escaped, Item Lost, Line Snapped, Nothing Caught, Item Found. (client locale)
- **What you catch.** Fish, junk and treasure. Fish are inventory items in five grades: the plain name and Small, Medium, Large and Extra Large (for example the item names "Flutterfish", "Small Flutterfish", "Medium Flutterfish", "Large Flutterfish", "Extra Large Flutterfish"). Junk catches seen in quests are Rusty Cans and Plastic Bags. The wiki says treasure chests can be reeled in. A catch card shows the fish name, its size grade and a weight printed with two decimals. (client locale, client `00cd0150`, wiki)
- **Fish Finder.** While fishing, the action bar has three buttons: Fish, Tackle and Rewards (icon ids 21640, 26830 and 28561). Each opens the Fish Finder panel on that tab. Fish lists the fish of this area, whether each is catchable and a lure hint; Tackle lists lures; Rewards lists the player's scored catches. Picking a fish calls `Ui.FindSelectedFish`; picking a lure assigns it to the item bar. (client Lua `FishFinder`, lines 44223 ff.)
- **Rods and lures.** Rods change reach and depth. The client has rod descriptions "cast a short distance", "cast a greater distance" and "fish in the deepest of waters", and five rod names (Simple Bamboo, Golden Reel, Metal, Red Scoped, Golden Scoped); the wiki pairs them in that order, two short, two greater, one deepest. Lures help "catch a specific fish": each lure text gives a 10% better chance for three named fish, and the Treasure Magnet does the same for treasure. (client locale; the wiki lists the same five rods, 12 lures and the magnet)
- **Fisherman job.** Fishing is the Fisherman job's gameplay, max level 20. The wiki lists the fish an area offers with a "catchable at level" number (1, 5, 10, 14, 17 or 20), and the client has a per-fish "catchable" column in its fish list. Quests, trainers and their goals are on [[Fisherman]]. Fishing quest goals are plain catch counts, sometimes with a size ("Reel in an extra large Butter Flyfish"). (client, video, wiki)
- **Results.** There is no score screen. For type 21 the framework quits straight back to the world (see [[Minigame Framework]]). Rewards are visible only through item messages, coins, stars and quest progress. (client Lua, video)
- **Shared space.** Other fishermen in the same area are drawn from the same packet family: the client rebuilds each one's cast and reel animation and bobber from the messages the server sends. (client `00b6cd80`)
- **Event use.** A client text advertises ice fishing and "Snow Days" fun at the Wintery Basin. Nothing else about the event is in the client. (client locale)

### How the client implements it

#### Main classes

| Class or object | Role | Evidence |
|---|---|---|
| `FishingProcessor` (ctor `00b69860`, dtor `00b69d40`, vtable `01822a44`) | The one fishing manager. The client keeps it at client +0x128: message id `0x8a` in the packet switch `009471a0` calls slot 4 of that object, and `0092f460` reads the same slot to switch fishing on or off. It owns the local state word (+0x1c), the on/off flag (+0x18), a state timer (+0x1b8), the underwater flag (+0x1bc), the lists of proxied players (+0x1e8), schools (+0x1f8) and underwater fish (+0x208), and nine sound slots. Its constructor also builds the three data sources, a `FishingZoneConfig` and a `FishingPlayerConfig`. | confirmed ctor and dtor; field offsets read from pseudocode |
| Receive dispatcher `00b6cd80` | `FishingProcessor::vftable[4]` (shard S0352). Reads the two-word header, then switches on the sub-type. It handles 1, 3, 4, 5, 7, 8, 10, 11, 12, 13, 14, 16 and 18. | label `unknown` (reject on wording only); cases read from pseudocode in this pass |
| `FishingProcessor::Update` (`00b6c290`, per frame) and `SetActive` (`00b6bb90`) | The local state machine and the on/off switch. | likely, both reviewed `agree` |
| `ProxiedFishingPlayer` (ctor `00ccf890`; `SetState` `00ccffb0`; `ShowResult` `00ccfbf0`) | One record per fisherman, local or remote, keyed by the 8-byte handle. Holds the state, the bobber (+0xd4) and the last catch (name id, size grade, icon, weight). Found or created by `00b64300`. | confirmed ctor; likely methods |
| `ProxiedFishingSchool` (`00cd1210`), `ProxiedFishingUnderwaterFish` (`00cd1760`, per-frame `00cd1a10`), `ProxiedFishingBobber` (`00cd3cf0`) | The school (a drifting group), each fish that can bite, and the floating bobber. | confirmed constructors; `00cd1a10` speculative |
| `ControllerFishing` (ctor `00af8940`, vtable `0181c2c4`, derives `ControllerRegular`) | The player-controller state while fishing. Slot 51 `00af8810` is its per-frame update. Slot 50 `00af8b20` turns action ids `0x13` to `0x15` into the Lua call `Fishing:ClickedAbility` with index 0 to 2 (the Fish, Tackle and Rewards buttons). | `00af8810` confirmed; `00af8940` and `00af8b20` unknown |
| Data sources `BaseClient.FishingProcessor.FishTypes`, `.FishingLures`, `.FishScored` | What the Fish Finder reads. Columns are under Data and formats. | confirmed (`00cd4a10`, `00cd4b50`, `00cd4ab0`) |
| `FishingZoneConfig` (ctor `0107a850`) and `FishingPlayerConfig` (ctor `0107a9f0`) | Tuning objects. Both are built inside the registration-response packet constructor `00b695f0` and inside the processor constructor. | confirmed constructors |
| Lua `Fishing`, `FishFinder`, `FishingStatusText`, `FishingCaughtStatusText`, `FishingTensionMeter`, `FishingDistanceMeter`, `FishingReel`, `FishingTutorial` | The UI layer. Flash movies `fishingMeter_REV2.gfx`, `fishingmeters_bbe.gfx`, `fishingStatus.gfx`, `fishFinder.gfx`, `icon_fish_combo_bbe.gfx`, `fish_moving_target.gfx` and `levelup_fishingjob.gfx` are in the asset manifest. | client Lua, client manifest |

#### Switching fishing on and off

`SetActive` (`00b6bb90`) is not called by a fishing packet. In `0092f460`, the local-player branch of the base player update compares bit 22 of the dword at +0x6f8 of the player object with the processor's flag (+0x18) and calls `SetActive` when they differ (read from pseudocode in this pass; the bit's meaning is my reading, not a label). `BeginPlayerZoning` (`0093d180`) and `BaseClient::Shutdown` (`009359e0`) call it with 0. So the server turns fishing on by setting that player flag in its normal player update; the registration packet follows from the client. (client)

Turning on: save and set the audio mixer (`00b61580`, `00b615e0`), load the nine `MG_Fishing_*` sound names (`00b65d90`), then register the local player (`00b6b720`). Turning off: restore audio, drop the effects, clear the state words and erase the school list. `WaitForWorldReady` (`00936d00`, confirmed) also checks the fishing processor, along with the race, derby, soccer, Battle Mages and ship processors, before it sends the finished-loading packet (S0247). What its fishing readiness test is has not been traced. The Lua `Fishing:Show` runs when the framework shows the HUD for type 21: it sets the group panel to encounter mode, sets `HUD.inMiniGame` and loads the three action-bar buttons. (client Lua)

#### The local state machine

`FishingProcessor::Update` (`00b6c290`, likely) switches on the state word at +0x1c. It only runs when the controller probe reports the fishing controller (`0x13`) and the on flag is set. The table is read from pseudocode in this pass; state names are mine.

| State | What the client does | Leaves when |
|---|---|---|
| 0 aiming | Builds a ray from the camera through the cursor, probes the scene with it (`00fd4290`, flags `0x103`), accepts a hit whose type field is 8, stores the point at +0x30 to +0x3c and its distance at +0x50, sets a has-target flag (+0x239) and, when the distance lies between the limits at +0xb0 and +0xb4, an in-range flag (+0x238). The limits' source is not resolved. | the cast input is pressed with a valid target |
| 1 cast animation | Locks the target (+0x40 to +0x4c), sets +0x23a to 1, builds the sub-5 packet (handle, distance, vector) and sends it with `00b6b0d0`, stores the player state and moves to 2 through `00b64580`. | immediately |
| 2 cast | If the gate `00ccf7b0` passes, sends the sub-6 cast request with the target vector and the byte at +0x23a (`00b6b7e0`), then computes a launch vector (`00b640e0`) for the bobber. | the server's sub-8 bobber message arrives |
| 3 bobber in flight | Waits until the state timer exceeds a constant (`DAT_018227b8`, unresolved), then, for the local player, moves to 4, calls `00b64360(1)` (underwater on) and posts `Fishing:StartUnderwater`. | timer |
| 4 underwater | Runs the per-fish update `00cd1a10` for every underwater fish. While an input bit (`TestBit3Of20` `00af84b0`, speculative; the client text says the left mouse button) is set and a fish record exists whose byte +0xd9 is clear (or which is an item), sends the sub-7 reel-in request with flag 1 (`00b6b8b0`) and moves to 5. | reel input |
| 5 reeling | Keeps running the per-fish update. | the server's result (sub 14) arrives |
| 6 result / idle | Stops the reel-loop sound (slot +0x35c) and, when the gate `00ccf7b0` passes and an input bit (`00af84d0`, speculative) is clear, returns to 0. | next cast |

`00b64580` is the entry point that changes the state: for the local player it stores the new value, clears the timer, and per state posts `Fishing:StartCasting` (state 1) or switches the underwater audio and view (`00b64360`: posts `Fishing:StartUnderwater` or `Fishing:EndUnderwater` and `Fishing:EndCasting`). Both functions are `unknown` with rejected wording; the strings are certain, the exact call order is not.

#### Registering, schools, fish and bobbers

1. **Register.** On activation the client sends sub 2 with the local handle (`00b6b720`). It then expects sub 3, the registration response. The response carries tuning values, a list of fish entries for the Fish Finder and one more integer list (see Network). (client)
2. **Schools.** Sub 11 spawns a school: a key, two vectors, a list of model entries and an integer list. The client looks the key up (`00b646e0`), creates the school if absent (`00b63870`) and builds its models (`00cd12a0`). Sub 13 moves a school (key plus a vector, handled by `00cd0df0`). Sub 12 removes it. (client; the name of each field is not recovered)
3. **Fish runs.** Sub 18 carries a flag, an integer, a string and a counted list of fish records. The client clears the player's underwater-fish list (`00b664f0`) and then walks the records resolving a model for each; it appears to rebuild the underwater fish from them. This is read from pseudocode, not labelled. (client)
4. **Fish states.** Underwater fish are found by key (`00b64710`). Sub 10 carries a fish key and two flag bytes. For the local player the client finds the fish and, read from pseudocode in this pass: with the first flag set it sets the fish byte +0xd9 and clears +0xd8 and +0xda; with the first clear and the second set it sets +0xd8; with both clear it sets +0xda and shows a status text, `bbe_fishing_fish_escaped` for a fish or `bbe_fishing_item_lost` for an item (the fish record's +0x10 byte tells which). For any player whose bobber exists and whose flags are both clear it also sets a reel flag on the player. What +0xd8, +0xd9 and +0xda mean is not resolved.
5. **Bobbers.** Sub 8 gives a player a bobber: the client stores the u32 in the player record (+0xd8), copies a 64-byte transform built from the two vectors, and creates the bobber proxy through `00ccfdb0` (read from pseudocode; that function's label says it creates a result presenter, which looks wrong, see Contradictions). The player then moves to state 3 or 4, chosen by a test of the player's actor (`00957f90`, speculative label). Sub 7 sets a reel flag on the player; when the player is the local one it marks every underwater fish with +0xdc. Sub 5 puts that player into the cast-animation state (2). (client)
6. **Lure model.** `00b67b80` updates the lure attachment on the model (`sg_fishing_lure_bbe`, bones `mouth` and `hinge_2`); its label was downgraded by the reviewer. (client)

#### Result and catch card

Sub 14 is the result message. The kind field (body offset +0x18) drives a switch in `00b6cd80`. Observed effects, read from pseudocode in this pass (low confidence on the meaning of each kind): 0 posts the "nothing caught" status text and sets the local state to 6; 1 clears a line-out flag (+0x1b6) and may change state; 2 sets the player's result state word (+0x34) to 1 and the local state to 6; 4 stops one effect and builds transforms from the bobber's `mouth` and `hinge_2` bones (probably the catch leaving the water); 5 stores and shows the catch (below); 6 and 7 return the player to idle; 8 clears the line-out flag and posts overhead text. Kind 3 and kinds above 8 have no case.

For kind 5 the client copies fields into the proxied player (`ProxiedFishingPlayer`, +0x94 to +0xd0) and, for the local player only, appends one row to the `FishScored` list from fields +0x20, +0x24, +0x44, +0x48, +0x3c, +0x4c and +0x40 (name id, icon id, difficulty, rarity, size, score, time to catch). The catch card is built by `00cd0150` (confirmed, reviewed `agree`): the name is looked up from a text id, the size grade 1 to 4 maps to `bbe_fishing_small`, `_medium`, `_large` and `_extra_large`, the weight is the float at +0xd0 formatted `%2.2f`, and `FishingCaughtStatusText:DisplayMessage` shows name, size, weight and icon. A bool in the packet separates items from fish and skips the size grade. The catch itself reaches the inventory as an ordinary item update (video: "You receive 1 Large Feral Catfish"). (client, client Lua, video)

### Data and formats

**Client resources (read by the client, not sent).**

| File or source | What it holds | Source |
|---|---|---|
| `MiniGameData.txt` | Eight TYPE 21 rows: ids 584 and 595 to 601, all difficulty 1, not members only, no shortcut, no preload file. Name ids are 0 (584), 417055 (595), 408604, 408599, 93710, 408591, 408622 and 408640 (596 to 601). Description ids are 37935 (584), 417056 (595) and 5235 on the six area rows. The name and description ids do not resolve in the locale file this guide uses. | client resource |
| `MiniGameTypeData.txt` | Type 21 row: every column 0 except `IS_ACTIVE` 1 (not Flash, not micro, not members only). | client resource |
| `MiniGameGroupData.txt`, `MiniGameGroupLinks.txt` | Group 21 holds links 601 to 608 for games 584 and 595 to 601. | client resource |
| `CodeStringMappings.txt` | Message-key to string-id pairs listed in the next table. | client resource |
| Asset manifest and extracted catalog | Rod models and skins (`fishingrod`, `fishingrodlauncher`, `fishingrodscoped`, `fishingrodscreel`, skins marked L2 to L5), fish models (`fishing_thin`, `_medium`, `_fat_fish_bbe`), reticle rings (`sg_fishing_reticle_*_cast/no_cast_bbe`), the lure model `sg_fishing_lure_bbe`, lure icons, bubble and school effects, nine `MG_Fishing_*` sounds, boats, fish animations, and terrain sets for fishing ponds and streams with the prefixes `sg_`, `bw_` and `sh_`. | client resource |
| `bbe_FishingZoneConfig.txt` (plus `.School`, `.SchoolPath` and `.Node`) and `bbe_FishingPlayerConfig.txt` | Read by loaders `01079260`, `01079d80`, `0107aae0` and `0107b140`. **No copy exists in the shipped `Resources` folder or the extracted asset catalog** (searched by name in the extracted pack catalog and by content in the loose client files). None of the four loaders has a caller in the call graph. See Open questions. | client resource search, client |

**Message keys and texts (client resource plus client locale).** The key-to-text link is by identical wording only, because the string ids in `CodeStringMappings.txt` live in a different id space from the locale file's ids.

| Key (id) | Text | Function whose string list names the key |
|---|---|---|
| `bbe_fishing_perfect_cast` (4966), `_great_cast` (416886), `_good_cast` (416887), `_bad_cast` (4964) | Perfect Cast!, Great Cast!, Good Cast!, Bad Cast Location | none in the map's string lists |
| `bbe_fishing_fish_nibbling` (4955), `_fish_hooked` (4958), `_item_hooked` (4957), `_item_found` (4956), `_fish_combo` (4959), `_line_snapped` (4961) | Fish Nibbling, Fish Hooked, Item Hooked, Item Found, Fish Combo, Line Snapped | none in the map's string lists |
| `bbe_fishing_fish_escaped` (4962), `_item_lost` (4963), `_nothing_caught` (4965) | Fish Escaped, Item Lost, Nothing Caught | `00b6cd80` |
| `bbe_fishing_small` (4087), `_medium` (4088), `_large` (4089), `_extra_large` (4090) | Small, Medium, Large, Extra Large | `00cd0150` |

Tutorial keys `fishingTutorial0001` to `0019` map to string ids, mostly 417047 to 417065. The tutorial lines the locale table has for fishing include: click the water to cast; match the casting ring to the target ring; tug on the line when the fish has a question mark; the reel meter; the shore meter ("When it runs out, you have reeled your line in"); the fish finder; lures. (client resource, client locale)

**Tuning keys the client knows how to read.** These are the key names in the two loaders. The numeric values are not in the client (see above). Offsets are into the config objects. `0107b140` and `0107aae0` read the player keys; `01079260` reads the zone keys.

| Object | Keys (offset) |
|---|---|
| `FishingPlayerConfig` (`0107b140`, `0107aae0`; likely) | LineSnapChance (+0x14), MinTimeBeforeFishEscapes (+0x18), MinTimeBeforeFishStopsNibbling (+0x1c), BobberMinDistanceFromPlayer (+0x20), BobberRunAgroundCatchMaxDistanceFromPlayer (+0x24), ReelSpeedMetersPerSecond (+0x28), HookingReelSpeedMetersPerSecond (+0x2c), LineSnapTensionMinPercent (+0x30), FishEscapeTensionMaxPercent (+0x34), MinTimeBeforeSpecial (+0x38), MinTimeBetweenSpecials (+0x3c), SuperReelSpeedMetersPerSecond (+0x40), SuperReelDuration (+0x44), MaxNibbleTime (+0x48), FishMaxDistanceFromHook (+0x4c), FishStartDistanceFromHook (+0x50), BobberModelId (+0x54, integer), BobberSplashCompositeEffectId (+0x58, integer), MinCastDistance (+0x5c), MaxCastDistance (+0x60), MinTargetRadius (+0x64), MaxTargetRadius (+0x68), PerfectDistance (+0x6c), MinBiteChance (+0x70), MaxBiteChance (+0x74), TensionMeterSize (+0x78), CameraDistance (+0x98), CameraPitch (+0x9c), CameraHeading (+0xa0), CameraTargetHLQ (+0xa4) |
| `FishingZoneConfig` (`01079260`; likely) | EncounterDefId (selects the zone), BaseConfig and PlayerConfig sub-nodes, TimeBetweenBiteChecks (+0x14), CastRayMinY (+0x18), CastRayMaxY (+0x1c), BobberGroundCheckMinY (+0x20), NumSchools (+0x24), UnderwaterSkyFile (+0x34, string), UnderwaterRunScale (+0x40), UnderwaterRunOffset (+0x44), ReelDuration (+0x48), TensionMeterAmplifier (+0x4c, integer). Per school: SchoolTypeId, StartLocX/Y/Z, StartFacingX/Y/Z, SpeedMetersPerSecond, SchoolPathId, SchoolPathStartNodeId. Per path: PathId, NumNodes, and per node NodeId, NextNodeId and three position values. Errors go to `FishingErrors.txt`. |

**Fish Finder data sources (client, confirmed).**

| Source | Columns | How the Lua uses them |
|---|---|---|
| `FishTypes` (`00cd4420`, `00cd4620`) | 0 FishType, 1 FishName, 2 FishIcon, 3 FishSpecial, 4 FishCatchable, 5 FishLureRequirement. Columns 1 and 5 are text ids resolved to strings. | id = col 0, name = col 1, icon = col 2, catchable = col 4, description = col 5. FishSpecial is not read by the Lua. |
| `FishingLures` (`00cd4b50`) | An inventory-style source (base `00cb1bb0`). | guid col 0, icon col 1, name col 2, description col 3, quantity col 4, item id col 23. |
| `FishScored` (`00cd44e0`, `00cd47b0`) | 0 FishName, 1 FishIconId, 2 FishDifficulty, 3 FishRarity, 4 FishSize, 5 FishScore, 6 TimeToCatch (a float). | name col 0, icon col 1. |

**Spot fish lists (client locale, the area description text; the wiki's "catchable at level" is added in brackets).**

| Area | Fish (client list) | Wiki level numbers |
|---|---|---|
| Sacred Grove Shallows | Flutterfish, slugmud skipper, butter flyfish, tickled trout, tangletooth shark, feral catfish, cheery salmon, chipsen fish, baconfish | Slugmud Skipper, Tickled Trout, Flutterfish 1; Butter Flyfish 5; Cheery Salmon 10; Chipsen Fish 14; Feral Catfish 17; Baconfish, Tangletooth Shark 20 |
| Brambleback's Bayou | Creeping cod, bitter betta, golden scaled nettler, globfish, blind swurglefish, fanged grouper, changed salmon, briar nibbler | Creeping Cod, Bitter Betta, Globfish 1; Blind Swurglefish 5; Changed Salmon 10; Fanged Grouper 14; Briar Nibbler 17; "Bitter Betta" again at 20 (see Contradictions) |
| Darklit Lagoon | Creeping cod, ink cod, purplenosed shark, old sole, thorny trout, golden scaled nettler, roach loach | Ink Cod, Creeping Cod, Golden Scaled Nettler 1; Thorny Trout 5; Old Sole 10; Purplenosed Shark 14; Roach Loach 17 |
| Rainbow Lake | Flutterfish, finless fish, tickled trout, tangletooth shark, peachy perch, calico catfish, lady tetra, toothy tetra | Flutterfish, Calico Catfish, Tickled Trout 1; Toothy Tetra 5; Peachy Perch 10; Finless Fish 14; Lady Tetra 17; Tangletooth Shark 20 |
| Wintery Basin | Winter piranha, frostgill smelt, coach loach, frozen char, ferocious fangler, blubracuda, spineless stickleback | Frozen Char, Winter Piranha 1; Frostgill Smelt 5; Spineless Stickleback 10; Blubracuda 14; Coach Loach 17; Ferocious Fangler 20 |
| Frostbitten Banks | Winter piranha, chilly grouper, blue thornfin, ferocious fangler, frostgill smelt, spineless stickleback, talking bass, pacu pacu, goofy grouper, plattypus | Chilly Grouper, Winter Piranha 1; Frostgill Smelt, Pacu Pacu 5; Blue Thornfin, Spineless Stickleback 10; Goofy Grouper 14; Talking Bass 17; Ferocious Fangler, Plattypus 20 |

**Lures (client locale).** Each lure's text gives a 10% better chance for three fish. The groups are: Talking Bass, Cheery Salmon, Bitter Betta; Plattypus, Lady Tetra, Roach Loach; Spineless Stickleback, Calico Catfish, Purplenosed Shark; Goofy Grouper, Finless Fish, Fanged Grouper; Blue Thornfin, Chipsen Fish, Thorny Trout; Pacu Pacu, Peachy Perch, Globfish; Chilly Grouper, Butter Flyfish, Briar Nibbler; Frostgill Smelt, Flutterfish, Ink Cod; Winter Piranha ("Pirahna" in the client text), Toothy Tetra, Blind Swurglefish; Coach Loach, Feral Catfish, Creeping Cod; Frozen Char, Slugmud Skipper, Old Sole; Blubracuda, Baconfish, Changed Salmon. The Treasure Magnet gives the same bonus to treasure. The wiki names the lures (16oz Steak, Flyfisher 3000, French Fry, Frostflies, Mega Slider, Nightcrawlers, Perch Pinpointer, Shiny Crankbait, Skipper Seeker, Sleek Clicker, Thorn Jig, Tiny Rib, Treasure Magnet) and says they are sold in the Coin Shop. The lure item names with "- Lure" are in the client locale; the mapping of each name to its fish group is by the wiki. (client locale, wiki)

### Network

**Envelope.** Fishing is message id 138 (`0x8a`), carried tunnelled in the client packet (`PacketTunneledClientPacket`, constructor `0089adc0`, sent through `00925640` with flag 1) when the transport at client +0x280 exists. Every body starts with two 16-bit words: the family id 138 and the sub-type (reader `00b66830`, writer `00b69550`, both confirmed). The sub-type is not a single byte as in the pet family. Handles are 8 bytes (two dwords). A vector is four 32-bit floats (`008e2410`, NaN sets the reader's NaN flag). A string is an `i32` length followed by the bytes. A short read zeroes the field and marks the packet bad. The sub-types with constructors are 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16 and 18. Sub-types 9 and 17 have no class; sub-type 0 is the empty header object `00b608b0`. No fishing datagram has been decoded from a real capture, so byte order on the wire is unverified. (client)

**Client to server (writers and senders, confirmed unless noted).**

| Sub | Class | Body after the header | Sender |
|---|---|---|---|
| 2 | RegisterPlayerRequest | u64 handle, u8 | writer `00b6a400`, sender `00b6b1e0`; built by `00b6b720` on activation. The visible code sets only the handle, so the byte keeps the constructor's default; its value and meaning are unresolved. |
| 5 | CastAnimRequest | u64 handle, f32, vector | writer `00b6a380`, sender `00b6b0d0`; built in state 1 of `00b6c290`. The f32 is the cast distance (processor +0x50) and the vector is derived from the locked target; its frame of reference is unresolved. |
| 6 | CastRequest | u64 handle, vector, u8 | writer `00b6a460`, sender `00b6b2f0`; `00b6b7e0`, state 2. The vector is the aim point; the byte is processor +0x23a, whose meaning is unresolved. |
| 7 | ReelInRequest | u64 handle, u8 | writer `00b6a4d0`, sender `00b6b400`; `00b6b8b0`, state 4, flag 1 in the one mapped call. |
| 15 | SpecialRequest | u64 handle | `00b6b510`; its builder `00b6ba10` has no caller, so the client never sends it in this build (class label speculative). |

**Server to client (all read by `00b6cd80`).** Offsets are into the decoded packet object; order is the read order.

| Sub | Class | Body after the header | What the client does with it |
|---|---|---|---|
| 1 | UpdateData | u64 handle, vector (`00b66890`) | Looks up the actor by handle; nothing else is visible. |
| 3 | RegisterPlayerResponse | player-config block, zone block, u32 array, fish entries (layouts below; `00b6a170`) | Copies the zone and player config into the processor (`00b6bae0`, `00b67a00`), applies camera values through `00af85e0` when the fishing controller is active, assigns and processes the fish entry list (`00b692d0`, `00b65ed0`), copies one field to processor +0x250. |
| 4 | FishInfoUpdate | `i32` count, then entries (`00b6ad20` to `00b691d0`) | Assigns and processes the fish entry list, then refreshes the data-source rows (`00cd4480`, speculative). |
| 5 | CastAnimRequest (echo) | u64 handle, f32, vector (`00b668f0`) | Puts that player into the cast-animation state; stores the vector. |
| 7 | ReelInRequest (echo) | u64 handle, u8 (`00b66990`) | Sets a reel flag on the player; for the local player marks every underwater fish. |
| 8 | SpawnProxiedFishingBobber | u64 handle, u32, vector, vector (`00b66a00`, likely) | Creates the bobber; local player to state 3, others to 4. |
| 10 | UpdateProxiedFishingBobber | u64 handle, u32 fish key, u8, u8 (`00b66a80`) | Sets fish state bytes and may show a status text (for the local player). |
| 11 | SpawnProxiedFishingSchool | u32 key, vector, vector, model entries, u32 array, u32, u32 (`00b69130`) | Creates the school. |
| 12 | DespawnProxiedFishingSchool | u32 key (`00b68810`) | Releases the school's models and removes it. |
| 13 | UpdateProxiedFishingSchool | u32 key, vector (`00b66b80`) | Refreshes the school's model effects. |
| 14 | FishingResult | see below (`00b66be0`) | Result switch and catch card. |
| 16 | SpecialResponse | u64 handle, u32, u8 (`00b66f50`) | Not traced; no fishing effect found. |
| 18 | SpawnFishRun | u8, u32, string, counted records (`00b6a2b0`) | Spawns underwater fish. |

Echo rule: sub-types 5 and 7 are accepted from the server as well as sent by the client. The dispatcher creates the matching packet object for them, so the server can send them to other fishermen. That it does so as an echo of someone's cast is an inference from the two shapes matching (S0349). Sub-type 6 is never accepted inbound. (client)

**Sub 3 layouts (read order).** The destinations coincide with the fields of `FishingPlayerConfig` and `FishingZoneConfig`, and the dispatcher copies the block with `00b67a00` (source offsets 0x14 to 0xa4, the span of the config). Names below are matched by offset; treat them as likely, not proven.

| # | Width | Dest offset | Matching config key |
|---|---|---|---|
| 1 | f32 | +0x20 | BobberMinDistanceFromPlayer |
| 2 | f32 | +0x5c | MinCastDistance |
| 3 | f32 | +0x60 | MaxCastDistance |
| 4 | f32 | +0x64 | MinTargetRadius |
| 5 | f32 | +0x68 | MaxTargetRadius |
| 6 | f32 | +0x6c | PerfectDistance |
| 7 | u32 | +0x58 | BobberSplashCompositeEffectId |
| 8 | f32 | +0x78 | TensionMeterSize |
| 9 | f32 | +0x7c | no key at this offset in the loaders |
| 10 to 13 | 4 x f32 | +0x98, +0x9c, +0xa0, +0xa4 | CameraDistance, CameraPitch, CameraHeading, CameraTargetHLQ |
| 14 | f32 | +0x4c | FishMaxDistanceFromHook |
| 15 | f32 | +0x50 | FishStartDistanceFromHook |
| 16 | f32 | +0x48 | MaxNibbleTime |

Then the zone block (`00b67240`): an `i32`-length string (matches UnderwaterSkyFile), then f32 +0x40 (UnderwaterRunScale), f32 +0x44 (UnderwaterRunOffset), f32 +0x48 (ReelDuration), u32 +0x4c (TensionMeterAmplifier) and f32 +0x20 (BobberGroundCheckMinY). Then an `i32` count and that many u32 (`008dab40`, purpose unknown). Then the fish entries.

**Fish entry (subs 3 and 4, `00b673b0`).** u32, u32, u32, u8, u8, u32, string, u32, u8, u32. The first three u32 are kept as string references (the cleanup in `00b65ed0` releases them as strings), so they are probably text ids such as the fish name. Two of the three u8 (offsets +0xd and +0x28) are copied into bool lists; the u8 at +0xc looks like a skip flag. Which field feeds which Fish Finder column is not resolved (reviewer: field meanings remain candidates).

**Spawn school (sub 11) records.** A model entry is u32, f32, f32 (`00b65cc0`). The two trailing u32 are stored in the school record at +0x30 and +0x34. **Spawn fish run (sub 18) record** (`00b67550`): u32, u32, string, string, u32, f32, u8, ten f32, then one nested value read by `008d53b0`. Names unknown.

**Fishing result (sub 14, `00b66be0`), read order.** The offsets are into the packet object. The "use" column comes from how `00b6cd80` and `00cd0150` consume the fields and is likely, not labelled.

| Order | Width | Offset | Use |
|---|---|---|---|
| 1 | u64 | +0x10 | player handle |
| 2 | u32 | +0x18 | result kind (switch: 0, 1, 2, 4, 5, 6, 7, 8) |
| 3 | u8 | +0x1c | caught thing is an item, not a fish |
| 4 | u32 | +0x20 | fish or item text id |
| 5 | u32 | +0x24 | icon id |
| 6 | string | +0x28 | unknown |
| 7 | f32 | +0x38 | weight, shown as `%2.2f` |
| 8 | u32 | +0x3c | size grade 1 to 4 (small to extra large) |
| 9 | f32 | +0x40 | time to catch (FishScored column 6) |
| 10 to 12 | u32 x 3 | +0x44, +0x48, +0x4c | difficulty, rarity, score (FishScored columns 2, 3, 5) |
| 13 | u32 | +0x50 | copied into the player record, unknown |
| 14, 15 | string, string | +0x54, +0x64 | unknown |
| 16 | u32 | +0x74 | copied into the player record, unknown |
| 17 | f32 | +0x7c | unknown |
| 18 | u8 | +0x80 | unknown |
| 19 | u32 | +0x78 | copied into the player record, unknown |

The existing label on `00b66be0` says "two length-counted blobs"; the code reads three strings. The packet constructor `00b64ec0` confirms three `IString` members (+0x28, +0x54, +0x64).

**Lua surface.** Events the processor raises to script: `Fishing:StartCasting`, `Fishing:EndCasting`, `Fishing:StartUnderwater`, `Fishing:EndUnderwater`, `FishingStatusText:Show` and `:DisplayText`, `FishingDistanceMeter:FishHooked`, `FishingCaughtStatusText:Show` and `:DisplayMessage`. The script calls back with `Fishing:ClickedAbility(0..2)` and `Ui.FindSelectedFish(id)`. (client, client Lua)

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour.

1. **Turn fishing on.** Set the local player's fishing flag in the normal player update (bit 22 of the player flag dword, see above), after the player exists and after the framework has put the player in the type 21 encounter ([[Minigame Framework]]). Clear it on leave and on zoning. (client)
2. **Answer registration.** On sub 2 reply with sub 3: the player-config and zone values for the area, the u32 array (its purpose is unknown) and the fish entries for this area and this player. The client fills the Fish Finder only from these entries, so the area's fish list, the catchable flag per player level (wiki level numbers; client column `FishCatchable`) and the lure hint text ids must come from our own content tables. Send sub 4 whenever the list changes, for example on level-up. (client; R for values)
3. **Own the outcome.** The client sends no tension, distance or timing report: only a cast aim, a distance and a reel-held flag. Bite checks, nibble, hook, line snap and escape must be decided by the server from elapsed time. The registration response carries only 16 player-config values and 6 zone values (`MaxNibbleTime`, `ReelDuration`, `TensionMeterSize` and the cast and camera values among them). The keys that decide bites and snaps (`TimeBetweenBiteChecks`, `MinBiteChance`, `MaxBiteChance`, `LineSnapChance`, the tension percentages, the reel speeds) are not on the wire, so they were server-side and are free for us to design. The original numbers are not recoverable from the client. (R)
4. **Cast.** Validate the aim against `MinCastDistance`, `MaxCastDistance`, the target radii and the rod's reach (client text: short, greater, deepest). Answer a good cast with sub 8 to every fisherman in range, then drive fish with subs 10 and 18 and schools with subs 11, 13 and 12. Echo subs 5 and 7 to nearby players. For a rejected cast the client text has "Bad Cast Location", but no field carrying the cast grade was found (see Open questions). (client, R)
5. **Result.** End each attempt with sub 14: kind, item flag, text id, icon, weight, size grade, time, difficulty, rarity, score. Send the item as a normal inventory update in the same step (video shows an item message per catch). A kind-5 result adds a row to the player's Rewards tab, so send it only for real catches. (client, video)
6. **Content the client lacks.** Fish and size tables per area and per Fisherman level, weights per grade, difficulty, rarity and score values, lure effects and stock, rod reach, junk and treasure tables, school layouts and paths, the real tuning values, and coin, XP and star rewards. Take the roster from the client's per-area fish lists and the wiki's level numbers, and make the numbers up to our own design. (R)
7. **Quests.** Fishing quest goals are catch counts, sometimes sized, so a catch should raise the normal quest objective for the item and size ([[Quests and Objectives]], [[Fisherman]]). Fisherman job experience for a catch is unknown. (client, video; R)
8. **Zoning.** Nothing fishing-specific to send for zoning. The client's world-ready check looks at its own fishing processor, which is built with the client. (client)
9. **Clean up.** On leave, send the school despawns (sub 12) so the client frees its models; the client erases its school list itself on `SetActive(0)`. There is no results screen to hold open. (client)

### Video and wiki evidence

**Video** (most frames come from one recording, `0LCKMQz3zMs`, of a level 20 Fisherman; no player names are recorded).

| Frame | What it shows |
|---|---|
| `0LCKMQz3zMs/00036`, `00048`, `00268` | Catch card with fish name, size and a number: "Feral Catfish, Extra Large, 20.39", "Feral Catfish, Large, 13.60", "Cheery Salmon, Medium, 3.94". The unit is not shown; the client code prints the number as a weight. |
| `0LCKMQz3zMs/00110`, `00124`, `00140`, `00260`, `00246`, `00272` | Item messages: "You receive 1 Tangletooth Shark", "1 Large Feral Catfish", "1 Extra Large Feral Catfish", "1 Medium Cheery Salmon", "1 Rusty Can", "1 Plastic Bag". |
| `0LCKMQz3zMs/00000`, `00092`, `00096`, `00246` | Quest "Keep it Clean": reel in Plastic Bags and Rusty Cans, then return to Melanie (a "Take Me There" button appears). |
| `0LCKMQz3zMs/00298`, `00304` | Quest "Shallows: Catch of the Day": four sized catches (an extra large Flutterfish, a large Cheery Salmon, an extra large Butter Flyfish, a large Chipsen Fish). The reward line reads "You receive 30 stars and 16 coins". |
| `L27vxozHOls/00326` | Quest "What's for Dinner?": catch Tickled Trout 2/5 and Flutterfish for the campers at Camp Robbie. Played by an Archer. |
| `UFsVcZ3X5iA/00014`, `4h2eVoje3Eg/00050` | A "Fishing" tile in the Games menu. |
| `bLxqx_aCizY/00112`, `00178` | "Brugo's Fishing Spot" is a combat duel (defeat Brugo), not the fishing minigame. Do not use it as fishing evidence. |

**Wiki** (Fisherman page and the six area pages, revisions 2021-12-27 to 2022-02-15): fish lists and level numbers as in the table above; all six areas are "Difficulty 1 fishing games"; area collections (for example Wilds Stream Fish, Briarwood Pond Fish, Snowhill Pond Fish, Extra Large Catch of the Day, Extra Large and in Charge); rods and lures as above, "bought from the Coin Shop under the supplies item category" or earned from quests or reeled in with a treasure chest.

### Open questions and contradictions

**Contradictions**

1. **Three "fishing" labels are pet code.** `00ccecf0` (`ControllerFishing::UpdateFrame`, likely), `00ccf3c0` (`ControllerFishing::SetState`, likely) and `00ccf260` (`FishingController::ResetResultState`, likely) check that the controller state object reports type 12 (the pet state, `ControllerPet`), read the pet handler's state word at +0x12ac (2 is the grooming state), walk grooming spots and build `PetTactileNotify` (`00ccdb70`, confirmed). `00ccf3c0` is called by `00b589c0`, a pets-handler function. They belong to the pet grooming path in [[Pets]]. This guide does not use them for fishing. The Pets guide's note that `00ccebc0` is "called only from the fishing controller" should read pet grooming code. (read from pseudocode in this pass; not yet re-labelled)
2. **`00ccfdb0` creates the bobber.** Its label says it creates a result presenter. The code destroys an old object at +0xd4, builds a `ProxiedFishingBobber` (`00cd3cf0`) from the player's bobber model id (+0xd8) and stores it back at +0xd4. It is called by the sub-8 case and by `00ccffb0`. (pseudocode)
3. **Class prefix of `00cd0150`.** The label is `ControllerFishing::UpdateFrame` (confirmed), but its only caller `00b6c290` passes each proxied player node, and the fields it reads (+0x10 handle pair, +0xd4 bobber, +0xa0 name id) are the proxied player's. Use the address, not the class. (pseudocode)
4. **Result reader.** `00b66be0` is labelled "two length-counted blobs" and `unknown`; the code reads three strings.
5. **Wiki against client.** The wiki's Brambleback's Bayou list shows Bitter Betta twice (levels 1 and 20) and no Golden Scaled Nettler, but the client area text and several quests name the Golden Scaled Nettler at the Bayou. The client list is used. The wiki puts the bait quest giver Jonah in Rainbow Lake; the client strings put him at Sacred Grove Shallows (see [[Fisherman]]). Spelling: the client says "Blubracuda" and "Winter Pirahna" in lure text; the wiki writes "Blubbercuda" and "Winter Piranha" in places. (client, wiki)

**Open questions**

1. **Where the tuning lives.** The four `bbe_Fishing*` text files are not in the shipped `Resources` folder or the extracted catalog, and the loaders have no callers. The registration response carries the player-config and zone values, so the original server probably read the files. No numeric value for bite chance, line snap chance, reel speed or tension is recovered. (client; inference)
2. **How the cast grade, the tension meter, the distance meter and the reel meter are driven.** The Lua defines `SetTension`, `SetDistanceMeter`, `SetHookedMeter`, `SetBobber` and the reel window, but no native function names those methods or the cast-grade message keys. Either the Flash movies or code paths not yet mapped drive them. This blocks a full server tension model.
3. **Meaning of sub-types 1, 15 and 16**, and of the missing 9 and 17. Sub 1 and 16 have readers but no visible effect.
4. **Result kinds 0 to 8** (what 1, 2, 6, 7 and 8 mean to the player) and the unknown fields in the result record (strings, the u32 at +0x50, +0x74 and +0x78, the f32 at +0x7c, the u8 at +0x80).
5. **Fish entry fields.** Which of the three u32, three bools and the string feed which Fish Finder column, and what the u32 array in the registration response is.
6. **School and path data.** The client builds school path tables in `FishingZoneConfig`, but no wire message carries paths. Whether schools move along server-side paths, or the client moves them, is not resolved. The meaning of the model-entry triple and the two trailing u32 in sub 11 is open.
7. **Row names and area mapping.** The name ids of the type 21 rows (417055, 408604 and so on) do not resolve in the locale file; whether row 595 ("Learn to Fish!") is really a tutorial row, and how rows 596 to 601 map to `EncounterDefId` values, is unknown. Row 595 uses the same ids (417055, 417056) that `CodeStringMappings.txt` gives to `fishingTutorial0009` and `0010`.
8. **Entry and exit.** How the boat or guide interaction launches the encounter is covered by the generic framework, not by fishing code. How a session ends (the player walking off, a timer, a quest) is unknown.
9. **Lures and rods.** How an equipped lure is communicated to the server (an ordinary item activation, as the Lua suggests, or a fishing packet) is not found; no fishing sub-type carries a lure id.
10. **Fish Finder "select fish".** The native handler behind `Ui.FindSelectedFish` was not traced; it may be the only place that highlights schools.
11. **Treasure and junk.** The item flag in the result, the Treasure Magnet lure and the video's junk catches show the shape, but the tables are unknown.
12. **Pets while fishing.** The wiki says a pet cannot be used in a minigame ([[Pets]]). Whether a summoned pet stays out during a fishing round is not established by the fishing code.

### Key functions

| Address | Name | Confidence | Role |
|---|---|---|---|
| `00b69860` | FishingProcessor::FishingProcessor | confirmed | Builds the processor, data sources, config objects. |
| `00b6cd80` | (receive dispatcher, `FishingProcessor::vftable[4]`) | unknown | Reads header, switches on sub-type. |
| `00b6c290` | FishingProcessor::Update | likely | Per-frame state machine and request sends. |
| `00b6bb90` | FishingProcessor::SetActive | likely | Fishing on and off. |
| `00b6b720` | FishingProcessor::RegisterLocalPlayer | likely | Builds and sends sub 2. |
| `00b6b7e0` | FishingProcessor::SendCastRequest | likely | Sends sub 6. |
| `00b6b8b0` | FishingProcessor::SendReelInRequest | likely | Sends sub 7. |
| `00b6b960` | SendCastAnimRequest | likely | Sends sub 5 (handle only; no caller). |
| `00b6b510`, `00b6ba10` | SendFishingSpecialRequest, SendSpecialRequest | likely | Sub 15 (never called). |
| `00b6b0d0`, `00b6b1e0`, `00b6b2f0`, `00b6b400` | SendFishing*Request | confirmed | Serialise and tunnel subs 5, 2, 6 and 7. |
| `00b6a380`, `00b6a400`, `00b6a460`, `00b6a4d0` | WriteFishing*Request | confirmed | Body writers for those four. |
| `00b66830`, `00b69550` | BaseFishingPacket::ReadHeader, WriteFishingPacketHeader | confirmed | The two-word header. |
| `00b66890`, `00b668f0`, `00b66990`, `00b66a80` | FishingPacket*_Read | confirmed | Subs 1, 5, 7, 10. |
| `00b66a00` | FishingPacketSpawnProxiedFishingBobber_Read | likely | Sub 8 (reviewer downgraded). |
| `00b66b80`, `00b68810` | ReadFishingSchoolUpdateBody, ReadFishingHeaderAndDword | confirmed | Subs 13 and 12. |
| `00b69130`, `00b68a80`, `00b65cc0` | ReadFishingSchoolSpawnBody and helpers | likely | Sub 11. |
| `00b6a170`, `00b65880`, `00b67240` | ReadHeaderConfigAndLists, (config block reader), ReadNestedFishingRecordFields | likely, unknown, confirmed | Sub 3. |
| `00b673b0`, `00b691d0` | ReadFishInfoUpdateEntry, ReadFishInfoUpdateEntries | confirmed | Fish entries, subs 3 and 4. |
| `00b66be0` | (result reader) | unknown | Sub 14. |
| `00b6a2b0`, `00b694e0`, `00b67550` | (fish-run readers) | unknown, likely, unknown | Sub 18. |
| `00b66f50` | ReadCompactFishingBody | likely | Sub 16. |
| `00b64300` | FishingProcessor::FindOrCreateProxiedPlayer | likely | Handle to player record. |
| `00b646e0`, `00b64710` | FindSchoolByKey, FindUnderwaterFishByKey | likely | Lookups. |
| `00b64580`, `00b64360` | (state entry, underwater switch) | unknown | State changes; see state machine. |
| `00b640e0` | BuildLaunchVector4 | likely | Cast launch vector. |
| `00ccffb0`, `00ccfbf0` | FishingProcessor::SetState, ProxiedFishingPlayer::ShowResult | likely | Proxied presentation. |
| `00ccfdb0` | (bobber creation, see Contradictions) | unknown | Creates `ProxiedFishingBobber`. |
| `00cd0150` | ControllerFishing::UpdateFrame (per proxied player) | confirmed | Catch card and timers. |
| `00cd1a10` | StateSwitchFrameUpdate | speculative | Underwater fish update, states 1 to 8. |
| `00cd1210`, `00cd1760`, `00cd3cf0` | Proxied school, fish, bobber constructors | confirmed | Proxy classes. |
| `00af8810`, `00af8b20` | ControllerFishing::Update, (slot 50) | confirmed, unknown | Controller update, ability click. |
| `00cd4420`, `00cd4620` | FishingDataSource::FieldName, GetCellValue | confirmed | FishTypes. |
| `00cd44e0`, `00cd47b0` | FishingScoringDataSource::FieldName, GetCellValue | confirmed | FishScored. |
| `00cd4b50` | FishingLureDataSource constructor | confirmed | FishingLures. |
| `01079260`, `01079d80`, `0107aae0`, `0107b140` | Fishing config loaders | likely | Key names; no callers. |
| `0107a850`, `0107a9f0` | FishingZoneConfig, FishingPlayerConfig constructors | confirmed | Config objects. |
| `00b65d90` | FishingProcessor::LoadMusicTracks | confirmed | Nine `MG_Fishing_*` sounds. |
| `0092f460` | BaseClient::DispatchBasePlayerUpdate | confirmed | Calls `SetActive` from the player flag. |
| `009471a0` | (packet switch) | unknown | Routes message `0x8a`. |
| `00936d00` | WaitForWorldReady | confirmed | Checks processor readiness (fishing included) before finished-loading. |

<!-- NARRATIVE END -->

## Mapped functions (generated)

216 accepted functions: 103 confirmed, 61 likely, 28 speculative, 24 unknown. Network-linked (`wire`): 50.

Subsystems: `net.fishing` (104), `minigame.fishing` (71), `game.fishing` (41)

### `game.fishing`

**ControllerFishing** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd0150` | ControllerFishing::UpdateFrame | confirmed | Advances fishing state and timers, updates local player UI and fish tension, emits status text and catch presentation, creates fishing effects, and updates res… |
| `00ccecf0` | ControllerFishing::UpdateFrame | likely | Updates fish tension and distance, processes local cast and reel state, manages fishing prompts and timers, creates tactile notifications, and tunnels activity… |
| `00ccf260` | FishingController::ResetResultState | likely | When the nested runtime object has type 12 and its progress exceeds 250, clears the result string and resets that progress field. |
| `00ccf3c0` | ControllerFishing::SetState | likely | Selects a UI state from the current activity and local player conditions, updates fishing result state, compares the local actor position with the target, and … |

**FishingProcessor** (21)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b69860` | FishingProcessor::FishingProcessor | confirmed | Installs the FishingProcessor vtable, initializes its state and list members, and constructs data-source members for fish types, lures, and scored fish. |
| `00b69d40` | FishingProcessor::Cleanup | confirmed | Restores the processor vtable, erases its player and school lists, clears many list and string members, releases data-source references, then restores the Asse… |
| `00b6a6c0` | ClearAndDeleteFishingProcessor | confirmed | Calls the FishingProcessor destructor body and frees the object only when flag bit zero is set; otherwise returns the original pointer. |
| `00b640e0` | BuildLaunchVector4 | likely | Fills a 4-float output vector: it takes the difference between param_3 and the point stored at +0x240..+0x24c, scales its length by +0x13c and adds +0x140 as t… |
| `00b64300` | FishingProcessor::FindOrCreateProxiedPlayer | likely | Returns 0 when the requested handle pair equals the shared default pair DAT_01be98b8/DAT_01be98bc; otherwise it walks the list whose head is at this+0x1e8 foll… |
| `00b646e0` | FindSchoolByKey | likely | Walks the list whose head is at this+0x1f8 following the next link at node+0x40 and returns the first node whose key field at +0xc equals param_2, or 0 when th… |
| `00b64710` | FindUnderwaterFishByKey | likely | Walks the list whose head is at this+0x208 following the next link at node+0x190 and returns the first node whose field at +0x14 equals param_2, or 0 when the … |
| `00b65460` | FishingProcessor::TouchLocalProxiedPlayer | likely | Reads the local player's handle pair through the client object at this+0x14 (interface at +0xc8, vtable slot 0x10) and, when FUN_00b64300 returns a proxied rec… |
| `00b6bb90` | FishingProcessor::SetActive | likely | thiscall (ECX = processor; one stack byte = flag). Stores the flag at +0x18 and returns when it is unchanged. Turning off: restores audio (00b61640), clears th… |
| `00b6c290` | FishingProcessor::Update | likely | thiscall (ECX = processor; one stack argument = integer tick delta). Adds the delta times the float at 0x017ebf0c to the accumulator at +0x1b8 and returns unle… |
| `00ccf6d0` | FishingProcessor::SetCastFlags | likely | Sets the byte at offset 0x3c when the input flag is nonzero, otherwise sets the byte at offset 0x3d. |
| `00ccf6f0` | FishingProcessor::SetPlayerState | likely | Stores one state value at offset 0x30 and copies four values from the supplied structure into offsets 0x20 through 0x2c. |
| `00ccffb0` | FishingProcessor::SetState | likely | Ignores unchanged states, resolves the actor, maps states 0, 1, 2, 4, and 6 to animation/resource actions, and invokes result display for state 6. |
| `00ccf7b0` | FishingProcessor::ValidatePlayerAction | speculative | Resolves the player, requires its active state, and rejects an action when the busy check and timing guard both exceed their conditions. |
| `00ccf820` | FishingProcessor::IsPlayerReady | speculative | Resolves the player by its handle pair and returns one only when the actor exists and the ready-state predicate passes. |
| `00b64360` | — | unknown | Returns immediately when the flag at +0x1bc already equals the argument; otherwise it stores the new flag and, for the off transition, clears the emitters/effe… |
| `00b64580` | — | unknown | Resolves the acting player with FUN_00b64300 and, when the local handle pair matches and the state field +0x1c differs from the new state, stores the new state… |
| `00b65200` | — | unknown | For every node of the school list at +0x1f8 it resolves the node's model id through FUN_00769710/FUN_007e7df0 twice (once with a zero resource and once with th… |
| `00b65ed0` | — | unknown | When the model list is non-empty it releases three sound slots with FUN_007798c0 and clears two bool lists with FUN_00b654e0; then it walks the fish entry list… |
| `00b65fb0` | — | unknown | Takes the camera's 4-float position from the client object (vtable slot 0x1c) and six per-slot scale calls into FUN_00b65390 for slots +0x254/+0x25c/+0x258/+0x… |
| `00ccf770` | — | unknown | Resolves the actor from the stored handle pair and, when found, calls the presentation helper with enable value one. |

**ProxiedFishingPlayer** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ccf890` | ProxiedFishingPlayer::Ctor | confirmed | Installs the ProxiedFishingPlayer vtable, stores three constructor values, zeros activity and resource fields, initializes two strings, and clears state fields. |
| `00ccf9c0` | ProxiedFishingPlayer::Dtor | confirmed | Restores the player vtable, releases two owned objects and one actor resource, detaches the actor when present, then decrements and releases both strings. |
| `00ccfbd0` | ProxiedFishingPlayer::ScalarDeletingDtor | confirmed | Destroys the proxied player and frees its allocation when the deleting flag is set. |
| `00ccf720` | ProxiedFishingPlayer::ReleaseResources | likely | Invokes the deleting virtual method on non-null resources at offsets 0xd4 and 0x130, then clears both pointers. |
| `00ccfb00` | ProxiedFishingPlayer::SetAnimation | likely | Resolves the actor, clears a temporary actor flag when present, stores the requested animation id, and queues a presentation update when the actor is active. |
| `00ccfbf0` | ProxiedFishingPlayer::ShowResult | likely | Releases an old result object, creates result presentation when a result id is positive, copies player and fish context, and assigns a result-specific display … |
| `00ccf860` | ProxiedFishingPlayer::ReleaseHandle | speculative | When the stored resource at offset 0x40 is nonzero, calls its virtual release method and clears the resource pointer. |
| `00ccfdb0` | — | unknown | Clears an existing display, creates the result presenter for a positive result id, initializes it with player data and timing fields, and starts presentation w… |

**(no class)** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b62eb0` | StorePos4AndDistanceFromPrev | likely | Copies the four floats pointed to by param_2 into +0x30, +0x34, +0x38 and +0x3c, then computes the Euclidean distance between that 4-tuple and the previously s… |
| `00b67b80` | UpdateFishingLureAttachments | likely | Updates fishing progress state and accesses the lure model attachments named sg_fishing_lure_bbe, mouth, and hinge_2 while updating the fishing model presentat… |
| `00b68600` | AppendResolvedActorValue | likely | Walks the supplied linked list, resolves each key through lookup helpers, and appends a derived dword to another list when resolution succeeds. |
| `00b6bae0` | CopyAssign_List50_Map60 | likely | Copy-assign (ECX = destination, one stack argument = source; ret 4; returns the destination). Copies the members at +4 and +0x30 with 006f9e60, copies dwords +… |
| `00cd0df0` | FishingSchool::RefreshModelEffects | likely | Walks school model entries, resolves each model id, builds a four-component effect value from the supplied vector, and invokes the resolved model's update meth… |
| `00b616e0` | CallSlot2On0x32C | speculative | Loads the pointer at this+0x32c; when it is non-zero it reloads it as this, fetches its vtable and tail-jumps to slot 2 (vtable offset 8); when the pointer is … |
| `00b62f30` | StoreTwoVec4BlocksAndScalars | speculative | Writes param_2 to +0x18 and param_5 to +0x1c, copies 16 dwords (64 bytes) from param_3 into +0x20, copies another 16 dwords from param_6 into +0x60, and writes… |
| `00b65390` | — | unknown | Resolves an object through the client's interface lookup (client+0xe8 vtable +4, slot 0x10) with param_4; when the object exists and param_3 is greater than DA… |

### `minigame.fishing`

**ControllerFishing** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af83d0` | ControllerFishing::Dtor | confirmed | Fastcall destructor body (ECX = this): stores ControllerFishing::vftable at +0, sets the SEH state to -1, calls base destructor 00b00110 (ControllerRegular) an… |
| `00af8500` | ControllerFishing::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor body: stores ControllerFishing::vftable at +0, calls base destructor 00b00110, then frees this with _free when bit 0 of the… |
| `00af8810` | ControllerFishing::Update | confirmed | Thiscall virtual at ControllerFishing vtable slot 51. When the byte at this+0x151 is zero, calls 00b01250 with the time argument and returns. Otherwise interpo… |
| `00af8940` | — | unknown | Thiscall constructor with three stack arguments: calls base constructor 00b00ac0 with those arguments and 0xffffffff, stores ControllerFishing::vftable at +0, … |
| `00af8b20` | — | unknown | Thiscall virtual at ControllerFishing vtable slot 50. Clears the byte at this+0x150, then walks the entry list at this+0xc8: for entries with kind field +0x18 … |

**ControllerRegular** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af8420` | ControllerRegular::SetF134AndFlag130 | speculative | Virtual at vtable slot 18 of both ControllerFishing and ControllerRegular: stores the dword argument at this+0x134 and sets the byte at this+0x130 to 1. |
| `00af8440` | ControllerRegular::ClearFlag130 | speculative | Virtual at vtable slot 19 of both ControllerFishing and ControllerRegular: sets the byte at this+0x130 to 0. |

**FishingDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd4420` | FishingDataSource::FieldName | confirmed | Virtual slot 1 of FishingDataSource (vtable 0183955c): returns the static name for column index 0 to 5: FishType, FishName, FishIcon, FishSpecial, FishCatchabl… |
| `00cd4620` | FishingDataSource::GetCellValue | confirmed | Virtual slot 4 of FishingDataSource: returns 0 when the count at +0x29c of the object at +0x224 is below arg2. Otherwise column arg3 (0 to 5) selects a list ac… |
| `00cd4930` | FishingDataSource::~FishingDataSource | confirmed | Stores FishingDataSource::vftable at this, +0x30 and +0x34, then calls base routine 0091bf60. |
| `00cd4a10` | FishingDataSource::FishingDataSource | confirmed | Calls 00922890 with the name pointer for BaseClient.FishingProcessor.FishTypes, stores FishingDataSource::vftable at this, +0x30 and +0x34, stores arg2 at +0x2… |
| `00cd4a90` | FishingDataSource::ScalarDeletingDtor | confirmed | Calls destructor body 00cd4930 on this, then frees this when bit 0 of the flag argument is set; returns this. |
| `00cd45e0` | FishingDataSource::Value29C | speculative | Virtual slot 2 of FishingDataSource: returns 0 when the pointer at this+0x224 is null; otherwise returns the dword at +0x29c of the object it points to. |

**FishingLureDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd4b50` | FishingLureDataSource::FishingLureDataSource | confirmed | Calls 00cb1bb0 with the name pointer for BaseClient.FishingProcessor.FishingLures, stores FishingLureDataSource::vftable at this, +0x30, +0x34 and +0x254, and … |
| `00cd4c90` | FishingLureDataSource::AdjustorThunk_Sub254 | confirmed | Adjustor thunk: subtracts 0x254 from ECX and jumps (jmp, not call) to 00cd4ca0. |
| `00cd4ca0` | FishingLureDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor of FishingLureDataSource: runs base routine 00cb1b10 on this inside an EH frame, then frees this when bit 0 of the flag argument is … |
| `00cd4c20` | — | unknown | Virtual slot 15 of FishingLureDataSource. Returns 0 unless 00b0d810 returns non-null twice; then returns 1 when 00bae6a0 is non-zero, the record from the first… |

**FishingPlayerConfig** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0107a9f0` | FishingPlayerConfig::FishingPlayerConfig | confirmed | Fastcall constructor (this in ECX): stores FishingPlayerConfig::vftable, then sets the IString at +4 to the static empty string at 01cb79a0 with zero length an… |
| `0107aa50` | FishingPlayerConfig::Destruct | confirmed | Fastcall destructor body (this in ECX): stores FishingPlayerConfig::vftable, then releases the IString at +4 (LOCK-ed refcount decrement at data-4; the string … |
| `0107aac0` | FishingPlayerConfig::ScalarDeletingDtor | confirmed | Thiscall with one stack byte argument (ret 4): calls 0107aa50 (FishingPlayerConfig::Destruct) on this, then calls _free(this) through thunk 00d82463 when bit 0… |

**FishingScoringDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd44e0` | FishingScoringDataSource::FieldName | confirmed | Virtual slot 1 of FishingScoringDataSource (vtable 018395d4): returns the static name for column index 0 to 6: FishName, FishIconId, FishDifficulty, FishRarity… |
| `00cd47b0` | FishingScoringDataSource::GetCellValue | confirmed | Virtual slot 4 of FishingScoringDataSource: returns 0 unless arg2 is below the count at +0x220 of the object at +0x224. It walks the list headed at +0x218 (nex… |
| `00cd49a0` | FishingScoringDataSource::~FishingScoringDataSource | confirmed | Stores FishingScoringDataSource::vftable at this, +0x30 and +0x34, then calls base routine 0091bf60. |
| `00cd4ab0` | FishingScoringDataSource::FishingScoringDataSource | confirmed | Calls 00922890 with the name pointer for BaseClient.FishingProcessor.FishScored, stores FishingScoringDataSource::vftable at this, +0x30 and +0x34, stores arg2… |
| `00cd4b30` | FishingScoringDataSource::ScalarDeletingDtor | confirmed | Calls destructor body 00cd49a0 on this, then frees this when bit 0 of the flag argument is set; returns this. |
| `00cd4600` | FishingScoringDataSource::Value220 | speculative | Virtual slot 2 of FishingScoringDataSource: returns 0 when the pointer at this+0x224 is null; otherwise returns the dword at +0x220 of the object it points to. |

**FishingZoneConfig** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0107a6e0` | FishingZoneConfig::Destruct | confirmed | Fastcall destructor body (this in ECX): stores FishingZoneConfig::vftable, destroys the HashListMap<int,FishingSchoolPathDefinition,16,-1> at this+0x60 through… |
| `0107a850` | FishingZoneConfig::FishingZoneConfig | confirmed | Fastcall constructor (this in ECX, returns this): stores FishingZoneConfig::vftable; sets the IString members at +4 and +0x30 to the empty string DAT_01cb7954 … |
| `0107a8f0` | FishingZoneConfig::ScalarDeletingDtor | confirmed | Thiscall with one stack byte argument (ret 4): calls 0107a6e0 (FishingZoneConfig::Destruct) on this, then calls _free(this) through thunk 00d82463 when bit 0 o… |

**ProxiedFishingBobber** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd3cf0` | ProxiedFishingBobber::ProxiedFishingBobber | confirmed | Stores ProxiedFishingBobber::vftable at this; stores arg3 at +0x10 and arg4 at +0x14; stores arg2 at +0x1c and the dword at arg2+4 at +0x20; sets +0xd0 to -1 a… |
| `00cd3de0` | ProxiedFishingBobber::ReleaseMemberThenFreeIfFlag | confirmed | Stores ProxiedFishingBobber::vftable at this; calls virtual slot 2 of the object at +0x20 with the dword at +0x18; frees this when bit 0 of the flag argument i… |

**ProxiedFishingSchool** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd1210` | ProxiedFishingSchool::ProxiedFishingSchool | confirmed | Stores ProxiedFishingSchool::vftable at this+0; zeroes +0x8, +0x14, +0x18, +0x1c, +0x24, +0x28, +0x2c, +0x30, +0x34; stores SoeUtil::List<ProxiedFishingSchoolM… |
| `00cd16d0` | ProxiedFishingSchool::ScalarDeletingDtor | confirmed | Calls the ProxiedFishingSchool destructor body 00cd1550 on this, then frees this when bit 0 of the flag argument is set; returns this. |
| `00cd1550` | — | unknown | Stores ProxiedFishingSchool::vftable at this; walks the list headed at +0x14 calling virtual slot 2 of the object at +0x3c for each node; walks the list headed… |

**ProxiedFishingUnderwaterFish** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd1760` | ProxiedFishingUnderwaterFish::ProxiedFishingUnderwaterFish | confirmed | Stores ProxiedFishingUnderwaterFish::vftable at this; sets the dword at +0x20 to 1 and the dword at +0x150 to 1; stores arg2 at +0x170; zeroes most other field… |
| `00cd1950` | ProxiedFishingUnderwaterFish::~ProxiedFishingUnderwaterFish | confirmed | Stores ProxiedFishingUnderwaterFish::vftable at this; when the dword at +0x15c is non-zero, calls 007dc240 and 008c52a0 on it and 009c36a0 on the result, then … |
| `00cd19f0` | ProxiedFishingUnderwaterFish::ScalarDeletingDtor | confirmed | Calls destructor body 00cd1950 on this, then frees this when bit 0 of the flag argument is set; returns this. |

**SoeUtil::HashList<FishingSchoolPathDefinition,16,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01078ee0` | SoeUtil::HashList::AllocBlock130 | confirmed | Returns a 0x130-byte block with 4-byte alignment: from the current memory manager (006fb860, its vtable slot +4 called with 0x130 and 4), else from the global … |
| `01078f20` | SoeUtil::HashList::FreeBlock4 | confirmed | Thiscall with one stack pointer argument (ret 4): calls 00401c00(pointer, 4), the free with alignment 4. |
| `0107a830` | SoeUtil::HashList::ScalarDeletingDtor | confirmed | Thiscall with one stack byte argument (ret 4): calls 0107a670 (the HashListMap destructor body) on this, then calls _free(this) through thunk 00d82463 when bit… |

**SoeUtil::HashListMap<int,FishingSchoolPathDefinition,16,-1>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01078cf0` | SoeUtil::HashListMap::Construct | confirmed | Fastcall constructor (this in ECX, returns this): stores the HashListMap vtable, zeroes +4, +8, +0xc and +0x10, and clears the 64-byte bucket array at +0x14 wi… |
| `0107a600` | SoeUtil::HashListMap::Destruct | confirmed | Fastcall destructor body: stores the HashListMap vtable, then repeatedly calls 00b69410 on the head node (param_1[2]) while the head is non-null. Does not free… |
| `0107a630` | SoeUtil::HashListMap::UnlinkAllFreeIfBit0 | confirmed | Thiscall with one stack byte argument: stores the vtable, unlinks head nodes with 00b69410 until the head is null (the destructor loop inlined), and calls _fre… |
| `0107a670` | SoeUtil::HashListMap::DestructEH | confirmed | Fastcall destructor body under an EH frame: the same vtable store and head-unlink loop as SoeUtil::HashListMap::Destruct (0107a600) through 00b69410, with no f… |
| `0107a7b0` | SoeUtil::HashListMap::ConstructAsHashList | confirmed | Fastcall constructor (this in ECX, returns this): stores the HashListMap vtable, zeroes +4, +8, +0xc and +0x10, clears the 0x40-byte bucket array at +0x14 with… |

**SoeUtil::HashListMap<int,FishingSchoolPathNodeDefinition,64,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01078fd0` | SoeUtil::HashListMap::ConstructNodeMap | confirmed | Fastcall constructor (this in ECX, returns this): stores the HashListMap vtable for FishingSchoolPathNodeDefinition, zeroes +0x10, +8 and +0xc, clears the 0x10… |

**(no class)** (25)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01078bd0` | InitFloatBlocksAndIds | likely | Fastcall with this in ECX and no stack arguments: writes zero to +0 and +4; float 0.0 to +0x10, +0x14, +0x18, +0x20, +0x24, +0x2c and +0x30; float 1.0 (constan… |
| `01078d50` | InitFloatBlocksReturnThis | likely | Fastcall with this in ECX and no stack arguments (ret 0): calls 01078bd0 on this and returns this in EAX. |
| `01079260` | LoadFishingZoneConfig | likely | Thiscall with a reader argument and an encounter id argument. Opens bbe_FishingZoneConfig.txt and finds the zone whose EncounterDefId matches; when none matche… |
| `01079d80` | LoadFishingZoneConfigViaNodeLookup | likely | Thiscall with a node/document argument and an encounter id (the signature types it as float, but the code compares it as an int). Looks up the root and Encount… |
| `0107aae0` | LoadFishingPlayerConfigFields | likely | Thiscall with the player config object in ECX, a reader argument and a source pointer whose +4 dword selects the zone. When BaseConfig exists it reads that sub… |
| `0107b140` | LoadFishingPlayerConfigFile | likely | Thiscall with the player config object in ECX, a reader argument and a source pointer whose +4 dword selects the entry. Opens bbe_FishingPlayerConfig.txt, find… |
| `00af8470` | CopyVec3ToSlot1A0 | speculative | Thiscall helper: copies the three dwords at param_2 to this+0x1a0, +0x1a4 and +0x1a8, and stores the float constant at 0x0175b420 to this+0x1ac. |
| `00af84b0` | TestBit3Of20 | speculative | Fastcall: returns bit 3 of the dword at this+0x20. |
| `00af84d0` | TestBit3Of2c | speculative | Fastcall: returns bit 3 of the dword at this+0x2c. |
| `00af84e0` | TestBit5Of2c | speculative | Fastcall: returns bit 5 of the dword at this+0x2c. |
| `00af8560` | SetFlag151Snapshot | speculative | Thiscall setter for the byte at this+0x151: returns when unchanged; otherwise stores the new flag and, for a non-zero value, copies the vector at +0x1a0..+0x1a… |
| `00af85e0` | ControllerFishing::SetRot1B0FromAngles | speculative | Thiscall with four stack arguments: stores the second argument at this+0x1c4, builds a 4x4 matrix with 00770e70 from the third argument and a constant axis, an… |
| `00cd12a0` | ProbeNodeListRebuild | speculative | Walks the node list headed at arg2+4 (next link at +0xc). For each node it calls 00769710, 00726e30, 0079d310 and 00402ce0 and virtual slot 1 of the object at … |
| `00cd1720` | ReadByte_0xd9 | speculative | Returns the byte at ECX+0xd9 (fastcall; mov al,[ecx+0xd9]; ret). |
| `00cd1a10` | StateSwitchFrameUpdate | speculative | Per-frame update that switches on the state dword at +0x20 (cases 1 to 8). It advances motion vectors at +0x130..+0x14c and timers at +0xc4, +0xc8, +0xcc, +0xd… |
| `00cd3a70` | ApplyArgsToStateAndCue | speculative | Stores arg2 (byte) at +0x10 and arg3 at +0x14. Looks up an object through virtual slot 1 of the object at (+0x170)+4; when non-null, calls its slot 7 with a co… |
| `00cd3c90` | FieldAddr110 | speculative | Returns ECX+0x110 (fastcall; lea eax,[ecx+0x110]; ret). |
| `00cd3ca0` | ZeroFloatAt154 | speculative | Stores 0.0f at ECX+0x154 (xorps xmm0; movss [ecx+0x154],xmm0; ret). |
| `00cd3cd0` | FieldAddr140 | speculative | Returns ECX+0x140 (fastcall; lea eax,[ecx+0x140]; ret). |
| `00cd3ce0` | FieldAddr130 | speculative | Returns ECX+0x130 (fastcall; lea eax,[ecx+0x130]; ret). |
| `00cd3e20` | ProbeHitAndDistance | speculative | Gets the object at +0x20 and calls its virtual slot 4 with handle +0xd8, computes a target point from its +0x180..+0x18c scaled by +0xe0 with offsets, calls 00… |
| `00cd42a0` | InitParamsAndSync | speculative | Stores arg5 at +0x28, arg6 at +0xdc, arg7 at +0xe4 and arg8 at +0xe0, and the dword at arg4+0x268 at +0xd8. Looks up an object through virtual slot 1 of the ob… |
| `00cd4480` | ForEachNestedRowCall | speculative | Reads the object pointer at this+0x30. In a do-while over the outer count from its virtual slot 2, it reads the inner count from its slot 0 and calls 010cb9a0(… |
| `00cd4bd0` | Wrapper_00cb1b10 | speculative | Thiscall with no stack arguments: installs an EH frame, saves ECX in the frame, calls 00cb1b10 with ECX as the receiver, removes the frame and returns. |
| `00cd3f80` | — | unknown | Converts arg2 to a step; when the byte at +0xd4 is set, moves the target at +0xf0..+0xfc toward the point at +0x140..+0x14c by a normalized step (speed +0xdc, … |

### `net.fishing`

**BaseFishingPacket** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b60980` | BaseFishingPacket::~BaseFishingPacket | confirmed | Restores the base vtables in order: stores BaseFishingPacket::vftable and then BasePacket::vftable into the object, with no other work; it is referenced from 4… |
| `00b609d0` | BaseFishingPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, then calls _free(param_1) when the flag byte has bit 0 set, and return… |
| `00b60a40` | BaseFishingPacket::~BaseFishingPacket | confirmed | Stores BaseFishingPacket::vftable and then BasePacket::vftable into the object and returns; identical body to the family's other base destructors, emitted agai… |
| `00b60a90` | BaseFishingPacket::~BaseFishingPacket | confirmed | Stores BaseFishingPacket::vftable and then BasePacket::vftable into the object and returns; byte-identical copy of the family's base destructor, called from FU… |
| `00b60ae0` | BaseFishingPacket::~BaseFishingPacket | confirmed | Stores BaseFishingPacket::vftable and then BasePacket::vftable into the object and returns; byte-identical copy of the fishing packet base destructor called fr… |
| `00b60b30` | BaseFishingPacket::~BaseFishingPacket | confirmed | Stores BaseFishingPacket::vftable and then BasePacket::vftable into the object and returns; byte-identical copy of the fishing packet base destructor reference… |
| `00b60b80` | BaseFishingPacket::~BaseFishingPacket | confirmed | Stores BaseFishingPacket::vftable and then BasePacket::vftable into the object and returns; byte-identical copy of the fishing packet base destructor called fr… |
| `00b60bd0` | BaseFishingPacket::~BaseFishingPacket | confirmed | Stores BaseFishingPacket::vftable and then BasePacket::vftable into the object and returns; byte-identical copy of the fishing packet base destructor called fr… |
| `00b60c20` | BaseFishingPacket::~BaseFishingPacket | confirmed | Stores BaseFishingPacket::vftable and then BasePacket::vftable into the object and returns; byte-identical copy of the fishing packet base destructor called fr… |
| `00b60cd0` | BaseFishingPacket::~BaseFishingPacket | confirmed | Stores BaseFishingPacket::vftable and then BasePacket::vftable into the object and returns; byte-identical copy of the fishing packet base destructor called fr… |
| `00b60d80` | BaseFishingPacket::~BaseFishingPacket | confirmed | Stores BaseFishingPacket::vftable and then BasePacket::vftable into the object and returns; byte-identical copy of the fishing packet base destructor called fr… |
| `00b60dd0` | BaseFishingPacket::~BaseFishingPacket | confirmed | Stores BaseFishingPacket::vftable and then BasePacket::vftable into the object and returns; byte-identical copy of the fishing packet base destructor reference… |
| `00b60e20` | BaseFishingPacket::~BaseFishingPacket | confirmed | Stores BaseFishingPacket::vftable and then BasePacket::vftable into the object and returns; byte-identical copy of the fishing packet base destructor called fr… |
| `00b66830` | BaseFishingPacket::ReadHeader | confirmed | Reads the fishing packet header: takes the u16 at the stream cursor (param_2+8) into this+4 and the next u16 into this+8, advancing the cursor by two bytes eac… |
| `00b608b0` | BaseFishingPacket::BaseFishingPacketDefault 🌐 | likely | Constructs a base fishing packet object with no sub-type: stores BasePacket::vftable, writes the SOE opcode 0x8a into field +4, overwrites the vtable with Base… |
| `00b60910` | — 🌐 | unknown | Constructs a base fishing packet with the caller's sub-type: stores BasePacket::vftable, writes opcode 0x8a into field +4, stores BaseFishingPacket::vftable an… |

**FishingPacketCastAnimRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b62820` | FishingPacketCastAnimRequest::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, calls _free on the object when bit 0 of the flag argument is set, and … |
| `00b668f0` | FishingPacketCastAnimRequest_Read | confirmed | Reads the sub-5 payload: header via FUN_00b66830, the 8-byte handle into +0x10/+0x14, a float into +0x18 (setting the NaN flag at param_2+0x11 when it is not a… |
| `00b627c0` | FishingPacketCastAnimRequest::FishingPacketCastAnimRequest 🌐 | likely | Calls the fishing family base constructor FUN_00b60910 with sub-type 5 and stores FishingPacketCastAnimRequest::vftable into the new object. |

**FishingPacketCastRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b628f0` | FishingPacketCastRequest::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, frees the object with _free when bit 0 of the flag argument is set, an… |
| `00b62890` | FishingPacketCastRequest::FishingPacketCastRequest 🌐 | likely | Calls the fishing family base constructor FUN_00b60910 with sub-type 6 and stores FishingPacketCastRequest::vftable into the new object. |

**FishingPacketDespawnProxiedFishingSchool** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b62bd0` | FishingPacketDespawnProxiedFishingSchool::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, frees the object with _free when bit 0 of the flag argument is set, an… |
| `00b60c70` | — 🌐 | unknown | Calls the family base constructor FUN_00b60910 with sub-type 0xc, then stores FishingPacketDespawnProxiedFishingSchool::vftable into the new object and returns… |

**FishingPacketFishInfoUpdate** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b69770` | FishingPacketFishInfoUpdate::FishingPacketFishInfoUpdate 🌐 | confirmed | Calls the base fishing-packet constructor with subtype 4, installs the FishInfoUpdate vtable, and initializes its client-fish-entry list. |
| `00b6a570` | ClearAndDeleteFishInfoUpdatePacket | confirmed | Calls the FishInfoUpdate cleanup body and frees packet storage only when flag bit zero is set. |

**FishingPacketFishingResult** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b64ec0` | FishingPacketFishingResult::FishingPacketFishingResult 🌐 | confirmed | Calls the fishing family base constructor FUN_00b60910 with sub-type 0xe (14), stores FishingPacketFishingResult::vftable, then default-constructs the packet's… |
| `00b64f50` | FishingPacketFishingResult::~FishingPacketFishingResult | confirmed | Destroys the packet's three SoeUtil::IString payload members (at +0x64, +0x54 and +0x28), each time restoring the string vtable and releasing the shared buffer… |
| `00b65d70` | FishingPacketFishingResult::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls FUN_00b64f50 (which releases the packet's three IString members and restores the base vtables), frees the object with _free w… |

**FishingPacketReelInRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b629c0` | FishingPacketReelInRequest::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, frees the object with _free when bit 0 of the flag argument is set, an… |
| `00b66990` | FishingPacketReelInRequest_Read | confirmed | Reads the sub-7 payload: header via FUN_00b66830, the 8-byte handle into +0x10/+0x14, then one byte into +0x18 converted to a bool, with the same short-read ha… |
| `00b62960` | FishingPacketReelInRequest::FishingPacketReelInRequest 🌐 | likely | Calls the fishing family base constructor FUN_00b60910 with sub-type 7 and stores FishingPacketReelInRequest::vftable into the new object. |

**FishingPacketRegisterPlayerRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b62750` | FishingPacketRegisterPlayerRequest::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, frees the object with _free when bit 0 of the flag argument is set, an… |
| `00b626f0` | FishingPacketRegisterPlayerRequest::FishingPacketRegisterPl… 🌐 | likely | Calls the fishing family base constructor FUN_00b60910 with sub-type 2 and stores FishingPacketRegisterPlayerRequest::vftable into the new object. |

**FishingPacketRegisterPlayerResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b695f0` | FishingPacketRegisterPlayerResponse::FishingPacketRegisterP… 🌐 | confirmed | Calls the base fishing-packet constructor with subtype 3, installs the response vtable, and initializes its integer and fish-entry lists. |
| `00b6a550` | ClearAndDeleteRegisterResponsePacket | confirmed | Calls the registration-response cleanup body and frees packet storage only when flag bit zero is set. |

**FishingPacketSpawnFishRun** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b6a590` | FishingPacketSpawnFishRun::FishingPacketSpawnFishRun 🌐 | confirmed | Calls the base fishing-packet constructor with subtype 0x12, installs the packet vtable, and initializes its string and spawn-info list members. |
| `00b6aeb0` | ClearAndDeleteSpawnFishRunPacket | confirmed | Calls the SpawnFishRun cleanup body and frees packet storage only when flag bit zero is set; otherwise returns the original pointer. |

**FishingPacketSpawnProxiedFishingBobber** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b62a30` | FishingPacketSpawnProxiedFishingBobber::FishingPacketSpawnP… 🌐 | confirmed | Calls the fishing family base constructor FUN_00b60910 with sub-type 8 and stores FishingPacketSpawnProxiedFishingBobber::vftable into the new object. |
| `00b62a90` | FishingPacketSpawnProxiedFishingBobber::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, calls _free when bit 0 of the flag argument is set, and returns the po… |
| `00b66a00` | FishingPacketSpawnProxiedFishingBobber_Read | likely | Reads the sub-8 payload: header via FUN_00b66830, the 8-byte handle into +0x10/+0x14, a dword into +0x18, then two four-float vectors into +0x20 and +0x30 thro… |

**FishingPacketSpawnProxiedFishingSchool** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b678e0` | FishingPacketSpawnProxiedFishingSchool::FishingPacketSpawnP… 🌐 | confirmed | Calls the base fishing-packet constructor with subtype 0x0b, installs the packet vtable, and initializes its model-entry and integer list members. |
| `00b68ba0` | FishingPacketSpawnProxiedFishingSchool::~FishingPacketSpawn… | confirmed | Calls the packet cleanup body and frees the packet storage only when flag bit zero is set; otherwise returns the original pointer. |

**FishingPacketSpecialRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b62d10` | FishingPacketSpecialRequest::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, calls _free when bit 0 of the flag argument is set, and returns the po… |
| `00b62cb0` | FishingPacketSpecialRequest::FishingPacketSpecialRequest 🌐 | speculative | Calls the fishing family base constructor FUN_00b60910 with sub-type 0xf (15) and stores FishingPacketSpecialRequest::vftable into the new object. |

**FishingPacketSpecialResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b62de0` | FishingPacketSpecialResponse::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, calls _free when bit 0 of the flag argument is set, and returns the po… |
| `00b62d80` | FishingPacketSpecialResponse::FishingPacketSpecialResponse 🌐 | likely | Calls the fishing family base constructor FUN_00b60910 with sub-type 0x10 (16) and stores FishingPacketSpecialResponse::vftable into the new object. |

**FishingPacketUpdateData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b62680` | FishingPacketUpdateData::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set, return… |
| `00b66890` | FishingPacketUpdateData_Read | confirmed | Reads the sub-1 payload into the packet object: first the header via FUN_00b66830, then the 8-byte handle into +0x10/+0x14 (zeroing both and flagging end-of-st… |
| `00b62620` | — 🌐 | unknown | Calls the fishing family base constructor FUN_00b60910 with sub-type 1 and then stores FishingPacketUpdateData::vftable into the new object. |

**FishingPacketUpdateProxiedFishingBobber** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b62b60` | FishingPacketUpdateProxiedFishingBobber::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, calls _free when bit 0 of the flag argument is set, and returns the po… |
| `00b66a80` | FishingPacketUpdateProxiedFishingBobber_Read | confirmed | Reads the sub-10 payload: header via FUN_00b66830, the 8-byte handle into +0x10/+0x14, a dword into +0x18, then two single bytes converted to bools at +0x1c an… |
| `00b62b00` | — 🌐 | unknown | Calls the fishing family base constructor FUN_00b60910 with sub-type 10 and stores FishingPacketUpdateProxiedFishingBobber::vftable into the new object. |

**FishingPacketUpdateProxiedFishingSchool** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b62c40` | FishingPacketUpdateProxiedFishingSchool::ScalarDeletingDtor | confirmed | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, frees the object with _free when bit 0 of the flag argument is set, an… |
| `00b60d20` | — 🌐 | unknown | Calls the family base constructor FUN_00b60910 with sub-type 0xd, then stores FishingPacketUpdateProxiedFishingSchool::vftable into the new object and returns … |

**FishingPlayerConfig** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b65880` | — | unknown | Reads 16 fields from the stream cursor at param_2+8 (bounded by the end pointer at +0xc) into +0x20, +0x5c, +0x60, +0x64, +0x68, +0x6c, +0x58, +0x78, +0x7c, +0… |

**FishingProcessor** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b6b720` | FishingProcessor::RegisterLocalPlayer | likely | thiscall (ECX = processor; stack: arg1 = client interface object; ret 0xc). Reads the local handle through vtable slot 4 of arg1+0x54, calls FUN_00b64580 with … |
| `00b6b7e0` | FishingProcessor::SendCastRequest | likely | thiscall (ECX = processor; arg1 = pointer to four floats, arg2 = flag byte; ret 8). Builds a FishingPacketCastRequest (00b62890) on the stack, copies the local… |
| `00b6b8b0` | FishingProcessor::SendReelInRequest | likely | thiscall (ECX = processor; arg1 = flag byte; ret 4). Builds a FishingPacketReelInRequest (00b62960) on the stack, copies the local handle (client interface, su… |
| `00b6cd80` | — | unknown | Receive dispatcher for the fishing opcode family: wraps (param_2,param_3) in the 6-field stream reader at local_11f4, reads the two 16-bit header words with FU… |

**(no class)** (45)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b66b80` | ReadFishingSchoolUpdateBody 🌐 | confirmed | Reads the common header, a dword into destination offset 0x0c, and a four-dword vector into offset 0x10; a short read sets the reader error state. |
| `00b67240` | ReadNestedFishingRecordFields | confirmed | Consumes a bounded length-prefixed region, then reads five dwords into destination offsets 0x40, 0x44, 0x48, 0x4c, and 0x20; it flags NaN in selected float fie… |
| `00b673b0` | ReadFishInfoUpdateEntry 🌐 | confirmed | Reads three dwords, two one-byte flags, a dword, a length-counted byte region, another dword, a flag, and a final dword into one entry. |
| `00b67960` | ClearFishingPacketListMembers | confirmed | Destroys the integer list and model-entry list, then restores the BaseFishingPacket and BasePacket vtables on the object. |
| `00b68810` | ReadFishingHeaderAndDword 🌐 | confirmed | Reads the common fishing header, then reads one dword into the destination at offset 0x0c; a short read sets the stream error and stores zero. |
| `00b691d0` | ReadFishInfoUpdateEntries 🌐 | confirmed | Clears the existing entry list, reads its count, and appends and decodes one FishInfoUpdate item per count while the bounded reader remains valid. |
| `00b69550` | WriteFishingPacketHeader 🌐 | confirmed | Writes the packet's two 16-bit header fields to the bounded output buffer, advancing the output cursor after each write. |
| `00b6a380` | WriteFishingCastAnimationRequest 🌐 | confirmed | Writes the common header, an 8-byte handle, one scalar, and four consecutive dwords from the vector field into the bounded output. |
| `00b6a400` | WriteFishingRegisterPlayerRequest 🌐 | confirmed | Writes the common header, an 8-byte handle, and one byte from the request object to the bounded output. |
| `00b6a460` | WriteFishingCastRequest 🌐 | confirmed | Writes the common header, an 8-byte handle, four dwords from a vector field, and one final byte into the bounded output. |
| `00b6a4d0` | WriteFishingReelInRequest 🌐 | confirmed | Writes the common header, an 8-byte handle, and one byte from the request object to the bounded output. |
| `00b6acb0` | BoundedBodyParser_00b6acb0 🌐 | confirmed | Builds a bounded reader, calls the registration-response body reader with the packet object, and returns success when no read error occurred and the trailing-d… |
| `00b6ad20` | BoundedBodyParser_00b6ad20 🌐 | confirmed | Builds a bounded reader, reads the common fishing header, decodes FishInfoUpdate entries, then returns success when no read error occurred and the trailing-dat… |
| `00b6b0d0` | SendFishingCastAnimationRequest 🌐 | confirmed | Serializes the cast-animation body, constructs a PacketTunneledClientPacket, calls gateway send helper 00925640, then destroys the temporary SoeUtil byte-array… |
| `00b6b1e0` | SendFishingRegisterPlayerRequest 🌐 | confirmed | Serializes the register-player body, constructs a PacketTunneledClientPacket, calls gateway send helper 00925640, then destroys the temporary SoeUtil byte-arra… |
| `00b6b2f0` | SendFishingCastRequest 🌐 | confirmed | Serializes the cast-request body, constructs a PacketTunneledClientPacket, calls gateway send helper 00925640, then destroys the temporary SoeUtil byte-array b… |
| `00b6b400` | SendFishingReelInRequest 🌐 | confirmed | Serializes the reel-in body, constructs a PacketTunneledClientPacket, calls gateway send helper 00925640, then destroys the temporary SoeUtil byte-array buffer… |
| `00b60f30` | Copy64ByteRecordTo0x50 | likely | Has no callees: it copies 16 dwords (64 bytes) from its stack argument to this+0x50 with a rep movsd (bytes 568b742408578d7950b910000000f3a55f5ec20400), i.e. i… |
| `00b65cc0` | ReadModelEntryRecord | likely | Reads three dwords from the stream cursor at param_2+8 into param_1[0], param_1[1] and param_1[2]; a short read leaves the field 0 and sets the end flag at par… |
| `00b66f50` | ReadCompactFishingBody 🌐 | likely | Reads the common header, two dwords, another dword, and a one-byte boolean from the bounded reader, setting its error state on short reads. |
| `00b68a80` | ReadFishingSchoolModelEntryList | likely | Clears the current model-entry list, reads its count, then creates one node and decodes one three-dword model-entry record per item while reads remain valid. |
| `00b68c30` | BoundedBodyParser_00b68c30 🌐 | likely | Builds a bounded reader over the supplied bytes, calls body reader 00b66890, and returns success when no read error occurred and the trailing-data condition pa… |
| `00b68ca0` | BoundedBodyParser_00b68ca0 🌐 | likely | Builds a bounded reader over the supplied bytes, calls body reader 00b668f0, and returns success when no read error occurred and the trailing-data condition pa… |
| `00b68d10` | BoundedBodyParser_00b68d10 🌐 | likely | Builds a bounded reader over the supplied bytes, calls body reader 00b66990, and returns success when no read error occurred and the trailing-data condition pa… |
| `00b68d80` | BoundedBodyParser_00b68d80 🌐 | likely | Builds a bounded reader over the supplied bytes, calls body reader 00b66a00, and returns success when no read error occurred and the trailing-data condition pa… |
| `00b68df0` | BoundedBodyParser_00b68df0 🌐 | likely | Builds a bounded reader over the supplied bytes, calls body reader 00b66a80, and returns success when no read error occurred and the trailing-data condition pa… |
| `00b68e60` | BoundedBodyParser_00b68e60 🌐 | likely | Builds a bounded reader over the supplied bytes, calls body reader 00b68810, and returns success when no read error occurred and the trailing-data condition pa… |
| `00b68ed0` | BoundedBodyParser_00b68ed0 🌐 | likely | Builds a bounded reader over the supplied bytes, calls body reader 00b66b80, and returns success when no read error occurred and the trailing-data condition pa… |
| `00b68f40` | BoundedBodyParser_00b68f40 🌐 | likely | Builds a bounded reader over the supplied bytes, calls body reader 00b66be0, and returns success when no read error occurred and the trailing-data condition pa… |
| `00b68fb0` | BoundedBodyParser_00b68fb0 🌐 | likely | Builds a bounded reader over the supplied bytes, calls body reader 00b66f50, and returns success when no read error occurred and the trailing-data condition pa… |
| `00b69130` | ReadFishingSchoolSpawnBody 🌐 | likely | Reads the school identifier and vector fields, decodes the model-entry list, reads the integer list, then consumes two trailing dwords from the bounded stream. |
| `00b694e0` | ReadCountedRecordList | likely | Clears existing record nodes, reads a count, then appends and decodes one record per count while the reader remains valid. |
| `00b696b0` | ClearFishingResponsePacketMembers | likely | Clears the fish-entry and integer lists, invokes two additional member cleanups, then restores the BaseFishingPacket and BasePacket vtables. |
| `00b697e0` | ClearFishInfoUpdateList | likely | Clears the client-fish-entry list, then restores the BaseFishingPacket and BasePacket vtables on the object. |
| `00b6a0c0` | BoundedBodyParser_00b6a0c0 🌐 | likely | Builds a bounded reader over the supplied bytes, calls body reader 00b69130, and returns success when no read error occurred and the trailing-data condition pa… |
| `00b6a170` | ReadHeaderConfigAndLists 🌐 | likely | Reads the fishing packet header, decodes a fixed configuration block and nested record, then reads a dword-counted list and a fish-entry list into the destinat… |
| `00b6a610` | ClearSpawnFishRunPacket | likely | Destroys the record list and string members, then restores the BaseFishingPacket and BasePacket vtables on the packet object. |
| `00b6aed0` | BoundedBodyParser_00b6aed0 🌐 | likely | Builds a bounded reader, calls the SpawnFishRun body reader, and returns success when no read error occurred and the trailing-data condition passes. |
| `00b6b510` | SendFishingSpecialRequest 🌐 | likely | stdcall (ret 8; arg1 = fishing request object, arg2 = flag). Initialises a stack stream writer with 0x10000000 (008d4a40), calls 00b69550 on arg1 to write its … |
| `00b6b960` | SendCastAnimRequest | likely | thiscall with no stack arguments (ECX = processor). Builds a FishingPacketCastAnimRequest (00b627c0) on the stack, copies the local handle (client interface, s… |
| `00b6ba10` | SendSpecialRequest | likely | thiscall with no stack arguments (ECX = processor). When client+0xc8 (client interface) is nonzero, builds a FishingPacketSpecialRequest (00b62cb0) on the stac… |
| `00b66be0` | — 🌐 | unknown | Reads the result body fields and two length-counted blobs into the destination, flags NaN values at selected float fields, and reads the trailing dword at offs… |
| `00b67550` | — | unknown | Reads leading dwords, flags, a bounded byte region, and successive dword values; selected values are checked for NaN and the bounded reader records short reads. |
| `00b67a00` | — | unknown | Calls a helper on the source object at offset 4, copies dwords from source offsets 0x14 through 0xa4 into the destination, and returns the destination pointer. |
| `00b6a2b0` | — 🌐 | unknown | Reads the packet header, a boolean, a dword, and a count-prefixed collection of records into the destination object. |

## Source shards

[[S0330|S0330]] (13), [[S0349 00b5ffe0-00b66a80|S0349]] (63), [[S0350 00b66b80-00b6b400|S0350]] (53), [[S0351 00b6b510-00b6c290|S0351]] (9), [[S0352 00b6cd80-00b71320|S0352]] (1), [[S0417 00ccbd60-00cd11e0|S0417]] (19), [[S0418 00cd1210-00cd5dd0|S0418]] (37), [[S0579|S0579]] (21)
