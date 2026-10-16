---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Demo Derby

Part of [[Build Guides]] › Gameplay. Related: [[Kart Racing]], [[Minigame Framework]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`, `unknown`) are the final native-map labels for a function; labels in `lab\native-map\run\map.sqlite` were read in this pass and all shards used here are accepted. Source tags: *client* = the shipped executable, read as a label, a shard finding or a field-level reading of the Ghidra pseudocode in this pass (**pseudocode** marks the last kind: an inference from decompiled code, not a captured fact); *client Lua* = the decompiled UI script `ScriptsBase.lua`; *client AS2* = ActionScript exported from `score-screen`; *client resource* = a file from the shipped `Resources` folder; *client locale* = the shipped text table (ids in parentheses; large ids are from `cq.py loc`, small ids such as 3053 are from the exported locale table that `MiniGameData.txt` points into); *video* = a reviewed frame record (frame keys given; no player names or chat are recorded here); *wiki* = the fan wiki. Precedence is client, then video, then wiki. **(R)** marks our own server policy or an inference the client cannot settle. No original server code exists, so every statement about what the server did is a hypothesis built from what the client requires.

**What this page does not use.** The vault note *Research/Derby Restoration* describes a derby packet capture. The capture file sits under `lab\references\OpenSourceFreeRealms`, which is community-reference data, so this page uses none of its frame counts, payload lengths or values. Every wire layout below is what the client's own readers and senders do. Where an accepted shard sentence quotes the capture note, it is left out. Known-wrong labels from `recheck-pending.txt` that sit near this code (`009ee730`, `009ffda0`, `00a06c30`, the array "shrink to N" functions) are not relied on. Related: [[Kart Racing]], [[Minigame Framework]], [[Matchmaking and Lobby]], [[Demolition Derby Driver]], [[Quests and Objectives]].

### Overview

Demo Derby is a native in-world arena game run by the client's own `VehicleDemolitionDerbyProcessor`. It is not a Flash movie: `MiniGameTypeData.txt` type 11 is marked non-flash and active, and the UI pieces are small windows (`derbyResults`, `derbyCrown`, a health and boost bar, a score box). It is game type 11 (`DEMO_DERBY_TYPE`) in the framework and launches and reports through it (see [[Minigame Framework]]; this page does not repeat the framework's start, close or reward packets). (client resource, client Lua)

- **Where.** Wildwood, Frost Ridge and Tangletrack have named type-11 rows in `MiniGameData.txt`. Thunder Falls appears in quest rows and lobby labels, and the wiki lists derby at four speedways: Frost Ridge, Thunder Falls, Tangletrack and Wildwood, with Sandstorm excluded. The client has a "Seaside Demo Derby" label and a Sandstorm bonus notice, which the wiki contradicts; unresolved (see [[Demolition Derby Driver]] Conflicts 3 and 4). (client resource, client locale, wiki Speedways)
- **Lobby.** The lobby is titled "Demo Derby Lobby" (locale 3053) and the arenas have their own lobby labels (Wildwood, Thunder Falls, Frost Ridge). The lobby text offers "Quick Play" for random players or "Invite Friends" (locale 37942). A seasonal event quest string asks for a solo game at Frost Ridge (locale 441194), so a derby can be played with only one human; the other drivers are then NPCs. The entry panel in video shows the arena name, "Click Here to Switch Carts", a Difficulty row (1 of 3 seen), an objective line and a Go! button (`V45-eAlBzn0/00056`). (client locale, video)
- **Match.** Eight drivers are shown ("5th /8" place counter), a countdown timer, a score counter, and two bars labelled DAMAGE! and BOOST!.  (video `V45-eAlBzn0/00180`, `V45-eAlBzn0/00194`, `A2kA_ji8PFs/00048`)
- **Damage and wrecking.** Health lowers when the kart takes damage. At zero the kart is wrecked, the player loses a point, and the kart is dropped back into the arena. Video shows a panel "You Got Knocked Out! Recover In 4 Seconds" over the arena (2 and 4 seen). Hits to the back of a kart do the most damage, the sides less, the front least. Boost recharges on its own when not in use. (client locale 5387230, 1395605149; client Lua `RespawnWindow`; video)
- **Scoring.** Points come only from knocking an opponent out: 2 for a normal opponent, 3 for the leader, who wears a crown. Doing damage alone scores nothing. The Wildwood Derby 1000 event is first to 22 points. (client locale 1086358690, 3417922616, 2192467831)
- **Power-ups.** Five kinds are named by the client: Health (snag it before others do), Boost Overload ("gives you more boost"), Shield ("invulnerable for a short time"), Attack Up ("increases the damage you do") and Ultimate ("invulnerable, doubles your damage, and infinite boost for a short time"). The kill feed prints "X now has a Shield" and "X has an Attack Up". (client locale 533432984, 602207105, 4170942142, 975811527, 2707619513; video `A2kA_ji8PFs/00232`, `V45-eAlBzn0/00282`)
- **Kill feed.** Messages seen: "X has knocked out Y", "X is now the leader!" and the power-up lines above. (video `afyPqIZqD-U/00074`, `V45-eAlBzn0/00202`)
- **Results.** A "DERBY COMPLETE!" panel titled "Derby Results" with the arena name under it, columns Place, Name, Damage Dealt and Points, and a "Waiting For All Players To Finish" line. Damage Dealt can be negative in video (for example -158 beside 5 points in the first-place row, `afyPqIZqD-U/00250`); the client computes it as a difference of two server-sent values (see Data and formats). (client AS2, client locale 401575, 401587, video)
- **Job.** Demo Derby Driver ("Demolition Derby Driver" on the wiki; max level 20) is the job for this game; the wiki says the derby is exclusive to it. Trainers, the 44-title quest line, the Crash Cup events, gear and karts belong to [[Demolition Derby Driver]]. About 25 of those quests are tied to type-11 rows in `MiniGameData.txt` (rows 603 to 627 carry the quest titles as names, each with a difficulty of 1 to 3). Karts are shared with the garage described in [[Kart Racing]]. (client resource, wiki)

### How the client implements it

#### Main classes

| Class | Role | Evidence |
|---|---|---|
| `VehicleDemolitionDerbyProcessor` (ctor `009f0f30`, dtor `009f11a0`) | One per client. Virtual slot 0 `009f39b0` receives packets; slot 1 `009f01f0` refreshes the HUD; slot 2 `009f20f0` is the countdown tick; slot 3 `009ef180` builds the results. Fields (pseudocode, shard S0281): owner +4, active byte +8, game state +0xc, countdown timer +0xc4, count +0xc8, pending byte +0xce, player count +0xd0, NPC count +0xd4, 19 sound-event pairs +0xdc to +0x16c, vehicle list +0x178, local vehicle +0x184, pickup list +0x18c, crown +0x198, DerbyInfo list +0x1a0, results list +0x1b0. | ctor, dtor confirmed; `009f01f0` confirmed; `009f39b0`, `009ef180`, `009f20f0` unknown |
| `ProxiedVehicleDemolitionDerby` (ctor `00c657a0`) | One per driver, local or remote, in a 0x1bf8-byte list node: key pair +0xe60/+0xe64, physics state +0x10b0, local phase +0x1624, last received pose record +0x1b40. Shares `ProxiedVehicle` (`00c59a10`, `00c5afd0`) with the kart racer. | ctor likely; dtor body likely |
| `ProxiedVehicleDemolitionDerbyPickup` (ctor `00c65aa0`) | A power-up pickup in the arena; list node 0x24 bytes, id at +4. | likely |
| `ProxiedVehicleDerbyCrown` (ctor `00c66ad0`) | The leader's crown actor, created when the local driver's vehicle is created and stored at processor +0x198. | likely |
| `DerbyInfo` (0xb0-byte node, ctor `009ef2a0`) | Maps a driver key (+0x98/+0x9c) to a display name (IString +0x88). Names come from an entity lookup by key (`00984960`) or a default string. | ctor likely; the name source is read from pseudocode of `009f39b0` |
| `ControllerVehicleDemolitionDerby` (ctor `00af6ad0`) | Local input. Slot 50 `00af6cb0` reads the held-action list: ids 1/2 drive the accumulator at +0x15cc, 7/8 the one at +0x15d4, ids 0xa and 0xf set flags, and ids 0x13 to 0x16 call `009cf430` with 0 to 3. Slot 51 is the shared `ControllerVehicle::UpdateMovement` (`00cc1a00`). | ctor, slot 50, slot 51 confirmed; meaning of 0x13 to 0x16 unresolved (the label of `009cf430` is an action-bar function and may not fit) |
| `DemolitionDerbyConfig` (ctor `0105f170`) | Two IStrings, a float at +0x24, a dword at +0x78, a `StartLocation` array (0x30-byte elements, +0x98) and an `NpcData` array (0x14-byte elements, +0x88). Only the float and dword are read from the wire. | ctor, dtor confirmed; array allocators confirmed |
| `VehicleDemolitionDerbyResultsDataSource` (ctor `00c665c0`) | Publishes `BaseClient.VehicleDemolitionDerbyProcessor.DerbyResults` (Position, Score, Damage). | shard S0398 (record not in the generated tables) |
| `VehicleDemolitionDerbyGroupHandler` (`00a89b30`) | Calls the Lua group handler `:Close` when the derby ends or is not active. | shard S0309 |
| `VehicleConfig` (ctor `010569f0`) | Per-vehicle config shared with karts: arrays of gears (inline 6), damage levels (inline 7), assist tiers and part particle effects. Field meanings are not recovered. | confirmed (shard S0573) |

#### Derby lifecycle

1. **Arming.** `DispatchBasePlayerUpdate` `0092f460` (confirmed) calls `VehicleDemolitionDerby_SetActive` `009f1b80` (confirmed) with 1 when bit 11 (0x800) of the local-actor status word is set, the derby processor exists and its active byte is 0. This is the same dispatcher and status word that arm the kart race (bits 10 and 19). `BeginPlayerZoning` `0093d180` (confirmed) is the other caller and passes 0. (pseudocode; the carrier packet for the derby status bit is not established, the kart analogue is a 35/20 player update)
2. **Activate.** `009f1b80` shows `DemoHealthBar` and `DemoScore`, switches the camera (`Camera_ChangeToVehicleDemolitionDerbySettings`), loads the 19 sound events (`009ee280`, confirmed), allocates the crown helper and sends the register record 34/6. On deactivate it hides the derby windows (`DerbyCrown`, `DerbyResults`, `RaceOrder`, `RespawnWindow`, `RaceProxArrow`), stops sounds and clears all four lists. This function was filed in an older vault note as a kart switch; that note (*Kart Entry Register Prerequisites*, line 123) is wrong. (client, shard S0282)
3. **Loading barrier.** While the derby is active, `WaitForWorldReady` `00936d00` (confirmed) runs its own loop: it steps physics, calls the derby state machine `009f2d90` and holds the loading screen. If the gateway drops it logs "Disconnect from gateway waiting for derby processor" and exits. (pseudocode)
4. **Setup.** The server sends 34/19 (config), 34/3 and 34/4 (counts), then one 34/10 per driver. The handler builds a vehicle node, marks it local when its key equals the local player key (`00705c30`, likely), creates a `DerbyInfo` and, for the local driver, sets processor +0x184, resets the camera and Lua group handler, and sends 34/2 with phase 2. (pseudocode of `009f39b0`)
5. **State machine.** Per frame `009f2d90` (confirmed, called from `PlayClient::Update` `0090c970` and from the loading loop) combines two values: the processor game state at +0xc (written by 34/1) and the local driver's phase at +0x1624. Each time a gate passes it writes the new phase and sends 34/2. The gates are:

| 34/1 value | Client waits for | Phase set | What the client does (pseudocode) |
|---|---|---|---|
| 3 | `009ed680`: count of drivers with flag byte +0xf00 equals the 34/3 count | 3 | acknowledges |
| 4 | `009ed6b0`: count without the flag equals the 34/4 count | 4 | acknowledges |
| 5 | all 19 sound events ready (`009ed3d0`), `009ed5a0`, every driver has a physics object (`009ed610`) | 5 | sets a per-driver round flag (`009ed640`) |
| 6 | `009eb380` threshold check on the local state | 6 | sets a global byte +0x108c, acknowledges |
| 7 | internal flag +0xb9 | 7 | sets countdown count to 4 and acknowledges |
| 8 | pending byte +0xcf clear | 8 | `Countdown:Show`, `Countdown:Reset`; slot 2 counts down and sends 34/7 |
| 9 | none | 9 | hides the countdown, `RaceTimer:Show` and `StartTimer`, `RaceOrder:Show`, `RaceOrder:SetNumberRacers`; per-frame HUD refresh and `009f22c0` |
| 10 | none | 10 | picks the win or lose sound from state byte +0x140, hides the score and health windows, shows `DerbyResults`; slot 3 `009ef180` rebuilds the results each frame |
| 11 | none | 11 | `ScoreScreen:EnableExitButton` |

The meaning of states 3 to 6 beyond the gate predicates is unknown. Unlike kart racing, **the client has no finish test of its own**: the derby ends only when the server sends game state 10, and Close is allowed only after 11. `DerbyResults:Show` also disables the exit button first. Video text "Waiting For All Players To Finish" fits this. (pseudocode of `009f2d90`, client Lua `DerbyResults`)

6. **During play.** `009f22c0` (likely) runs every frame in phases 5 to 9. For remote drivers it keeps their pose and effect slots up to date from the last 34/5 record (applying the record only if its 64-bit time field is newer, `00c659b0`, speculative). For the local driver it sends: 34/5 (pose and input state) every frame it has a physics object; 34/14 pick-up claims when state flag +0x1d0 is set; and 34/16 hit reports, one per contact in the vehicle's contact list whose type field is 8 (a vehicle contact). Damage, health and score are not computed on the client: they arrive in 34/11. (pseudocode)
7. **HUD.** `009f01f0` (confirmed) pushes state +0x124 to `DemoScore:Set`, state +0x1e8 and the absolute boost value at +0x114 to `DemoHealthBar:SetHealthandBoost`, the boost to `RaceBoostBar:SetBoost`, and the local place to `RaceOrder:SetPlayerPlace`. On a 34/11 for the local driver the client shows `RespawnWindow:DisplayRespawn` when incoming health falls to zero or less and `RespawnWindow:Hide` when it returns above zero, and plays `UI_LowHealthNotify` while the incoming value is below 26. A score change spawns a floating text of the difference. (pseudocode of `009f39b0`)
8. **Leader and crown.** The leader is computed on the client: the driver with the strictly greatest score in state +0x124. Ties, or all scores at zero, give no leader (`009ed290` and `009ed2e0`, labels unknown and rejected in review for exactly these edge cases). The crown actor follows that driver and plays `Vehicle_Take_Lead` / `Vehicle_Lose_Lead`. Whether the server also tracks the leader (it must, for the 3-point rule) is a server matter. (pseudocode)
9. **Teardown.** `SetActive(0)` clears everything. 34/8 removes one driver; 34/9 kicks one and, if the local driver is in phase 9, refreshes `RaceOrder:SetNumberRacers`. The score screen's Close goes through the framework (`Ui.LeaveMiniGameOrEncounter`; see [[Minigame Framework]]). (pseudocode)

#### Power-ups, as the client applies them

The client has no stored item queue for the derby (the kart's 3-slot FIFO is not touched; `RaceItemSlots` is hidden). A 34/15 pickup response applies the effect at once, and a 34/17 expiry removes it. The mapping of the type number to the five power-ups follows the layout of the 19 sound-event pairs: the handle the response plays is the Health, Boost, Shield, Attack-Up or Ultimate "collected/activated" event for types 1 to 5 in order (pseudocode of `009f39b0` against the event order of `009ee280`; this is an inference from layout, strong but not stated anywhere).

| Type | Power-up | Client effect on response (34/15) | On expiry (34/17) |
|---|---|---|---|
| 1 | Health | plays `Vehicle_Health_Collected` | none |
| 2 | Boost Overload | stores the first response float into state +0x114 (the boost value); plays `Vehicle_Boost_Collected` | none |
| 3 | Shield | turns on effect slot `00c57610` (the kart shield slot per shard S0395); stores a response float at state +0x1d4; plays Shield activated and active | slot off, plays Shield expired |
| 4 | Attack Up | turns on slot `00c579b0`; stores a response float at state +0x1d8 | slot off, plays Attack Up expired |
| 5 | Ultimate | turns on slot `00c57440`; stores a response float at state +0x1d8 | slot off, plays Ultimate expired |

The flag bytes in a driver's 34/5 pose record (below) carry the same three slots to the other clients. What the stored floats mean (duration, multiplier) is unknown. (pseudocode)

### Data and formats

**Client resource tables (read by the client, not sent by the server)**

| Table | Notes | Source |
|---|---|---|
| `MiniGameData.txt` | 48 of 783 rows are TYPE 11. Named: Wildwood Demo Derby (ids 39, 566, 568, 570), Frost Ridge (40), Tangletrack (41), a Wildwood row named "Wildwood Kart Racing" with derby text (602), Demolition Derby Tutorial and Advanced Demolition Derby Tutorial (289/290, 293/294, 297/298), 25 quest rows named after quests (603 to 627: Taking the Wheel, Break 'er In, License to Drive, ... Wildwood Derby 1000, Frost Ridge Invitational), "Fighting for Funds" (1104). Rows with name id 0 (291, 292, 295, 296, 299, 300, 1109, 1111, 1113) are difficulty variants (2 to 5). DIFFICULTY is 1 to 3 on the quest rows. MIN_PLAYERS, MAX_PLAYERS, START_SOUND, PRELOAD_FILE and LEADERBOARD_ID are 0 or empty in every type-11 row, so player counts do not come from this table. | client resource |
| `MiniGameTypeData.txt` | Type 11: not flash, not micro, active; name and description ids 0 (the name comes from the Lua enum). | client resource |
| `ControllerElements.txt` | The options menu has a "demolition derby" block with one action, `UsePowerup` (row `dd1`), but the whole block is commented out in the shipped file. Whether the derby has a use-power-up key is therefore unknown. | client resource |
| `Models.txt` | Derby models: car, car_crusher, car_ripper, junk_car, tire, and two collection nodes. Ids 886, 903 to 906, 702, 703. | client resource |
| `VehicleItemClasses.txt` | `VEHICLE_ID^ITEM_CLASS_ID`, 124 rows, loaded by `0104aea0` (see [[Kart Racing]]). Whether derby karts use it is unknown. | client resource |
| `CodeStringMappings.txt` and locale | `DerbyComplete` (401575) and `DerbyResults` (401587); place strings 39797 ("1st") down to 39788 ("10th"). | client resource, client AS2 |
| Sound events | 19 names: Countdown_Beep, Countdown_Go, Finish_Win, Finish_Lose, UI_LowHealthNotify, Take_Lead, Lose_Lead, Boost_Collected, Health_Collected, Shield Activated/Active/Expired/Hit, Attackup Activated/Active/Expired, Ultimate Activated/Active/Expired. | `009ee280`, confirmed |
| Vehicle parts, gear, karts | Names and tiers are in [[Demolition Derby Driver]] (Gear section). The vehicle definition the server sends in 34/10 is the shared `ReadVehicleDefinition` `009d7b50` (likely) that kart racing also uses; its interior is not recovered. | client locale |

**Results screen (client AS2 and Lua).** `DerbyResults:Populate` (fired by `009ef180`) reads the data source row by row: column 0 place, 1 guid, 2 name, 3 score, 4 damage, then calls `ScoreScreen:SetPlayerDerbyResults`. The Flash page (`DerbyState`, `DerbyResultItem`, `DerbyListHeading`) shows Place, Name, DamageDealt and Points and requests rows through `RequestDerbyData(offset, count)`. Place comes from sort order: the client sorts its own vehicle list by score descending with damage as tie-break (comparator at `009eb120`), after reading score from state +0x124 and damage as state +0x128 minus +0x12c. Names are resolved from `DerbyInfo` (`00c66780`, speculative). So the server must have delivered a final score and both damage terms for every driver (via 34/11) before sending game state 10. (client Lua, client AS2, pseudocode)

**Header.** Every derby message carries inner opcode 34 (0x22, written by the base ctor `009e9f80`, confirmed) and a subtype, each a 16-bit value, read by `009ee770` (likely). The receive dispatcher does not check the opcode, only the subtype. Offsets below are bytes after that 4-byte header. Sizes are what the client's readers consume, checked against raw bytes in shard reviews.

**34/5 UpdateVehicleDemolitionDerbyData (128 bytes; reader `009ef5e0` likely, serializer `009f0ce0` likely, both directions).**

| Off | Type | Meaning (pseudocode) |
|---|---|---|
| 0x00 | u64 | driver key |
| 0x08 to 0x47 | 4 x f32[4] | the vehicle's 4x4 transform (from `00c54660`) |
| 0x48 | f32[4] | vector from `00c546a0` (not identified) |
| 0x58 | f32[4] | vector from `00c546e0` (not identified) |
| 0x68 | 3 x f32 | the driver's input accumulators (vehicle +0x15cc, +0x15d0, +0x15d4; throttle, reverse, steer per the kart controller) |
| 0x74 | 4 x u8 | shield on, attack up on, ultimate on, and a fourth flag (vehicle +0x15df, +0x15e1, +0x15de, +0x15dc) |
| 0x78 | u64 | client-side 64-bit value from the owner (+0x98/+0x9c), apparently a time or sequence; the receiver keeps the newer record |

The receiver applies 34/5 only to drivers other than the local one and only while the game state is below 10. (pseudocode of `009f39b0` case 5)

**34/10 CreateProxiedVehicleDemolitionDerby (reader `009f3800` likely; dispatcher case in `009f39b0`).** u64 key; i32 length then that many bytes (name blob, `00894b10`); the vehicle definition (`009d7b50`); 4 x f32[4] (a 4x4 block, probably the start transform); u8 bool; f32. The key is what the client compares with its own player key to decide which driver is local. The first float4 block, bool and float meanings are unknown.

**34/11 UpdateVehicleDemolitionDerbyStateData (51 bytes, reader `009ee850` likely; 55 with the header).**

| Off | Type | Use (pseudocode of `009f39b0` case 0xb) |
|---|---|---|
| 0x00 | u64 | driver key |
| 0x08 | u32 A | stored at vehicle +0xf04 and a client object; for the local driver compared with state +0x1e8, which the HUD shows as health; below 26 triggers the low-health sound. Likely health (inference) |
| 0x0c | u32 B | score, stored at state +0x124 and in the results list |
| 0x10 | u32 C | stored at state +0x130 |
| 0x14 | u32 D | stored at state +0x134 |
| 0x18 | u32 E | stored at state +0x128 (damage term) |
| 0x1c | u32 F | stored at state +0x12c; results damage = E minus F |
| 0x20 | 3 x u8 | passed to `00fe9650` by index 0 to 2 (flag array) |
| 0x23 | 4 x f32 | stored by index 0 to 3 (`009ef130`) |

Fields C, D, the three bytes and the four floats have no known meaning. 

**34/12 CreateProxiedVehicleDemolitionDerbyPickup (36 bytes with header; reader `009ee9e0` likely).** u32 pickup id (+0xc), u32 definition id (+0x10, looked up as a client definition), f32[4] position, f32, f32. The client ignores it if there is no local driver or the game state is 10 or above. Pickup ids must be unique; a repeat id is ignored (`009ed260`, likely). 34/13 DestroyProxiedVehicleDemolitionDerbyPickup is one u32 id (8 bytes with header).

**34/14 PickupHitRequest (C>S, sender `009f1550`, confirmed).** u64 local driver key, f32[4] position (24 bytes). No pickup id: the server attributes the touch to a pickup by position, as for the kart. Sent when a local flag is set by the physics and cleared after sending.

**34/15 PickupHitResponse (28 bytes with header, reader `009eec60` likely).** u64 driver key, u32 type (1 to 5), 3 x f32. Applied only if the driver exists and has a physics object, and the game state is below 10.

**34/16 VehicleHit (both directions, 24 bytes with header, reader `009eeba0`).** u64 key, u64 key, f32. The client sends it with its own key first and the other driver second, once per vehicle contact, with a float taken from the contact record (likely an impact magnitude; unit unknown). On receive, the client applies an impulse scaled by the float along the line between the two vehicles' positions (`00fe9710`) and calls a follow-up helper (`00c54800`) twice. Which of the two keys is the attacker is not established (review downgraded `009efcc0`).

**34/17 ItemExpired (8 bytes with header, reader `009ef4d0`, speculative).** One u32 that the client treats as 3, 4 or 5 (shield, attack up, ultimate). It is a category, not an item id.

**34/18 ResetProxiedVehicleDemolitionDerby (12 bytes with header, reader `009ef520`).** u64 key. The client calls `00c56c60` with the 4x4 block stored at vehicle +0x1540 (set from the creation data, and for remote drivers refreshed by 34/5). The packet carries no position, so where a wrecked kart reappears depends on client-held state. Unknown how the original chose respawn points; the config's start-location array is the obvious source but is not on the wire. (pseudocode)

**34/19 SetDerbyConfig (reader `009ee190` likely, wrapper `009f0700` unknown).** f32 (config +0x24), u32 (config +0x78): 8 bytes. The client copies the whole config object (`009f0830`, likely) but only these two values arrive on the wire. Their meanings are unknown.

**34/20 AnnouncerMessage (28 bytes with header, reader `009eeeb0`).** u32 kind, u32 sub-kind, u64 key, u64 key. Pseudocode of the handler: kind 0 with sub-kind 3, 4 or 5 prints a power-up line (string keys `dd_shieldstring_bbe`, `dd_attackupstring_bbe`, `dd_ultistring_bbe`) for the first key; kind 1 prints the new-leader line (`dd_leaderstring_bbe`), using one name; kind 2 prints a knock-out line (`dd_knockoutstring_bbe`) with the two names as `player_1` and `player_2`. Output goes to `NotificationHandler:SystemMessage` with speaker and listener tokens. The roles of the two keys in kind 2 (who knocked out whom) are inferred from the video text, not read from the code.

**Small messages.** 34/1 is one i32 (game state, to processor +0xc). 34/3 and 34/4 are one i32 each (player count to +0xd0, NPC count to +0xd4). 34/2 (C>S) is u64 key plus i32 phase (12 bytes; serializer `009f0a50`). 34/7 (both) is u64 key plus u32 count (16 bytes with header; serializer `009f0b80`); the server's value goes to +0xc8 and sets a pending flag. 34/6 (C>S) is two 64-bit values (the first is the local key; serializer inline in `009f1790`, meaning of the second unknown). 34/8 and 34/9 are one u64 key each (12 bytes with header; reviews corrected earlier 16-byte claims at `009efc50`, `009efbe0` and `009efe80`).

### Network

All derby traffic is inner opcode 34 inside the tunnel (`PacketTunneledClientPacket`, client sends through `00925640`). The packet catalogue has no derby classes, so the map's `wire` field links only the tunnel. The class list below is **confirmed** from the 20 constructors; directions come from where each class is built or parsed.

| Sub | Class | Dir | Ctor | Handler or sender |
|---|---|---|---|---|
| 1 | UpdateVehicleGameState | S>C | `009ea0b0` | `009f39b0` case 1; reader `009ef570` |
| 2 | UpdateVehicleGamePlayerState | C>S | `009ebc80` | `009f2d90`, `009f39b0` via `009f1440` |
| 3 / 4 | UpdatePlayerCount / UpdateNpcCount | S>C | `009ea1b0` / `009ea260` | cases 3, 4 |
| 5 | UpdateVehicleDemolitionDerbyData | both | `009ebe30` | receive case 5; send `009f22c0` via `009f1a70` |
| 6 | RegisterVehicleDemolitionDerbyPlayer | C>S | `009ebf00` | `009f1b80` via `009f1790` |
| 7 | VehicleDemolitionDerbyCountdownStatus | both | `009ebfd0` | receive case 7; send `009f20f0` via `009f18b0` |
| 8 / 9 | Destroy / Kick ProxiedVehicle | S>C | `009ec1e0` / `009ec110` | cases 8, 9 |
| 10 | CreateProxiedVehicleDemolitionDerby | S>C | `009edde0` | case 10 |
| 11 | UpdateVehicleDemolitionDerbyStateData | S>C | `009ec2b0` | case 11 |
| 12 / 13 | Create / Destroy ProxiedVehicleDemolitionDerbyPickup | S>C | `009ecfd0` / `009ea450` | cases 12, 13 |
| 14 | PickupHitRequest | C>S | `009ec380` | `009f22c0` via `009f1550` |
| 15 | PickupHitResponse | S>C | `009ec450` | case 15 |
| 16 | VehicleHit | both | `009ec590` | receive case 16; send `009f22c0` via `009f1680` |
| 17 | ItemExpired | S>C | `009ea690` | case 17 |
| 18 | ResetProxiedVehicleDemolitionDerby | S>C | `009ec660` | case 18 |
| 19 | SetDerbyConfig | S>C | `009ea7e0` | case 19 |
| 20 | AnnouncerMessage | S>C | `009ec750` | case 20 |

The dispatcher (`009f39b0`, label unknown) handles subtypes 1, 3, 4, 5, 7 to 13 and 15 to 20. Subtypes 2, 6 and 14 have no receive case and are send-only on the client. For 5, 7 and 16 the client both sends and receives. Whether the server relays 7 and 16 to other clients (as the kart does for hits) is not recoverable from the client. (client, shards S0281 and S0283)

Subtypes 5, 12, 13, 15, 16, 17 and 20 are ignored unless the local driver exists and the game state is below 10 (12 also stops above 9), so late packets after the end are dropped by the client itself. 34/11 and 34/18 have no such check. (pseudocode)

**Order the client needs.** 34/19, 34/3 and 34/4 and then 34/10 for every driver before 34/1 value 3, because state 3 and 4 only pass once the number of created drivers equals the counts. 34/11 and 34/5 are looked up by key and are dropped if the driver does not exist yet. The client answers 34/10 for the local driver with a 34/2 and answers each later gate with another 34/2. (pseudocode)

**Framework and other packets around the derby.** Entry, goal list, start request, end dialog, Close, `41/x` encounter messages, the lobby catalogue `102/1`, `102/2` and reward bundles are the framework's; see [[Minigame Framework]] and [[Matchmaking and Lobby]]. The derby results panel is opened by the derby code itself (`DerbyResults:Show` calls `ScoreScreen:Show(4, 11)`), not by 39/18. Whether the original sent 39/18, 39/47 or reward packets for a derby is not recoverable from the client; no packet source for this page records one.

### What the Rust server must do

Items marked (R) are reconstructed policy, not recovered original behaviour. Nothing here has been tried against the real client.

1. **Launch through the framework.** Serve the type-11 rows, goals and info record as for any native game, and set the derby status bit (bit 11) that arms `SetActive` (R: use a 35/20 player update as for the kart; not established for the derby). Deliver every kart, part, arena and effect asset; a missing vehicle asset stops 34/10 the way it stops kart 33/11.
2. **Lobby and entry.** Offer Quick Play and Invite Friends, fill with NPC drivers up to eight, and allow a solo player (client locale 441194, 37942). Gate entry on the job and send the job-switch message the client shows for karts (R).
3. **On 34/6,** bind the key to the authenticated player. Then send 34/19 (two values, meanings unknown, so replay plausible numbers (R)), 34/3 and 34/4 matching what you create, and 34/10 for the player and each NPC. The player's own 34/10 key must equal the key the client considers its own, or no driver becomes local and the HUD never starts.
4. **Drive the state machine.** Send 34/1 values 3 to 11 in order. Wait for the client's 34/2 acks if you want the gates to be meaningful (R; whether the original waited is unknown). Send 34/7 with the starting count (the client defaults to 4) and expect its 34/7 echoes.
5. **Relay poses.** Accept 34/5 from each human, validate that the key is theirs, and send it to the other humans. NPC drivers have no client, so the server must synthesise their 34/5 and 34/11 (R; the original AI is unknown).
6. **Own combat.** The client reports contacts only (34/16 with a float). The server must turn them into health, damage and score and send 34/11 for every affected driver. Rules to implement from the client's own text: points only for knock-outs (2, or 3 if the victim is the leader); a wrecked kart loses a point, shows a respawn countdown and returns to the arena (video shows 4 seconds); back hits hurt most, side hits less, front hits least (client locale 5387230, 1086358690, 1395605149). All numbers (health scale, damage per impact, respawn point, countdown) are unknown (R). Keep the leader rule consistent with the client's crown (strict greatest score).
7. **Pickups.** Send 34/12 with unique ids and a definition id, attribute each 34/14 to the nearest live pickup by position (R: bounded radius), reply 34/15 with type 1 to 5 and floats (R), announce with 34/20 kind 0, send 34/13 when taken and respawn later (R timing). End Shield, Attack Up and Ultimate with 34/17 values 3, 4, 5. Durations are unknown (R).
8. **Announcer.** Send 34/20 kind 2 for each knock-out and kind 1 on each leader change.
9. **Finish.** Decide the end (R: first to a score target for Derby 1000 type events, otherwise the timer; the video timer runs near 3 to 4 minutes but its start value is unknown). Send final 34/11 for every driver before 34/1 value 10 so the results table has score and damage terms (34/11 is accepted in any state and slot 3 rebuilds the table every frame in phase 10, so a late 34/11 also shows). Then 34/1 value 10, and 34/1 value 11 when you want Close enabled. Keep sending nothing that changes scores after 10.
10. **After the results.** Handle Close and the leave through the framework. Grant rewards and job experience through ordinary inventory and job updates; the derby tutorial and quest rows (type-11 ids 289 to 300, 603 to 627) are server-run encounters keyed on these rows. Reward amounts are unknown (R).
11. **Disconnects.** Remove the driver with 34/8 or 34/9 and refresh counts; abort the match if no humans remain (R).

### Video and wiki evidence

- **Video.** The reviewed frames cover derby HUD (`V45-eAlBzn0/00056`, `00180`, `00194`, `00202`, `00282`; `A2kA_ji8PFs/00024`, `00048`, `00232`; `afyPqIZqD-U/00074`), results screens (`V45-eAlBzn0/00320`, `ZmrgJdhXz3w/00248`, `afyPqIZqD-U/00250`) and the Briarwood derby (`Q5gizyhmuZ0/00214`). They show: eight drivers; place counters; score counter (a 10-point score seen in one frame); the knocked-out panel with a 4-second or 2-second recovery; kill-feed lines; arena names under the results title (Wildwood, Frost Ridge); and a results table where the Damage Dealt column includes negative numbers and the Points column runs 5, 4, 3, 3, 2, 1 down the places. NPC drivers appear by name in the kill feed. Player names and chat in these frames are not recorded.
- **Wiki.** Derby is exclusive to the Demo Derby Driver job; the speedways list gives Frost Ridge in Snowhill, Thunder Falls in Lakeshore, Tangletrack in Briarwood and Wildwood in Sanctuary, with Sandstorm offering no derby; the job blurb promises armor and repair power-ups, boosts and extra damage points, and "Members Only leaderboards" (the client has `LEADERBOARD_ID` 0 in every type-11 row). The wiki says Wildwood and Thunder Falls are the same demolition area. The wiki's Community Events page describes a Wildwood Demolition Day event where players fought an ambassador in derby games (no separate page exists in the archive). The wiki's Crash Cup page says the weapon comes from finishing the questline. All of these are lower precedence than the client strings above.

### Open questions and contradictions

- **No original server and no usable capture.** The one derby capture the vault describes is community-reference data and is not used here. Every server-side rule above is inferred. If a capture of original traffic with clear provenance becomes available, 34/10, 34/11, 34/12 and 34/19 values are the first to check.
- **Meaning of fields:** 34/11 fields C, D, three bytes and four floats; the two vectors and the fourth flag byte in 34/5; 34/10 float4 block, bool and float; the two 34/19 values; 34/6 second value; 34/15 floats; 34/16 float unit and key roles; the 34/1 gates for states 3 to 6; the high-order effect of state 6's global byte.
- **Numbers:** health scale (HUD resets show 100/100 in the tutorial; low-health sound below 26 is the only client constant), damage per hit, point rule for ties, match length, respawn time and point, power-up durations and drop odds, NPC behaviour, difficulty effects, rewards and experience.
- **Start of play and the end condition.** The client has no end test; what the original server used to end a match (timer, score target, last kart) is not recoverable. Derby 1000's 22-point rule is client text; whether other events used a score target is unknown.
- **Use-power-up input.** `UsePowerup` exists in a commented options row, action ids 0x13 to 0x16 are handled by the controller, but the pickup response applies effects immediately. Whether the derby ever needed a key press for power-ups is unknown.
- **Where kart and derby differ.** Kart racing is opcode 33 with a client-side finish and a 3-slot item queue; the derby is opcode 34 with a server-driven finish and immediate power-ups. Do not copy kart packet logic. Vault note corrections already recorded: `009f1b80` is derby, not kart; subtype 8, 9 and 18 readers consume 12 bytes with header; `ItemExpired` carries a category, not an item id.
- **Labels with weak or unknown confidence** that this page leans on: `009f39b0`, `009ef180`, `009f20f0`, `009ed290`, `009ed2e0`, `009ed5a0` (unknown); `009f22c0`, `00c659b0` (likely, speculative); the whole results-array family `00c660c0` to `00c66780` (speculative). Pseudocode readings here should be re-checked in a native run.
- **Source conflicts.** Sandstorm derby (wiki says none, client has a label); Wildwood and Thunder Falls as one area (wiki) or two arenas (client labels and quests); the wiki's speedway locations differ from the client's (see [[Demolition Derby Driver]]). The video entry panel reads "Switch Carts"/"car" in places where the client text says "kart". Client strings are used for names.
- **Other uses of derby code.** `ProxiedVehicle` and the physics, `ControllerVehicle::UpdateMovement` and `VehicleConfig` are shared with [[Kart Racing]]; the vehicle definition read by `009d7b50` and the unrelated `VehiclePartProcessor` (opcode 0x8b) are not derby specific.

### Key functions

| Address | Role | Confidence |
|---|---|---|
| `009f39b0` | `VehicleDemolitionDerbyProcessor` receive dispatcher (slot 0), all derby subtypes | unknown (accepted, subsystem `minigame.derby`) |
| `009f2d90` | Per-frame derby state machine, sends 34/2, drives HUD and results | confirmed |
| `009f22c0` | Per-frame vehicle tick, sends 34/5, 34/14, 34/16 | likely |
| `009f1b80` | `SetActive`: HUD, camera, sound events, sends 34/6 | confirmed |
| `009f01f0` | HUD refresh (score, health, boost, place) | confirmed |
| `009f20f0` | Countdown tick, sends 34/7 | unknown |
| `009ef180` | Builds the results table and fires `DerbyResults:Populate` | unknown |
| `009ee280` | Loads 19 sound events | confirmed |
| `009e9f80` | Base constructor, stores opcode 0x22 and subtype | confirmed |
| `009ee770` | Reads the 16-bit opcode and subtype | likely |
| `009f3800` | 34/10 body reader | likely |
| `009ee850` | 34/11 body reader | likely |
| `009ef5e0` / `009f0ce0` | 34/5 reader and serializer | likely |
| `009ee190` | 34/19 config reader | likely |
| `009ed1e0`, `009ed260`, `009ed220`, `009ed350` | List lookups by key for drivers, pickups, DerbyInfo, results | likely |
| `009ed680`, `009ed6b0`, `009ed3d0`, `009ed610`, `009eb380` | State gates | likely |
| `00af6cb0` | Derby controller input step (slot 50) | confirmed |
| `0105f170` | `DemolitionDerbyConfig` constructor | confirmed |
| `00c657a0`, `00c65aa0`, `00c66ad0` | Proxied vehicle, pickup and crown constructors | likely |
| `00c659b0` | Keep the newer 34/5 record | speculative |
| `0092f460`, `0093d180`, `00936d00` | Status dispatcher (arms the derby), zoning teardown, loading barrier | confirmed |
<!-- NARRATIVE END -->

## Mapped functions (generated)

236 accepted functions: 128 confirmed, 59 likely, 18 speculative, 31 unknown. Network-linked (`wire`): 63.

Subsystems: `game.derby` (124), `net.derby` (112)

### `game.derby`

**ControllerVehicle** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cc1070` | ControllerVehicle::Destroy | confirmed | Restores the ControllerVehicle vtables at the primary object and offset 0xe0, then calls FUN_0094fab0 to release the base vehicle state. |
| `00cc10d0` | ControllerVehicle::CallBaseThenVirtualA4 | confirmed | Calls FUN_0094e210, then invokes the object's vtable entry at offset 0xa4 with the constant stored at 0x01814c18. |
| `00cc14b0` | ControllerVehicle::ControllerVehicle | confirmed | Constructs the base vehicle, installs ControllerVehicle and physics-tick callback vtables, and initializes numerous controller state, transform, timing and fla… |
| `00cc16d0` | ControllerVehicle::ScalarDeletingDtor | confirmed | Calls the ControllerVehicle destructor and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
| `00cc1810` | — | unknown | Transforms four basis vectors through FUN_00770e70, scales their components with stored constants, and writes the resulting position and three basis vectors at… |

**ControllerVehicleDemolitionDerby** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af6ad0` | ControllerVehicleDemolitionDerby::Ctor | confirmed | Calls ControllerVehicle::ControllerVehicle (00cc14b0) with the forwarded arguments, stores the Derby vtable at +0 and at +0xe0, calls 0094c8a0 with 0, then reg… |
| `00af6c10` | ControllerVehicleDemolitionDerby::DtorBody | confirmed | Stores the Derby vtable at +0 and +0xe0, then calls ControllerVehicle::Destroy (00cc1070), which restores the ControllerVehicle vtables and releases the base. |
| `00af6c90` | ControllerVehicleDemolitionDerby::ScalarDtor | confirmed | Virtual slot 7, scalar deleting destructor shape: calls the destructor body 00af6c10, then calls _free when bit 0 of the argument is set. |
| `00af6cb0` | ControllerVehicleDemolitionDerby::ApplyActionList | confirmed | Thiscall virtual at ControllerVehicleDemolitionDerby vtable slot 50. Walks the entry list at this+0xc8. Entries with kind field +0x18 == 0 and flag byte +0xd s… |
| `00af6c70` | ControllerVehicleDemolitionDerby::GetSub130Field184 | speculative | Virtual slot 53: returns 0 when the owner at this+0x74 or its +0x130 sub-object is null; otherwise returns the dword at that sub-object +0x184. |

**DemolitionDerbyConfig** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0105f170` | DemolitionDerbyConfig::DemolitionDerbyConfig | confirmed | Stores DemolitionDerbyConfig::vftable at +0. Builds two embedded SoeUtil::IString<char> members (vtable 0175b24c): one at +4 and one at +0x14, each with data p… |
| `0105f240` | DemolitionDerbyConfig::~DemolitionDerbyConfig | confirmed | Stores DemolitionDerbyConfig::vftable, then tears members down: stores the StartLocation array vtable, releases its buffer at +0x9c through 00401c00 with align… |
| `0105f360` | DemolitionDerbyConfig_ScalarDeletingDtor | confirmed | Thiscall with a flag byte (ret 4). Calls the DemolitionDerbyConfig destructor body 0105f240 on this, then when bit 0 of the flag is set calls _free on this (00… |

**DerbyInfo** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009eef70` | DerbyInfo_DtorWrapper | likely | SEH-framed wrapper preserves incoming ECX as the receiver for destructor body 009ee630, restores the prior exception chain, and returns; the stack-cookie push … |
| `009ef2a0` | DerbyInfo_ctor | likely | Constructor of the DerbyInfo record: runs the base constructor FUN_0070bac0 (zeroing the collection triple at +0x7c/+0x80/+0x84) and initialises the embedded I… |
| `009ef840` | DerbyInfo_scalarDeletingDtor | likely | Scalar deleting destructor for a DerbyInfo node: calls the destructor body 009ee630 and frees this through _free when bit 0 of param_2 is set; returns this oth… |
| `009effd0` | DerbyInfo_ctorWrapper | likely | SEH-framed constructor wrapper: calls 009ef2a0 (the DerbyInfo initialiser, base constructor plus the IString member at +0x88) and returns param_1. |
| `009ee630` | — | unknown | Destructor of the 0xb0-byte DerbyInfo node: stores the IString vtable at +0x88, and when the length dword at +0x94 is positive atomically decrements the refcou… |

**ProxiedVehicleDemolitionDerby** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c65780` | ProxiedVehicleDemolitionDerby::AdjustorThunk50ToScalarDtor | confirmed | Adjustor thunk: 83 e9 50 subtracts 0x50 from ECX and jumps to 00c65890 (8 bytes). It is vftable slot [0] of 01833190, the secondary vtable of ProxiedVehicleDem… |
| `00c65790` | ProxiedVehicleDemolitionDerby::AdjustorThunk54ToScalarDtor | confirmed | Adjustor thunk: 83 e9 54 subtracts 0x54 from ECX and jumps to 00c65890 (8 bytes). It is vftable slot [0] of 0183317c, the third vtable of ProxiedVehicleDemolit… |
| `00c65890` | ProxiedVehicleDemolitionDerby::ScalarDeletingDtor | confirmed | Scalar deleting destructor in vftable slot [6] of 018331cc ProxiedVehicleDemolitionDerby::vftable: calls the body 00c656b0, tests bit 0 of the flag argument an… |
| `009eb5b0` | ProxiedVehicleDemolitionDerby_DtorWrapper | likely | SEH-framed wrapper preserves incoming ECX as the receiver for destructor body 00c656b0, restores the prior exception chain, and returns; the stack-cookie push … |
| `00c656b0` | ProxiedVehicleDemolitionDerby::DtorBody | likely | Destructor body (no free). Sets vtable 018331cc at this, 01833190 at this+0x50 and 0183317c at this+0x54, zeroes the byte and float fields at this+0x1bc0..0x1b… |
| `00c657a0` | ProxiedVehicleDemolitionDerby::Ctor | likely | Constructor with five stack args (ret 0x14). Calls 00c59a10 with ECX = this and two values from the caller's argument structure, installs vtables 018331cc (pri… |
| `009eb8e0` | — | unknown | SEH-framed placement-construction wrapper: calls FUN_00c657a0(*param_2, *param_3), i.e. the actor constructor receives the two dwords its pointer arguments poi… |
| `009ecd70` | — | unknown | Scalar deleting destructor: calls FUN_00c656b0 (the ProxiedVehicleDemolitionDerby destructor) and frees this through _free when bit 0 of param_2 is set; return… |

**ProxiedVehicleDemolitionDerbyPickup** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c65ad0` | ProxiedVehicleDemolitionDerbyPickup::ScalarDeletingDtor | confirmed | Scalar deleting destructor in vftable slot [0] of 0183324c ProxiedVehicleDemolitionDerbyPickup::vftable: calls the body 00c65a10, tests bit 0 of the flag argum… |
| `009eb600` | ProxiedVehicleDemolitionDerbyPickup_DtorWrapper | likely | SEH-framed wrapper preserves incoming ECX as the receiver for destructor body 00c65a10, restores the prior exception chain, and returns; the stack-cookie push … |
| `009eb960` | ProxiedVehicleDemolitionDerbyPickup_ConstructWrapper | likely | SEH-framed placement-construction wrapper: calls FUN_00c65aa0(*param_2, *param_3) and returns param_1. |
| `00c65aa0` | ProxiedVehicleDemolitionDerbyPickup::Ctor | likely | Constructor with two stack args (ret 8). Stores arg2 at this+4, installs vtable 0183324c, stores arg1 at this+0x18, copies the dword at arg1+4 to this+0x1c, an… |
| `009ecdd0` | — | unknown | Scalar deleting destructor: calls FUN_00c65a10 (the ProxiedVehicleDemolitionDerbyPickup destructor) and frees this through _free when bit 0 of param_2 is set; … |
| `00c65a10` | — | unknown | Destructor body (no free). Sets vtable 0183324c at this. If this+0x20 is non-null it calls 007d01f0 and 007d06b0 (push 1) on it and clears +0x20. If this+0x14 … |

**ProxiedVehicleDerbyCrown** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c66ca0` | ProxiedVehicleDerbyCrown::ScalarDeletingDtor | confirmed | Scalar deleting destructor in vftable slot [0] of 0183334c ProxiedVehicleDerbyCrown::vftable: installs the vtable, passes the dword at [this+4]+0x268 through t… |
| `00c66ad0` | ProxiedVehicleDerbyCrown::Ctor | likely | Constructor with one stack arg (ret 4). Stores arg at this+8, installs vtable 0183334c, copies the dword at arg+4 to this+0xc, and zeroes this+4 and this+0x10.… |

**SoeUtil::Array<DemolitionDerbyConfigNpcData,0,1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0105ee30` | ArrayCapacityAllocVirt1_0x14 | confirmed | Virtual slot [1] of the DemolitionDerbyConfigNpcData array (thiscall, flag byte). Current capacity = [this+0xc]. Without the flag: when capacity < n the new ca… |
| `0105eed0` | VirtFree_Align4_DerbyNpc | confirmed | Virtual slot [2] of the DemolitionDerbyConfigNpcData array. Calls 00401c00(p, 4) and returns. |
| `0105eef0` | ArraySwapHeader_0x14 | confirmed | Virtual slot [3] of the DemolitionDerbyConfigNpcData array (thiscall with the other array pointer and a flag byte). With the flag set it calls vtable slot [3] … |

**SoeUtil::Array<DemolitionDerbyConfigStartLocation,0,1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0105ef60` | ArrayCapacityAllocVirt1_0x30 | confirmed | Virtual slot [1] of the DemolitionDerbyConfigStartLocation array (thiscall, flag byte). Same capacity policy as ArrayCapacityAllocVirt1_0x14 (growth to (5n)/4-… |
| `0105f000` | VirtFree_Align16_DerbyStartLoc | confirmed | Virtual slot [2] of the DemolitionDerbyConfigStartLocation array. Calls 00401c00(p, 0x10) and returns. |

**SoeUtil::Array<VehicleAssistTierConfig,0,1>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01054490` | ArrayAllocateGrow_AssistTier | confirmed | Virtual slot [1] (thiscall; stack args: requested count, out pointer for the new capacity, flag byte). Same capacity policy as 010541d0. Stores the capacity at… |
| `01054530` | ArrayFreeAlign4_AssistTier | confirmed | Virtual slot [2] (thiscall; stack argument = block pointer). Calls 00401c00(block, 4) to free the block. Returns nothing. |
| `01054550` | ArraySwapOrForward_AssistTier | confirmed | Virtual slot [3] (thiscall; stack args: other array pointer, flag byte). Flag nonzero: calls the other array's vtable slot [3] and returns. Flag zero: exchange… |
| `01056120` | ArrayDtorBody_AssistTier0 | confirmed | Fastcall (this in ECX). Stores the Array<VehicleAssistTierConfig,0,1> vftable, releases the elements through 009d6480(count), frees the block through 00401c00 … |
| `01056220` | ArrayScalarDeletingDtor_AssistTier0 | confirmed | Virtual slot [0] (thiscall; flag byte on the stack). Stores the vftable, releases the elements through 009d6480(count), frees the block through 00401c00 with a… |

**SoeUtil::Array<VehicleAssistTierConfig,1,1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010545c0` | ArrayFreeIfNotInline_AssistTier1 | confirmed | Virtual slot [2] (thiscall; stack argument = block pointer). When the block is not the inline buffer (this+0x13) & ~3 it calls 00401c00(block, 4). Returns noth… |
| `01056150` | ArrayDtorBody_AssistTier1 | confirmed | Fastcall (this in ECX) with an EH frame. Stores the Array<VehicleAssistTierConfig,1,1> vftable, releases the elements through 009d6480(count), frees the block … |
| `010561f0` | ArrayInline1OrAllocate_AssistTier | confirmed | Virtual slot [1] (thiscall; stack args: requested count, out pointer for the new capacity). When the count is above 1 returns 01054490 (heap allocation path of… |
| `01056970` | ArrayCtorInline_AssistTier1 | confirmed | Fastcall ctor (this in ECX). Stores the Array<VehicleAssistTierConfig,0,1> vftable, zeroes data, count and capacity, then stores the Array<VehicleAssistTierCon… |
| `010569d0` | ArrayScalarDeletingDtor_AssistTier1 | confirmed | Virtual slot [0] (thiscall; flag byte on the stack). Calls the dtor body 01056150, then if bit 0 of the flag is set calls _free on this. Returns this. |
| `01057460` | ArrayMoveViaHelpers_AssistTier1 | speculative | Virtual slot [3] (thiscall; other array pointer on the stack). Calls 009d6c50(data, count) for the other array and for this, with 009d6480(count) teardown on e… |

**SoeUtil::Array<VehicleDamageLevelConfig,0,1>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010545e0` | ArrayAllocateGrow_DamageLevel | confirmed | Virtual slot [1] (thiscall; stack args: requested count, out pointer for the new capacity, flag byte). Same capacity policy as 010541d0. Stores the capacity at… |
| `01054680` | ArrayFreeAlign4_DamageLevel | confirmed | Virtual slot [2] (thiscall; stack argument = block pointer). Calls 00401c00(block, 4) to free the block. Returns nothing. |
| `010546a0` | ArraySwapOrForward_DamageLevel | confirmed | Virtual slot [3] (thiscall; stack args: other array pointer, flag byte). Flag nonzero: calls the other array's vtable slot [3] and returns. Flag zero: exchange… |
| `01055ad0` | ArrayDtorBody_DamageLevel0 | confirmed | Fastcall (this in ECX). Stores the Array<VehicleDamageLevelConfig,0,1> vftable, releases the elements through 009d4de0(count), frees the block through 00401c00… |
| `01055bc0` | ArrayScalarDeletingDtor_DamageLevel0 | confirmed | Virtual slot [0] (thiscall; flag byte on the stack). Stores the vftable, releases the elements through 009d4de0(count), frees the block through 00401c00 with a… |

**SoeUtil::Array<VehicleDamageLevelConfig,7,0>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01054710` | ArrayFreeIfNotInline_DamageLevel7 | confirmed | Virtual slot [2] (thiscall; stack argument = block pointer). When the block is not the inline buffer (this+0x13) & ~3 it calls 00401c00(block, 4). Returns noth… |
| `01055b00` | ArrayDtorBody_DamageLevel7 | confirmed | Fastcall (this in ECX) with an EH frame. Stores the Array<VehicleDamageLevelConfig,7,0> vftable, releases the elements through 009d4de0(count), frees the block… |
| `01055ba0` | ArrayInlineCapacityGet_DamageLevel7 | confirmed | Virtual slot [1] (thiscall; stack args: an unused dword and an out pointer). Writes 7 to the out pointer and returns the inline buffer (this+0x13) & ~3. No all… |
| `01055c50` | ArrayCtorInline_DamageLevel7 | confirmed | Fastcall ctor (this in ECX). Stores the Array<VehicleDamageLevelConfig,0,1> vftable, zeroes data, count and capacity, then stores the Array<VehicleDamageLevelC… |
| `01055cb0` | ArrayScalarDeletingDtor_DamageLevel7 | confirmed | Virtual slot [0] (thiscall; flag byte on the stack). Calls the dtor body 01055b00, then if bit 0 of the flag is set calls _free on this. Returns this. |
| `01056020` | ArrayMoveViaHelpers_DamageLevel7 | speculative | Virtual slot [3] (thiscall; other array pointer on the stack). Calls 009d5f00(data, count) for the other array and for this, with 009d4de0(count) teardown on e… |

**SoeUtil::Array<VehiclePartParticleConfig,0,1>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010541d0` | ArrayAllocateGrow_PartCfg | confirmed | Virtual slot [1] (thiscall; stack args: requested count, out pointer for the new capacity, flag byte). Flag 0: capacity grows to about 5/4 of the count when it… |
| `01054270` | ArrayFreeAlign16_PartCfg | confirmed | Virtual slot [2] (thiscall; stack argument = block pointer). Calls 00401c00(block, 0x10) to free the block with alignment 16. Returns nothing. |
| `01054290` | ArraySwapOrForward_PartCfg | confirmed | Virtual slot [3] (thiscall; stack args: other array pointer, flag byte). Flag nonzero: calls the other array's vtable slot [3] and returns. Flag zero: exchange… |
| `01054af0` | ArrayDtorBody_PartCfg0 | confirmed | Fastcall (this in ECX). Stores the Array<VehiclePartParticleConfig,0,1> vftable, clears the count at +8, frees the block through 00401c00 with alignment 0x10 a… |
| `01054e10` | ArrayScalarDeletingDtor_PartCfg0 | confirmed | Virtual slot [0] (thiscall; flag byte on the stack). Stores the vftable, clears the count, frees the block through 00401c00 with alignment 0x10 and clears the … |

**SoeUtil::Array<VehiclePhysicsGear,0,1>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01054340` | ArrayAllocateGrow_Gear | confirmed | Virtual slot [1] (thiscall; stack args: requested count, out pointer for the new capacity, flag byte). Same capacity policy as 010541d0 (5/4 grow, 4/3 keep, 6/… |
| `010543e0` | ArrayFreeAlign4_Gear | confirmed | Virtual slot [2] (thiscall; stack argument = block pointer). Calls 00401c00(block, 4) to free the block. Returns nothing. |
| `01054400` | ArraySwapOrForward_Gear | confirmed | Virtual slot [3] (thiscall; stack args: other array pointer, flag byte). Flag nonzero: calls the other array's vtable slot [3] and returns. Flag zero: exchange… |
| `01054c80` | ArrayDtorBody_Gear0 | confirmed | Fastcall (this in ECX). Stores the Array<VehiclePhysicsGear,0,1> vftable, clears the count at +8, frees the block through 00401c00 with alignment 4 and clears … |
| `01054e50` | ArrayScalarDeletingDtor_Gear0 | confirmed | Virtual slot [0] (thiscall; flag byte on the stack). Stores the vftable, clears the count, frees the block through 00401c00 with alignment 4 and clears the blo… |

**SoeUtil::Array<VehiclePhysicsGear,6,0>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01054470` | ArrayFreeIfNotInline_Gear6 | confirmed | Virtual slot [2] (thiscall; stack argument = block pointer). When the block is not the inline buffer (this+0x13) & ~3 it calls 00401c00(block, 4). Returns noth… |
| `01054cb0` | ArrayDtorBody_Gear6 | confirmed | Fastcall (this in ECX). Stores the Array<VehiclePhysicsGear,6,0> vftable, clears the count, frees the block through 00401c00(block, 4) when it is not the inlin… |
| `01054d40` | ArrayInlineCapacityGet_Gear6 | confirmed | Virtual slot [1] (thiscall; stack args: an unused dword and an out pointer). Writes 6 to the out pointer and returns the inline buffer (this+0x13) & ~3. No all… |
| `010553a0` | ArrayCtorInline_Gear6 | confirmed | Fastcall ctor (this in ECX). Stores the Array<VehiclePhysicsGear,0,1> vftable, zeroes data, count and capacity, then stores the Array<VehiclePhysicsGear,6,0> v… |
| `01055540` | ArrayScalarDeletingDtor_Gear6 | confirmed | Virtual slot [0] (thiscall; flag byte on the stack). Calls the dtor body 01054cb0, then if bit 0 of the flag is set calls _free on this. Returns this. |
| `01055940` | — | unknown | Virtual slot [3] (thiscall; other array pointer on the stack). Calls 009d39d0 (12-byte stride expansion helper, label supporting only) on the two arrays, reset… |

**VehicleConfig** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01056260` | VehicleConfig_DtorBody | confirmed | Fastcall dtor body (this in ECX) with an EH frame. Stores VehicleConfig::vftable at *this and then destroys its member arrays and strings through the helpers 0… |
| `010569f0` | VehicleConfig_Ctor | confirmed | Fastcall ctor (this in ECX, large body of about 2.4 KB). Stores VehicleConfig::vftable at *this, installs IString<char> members with the static empty buffer 01… |
| `010573f0` | VehicleConfig_ScalarDeletingDtor | confirmed | Virtual slot [0] of VehicleConfig::vftable (thiscall; flag byte on the stack). Calls the dtor body 01056260, then if bit 0 of the flag is set calls _free on th… |

**VehicleDemolitionDerbyProcessor** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ee280` | VehicleDemolitionDerbyProcessor_LoadSoundEvents | confirmed | Resolves 19 event names through FUN_0070aab0 (Vehicle_Countdown_Beep, Vehicle_Countdown_Go, Vehicle_Finish_Win, Vehicle_Finish_Lose, UI_LowHealthNotify, Vehicl… |
| `009f01f0` | VehicleDemolitionDerbyProcessor::RefreshHudValues | confirmed | Returns unless this+0x184 (local vehicle) and its +0x10b0 state object are non-null. Otherwise pushes values from the state object (float at +0x114 as abs, dwo… |
| `009f0f30` | VehicleDemolitionDerbyProcessor::VehicleDemolitionDerbyProc… | confirmed | Installs VehicleDemolitionDerbyProcessor::vftable; stores param_2 at +4; clears +8, +0xc, +0x184 and several flag bytes; installs SoeUtil::List vtables and zer… |
| `009f11a0` | VehicleDemolitionDerbyProcessor::~VehicleDemolitionDerbyPro… | confirmed | Installs the processor vtable, removes all nodes of the vehicle list (+0x178, via 009ed6e0), pickup list (+0x18c, 009ed780), results list (009ee050) and DerbyI… |
| `009eb140` | VehicleDemolitionDerbyProcessor_ApplyFloatTo17Handles | likely | For each of the 17 element handles at this+0xf0, +0xf8, +0x100 ... +0x170 (stride 8): when the handle is non-null it calls FUN_0078fcc0 with the handle as this… |
| `009eb300` | VehicleDemolitionDerbyProcessor_StopFourHandles | likely | For the four element handles at this+0x100, +0x130, +0x150 and +0x168: when non-null, sets the element's byte +0x4d to 1 and calls the element's vtable slot 3 … |
| `009eb380` | VehicleDemolitionDerbyProcessor_CheckLocalVehicleThreshold | likely | Returns FUN_00fe3e10(localState, DAT_017ed858) when this+0x184 (the local vehicle pointer) and its +0x10b0 state object are both non-null, otherwise 0; FUN_00f… |
| `009eacc0` | — | unknown | For 19 pairs starting at this+0xdc with stride 8, when the first pointer is non-null it calls the second pointer virtual slot 0 with argument 1 if present, cle… |
| `009ef180` | — | unknown | Virtual method (processor vtable slot 3): with the local actor at +0x184 set it builds a temporary SoeUtil::List<DerbyResultsStruct,-1> holding one node per ve… |
| `009f20f0` | — 🌐 | unknown | Returns unless the byte at this+0xce is set. Subtracts param_2 times a constant from the float at this+0xc4; when it is <= 0 resets it to a constant, pushes th… |

**(no class)** (34)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009f1b80` | VehicleDemolitionDerby_SetActive 🌐 | confirmed | Stores the byte param_2 at this+8 and does nothing more when the value is unchanged. On change to 0: hides the demolition HUD widgets (Lua), calls 00d36480(1) … |
| `009f2d90` | VehicleDemolitionDerby_UpdateStateMachine 🌐 | confirmed | Returns unless the local vehicle (this+0x184) is non-null and this+0xc is at least 2. Switches on the local state dword at +0x1624: states 6, 7 and 8 loop 00c5… |
| `009eaad0` | Copy64BytesAndSetFlag | likely | Copies 16 dwords (64 bytes) from param_2 to this+0x1540, then writes 1 to byte this+0x15c0 and returns. |
| `009ed1e0` | DerbyVehicleList_FindByKeyPair | likely | Walks the list whose head sits at this+0x178 following the next links at node+0x1bf0 and returns the first node whose dwords at +0xe60 and +0xe64 equal (param_… |
| `009ed220` | DerbyInfoList_FindByKeyPair | likely | Walks the DerbyInfo list whose head sits at this+0x1a0 following the next links at node+0xa8 and returns the first node whose dwords at +0x98 and +0x9c equal (… |
| `009ed260` | DerbyPickupList_FindById | likely | Walks the pickup list whose head sits at this+0x18c following the next links at node+0x24 and returns the first node whose dword at +4 equals param_2, else 0. |
| `009ed350` | DerbyResultsList_FindByKeyPair | likely | Walks the list whose head sits at this+0x1b0 following the next links at node+0x18 and returns the first node whose dwords at +0 and +4 equal (param_2, param_3… |
| `009ed3d0` | DerbyStartGate_AllElementsAndVehiclesReady | likely | Readiness gate: for each of the 19 element slots at this+0xdc..+0x16c it calls the element's vtable slot 3 and requires a non-zero result (short-circuiting on … |
| `009ed610` | DerbyVehicles_AllHaveStateObject | likely | Returns false as soon as a node of the vehicle list at this+0x178 has a null +0x10b0 state object, otherwise true (raw bytes: loop on +0x1bf0 comparing the dwo… |
| `009ed640` | DerbyVehicles_SetRoundFlagAndClearField | likely | For every vehicle in the list at this+0x178 that has a state object at +0x10b0: stores param_2 into the state's byte +0x18c and zeroes the state dword at +0x19… |
| `009ed680` | DerbyVehicles_CountFlaggedEqualsD0 | likely | Counts the nodes of the vehicle list at this+0x178 whose byte at +0xf00 is non-zero and returns whether that count equals the dword at this+0xd0. |
| `009ed6b0` | DerbyVehicles_CountUnflaggedEqualsD4 | likely | Counts the nodes of the vehicle list at this+0x178 whose byte at +0xf00 is zero and returns whether that count equals the dword at this+0xd4. |
| `009f0830` | RecordCopyAssign_ArraysAt8cAnd9c | likely | Copy-assigns param_2 into this and returns this: calls 006f9e60 with source+4 and source+0x14, copies the dword at +0x24, six dwords at +0x28..+0x3c and six at… |
| `009f22c0` | VehicleDemolitionDerby_TickVehicles 🌐 | likely | Per-frame pass over the derby vehicle list (head +0x178, next +0x1bf0): for each vehicle other than the local one that has a state object (+0x10b0) and flag +0… |
| `009ed380` | ForEachDerbyVehicle_CallTwoStubs | speculative | Walks the vehicle list at this+0x178 twice: the first loop calls FUN_00c548c0() once per node, the second calls FUN_00c548d0() once per node; both callees are … |
| `00c658d0` | DerbyRecordCopyAssign | speculative | Thiscall with one stack arg (source record), returns this (ret 4). Copies dwords +4, +8, +0x10, +0x14, the 16 dwords at +0x20 (rep movsd), and dwords +0x60 to … |
| `00c659b0` | DerbyRecordCopyIfNewer | speculative | Thiscall with one stack arg (source record). If byte this+0x1be0 is zero it copies the record into this+0x1b40 through 00c658d0. Otherwise it takes the 64-bit … |
| `00c660c0` | DsDerbyResultsArrayTruncate | speculative | Thiscall with one stack arg (a count). Computes new = this[+8] - arg. When new is below the old count it calls 00c66030 on each element from index new to the o… |
| `00c66100` | DsDerbyResultsElementCopyCtor | speculative | Copy constructor of a 0x20-byte element, one stack arg (source element) with ret 4: copies dwords +0 and +4, initializes the embedded IString at +8 (vtable 017… |
| `00c66300` | DsDerbyResultsArrayEnsureIndex | speculative | Thiscall with one stack arg (index), returns a pointer to element index (ret 4). If index is below the count at this+8 it releases that element's embedded IStr… |
| `00c663d0` | DsDerbyResultsArrayAppend | speculative | Thiscall with no stack args (ret 0). Grows the array by one when the count at this+8 is at its capacity (00c661a0), takes the element at index count, increment… |
| `00c66450` | DsDerbyResultsEnsureElementInit | speculative | Thiscall with one stack arg (index, ret 4). Calls 00c66300 with the index to obtain the element, initialises that element's embedded IString through 00c65fd0 w… |
| `00c66590` | DsDerbyResultsGetOrCreate | speculative | Thiscall with one stack arg (index, ret 4). When the index is at or past the count at this+8 it calls 00c66450 with index to create it. Returns the element add… |
| `00c66780` | DerbyResultsPopulateRows | speculative | Thiscall with one stack arg (a source list of result records). Reads the row count through vtable+8 of [this+0x30]. For each row index it grows or initialises … |
| `00c66af0` | DerbyMemberSetFlagBit1AndNotify | speculative | Thiscall with one stack arg (bool). If the object at this+4 is non-null, it sets bit 1 of the word at +0x10 to the argument, releasing the object at +4 through… |
| `00c66b40` | DerbyMemberGetFlagBit1 | speculative | Thiscall (ret 0). Returns bit 1 of the word at +0x10 of the object at this+4, or 0 when that object is null. |
| `009ed290` | — | unknown | Scans the vehicle list at this+0x178; for every node with a non-null +0x10b0 state object it compares the state dword at +0x124, keeps the node holding the str… |
| `009ed2e0` | — | unknown | Finds the node with the greatest +0x10b0->+0x124 value in the list at this+0x178 and then writes the boolean (node == that maximum node) into the byte at state… |
| `009ed5a0` | — | unknown | Loads the object at *(this+4)+0xc8, calls its vtable slot 9 and, when that returns non-zero, calls FUN_0076c590 and requires the byte at (result+0x232); the re… |
| `009edf00` | — | unknown | Copy-constructs a VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyData record into param_1+0x1b40 through the copy ctor 009ed0b0 and returns param_2 (t… |
| `00c65af0` | — | unknown | Thiscall with four stack args (ret 0x10) that returns a bool in AL. Zeroes a temporary record through 0079d310, reads the object from 00769710 (its +0x1c membe… |
| `00c661a0` | — | unknown | Thiscall with two stack args (ret 8). Calls vtable+4 of this (00c65ea0) to obtain the new element count and storage, then when the count changes it constructs … |
| `00c66b80` | — | unknown | Thiscall (ret 4, one stack arg, a float). Returns early when this+0x10 or this+4 is null, or the object at this+0x10 has no flag at +0x10b0 or the flag bit at … |
| `00c66ce0` | — | unknown | Thiscall with two stack args (ret 8). Zeroes a 0x1e0-byte stack object through 0079d310 and loads the dword at +0x1c of the object from 00769710 into its first… |

### `net.derby`

**BaseVehicleDemolitionDerbyPacket** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009e9f20` | BaseVehicleDemolitionDerbyPacket_ctorZero 🌐 | confirmed | Constructor: stores the BasePacket vtable, writes the derby base opcode 0x22 at +4, overwrites the vtable with BaseVehicleDemolitionDerbyPacket::vftable and st… |
| `009e9f80` | BaseVehicleDemolitionDerbyPacket_ctor 🌐 | confirmed | Constructor: stores the BasePacket vtable, writes the base opcode 0x22 at +4, overwrites the vtable with BaseVehicleDemolitionDerbyPacket::vftable and stores t… |
| `009e9ff0` | BaseVehicleDemolitionDerbyPacket_dtor | confirmed | Destructor body: rewrites the vtable to BaseVehicleDemolitionDerbyPacket::vftable and then to BasePacket::vftable, with no member teardown. |
| `009ea040` | BaseVehicleDemolitionDerbyPacket_scalarDeletingDtor | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket and BasePacket vtables and frees this through _free when bit 0 of the flag byte is se… |
| `009ec880` | BaseVehicleDemolitionDerbyPacket_CopyCtor | likely | Copy-constructor body of the derby packet base: writes BasePacket::vftable, copies the source dword at +4 (the base opcode field), then writes BaseVehicleDemol… |

**VehicleDemolitionDerbyPacketAnnouncerMessage** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ec750` | VehicleDemolitionDerbyPacketAnnouncerMessage_ctor 🌐 | confirmed | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 0x14 and stores the AnnouncerMessage vtable; returns this. |
| `009ec7b0` | VehicleDemolitionDerbyPacketAnnouncerMessage_scalarDeleting… | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |
| `009eeeb0` | VehicleDemolitionDerbyPacketAnnouncerMessage_ReadBody 🌐 | likely | Payload reader for subtype 0x14: header pair via 009ee770, two dwords at +0xc/+0x10 and two 64-bit values at +0x18 and +0x20; consumes exactly the 24-byte capt… |
| `009eff60` | VehicleDemolitionDerbyPacketAnnouncerMessage_Deserialize 🌐 | likely | Deserialises one subtype-0x14 payload through the 28-byte reader 009eeeb0 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and th… |

**VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionDerby** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009edde0` | VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionD… 🌐 | confirmed | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 10, stores the CreateProxiedVehicleDemolitionDerby vtable, initialises the embedded I… |
| `009ee200` | VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionD… | confirmed | Scalar deleting destructor: calls the destructor body 009ede60 (releases the embedded IString member and restores the base vtables) and frees this through _fre… |
| `009ede60` | — | unknown | Destructor body: calls FUN_01056260, stores the IString vtable at +0x18 and releases the member's buffer (atomic decrement in front of the pointer at +0x1c; fr… |

**VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionDerbyPickup** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ecfd0` | VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionD… 🌐 | confirmed | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 0xc, stores the CreateProxiedVehicleDemolitionDerbyPickup vtable and zeroes the membe… |
| `009ed030` | VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionD… | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |
| `009ee9e0` | VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionD… 🌐 | likely | Payload reader for subtype 0x0c: header pair via 009ee770, dwords into +0xc and +0x10, a 16-byte float4 block at +0x20 through FUN_008e2410 and floats at +0x30… |
| `009efb00` | VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionD… 🌐 | likely | Deserialises one subtype-0x0c payload through the 36-byte reader 009ee9e0 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and th… |

**VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolitionDerby** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ec1e0` | VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolition… 🌐 | confirmed | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 8 and stores the DestroyProxiedVehicleDemolitionDerby vtable; returns this. |
| `009ec240` | VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolition… | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |
| `009ef460` | VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolition… 🌐 | likely | Payload reader: header pair through 009ee770 and a 64-bit value into the record's +0x10/+0x14, with the usual zeroing and overrun flag. |
| `009efc50` | — 🌐 | unknown | Deserializes subtype 8 into param_1 over (param_2,param_3) through reader 009ef460, which consumes the 4-byte opcode/subtype header and one 8-byte key (12 byte… |

**VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolitionDerbyPickup** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ea450` | VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolition… 🌐 | confirmed | Constructor: runs the derby base constructor with sub-opcode 0xd and stores the destroy-proxied-pickup vtable. |
| `009ec0a0` | VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolition… | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |
| `009ef3d0` | VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolition… 🌐 | likely | Payload reader: header pair through 009ee770 and one dword from the stream into the record's +0xc (zeroed with the overrun flag on failure); consumes exactly t… |
| `009efb70` | VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolition… 🌐 | likely | Deserialises one subtype-0x0d payload through the 8-byte reader 009ef3d0 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the… |

**VehicleDemolitionDerbyPacketItemExpired** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ea690` | VehicleDemolitionDerbyPacketItemExpired_ctor 🌐 | confirmed | Constructor: runs the derby base constructor with sub-opcode 0x11 and stores the VehicleDemolitionDerbyPacketItemExpired vtable. |
| `009ec520` | VehicleDemolitionDerbyPacketItemExpired_scalarDeletingDtor | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |
| `009ef4d0` | VehicleDemolitionDerbyPacketItemExpired_ReadBody 🌐 | speculative | Payload reader: header pair through 009ee770 and one dword from the stream into the record's +0xc, with the usual zeroing and overrun flag. |
| `009efda0` | VehicleDemolitionDerbyPacketItemExpired_Deserialize 🌐 | speculative | Deserialises one subtype-0x11 payload through the 8-byte reader 009ef4d0 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the… |

**VehicleDemolitionDerbyPacketKickProxiedVehicleDemolitionDerby** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ec110` | VehicleDemolitionDerbyPacketKickProxiedVehicleDemolitionDer… 🌐 | confirmed | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 9 and stores the KickProxiedVehicleDemolitionDerby vtable; returns this. |
| `009ec170` | VehicleDemolitionDerbyPacketKickProxiedVehicleDemolitionDer… | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |
| `009ef410` | VehicleDemolitionDerbyPacketKickProxiedVehicleDemolitionDer… 🌐 | likely | Payload reader: header pair through 009ee770 and a 64-bit value into the record's +0x10/+0x14 (both zeroed with the overrun flag on failure). |
| `009efbe0` | — 🌐 | unknown | Deserialises one subtype-9 payload through the 16-byte reader 009ef410 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the b… |

**VehicleDemolitionDerbyPacketPickupHitRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ec380` | VehicleDemolitionDerbyPacketPickupHitRequest_ctor 🌐 | confirmed | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 0xe and stores the PickupHitRequest vtable; returns this. |
| `009ec3e0` | VehicleDemolitionDerbyPacketPickupHitRequest_scalarDeleting… | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |

**VehicleDemolitionDerbyPacketPickupHitResponse** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ec4b0` | VehicleDemolitionDerbyPacketPickupHitResponse_scalarDeletin… | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |
| `009ec450` | VehicleDemolitionDerbyPacketPickupHitResponse_ctor 🌐 | likely | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 0xf and stores the PickupHitResponse vtable; returns this. |
| `009eec60` | VehicleDemolitionDerbyPacketPickupHitResponse_ReadBody 🌐 | likely | Payload reader for subtype 0x0f: header pair via 009ee770, a 64-bit key at +0x10/+0x14, a dword at +0x18 and three floats at +0x1c/+0x20/+0x24; consumes exactl… |
| `009efd30` | VehicleDemolitionDerbyPacketPickupHitResponse_Deserialize 🌐 | likely | Deserialises one subtype-0x0f payload through the 28-byte reader 009eec60 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and th… |

**VehicleDemolitionDerbyPacketRegisterVehicleDemolitionDerbyPlayer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ebf00` | VehicleDemolitionDerbyPacketRegisterVehicleDemolitionDerbyP… 🌐 | confirmed | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 6 and stores the RegisterVehicleDemolitionDerbyPlayer vtable; returns this. |
| `009ebf60` | VehicleDemolitionDerbyPacketRegisterVehicleDemolitionDerbyP… | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |

**VehicleDemolitionDerbyPacketResetProxiedVehicleDemolitionDerby** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ec660` | VehicleDemolitionDerbyPacketResetProxiedVehicleDemolitionDe… 🌐 | confirmed | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 0x12 and stores the ResetProxiedVehicleDemolitionDerby vtable; returns this. |
| `009ec6c0` | VehicleDemolitionDerbyPacketResetProxiedVehicleDemolitionDe… | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |
| `009ef520` | VehicleDemolitionDerbyPacketResetProxiedVehicleDemolitionDe… 🌐 | likely | Payload reader: header pair through 009ee770 and a 64-bit value into the record's +0x10/+0x14, with the usual zeroing and overrun flag; consumes exactly the 8-… |
| `009efe80` | — 🌐 | unknown | Deserialises one subtype-0x12 payload through the 16-byte reader 009ef520 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and th… |

**VehicleDemolitionDerbyPacketSetDerbyConfig** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ec730` | VehicleDemolitionDerbyPacketSetDerbyConfig_scalarDeletingDt… | confirmed | Scalar deleting destructor: calls the destructor body FUN_009ea840 (which destroys the embedded DemolitionDerbyConfig member through FUN_0105f240 and restores … |
| `009ea840` | VehicleDemolitionDerbyPacketSetDerbyConfig_dtor | likely | Destructor: destroys the embedded DemolitionDerbyConfig member with FUN_0105f240 and then restores the BaseVehicleDemolitionDerbyPacket and BasePacket vtables. |
| `009ea7e0` | — 🌐 | unknown | Constructor: runs the derby base constructor with sub-opcode 0x13, stores the VehicleDemolitionDerbyPacketSetDerbyConfig vtable and constructs the embedded Dem… |

**VehicleDemolitionDerbyPacketUpdateNpcCount** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ea260` | VehicleDemolitionDerbyPacketUpdateNpcCount_ctor 🌐 | confirmed | Constructor: runs the derby base constructor with sub-opcode 4 and stores the VehicleDemolitionDerbyPacketUpdateNpcCount vtable. |
| `009ebdc0` | VehicleDemolitionDerbyPacketUpdateNpcCount_scalarDeletingDt… | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |
| `009ef370` | VehicleDemolitionDerbyPacketUpdateNpcCount_ReadBody 🌐 | likely | Payload reader: header pair through 009ee770 and one dword from the stream into the record's +0xc, with the usual zeroing and overrun flag; consumes exactly th… |
| `009efa20` | VehicleDemolitionDerbyPacketUpdateNpcCount_Deserialize 🌐 | likely | Deserialises one subtype-4 payload through the record reader 009ef370 into param_1 over (param_2, param_3) and returns 1 only when no error was flagged and the… |

**VehicleDemolitionDerbyPacketUpdatePlayerCount** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ea1b0` | VehicleDemolitionDerbyPacketUpdatePlayerCount_ctor 🌐 | confirmed | Constructor: runs the derby base constructor with sub-opcode 3 and stores the VehicleDemolitionDerbyPacketUpdatePlayerCount vtable. |
| `009ebd50` | VehicleDemolitionDerbyPacketUpdatePlayerCount_scalarDeletin… | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |
| `009ef330` | VehicleDemolitionDerbyPacketUpdatePlayerCount_ReadBody 🌐 | likely | Payload reader: header pair through 009ee770 (reader in param_1, record in param_2 here) and one dword from the stream into the record's +0xc, zeroing it and s… |
| `009ef9b0` | VehicleDemolitionDerbyPacketUpdatePlayerCount_Deserialize 🌐 | likely | When param_1 and param_2 are non-null, builds a 0x14-byte stack reader record with four dwords (buffer, length, cursor, end) and two status bytes, then calls 0… |

**VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyData** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ebe30` | VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyDat… 🌐 | confirmed | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 5 and stores the VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyData vtable;… |
| `009ebe90` | VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyDat… | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |
| `009ef5e0` | VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyDat… 🌐 | likely | Payload reader for subtype 5: header pair via 009ee770, a 64-bit key at +0x10/+0x14, six 16-byte float4 blocks at +0x20, +0x30, +0x40, +0x50, +0x60 and +0x70 r… |
| `009ed0b0` | — | unknown | Copy constructor: runs the base copy body 009ec880 on the source packet, stores the UpdateVehicleDemolitionDerbyData vtable, then copies the remaining 0x90 pay… |

**VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyStateData** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ec2b0` | VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbySta… 🌐 | confirmed | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 0xb and stores the UpdateVehicleDemolitionDerbyStateData vtable; returns this. |
| `009ec310` | VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbySta… | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |
| `009ee850` | VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbySta… 🌐 | likely | Payload reader for subtype 0x0b: header pair via 009ee770, a 64-bit key into +0x10/+0x14, six dwords into +0x18..+0x2c, three bool bytes into +0x30..+0x32 and … |
| `009efa90` | — 🌐 | unknown | Deserialises one subtype-0x0b payload through the 51-byte reader 009ee850 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and th… |

**VehicleDemolitionDerbyPacketUpdateVehicleGamePlayerState** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ebc80` | VehicleDemolitionDerbyPacketUpdateVehicleGamePlayerState_ct… 🌐 | confirmed | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 2 and stores the VehicleDemolitionDerbyPacketUpdateVehicleGamePlayerState vtable; ret… |
| `009ebce0` | — | unknown | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set … |

**VehicleDemolitionDerbyPacketUpdateVehicleGameState** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ea0b0` | VehicleDemolitionDerbyPacketUpdateVehicleGameState_ctor 🌐 | confirmed | Constructor: runs the derby base constructor with sub-opcode 1 and stores the VehicleDemolitionDerbyPacketUpdateVehicleGameState vtable; returns this. |
| `009ef570` | VehicleDemolitionDerbyPacketUpdateVehicleGameState_ReadBody 🌐 | likely | Payload reader: header pair through 009ee770 and one dword from the stream into the record's +0xc, with the usual zeroing and overrun flag; consumes exactly th… |
| `009efef0` | VehicleDemolitionDerbyPacketUpdateVehicleGameState_Deserial… 🌐 | likely | Deserialises one subtype-1 payload through the 8-byte reader 009ef570 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the bu… |
| `009ebc10` | — | unknown | Scalar deleting destructor: restores the two base vtables in order (BaseVehicleDemolitionDerbyPacket::vftable then BasePacket::vftable) and, when bit 0 of para… |

**VehicleDemolitionDerbyPacketVehicleDemolitionDerbyCountdownStatus** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ec030` | VehicleDemolitionDerbyPacketVehicleDemolitionDerbyCountdown… | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |
| `009ebfd0` | VehicleDemolitionDerbyPacketVehicleDemolitionDerbyCountdown… 🌐 | likely | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 7 and stores the CountdownStatus vtable; returns this. |
| `009eedb0` | VehicleDemolitionDerbyPacketVehicleDemolitionDerbyCountdown… 🌐 | likely | Payload reader for subtype 7: header pair via 009ee770, a 64-bit key at +0x10/+0x14 and a dword at +0x18; consumes exactly the 12-byte captured payload plus th… |
| `009efe10` | VehicleDemolitionDerbyPacketVehicleDemolitionDerbyCountdown… 🌐 | likely | Deserialises one subtype-7 payload through the 16-byte reader 009eedb0 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the b… |

**VehicleDemolitionDerbyPacketVehicleHit** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ec590` | VehicleDemolitionDerbyPacketVehicleHit_ctor 🌐 | confirmed | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 0x10 and stores the VehicleHit vtable; returns this. |
| `009ec5f0` | VehicleDemolitionDerbyPacketVehicleHit_scalarDeletingDtor | confirmed | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set;… |
| `009efcc0` | VehicleDemolitionDerbyPacketVehicleHit_Deserialize 🌐 | speculative | Deserialises one subtype-0x10 payload through the 24-byte reader 009eeba0 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and th… |
| `009eeba0` | — 🌐 | unknown | Payload reader for subtype 0x10: header pair via 009ee770, then two 64-bit values (attacker at +0x10/+0x14, victim at +0x18/+0x1c) and a non-finite-checked flo… |

**(no class)** (35)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ea110` | DerbyPacketTearDown_009ea110 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea160` | DerbyPacketTearDown_009ea160 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea210` | DerbyPacketTearDown_009ea210 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea2c0` | DerbyPacketTearDown_009ea2c0 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea310` | DerbyPacketTearDown_009ea310 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea360` | DerbyPacketTearDown_009ea360 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea3b0` | DerbyPacketTearDown_009ea3b0 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea400` | DerbyPacketTearDown_009ea400 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea4b0` | DerbyPacketTearDown_009ea4b0 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea500` | DerbyPacketTearDown_009ea500 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea550` | DerbyPacketTearDown_009ea550 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea5a0` | DerbyPacketTearDown_009ea5a0 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea5f0` | DerbyPacketTearDown_009ea5f0 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea640` | DerbyPacketTearDown_009ea640 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea6f0` | DerbyPacketTearDown_009ea6f0 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea740` | DerbyPacketTearDown_009ea740 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea790` | DerbyPacketTearDown_009ea790 | confirmed | Teardown body: restores the BaseVehicleDemolitionDerbyPacket vtable and then the BasePacket vtable; identical body to the other derby teardown variants in this… |
| `009ea8a0` | DerbyPacketTearDown_009ea8a0 | confirmed | SEH-framed teardown body: writes BaseVehicleDemolitionDerbyPacket::vftable and then BasePacket::vftable into *param_1 and returns; it destroys no member and fr… |
| `009f1440` | SendRecord_Via_9f0a50 🌐 | confirmed | Serialises the record at param_1 with 009f0a50 into a local writer (PacketWriterInit with 0x10000000). Stores the writer's buffer span and the flag byte in the… |
| `009f1550` | SendRecord_HdrH10_Q20 🌐 | confirmed | Serialises the record at param_1 inline into a local writer (PacketWriterInit with 0x10000000): header via 009f0790, 8 bytes at record+0x10 via 008fb810, four … |
| `009f1680` | SendRecord_Via_9f0ae0 🌐 | confirmed | Serialises the record at param_1 with 009f0ae0 into a local writer (PacketWriterInit with 0x10000000). Stores the writer's buffer span and the flag byte in the… |
| `009f1790` | SendRecord_HdrH10_Ref18 🌐 | confirmed | Serialises the record at param_1 inline into a local writer (PacketWriterInit with 0x10000000): header via 009f0790, 8 bytes at record+0x10 via 008fb810, 8 byt… |
| `009f18b0` | SendRecord_Via_9f0b80 🌐 | confirmed | Serialises the record at param_1 with 009f0b80 into a local writer (PacketWriterInit with 0x10000000). Stores the writer's buffer span and the flag byte in the… |
| `009f1a70` | SendRecord_Via_9f0ce0 🌐 | confirmed | Serialises the record at param_1 with 009f0ce0 into a local writer (PacketWriterInit with 0x10000000). Stores the writer's buffer span and the flag byte in the… |
| `009ee190` | DemolitionDerbyConfig_Read 🌐 | likely | Reads two dwords from the stream param_2 into this+0x24 (with the non-finite check writing stream+0x11) and this+0x78, zeroing the destination and setting stre… |
| `009ee770` | DerbyRecord_ReadOpcodeAndSubtype 🌐 | likely | Reads two 16-bit values from the stream param_2 (bounds-checked: on overrun it zeroes the destination, sets the error byte at stream+0x10 and moves the cursor … |
| `009f0790` | RecordHeader_WriteU16Pair | likely | Writes two 16-bit header fields to the writer (ECX = record, stack arg = writer): the low word of record+4 and the low word of record+8, each through 007cc400 … |
| `009f0a50` | RecordSerialize_HdrH10_D18 | likely | Serialises a derby record into the writer (ECX = record, stack arg = writer): header via 009f0790; 8 bytes from record+0x10 through 008fb810; then min(writer r… |
| `009f0ae0` | RecordSerialize_HdrH10_H18_F20 | likely | Serialises a derby record into the writer: header via 009f0790; 8 bytes from record+0x10 via 008fb810; 8 bytes from record+0x18 via 008fb810; then min(writer r… |
| `009f0b80` | RecordSerialize_HdrH10_D18 | likely | Byte-identical to 009f0a50: header via 009f0790; 8 bytes from record+0x10 via 008fb810; min(writer remaining, 4) bytes of the dword at record+0x18 through 007c… |
| `009f0ce0` | RecordSerialize_Sub5Body | likely | Serialises a derby record into the writer: header via 009f0790; 8 bytes from record+0x10 via 008fb810; six 16-byte blocks from record+0x20, +0x30, +0x40, +0x50… |
| `009f3800` | ReadBody_Sub0xa | likely | Reads the u16 header pair via 009ee770, then 8 bytes into this+0x10/+0x14 (zeros and the stream error byte at stream+0x10 when the stream is short), four float… |
| `009f0690` | — | unknown | Returns 0 when param_1 or param_2 is null. Otherwise builds a local read stream (begin=param_2, length=param_3, cursor=param_2, end=param_2+param_3, error byte… |
| `009f0700` | — | unknown | Same null and span check as 009f0690 (local stream begin/length/cursor/end/error). Calls 009ee770 on the record (two u16 header fields), then 009ee190 on recor… |
| `009f3940` | — | unknown | Same null and span check as 009f0690 (local stream: begin, length, cursor, end, error byte). Calls 009f3800 on the record and returns 1 when the error byte is … |

## Source shards

[[S0280 009e4570-009ea840|S0280]] (28), [[S0281 009ea8a0-009f0100|S0281]] (101), [[S0282 009f01f0-009f3940|S0282]] (23), [[S0329|S0329]] (4), [[S0330|S0330]] (1), [[S0398|S0398]] (25), [[S0414 00cbc8e0-00cc18f0|S0414]] (5), [[S0573|S0573]] (41), [[S0574|S0574]] (5), [[S0575|S0575]] (3)
