---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Housing

Part of [[Build Guides]] › Gameplay. Related: [[Items and Inventory]], [[Coin Store and Purchases]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`) are the native-map labels for a function. Source tags: *client* = read from the shipped client in this pass or in a shard; *vault* = an earlier research note, used only for facts it derived from the client; *video* = a reviewed frame record; *wiki* = the fan wiki. Precedence is client, then video, then wiki. *Reconstructed* (R) is our own server policy, not original behaviour. No original server code exists, so every statement about what the server did is a hypothesis built from what the client requires. OSFR-held data (packet captures and server JSON) is not used. No player names, chat or player-chosen lot names are recorded here. Several older vault notes derive field names, permission levels or lot tables from community emulator material (vault 023, 027 and 029 each cite a community packet model or table beside their client findings); this guide takes only their client-read facts, tagged as client below, and not those names, levels or tables (see Open questions). Related: [[Items and Inventory]], [[Coin Store and Purchases]], [[Social (Friends, Guilds, Chat)]], [[Zones and Terrain]], [[Quests and Objectives]], [[Pets]].

### Overview

Housing is the player-owned lot system. Each character owns one or more lots, each a separate private zone with its own build limits, that the player decorates with furniture items ("fixtures") from the inventory, then can lock, share, name and publish to a rated Housing Directory.

- **Lots and houses.** A "house" is a lot type: an Apartment (old starter), the Wilds Condo (starter from 2012-01-26), a Small Wilds House (free to members through the "Members Only New House Package"), a Large Wilds House and about twenty more regional lots and houses sold in the Station Cash Marketplace (wiki; video prices below). Farms (Wilds Farm, Briarwood Farm) appear in the same "My Houses" list and use the same item counter (video). Client strings name more deed-like entries (Small and Large Briarwood House, Large Snowhill House, Seaside House).
- **Getting into your house.** The house icon in the bottom panel (wiki) opens the "My Houses" window with a "Housing Directory" entry, the owned houses and a daily "Item Of The Day" panel with a BUY button (video). With several lots the player picks one.
- **Decorate mode.** Entering a lot shows a "Click to Decorate" or "Go To Decorate Mode" button (video, client strings "Enter Decorate Mode"). In decorate mode a bar at the bottom lists the owned housing items ("Visit the Marketplace or Coin Shop to get items!" when empty), an "Items/Limit" counter shows used and maximum fixtures, and a "Housing Options" menu holds Lock House, Snap to Grid, Auto Rotation, Free Camera, Hide Flora, Hide Pets and Pick Up All Items (client strings and video). Clicking a fixture opens a radial menu with Move, Rotate Left, Rotate Right and Remove (video). Placing on an invalid spot shows "That fixture can not be placed there." (client string, video).
- **Wallpaper and flooring.** Walls, floors, roofs and exteriors are customized by group and type: the player applies a texture and a tint to every fixture of a group and type (client Lua `GetFixtureGroupByType`, `ApplyFixtureCustomizationToGroup`; groups seen are Floor, Wall, Roof and Exterior). A tutorial quest hands out "Bold Light Spiral Wallpaper" for this (video, client string).
- **Name, lock, invite.** Houses get a three-part generated name or a purchased custom name; the owner can lock the house, invite players, decline invites and hide flora or roaming pets (client). Players can block house invites (client option "Block Housing Invite").
- **Directory and ratings.** The owner submits the home to the Housing Directory; others browse tabs (Search, Popular, New, Friends, Featured House), press Visit!, then cast a 1 to 5 star vote ("Select a Star to cast your vote!", video). Removing a house from the directory loses all votes (client string). A quest, "Visiting the Neighbors", sends the player to the directory (video; NPC Melanie Lane per client text).
- **Interactive furniture.** Some fixtures act when clicked: Mega Dance Floor, Rocket Ship, Jackpot Plant, launch pads, checkpoint blocks for home race tracks, music blocks, a Gumball Machine prompt (wiki, client strings, video). Fixtures carry named actions and an optional Lua call (client).
- **Rewards.** None recovered. Housing is a spend-and-display system; the only coin payout found is the Jackpot Plant's watering reward for visitors (wiki).

### How the client implements it

#### Main classes

| Class | Role | Evidence |
|---|---|---|
| `HousingProcessor` (vftable `01819584`, ctor `00abf850`, dtor `00ac2280`) | The one housing manager. Dispatcher `00aca5d0` (packet family 127), per-frame update `00ac4630`, edit-mode switch `00ab0a80`. Holds the process pointer (+0x10), "in house" flags (+0x14/+0x15), edit flag (+0x16), selection id (+0x40e0), edit state word (+0x4140), placement scratch (+0x4150..+0x417c), grid step (+0x4188), UI data-source pointers (+0x418c..+0x41a0), the loaded house object (+0x41b0), pending fixture-asset list (+0x4258), camera and orientation options (+0xeedc/+0xeedd) and a 16-bucket by-string fixture-group table (+0x28b0). | dispatcher confirmed; fields from pseudocode in S0315, S0316, S0321 |
| House instance object (ctor `0105eb60`, dtor body `0105e830`, clear `0105e280`) | The client copy of one lot: guids, owner, name, flags, counts, permissions, fixtures and a placement-box list (+0x108). It logs to `Housing.log` ("Desroying instance %lld with %d fixtures that was never saved to the db", "Removing %d fixtures ... from instance %lld"), which suggests this class is shared with server code. | ctor unknown label, dtor strings read (S0574) |
| `Fixture` and `HousePiece` (ctors `00cba340`, `00cb9dc0`/`00cb9f80`, vftables `018373b4`/`01837374` and `018372cc`) | World actors for one placed fixture. Two vftables per class (primary at +0, secondary at +0x50, adjustor thunks `00cba4d0`/`00cb9a50`). A fixture owns an occupant array and a per-character effect map (`HashListMap<CharacterGuid, Fixture::EffectStorageNode,32>` at +0x184, insert `00cba250`) and resolves its definition through slot 9 (`00cb91d0`). | ctors confirmed; many slot labels speculative |
| `FixtureDefinition` (dtor `00abd600`) | Definition record: ids, bool flags, Category and LuaCall strings, an action map `HashListMap<int,FixtureDefinition::FixtureAction,16,-1>` (+0xc8), a composite effect id, two floats and four flags. Reader `00abeb40`. | reader confirmed |
| `HouseActorEventHandler`, `HouseTileEventHandler`, `HousePlayerEventHandler` (ctors `00aae3e0`, `00aaad70`, `00aaac60`) | Click handlers on the `TargetProcessor` (ctor `00acb770`, ray/shape query target picker with 0x30-byte candidate records). Created together by `00ab0a80` when edit mode turns on and released when it turns off. `00ac9180` dispatches their events (kind 8 = place the held fixture). | confirmed |
| Housing data sources | UI tables the Lua panels bind to: `BaseClient.Housing.Instance` (`HousingDataSource`, ctor `00cbf9b0`), `.Fixtures`, `.SelectedFixture`, `.AvailableNames`, `HouseCustomNameList`, `HouseRatingKeywordsList`, `HouseSearchList`, `Housing.MyHouseList`, `PlayerHouseList`, `InstanceInfo`, `HouseRating`, `FeaturedRating`. Column lists are under Data and formats. | ctors and column labels confirmed |
| `FactoryPlot` and factory-state block (`01059780` AddFoundation, `01059ec0` PlaceObject, state `01058860`) | Farm and ranch foundation grid hung off the house instance (+0xd0). | likely (S0574) |
| Lua module `House` (registrar `00c0b410`, table `0182add0`, 53 entries) and the UI-side `Housing` object | The script surface; the handlers are thin wrappers over `HousingProcessor` senders. Full list under Network. | table decoded from the binary |

#### Entering, visiting and leaving

1. **Request.** Entering is a Lua call: `House.Enter(guid)` sends 127/19 (`00ac3fe0`, body `u64 house guid, i32, u8`, tunneled and gated on the client's connected flag). `House.EnterPreview(n)` sends the same packet with the two default guid globals and the integer (`00ac40b0`); the integer is probably a preview flag (vault inference, not proven). `House.ShowingHouses` sends the one-shot 127/23 grant request (`00ac7af0`, latch +0x2880). `House.RequestPlayerHouses(guid, text)` sends 127/14 (`00ac3f40`).
2. **Lists.** 127/39 (instance list, handler `00ac2a10`, reader `00abf210`: a count, the records, then the owner guid pair) is compared with the local guid. For the client's own list it fills "My Houses" and sends a rating `CandidateInfoRequest` in the Housing category; for another player's list it fills the visit picker and shows "That player does not have any houses you can visit at this time." when empty (client).
3. **Zoning.** The server moves the player with the ordinary scene change (a begin-zoning message to a housing scene; the client's asset catalogue names the empty-lot scenes `hsg_emptylot_*`). The client has no lot table: the lot's scene, spawn and build boxes are not in any shipped resource (see Open questions).
4. **Zone data.** 127/45 (`00ac36c0`, reader `00ab6c80`; two bools, a 32-bit head-size word, then the instance info) passes `(true, flag)` to Lua `Housing:SetIsInInstance` (`00ab94b0`), applies the size word through `CuteMode:SetSize` (`00ab96e0`), refreshes the housing data sources and sends the rating `CandidateInfoRequest` with the house guid (as decimal text) and the client guid. The client only passes this word to a "cute mode" size call, so what it scales and what value the server should send are not shown here ("head size" is a vault label, not a client one; value unknown). 127/50 changes it alone (`00abb050`, one u32, speculative label).
5. **Instance data.** 127/28 (`00aca330`, reader `00ac9dd0`) creates the client house object on first receipt; if one already exists it takes an error path and frees the new body. It reads three length-prefixed strings, guids, flags, counts, nested objects and a linked element list that gets a back-pointer to the house. **Every later fixture message is ignored while the house object is null**: 127/40 (`00ac8340`), 41 (`00abae70`), 42 (`00ac7f40`) all test the pointer. So the server must send 127/28 before any fixtures (client requirement).
6. **House info.** 127/44 (`00ab8ee0`) rewrites three flag bytes (+0x68..+0x6a), two counts and the furniture score on the house object and drives the edit mode through `00ab0a80` with its first byte. Edit mode is therefore likely server-confirmed: the client sends 127/6 to toggle, and the 127/44 that follows turns it on or off.
7. **Errors.** 127/46 (`00ab9080`, one u32) clears the busy flag (+0xbe1/+0xbe4 on the process object) and picks a message: 1 server not found (`00ab54b0`), 2 and 6 permission denied (`00ab5590`, "You do not have permission to enter that house."), 3 zone not found, 4 zone failed to launch, 5 zone full ("visitor limit"). The same pair of process fields is set by a received 127/19 (which reads no body) and by the frame update after it sends 127/20, so the pair is a request-in-progress indicator (purpose guessed).
8. **Invites.** The owner calls `House.InviteToHouse(guid)` (127/15 `00ac4cd0`, then the local text "House invite sent to player."). The target receives 127/47 (`00abd510`, dispatched through `00a0bdb0`) and a prompt "[Player] has invited you to enter their house. Do you want to accept?" (client string, video). Accepting is an ordinary enter; declining sends 127/16 (`00ac4e00`, one u64) and the inviter gets 127/48 (`00abaca0`: two u64 pairs, Lua `HousingInvitationDeclined` with key `invited_player`, text "has declined your house invitation").
9. **Neighbors.** `House.VisitRandomNeighbor` sends 127/21 with the client's own guid (`00ac7a50`); the server chooses a house. A directory row's Visit! does an ordinary enter with the row's house guid.
10. **Leaving.** `House.LeaveHouse` sends 127/10 (`00ac4b80`, empty). The return trip is a normal scene change.
11. **New house.** 127/49 (`00ab91b0`, one string) calls Lua `HousingNotifyNewHouse` or prints "You have received a new %s." The 127/23 request is empty, so the server decides which property to grant from its own state.
12. **Browser.** 127/54 calls Lua `HousingBrowserHandler:Show` (inline in the dispatcher; no body). 127/57 carries a server-version string that the client prints as "HousingServer Version: %s" (`00ab7f50`).

#### Decorate mode and placement

The sequence below is assembled from the sender and handler chains; the order of states is likely, not step-traced in a live client.

1. **Edit mode on.** `House.ToggleEditMode` sends 127/6 (`00ac4160`, `u8`, the inverse of the current flag). On the confirming 127/44, `00ab0a80` stores the flag, reads the two options (`UseFreeFlyCamera` `00aaf090`, `UseAutoOrientation` `00aaf1b0`) and builds the three event handlers. Outside edit mode, selection helpers say "not in housing edit mode" (`00aa5f30`).
2. **Pick an item and ask for its asset.** The decorate bar's Lua calls `House.PlaceFixture(...)` (one string argument); the client sends 127/1 (`00ac4200` through writer `00abcb90`, three dwords and a byte). The server answers 127/42 (`00ac7f40`, reader `00ac1f50`): the echoed selection id, then the fixture definition, model id and item definition id (client reader `00ac1f50`; the order inside the definition block is not traced here). The client keeps the record in a list (+0x4258) unless a record with the same key exists, resolves the model (`FixtureActorErrors.log`: "Failed to find ModelDefinition for modelId=%d.") and creates the 0x230-byte preview actor (`00aaae80`, held at +0x40bc). In the branch that runs when a saved transform is present (after a pick-up, see step 5), the same handler instead builds a `Fixture` actor (`00cba340`) and sends 127/2 itself with the saved transform, so a 127/42 can trigger the commit with no click (likely; the label only proves the send path). `House.EnterPreview` and `127/20` (item-definition preview, `00ac0a80`, queued at +0x2884 and sent from `00ac4630`) are a separate preview-by-item path.
3. **Position the preview, per frame.** `00abb1c0` (surface target) and `00aae6e0` (ray-test the preview against the surface) place it; `00ab0f20` and `00ab1380` run convex-hit queries against the world and test candidate transforms; `00aaed30` is a transformed-bounds test shared with player-position handling. Snap: `00aab720` quantizes only x and z with `(int)(step*v)*(1/step)` using the step at +0x4188 (Lua `SetGridSpacingInCentimeters` computes `100/cm`; the client Lua defines exactly two spacings, 25 and 5 cm, as `GRID_SIZE_ON` and `GRID_SIZE_OFF`; client Lua, read by vault 027), optionally rotated through the surface matrix when auto-orientation is on (+0xeedd); `00aab3b0` orients the preview to the surface normal. Rotation steps are 15 degrees (action 6 counter-clockwise, 7 clockwise; shared transform `00ab1e30`); a second constant of 90 degrees is chosen by a per-record flag at +0x128 whose meaning is unknown. The preview carries a validity byte (+0x165); when it is clear the client shows `CantPlaceFixtureHere` / "That fixture can not be placed there." and sends nothing. The client never holds a numeric placement limit.
4. **Commit.** The click (event kind 8) reaches `00ac8940`, which copies selection id, position, orientation and scale into the scratch block and sends 127/2 (`00abffb0`, writer `00abc9a0`: a u32 selection id, two 16-byte groups, a final dword). The server replies 127/40 (fixture record) and 127/44 (updated counts) (our reconstructed pairing; R). A debug branch logs "Creating a duplicate" and uses the 127/1 path.
5. **Select, move, rotate.** Selecting a placed fixture runs `00ab5910` and updates `...Housing.Instance.SelectedFixture` (one column, Fixture Guid; -1 means none). The radial menu asks `House.IsActionPermitted(guid, name)` (`00ab5af0`); the shipped action names are `MoveFixture`, `RemoveFixture` and `Elevate` (config keys `Housing.*FixtureActionName`; `Elevate` has no call site). `House.MoveFixture` runs the mode-1 callback (`00ab0d60`) and `House.MoveFixtureVertically` the mode-2 callback (`00ab2250`). The edit-state word at +0x4140 drives `00ac8d90`: state 1 swaps the saved and live transform and calls the save routine; state 2 sends the pickup (127/3) and moves to state 3; state 3 re-sends 127/1 once the old fixture no longer resolves. Save (`House.SaveFixture`, `00ac8790`) sends 127/5 (`00ac7ce0`, writer `00abf620`: a u64 fixture guid, two 16-byte groups, a dword, a nested block of strings and a final dword). `House.UndoLastAction` also enters `00ac8d90`; `CancelFixturePlacement` commits the preview back.
6. **Pick up.** `House.PickupFixture(guid)` sends 127/3 (`00ac42d0`, u64) and the server answers 127/41 (`00abae70`, u64 guid): the client drops the node, decrements a by-key count and clears the selection if it matches. `House.PickupAllFixtures` sends the empty 127/4 (`00ac8680`) and, when the lot was published, a rating remove-candidate (the client warns "Homes submitted to the Housing Directory will be removed when all items are picked up").
7. **Fixture moves reported by the server.** 127/51 (`00ab8dc0`, "UpdateFixturePosition") finds the world object and applies a transform (which request the server answers with it is unknown; R).
8. **Item list.** 127/43 (`00ac9bd0`, reader `00ac9a90`) fills the owned-fixture list (a guid plus the item definition) the decorate bar draws from. It stops an active preview first.
9. **Quest hook.** The client has an objective type "housing fixture move" (objective id 0xc, `ClientObjectiveHousingFixtureMove`: a fixture id and an action) used by the "move blocks" tutorial (vault 029; text "Redecorate your Wilds Condo by moving some of the blocks it came with").

#### Customization, naming and options

- **Group customization.** `House.GetFixtureGroupByType(type)` looks the group up in the table at +0x28b0 (`00aab1a0`); `DoesGroupHaveCustomizableFixture` is `00aae6b0`. `House.PreviewTextureAlias(a)` and `PreviewTintAlias(a)` retexture a preview locally (`00aa5f70`); `ApplyFixtureCustomizationToGroup(group, type, id)` sends 127/59 (`00ac0740`: u32 plus two strings) and `RemoveFixtureCustomizationFromGroup` sends 127/60 (`00ac0850`, two strings). `FixtureDataSource` exposes Fixture Guid, Texture Alias, Tint Alias. The fixture record itself carries a customization block with the same two aliases and a tint id (client reader; field order inside it not traced here).
- **Naming.** Default names are built from three parts. `HouseNames.txt` (364 lines) has 142 first-part glossary ids, 142 second-part, 78 third-part and one `T4LIST_HOUSE_NAME_FORMAT` row (client resource). `House.SetName(i,j,k)` sends 127/17 from the selected entries (`00ac4e90`); `House.SetHouseName(id)` sends 127/9 (`00ac4c40`, one u32); `House.SetNameCustom(text)` sends 127/17 with a string and a byte (`00ac51b0`, writer `00abcf40`). The reply (the same number 0x11, `00aca5d0`, reader `00ab7fc0`) either shows `HouseNameRejected` or updates the house and the saved `Houses` thumbnails (`%s_%llu_1/2/3`). The client sells a permanent custom name ("House Name Change", "Buy Name"; client strings). The wiki says the name filter can reject names.
- **Flags.** `ToggleIsHouseLocked` 127/11, `ToggleFloraAllowed` 127/12, `TogglePetAutospawn` 127/13 (all empty), `LockHouse(guid, bool)` 127/22 (u64 + u8), `PayUpKeep` 127/7 (empty). No upkeep text exists in the client locale or the wiki; the packet may be vestigial.
- **Local options.** Free fly camera, auto orientation, snap-to-grid spacing and Hide Flora and Hide Pets are user options stored on the client (`Housing.UseFreeFlyCamera`, `Housing.UseAutoOrientation`).

#### Directory, ratings and reports

The directory is a separate rating service, not part of family 127. Housing Lua calls `RatingDataRequestGeneral/HighScore/Friends/New` go through `00ac9340`, which resolves the interned key `RatingSystem.HousingId` (default text `Housing`) and sends a rating request wrapped as a world-tunneled packet. `RatingFeaturedRequest` (174/20, `00ac62e0`), `Publish` (174/6 AddCandidate with seven strings, `00ac5250`, which also follows with the lock toggle 127/11), `Unpublish` (174/7, `00ac5b90`), `RatingVote` (174/8, `00ac6a00`), `SearchRating` (174/12, `00ac7260`, throttled by an elapsed-time test against a default of 2500) and the per-house `CandidateInfoRequest` (174/16, sent on entering a house) are the six requests. `ReportOffensive` sends 68/3 (`00ac71a0`, two ids and two strings). `SetDescription` and `SetRatingKeyword` edit local state (`HouseRatingKeywords.txt` ships empty). The replies are consumed by data sources (`HouseRatingDataSource`, `FeaturedDataSource`, `HouseSearchDataSource`) but their packet classes are not mapped. The client considers a house "published" through a candidate record; wiki and client agree that a submitted house cannot be edited until removed (wiki) and that removal discards votes (client string).

#### Farms (adjacent)

Farm and ranch lots reuse the housing instance. `FactoryPlots.txt` (8 rows, client resource) lists `FARMING` or `RANCHING` plots by housing instance id with a grid size (2.0, 0.5 or 1.0 m), an X/Z rectangle, a misfortune check time, a maximum misfortune count and a percent. The Wilds Farm HUD shows the same "Items/Limit" counter (9/18) with a Tool Shed, Watering Can and "Go To Planning Mode" (video). Crop mechanics are outside this guide.

### Data and formats

**Client resources (read by the client, not sent).**

| File | Contents | Source |
|---|---|---|
| `HouseNames.txt` | `GLOSSARY_ID^PART_ID^`: 142 first, 142 second, 78 third parts and one format row. Loader `00abd680` (reads `%sHouseNames.txt`, skips `#`, splits a bracketed group token) and start-up `00abdb20`. | client; glossary text not resolved here |
| `HousingNames.txt` | Two string ids (419500, 419501). Loader `00ab41a0`. | client |
| `HouseRatingKeywords.txt` | Header only. Loader `00ab5f10`, flag editor `00ab6110`. | client |
| `HousingPlacementData.txt`, `HousingPlacementAssociations.txt` | Two developer rows (`test.adr`, `test2.adr`) and one parent/child pair. | client |
| `FactoryPlots.txt` | 8 farm and ranch plot rows (see Farms). | client |
| `StringHashToValue.txt` | `Housing.MoveFixtureActionName^MoveFixture`, `Housing.RemoveFixtureActionName^RemoveFixture`, `Housing.ElevateFixtureActionName^Elevate`, `Housing.StaticSeatingDisabled^1`, `RatingSystem.HousingId^Housing` (vault 029). | client |
| `Requirements.txt` | 43 requirement rows (six factory names) that a `FixtureAction.RequirementId` points at (vault 029). | client |

The client ships **no** lot, zone or build-box table; a fixture's model comes from the asset system by model id (`hsg_*` model names, `hsg_emptylot_*` scenes).

**What the client stores for one lot (UI column lists, confirmed).** `HousingDataSource` (18 columns): Instance Guid, Owner Guid, Owner Name, Permission Level, In Edit Mode, Max Fixture Count, Cur Fixture Count, Max Landmark Count, Cur Landmark Count, House Name, Is Locked, Is Outside, Can Edit, Icon, FurnitureScore, Is Members Only, Is Flora Allowed, Pet Autospawn. `HousingInstanceDataSource` (My House, Owner and Player lists, 18 columns): Instance Guid, Owner Guid, Owner Name, House Name, Icon Id, FixtureCount, LastVisited, FurnitureScore, IsLocked, Is Members Only, Description, Keyword List, Rating, Votes, HasRating, CanVote, WhenCreated, IsFloraAllowed. `HouseSearchDataSource`: HouseGuid, OwnerName, HouseName, OwnerGuid, Snapshot, Rating, NumVotes. `FeaturedDataSource` adds a System column. `FixtureItemDataSource`: 41 item columns plus FixtureGuid and Location. Max and Cur Landmark counts exist, but nothing recovered says what a landmark is (a "Landmark" label appears in one video frame).

**Wire bodies (client readers and writers only; subtype `u16` included in the sizes below).** No body here comes from a recorded original session.

| Subtype | Size | Layout |
|---|---|---|
| 127/2 PlaceFixture (C>S) | 42 | `u32 selection id`, two 16-byte groups (likely position and orientation, per the commit routine `00ac8940`), a final dword (likely scale). Client writer `00abc9a0` (u32, two 16-byte groups, a dword). Meaning of the components of each group (for example the fourth float) is not traced. |
| 127/40 FixtureUpdate (S>C) | unknown | A fixture record (read by `00ab6f40`, including a customization block with texture and tint aliases), a fixture-instance block (read by `00aafd30`/`00aafa90`, with the item definition id), a fixture definition (`00abeb40`, layout under Fixture definition below) and a guid-array count; chain entered at `00ac2180`. Per-field widths and total length are not traced from the client in this pass. |
| 127/41 FixtureRemove (S>C) | 10 | `u64 fixture guid`. |
| 127/44 HouseInfo (S>C) | 22 | `u8` edit mode, `i32` (the handler does not store it; value unknown), three flag bytes (locked, flora allowed, pet autospawn), three `i32` (current fixtures, current landmarks, furniture score); from handler `00ab8ee0` (client). |
| 127/45 ZoneData (S>C) | unknown | two bools (the second goes to `Housing:SetIsInInstance`), an `i32` size word (value unknown), then the instance info (reader `00ab6c80`). Total length and any trailing fields are not traced. |
| 127/53 PlacementDenied (S>C) | variable | one length-prefixed string; empty falls back to the local `HousingMaxFixtures` text. |
| 127/19 EnterRequest (C>S) | 17 w/ framing | `u64 house guid`, `i32`, `u8`. |

All C>S housing bodies are written by `00aba910`: base opcode 127 as `u16`, then the inner opcode as `u16` (so datagrams start `7f 00 <inner>`), inside the tunneled client packet (`0089adc0`).

**Fixture identity.** The fixture guid is minted by the server (R: the client's 127/2 carries no fixture guid, so the server must mint unique guids). The 127/2 selection id is the client placement object's key (`+0x40` of the object returned by the two-word lookup `0105bfa0`), shared by 127/1, 127/2 and the 127/42 echo; nothing recovered shows it is the item definition id. The authoritative ids arrive next to the fixture record (fixture definition id, item definition id and model id).

**Fixture definition.** Four ids, six bools, Category and LuaCall strings, an action dictionary (`count`, then `u32 key` and a value of `i32, string, i32, i32` per entry), a composite effect id, two floats and four flags (`00abeb40`). Actions have a name and a requirement id; the interaction names `RemoveFixture`, `MoveFixture`, `Elevate` are the only ones in the config.

**Permissions.** The instance permission map is `HashListMap<int,InstancePermission>` (client RTTI), and the house-browser row exposes the columns Permission Level and Can Edit (client; vault 023 read them from the pseudocode). Which level number means "can edit", and what other levels mean, is not shown by the client; an earlier vault claim that level 3 is "can edit" came from a community packet model and is not used.

### Network

All traffic is the housing family, base opcode 127 (`BaseHousingPacket`, constructor `00aa4930`, inner constructor `00aa4990`), tunneled both ways. Receive dispatch is `00aca5d0`. Directions come from the sender wrappers (C>S proven when a wrapper calls the writer) and from the dispatcher cases (S>C proven). The packet catalogue lacks most of these classes, so the map's `wire` field is filled for only about a quarter of the functions.

| Sub | Name | Dir | Body | Handler or sender |
|---|---|---|---|---|
| 1 | PlaceFixtureRequest | C>S | `u32, u32, u32, u8` | `00ac4200`, wrapper `00ac0400` |
| 2 | PlaceFixture | C>S | `u32`, 2 x 16 B, `u32` | `00ac8940`, wrapper `00abffb0` |
| 3 | PickupFixture | C>S | `u64` | `00ac42d0`, wrapper `00ac0510` |
| 4 | PickupAllFixtures | C>S | empty | `00ac8680`, wrapper `00ac0630` |
| 5 | SaveFixture | C>S | guid, transform groups, nested block | `00ac8790`, `00ac7ce0` |
| 6 | SetEditMode | C>S | `u8` | `00ac4160` |
| 7 | PayUpkeep | C>S | empty | `00ac45b0` |
| 9 | ChangeHouseName | C>S | `u32` | `00ac4c40` |
| 10 | LeaveHouse | C>S | empty | `00ac4b80` |
| 11 | ToggleLocked | C>S | empty | `00ac4940` (also after a directory publish) |
| 12 | ToggleFloraAllowed | C>S | empty | `00ac4a70` |
| 13 | TogglePetAutospawn | C>S | empty | `00ac4b00` |
| 14 | RequestPlayerHouses | C>S | guid pair + string | `00ac3f40` |
| 15 | InvitePlayer | C>S | `u64` | `00ac4cd0` |
| 16 | DeclineInvite | C>S | `u64` | `00ac4e00` |
| 17 | RequestName | C>S, reply S>C | string + `u8`; reply flag + name | `00ac51b0`, `00ac4e90`; reply in dispatcher |
| 19 | EnterRequest | C>S; S>C sets a flag only | `u64, i32, u8` | `00ac3fe0`, `00ac40b0` |
| 20 | PreviewByItemDefinitionId | C>S | `u32` | `00ac4630` queue, wrapper `00ac0a80` |
| 21 | RequestVisitToNeighbor | C>S | own guid pair | `00ac7a50` |
| 22 | LockHouse | C>S | `u64, u8` | `00ac49d0` |
| 23 | RequestGrant | C>S | empty | `00ac7af0` |
| 28 | InstanceData | S>C | house record | `00aca330`, `00ac9dd0` |
| 39 | InstanceList | S>C | count, records, owner pair | `00ac2a10`, `00abf210` |
| 40 | FixtureUpdate | S>C | see Data | `00ac8340`, `00ac2180` |
| 41 | FixtureRemove | S>C | `u64` | `00abae70` |
| 42 | FixtureAsset | S>C | selection echo, definition block, guid array | `00ac7f40`, `00ac1f50` |
| 43 | FixtureItemList | S>C | counted guid list | `00ac9bd0`, `00ac9a90` |
| 44 | UpdateHouseInfo | S>C | see Data | `00ab8ee0` |
| 45 | ZoneData | S>C | see Data | `00ac36c0` |
| 46 | HousingError | S>C | `u32` code | `00ab9080` |
| 47 | InviteNotification | S>C | inviter ids | `00abd510` |
| 48 | DeclineNotification | S>C | two `u64` pairs | `00abaca0` |
| 49 | NotifyHouseAdded | S>C | string | `00ab91b0` |
| 50 | HeadSize | S>C | `u32` | `00abb050` (speculative) |
| 51 | UpdateFixturePosition | S>C | guid + transform | `00ab8dc0` |
| 53 | FixturePlacementDenied | S>C | string | `00ab9860` |
| 54 | ShowHousingBrowser | S>C | none | dispatcher, Lua `HousingBrowserHandler:Show` |
| 55 | Explosion | S>C | record list | `00ab99a0` (likely) |
| 56 | ClearExplosions | S>C | none | `00ab9a90` (speculative) |
| 57 | ServerVersion | S>C | string | dispatcher, `00ab7f50` |
| 59 | ApplyCustomization | C>S | `u32` + 2 strings | wrapper `00ac0740` |
| 60 | RemoveCustomization | C>S | 2 strings | wrapper `00ac0850` |
| (n/a) | ExplosionResetRequest | C>S | empty | wrapper `00ac1e40`, subtype not read in this pass |

Other families used by housing: rating base 174 (6, 7, 8, 12, 16, 20; replies unmapped), report base 68 (sub 3), the begin-zoning scene change.

**Lua `House` surface (53 entries, decoded from the module table; grouped).** Entry: `Enter`, `EnterPreview`, `RequestPlayerHouses`, `ShowingHouses`, `VisitRandomNeighbor`, `LeaveHouse`, `GetCurrentHouseInstanceGuid`, `InviteToHouse`. Edit: `ToggleEditMode`, `PlaceFixture`, `PlacePendingFixture`, `CancelFixturePlacement`, `MoveFixture`, `MoveFixtureVertically`, `RotateFixtureClockwise`, `RotateFixtureCounterClockwise`, `SaveFixture`, `UndoLastAction`, `PickupFixture`, `PickupAllFixtures`, `IsActionPermitted`, `IsFixtureMembersOnly`, `SetGridSpacingInCentimeters`, `Set/GetUseFreeFlyCamera`, `Set/GetUseAutoOrientation`. Customization: `GetFixtureGroupByType`, `DoesGroupHaveCustomizableFixture`, `PreviewTextureAlias`, `PreviewTintAlias`, `ApplyFixtureCustomizationToGroup`, `RemoveFixtureCustomizationFromGroup`. Name and flags: `SetHouseName`, `SetName`, `SetNameCustom`, `ToggleIsHouseLocked`, `LockHouse`, `ToggleFloraAllowed`, `TogglePetAutospawn`, `PayUpKeep`. Directory: `SetRatingKeyword`, `SetDescription`, `Publish`, `Unpublish`, `RatingDataRequestGeneral`, `RatingDataRequestHighScore`, `RatingDataRequestFriends`, `RatingDataRequestNew`, `RatingFeaturedRequest`, `RatingVote`, `ReportOffensive`, `SearchRating`. Handlers in `00c0b3c0..00c0c530`; most are table-bound thunks (the senders above), so only 19 are mapped functions.

**Entry and placement flow the client expects (client senders and handlers; the reply pairings are our reconstruction, R).**

1. C>S 127/23 (grant request) and 127/19 (enter, with the chosen house guid).
2. S>C begin-zoning to a housing scene, then 127/45, 127/28 and 127/44 (R for the exact order beyond "127/28 before any fixture message").
3. Per placement: C>S 127/1, answered by S>C 127/42; C>S 127/2, answered by S>C 127/40 and 127/44; C>S 127/3, answered by 127/41; C>S 127/5, answered by 127/51 (R for each pairing; the client only shows which messages it can receive). Which replies the original sent, and whether it ever sent 127/53, is unknown.

**What is known.** Every subtype's sender or handler, the exact reader order for 40/41/42/44/45/46/48/49/53, the framing, the Lua surface, the column lists, and the full set of client-side placement checks. **What is not known.** The original server's placement validation, the fixture-count and landmark limits per lot, the furniture score formula, upkeep (if any), directory ranking, reply classes of the rating service, the 127/28 and 127/39 byte layouts field by field, invite and lock rules, and the identity of the 127/2 selection id's source table.

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour.

1. **Persist lots.** Per character, any number of houses: house guid, owner guid, lot type (scene), name id or custom name, icon, flags (locked, flora allowed, pet autospawn, members only, outside), counts and caps, permission map (the level numbering is ours, R; the original numbering is unknown), description, keywords, rating state, created and last-visited times, and the fixture list with the full record contents the client reads in 127/40. Instances are per owner; a visit shares the owner's instance (R for the empty-zone eviction policy).
2. **Content tables the client does not have.** Lot list (scene, spawn point, build boxes, indoor or outdoor), deed items and prices, fixture definitions (id, item definition, model id, category, Lua call, actions, composite effect), and the item-to-fixture mapping. Our own scene and terrain data supplies boxes; the community lot table is excluded from this project.
3. **Grants.** Answer 127/23 and 127/14 with a 127/39 list. On a purchase or grant send 127/49 (string with the house type text) and a fresh 127/39. Starter lot, member lot and card-code lots follow the wiki channels (R for the mechanics).
4. **Enter flow.** On 127/19: resolve the house, check lock, permission, member-only and visitor cap, and refuse with 127/46 codes (1 server, 2 or 6 permission, 3 zone missing, 4 launch failure, 5 full). On success: send the scene change, then 127/45, **127/28 (before any 127/4x)**, 127/44, the owned-item list 127/43 and the existing fixtures in the order the client reads them. Also the rating candidate info reply (unmapped).
5. **Edit mode.** On 127/6 verify ownership or an edit permission under our own numbering (R), set the instance flag and answer 127/44 with the new mode. The client does not enter edit mode without it.
6. **Placement.** 127/1: resolve the selection against the player's owned items, answer 127/42 with model id, fixture definition and item definition. 127/2: validate owner or editor, caps (`HousingMaxFixtures` via 127/53 on refusal), build box (R, the client does the collision test but the server must still bound), consume one unit of the item, mint a unique fixture guid, store the transform, reply 127/40 with the full record and 127/44 with the new counts. Do not trust the client's validity byte.
7. **Edit operations.** 127/3: return the item, send 127/41 and 127/44. 127/4: return all items, remove fixtures, remove the directory entry if published. 127/5: validate and store the new transform, reply 127/51 and 127/44. 127/59 and 127/60: store or clear group and type texture and tint aliases and broadcast the effect to everyone in the instance (R for broadcast).
8. **Flags and names.** 127/9, 127/17: reserve names, run the filter, reply with the accept or reject form of 0x11. 127/11, 127/12, 127/13, 127/22: toggle and answer 127/44. Treat 127/7 as a no-op until upkeep evidence appears (R).
9. **Social.** 127/15: route 127/47 to the target unless they set Block Housing Invite; 127/16: send 127/48 to the inviter; 127/21: choose a house a stranger may visit (R). Respect locks on direct visits.
10. **Directory and ratings.** Implement the rating service for 174/6, 7, 8, 12, 16, 20 and the report 68/3: one rating per account per house, 1 to 5 stars, popular, new, friends and featured lists, a search with a minimum interval, vote counts, removal on unpublish or pick-up-all. Reply layouts are unrecovered; the data-source columns (Rating, Votes, HasRating, CanVote, Snapshot, Owner Name, House Name) define what must be supplied. Snapshot thumbnails use the saved `Houses` entries `%s_%llu_1/2/3` (R).
11. **Interactive furniture and objectives.** Run fixture actions (named actions with requirement checks, Lua calls, per-character effect state, seats), the Jackpot Plant's 30-day cycle (water every 15 minutes, visitor reward 50 coins; wiki), and emit the "housing fixture move" objective when a tutorial fixture is moved.
12. **Farms.** Serve foundation plots from `FactoryPlots` data and keep the same counters (adjacent system).
13. **Pets and head size.** Honour pet autospawn, dismiss followers on entry (video line "Your pet can't follow you, and has been dismissed."), send the size word in 127/45 (value unknown; choose by testing against the client, R).
14. **Errors and logging.** Every refusal must be one of the 127/46 codes or a 127/53 string; never silent.

### Video and wiki evidence

At least 121 frame records tagged as housing across 32 videos, plus store and inventory frames. House and lot names players chose are not recorded.

- **Lot types seen in banners or lists.** Apartment (room with a bed and wooden floor, "Go To Decorate Mode"), Wilds Condo, Small Wilds House, Snowhill Lodge, Sunstone Dam, Lonely Island Lot, Wilds Farm (video, banners equal to the type). The My Houses window lists the Wilds Condo, a Small Wilds House, the Wilds Farm and several houses with composed names.
- **Prices in SC (video, with the wiki regular and member price).**

| Lot | Video | Wiki |
|---|---|---|
| Snowhill Lodge | 855 (Marketplace list, 3 frames) | 950 / 855 |
| Large Wilds House | 375 (Item Of The Day), 750 also shown in red in one frame | 750 / 675 |
| Briarwall Towers Lot | 250 (Item Of The Day, 4 frames), 500 in a Marketplace ad beside a 650 Bounce Castle | 500 / 450 |
| Vale Stream Lot | 200 (Item Of The Day) with a 400 figure beside it | 400 / 360 |

  The Item Of The Day figures are consistently half of the wiki regular price, so the daily panel looks like a 50 percent sale; the 855 reading is the wiki member price, so the recording player probably held membership. Other wiki prices (regular / member): Blackspore Swamp Lot 300/270, Blackspore Swamp House 950/855, Bog Shore Lot 400/360, Briar Falls Lot 500/450, Briarwood Lot 300/270, Club House 1499/1349, Crystal Mines Lot 400/360, Lake Tree Lot 500/450, Lonely Island Lot 499/449, Merry Vale Lot 300/270, Sandy Beach Lot 400/360, Seaside Lot 300/270, Shrouded Glade Lot 300/270, Shrouded Stairs Lot 400/360, Small Wilds House 549/494, Snowhill Lot 300/270, Sunstone Dam 1499/1349, Wugachug Lot 300/270. Members get a 10 percent discount (wiki; matches the reading).
- **Other acquisition channels (wiki).** Apartment: formerly every new player; Wilds Condo replaced it on 2012-01-26 and existing Apartment owners also received a Condo. Small Wilds House: free for members via the Members Only New House Package. Trading Card Game loot cards: Sunstone Valley Lot (series 3), Shrouded Gloam Lot (series 4), Rumbledome Lot (series 5), Bat Cavern Lot (Treasure War; a duplicate claim pays 2500 coins, per the card text). Snow Days Lot: 12 Days of Presents 2012, no longer obtainable. Sunset Party Boat: February 2014 membership pack. Lake Tree Lot: first handed out in a 2011 Fan Faire pack, then sold from 2012-04-05. Club House (VIP rank 5) and Briar Falls Lot (VIP rank 3) were VIP-only until 2013-05-22. Sunstone Dam bundles a Giant Wood Builders Pack. Farms: Wilds Farm free, Briarwood Farm in the Marketplace. Never released: Bixie Field Lot, Girl House, a golden yacht lot.
- **Sizes (wiki, in blocks).** Apartment 4 x 9.5 x 10.5 (volume 399); Small Wilds House 39 x 47 x 36 (65,988); Large Wilds House 40 x 62 x 75 (186,000).
- **Item limits (video).** Items/Limit shows 613/9999, 615/9999, 18/9999, 24/9999, 2436/9999 and 26 or 27/3000 (two frames, house type not identified); farm 9/18. The client says the cap is supplied by the server per lot (Max Fixture Count column) and prints "Max housing fixture count reached." at the cap.
- **Decorate UI (video).** "Click to Decorate" with a fixture list at the bottom, "Housing Options" and Lock House and Snap to Grid entries, radial menu Move, Rotate Left, Rotate Right, Remove, a red block shown when a placement is invalid with "That fixture can not be placed there.", placement prompts ("Press X or ..."), and "Submit your home to the Housing Directory." beside a Submit button and later "Your home has been submitted." with a Remove button.
- **Directory (video).** Window titled Housing Directory with Search, Popular, New, Friends, Results tabs, a Featured House strip with vote counts, Visit! and More! buttons, rows with a snapshot and a vote count; the vote panel inside a house reads "Select a Star to cast your vote!" with Vote. Wiki: members submit; sorted by highest rated, most viewed or newest; 1 to 5 stars; removal resets rating and views to 0; no edits while listed.
- **Furniture names (video, wiki).** Video: Standard Table Lamp, Standard Floor Lamp, Sylvan Wall Lamp, Birthday Candle (in a decorate list), colored blocks (green, red, yellow), a Gumball Machine prompt. Client text: Chicken, Basic Square Block, Basic Flat Square Block, and block names with polka dot and tiger patterns. Marketplace paintings at 180 or 225 SC (Creepy Laughter, Frostfang Cub, From the Trash Bin, Ernie). Wiki: Mega Dance Floor 600 SC (plays music, makes nearby players and pets dance), Rocket Ship 1000/900, Juice Bar 600/540, Basic Swimming Pool 799/719, Love Struck Piano 299/269, Fog Machine 9,999/8,999 coins, launch pads in two sizes, Blocks (SC or coins). The wiki Furniture category lists about 120 pages, paintings, seasonal sets and Spooky items. Jackpot Plant: a small tree that waters for 30 days, cannot be returned to the inventory once placed (wiki and client text), friends watering it earn 50 coins every 15 minutes.
- **Quests and text (video, client).** Wilds Condo tutorial: place a Chicken, move a Basic Square Block with the radial menu, apply Bold Light Spiral Wallpaper ("Bold Customizations"); "Visiting the Neighbors" (Melanie Lane, click the Housing Directory, visit a house). Invites: "[Player] has invited you to enter their house. Do you want to accept?" and "House invite sent to player."

### Open questions and contradictions

- **No original server code.** All server-side placement, limit and directory behaviour is inferred.
- **127/2 direction.** An older vault note (Housing Restoration) calls 127/2 a server response; the client constructor `00aa53f0` and the sender `00abffb0` show it is C>S. The shard findings carry the correction.
- **127/2 first dword.** The key of the client placement object (`0105bfa0` two-word lookup), echoed by 127/42; its owning table is unidentified (nothing recovered shows it is an item id).
- **127/28 ordering.** The client ignores 40/41/42/43 until 127/28 builds the house; where in the entry sequence the original sent 127/28 is unknown.
- **Unmapped bodies.** 127/28, 39, 42, 43, 47, 48 (partly), 49 fields beyond the string, 50, 51, 55, 56, 57, 59, 60 byte layouts; 127/45 total length and trailing fields; the first `i32` of 127/19; the byte widths and total length of 127/40 and 127/42 (no field-level layout is verified).
- **127/53 and limits.** No numeric cap is attached to `HousingMaxFixtures`. Video shows 9999, 3000 and 18; the 3000 frames do not name the house type.
- **Landmarks and furniture score.** Max and Cur Landmark counts and a FurnitureScore exist on every lot; their rules are not recovered.
- **Upkeep.** `PayUpKeep` (127/7) is a real packet with no text anywhere in the locale or wiki.
- **Edit state machine.** The meaning of edit states 1 to 3, the mode-2 "move vertically" path and the +0x128 per-record flag (90 degree step) are inferred; the Elevate action has no call site.
- **Explosions.** 127/55 and 127/56 and `ExplodingFixture` records (0x30 B arrays) are probably trap or popper furniture; unconfirmed.
- **Permission levels.** The client names a Permission Level column and a Can Edit column but gives no level numbers, so the original level scheme is unknown and any server value is our own policy (R).
- **Lot table.** The client has no lot, spawn or build-box data. The vault holds a community lot table that this guide does not use; our server needs boxes derived from the client's own scenes. The complete list of lot types cannot be tied to the client alone; the wiki list (about 30 lots) is the only roster.
- **Store prices.** Video Item Of The Day figures are half of the wiki price; whether the sale was always 50 percent or a one-off is unknown. Some wiki pages carry prices only in template fields.
- **Rating service.** Reply classes are unmapped; `RatingSystem.HousingId` shows the service was keyed by system name. A throttle constant of 2500 is read from default data (units unresolved).
- **Naming.** Glossary ids in `HouseNames.txt` are not resolved here; the wiki says the filter can refuse names; how custom names are priced is unknown.
- **Speculative labels in the map.** Adjustor and tail-call thunks on `HousePiece`/`Fixture`, `00cb9aa0`/`00cb98f0` (unknown), `HouseRating`/`Featured` row-count and cleanup helpers, `00ac9140` (save wrapper), `00ab3...` record readers that are mechanical only, and the `00ab73e0` explosion handler body.
- **Vault note corrections to carry forward.** `00aba9e0` is a reader (an older note called it an emitter); `00ac9bd0` does not destroy actor nodes; `00acb920` removes owner matches (not mismatches); `00ac8340` is not an overflow gate; 127/57 is S>C (dispatcher case 0x39).

### Key functions

| Address | Role |
|---|---|
| `00aca5d0` | `HousingProcessor::DispatchMessage`, the 127-family receive switch (confirmed) |
| `00ac4630` | `HousingFrameUpdate`: queued 127/20, edit-mode per-frame work (likely) |
| `00ab0a80` | Edit-mode switch; builds and frees the three event handlers (unknown label) |
| `00aca330`, `00ac9dd0` | 127/28 instance data: creates the house object, parses the record |
| `00ac2a10`, `00abf210` | 127/39 instance list handler and reader (confirmed) |
| `00ac36c0` | 127/45 zone data: in-instance flag, head size, rating info request |
| `00ab8ee0` | 127/44 house info (confirmed) |
| `00ab9080` | 127/46 error codes (confirmed) |
| `00ac8340`, `00ac2180`, `00ab6f40` | 127/40 fixture update and its readers |
| `00ac7f40`, `00ac1f50` | 127/42 fixture asset handler and reader (likely) |
| `00abae70` | 127/41 remove (likely) |
| `00ac9bd0` | 127/43 owned item list |
| `00ab9860` | 127/53 placement denied / `HousingMaxFixtures` (confirmed) |
| `00ac8940`, `00ac8d90`, `00ac8790` | Place commit, edit-state machine, save (unknown labels, code read) |
| `00ac42d0`, `00ac4200`, `00ac8680` | Pick up, place request, pick up all (senders) |
| `00ac3fe0`, `00ac40b0`, `00ac4b80` | Enter, enter preview, leave senders |
| `00ac4cd0`, `00ac4e00`, `00abd510`, `00abaca0` | Invite, decline, invite notification, decline notification |
| `00ab5af0` | `IsHousingActionPermitted` (MoveFixture, RemoveFixture, Elevate) |
| `00aab720`, `00ab1e30`, `00ab0f20` | Grid snap, rotation step, convex-hit filter |
| `00aaae80`, `00aab230`, `00aaafc0`, `00aab3b0` | Preview actor create, rebuild, commit/clear, orient |
| `00abb1c0`, `00aae6e0` | Per-frame surface target and preview transform |
| `00ab0d60`, `00ab2250` | Selection callbacks for move and vertical move |
| `00aaf090`, `00aaf1b0` | Free-fly camera and auto-orientation options |
| `00ac9340`, `00ac5250`, `00ac6a00`, `00ac7260`, `00ac62e0`, `00ac71a0` | Rating lookup/send, publish, vote, search, featured, report |
| `00ab94b0`, `00ab96e0` | `Housing:SetIsInInstance` and `CuteMode:SetSize` calls |
| `00abd680`, `00ab41a0`, `00ab5f10` | `HouseNames.txt`, `HousingNames.txt`, `HouseRatingKeywords.txt` loaders |
| `00c0b410`, `0182add0` | `House` Lua registrar and its 53-entry table |
| `00cbf9b0`, `00cbdc90`, `00cbd760` | Housing data source and value getters |
| `00cba340`, `00cb9dc0`, `00cba250` | `Fixture` and `HousePiece` constructors, effect-map insert |
| `00acb770`, `00acbc80`, `00ac9180` | `TargetProcessor` ctor, per-frame target query, housing selection events |
| `0105eb60`, `0105e830`, `0105e280` | House instance constructor, destructor, fixture clear |
| `01059780`, `01059ec0` | Factory plot AddFoundation and PlaceObject (farms) |
<!-- NARRATIVE END -->

## Mapped functions (generated)

229 accepted functions: 103 confirmed, 65 likely, 30 speculative, 31 unknown. Network-linked (`wire`): 52.

Subsystems: `game.housing` (96), `ui.housing` (84), `net.housing` (49)

### `game.housing`

**Fixture** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cba340` | Fixture::Ctor | confirmed | Thiscall with seven stack arguments (param_2..param_8): calls 00cb9f80 on the receiver with param_2..param_7; stores Fixture::vftable at p+0 and p+0x50; instal… |
| `00cba4e0` | Fixture::CtorFromObject | confirmed | Thiscall with three stack arguments (param_2..param_4): calls 00cb9dc0 on the receiver (HousePiece constructor), then installs Fixture::vftable at p+0 and p+0x… |
| `00cba720` | Fixture::DtorBody | confirmed | Fastcall on receiver p: stores Fixture::vftable at p+0 and p+0x50; calls slot 4 of the object at p+0x68 with p+0x70 as argument; when that result is nonzero an… |
| `00cbb040` | Fixture::DeletingDtor | confirmed | Virtual slot 6 of the primary Fixture vftable (018373b4), thiscall with one byte flag: calls the Fixture dtor body 00cba720 on the receiver, then when flag bit… |
| `00cba4d0` | Fixture::AdjustorThunk50 | speculative | Adjustor thunk at slot 0 of the secondary Fixture vftable (01837374): bytes 83 e9 50 e9 68 0b 00 00 subtract 0x50 from ECX and jump to 00cbb040 (rel32). |
| `00cbb060` | Fixture::Slot1ThenCandidateBuild | speculative | Virtual slot 1 of the secondary Fixture vftable (01837374), thiscall with one stack argument: stores 1 at receiver+0x25; when byte +0x24 is nonzero and bit 1 o… |

**FixtureDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abd600` | FixtureDefinitionDtor | likely | Destructor of a fixture-definition record: destroys the embedded SoeUtil::HashListMap<int,FixtureDefinition::FixtureAction> member at +0xc8 through FUN_00abd31… |
| `00abf010` | FixtureDefinitionScalarDeletingDtor | likely | Scalar deleting destructor of a fixture-definition record: runs the destructor body 00abd600 in place and frees the object through _free when the low bit of th… |

**HouseActorEventHandler** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa5ed0` | HouseActorEventHandler::MatchesValue8Or1 | confirmed | Returns 1 when the input value is 8 or 1, otherwise 0. |
| `00aae3e0` | HouseActorEventHandler::ctor | confirmed | Constructor: runs the base initialiser FUN_008c49a0, stamps HouseActorEventHandler::vftable at this+0 and stores param_2 at this+0xc. |
| `00aae490` | HouseActorEventHandler::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): resets the vtable to the base TargetProcessorEventHandler::vftable and, when the low bit of param_2 is set, frees… |
| `00ac9b30` | HouseActorEventHandler::OnEvent | confirmed | Virtual slot 3 of the HouseActorEventHandler vtable: obtains the housing controller through FUN_008a67a0(this+0xc), returns immediately when the controller's b… |

**HousePiece** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cb9da0` | HousePiece::DeletingDtor | confirmed | Thiscall with one stack argument (a byte flag): calls the HousePiece dtor body 00cb98f0 on the receiver, then when flag bit 0 is set calls _free (thunk 00d8246… |
| `00cb9dc0` | HousePiece::Ctor | confirmed | Thiscall with two stack arguments and receiver p: calls 00bf00e0 on the receiver, stores ActorHandler::vftable at p+0x50 and then HousePiece::vftable at p+0 an… |
| `00cb9f80` | HousePiece::CtorWithStrings | confirmed | Thiscall with six stack arguments (C strings and dwords, plus an object pointer at the seventh): calls 00bf00e0 on the receiver, stores ActorHandler::vftable a… |
| `00cb90a0` | HousePiece::SetByte25One | speculative | Virtual slot 2 of HousePiece (thiscall, one ignored stack argument, ret 4): stores 1 in the byte at receiver+0x25. Bytes c6 41 25 01 c2 04 00. |
| `00cb90b0` | HousePiece::SetChildByte100IfPresent | speculative | Virtual slot 7 of HousePiece (thiscall, one ignored stack argument, ret 4): when the pointer at receiver+0x1c is nonnull, stores 1 in the byte at that pointer … |
| `00cb91d0` | HousePiece::Slot9ForwardKey70 | speculative | Virtual slot 9 of HousePiece and Fixture (shared implementation; thiscall with no stack argument of its own): bytes 8b 51 70 51 8b c4 89 10 8b 49 68 8b 01 8b 5… |
| `00cb9a40` | HousePiece::TailCallSlot9 | speculative | Virtual slot 0 of HousePiece and slot 0 of Fixture (shared implementation): bytes 8b 01 8b 50 24 ff e2 load the vtable, load slot 9 (+0x24) and tail-jump to it… |
| `00cb9a50` | HousePiece::AdjustorThunk50 | speculative | Adjustor thunk at slot 0 of the secondary HousePiece vftable (018372cc): bytes 83 e9 50 e9 48 03 00 00 subtract 0x50 from ECX and jump to 00cb9da0 (rel32). It … |
| `00cb9a60` | HousePiece::ClearDword20 | speculative | Virtual slot 13 of HousePiece and Fixture (shared implementation): bytes c7 41 20 00 00 00 00 c2 04 00 store zero in the dword at receiver+0x20 and return, pop… |
| `00cb9e80` | HousePiece::SetFlag25ThenLazyCreate | speculative | Virtual slot 1 of the secondary HousePiece vftable (018372cc), thiscall with one stack argument: stores 1 at receiver+0x25; when byte +0x24 is nonzero and bit … |
| `00cb98f0` | — | unknown | Fastcall on receiver p: stores HousePiece::vftable at p+0 and at p+0x50; frees the buffer at p+0x6c when set (_free thunk after 0072d130); when byte p+0x76 is … |
| `00cb9aa0` | — | unknown | Virtual slot 8 of HousePiece and Fixture (shared implementation; thiscall, one stack argument, ret 4): calls slot 9 of the receiver with one stack argument; wh… |

**HousePlayerEventHandler** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa5ec0` | HousePlayerEventHandler::MatchesValue8 | confirmed | Returns 1 exactly when the input value is 8, otherwise 0. |
| `00aaac60` | HousePlayerEventHandler::ctor | confirmed | Constructor: runs the base initialiser FUN_008b3470, stamps HousePlayerEventHandler::vftable at this+0 and stores param_2 at this+0xc. |
| `00aaad10` | HousePlayerEventHandler::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): resets the vtable to the base TargetProcessorEventHandler::vftable and, when the low bit of param_2 is set, frees… |
| `00ab7380` | HousePlayerEventHandler::OnSelectedFixture | confirmed | Checks a guarded housing pointer and its active byte, calls the base event callback, then forwards the event arguments to fixture-selection handling. |

**HouseTileEventHandler** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa5ef0` | HouseTileEventHandler::MatchesValue8Or1 | confirmed | Returns 1 when the input value is 8 or 1, otherwise 0. |
| `00aaad70` | HouseTileEventHandler::ctor | confirmed | Constructor: runs the base initialiser FUN_008b3590, stamps HouseTileEventHandler::vftable at this+0 and stores param_2 at this+0xc. |
| `00aaae20` | HouseTileEventHandler::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): resets the vtable to the base TargetProcessorEventHandler::vftable and, when the low bit of param_2 is set, frees… |
| `00ac9b80` | HouseTileEventHandler::OnEvent | confirmed | Virtual slot 3 of the HouseTileEventHandler vtable: gets the housing controller through FUN_008a67a0(this+0xc), returns when the controller's byte +0x16 is cle… |

**HousingProcessor** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abefc0` | HousingProcessor::EnsureResourcesLoaded | likely | When this+0x1c is clear, passes "Resources/" to FUN_00abd680 and FUN_00ab41a0, then sets the byte. Every call then passes the house object at this+0x41b0 to FU… |
| `00abfda0` | HousingProcessor::ResetLoadedHouse | likely | When this+0x41b0 is non-null, clears this+0x18 and the byte at +0x17, releases and frees the house object, clears fields at +0x41b0, +0x40e0 and +0x2888, pops … |
| `00ab0a80` | — | unknown | Stores the requested edit-mode byte, clears preview state, and on entry reads both housing preferences and creates actor, tile, and player event handlers; on e… |
| `00ac2280` | — | unknown | Destructor of the housing processor: installs HousingProcessor::vftable, tears the loaded house down with 00abfda0, releases the inline arrays (SoeUtil::Array<… |

**TargetProcessorEventHandler** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aae440` | TargetProcessorEventHandler::ctor | confirmed | Minimal constructor: stamps TargetProcessorEventHandler::vftable at this+0 and returns the object; no base class call and no member writes. |
| `00aaacc0` | TargetProcessorEventHandler::ctor | likely | Minimal constructor: stamps TargetProcessorEventHandler::vftable at this+0 and returns the object; no base class call and no member writes. |
| `00aaadd0` | TargetProcessorEventHandler::ctor | likely | Minimal constructor: stamps TargetProcessorEventHandler::vftable at this+0 and returns the object; no base class call and no member writes. |

**(no class)** (57)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa5f30` | NotifyNotInHousingEditMode | confirmed | If the byte at this+0x16 is zero, reads the notifier from this+0x10 +8, calls FUN_0102cfe0 and FUN_0102cff0 for the colour argument, and calls the notifier vta… |
| `00aab720` | SnapPositionToGridXZ | confirmed | Quantises the position stored at this+0x40c0..0x40cc: after FUN_009d9b70 touches the field and, when the byte at this+0xeedd is set, the position is rotated in… |
| `00aaf090` | ApplyUseFreeFlyCameraOption | confirmed | Reads the Housing.UseFreeFlyCamera option, stores its enabled state at +0xeedc, and when edit mode is active calls the camera-state helpers according to the cu… |
| `00aaf1b0` | ApplyUseAutoOrientationOption | confirmed | Reads the Housing.UseAutoOrientation option and stores whether the retrieved value is zero at +0xeedd when a housing instance is active. |
| `00ab0d60` | PlacementSelectionMode1Callback | confirmed | When a loaded house exists, sets mode 1, recreates the preview actor, resolves the selection pair, copies selection and transform data into processor fields, u… |
| `00ab0f20` | FilterPlacementConvexHits | confirmed | Builds physics queries into a ConvexHit array, examines returned hit objects and their type fields, stops scanning when required lookup/type checks fail, and t… |
| `00ab2250` | PlacementSelectionMode2Callback | confirmed | When a loaded house exists, sets mode 2, recreates the preview actor, resolves the selection pair, copies selection and transform data into processor fields, u… |
| `00ab4690` | ExportHouseFixtures | confirmed | Opens an output file, walks fixture rows, filters objects by supplied bounds, calculates group bounds and pivot, resolves model metadata and writes object-grou… |
| `00ab5af0` | IsHousingActionPermitted | confirmed | Looks up action labels, checks housing action and permission state, applies actor or house restrictions, and returns a boolean for the requested action. |
| `00ac9180` | HandleHouseSelectionEvent | confirmed | Dispatches on the event kind: with a held preview fixture (this+0x40bc non-null) and this+0x18 non-zero, kind 8 calls FUN_00ac8940 (place the held fixture), ki… |
| `0105e280` | HousingInstanceClearFixtures | confirmed | Fastcall (ECX=this). Logs 'Removing %d fixtures, %d bytes OOB and FactoryProperties from instance %lld' to Housing.log with [this+0xe4], [this+0xc4], and the 6… |
| `0097c680` | HouseNameTemplateExpand | likely | Gets the handle for the template 'HouseName' from the object at this+8+4 through its vtable slot 0x14, builds a temporary IString pair with 0097a330, assigns '… |
| `00aaae80` | CreatePreviewActorAtOffset | likely | When the byte at this+0x16 is set it reads two 4-float vectors through FUN_008a7d50/FUN_008a7d90, adds them (scale 1.0, _DAT_0175b420) into a position, allocat… |
| `00aaafc0` | CommitPreviewTransformAndClear | likely | Clears the dword at this+0x18 and the byte at this+0x40ec and, when the preview object at this+0x40bc exists, copies its transform dwords at +0x110..+0x11c ont… |
| `00aab1a0` | LookupHashedStringValue | likely | Hashes param_2 with FUN_00705b00, indexes the table at this+0x28b0 with (hash & 0xf) and walks the chain at +0xb8 comparing each node's key with FUN_006fc5b0; … |
| `00aab230` | RecreatePreviewActorFromTarget | likely | Resolves the current target through FUN_0105bfa0, FUN_00984960 and FUN_0095a3e0; when all succeed it tears down any existing preview (00aaafc0), allocates and … |
| `00aab3b0` | OrientPreviewToSurfaceNormal | likely | Reads the preview object's matrix (virtual slot +0x24 on *(this+0x40bc), +0x160 block) and picks an orientation vector from +0x170 or +0x180 depending on param… |
| `00aab5c0` | GetPlacementTransformMatrix | likely | Fills param_2 with a 4x4 matrix: when FUN_0105bfa0(param_3,param_4) finds no object it copies 16 dwords from the preview object at this+0x40bc (+0xb0) when tha… |
| `00aab6e0` | SetSelectionId | likely | Stores param_2 at this+0x40e0, calls virtual slot +0x10 on the object at this+0x4198, sets the byte at this+0x17 to (param_2 != 0) and calls FUN_00aa8da0() whe… |
| `00aab960` | LookupNodeFlag74ByTargetId | likely | Resolves an object through FUN_0105bfa0(param_2,param_3) then FUN_01070770, indexes the chained table at *(*(this+0x10)+0x20c)+0x64 with (id at +0xc) & 0x3ff a… |
| `00aae6e0` | UpdatePreviewTransformAgainstSurface | likely | Clears the preview collision flag, ray-tests transforms derived from the housing object, updates position and basis fields at +0x40c0..+0x40dc, and calls the g… |
| `00ab41a0` | LoadHousingNamesFile | likely | Builds a HousingNames.txt path, reads lines, skips comments and short rows, parses each remaining line and appends identifiers to an array. |
| `00ab50d0` | PrintFixtureTableRows | likely | Reads each fixture table row, formats its columns and values, and emits the formatted row through the housing logger. |
| `00ab52d0` | PrintFixtureTableCells | likely | Iterates fixture table rows and cells, appends each cell label and value to a string, and emits one diagnostic line per row. |
| `00ab54b0` | HandleHousingServerUnavailable | likely | Looks up the HousingServerNotFound localization key, uses an English fallback when missing, and emits the resulting client notification. |
| `00ab5590` | HandleHousingPermissionDenied | likely | Looks up HousingPermissionDenied localization, falls back to the English denial text, and emits the message through the client notification helper. |
| `00ab5670` | HandleHousingZoneNotFound | likely | Looks up HousingZoneNotFound localization, falls back to its English sentence, and emits a client notification. |
| `00ab5750` | HandleHousingZoneLaunchFailure | likely | Looks up HousingZoneFailedToLaunch localization, falls back to its English sentence, and emits a client notification. |
| `00ab5830` | HandleHousingZoneFull | likely | Looks up HousingZoneFull localization, falls back to its English visitor-limit text, and emits a client notification. |
| `00ab5910` | SelectHouseFixture | likely | Checks selection state and fixture flags, invokes selection and property callbacks, then formats a diagnostic with fixture id, name, position and heading. |
| `00ab5f10` | LoadHouseRatingKeywords | likely | Builds the HouseRatingKeywords.txt path, reads non-comment lines, stores keyword and paired flag entries, and notifies an observer if present. |
| `00ab9400` | FindOrInsertTwoStringRecord | likely | Given two non-empty C strings, returns early when either is null or empty. It looks up an existing two-string record with FUN_00aab0e0 and, when found, increme… |
| `00abb1c0` | UpdatePlacementSurfaceTarget | likely | Per frame: reads the client transform through a vtable call on the object at this+0x10 (+0xc8 then slot +0x24) and the held preview at this+0x40bc, scans the q… |
| `00ac4630` | HousingFrameUpdate 🌐 | likely | When this+0xeede is clear, sends the queued subtype-20 preview request from this+0x2884 when connected, then sets process+0xbe1=1 and +0xbe4=0x3c, clears the q… |
| `00ac8340` | ReadAndLinkHousingRecord 🌐 | likely | When this+0x41b0 is set, creates temporary records and a HousingPacketFixtureUpdate object, parses the supplied body through FUN_00ac2180, and on the continuin… |
| `00ac8680` | SendPickupAllFixturesAndOptionalRatingRemoval 🌐 | likely | Constructs ClientHousingPacketPickupAllFixtures (housing family 127, subtype 4) in a 12-byte stack object. If the client connection flag is non-zero it sends t… |
| `00ac9bd0` | ApplyFixtureItemListMessage 🌐 | likely | Constructs a temporary item-list record, parses the body with FUN_00ac9a90, and on success drains the temporary node list through FUN_00aaef30 and this+0xded0 … |
| `00cb93f0` | ResolveSubBlock2f0OrDefault | likely | Fastcall on receiver p: calls the object at p+0x68 through its vtable slot 4 with p+0x70 as one stack argument (callee-cleaned); when the result is nonnull ret… |
| `00cb9420` | ResolveSubBlock190OrDefault | likely | Fastcall on receiver p: calls the object at p+0x68 through its vtable slot 4 with p+0x70 as one stack argument; when the result is nonnull returns that pointer… |
| `00aab920` | GetActorUidFromField41A0 | speculative | Field getter: returns the dword at +0x18 of the object reached through *(this+0x41a0)+0x260, or 0 when either pointer is null. |
| `00ab39e0` | CopyFixtureFields | speculative | Copies a word at source offset 0x7c and embedded fields through offset 0xa4 into the destination when its pointer is non-null. |
| `00ab43e0` | FindFixtureItemRow | speculative | Queries the FixtureItems GUI table, compares cell strings against the supplied key, and returns a matching row or -1. |
| `00abb130` | RemoveKeyedChainEntries | speculative | Hashes the key argument with FUN_00705b00 and scans the 16-bucket table at this+0x28b0 for the matching entry, then walks the chains linked at node+0xb8 and no… |
| `00ac9140` | SavePlacementAtPositionWrapper 🌐 | speculative | Calls FUN_00ab1e30(param_2,param_3,param_4) first, saves this+0x40e0, calls FUN_00ac8790, then restores the saved pointer to this+0x40e0. The save call itself … |
| `00ab1380` | — | unknown | Reads the preview matrix and oriented bounds, adjusts the candidate position along the preview basis, tests a sequence of candidate transforms with convex quer… |
| `00ab1e30` | — | unknown | Builds a placement transform from the selection pair and action value, uses the recorded 15-degree rotation step and the conditional 90-degree angle path, chec… |
| `00ab6110` | — | unknown | Finds a keyword row, updates its stored byte when the requested flag differs, and notifies the observer if present. |
| `00abb630` | — | unknown | Per frame, returns when the held preview object at this+0x40bc is absent or not visible. Otherwise it reads the kind at (preview+0x16c)+0x10 and proceeds only … |
| `00abd680` | — | unknown | Formats a '%sHouseNames.txt' path from the caller's prefix, opens it, reads lines until exhausted, skips lines beginning with '#', splits eligible lines into n… |
| `00abdb20` | — | unknown | Start-up loader for the housing tables: calls FUN_00abd680 and then FUN_00ab5f10, both with the 'CResources/' path constant (the literal at 017ed7d8 is pushed … |
| `00abf1b0` | — | unknown | Pops the head node of a doubly linked list (head at +0, tail at +4, count at +8, node links prev +0 / next +4): when the head is non-null it unlinks it from bo… |
| `00ac8790` | — 🌐 | unknown | Requires housing active (+0x16), a house (+0x41b0) and a selected fixture (+0x40e0). It resolves the actor, invokes the actor virtual at +0x24 while preparing … |
| `00ac8940` | — 🌐 | unknown | Checks preview object this+0x40bc and shows the two placement error strings if byte +0x165 is clear. If word +0x168 is zero, saves +0x160, copies two 16-byte v… |
| `00ac8d90` | — 🌐 | unknown | Switches on the edit-state word this+0x4140: state 1 copies the saved bbox words this+0x4160..+0x416c onto the object found by FUN_0105bfa0 (stored at this+0x4… |
| `00aca330` | — 🌐 | unknown | Creates the client house object on first use (FUN_0105eb60, stored at this+0x41b0) or, when this+0x41b0 is already set, takes the error path (FUN_0105e830 plus… |
| `0105e830` | — | unknown | Fastcall (ECX=this), EH-framed. If [this+0x79] is zero: logs to Housing.log either 'Desroying instance %lld with %d fixtures that was never saved to the db' (w… |
| `0105eb60` | — | unknown | Thiscall with one stack argument (param_2, stored at +0x20). Zeroes the leading dwords and flags, writes DAT_01cb70c8/70cc/70b8/70bc at +0x28..+0x34, IString<c… |

### `net.housing`

**ClientHousingPacketApplyCustomizationToFixtureGroupAndType** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abcce0` | ClientHousingPacketApplyCustomizationToFixtureGroupAndType_… 🌐 | likely | Calls FUN_00aba910 for the housing header, writes the dword at source+0xc through FUN_007cc400, then calls FUN_00911c30 on string members at source+0x10 and so… |
| `00ac0740` | SendClientHousingPacketApplyCustomizationToFixtureGroupAndT… 🌐 | likely | Transport wrapper for 127/59: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abcce0 over the caller's packet (one u32 plus t… |

**ClientHousingPacketChangeHouseName** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abce90` | ClientHousingPacketChangeHouseName_SerializeBody 🌐 | likely | Calls FUN_00aba910 for the housing header, then writes only the dword at source+0xc through FUN_007cc400; no string helper is called. |
| `00ac10e0` | SendClientHousingPacketChangeHouseName 🌐 | likely | Transport wrapper for 127/9: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abce90 over the caller's packet (a single dword)… |

**ClientHousingPacketDeclineInvite** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac1310` | SendClientHousingPacketDeclineInvite 🌐 | likely | Transport wrapper for 127/16: opens the scratch scope with FUN_008d4a40(0x10000000), emits the family header with FUN_00aba910, writes eight bytes from source+… |

**ClientHousingPacketEnterRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac01e0` | — 🌐 | unknown | Transport wrapper for 127/19: opens the shared transport scratch scope with FUN_008d4a40(0x10000000), runs the enter-request body writer FUN_00abca80 over the … |

**ClientHousingPacketExplosionResetRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac1e40` | SendClientHousingPacketExplosionResetRequest 🌐 | likely | Transport wrapper for the payloadless explosion-reset request: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00a… |

**ClientHousingPacketInvitePlayer** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac11f0` | SendClientHousingPacketInvitePlayer 🌐 | likely | Transport wrapper for 127/15: opens the scratch scope with FUN_008d4a40(0x10000000), emits the family header with FUN_00aba910, writes the 64-bit field at +0x1… |

**ClientHousingPacketLeaveHouse** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac0fd0` | SendClientHousingPacketLeaveHouse 🌐 | likely | Transport wrapper for the payloadless 127/10 leave request: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00aba9… |

**ClientHousingPacketLockHouse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abce00` | ClientHousingPacketLockHouse_SerializeBody 🌐 | likely | Calls FUN_00aba910 to write the housing header, serializes eight bytes beginning at source+0x10 through FUN_008fd1a0, then writes the byte at source+0x18 throu… |
| `00ac0ca0` | SendClientHousingPacketLockHouse 🌐 | likely | Transport wrapper for 127/22: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abce00 over the caller's packet (one u64 and on… |

**ClientHousingPacketPayUpkeep** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac0970` | SendClientHousingPacketPayUpkeep 🌐 | likely | Transport wrapper for the payloadless 127/7 upkeep payment: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00aba9… |

**ClientHousingPacketPickupAllFixtures** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac0630` | SendClientHousingPacketPickupAllFixtures 🌐 | speculative | Transport wrapper for a payloadless housing request: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00aba910 over… |

**ClientHousingPacketPickupFixture** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac0510` | SendClientHousingPacketPickupFixture 🌐 | confirmed | Transport wrapper for 127/3: opens the scratch scope with FUN_008d4a40(0x10000000), emits the housing family header with FUN_00aba910 and writes the 64-bit fix… |

**ClientHousingPacketPlaceFixture** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abffb0` | — 🌐 | unknown | Transport wrapper for the 127/2 place-fixture commit: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abc9a0 over the caller'… |

**ClientHousingPacketPlaceFixtureRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abcb90` | ClientHousingPacketPlaceFixtureRequest_SerializeBody 🌐 | confirmed | Calls FUN_00aba910 to write the housing header, then writes dwords from source+0xc, +0x10 and +0x14 and one byte from +0x18 through bounded writer FUN_007cc400… |
| `00ac0400` | — 🌐 | unknown | Transport wrapper for 127/1: opens the scratch scope with FUN_008d4a40(0x10000000), runs the place-fixture body writer FUN_00abcb90 over the caller's packet (t… |

**ClientHousingPacketPreviewByItemDefinitionIdRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abcd90` | ClientHousingPacketPreviewByItemDefinitionIdRequest_Seriali… 🌐 | confirmed | Calls FUN_00aba910 to write the housing header, then writes the dword at source+0xc through FUN_007cc400 and advances the bounded cursor. |
| `00ac0a80` | SendClientHousingPacketPreviewByItemDefinitionIdRequest 🌐 | confirmed | Transport wrapper for 127/20: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abcd90 over the caller's packet (a single u32 i… |

**ClientHousingPacketRemoveCustomizationFromFixtureGroupAndType** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac0850` | SendClientHousingPacketRemoveCustomizationFromFixtureGroupA… 🌐 | likely | Transport wrapper for 127/60: opens the scratch scope with FUN_008d4a40(0x10000000), emits the family header with FUN_00aba910 and appends the two string membe… |

**ClientHousingPacketRequestGrant** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac1d30` | SendClientHousingPacketRequestGrant 🌐 | likely | Transport wrapper for the payloadless property-grant request: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00ab… |

**ClientHousingPacketRequestName** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abcf40` | ClientHousingPacketRequestName_SerializeBody 🌐 | likely | Calls FUN_00aba910, serializes the string member at source+0xc with FUN_00911c30, then writes one byte from source+0x1c through FUN_007cc400. |
| `00ac1430` | SendClientHousingPacketRequestName 🌐 | likely | Transport wrapper for the house-name request: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abcf40 over the caller's packet… |

**ClientHousingPacketRequestPlayerHouses** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac00c0` | SendClientHousingPacketRequestPlayerHouses 🌐 | likely | Transport wrapper for the 127/14 player-houses request: opens the scratch scope with FUN_008d4a40(0x10000000), emits the housing family header with FUN_00aba91… |

**ClientHousingPacketRequestVisitToNeighbor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac1c10` | SendClientHousingPacketRequestVisitToNeighbor 🌐 | likely | Transport wrapper for 127/21: opens the scratch scope with FUN_008d4a40(0x10000000), emits the family header with FUN_00aba910 and appends the client's own gui… |

**ClientHousingPacketSaveFixture** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abf620` | ClientHousingPacketSaveFixture_SerializeBody 🌐 | confirmed | Body writer for housing subtype 5: emits the 127 family header with FUN_00aba910, writes the u64 at source+0x10 with the u64 emitter FUN_008fd1a0, writes the t… |

**ClientHousingPacketSetEditMode** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac02f0` | SendClientHousingPacketSetEditMode 🌐 | confirmed | Transport wrapper for 127/6: opens the scratch scope with FUN_008d4a40(0x10000000), runs the edit-mode body writer FUN_00abcb30 over the caller's packet (a sin… |

**ClientHousingPacketToggleFloraAllowed** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac0db0` | SendClientHousingPacketToggleFloraAllowed 🌐 | likely | Transport wrapper for the payloadless 127/12 flora toggle: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00aba91… |

**ClientHousingPacketToggleLocked** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac0b90` | SendClientHousingPacketToggleLocked 🌐 | likely | Transport wrapper for the payloadless 127/11 lock toggle: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00aba910… |

**ClientHousingPacketTogglePetAutospawn** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac0ec0` | SendClientHousingPacketTogglePetAutospawn 🌐 | likely | Transport wrapper for the payloadless 127/13 pet-autospawn toggle: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN… |

**FixtureDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abeb40` | ReadFixtureDefinitionTail 🌐 | confirmed | Reads the fixture-definition tail (50 bytes in the captured empty-string/empty-action case; variable strings and actions change the total length) from the boun… |

**HousingPacketFixtureAsset** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ac1f50` | — 🌐 | unknown | Body parser for the fixture-asset message: initialises the bounded reader with FUN_00ab2bc0, reads two dwords into destination+0xc and +0x10, reads the fixture… |

**HousingPacketInstanceList** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abd380` | HousingPacketInstanceList::HousingPacketInstanceList 🌐 | confirmed | Calls FUN_00aa4990 with 0x27, installs HousingPacketInstanceList and embedded List<PlayerHousingInstanceInfo,-1> vtables, and zeroes list head, tail and count … |
| `00abd3f0` | HousingPacketInstanceList::~HousingPacketInstanceList | confirmed | Resets the embedded list vtable at this+0x18, removes each node through FUN_00ab6dd0, then rewrites this to BaseHousingPacket and BasePacket vtables. |
| `00abef00` | HousingPacketInstanceList::ScalarDeletingDtor | confirmed | Scalar deleting destructor of the instance-list packet: calls the destructor body 00abd3f0 in place and frees the object with _free when the low bit of the fla… |
| `00abf210` | — 🌐 | unknown | Body parser for the inbound instance list: builds the bounded reader over [param_2, param_2+param_3) with FUN_00ab2bc0, reads the record list at destination+0x… |

**HousingPacketInviteNotification** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abd510` | HousingPacketInviteNotification_ReadBody 🌐 | likely | Builds a bounded reader over input data/length, constructs a local invite-notification object through FUN_00ab3be0, parses it through FUN_00ab8bd0, dispatches … |

**HousingProcessor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abf850` | HousingProcessor::HousingProcessor | confirmed | Constructor of the housing processor: installs HousingProcessor::vftable, stores the first argument at +0x10, zeroes the state bytes at +0x14..+0x1d, builds th… |

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aafa90` | ReadCharacterGuidArray 🌐 | confirmed | Reads a signed count, checks it against the remaining reader span, resizes the destination array, and reads one 64-bit value per element; short reads or an inv… |
| `00ab0340` | ReadIntStringIntIntRecord 🌐 | confirmed | Reads a dword, reads a signed byte length and skips that byte span, then reads two dwords into the destination; invalid lengths or short fields set the bounded… |
| `00ab2bc0` | ReadHousingFamilyAndSubtype 🌐 | confirmed | Reads two signed 16-bit values from the bounded reader into destination offsets +4 and +8, advances the cursor by four bytes, and marks failure with the cursor… |
| `00abd1d0` | SerializeSaveFixtureNestedBlock | confirmed | Writes the first source dword, length-prefixed blocks from source+4 and +0x14, a dword at +0x24 and another length-prefixed block at +0x28. |
| `00aafb50` | RebuildCountedU32List 🌐 | likely | Drains the destination linked list, reads a signed count, then allocates one node per entry and reads one dword into each; it stops when the bounded reader rep… |
| `00ab01e0` | ZoneDataPayloadInitWrapper | likely | Runs an exception-frame wrapper around the zone-data payload initializer and returns its original input value. |
| `00abf760` | ReadFixtureDefinitionList 🌐 | likely | Destroys the current head element of the destination list with FUN_00abf1b0, reads a count from the bounded reader in param_1 and then, for each of that many e… |
| `00aafce0` | DestroyTemporaryZoneDataStrings | speculative | Runs an exception-frame wrapper and invokes the zone-data payload string cleanup helper, then restores the previous exception list. |
| `00aafd30` | — 🌐 | unknown | Reads a 64-bit value, two dwords, a counted sequence of 64-bit values, and two more dwords into the fixture-info record; short fields set the bounded reader er… |
| `00aafe10` | — 🌐 | unknown | Reads fixed pairs and scalar fields, two signed-length byte spans, several flags, and a trailing pair into the destination; each short or invalid span sets the… |
| `00ac2180` | — 🌐 | unknown | Reads one fixture record of the 127/40 update: initialises the bounded reader with FUN_00ab2bc0, reads the leading record through FUN_00ab6f40, the 28-byte fix… |

### `ui.housing`

**FeaturedDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf9dc0` | FeaturedDataSource::GetColumnName | confirmed | Returns names for System, Guid, OwnerName, Name, OwnerGuid, Snapshot, Rating, and NumVotes columns. |
| `00cfa100` | FeaturedDataSource::GetCell | confirmed | Resolves a row and returns system, GUID, owner, name, snapshot, rating, or vote count fields; missing data yields an error string. |
| `00cfa460` | FeaturedDataSource::FeaturedDataSource | confirmed | Initializes the datasource base using BaseClient.FeaturedRating, installs the class vtables, stores two arguments, and clears its populated flag. |
| `00cf9f40` | FeaturedDataSource::GetRowCount | speculative | If the source is dirty, refreshes its rows, then returns the collection count at offset 8. |
| `00cfa330` | FeaturedDataSource::Cleanup | speculative | Restores the FeaturedDataSource vtables and destroys its embedded GUI userdata table. |
| `00cfa4f0` | FeaturedDataSource::ScalarDeletingDtor | speculative | Calls the class cleanup and frees the object when the delete flag's low bit is set. |

**FixtureDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbb420` | FixtureDataSource::ColumnHeaderText | confirmed | Virtual slot 1 of FixtureDataSource (thiscall, index argument): index 0 returns Fixture Guid, 1 returns Texture Alias, 2 returns Tint Alias; any other index re… |
| `00cbe150` | FixtureDataSource::GetValue | confirmed | Checks that fixture data exists, refreshes a cached field when its flag is set, bounds-checks the row index, then formats the selected fixture's 64-bit identif… |
| `00cbf3f0` | FixtureDataSource::ScalarDeletingDtor | confirmed | Calls the fixture-data-source destructor and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
| `00cbf2a0` | FixtureDataSource::FixtureDataSource | likely | Registers BaseClient.Housing.Instance.Fixtures, installs the class and guid-array vtables, initializes array counts and pointers, stores one argument, and clea… |
| `00cbb460` | FixtureDataSource::GetSub238Field7c | speculative | Virtual slot 2 of FixtureDataSource (thiscall, no arguments): when the pointer at receiver+0x238 is nonzero returns the dword at that pointer plus 0x7c, otherw… |

**FixtureItemDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbb160` | FixtureItemDataSource::ColumnHeaderText | confirmed | Virtual slot 1 of FixtureItemDataSource (thiscall, index argument): for index below 0x29 returns the result of 00c92c10 on the index; index 0x29 returns Fixtur… |
| `00cbbac0` | FixtureItemDataSource::Ctor | confirmed | Thiscall with three stack arguments: calls 00c96a30 on the receiver with param_4 and param_2, stores FixtureItemDataSource::vftable at p+0, p+0x30 and p+0x34, … |
| `00cbc260` | FixtureItemDataSource::DeletingDtor | confirmed | Virtual slot 0 of FixtureItemDataSource (thiscall, one byte flag): calls 00c968b0 on the receiver (dtor body), then when flag bit 0 is set calls _free (thunk 0… |
| `00cbd390` | FixtureItemDataSource::GetValue | confirmed | Returns values for an indexed fixture row; handles column 0x29 by formatting an integer and column 0x2a by building a player-location string from actor coordin… |
| `00cbbba0` | FixtureItemDataSource::GetField52d0 | speculative | Virtual slot 2 of FixtureItemDataSource: bytes 8b 81 d0 52 00 00 c3 decode to mov eax,[ecx+0x52d0]; ret, returning the dword at receiver+0x52d0. |

**HouseCustomNameDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbb480` | HouseCustomNameDataSource::ColumnHeaderText | confirmed | Virtual slot 1 of HouseCustomNameDataSource (thiscall, index argument): 0 returns String, 1 returns Id, 2 returns Part, 3 returns Selected; any other index ret… |
| `00cbe200` | HouseCustomNameDataSource::GetValue | confirmed | Resolves a row into the house custom-name table, returns fields by column, and on the custom lookup column forms a player/house key and queries the Houses tabl… |
| `00cbedd0` | HouseCustomNameDataSource::Destroy | confirmed | Restores HouseCustomNameDataSource vtables at the primary and two embedded bases, then destroys the embedded GUI user-data holder. |
| `00cbf410` | HouseCustomNameDataSource::HouseCustomNameDataSource | confirmed | Registers BaseClient.HouseCustomNameList, installs the class's three vtables, stores two constructor arguments, and clears the byte flag at offset 0x25c. |
| `00cbf4a0` | HouseCustomNameDataSource::ScalarDeletingDtor | confirmed | Calls the custom-name data-source destructor and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
| `00cbc730` | HouseCustomNameDataSource::GetValueAfterRebuildType123 | speculative | Virtual slot 2 of HouseCustomNameDataSource (vtable 018378c0; thiscall, no arguments): when the byte at receiver+0x22c is zero calls 00cbc400 with ECX = receiv… |

**HouseNamesDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbb1b0` | HouseNamesDataSource::ColumnHeaderText | confirmed | Virtual slot 1 of HouseNamesDataSource (stdcall-style, one stack argument, ret 4): index 0 returns Name Id, index 1 returns Name, any other index returns <erro… |
| `00cbd530` | HouseNamesDataSource::GetValue | confirmed | Reads the indexed house-name id; column 0 formats that id, while column 1 resolves a wide-string name and builds a player-specific key before returning the res… |
| `00cbef50` | HouseNamesDataSource::HouseNamesDataSource | confirmed | Registers the BaseClient.Housing.AvailableNames data source, installs its vtables, and stores two constructor arguments in fields 0x254 and 0x258. |
| `00cbf030` | HouseNamesDataSource::ScalarDeletingDtor | confirmed | Calls the class destructor and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
| `00cbbbe0` | HouseNamesDataSource::GetViaSub228Dec0 | speculative | Virtual slot 2 of HouseNamesDataSource: bytes 8b 81 28 02 00 00 8b 80 c0 de 00 00 c3 decode to load the pointer at receiver+0x228, load the dword at that point… |

**HouseRatingDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf9d40` | HouseRatingDataSource::GetColumnName | confirmed | Returns names for HouseGuid, OwnerName, HouseName, OwnerGuid, Snapshot, Rating, and NumVotes columns. |
| `00cf9f70` | HouseRatingDataSource::GetCell | confirmed | Resolves a row and returns house GUID, owner name, house name, owner GUID, snapshot, rating, or vote count; missing data yields an error string. |
| `00cfa3b0` | HouseRatingDataSource::HouseRatingDataSource | confirmed | Initializes the datasource base using BaseClient.HouseRating, installs the class vtables, stores two arguments, and clears its populated flag. |
| `00cf9f10` | HouseRatingDataSource::GetRowCount | speculative | If the source is dirty, refreshes its rows, then returns the collection count at offset 8. |
| `00cfa2b0` | HouseRatingDataSource::Cleanup | speculative | Restores the HouseRatingDataSource vtables and destroys its embedded GUI userdata table. |
| `00cfa440` | HouseRatingDataSource::ScalarDeletingDtor | speculative | Calls the class cleanup and frees the object when the delete flag's low bit is set. |

**HouseRatingKeywordsDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbb4e0` | HouseRatingKeywordsDataSource::ColumnHeaderText | confirmed | Virtual slot 1 of HouseRatingKeywordsDataSource (thiscall, index argument): 0 returns Id, 1 returns String; any other index returns <error>. |
| `00cbe430` | HouseRatingKeywordsDataSource::GetValue | confirmed | Looks up a rating-keyword row and returns one of its three fields, resolving the middle field through a secondary lookup; reports explicit errors for missing r… |
| `00cbee50` | HouseRatingKeywordsDataSource::Destroy | confirmed | Restores HouseRatingKeywordsDataSource vtables at the primary and two embedded bases, then destroys the embedded GUI user-data holder. |
| `00cbf4c0` | HouseRatingKeywordsDataSource::HouseRatingKeywordsDataSource | confirmed | Registers BaseClient.HouseRatingKeywordsList, installs the class's three vtables, stores two constructor arguments, and clears the byte flag at offset 0x25c. |
| `00cbf550` | HouseRatingKeywordsDataSource::ScalarDeletingDtor | confirmed | Calls the rating-keyword data-source destructor and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
| `00cbc480` | HouseRatingKeywordsDataSource::GetValueAfterRebuild | speculative | Virtual slot 2 of HouseRatingKeywordsDataSource (vtable 01837938; thiscall, no arguments): when the byte at receiver+0x22c is zero calls 00cbbc50 with ECX = re… |

**HouseSearchDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbb520` | HouseSearchDataSource::ColumnHeaderText | confirmed | Virtual slot 1 of HouseSearchDataSource (thiscall, index argument): 0 returns HouseGuid, 1 returns OwnerName, 2 returns HouseName, 3 returns OwnerGuid, 4 retur… |
| `00cbe570` | HouseSearchDataSource::GetValue | confirmed | Looks up a search-result row and maps columns 0 through 6 to its integer id, string fields and numeric fields, formatting the 64-bit key for column zero. |
| `00cbeed0` | HouseSearchDataSource::Destroy | confirmed | Restores HouseSearchDataSource vtables at the primary and two embedded bases, then destroys the embedded GUI user-data holder. |
| `00cbf570` | HouseSearchDataSource::HouseSearchDataSource | confirmed | Registers BaseClient.HouseSearchList, installs the class's three vtables, stores two constructor arguments, and clears the byte flag at offset 0x25c. |
| `00cbf600` | HouseSearchDataSource::ScalarDeletingDtor | confirmed | Calls the search data-source destructor and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
| `00cbc4a0` | HouseSearchDataSource::GetValueAfterRebuildSlot4 | speculative | Virtual slot 2 of HouseSearchDataSource (thiscall, no arguments): when the byte at receiver+0x22c is zero calls 00cbbcb0 with ECX = receiver-0x30; then returns… |

**HousingDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbb310` | HousingDataSource::ColumnHeaderText | confirmed | Virtual slot 1 of HousingDataSource (thiscall, index argument): switches on indices 0 to 0x11 and returns a header string: 0 Instance Guid, 1 Owner Guid, 2 Own… |
| `00cbdc90` | HousingDataSource::GetValue | confirmed | Reads the active housing instance and maps column indices to its two 64-bit identifiers, strings, booleans, counts and numeric fields; some columns resolve hou… |
| `00cbf9b0` | HousingDataSource::HousingDataSource | confirmed | Registers BaseClient.Housing.Instance, installs class and housing-guid-array vtables, initializes storage, then creates and attaches the fixture list and fixtu… |
| `00cbfbf0` | HousingDataSource::ScalarDeletingDtor | confirmed | Calls FUN_00cbfb00 to release the housing data source's subviews, array and holder, then frees the object when the low flag bit is set. |
| `00cbfb00` | — | unknown | Deletes the two owned data-source objects when the global deletion guard is enabled, clears their pointers, frees the housing-guid array with alignment 8, clea… |

**HousingInstanceDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbb210` | HousingInstanceDataSource::ColumnHeaderText | confirmed | Virtual slot 1 of the housing instance data sources (shared implementation; thiscall, index argument): switches on indices 0 to 0x11 and returns the header tex… |
| `00cbd760` | HousingInstanceDataSource::GetValue | confirmed | Selects a housing-instance row and returns its identifier, strings, booleans, numeric properties, player name, or formatted custom values according to the requ… |
| `00cbf1d0` | HousingInstanceDataSource::HousingInstanceDataSource | confirmed | Registers the supplied data-source key, installs HousingInstanceDataSource vtables, stores two arguments, and initializes the PlayerHousingInstanceInfo array v… |
| `00cbf280` | HousingInstanceDataSource::ScalarDeletingDtor | confirmed | Calls FUN_00cbed40 to release array and holder state, then tail-jumps to the allocator thunk when the low flag bit requests object deletion; otherwise returns … |
| `00cbbbf0` | HousingInstanceDataSource::GetField234 | speculative | Virtual slot 2 of the housing instance data sources (shared implementation): bytes 8b 81 34 02 00 00 c3 decode to mov eax,[ecx+0x234]; ret, returning the dword… |

**MyHousingInstanceDataSource** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbf750` | MyHousingInstanceDataSource::MyHousingInstanceDataSource | confirmed | Constructs the HousingInstanceDataSource base with BaseClient.Housing.MyHouseList, then installs the MyHousingInstanceDataSource vtables. |
| `00cbf820` | MyHousingInstanceDataSource::ScalarDeletingDtor | confirmed | Calls FUN_00cbed40 to destroy the inherited array and holder members, then frees the object when the low flag bit is set. |

**OwnerHousingInstanceDataSource** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbf620` | OwnerHousingInstanceDataSource::OwnerHousingInstanceDataSou… | confirmed | Constructs the HousingInstanceDataSource base using BaseClient.Housing.InstanceInfo, then installs the OwnerHousingInstanceDataSource vtables. |
| `00cbf6f0` | OwnerHousingInstanceDataSource::ScalarDeletingDtor | confirmed | Calls FUN_00cbed40 to destroy the inherited array and holder members, then frees the object when the low flag bit is set. |

**PlayerHousingInstanceDataSource** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbf880` | PlayerHousingInstanceDataSource::PlayerHousingInstanceDataS… | confirmed | Constructs the HousingInstanceDataSource base with BaseClient.Housing.PlayerHouseList, then installs the PlayerHousingInstanceDataSource vtables. |
| `00cbf950` | PlayerHousingInstanceDataSource::ScalarDeletingDtor | confirmed | Calls FUN_00cbed40 to destroy the inherited array and holder members, then frees the object when the low flag bit is set. |

**SelectedFixtureDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbb1e0` | SelectedFixtureDataSource::ColumnHeaderText | confirmed | Virtual slot 1 of SelectedFixtureDataSource (one stack argument, ret 4): index 0 returns Fixture Guid, any other index returns <error>. Bytes: cmp dword [esp+4… |
| `00cbd700` | SelectedFixtureDataSource::GetValue | confirmed | Reads the selected fixture pointer; returns false if it is absent or the requested column is nonzero, otherwise formats the fixture's 64-bit value into the out… |
| `00cbf090` | SelectedFixtureDataSource::SelectedFixtureDataSource | confirmed | Registers BaseClient.Housing.Instance.SelectedFixture, installs three class vtables, and stores the two supplied arguments in fields 0x254 and 0x258. |
| `00cbf170` | SelectedFixtureDataSource::ScalarDeletingDtor | confirmed | Calls the embedded-holder destructor and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |

**(no class)** (19)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0b410` | RegisterHouseModule | confirmed | Registration wrapper: pushes nup=0, the entry table 0x0182add0 and the 'House' module-name pointer, calls FUN_00e39e40 and returns 0. |
| `00c0b490` | Housing_SetUseFreeFlyCamera | confirmed | Lua 'SetUseFreeFlyCamera' handler: with one numeric argument and an active housing instance (client+0x6d8), it picks one of two static option-value blocks from… |
| `00c0b510` | Housing_SetUseAutoOrientation | confirmed | Lua 'SetUseAutoOrientation' handler: same shape as 00c0b490 but writes the 'Housing'.'UseAutoOrientation' user option and refreshes the housing manager through… |
| `00c0c0a0` | House_RotateFixtureCounterClockwise | confirmed | Lua 'RotateFixtureCounterClockwise' handler: requires one argument (FUN_00e3a630 gate), reads the fixture guid pair with FUN_00bfea90, selects action id 6 and … |
| `00c0c0f0` | House_RotateFixtureClockwise | confirmed | Lua 'RotateFixtureClockwise' handler: requires one argument, reads the fixture guid pair with FUN_00bfea90, selects action id 7 and calls the shared housing ro… |
| `00c0c210` | Housing_GetUseFreeFlyCamera | confirmed | Lua 'GetUseFreeFlyCamera' handler: reads the 'Housing'.'UseFreeFlyCamera' user option into a stack SoeUtil::IString (FUN_006feda0) and compares it against the … |
| `00c0c310` | Housing_GetUseAutoOrientation | confirmed | Lua 'GetUseAutoOrientation' handler: reads the 'Housing'.'UseAutoOrientation' user option into a stack SoeUtil::IString (FUN_006feda0) and pushes 1 when the st… |
| `00c0b5b0` | Housing_SetHouseName | likely | Lua 'SetHouseName' handler: with one numeric/id argument it resolves the housing manager through FUN_008a67a0 and calls FUN_00ac4c40(value); returns 0. |
| `00c0b600` | Housing_EnterPreview | likely | Lua 'EnterPreview' handler: with one string argument (FUN_00e3a630 gate) it resolves the housing manager via FUN_008a67a0 and calls FUN_00ac40b0(value); return… |
| `00c0b6d0` | Housing_PreviewTintAlias | likely | Lua 'PreviewTintAlias' handler: with one string argument it resolves the housing manager via FUN_008a67a0 and calls FUN_00aa5f70(tint alias); returns 0. |
| `00c0b860` | Housing_GetFixtureGroupByType | likely | Lua 'GetFixtureGroupByType' handler: requires one string argument, resolves the housing manager via FUN_008a67a0 and looks the group name up with FUN_00aab1a0;… |
| `00c0b990` | Housing_RemoveFixtureCustomizationFromGroup | likely | Lua 'RemoveFixtureCustomizationFromGroup' handler: requires two string arguments; resolves the housing manager via FUN_008a67a0 on the first and calls FUN_00ac… |
| `00c0c530` | Housing_IsActionPermitted | likely | Lua 'IsActionPermitted' handler: requires exactly two string arguments (gettop==2 plus two FUN_00e3a630 gates), reads both (FUN_00e3a820, FUN_00bfea90 for the … |
| `00c0b3c0` | Housing_ClearPlacementFields | speculative | Zeroes the four 32-bit fields at +0x4130, +0x4134, +0x4138 and +0x413c of the object passed in (16 bytes, a placement/rotation scratch block) and returns. |
| `00c0b690` | — | unknown | Lua 'PreviewTextureAlias' handler: with one string argument (FUN_00e3a630 gate, FUN_00e3a820 read) it resolves the housing manager via FUN_008a67a0 and calls F… |
| `00c0b710` | — | unknown | Lua 'PlaceFixture' handler: with a string argument it starts the guid/string copy (FUN_0101cd40/FUN_0101cd50), resolves the housing manager via FUN_008a67a0, z… |
| `00c0b8d0` | — | unknown | Lua 'ApplyFixtureCustomizationToGroup' handler: requires two strings and one numeric argument; reads the number (FUN_00e3a770 + FUN_00d83c90 into a long), star… |
| `00c0ba80` | — | unknown | Lua 'SetGridSpacingInCentimeters' handler: reads the first argument as a number (FUN_00e3a770 + FUN_00d83c90), rejects non-positive values, computes 100.0 / ce… |
| `00c0c410` | — | unknown | Lua 'RequestPlayerHouses' handler: with a string argument it reads the guid pair (FUN_00bfea90) and copies the text into a local IString (FUN_00402ce0), then r… |

## Source shards

[[S0261 00976740-0097c910|S0261]] (1), [[S0314 00aa2560-00aa90c0|S0314]] (4), [[S0315 00aa90e0-00aae6b0|S0315]] (19), [[S0316 00aae6e0-00ab2c20|S0316]] (17), [[S0317 00ab2c80-00ab7380|S0317]] (16), [[S0318 00ab73e0-00abcb30|S0318]] (4), [[S0319 00abcb90-00ac2280|S0319]] (49), [[S0320 00ac2a10-00ac7f40|S0320]] (1), [[S0321 00ac8340-00acd3f0|S0321]] (11), [[S0383 00c05780-00c0db00|S0383]] (19), [[S0413 00cb6d20-00cbc7f0|S0413]] (38), [[S0414 00cbc8e0-00cc18f0|S0414]] (35), [[S0425 00cf69c0-00cfc480|S0425]] (12), [[S0574|S0574]] (3)
