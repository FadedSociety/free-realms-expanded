---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Rewards and Collections

Part of [[Build Guides]] › Gameplay. Related: [[Quests and Objectives]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`) are the native-map labels for a function. Source tags: *client* = read from the shipped client in this pass or in an accepted shard (function pseudocode, bytes and disassembly, the shipped `Resources/*.txt` files, the vtable slots read from the binary); *client Lua* = the client's own UI script (`ScriptsBase.lua`, decompiled in the lab, not redistributed); *client locale* = the shipped text table (its keys are hashed, so a string is found by its text, not by the numeric ids in the resource files); *video* = a reviewed frame record, cited as `video/frame`; *wiki* = the fan wiki; *vault* = an earlier research note; none is relied on here, and the reward-toast note is not used for any statement that the client code in this pass does not also show. Precedence is client, then video, then wiki. *(R)* marks reconstructed server policy that is ours, not original behaviour. No original server code or capture of reward traffic exists in the project, so every statement about what the server sent is a hypothesis built from what the client requires. No player names, chat or player-chosen names are recorded. Related: [[Quests and Objectives]], [[Items and Inventory]], [[Minigame Framework]], [[Trading Card Game]], [[Coin Store and Purchases]], [[Pets]], [[Housing]], [[Combat and Abilities]], [[Gateway and Transport]], [[Text and Localization]].

This guide covers six things that the player experiences as "I got something": the **reward bundle** (the packet-borne record behind every "You receive" line), the **Claim window** (items waiting to be collected), **collections** (the world-collectible feature), **achievements and trophies**, **titles**, and the **prize wheels**. The wheel's mechanics and the minigame results screen are in [[Minigame Framework]]; quest turn-in is in [[Quests and Objectives]]; the Trading Card Game's own "collection" (cards and packs) is a different system, covered in [[Trading Card Game]] and only named here so the two are not confused.

### Overview

**What the player sees.**

- **Reward lines.** A completed quest, level-up, mystery chest or similar shows "You receive 12 stars and 54 coins" (coins and stars together) or "You receive 1 Blast Speedster" (one item). The patterns are the locale texts `RewardPatternTwo` ("You receive #count([*experience*]) and #count([*coins*])") and `RewardPatternOne` ("You receive #count([*item*])"); `CodeStringMappings.txt` ties them to string ids 3 and 2 (client resource, client locale). A third text, "You receive the title, [*item*]!", exists (client locale). Video shows the item form after a level-up banner: "Adventurer Level Up!" then "You receive 1 Seeker's Midriff Shirt" (video `nuSlsBsKmrg/00118`).
- **Results screens and wheels.** Minigames and dungeons end in a "YOU WIN! Congratulations! You won ..." panel, sometimes followed by a prize wheel (video `f3ZJ9i5YnaI/00198` Coins; `qCG_K2piQgE/00176` a Battle Item Mystery Pack). That screen is [[Minigame Framework]]; its reward rows use the same bundle record described below.
- **Mystery chests and gifts.** A reward bundle of kind 16 opens the mystery-chest panel (client, `00b899f0`). Quest rewards named "Mystery Gift" appear on seasonal quests (video `yWuHe_jYyc4/00264`, `yWuHe_jYyc4/00272`; the item family has Small, Medium and Large Ordinary, Tattered and Ornate gifts, client locale). The chest and key flow is in [[Items and Inventory]].
- **Membership bank.** Non-members see stars and coins they earned but cannot yet spend collected in a "bank" button; a member upgrade pays it out in a modal panel (client, reward subtypes 6 and 7 below).
- **Claim window.** The Marketplace has a Claim window with a code field and a "Pending Claims" list with a Claim button; each claim item can be claimed once per account (video `4h2eVoje3Eg/00288`, `4h2eVoje3Eg/00290`, `jfcerhMCY6g/00042`; client locale: "Code redemption successful! If you received a Game Item, you can claim it in the Claim Window."). Items bought in the store or won (for example tournament winnings) wait here until claimed.
- **Collections.** A "Collections" entry on the main menu opens a browser of collections grouped by category; a banner announces a collection starting, gaining an item and completing ("Collection complete!"). Exploration collections fill automatically as the player enters new places; item collections need sparkling world objects clicked (video `nuSlsBsKmrg/00124`, `CXeswcNcDTM/00076`, `LgrhYq2Hs1I/00040`; client locale; wiki). Completing exploration collections levels the Adventurer job and some collections give a reward on completion (video text; wiki).
- **Achievements.** An "Achievements" entry on the main menu opens a category tree, a summary page with an overall completed-over-total count and "most recent" list, and per-achievement pages with objectives. A banner slides up when one completes. Each achievement has a trophy tier and a point value (client Lua, client resource; wiki). The client locale announces them: "Achievements have arrived! Complete challenging feats throughout Free Realms and boost your score. Now's your chance to earn unique rewards and titles!"
- **Titles.** A player can own titles and select one. The client has a full title packet family and a data source, but the only title picker found in the Lua is gated on the China environment flag (client, client Lua; Open questions). The wiki lists VIP, job-level and questline titles.
- **Daily wheel and event trackers.** The Daily Rewards wheel ("Spin the wheel once a day to win prizes that range from potions to pets to TCG cards!") is a minigame type, covered in [[Minigame Framework]]. Seasonal wheels and present trackers exist as event text ("Spin the special Snow Days wheel every day", "Log in every day from the 13th through the 24th", client locale). The client has no general login-streak reward screen that was found.
- **Mailbox rewards.** None found. "Mail Sorting" is a match-three minigame and "Postmaster" is a job; the Claim window is the nearest thing to a reward inbox.

**What the client does not have.** It ships no reward tables: no quest, minigame, chest, wheel or collection payouts, no trophy-to-reward map (the `REWARD_ID` column is zero in every shipped row), and no collection or achievement definitions beyond category and trophy shells. All of that arrives from the server or is rebuilt by us.

### How the client implements it

#### Main classes

| Class or object | Role | Evidence |
|---|---|---|
| `RewardBundleBase` (ctor `0091c110`, vtable `018056ec`) and `ClientRewardBundle` (ctor `00b8a390`, vtable `01823e2c`, shares the base slots) | The reward record. Coins at `+0x50`, stars (XP) at `+0x48`, a third accumulator at `+0x4c`, reward kind at `+0x54`, a success byte at `+0x74`, two 8-byte id pairs at `+0x80` and `+0x88`, two string lists at `+0x94` and `+0xa4`, and an intrusive list of typed entries (head `+0x38`, tail `+0x3c`, count `+0x40`). `AddRewardEntry` is `0091c5e0`. | confirmed (S0242) |
| 17 `RewardBundleEntry*` classes on base `RewardBundleEntryBase` (ctor `0103a3e0`) | One typed entry in a bundle (item, chest, profile experience and so on; table under Data and formats). Each has TypeName, CopyFields, FormatText, ReadBody and WriteBody virtual slots. Entries are created by `CreateEntry` `0103c070` from a numeric type id. | confirmed (S0568) |
| `ClientRewardManager` (ctor `00b88ec0`) | Receives the reward family, keeps the deferred-bank totals (coins at `+0x3c`, stars at `+0x40`) and a ten-bucket reward-definition table at `+0x14`; its presentation code is `00b89ca0` and the item-notification router is `00b899f0`. | confirmed ctor, `FindReward` `00b88850` speculative (S0357) |
| `PlayerRewardEvent` (ctor `00b8a4d0`) | A queued event that holds a copy of a bundle and presents it when processed, only if the local player exists. | confirmed ctor, slot 1 `00b8a5f0` unknown label (S0357) |
| `ClientClaimManager` (ctor `00a03910`) | The Claim window's data model: a 0x168-byte-element array of claim items, a list-valid flag at `+9`, a pending counter at `+0xc`, an 8-bucket tracker map for requests in flight, and a log called `ClaimSystem.log`. | confirmed ctor, logging leaves likely (S0284, S0285) |
| `BaseClaimPacket` family (base `009ff290`, opcode 108) and `BasePromotionalPacket` (`00a057e0`, opcode 114) | The wire classes of the claim and promotional traffic. | confirmed / likely (S0285) |
| `ClientUpdatePacketCollectionStart`, `...CollectionRemove`, `...CollectionAddEntry`, `...CollectionRemoveEntry` (family 38, subtypes 8 to 11) | The collection updates the server pushes. Applied by `HandleClientUpdate` `009392c0` to a 50-bucket collection table (key at node `+0x9c`, node 400 bytes holding an `IconData` and a `RewardBundleBase`). | confirmed ctors (S0218, S0219), table helpers likely (S0265) |
| `CollectionCategoryManager` (`LoadCategories` `009fd4e0`) and data source `BaseClient.CollectionCategory` | Loads `CollectionCategories.txt` into a ten-bucket table keyed by id. | confirmed (S0284) |
| Collection data sources `BaseClient.CollectionData`, `BaseClient.CollectionEntriesData`, `BaseClient.CollectionData.Reward` and `.Reward.Entries` (ctors `00c32ea0`, `00c32f50`, `009931e0`) | What the Lua collection screens read. | confirmed / likely (S0388, S0265) |
| `ClientAchievementData` pool (`HashList<ClientAchievementData,256,8>`, ctor `00bad7a0`) and the owner `00bad8f0` | The player's achievement store. The owner builds the data sources `.Achievements`, `.Achievements.MostRecent`, `.Achievements.Objectives` and `.Achievements.Rewards` under a prefix, and is used for the player's own data and for an inspected player (prefix `BaseClient.InspectTarget.`). | ctor confirmed, owner `00bad8f0` unknown label (S0365) |
| `AchievementCategoryDefinitionManager` (`LoadCategories` `0104d880`) and `AchievementTrophyDefinitionManager` (loader `0104f870`) | Load `AchievementCategories.txt` and `AchievementTrophies.txt`. | confirmed (S0572); trophy loader unknown label |
| Achievement data sources (`AchievementDataSource`, `AchievementObjectiveDataSource`, `AchievementCategoryDataSource`, `AchievementSummaryDataSource`, `AchievementMostRecentDataSource`; ctors `00ce52a0` to `00ce6db0`) | Feed the Lua achievement window. | column-name methods confirmed, most others speculative (S0421, S0422) |
| `PlayerTitleDataSource` (ctor `00cc5930`, registers `BaseClient.PlayerTitles`), title map `HashList<int,PlayerTitleData,32,-1>` (ctor `00b49d60`) | The title store and its UI source. | confirmed (S0344, S0415) |
| Lua handlers `AchievementsHandler`, `NotificationHandler` (`CompleteAchievement`, `StartCollection`, `UpdateCollection`, `CompleteCollection`), `CollectionHandler`, `CollectionsBrowser`, `CollectionDetails`, `AboutPlayer` | The script side of all of the above. | client Lua |

#### The reward bundle

One record serves every reward in the game. The same body reader `008e7930` is called by the reward-family reader (`00b8a300`), the collection-start reader (`009099d0`), the minigame results and loot-wheel readers (`009b8230`, `009b9700`; see [[Minigame Framework]]), the quest-info and quest-end readers (`00c7b7a0`, `00c7b990`; see [[Quests and Objectives]]), the encounter-complete reader (`00aa02b0`) and a command reply for a reward-preview update (`00aa1420`) (caller list from the map; each label is `likely` or better).

**Reading a bundle** (`008e7930`, read from pseudocode and disassembly in this pass; layout in Data and formats):

1. Nine fixed fields, two 8-byte id pairs, three more dwords and an entry count are read in a fixed order.
2. For each entry the client reads a **type id** (u32) and calls the bundle's `CreateEntry` (vtable slot `+0xc0`). If a delegate object is set at bundle `+0x90`, the delegate creates the entry instead (`0103c070`).
3. If an entry object came back, the client copies the bundle's success byte (`+0x74`) into the entry through its slot 8 (`Set_0x41`, `0103a4e0`), calls the entry's `ReadBody` (slot 9) and adds it to the list (bundle slot `+0xac`).
4. If the type id is unknown the client logs "Failed to CreateRewardBundleEntry ( type: %d" and reads **no body**, then goes on to the next id. A server that sends an unknown type id therefore corrupts the rest of the stream (client, disassembly `008e7bc0`).
5. A final u32 is stored at `+0x58`.

**Entry types** are listed under Data and formats. Three observations from the code:

- The entry base reader is shared. `Experience`, `FactoryExperience`, `Token`, `WheelSpin`, `ClientExitUrl` and `Trophy` have **no body of their own**: their ReadBody and WriteBody slots are jump thunks to the base reader `0103c460` and base writer `0103d0d0` (verified from the thunk bytes in this pass).
- `Item` and `Chest` entries read an extra u32 (labelled "Item Guid" in the debug text) **only if the entry's `+0x41` byte is non-zero**, and that byte is the bundle's success byte, copied in at step 3. So the same wire entry is a different length depending on the bundle's first byte (client, `0103c5c0`, `0103c620`).
- Presentation (`00b89ca0`) handles only some entries: `Item` (type 1) and `Chest` (type 0x17) go to the item-received path, and `Token` (type 0xc) prints "You gained %d token(s)". No presentation was found for the other types in that function; they may act through other code (Open questions).

**Presentation** (`00b89ca0`, `00b899f0`, `00b896b0`):

- It reads stars from bundle slot `+0x3c`, adjusted by a player-side helper `00be8980`, and coins from slot `+0x50`. If stars are above zero and coins are zero it uses `RewardPatternOne` with the star count; coins only uses `RewardPatternOne` with the coin count; both use `RewardPatternTwo`; both zero skips the line. (Read from the pseudocode of `00b89ca0` in this pass.)
- Item entries call the item-notification router. Kind `0x10` (16) looks up the item definition in the client's definition table and fires `CallHandler:ShowMysteryChestReward`, which opens the reward panel; any other kind forwards a generic item-received event (`ItemReceived`, six second display, `00b896b0`).
- For reward kinds 4 and 5, `00b8a640` looks up an actor by the id pair at bundle `+0x80` and applies a float limit read from a constant before queuing the event; what that delay is for is not resolved.

**These packets show rewards; they do not grant them.** Spendable coins, XP and rank arrive in the client-update family (38/19 coin count, 38/14 profile experience, 38/18 rank; see [[Items and Inventory]] and [[Quests and Objectives]]), and items arrive as item adds (38/2). The client's own reward code never writes a wallet or inventory value (client; S0357 and S0358 reach the same reading). A server that sends a bundle and also the grants must make sure the two happen once.

#### Reward family (opcode 50)

The dispatcher `00b8a640` (called from `009471a0`) reads `u16` family 50 and `u8` subtype with `00b89190`. Four subtypes are handled (client, S0357):

| Sub | Class | What the client does |
|---|---|---|
| 1 | `RewardBundlePacket` (ctor `00b88c40`, reader `00b8a300`) | Reads a bundle (success byte, then the layout above), builds a `PlayerRewardEvent` and queues it. The player sees the toast, item notices and chest panel. |
| 2 | `RewardNonBundledItem` (ctor `00b88d10`, reader `00b89590`/`00b891f0`) | Five dwords (the third and fourth form an 8-byte pair). Calls the item-notification router directly with notification type 8. The class pairing is `likely`: it rests on field offsets, not RTTI. |
| 6 | `RewardPacketNonMemberRewardDeferred` (ctor `00b88e10`, reader `00b89600`) | Three dwords. The second is added to the cached star total (`+0x40`) and the third to the cached coin total (`+0x3c`); the Lua setters `CallHandler:SetCurrentXpBank` and `SetCurrentCoinBank` receive the **running totals**. Repeating the packet adds again. The first dword is unused by this handler. |
| 7 | `RewardPacketNonMemberRewardGranted` (ctor `00b893a0`, reader `00b89c00`) | One dword (granted coins), then a counted list of two-dword rows. The cached totals are cleared, the second dword of every row is summed, and if coins or the sum is positive `CallHandler:ShowXpCoinBankReward(stars, coins)` opens the membership panel. The first dword of each row is unresolved. |

The deferred-bank buttons only display totals. They do not credit spendable coins (client, S0357 and S0358).

#### The Claim window

The Claim system is a request-and-answer exchange on opcode 108 (`u16` opcode, `u16` subtype). Its flow (client, S0285):

1. **List.** When the Claim window opens and the list-valid byte (`+9`) is clear, the client sends **108/1** (`GetAllAvailableClaimItems`, `00a04a30`) if it has a world connection. The answer is **108/2** (`00a05390`): a bool, then if true a counted array of claim items. A false flag shows a failure message; a good parse sets the valid byte and pushes every item to the UI (`00a02d30`).
2. **Claim.** Choosing Claim calls `SendClaimItemsRequestForOneId` (`00a04fd0`) or the batch senders (`00a05130`, `00a052e0`). The request is **108/3**: a u32 tracker id and a counted array of 8-byte claim ids. The client then records a **tracker** (type 1, state 1) in an 8-bucket map. The answer **108/4** (`00a05490`) carries the tracker id, a bool and the claimed items. Failure logs "Claim item request failed for 1 or more items." and fires `OnEquipClaimItem` with 0; success asks for the item definitions.
3. **Definitions.** **108/5** (`00a04c10`) carries a tracker id and the item ids to describe; the tracker moves to state 2. The answer **108/6** (`00a041b0`) is accepted only for a tracker in state 2 and carries the item content (0x230-byte item records read by the same item readers as inventory). By tracker type, type 1 builds an `ItemRecord` list and fires `OnEquipClaimItem` (the equip-on-claim path that reaches `00a1ec00`, see [[Items and Inventory]]), type 2 fires `OnClaimItemContents`, and type 3 pushes the item into the tooltip data source and decrements the pending counter. The labels of the two UI entry points `00a04ea0` (type 2) and `00a04f40` (type 3) say "contents" and "equip"; that naming does not match the handler's type-3 branch and is marked speculative.
4. **Events.** The UI is told through the Lua handler `HandlerClaimSystem` with events `OnClaimItems`, `OnEquipClaimItem` and `OnClaimItemContents` (`00a02ef0`).

Claim items are things like "Extra Large Tournament Winnings" or a pending gift item (video `jfcerhMCY6g/00042`). The code-entry half of the window ("Redeem a Code!") is **not** on the game gateway: the redemption request goes through the store layer (see [[Coin Store and Purchases]]). The game connection only receives a **promotional** packet afterwards: opcode **114**, subtype 1 (`KeyCodeRedemptionNotification`, parser `00a08d70`) and subtype 2 (`PromotionalBundleData`, parser `00a08b10`), both dispatched by `00a08e60`. Subtype 1 is described in the code as a loot-card or key-code redemption result; whether it signals success, failure or a particular reward is not established.

#### Collections

There are two unrelated collection systems in the client. The Trading Card Game's `Engine::Collection*` and `FRTWCollection*` classes (most of the 172 `engine.collection` and `ui.collection` functions on this page) manage cards, packs, tickets and redemption for the card client and are explained in [[Trading Card Game]]. The rest of this section is the **world collections** feature: sets of items or places the player gathers across the zones.

- **Categories** come from `CollectionCategories.txt` (id, name text id, icon id). They group the browser.
- **Definitions** (collection id, name, description, image, entry count, tint, category, and up to eight "collected" flags) are **not shipped**. The Lua data source `BaseClient.CollectionData` has 17 columns: id, name, description, image id, two unused, entry count, tint, category id, then `EntryCollected1` to `EntryCollected8` (client Lua). They arrive from the server.
- **Start.** Family 38 subtype 8 (`CollectionStart`) carries a collection record. `HandleClientUpdate` reads a header (two `i16`), a length-prefixed blob, and inside the blob four dwords (the first, at record `+0x9c`, is the key), an 8-byte pair, a dword at `+0xb8`, a full **reward bundle** (the collection's completion reward), and a counted list of keyed seven-dword records (`009099d0`, `008fb360`, `008dacc0`). If the key is new the record is inserted into the 50-bucket table and, when the record is eligible (`00992ad0`) and a Lua owner exists, `NotifyCollectionStarted` (`00a93c80`) posts the "StartCollection" banner. A duplicate key discards the new record.
- **Add entry.** Subtype 10 (`CollectionAddEntry`) carries a header, one dword (probably the collection key), an 8-byte pair, four dwords and a bool: 33 bytes with its four-byte header (`008d8570`). The handler finds the collection by key, replaces or adds the entry (`009939a0`), posts `NotifyCollectionUpdated` (`UpdateCollection` banner) and, if every child is now marked collected (`00992b10`), posts `NotifyCollectionCompleted` (`CompleteCollection` banner). The four dwords line up with the reward entry `CollectionEntryAdd` (type, parm1, parm2, parm3); the match is by shape (speculative).
- **Remove.** Subtype 9 (`CollectionRemove`, ctor `0089da00`) has a constructor in the shards and a case in the dispatcher whose body was not read; subtype 11 (`CollectionRemoveEntry`) carries a header and two dwords (`008d8670`) and removes an entry (`009934a0`).
- **Via a reward bundle.** Two entry types add to collections from a bundle: `CollectionAdd` (type 10: collection id) and `CollectionEntryAdd` (type 0xb: type, parm1 to parm3). Which path the original server used for which collection is unknown.
- **Screens.** The Lua `CollectionsBrowser` and `CollectionDetails` show a collection and its entries; `CollectionHandler.Show` sets a category filter (`Ui.SetCollectionFilterByCategory`, `00992e60`). Three tutorial dialogs ("Collect1" to "Collect3") and first-time events `FtesCollections` and `FtesCollectionFound` teach it (client Lua, `FirstTimeEvents.txt`). Player profile and inspect screens show "collections completed" (`PlayerInfo_DS.CollectionsCompleted`, `PlayerInspectDS.CollectionCount`).
- **How a collection is triggered.** Exploration collections activate when the player walks into places; item collections come from sparkling spawn objects (wiki). Which client code reports a visit or a pickup, and what the client sends, was not traced. Those triggers are probably server-side zone and spawn logic (R).

#### Achievements and trophies

- **Data.** Category and trophy shells come from two resource files (below). The achievement records themselves (title, description, time started and finished, completion, objectives, members-only flag, icon, trophy, points, tint, category, reward id and message) come from the server. The Lua column list is `AchievementsDSColumn`: Id, Title, Description, TimeStarted, TimeFinished, Complete, ObjectiveCount, MembersOnly, IconId, TrophyId, TrophyName, TrophyDescription, Points, TintAlias, Category, rewardID, rewardMessage (17 columns, client Lua; the native column names agree, `00ce37d0`).
- **Objectives.** Each achievement has objectives shaped like quest objectives (Name, Description, Status, Completed, Id). The family reuses the quest record shapes: the add packet embeds a record built by `0091c9f0` and the objective-added packet one built by `0091c8b0`, both of which carry a **reward bundle** (S0242, S0364).
- **Packets.** Family **150 (`0x96`)**, `BaseAchievementPacket` (`00ba97d0`). The dispatcher `00badec0` has cases 2 to 8 (read from its pseudocode): 2 `AchievementAdd`, 3 `AchievementComplete` (two dwords and a byte), 4 `AchievementInitialize` (a list of achievements, the initial load), 5 `AchievementObjectiveAdded`, 6 `AchievementObjectiveActivated` (three dwords and a byte), 7 `AchievementObjectiveUpdate` (four dwords), 8 `AchievementObjectiveComplete` (three dwords, one more, a byte). Field meanings and widths beyond what the constructors store were not decoded (S0364, S0365).
- **Completion banner.** `NotifyAchievementCompleted` (`00a95940`) formats "AchievementCompleted" and posts `CompleteAchievement` to the Lua `NotificationHandler` with colour, points, name, icon id and description. The banner slides in for three seconds, only if the main HUD is shown; others queue (client Lua).
- **Window.** `AchievementsHandler` (4 items per page): category tree from `BaseClient.Achievements.Categories` (an "All Achievements" root), a summary page from `.Summary` (Id, Name, Total, Completed, Percentage) with the overall counts, the five most recent from `.MostRecent`, objectives from `.Objectives`. It is opened from the main menu and the profile's "GotoAchievements" action. Lua calls into the native module `Achievement` (`SelectAchievement`, `SelectCategory`, `SelectInspectAchievement`, `SelectInspectCategory`; registrar `00c09de0`). A total of points is sent to the Scaleform side by the summary source (`00ce6800`, speculative).
- **Inspect.** `BaseClient.InspectTarget.Achievements` lets the profile of another player show their achievements and points (`PlayerInspectDS.AchievementPoints`; S0365).
- **Trophy tooltip.** A "Trophy" tooltip kind (`00a78e80`, seven rows) resolves a trophy record by id. The minigame score screen also shows a minigame's trophies (`BaseClient.MiniGame.Trophies`: Id, Name, Description, Icon ID; `00c47680`), which is the same trophy idea in another place (see [[Minigame Framework]]).
- **Reward entry.** A bundle entry of type 0x13 (`Trophy`) awards a trophy by id (value in the base field `+0x24`).

#### Titles

- **Packets.** Family **152 (`0x98`)**, `BasePlayerTitlePacket` (`00b49140`, `00b491a0`). Header is a `u16` and a `u32` (`00b49e00`). The receive switch `HandlePlayerTitlePacket` (`00b4a3a0`) handles: **1 Add** (reader `00b4a280`: four dwords after the header, inserted into the 32-bucket title map if the key is absent), **2 Remove** (a key), **3 UpdateAll** (a list applied by `0090a890`). Client to server: **4 RequestSelect** (`SendPlayerTitleRequestSelect` `00b4a880`: one u32 title id, called from the Lua binding `SelectPlayerTitle` `00c18c80`, a speculative label) and **5 RefreshRequest** (`00b4a910`, no body). Subtype 7 (`PlayerTitleSNSUpdate`, `00bb6ba0`) belongs to the China social-network integration (list of ints).
- **Other players.** A player-update message `PlayerUpdatePacketPlayerTitle` (family 35, subtype `0x28`, ctor `008b50a0`) exists for titles on other characters; its body was not read. Name templates `PlayerTitleFormatPrefix`, `PlayerTitleFormatSuffix` and `PlayerTitleFormatAlias` decide where a title goes around a name (`0097c390`, likely).
- **Data source.** `BaseClient.PlayerTitles`: per the Lua, column 0 is the title id, column 3 a "selected" flag and column 4 the display text.
- **Reward text.** "You receive the title, [*item*]!" is a locale string, so a title can be handed out as part of a reward line (client locale). The bundle entry type for that is not identified.
- **The picker.** In `AboutPlayer:Show` the title list is requested (`Ui.RequestPlayerTitles`) and filled only inside the `Ui.IsChinaEnvironment()` branch, next to the SNS "about me" fields (client Lua). `DiplomacyTitles.txt` and `DiplomacyRanks.txt` ship with a header row and no data.

#### Prize wheels and the daily wheel

The loot wheel is part of the minigame results screen: the preview entries and the landing item arrive in minigame packets 39/68 and 39/45, the client sends a "stopped" report 39/46, and the Lua picks the stop index (client; full details and the exact packet fields are in [[Minigame Framework]]). The reward entry `WheelSpin` (bundle type 0x12, value in `+0x24`) is the other way to hand a wheel spin over, presumably granting one spin on a named wheel (the value is an id; its meaning is not traced). The Daily Rewards wheel is minigame type 22 and is started by interaction `26/11` ([[Minigame Framework]]). A locale string says the wheel cannot be spun in "unhealthy gaming time", when in-game gains are zero (client locale; a China fatigue rule, see Open questions).

### Data and formats

**Client resources (read by the client, not sent).** Read in this pass from the shipped baseline's `Resources` folder (`lab/runs/client-baseline-*/client/Resources`). The `*_ID` text columns are locale ids; the locale file is keyed by hashes, so these names were not resolved in this pass except where `CodeStringMappings.txt` names the message.

| File | Contents | Source |
|---|---|---|
| `AchievementCategories.txt` | Caret-separated `ID^PARENT_CATEGORY_ID^NAME_ID^DESCRIPTION_ID^IMAGE_SET_ID^IMAGE_TINT_ID^TYPE^TYPE_DATA_ID^CONTENT_ID^`. 23 rows, ids 1 to 10, 13, 16 to 27. Twelve are roots with parent 0 (ids 1 to 8 and 20 to 23); ids 9, 10, 13, 16, 17, 18 and 19 sit under roots 3, 2, 2, 7, 5, 5 and 1; ids 24 to 27 sit under 23. All `TYPE` and `TYPE_DATA_ID` are 0, all descriptions are 0, `CONTENT_ID` is 93 or 0. Category 3 uses name id 39, which `CodeStringMappings.txt` names `Collections`. Loader `0104d880`. | client |
| `AchievementTrophies.txt` | `ID^NAME_ID^DESCRIPTION_ID^IMAGE_SET_ID^GAIN_COMPOSITE_ID^REWARD_ID^POINT_VALUE^IMAGE_TINT_ID^CONTENT_ID^`. 10 rows. Ids 1 to 4: image sets 1 to 4, **0 points**, content 93. Ids 5 to 9: image sets 1 to 5, points **5, 10, 15, 25, 40**, content 94. Id 10: image set 3, 0 points, content 94. `REWARD_ID` is 0 in all rows. Gain composite ids 15895 to 15898 name the pickup effects. Loader `0104f870`. | client |
| `CollectionCategories.txt` | `ID^NAME_ID^ICON_ID^`. 10 rows (ids 1 to 10). Loader `009fd4e0`; the Lua source `BaseClient.CollectionCategory` has the same three columns. | client |
| `DiplomacyRanks.txt`, `DiplomacyTitles.txt` | Header row only (`TITLE_ID^REQUIRED_RANK^REQUIRED_RANK_COUNT^TITLE_STRING_ID^ICON_ID` for the second). No shipped data. | client |
| `CodeStringMappings.txt` | Message names used above: `RewardPatternOne` 2, `RewardPatternTwo` 3, `Rewards` 3486, `Achievements` 96, `AllAchievements` 3158, `AchievementsUI` 3159, `AchievementUI` 3160, `RecentAchievements` 3161, `AchievementPoints` 442222, `Collection` 351, `Collections` 39, `CollectionStarted` 41572, `CollectionCompleted` 23023, `CollectionBrowserCollected` 9379, `Collect1` to `Collect3` 38809 to 38811, `EarnedTrophyTitle` 416893, `Accept_Claim` 436332, `Deny_Claim` 436333, `ClaimOncePerAccountWarningString` 426802, `DailyBundle` 427582. | client |
| `FirstTimeEvents.txt` | Event 21 `FtesCollections`, event 22 `FtesCollectionFound` (first-time tutorials). | client |

The trophy points (5, 10, 15, 25) match the wiki's Bronze, Silver, Gold and Platinum values, and the zero-point rows match its "Wonderous Trophy - 0 Points" (wiki). The 40-point row has no wiki counterpart.

**The reward bundle on the wire** (reader `008e7930`; all integers little-endian; offsets are the in-memory field in brackets; widths and order are client-read, field names are only given where a getter or debug text supports them).

| Order | Width | Field | Notes |
|---|---|---|---|
| 1 | u8 | success flag `[+0x74]` | Copied into every entry as `[+0x41]` (see above). |
| 2 | i32 | coins `[+0x50]` | `GetCoinRewardAmount` `0091c2e0`. |
| 3 | i32 | stars / XP `[+0x48]` | `GetXpRewardAmount` `0091c290`. |
| 4 | i32 | unresolved accumulator `[+0x4c]` | `Add_0x4c` exists; no use found. |
| 5 | i32 | reward kind `[+0x54]` | `GetRewardKind` `0091c380`. 16 = mystery chest panel. |
| 6, 7 | i32, i32 | `[+0x6c]`, `[+0x70]` | Unresolved. |
| 8 | f32 | `[+0x78]` | NaN sets the reader's NaN flag; a multiplier by shape, unresolved. |
| 9, 10 | i32, i32 | `[+0x5c]`, `[+0x60]` | Unresolved. |
| 11, 12 | 2 x (u32, u32) | id pairs `[+0x80]`, `[+0x88]` | Set through vtable slots `+0x30`, `+0x34`. Looked up as an actor id pair for kinds 4 and 5. |
| 13, 14 | i32, i32 | `[+0x64]`, `[+0x68]` | Unresolved. |
| 15 | i32 | entry count | Loop bound; loop stops early on a reader error. |
| per entry | u32 type id, then body | | See the entry table. |
| last | i32 | `[+0x58]` | Unresolved. |

An empty bundle is therefore 1 + 9 x 4 + 16 + 8 + 4 + 4 = **69 bytes** (matches the size the quest guide gives). Family 50 subtype 1 adds the three-byte header.

**The entry base body** (reader `0103c460`, writer `0103d0d0`; every entry starts with this, after its type id):

| Width | Field |
|---|---|
| u8 | `[+0x0c]` bool |
| u32, u32 | `[+0x04]`, `[+0x08]` |
| u32 x 4 | `[+0x10]`, `[+0x20]`, `[+0x24]`, `[+0x28]` |
| i32 length, bytes | block stored as a string at `[+0x2c]` (negative or past-end length is an error) |
| u32 | `[+0x3c]` |
| u8 | `[+0x40]` bool |

An entry with an empty string is 34 bytes; with its type id, 38. `[+0x24]` carries the main id for most types (the debug text prints it as the item, profile, token, trophy or wheel value).

**Entry types** (type id from `CreateEntry` `0103c070`; ids 2, 4, 5, 9, 0x10 and 0x15 return no object):

| Id | Class (ctor) | Extra body after the base | Debug text (what the fields are called) |
|---|---|---|---|
| 1 | Item (`0103a4f0`) | u32 `[+0x44]` only if `[+0x41]` | `Item=%d, Item Guid=%llu` (`[+0x24]`; setter also takes `[+0x28]`) |
| 3 | Experience (`0103a680`) | none | `PROFILE=%d` (`[+0x24]`) |
| 6 | QuestAdd (`0103a780`) | u32 `[+0x44]` | `QuestID=%d` |
| 7 | ProfileAdd (`0103a800`) | u32 `[+0x44]` | `PROFILE=%d` |
| 8 | AbilityLine (`0103adb0`) | u32 `[+0x44]` | `AbilityLineID=%d` |
| 10 (0xa) | CollectionAdd (`0103a8f0`) | u32 `[+0x44]` | `CollectionID=%d` |
| 11 (0xb) | CollectionEntryAdd (`0103a9c0`) | 4 x u32 `[+0x44..+0x50]` | `CollectionEntryType`, `Parm1`, `Parm2`, `Parm3` |
| 12 (0xc) | Token (`0103aaa0`) | none | `TokenId=%d` (`[+0x24]`) |
| 13 (0xd) | PetTrickExperience (`0103ab90`) | 2 x u32 `[+0x44]`, `[+0x48]` | `TrickID=%d` |
| 14 (0xe) | Recipe (`0103ac70`) | u32 `[+0x44]` | `RecipeID=%d` |
| 15 (0xf) | ZoneFlag (`0103ad10`) | length-prefixed block, u32 `[+0x54]`, u32 `[+0x58]` | `ZoneFlag=%s, Value=%d, Type=%d` |
| 17 (0x11) | CharacterFlag (`0103b350`) | length-prefixed block, u32 `[+0x78]`, u32 `[+0x7c]`, u8 `[+0x80]`, u32 `[+0x84]`, u8 `[+0x88]` | `Key=%s, Operation=%d, Amount=%d, Persistent=%d, CountTimeOffline=%d` |
| 18 (0x12) | WheelSpin (`0103ae50`) | none | `WheelSpin=%d` |
| 19 (0x13) | Trophy (`0103b020`) | none | `Trophy=%d` |
| 20 (0x14) | ClientExitUrl (`0103af40`) | none | `ClientExitUrl=%s` (`[+0x30]`) |
| 22 (0x16) | FactoryExperience (`0103a700`) | none | `Factory Type=%s` (from `[+0x2c]` to `[+0x38]`) |
| 23 (0x17) | Chest (`0103a5d0`) | u32 `[+0x44]` only if `[+0x41]` | `Chest: item id=%d, Item Guid=%llu` |

The class names are client RTTI; the type-id mapping is read from the constructor calls in `0103c070` in this pass. The per-entry ReadBody labels for `QuestAdd`, `CollectionEntryAdd`, `PetTrickExperience`, `CharacterFlag` and `ZoneFlag` are `confirmed` or `likely`; the others are `unknown` labels with the same shape. `PetTrickExperience` ties to [[Pets]], `Recipe` to crafting, `ZoneFlag` and `CharacterFlag` to scripted state, `FactoryExperience` to the factory minigame, and `AbilityLine` to [[Combat and Abilities]].

**Claim item on the wire** (reader `00a040e0`, label `likely`): a counted array of 8-byte ids, two u32 (`[+0x144]`, `[+0x148]`), a further field read by `008caaa0` (two u32), a counted list of entries each ending in a dword (the 0x18-byte list at `[+0x70]`) and 8 bytes (`[+0x150]`, `[+0x154]`). Names and order of the middle part are not firm. The claim response is: 108/2 `u16 u16`, bool, array; 108/4 `u16 u16`, u32 tracker id, bool, array; 108/6 `u16 u16`, u32 tracker id, bool, and, if true, a length-prefixed blob of item content (`00a02a60`, `00a049b0`, `00a02ac0`).

**Data sources** (what the Lua reads): `BaseClient.Achievements` (17 columns above), `.Achievements.Categories` (a tree; node columns Id, Name, Description, Image Set Id, Tint Id, Type, TypeDataId; the Lua names six), `.Achievements.Objectives` (Name, Description, Status, Completed, NotUsed, Id), `.Achievements.Summary` (Id, Name, Total, Completed, Percentage), `.Achievements.MostRecent`, `.Achievements.Rewards`; `BaseClient.CollectionData` (17 columns above), `.CollectionEntriesData`, `.CollectionData.Reward(.Entries)`, `.CollectionCategory` (CollectionCategoryID, NAME, ICON_ID); `BaseClient.PlayerTitles`; the claim list is shown through `HandlerClaimSystem` and the `TooltipData` source (`ClaimItem` tooltip, eight rows, `00a77680`).

### Network

**Envelope.** Reward, claim, collection, achievement and title traffic arrives as inner packets of the world connection; header widths differ by family. Directions are proven when a sender builds the packet (C>S) or a dispatcher case parses it (S>C). No packet of these families has been captured, so framing and byte order are read from the client, not verified. Strings and blobs are an `i32` length followed by bytes.

| Fam/sub | Name | Dir | Header | Body | Handler or sender |
|---|---|---|---|---|---|
| 50/1 | RewardBundle | S>C | `u16`, `u8` | Reward bundle (above) | `00b8a640`, `00b8a300`, `008e7930` |
| 50/2 | RewardNonBundledItem | S>C | `u16`, `u8` | 5 x u32 (third and fourth an 8-byte pair) | `00b89590`, `00b891f0` |
| 50/6 | NonMemberRewardDeferred | S>C | `u16`, `u8` | 3 x u32; stars delta is the second, coins delta the third | `00b89600`, `00b892b0` |
| 50/7 | NonMemberRewardGranted | S>C | `u16`, `u8` | u32 coins, i32 count, rows of 2 x u32 | `00b89c00`, `00b894a0`, `00b89340` |
| 38/8 | CollectionStart | S>C | 2 x `i16` | length-prefixed blob: 4 x u32, pair, u32, bundle, keyed record list | `009392c0` case 8, `009099d0`, `008fb360` |
| 38/9 | CollectionRemove | S>C | 2 x `i16` | not read in the shards | ctor `0089da00` |
| 38/10 | CollectionAddEntry | S>C | 2 x `i16` | u32, pair, 4 x u32, bool (33 bytes with header) | `009392c0` case 10, `008d8570` |
| 38/11 | CollectionRemoveEntry | S>C | 2 x `i16` | 2 x u32 | `009392c0` case 11, `008d8670` |
| 38/14, 38/18, 38/19 | Profile experience, rank, coin count | S>C | as family 38 | the actual grants | `009392c0`, see [[Items and Inventory]] |
| 150/2 to 150/8 | Achievement add, complete, initialize, objective added, activated, update, complete | S>C | not traced | see Achievements and trophies | `00badec0` |
| 152/1, 2, 3 | Title add, remove, update all | S>C | `u16`, `u32` | 4 x u32; key; entry list | `00b4a3a0` |
| 152/4 | Title select | C>S | `u16`, `u32` | u32 title id | `00b4a880` |
| 152/5 | Title refresh | C>S | `u16`, `u32` | none | `00b4a910` |
| 152/7 | Title SNS update | C>S (China social) | `u16`, `u32` | list of ints | `00bb6ba0`, `00bb85d0` |
| 35/0x28 | PlayerUpdate player title | S>C | | not read | ctor `008b50a0` |
| 108/1 | GetAllAvailableClaimItems | C>S | `u16 u16` | none | `00a04a30`, `009ff3c0` |
| 108/2 | GetAllAvailableClaimItemsResponse | S>C | `u16 u16` | bool, claim item array | `00a05390`, `00a04950` |
| 108/3 | ClaimItemsRequest | C>S | `u16 u16` | u32 tracker id, counted 8-byte ids | `00a04fd0`, `00a05130`, `00a052e0`, writer `00a045d0` |
| 108/4 | ClaimItemsResponse | S>C | `u16 u16` | u32 tracker id, bool, claim item array | `00a05490`, `00a049b0` |
| 108/5 | ClaimItemsItemDefinitionsRequest | C>S | `u16 u16` | u32 tracker id, counted ids | `00a04c10`, writer `00a04520` |
| 108/6 | ClaimItemsItemDefinitionsResponse | S>C | `u16 u16` | u32 tracker id, bool, blob | `00a041b0`, `00a02ac0` |
| 114/1, 114/2 | KeyCodeRedemptionNotification, PromotionalBundleData | S>C | `u16 u16` | see above | `00a08e60`, `00a08d70`, `00a08b10` |

Claim requests 108/1 and 108/3 are wrapped in the world-tunnel packet (gateway id 6, `0089ae70`); 108/5 in the client-tunnel packet (gateway id 5, `0089adc0`) (client, S0285). Related traffic covered elsewhere: quest end 49/13 carries a bundle ([[Quests and Objectives]]); minigame 39/22, 39/45, 39/46 and 39/68 carry bundles and wheel state ([[Minigame Framework]]); progressive-quest family 207 drives the Mystery Reward Browser ([[Quests and Objectives]]); repeating-activity family 143 may hold daily wheel state ([[Minigame Framework]]); loot event 35/29 is a visual effect only ([[Items and Inventory]]).

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour.

1. **Keep grant and display apart.** One transaction applies the grant (wallet, XP, rank, items) and sends the matching 38/19, 38/14, 38/18 and 38/2 messages; the bundle is for display. Never let a repeated display packet grant again, and never let the bundle's numbers differ from the grant (R).
2. **Write bundles exactly.** Use the 69-byte empty layout; set coins, stars and kind; put real zeros in unresolved fields. For every entry write the type id, the base body (34 bytes minimum) and the type's extra fields. Set the success byte to 1 for a normal grant and include the trailing u32 on `Item` and `Chest` entries when it is 1, or send 0 and omit it; test both against the client (R). Never send type ids 2, 4, 5, 9, 0x10 or 0x15 (the client then skips the body and desynchronises).
3. **Pick the toast.** Coins only: coins > 0, stars 0. Stars only: the reverse. Both: both. For a chest, kind 16. For membership payouts use family 50 subtypes 6 (running deltas) and 7 (grant and clear) (client).
4. **Own all reward content.** No payout table is in the client. Keep per-quest, per-objective, per-minigame, per-chest, per-wheel and per-collection reward rows in our own content store; source amounts from our content pages ([[Content]]) and tag video-sampled numbers as such (R).
5. **Claims.** Keep a per-account list of claim items, each with an 8-byte claim id and item content. Answer 108/1 with 108/2 once per session. On 108/3 validate that every id is open and belongs to the account, claim all or fail the whole request (the client treats "1 or more items" as one failure), grant exactly once, then answer 108/4 with the same tracker id. Answer 108/5 with 108/6 for the same tracker id, only after the client has registered it. Each claim item can be claimed once per account (video, client locale). Do not promise code redemption until the store layer path is traced (R; see [[Coin Store and Purchases]]).
6. **Collections.** Own definitions: collection id, name, description, image, category, entries (up to eight shown), eligibility, completion reward bundle. At login send 38/8 for each collection the player has started, with the entries already found; on a new find send 38/10; on completion send the update that makes every child collected and the completion bundle. Exploration collections trigger from entering zone areas and item collections from clicking spawn objects (wiki); both are server logic here (R). Keep the Adventurer XP for exploration and the rare-versus-common rules from the wiki as design input (R). Do not use the card client's `Collection*` handlers for this.
7. **Achievements.** Own the list. At login send 150/4 with every achievement and its objectives; send 150/2, 150/5 to 150/8 and 150/3 as progress changes. Award the trophy tier, points, title and item from our own table; the shipped `REWARD_ID` is empty, so the reward column is our design (R). Update `PlayerInspect` data for other viewers. Use the wiki categories (Jobs, Housing, Minigames, Combat, Quests, Items, TCG, Holidays and Events) as the content outline, and the shipped category tree shape as the container; the locale ids for names need a locale lookup we do not yet have.
8. **Titles.** Keep owned titles and the selected one per character. Send 152/1 on award, 152/3 at login, handle 152/4 by validating ownership and broadcasting the choice to other players through the player update (35/0x28, body not yet read) (R). Source titles from the wiki list (job level 20, questline, VIP, five million coins) and our own table.
9. **Wheels.** Follow [[Minigame Framework]]: decide the prize on the server, send the preview and landing item before the spin, and grant exactly once. The `WheelSpin` entry grants a spin; define the wheel ids ourselves (R). Daily wheel: one spin per account per day, limit enforced by the server (R).
10. **Promotional traffic.** Stub 114 until the store-side redemption is traced.
11. **Errors.** Every refused claim, award or title selection must end in a visible reply or an explicit failure flag, never silence.

### Video and wiki evidence

**Video frames reviewed:** frames showing banners, reward lines, the Claim window, collections and the main menu (names over characters omitted).

- **Reward lines (video).** "You receive 1 Seeker's Midriff Shirt" with an "Adventurer Level Up!" banner (`nuSlsBsKmrg/00118`). Quest reward lists show "Mystery Gift" on seasonal quests (`yWuHe_jYyc4/00264`, `yWuHe_jYyc4/00272`) and item rewards such as a seed packet on a shopping-list game (`ljDRabG8Akk/00434`). Result panels show the "YOU WIN! ... Congratulations! You won" layout with a coin line, an item or a mystery pack (`f3ZJ9i5YnaI/00198`, `qCG_K2piQgE/00176`, `fktAs_GRrjM/00504`, `dOauCv91EFw/00246`).
- **Claim window (video).** A "Redeem a Code!" panel for in-game items, membership packages and Trading Card Game virtual reward cards, with a code field, "Pending Claims: (2)" and a note that each claim item can only be claimed once per account (`4h2eVoje3Eg/00290`, `jfcerhMCY6g/00042`, `jfcerhMCY6g/00044`, `1LjCvhboCkQ/00192`).
- **Collections (video).** A "Collection complete!" banner naming an exploration collection ("Merry Vale Rare Exploration", `CXeswcNcDTM/00076`; "Merry Vale Elite Exploration", `LgrhYq2Hs1I/00040`; "Major Warpstones", `-itStdznbPo/00060`) and the first-time text "You've updated a collection! You can find collection items by exploring the world. You get some collections automatically by exploring a new part of the world. Completing exploration collections levels up your adventurer job." (`nuSlsBsKmrg/00122`, `00124`; the same text is in the client locale). The main menu lists Achievements, Hero's Journal, Jobs, Stuff, Collections, Profile, Pets and more (`WOrM69vQT8A/00264`, `lPWru_CDw5I/00330`).
- **Achievements (video).** No frame shows an achievement banner or window in the reviewed set; only the menu entry. The achievement text comes from the client locale and the wiki.
- **Collections (wiki).** Collections are items found around the world by clicking sparkling objects, or sets of places to discover. Types: Exploration, Rare and Elite Exploration (sparkling coins; 22 coin collections over nine realms), Explorer (collections of collections), Item (common and rare spawns), Meta, Job (spawns only for a job in use) and Cascading. Finding a new item gives Adventurer experience; completing a collection may give a reward; some are members-only.
- **Achievements (wiki).** Added 14 February 2013. Rewards are achievement points, a title or an item. Categories on the page: Jobs, Housing, Minigames, Combat, Quests, Items, TCG, Holidays and Events. Trophy tiers: Bronze 5 points, Silver 10, Gold 15, Platinum 25, "Wonderous Trophy" 0. Examples: level 4, 8, 12, 16 and 20 in each job (the last gives a title); housing item counts; completing a questline gives a Gold Trophy and a title; earning 500,000, 1,000,000 or 5,000,000 coins through rewards. The client trophy points agree with these tiers.
- **Titles (wiki).** Titles come from achievements, VIP levels and questlines: five VIP titles, a title for level 20 in each job, questline titles, and "Millionaire" for five million coins.
- **Daily and event wheels (wiki, client locale).** One spin a day at the daily wheel; seasonal wheels during events; the Daily Rewards text is also in the client locale.

### Open questions and contradictions

- **No captured reward traffic.** The bundle layout is read from the client's reader; no original server frame exists. Header widths for family 150, the unnamed middle fields of a claim item, and the exact family 38 collection-record fields are not decoded.
- **Unresolved bundle fields.** Order 4 (`+0x4c`), 6, 7, 9, 10, 13, 14, the float at `+0x78`, the id pairs, the last dword (`+0x58`) and the reward kinds other than 16 (kinds 4 and 5 are special-cased in `00b8a640`) have no meaning yet. Whether the first byte also gates anything besides the `Item` and `Chest` trailing field is unknown.
- **Entry types without presentation.** `00b89ca0` presents only `Item`, `Chest` and `Token`. Whether `Experience`, `ProfileAdd`, `QuestAdd`, `Recipe`, `Trophy`, `WheelSpin`, `CollectionAdd`, `ZoneFlag`, `CharacterFlag`, `ClientExitUrl` and `FactoryExperience` act on client state when they arrive, or are only data for a script, was not traced. Their mapped functions are readers, writers and text formatters only.
- **Two collection paths.** A collection can be started by 38/8 or by a `CollectionAdd` reward entry. How the original server used both is unknown. The completion test in the 38/10 handler (every child flagged) is read from helper `00992b10` (`likely`).
- **Collection definitions.** Names, entry counts and images are not in the client. The category names are locale ids that this pass could not resolve (the locale file is hashed). The wiki's collection types cannot be mapped to client ids.
- **`DiplomacyTitles.txt` and `DiplomacyRanks.txt`** ship empty. Whether they are the dead remains of a diplomacy-rank feature that titles replaced is unknown.
- **Title picker.** The only Lua that fills the title list is in the China branch of the profile window. Whether the global client let a player choose a title, and from where, is not shown by the client text found; the wiki and the "You receive the title" string say titles existed. The wiki says achievements arrived on 14 February 2013 and the client locale announces them, so the achievement and title code is in this build.
- **Trophy points.** The 40-point row (image set 5) and the three zero-point rows 1 to 4 (content id 93) have no wiki twin beyond "Wonderous Trophy". `CONTENT_ID` 93 versus 94 (and 0 in the category file) is unexplained. `GAIN_COMPOSITE_ID` is the pickup effect (inferred).
- **Achievement packet semantics.** The constructors store dwords and bytes; what each means is not decoded. The shard labelled the dispatcher cases 2 to 6 in one place and 2 to 8 in another; the pseudocode of `00badec0` has seven cases (2 to 8).
- **Claim tracker types.** The handler (`00a041b0`) names type 1 equip, type 2 contents, type 3 tooltip; the UI entry-point labels for `00a04ea0` and `00a04f40` read the other way round. Treat the labels as unreliable until traced. The meaning of the first claim reply field (stored at packet `+0x10`) is unknown.
- **Key-code notification.** Whether 114/1 reports success, failure or a reward is not known.
- **Fatigue rule.** Two locale strings describe "unhealthy" and "fatigue" gaming time in which gains drop to 0 or 50 percent and quests and wheel spins are blocked. They belong to a regional rule; no client code that applies it was traced, and whether the server should model it is a policy question (R).
- **Recheck list.** None of the addresses cited above is on the known-wrong list (`lab/native-map/context/recheck-pending.txt`) at the time of writing; the generated tables below carry their own labels.
- **Speculative labels.** The `DeckService` and `DeckServiceBase` forwarding stubs, the `Array`, `HashList` and `List` template helpers for claim data, the achievement data-source members (`00ce5170` to `00ce6e50`), and `ClientRewardManager::FindReward` rest on shape alone.

### Key functions

| Address | Role |
|---|---|
| `00b8a640`, `009471a0` | Reward family dispatcher (confirmed) and the client packet switch that calls it |
| `00b8a300`, `00b89590`, `00b89600`, `00b89c00` | Readers for reward subtypes 1, 2, 6, 7 |
| `008e7930` | Shared reward-bundle body reader (read from disassembly this pass) |
| `0103c070` | `CreateEntry`: entry type id to class (confirmed from the constructor calls) |
| `0103c460`, `0103d0d0` | Entry base reader and writer |
| `0091c110`, `0091c290`, `0091c2e0`, `0091c380`, `0091c5e0` | Bundle ctor, stars, coins, kind, add entry |
| `00b89ca0`, `00b899f0`, `00b896b0`, `00b8a5f0` | Presentation, item router, item-received notice, reward event delivery |
| `00a04a30`, `00a04fd0`, `00a04c10`, `00a05390`, `00a05490`, `00a041b0`, `00a056a0` | Claim list request, claim request, definitions request, answers and dispatcher |
| `00a08e60`, `00a08d70`, `00a08b10` | Promotional family dispatcher and parsers |
| `009392c0` | Client-update dispatcher (collection subtypes 8 to 11) |
| `009099d0`, `008d8570`, `008d8670`, `008fb360`, `00993a20`, `009939a0`, `009934a0` | Collection record reader, entry readers, keyed list, insert, replace, remove |
| `00a93c80`, `00a93f40`, `00a94380` | Collection started, updated and completed banners |
| `009fd4e0`, `0104d880`, `0104f870` | Load `CollectionCategories.txt`, `AchievementCategories.txt`, `AchievementTrophies.txt` |
| `00badec0`, `00ba97d0`, `00bad510`, `00bad8f0` | Achievement dispatcher, base packet, initialize packet, data-source owner |
| `00a95940`, `00ce37d0`, `00c09de0` | Achievement banner, column names, Lua module registration |
| `00b4a3a0`, `00b4a880`, `00b4a910`, `00cc5930`, `0097c390` | Title dispatcher, select, refresh, data source, name template |
<!-- NARRATIVE END -->

## Mapped functions (generated)

478 accepted functions: 299 confirmed, 78 likely, 37 speculative, 64 unknown. Network-linked (`wire`): 7.

Subsystems: `game.reward` (246), `ui.collection` (129), `engine.collection` (43), `game.collection` (27), `game.achievement` (18), `ui.reward` (15)

### `engine.collection`

**Engine::CollectionService** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013dc9d0` | Engine::CollectionService::SumMatchingCounts | likely | Vtable slot [10] shared by Engine::CollectionService and Engine::FRTWCollectionService (157 bytes, no arguments). Walks the tree of collections from 0139f770 w… |
| `013dca70` | Engine::CollectionService::SumMatchingKeyedCounts | likely | Vtable slot [11] shared by Engine::CollectionService and Engine::FRTWCollectionService (222 bytes, one stack argument). Walks the same collection tree as slot … |

**Engine::DeckService** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013dd700` | Engine::DeckService::ResetOwnedChecked | likely | Vtable slot [8] of Engine::DeckService (96 bytes, thiscall, one std::string argument). Rejects when the owned pointer at +0xc is null or 013ce250 reports read-… |
| `013dd890` | Engine::DeckService::AssignStringAt0xfc | speculative | Vtable slot [24] of Engine::DeckService (23 bytes, ret 4). Adds 0xfc to ECX and calls 00420ff1 (std::string assign_substr) with the first stack argument and th… |
| `013dd6a0` | — | unknown | Vtable slot [7] of Engine::DeckService (95 bytes, thiscall, one argument). Calls 012aab80 on the argument, loads a value with 012aaeb0 and stores it into the o… |
| `013dd760` | — | unknown | Vtable slot [10] of Engine::DeckService (295 bytes, thiscall, one std::string argument). Builds a temporary std::string from the argument (length loop and assi… |
| `013debb0` | — | unknown | Vtable slot [22] of Engine::DeckService (508 bytes, fastcall, result returned through the receiver). Copies the entry list from 012aa280 into a temporary vecto… |

**Engine::DeckServiceBase** (33)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013de860` | Engine::DeckServiceBase::Ctor | confirmed | Fastcall constructor (191 bytes, returns this): calls 01230d10 (base constructor), stores vtable 018e24fc, clears the byte at this+8 and the byte at this+9, ze… |
| `013dea10` | Engine::DeckServiceBase::ScalarDeletingDtor | confirmed | Vtable slot [0] of Engine::DeckServiceBase (30 bytes, ret 4): calls the destructor body 013de920, then if bit 0 of the argument is set calls _free on this and … |
| `013dcb50` | Engine::DeckServiceBase::SetFlag8 | speculative | Vtable slot [4] shared by Engine::DeckServiceBase, DeckService and FRTWDeckService (bytes c6 41 08 01 c3): stores 1 in the byte at this+8. |
| `013dcb60` | Engine::DeckServiceBase::GetFlag8 | speculative | Vtable slot [5] shared by Engine::DeckServiceBase, DeckService and FRTWDeckService (bytes 8a 41 08 c3): returns the byte at this+8. |
| `013dcbe0` | Engine::DeckServiceBase::FlagGatedNotify | speculative | Vtable slot [17] shared by Engine::DeckServiceBase, DeckService and FRTWDeckService (59 bytes). If the first flag argument is non-zero it calls vtable slot [2]… |
| `013dcc20` | Engine::DeckServiceBase::GetFlag9 | speculative | Vtable slot [11] shared by DeckServiceBase, DeckService and FRTWDeckService (bytes 8a 41 09 c3): returns the byte at this+9. |
| `013dcc30` | Engine::DeckServiceBase::NotifyIfOwned_Ce740 | speculative | Vtable slot [18] shared by DeckServiceBase, DeckService and FRTWDeckService (40 bytes, one argument). When the pointer at this+0xc is non-null and 013ce250 ret… |
| `013dcc60` | Engine::DeckServiceBase::NotifyIfOwned_Ce780 | speculative | Vtable slot [19] shared by DeckServiceBase, DeckService and FRTWDeckService (40 bytes, one argument). Same shape as slot [18] (013dcc30) but calls 013ce780 wit… |
| `013dcc90` | Engine::DeckServiceBase::NotifyIfOwned_Ce330 | speculative | Vtable slot [20] shared by DeckServiceBase, DeckService and FRTWDeckService (40 bytes, one argument). Same shape as slot [18] but calls 013ce330 with the argum… |
| `013dccc0` | Engine::DeckServiceBase::NotifyIfNotOwned_Ce2a0 | speculative | Vtable slot [21] shared by DeckServiceBase, DeckService and FRTWDeckService (41 bytes, one argument). When the pointer at this+0xc is null and 013ce250 returns… |
| `013dccf0` | Engine::DeckServiceBase::AddrOfField_fc | speculative | Vtable slot [23] shared by DeckServiceBase, DeckService and FRTWDeckService (bytes 8d 81 fc 00 00 00 c3): returns this+0xfc (lea eax,[ecx+0xfc]). |
| `013dcd00` | Engine::DeckServiceBase::Forward_01438e70 | speculative | Vtable slot [28] shared by DeckServiceBase, DeckService and FRTWDeckService (37 bytes, ret 12). Sets ECX to this+0x50, calls 01435790 (returns ECX+4, i.e. this… |
| `013dcd30` | Engine::DeckServiceBase::Forward_01439170 | speculative | Vtable slot [27] shared by DeckServiceBase, DeckService and FRTWDeckService (37 bytes). Same shape as slot [28] (013dcd00): 01435790 on the this+0x50 sub-objec… |
| `013dcd60` | Engine::DeckServiceBase::Forward_014390b0 | speculative | Vtable slot [26] shared by DeckServiceBase, DeckService and FRTWDeckService (35 bytes). Calls 01435790 on the this+0x50 sub-object, then 014390b0 with the thre… |
| `013dcd90` | Engine::DeckServiceBase::Forward_01438fb0 | speculative | Vtable slot [34] shared by DeckServiceBase, DeckService and FRTWDeckService (35 bytes). Calls 01435790 on the this+0x50 sub-object, then 01438fb0 with the thre… |
| `013dcdc0` | Engine::DeckServiceBase::Forward_014372e0 | speculative | Vtable slot [31] shared by DeckServiceBase, DeckService and FRTWDeckService (35 bytes). Calls 01435790 on the this+0x50 sub-object, then 014372e0 with the thre… |
| `013dcdf0` | Engine::DeckServiceBase::Forward_01438740 | speculative | Vtable slot [30] shared by DeckServiceBase, DeckService and FRTWDeckService (35 bytes). Calls 01435790 on the this+0x50 sub-object, then 01438740 with the thre… |
| `013dce40` | Engine::DeckServiceBase::Forward_014374b0 | speculative | Vtable slot [32] shared by DeckServiceBase, DeckService and FRTWDeckService (20 bytes, no stack arguments). Calls 01435790 and then 014374b0 with the constant … |
| `013dce60` | Engine::DeckServiceBase::InitEntries_0x31d | speculative | Vtable slot [33] shared by DeckServiceBase, DeckService and FRTWDeckService (96 bytes, one stack argument). Calls its own slot [32] (0x80) with the argument, t… |
| `013dcec0` | Engine::DeckServiceBase::Forward_01438a20 | speculative | Vtable slot [36] shared by DeckServiceBase, DeckService and FRTWDeckService (28 bytes, ret 8). Calls 01435790 on the this+0x50 sub-object (returns this+0x54), … |
| `013dcee0` | Engine::DeckServiceBase::TailCall_01437600 | speculative | Vtable slot [37] shared by DeckServiceBase, DeckService and FRTWDeckService (15 bytes). Adds 0x50 to ECX, calls 01435790 (returns ECX+4), then tail-jumps to 01… |
| `013dcf30` | Engine::DeckServiceBase::RebuildFromOwnedPairs | speculative | Vtable slot [6] shared by DeckServiceBase, DeckService and FRTWDeckService (128 bytes, fastcall). Calls 0143b580 with this+0xf0, this+0xf8 and this+0xdc. If th… |
| `013dd080` | Engine::DeckServiceBase::CallSlot16_True0 | speculative | Vtable slot [12] shared by DeckServiceBase, DeckService and FRTWDeckService (95 bytes, SEH-framed, thiscall, one argument). Calls the receiver's vtable slot [1… |
| `013dd0e0` | Engine::DeckServiceBase::CallSlot16_False1 | speculative | Vtable slot [15] shared by DeckServiceBase, DeckService and FRTWDeckService (95 bytes, SEH-framed, thiscall, one argument). Calls the receiver's vtable slot [1… |
| `013dcba0` | — | unknown | Vtable slot [9] shared by Engine::DeckServiceBase, DeckService and FRTWDeckService (59 bytes, no arguments, returns 1). Deletes the object at this+0xc if prese… |
| `013dce20` | — | unknown | Vtable slot [29] shared by DeckServiceBase, DeckService and FRTWDeckService (28 bytes, ret 0). Calls 01435790 with arguments 1 and 3, then 01436f20 with the sa… |
| `013dcfb0` | — | unknown | Vtable slot [13] shared by DeckServiceBase, DeckService and FRTWDeckService (97 bytes, SEH-framed). Calls 014398c0 with its one stack argument and returns its … |
| `013dd020` | — | unknown | Vtable slot [14] shared by DeckServiceBase, DeckService and FRTWDeckService (92 bytes, SEH-framed). Calls 01439ba0 on the receiver and returns the receiver. |
| `013dd140` | — | unknown | Vtable slot [16] shared by DeckServiceBase, DeckService and FRTWDeckService (244 bytes, thiscall, two flag bytes and an argument). Builds a local 180-byte reco… |
| `013dd240` | — | unknown | Vtable slot [35] shared by DeckServiceBase, DeckService and FRTWDeckService (468 bytes, thiscall, one std::string argument). Copies the argument into a tempora… |
| `013dd5b0` | — | unknown | Vtable slot [3] shared by DeckServiceBase, DeckService and FRTWDeckService (227 bytes, fastcall). Sets the byte at this+8 to 1, calls 01435790 and 014374b0, bu… |
| `013dd8b0` | — | unknown | Vtable slot [25] shared by DeckServiceBase and the Deck service classes (105 bytes, fastcall). Sets the std::string at the receiver to empty (capacity 0xf, siz… |
| `013de920` | — | unknown | Destructor body (211 bytes, fastcall, no free): stores vtable 018e24fc, resets the string at this+0x100, then calls 0143a510, 0143a620, 013ae2f0, 014431d0, 014… |

**Engine::FRTWCollectionService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013dc920` | Engine::FRTWCollectionService::~FRTWCollectionService | confirmed | Stores Engine::FRTWCollectionService vtable 018e248c into *this, then calls 013ab270 (base destructor body) on the object. It does not free the object (72 byte… |
| `013dc970` | Engine::FRTWCollectionService::ScalarDeletingDtor | confirmed | Vtable slot [0] of Engine::FRTWCollectionService (96 bytes): stores vtable 018e248c, calls 013ab270 (base destructor body), then if bit 0 of the argument is se… |
| `013dc850` | Engine::FRTWCollectionService::Ctor | likely | Fastcall (receiver in ECX, 199 bytes): calls 013ab190 (base construction; its vtable references Engine::CollectionServiceBase), stores vtable 018e248c into *th… |

### `game.achievement`

**AchievementCategoryDefinitionManager** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0104d470` | AchievementCategoryDefinitionManager::DrainMap | confirmed | Drains three maps owned by the manager: the head list at this+0x10 through 0104d0e0, then the lists at this+0x83c and this+0xbe8 through 0104ccc0; returns void. |
| `0104d880` | AchievementCategoryDefinitionManager::LoadCategories | confirmed | Opens AchievementCategories.txt through the text reader, splits each non-comment row with 006fc270 and 0072d900 (separator global DAT_01b9e39c), builds a tempo… |
| `0104e780` | AchievementCategoryDefinitionManager::DtorBody | confirmed | Restores AchievementCategoryDefinitionManager::vftable, runs the pool map teardown 0104d4d0 and the category map teardown 0104db20, then stores BaseDefinitionM… |
| `0104e8e0` | AchievementCategoryDefinitionManager::Ctor | confirmed | Installs BaseDefinitionManager::vftable then AchievementCategoryDefinitionManager::vftable, stores param_2 at +4, runs the category container constructor 0104e… |
| `0104e970` | AchievementCategoryDefinitionManager::ScalarDeletingDtor | confirmed | Runs the manager teardown 0104e780, then calls FreeMemory (00401c00) with alignment 0 on this only when bit 0 of the flag argument is set; returns this. |

**AchievementDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce37d0` | AchievementDataSource::ColumnNameByIndex | confirmed | Virtual at AchievementDataSource::vftable +0x30 slot [1] (one stack argument): switch on the index returning Id, Title, Description, Time Started, Time Finishe… |
| `00ce38c0` | AchievementDataSource::FlagGatedTailCall | speculative | Virtual at AchievementDataSource::vftable +0x30 slot [2] (no stack arguments): when the byte at this+0x22c is zero it calls vtbl+0x38 of the primary object at … |
| `00ce38f0` | AchievementDataSource::FlagGatedCall010c90c0 | speculative | Virtual at AchievementDataSource::vftable +0x34 slot [4] (one stack argument, ret 4): when the byte at this+0x228 is zero it calls vtbl+0x38 of the primary obj… |

**AchievementObjectiveDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce3980` | AchievementObjectiveDataSource::ColumnNameByIndex | confirmed | Virtual at AchievementObjectiveDataSource::vftable +0x30 slot [1] (one stack argument): switch on the index returning Name, Description, Status, Completed, Not… |
| `00ce3c40` | AchievementObjectiveDataSource::LinkedIdOrZero | speculative | Fastcall with no stack arguments: returns the dword at +0xf0 of the object pointed to by this+0x22c, or 0 when that pointer is null. |
| `00ce4770` | — | unknown | Virtual at AchievementObjectiveDataSource::vftable +0x30 slot [4] (two stack arguments): returns 0 unless the dword at this+0x22c is nonzero, the vtbl+0x10 cal… |

**AchievementTrophyDefinitionManager** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0104f480` | AchievementTrophyDefinitionManager::DrainMap | confirmed | Drains the trophy map at this+0x10 through 0104f1c0 and the pointer map at this+0x83c through 0104ed80; returns void. |
| `0104fda0` | AchievementTrophyDefinitionManager::DtorBody | confirmed | Restores AchievementTrophyDefinitionManager::vftable, runs the pool map teardown 0104f4c0 and the trophy map teardown 0104faf0, then stores BaseDefinitionManag… |
| `0104fef0` | AchievementTrophyDefinitionManager::Ctor | confirmed | Installs BaseDefinitionManager::vftable then AchievementTrophyDefinitionManager::vftable, stores param_2 at +4, runs the pool-backed trophy container construct… |
| `0104ff70` | AchievementTrophyDefinitionManager::ScalarDeletingDtor | confirmed | Runs the manager teardown 0104fda0, then calls FreeMemory (00401c00) with alignment 0 on this only when bit 0 of the flag argument is set; returns this. |
| `0104f870` | — | unknown | Opens AchievementTrophies.txt through the text reader, splits each non-comment row with 006fc270 and 0072d900 (separator global DAT_01b9e39c), builds a tempora… |

**SoeUtil::HashList<ClientAchievementData,256,8>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bad7a0` | HashList_ClientAchievementData_256_8_Ctor | confirmed | Constructor: calls the HashListMap<int,ClientAchievementData,256,8> constructor 00bad3e0 on ECX, then stores the HashList<ClientAchievementData,256,8> vtable (… |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bad8f0` | — | unknown | Constructor for an owner object (param_2 stored at +0): constructs a HashList<ClientAchievementData,256,8> at +4 (00bad7a0), zeroes fields at +0x10f4 to +0x110… |

### `game.collection`

**CollectionData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013a2630` | CollectionData::CollectionData | confirmed | Constructor body (ECX = object). Stores the PersistentBase, PersistentRoot and CollectionData vtables in that order; assigns an empty std::string to the member… |
| `013a2730` | CollectionData::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of CollectionData, ECX = object, arg = flags). Calls the body destructor 013a2110, then calls the _free thunk on the … |
| `013a2110` | — | unknown | Body of the CollectionData destructor (ECX = object). Stores CollectionData::vftable, calls the base destructors 012adee0 and 012ad020, walks the map at +0x58 … |

**FilterCriteria** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012c5180` | FilterCriteria::FilterCriteria | confirmed | Constructor body. Stores FilterCriteria::vftable, calls 012fa910 (base-class initialisation, which references the PersistentBase/PersistentComponent/ValueData … |
| `012c5e20` | FilterCriteria::~FilterCriteria | confirmed | Thiscall with one flag-byte argument (ret 4). Calls FilterCriteria::DtorBody (012c51e0) on the object, then calls _free when bit 0 of the flag is set. Returns … |
| `012c51e0` | FilterCriteria::DtorBody | likely | Destructor body. Stores FilterCriteria::vftable, clears the pointer at +0x18 when it is non-zero, then calls 01300cd0 (base teardown with the same object). Ret… |
| `012c5240` | FilterCriteria::MatchSortable | likely | Virtual method of FilterCriteria (vftable slot 1), thiscall with one pointer argument. Asserts (012f5a60, line 0xc0) that the argument is non-null with the nam… |

**SearchCriteria** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012c5550` | SearchCriteria::SearchCriteria | confirmed | Fastcall constructor body. Calls the base constructor 012c5180 (FilterCriteria::FilterCriteria) on the same object, then stores SearchCriteria::vftable. Return… |
| `012c55b0` | SearchCriteria::DtorBody | confirmed | Fastcall destructor body. Stores SearchCriteria::vftable and calls 012c51e0 (FilterCriteria::DtorBody) on the same object. Returns nothing. |
| `012c5e60` | SearchCriteria::~SearchCriteria | confirmed | Thiscall with one flag-byte argument (ret 4). Stores SearchCriteria::vftable, calls FilterCriteria::DtorBody (012c51e0) on the object, then calls _free when bi… |

**SortService** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012c5e00` | SortService::~SortService | confirmed | Thiscall with one flag-byte argument (ret 4). Stores SortService::vftable and calls _free on the object when bit 0 of the flag is set. Returns the object point… |
| `012c5d90` | SortService::EnsureGlobalInstance | likely | No arguments. When the global DAT_01cbd698 is null it allocates 4 bytes with operator new (00d833a6), stores SortService::vftable into the new object and keeps… |

**SynchronizationService** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012c4bb0` | SynchronizationService::DtorBody | confirmed | Destructor body (no delete flag test). Stores SynchronizationService::vftable, clears the global DAT_01cbd694 when it points at this object, calls 012c4b40 (Tr… |
| `012c4fd0` | SynchronizationService::SynchronizationService | confirmed | Constructor body. Stores SynchronizationService::vftable, sets the owner-name string at +0xc to empty (capacity +0x24 = 15, size +0x20 = 0, flag +0x10 = 0), ca… |
| `012c5060` | SynchronizationService::~SynchronizationService | confirmed | Thiscall with one flag-byte argument (ret 4). Calls the destructor body 012c4bb0 with ECX = this, then calls _free (00d82463) on the object when bit 0 of the f… |
| `012c3310` | SynchronizationService::StartCollectionSync | likely | Member of SynchronizationService. Calls 012c2ff0 (InitOwnerCollection) with ECX = param_2 and stack argument param_3, asserts (012f5a60, line 0xd0) that this+8… |
| `012c35c0` | SynchronizationService::InitAccountOnlineCollection | likely | Member of SynchronizationService. Calls 012d46a0, asserts (012f5a60, line 0x1da) that 012d08e0() is non-null for the "myAccount" check, calls 013a3480 and pass… |
| `012c1240` | SynchronizationService::ResetAndQueueEvents | speculative | Member of SynchronizationService (asserts this+8 (mCollection) non-null at line 0x19e). Clears byte this+0x40, calls 0139f790(this+0x44), 012adee0, 012ac430, t… |
| `012c3430` | SynchronizationService::ApplyEntryList | speculative | Member of SynchronizationService. Asserts (012f5a60, line 0x173) that 012d08e0() returned non-null for the "myAccount" check, calls 013a3480 and passes its res… |
| `012c4c40` | SynchronizationService::StartSyncPass | speculative | Member of SynchronizationService with 11 arguments. Sets this+0x40 = 1, calls 012d46a0 and 012d08e0 (asserts "myAccount" at line 0xf4), calls 013a3480 and pass… |

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00992e60` | SetCollectionFilterAndRefresh | likely | Stores the requested value at nested object +0x260, sets its dirty byte +0x25c, then invokes two refresh helpers. |
| `012c2ff0` | InitOwnerCollection | likely | Thiscall with ECX = object and two stack arguments: param_2 is formatted with %s and param_3 selects the format (3 = trade) and is stored at object+0x28. Forma… |
| `012c5080` | EnsureSingleton0x50 | likely | Cdecl-style function with no arguments. When the global DAT_01cbd694 is null it allocates 0x50 bytes with operator new (00d833a6) and stores the result of 012c… |
| `013a1950` | CollectionItemMapRead | likely | Thiscall (ECX = pointer to the serial reader holder; arg = destination tree, the CollectionData map at +0x58). Reads the element count with 013c2690 (SerialSta… |
| `012c5120` | ReleaseGlobalDAT01cbd698 | speculative | No arguments. If the global DAT_01cbd698 is non-null it calls its vtable slot 0 with argument 1 (deleting-destructor shape with the free flag set), then sets D… |
| `012c6ee0` | CriteriaCompareSortables | speculative | Thiscall on a FilterCriteria-shaped object (criterion code at +0, kind at +4, flag at +8; param_2 and param_3 are the two sortable entries). Takes the RTTI-che… |
| `013a22c0` | — | unknown | Thiscall (ECX = this; arg = object pointer whose item map sits at +0x58; the find and insert use this+0x5c as the head). Asserts the object is non-null (wa_ass… |

### `game.reward`

**ClientClaimManager** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a03910` | ClientClaimManager::ClientClaimManager | confirmed | Constructor: stores ClientClaimManager::vftable, stores the argument at +4 (the client/owner), clears the bytes at +8/+9 and the dword at +0xc, builds the call… |
| `009ff860` | ClaimSystemLog_Level5 | likely | Logging leaf: forwards a varargs format string to the shared logger 00700f90 with level 5 and the file name 'ClaimSystem.log'. |
| `009ff880` | ClaimSystemLog_Level4 | likely | Logging leaf: forwards a varargs format string to the shared logger 00700f90 with level 4 and the file name 'ClaimSystem.log'. |
| `009ff8a0` | ClaimSystemLog_Level3 | likely | Logging leaf: forwards a varargs format string to the shared logger 00700f90 with level 3 and the file name 'ClaimSystem.log'. |
| `009ff8c0` | ClaimSystemLog_Level2 | likely | Logging leaf: forwards a varargs format string to the shared logger 00700f90 with level 2 and the file name 'ClaimSystem.log'. |
| `00a03490` | — | unknown | Destructor body: stores ClientClaimManager::vftable, frees the 8-byte member at +0x98 through 00c694d0, clears that member and the byte at +8, then tears down … |

**RewardBundleBase** (44)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091c110` | RewardBundleBase::Ctor | confirmed | Constructor of the reward-bundle data model: calls FUN_008cf6e0, installs RewardBundleBase::vftable, zeroes/initializes the fields from +0x38 to +0xb4 (includi… |
| `0091c260` | RewardBundleBase::Set_0x90 | confirmed | Virtual setter: stores the argument into this+0x90 and returns. |
| `0091c270` | RewardBundleBase::Get_0x90 | confirmed | Virtual getter: returns the dword at this+0x90. |
| `0091c280` | RewardBundleBase::Add_0x48 | confirmed | Virtual accumulator: adds the argument to the dword at this+0x48. |
| `0091c290` | RewardBundleBase::GetXpRewardAmount | confirmed | Virtual getter: returns the dword at this+0x48, the reward XP/stars award amount. |
| `0091c2a0` | RewardBundleBase::Set_0x48 | confirmed | Virtual setter: stores the argument into this+0x48. |
| `0091c2b0` | RewardBundleBase::Add_0x4c | confirmed | Virtual accumulator: adds the argument to the dword at this+0x4c. |
| `0091c2c0` | RewardBundleBase::Get_0x4c | confirmed | Virtual getter: returns the dword at this+0x4c. |
| `0091c2d0` | RewardBundleBase::Add_0x50 | confirmed | Virtual accumulator: adds the argument to the dword at this+0x50. |
| `0091c2e0` | RewardBundleBase::GetCoinRewardAmount | confirmed | Virtual getter: returns the dword at this+0x50, the reward coin award amount. |
| `0091c2f0` | RewardBundleBase::Set_0x50 | confirmed | Virtual setter: stores the argument into this+0x50. |
| `0091c300` | RewardBundleBase::Set_0x64 | confirmed | Virtual setter: stores the argument into this+0x64 (printed as +100). |
| `0091c310` | RewardBundleBase::Set_0x68 | confirmed | Virtual setter: stores the argument into this+0x68. |
| `0091c320` | RewardBundleBase::Get_0x70 | confirmed | Virtual getter: returns the dword at this+0x70. |
| `0091c330` | RewardBundleBase::Set_0x70 | confirmed | Virtual setter: stores the argument into this+0x70. |
| `0091c340` | RewardBundleBase::Get_0x6c | confirmed | Virtual getter: returns the dword at this+0x6c. |
| `0091c350` | RewardBundleBase::Set_0x6c | confirmed | Virtual setter: stores the argument into this+0x6c. |
| `0091c360` | RewardBundleBase::GetByte_0x44 | confirmed | Virtual getter: returns the byte at this+0x44. |
| `0091c370` | RewardBundleBase::SetByte_0x44 | confirmed | Virtual setter: stores the byte argument at this+0x44. |
| `0091c380` | RewardBundleBase::GetRewardKind | confirmed | Virtual getter: returns the dword at this+0x54, the reward kind used to choose item-notification behaviour. |
| `0091c390` | RewardBundleBase::Set_0x54 | confirmed | Virtual setter: stores the argument into this+0x54. |
| `0091c3a0` | RewardBundleBase::Get_0x5c | confirmed | Virtual getter: returns the dword at this+0x5c. |
| `0091c3b0` | RewardBundleBase::Set_0x5c | confirmed | Virtual setter: stores the argument into this+0x5c. |
| `0091c3c0` | RewardBundleBase::Get_0x60 | confirmed | Virtual getter: returns the dword at this+0x60. |
| `0091c3d0` | RewardBundleBase::Set_0x60 | confirmed | Virtual setter: stores the argument into this+0x60. |
| `0091c3e0` | RewardBundleBase::Get_0x58 | confirmed | Virtual getter: returns the dword at this+0x58. |
| `0091c3f0` | RewardBundleBase::Set_0x58 | confirmed | Virtual setter: stores the argument into this+0x58. |
| `0091c400` | RewardBundleBase::GetPtr_0x94 | confirmed | Virtual accessor: returns the address this+0x94, the head of one of the bundle's StringFixed lists. |
| `0091c410` | RewardBundleBase::SetByte_0x74 | confirmed | Virtual setter: stores the byte argument at this+0x74. |
| `0091c440` | RewardBundleBase::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls FUN_0103d010 (the reward-bundle destructor body) and then frees this when bit 0 of the flag argument is set; returns this. |
| `0091c530` | RewardBundleBase::CopyIds_0x80_Out | confirmed | Byte-verified (x86 decode; Ghidra prints an empty body): copies this+0x80 and this+0x84 into *param_2 and *(param_2+4), i.e. hands out the first 8-byte id pair… |
| `0091c550` | RewardBundleBase::CopyIds_0x88_Out | confirmed | Byte-verified (x86 decode; Ghidra prints an empty body): copies this+0x88 and this+0x8c into *param_2 and *(param_2+4), handing out the second 8-byte id pair b… |
| `0091c570` | RewardBundleBase::SetIds_0x80 | confirmed | Virtual setter: stores param_2 at this+0x80 and param_3 at this+0x84, writing the whole first 8-byte id pair. |
| `0091c590` | RewardBundleBase::SetIds_0x88 | confirmed | Virtual setter: stores param_2 at this+0x88 and param_3 at this+0x8c, writing the whole second 8-byte id pair. |
| `0091c5b0` | RewardBundleBase::Get_0x40 | confirmed | Virtual getter: returns the dword at this+0x40, the count of entries in the bundle's intrusive reward list. |
| `0091c5c0` | RewardBundleBase::Get_0x38 | confirmed | Virtual getter: returns the dword at this+0x38, the head pointer of the bundle's intrusive reward list. |
| `0091c5e0` | RewardBundleBase::AddRewardEntry | confirmed | Appends the reward entry param_2 to the bundle's intrusive doubly-linked list (head at this+0x38, tail at this+0x3c, prev at node+4, next at node+8, count at t… |
| `0091c680` | RewardBundleBase::RemoveRewardEntry | confirmed | Unlinks the reward entry param_2 from the bundle's intrusive list: when the node has no predecessor (node+4) the head at this+0x38 becomes node+8, otherwise th… |
| `0091c7c0` | RewardBundleBase::AppendStringEntry | confirmed | Byte-verified wrapper: add ecx,0x94; call 0091c720 (append a new entry to the StringFixed<256> list at this+0x94); then mov ecx,eax and tail-jump to SoeUtil::I… |
| `00c33830` | RewardBundleBaseCtor | confirmed | Constructs logging state, installs RewardBundleBase vtable, zeroes fields and both embedded lists, then initializes a following member through FUN_0103a1e0. |
| `0103c070` | RewardBundleBase::CreateEntry | confirmed | Virtual slot 48 of RewardBundleBase (vtable +0xc0; ClientRewardBundle shares it), thiscall with one stack argument (type id). If the delegate pointer at this+0… |
| `0103d010` | RewardBundleBase::DtorBody | confirmed | Destructor body: stores RewardBundleBase::vftable at *this, drains the entry list at this+0x38 through 0103b110, stores SoeUtil::List<StringFixed<512>> vftable… |
| `01039d30` | RewardBundleBase::Get_0x64_OrSlot41 | likely | Virtual slot 24 (vtable +0x60) of RewardBundleBase (ClientRewardBundle shares it): if the dword at this+0x64 is -1 it calls vtable slot 41 (+0xa4) with ECX=thi… |
| `01039d80` | RewardBundleBase::Get_0x68_OrSlot41Slot4 | likely | Virtual slot 26 (vtable +0x68) of RewardBundleBase (ClientRewardBundle shares it): if the dword at this+0x68 is -1 and vtable slot 41 (+0xa4) returns a non-nul… |

**RewardBundleEntryAbilityLine** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103a190` | RewardBundleEntryAbilityLine::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryAbilityLine: same copy as RewardBundleEntryItem::CopyFields (base fields plus the dword at +0x44); no return value. |
| `0103adb0` | RewardBundleEntryAbilityLine::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this, stores RewardBundleEntryAbilityLine::vftable at *this, zeroes the dword at +0x44; returns th… |
| `0103ae10` | RewardBundleEntryAbilityLine::Set_0x44 | confirmed | Virtual slot 14 of RewardBundleEntryAbilityLine: stores its one stack argument dword into this+0x44 (ret 4). |
| `0103ae20` | RewardBundleEntryAbilityLine::Get_0x44 | confirmed | Virtual slot 15 of RewardBundleEntryAbilityLine: returns the dword at this+0x44. |
| `0103ae40` | RewardBundleEntryAbilityLine::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryAbilityLine: returns the pointer to the string AbilityLine. |
| `0103bca0` | RewardBundleEntryAbilityLine::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryAbilityLine: formats 'AbilityLineID=%d' with the dword at +0x44 through 00402fd0 into the static IString at 01cb6694 and re… |
| `0103d730` | RewardBundleEntryAbilityLine::WriteBody | likely | Virtual slot 10 of RewardBundleEntryAbilityLine (reached through thunk 0103da40): base writer 0103d0d0, then the dword at +0x44; no return value. |
| `0103ccd0` | — | unknown | Virtual slot 9 of RewardBundleEntryAbilityLine: calls the base reader 0103c460, then reads one u32 into +0x44 (zero on exhaustion without setting the reader er… |

**RewardBundleEntryBase** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103a000` | RewardBundleEntryBase::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryBase (Experience and FactoryExperience share it): copies the byte at +0xc and +0x41, the dwords at +0x10, +0x18, +0x1c, +0x… |
| `0103a3e0` | RewardBundleEntryBase::Ctor | confirmed | Constructor: stores RewardBundleEntryBase::vftable at *this; zeroes +4, +8, the byte +0xc, dword +0x10, dwords +0x20, +0x24, +0x28, the dword +0x3c and bytes +… |
| `0103a480` | RewardBundleEntryBase::Get_0x0c | confirmed | Virtual slot 2 of RewardBundleEntryBase (shared by all entry classes): returns the byte at this+0xc (mov al,[ecx+0xc]; ret). |
| `0103a490` | RewardBundleEntryBase::Set_0x0c | confirmed | Virtual slot 3 of RewardBundleEntryBase: stores its one stack argument byte into this+0xc (ret 4). |
| `0103a4a0` | RewardBundleEntryBase::Get_0x10 | confirmed | Virtual slot 4 of RewardBundleEntryBase: returns the dword at this+0x10. |
| `0103a4b0` | RewardBundleEntryBase::Set_0x10 | confirmed | Virtual slot 5 of RewardBundleEntryBase: stores its one stack argument dword into this+0x10 (ret 4). |
| `0103a4c0` | RewardBundleEntryBase::Get_0x20 | confirmed | Virtual slot 6 of RewardBundleEntryBase: returns the dword at this+0x20. |
| `0103a4d0` | RewardBundleEntryBase::Set_0x20 | confirmed | Virtual slot 7 of RewardBundleEntryBase: stores its one stack argument dword into this+0x20 (ret 4). |
| `0103a4e0` | RewardBundleEntryBase::Set_0x41 | confirmed | Virtual slot 8 of RewardBundleEntryBase: stores its one stack argument byte into this+0x41 (ret 4). |
| `0103d0d0` | RewardBundleEntryBase::WriteBody | likely | Thiscall with the bounded writer as its stack argument: writes, in order, the byte at +0xc, the dword pair at +4 and +8 (through 0103cf60), the dwords at +0x10… |
| `0103c460` | — 🌐 | unknown | Thiscall with the bounded reader as its stack argument (cursor at reader+8, end at reader+0xc, error flag at reader+0x10): reads in order a byte to +0xc (bool)… |

**RewardBundleEntryCharacterFlag** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103b350` | RewardBundleEntryCharacterFlag::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this, stores RewardBundleEntryCharacterFlag::vftable at *this, constructs a fixed string (00703540… |
| `0103b3e0` | RewardBundleEntryCharacterFlag::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryCharacterFlag: returns the pointer to the string CharacterFlag. |
| `0103b3f0` | RewardBundleEntryCharacterFlag::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryCharacterFlag: copies the shared base fields, assigns the string at +0x44 from the source through 006f9e60, then copies the… |
| `0103ba90` | RewardBundleEntryCharacterFlag::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryCharacterFlag: formats 'Key=%s, Operation=%d, Amount=%d, Persistent=%d, CountTimeOffline=%d' through 00402fd0 with the dwor… |
| `0103c9a0` | RewardBundleEntryCharacterFlag::ReadBody | likely | Virtual slot 9 implementation for CharacterFlag (reached through thunk 0103cd10): calls the base reader 0103c460, reads a length-prefixed block (signed int32 l… |
| `0103d790` | RewardBundleEntryCharacterFlag::WriteBody | likely | Virtual slot 10 of RewardBundleEntryCharacterFlag (reached through thunk 0103da50): base writer 0103d0d0, the length-prefixed string at +0x44 (00911c30), the d… |

**RewardBundleEntryChest** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103a0a0` | RewardBundleEntryChest::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryChest: same copy as RewardBundleEntryItem::CopyFields (base fields plus the dword at +0x44); no return value. |
| `0103a5d0` | RewardBundleEntryChest::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this, stores RewardBundleEntryChest::vftable at *this, then calls 0101cd40 with ECX=this+0x44 (zer… |
| `0103a630` | RewardBundleEntryChest::Set_0x24 | confirmed | Virtual slot 14 of RewardBundleEntryChest: stores its one stack argument dword into this+0x24 (ret 4). |
| `0103a650` | RewardBundleEntryChest::Get_0x44_Out | confirmed | Virtual slot 15 of RewardBundleEntryChest: writes the dword at this+0x44 through its stack pointer argument (ret 4). |
| `0103a660` | RewardBundleEntryChest::Set_0x44 | confirmed | Virtual slot 16 of RewardBundleEntryChest: stores its one stack argument dword into this+0x44 (ret 4). |
| `0103a670` | RewardBundleEntryChest::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryChest: returns the pointer to the string Chest. |
| `0103b550` | RewardBundleEntryChest::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryChest: calls vtable slot 15 with an out pointer to read the dword at +0x44, formats 'Chest: item id=%d, Item Guid=%llu' wit… |
| `0103d980` | RewardBundleEntryChest::WriteBody | confirmed | Virtual slot 10 of RewardBundleEntryChest: same body as RewardBundleEntryItem::WriteBody: base writer 0103d0d0, then when the byte at this+0x41 is non-zero the… |
| `0103c620` | — | unknown | Virtual slot 9 implementation for Chest (reached through the Chest vftable thunk 0103cb80): same body as RewardBundleEntryItem::ReadBody: base reader 0103c460,… |

**RewardBundleEntryClientExitUrl** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103af40` | RewardBundleEntryClientExitUrl::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this and stores RewardBundleEntryClientExitUrl::vftable at *this; returns this in EAX. |
| `0103afb0` | RewardBundleEntryClientExitUrl::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryClientExitUrl: returns the pointer to the string ClientExitUrl. |
| `0103afc0` | RewardBundleEntryClientExitUrl::Get_0x30 | confirmed | Virtual slot 15 of RewardBundleEntryClientExitUrl: returns the dword at this+0x30. |
| `0103afd0` | RewardBundleEntryClientExitUrl::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryClientExitUrl: copies the shared base fields (the same ten copies as RewardBundleEntryBase::CopyFields); no return value. |
| `0103b480` | RewardBundleEntryClientExitUrl::SetUrl | confirmed | Virtual slot 14 of RewardBundleEntryClientExitUrl (one stack argument): if the argument is null it substitutes the static empty string 0175b400; then it tail-j… |
| `0103bf10` | RewardBundleEntryClientExitUrl::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryClientExitUrl: calls vtable slot 15 for its value (the dword at +0x30), formats 'ClientExitUrl=%s' with that value through … |

**RewardBundleEntryCollectionAdd** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103a8f0` | RewardBundleEntryCollectionAdd::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this, stores RewardBundleEntryCollectionAdd::vftable at *this, zeroes the dword at +0x44; returns … |
| `0103a960` | RewardBundleEntryCollectionAdd::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryCollectionAdd: returns the pointer to the string CollectionAdd. |
| `0103a970` | RewardBundleEntryCollectionAdd::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryCollectionAdd: same copy as RewardBundleEntryItem::CopyFields (base fields plus the dword at +0x44); no return value. |
| `0103b800` | RewardBundleEntryCollectionAdd::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryCollectionAdd: formats 'CollectionID=%d' with the dword at +0x44 through 00402fd0 into the static IString at 01cb6608 and r… |
| `0103d3f0` | RewardBundleEntryCollectionAdd::WriteBody | likely | Virtual slot 10 of RewardBundleEntryCollectionAdd (reached through thunk 0103d9e0): base writer 0103d0d0, then the dword at +0x44; no return value. |
| `0103cc10` | — | unknown | Virtual slot 9 of RewardBundleEntryCollectionAdd: calls the base reader 0103c460, then reads one u32 into +0x44 (zero on exhaustion without setting the reader … |

**RewardBundleEntryCollectionEntryAdd** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103a9c0` | RewardBundleEntryCollectionEntryAdd::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this, stores RewardBundleEntryCollectionEntryAdd::vftable at *this, zeroes the dwords at +0x44, +0… |
| `0103aa30` | RewardBundleEntryCollectionEntryAdd::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryCollectionEntryAdd: returns the pointer to the string CollectionEntryAdd. |
| `0103aa40` | RewardBundleEntryCollectionEntryAdd::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryCollectionEntryAdd: copies the base fields plus the dwords at +0x44, +0x48, +0x4c and +0x50 from the source into this; no r… |
| `0103b8a0` | RewardBundleEntryCollectionEntryAdd::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryCollectionEntryAdd: formats 'CollectionEntryType=%d, Parm1=%d, Parm2=%d, Parm3=%d' through 00402fd0 with the dwords at +0x4… |
| `0103c740` | RewardBundleEntryCollectionEntryAdd::ReadBody | likely | Virtual slot 9 implementation for CollectionEntryAdd (reached through thunk 0103cc50): calls the base reader 0103c460, then reads four u32 values (zero on exha… |
| `0103d450` | RewardBundleEntryCollectionEntryAdd::WriteBody | likely | Virtual slot 10 of RewardBundleEntryCollectionEntryAdd (reached through thunk 0103d9f0): base writer 0103d0d0, then the dwords at +0x44, +0x48, +0x4c and +0x50… |

**RewardBundleEntryExperience** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103a680` | RewardBundleEntryExperience::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this and stores RewardBundleEntryExperience::vftable at *this; returns this. |
| `0103a6f0` | RewardBundleEntryExperience::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryExperience: returns the pointer to the string Experience. |
| `0103b600` | RewardBundleEntryExperience::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryExperience: formats 'PROFILE=%d' with the dword at +0x24 through 00402fd0 into the static IString at 01cb65cc and returns i… |

**RewardBundleEntryFactoryExperience** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103a700` | RewardBundleEntryFactoryExperience::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this and stores RewardBundleEntryFactoryExperience::vftable at *this; returns this. |
| `0103a770` | RewardBundleEntryFactoryExperience::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryFactoryExperience: returns the pointer to the string FactoryExperience. |
| `0103b6a0` | RewardBundleEntryFactoryExperience::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryFactoryExperience: formats 'Factory Type=%s' through 00402fd0 into the static IString at 01cb65e0, passing the dwords at +0… |

**RewardBundleEntryItem** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103a050` | RewardBundleEntryItem::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryItem: copies the same fields as RewardBundleEntryBase::CopyFields (bytes +0xc, +0x41, +0x40; dwords +0x10, +0x18, +0x1c, +0… |
| `0103a4f0` | RewardBundleEntryItem::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this, stores RewardBundleEntryItem::vftable at *this, then calls 0101cd40 with ECX=this+0x44 (zero… |
| `0103a570` | RewardBundleEntryItem::Set_0x24_0x28 | confirmed | Virtual slot 14 of RewardBundleEntryItem: stores its two stack arguments into the dwords at this+0x24 (first) and this+0x28 (second); ret 8. |
| `0103a5a0` | RewardBundleEntryItem::Get_0x44_Out | confirmed | Virtual slot 15 of RewardBundleEntryItem: writes the dword at this+0x44 through its one stack pointer argument (ret 4). |
| `0103a5b0` | RewardBundleEntryItem::Set_0x44 | confirmed | Virtual slot 16 of RewardBundleEntryItem: stores its one stack argument dword into this+0x44 (ret 4). |
| `0103b4a0` | RewardBundleEntryItem::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryItem: on first use initialises the static IString at 01cb65a4 (vtable 0175b24c, buffer 01cb65a0, zero length; guarded by bi… |
| `0103d940` | RewardBundleEntryItem::WriteBody | confirmed | Virtual slot 10 of RewardBundleEntryItem: calls the base writer 0103d0d0; when the byte at this+0x41 is non-zero it reads the dword through vtable slot 15 (out… |
| `0103a5c0` | RewardBundleEntryItem::TypeName | likely | Virtual slot 12 of RewardBundleEntryItem: returns the address of the static text at 018098ec (bytes read as the string Item followed by NUL), with no arguments. |
| `0103c5c0` | — | unknown | Virtual slot 9 implementation for Item (reached through the Item vftable thunk 0103cb70): calls the base reader 0103c460 on the reader; if the byte at this+0x4… |

**RewardBundleEntryPetTrickExperience** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103ab90` | RewardBundleEntryPetTrickExperience::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this, stores RewardBundleEntryPetTrickExperience::vftable at *this, zeroes the dwords at +0x44 and… |
| `0103ac00` | RewardBundleEntryPetTrickExperience::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryPetTrickExperience: returns the pointer to the string TrickExperience. |
| `0103ac10` | RewardBundleEntryPetTrickExperience::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryPetTrickExperience: copies the shared base fields plus the dwords at +0x44 and +0x48 from the source; no return value. |
| `0103b9f0` | RewardBundleEntryPetTrickExperience::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryPetTrickExperience: formats 'TrickID=%d' with the dword at +0x44 through 00402fd0 into the static IString at 01cb6644 and r… |
| `0103c800` | RewardBundleEntryPetTrickExperience::ReadBody | likely | Virtual slot 9 implementation for PetTrickExperience (reached through thunk 0103cc70): calls the base reader 0103c460, then reads two u32 values (zero on exhau… |
| `0103d580` | RewardBundleEntryPetTrickExperience::WriteBody | likely | Virtual slot 10 of RewardBundleEntryPetTrickExperience (reached through thunk 0103da10): base writer 0103d0d0, then the dwords at +0x44 and +0x48; no return va… |

**RewardBundleEntryProfileAdd** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103a800` | RewardBundleEntryProfileAdd::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this, stores RewardBundleEntryProfileAdd::vftable at *this, zeroes the dword at +0x44; returns thi… |
| `0103a860` | RewardBundleEntryProfileAdd::Set_0x44 | confirmed | Virtual slot 14 of RewardBundleEntryProfileAdd: stores its one stack argument dword into this+0x44 (ret 4). |
| `0103a880` | RewardBundleEntryProfileAdd::EqualsDword44 | confirmed | Virtual slot 15 of RewardBundleEntryProfileAdd: returns al=1 when its one stack argument equals the dword at this+0x44, else al=0 (cmp, sete; the callee's ret … |
| `0103a890` | RewardBundleEntryProfileAdd::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryProfileAdd: returns the pointer to the string ProfileAdd. |
| `0103a8a0` | RewardBundleEntryProfileAdd::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryProfileAdd: same copy as RewardBundleEntryItem::CopyFields (base fields plus the dword at +0x44); no return value. |
| `0103b760` | RewardBundleEntryProfileAdd::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryProfileAdd: formats 'PROFILE=%d' with the dword at +0x44 through 00402fd0 into the static IString at 01cb65f4 and returns i… |
| `0103d390` | RewardBundleEntryProfileAdd::WriteBody | likely | Virtual slot 10 of RewardBundleEntryProfileAdd (reached through thunk 0103d9d0): base writer 0103d0d0, then the dword at +0x44; no return value. |
| `0103cbd0` | — | unknown | Virtual slot 9 of RewardBundleEntryProfileAdd: calls the base reader 0103c460, then reads one u32 into +0x44; on exhaustion it stores 0 there without setting t… |

**RewardBundleEntryQuestAdd** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103a0f0` | RewardBundleEntryQuestAdd::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryQuestAdd: copies the same fields as RewardBundleEntryItem::CopyFields (bytes +0xc, +0x41, +0x40; dwords +0x10, +0x18, +0x1c… |
| `0103a780` | RewardBundleEntryQuestAdd::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this, stores RewardBundleEntryQuestAdd::vftable at *this, zeroes the dword at +0x44; returns this. |
| `0103a7f0` | RewardBundleEntryQuestAdd::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryQuestAdd: returns the pointer to the string QuestAdd. |
| `0103bb50` | RewardBundleEntryQuestAdd::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryQuestAdd: formats 'QuestID=%d' with the dword at +0x44 through 00402fd0 into the static IString at 01cb666c and returns its… |
| `0103cb90` | RewardBundleEntryQuestAdd::ReadBody | confirmed | Virtual slot 9 of RewardBundleEntryQuestAdd: calls the base reader 0103c460, then reads one u32 into +0x44 (zero on exhaustion, error flag set); no return valu… |
| `0103d330` | RewardBundleEntryQuestAdd::WriteBody | likely | Virtual slot 10 of RewardBundleEntryQuestAdd (reached through thunk 0103d9c0): calls the base writer 0103d0d0, then writes the dword at +0x44 with the bounded … |

**RewardBundleEntryRecipe** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103a140` | RewardBundleEntryRecipe::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryRecipe: same copy as RewardBundleEntryItem::CopyFields (base fields plus the dword at +0x44); no return value. |
| `0103ac70` | RewardBundleEntryRecipe::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this, stores RewardBundleEntryRecipe::vftable at *this, zeroes the dword at +0x44; returns this in… |
| `0103acd0` | RewardBundleEntryRecipe::Set_0x44 | confirmed | Virtual slot 14 of RewardBundleEntryRecipe: stores its one stack argument dword into this+0x44 (ret 4). |
| `0103ace0` | RewardBundleEntryRecipe::Get_0x44 | confirmed | Virtual slot 15 of RewardBundleEntryRecipe: returns the dword at this+0x44 as its value (not through an out pointer). |
| `0103ad00` | RewardBundleEntryRecipe::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryRecipe: returns the pointer to the string Recipe. |
| `0103bbf0` | RewardBundleEntryRecipe::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryRecipe: calls vtable slot 15 for its value, formats 'RecipeID=%d' with that value through 00402fd0 into the static IString … |
| `0103d620` | RewardBundleEntryRecipe::WriteBody | likely | Virtual slot 10 of RewardBundleEntryRecipe (reached through thunk 0103da20): base writer 0103d0d0, then the dword at +0x44; no return value. |
| `0103cc80` | — | unknown | Virtual slot 9 of RewardBundleEntryRecipe: calls the base reader 0103c460, then reads one u32 into +0x44 (zero on exhaustion without setting the reader error f… |

**RewardBundleEntryToken** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103aaa0` | RewardBundleEntryToken::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this, stores RewardBundleEntryToken::vftable at *this; returns this in EAX. |
| `0103ab00` | RewardBundleEntryToken::Set_0x28 | confirmed | Virtual slot 5 of RewardBundleEntryToken: stores its one stack argument dword into this+0x28 (ret 4). |
| `0103ab10` | RewardBundleEntryToken::Get_0x28 | confirmed | Virtual slot 4 of RewardBundleEntryToken: returns the dword at this+0x28. |
| `0103ab30` | RewardBundleEntryToken::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryToken: returns the pointer to the string Token. |
| `0103ab40` | RewardBundleEntryToken::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryToken: copies the shared base fields (bytes +0xc, +0x41, +0x40; dwords +0x10, +0x18, +0x1c, +0x20, +0x24, +0x28, +0x3c) fro… |
| `0103b950` | RewardBundleEntryToken::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryToken: formats 'TokenId=%d' with the dword at +0x24 through 00402fd0 into the static IString at 01cb6630 and returns its bu… |

**RewardBundleEntryTrophy** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103b020` | RewardBundleEntryTrophy::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this and stores RewardBundleEntryTrophy::vftable at *this; returns this in EAX. |
| `0103b080` | RewardBundleEntryTrophy::Set_0x24 | confirmed | Virtual slot 14 of RewardBundleEntryTrophy: stores its one stack argument dword into this+0x24 (ret 4). |
| `0103b090` | RewardBundleEntryTrophy::Get_0x24 | confirmed | Virtual slot 15 of RewardBundleEntryTrophy: returns the dword at this+0x24. |
| `0103b0b0` | RewardBundleEntryTrophy::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryTrophy: returns the pointer to the string Trophy. |
| `0103b0c0` | RewardBundleEntryTrophy::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryTrophy: copies the shared base fields (the same ten copies as RewardBundleEntryBase::CopyFields); no return value. |
| `0103bfc0` | RewardBundleEntryTrophy::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryTrophy: calls vtable slot 15 for its value, formats 'Trophy=%d' with that value through 00402fd0 into the static IString at… |

**RewardBundleEntryWheelSpin** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103ae50` | RewardBundleEntryWheelSpin::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this and stores RewardBundleEntryWheelSpin::vftable at *this; returns this in EAX. |
| `0103aeb0` | RewardBundleEntryWheelSpin::Set_0x24 | confirmed | Virtual slot 14 of RewardBundleEntryWheelSpin: stores its one stack argument dword into this+0x24 (ret 4). |
| `0103aec0` | RewardBundleEntryWheelSpin::Get_0x24 | confirmed | Virtual slot 15 of RewardBundleEntryWheelSpin: returns the dword at this+0x24. |
| `0103aee0` | RewardBundleEntryWheelSpin::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryWheelSpin: returns the pointer to the string WheelSpin. |
| `0103aef0` | RewardBundleEntryWheelSpin::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryWheelSpin: copies the shared base fields (the same ten copies as RewardBundleEntryBase::CopyFields, no +0x44); no return va… |
| `0103be60` | RewardBundleEntryWheelSpin::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryWheelSpin: calls vtable slot 15 for its value, formats 'WheelSpin=%d' with that value through 00402fd0 into the static IStr… |

**RewardBundleEntryZoneFlag** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103ad10` | RewardBundleEntryZoneFlag::Ctor | confirmed | Constructor: calls RewardBundleEntryBase::Ctor (0103a3e0) on this, stores RewardBundleEntryZoneFlag::vftable at *this, stores SoeUtil::IString<char>::vftable a… |
| `0103ad80` | RewardBundleEntryZoneFlag::Get_0x54 | confirmed | Virtual slot 17 of RewardBundleEntryZoneFlag: returns the dword at this+0x54. |
| `0103ada0` | RewardBundleEntryZoneFlag::TypeName | confirmed | Virtual slot 12 of RewardBundleEntryZoneFlag: returns the pointer to the string Zone Flag. |
| `0103b1a0` | RewardBundleEntryZoneFlag::SetNameValue_Type0 | confirmed | Virtual slot 14 of RewardBundleEntryZoneFlag (five stack arguments, ret 0x14): assigns the string at +0x44 from the first string argument through 006f9e60, sto… |
| `0103b240` | RewardBundleEntryZoneFlag::SetNameValue_Type1 | confirmed | Virtual slot 15 of RewardBundleEntryZoneFlag (five stack arguments, ret 0x14): the same body as the slot-14 setter, but stores 1 at +0x58. |
| `0103b2e0` | RewardBundleEntryZoneFlag::GetName_Into | confirmed | Virtual slot 16 of RewardBundleEntryZoneFlag (one stack argument, ret 4): initialises the caller's IString at the argument (vtable, static empty buffer 01cb65a… |
| `0103bd40` | RewardBundleEntryZoneFlag::CopyFields | confirmed | Virtual slot 11 of RewardBundleEntryZoneFlag: copies the shared base fields (the ten copies of RewardBundleEntryBase::CopyFields), the dword at +0x54 and the d… |
| `0103bdb0` | RewardBundleEntryZoneFlag::FormatText | confirmed | Virtual slot 13 of RewardBundleEntryZoneFlag: formats 'ZoneFlag=%s, Value=%d, Type=%d' with the dwords at +0x48 (string buffer pointer), +0x54 and +0x58 throug… |
| `0103c8b0` | RewardBundleEntryZoneFlag::ReadBody | likely | Virtual slot 9 implementation for ZoneFlag (reached through thunk 0103ccc0): calls the base reader 0103c460, reads a signed int32 length n (error when negative… |
| `0103d680` | RewardBundleEntryZoneFlag::WriteBody | likely | Virtual slot 10 of RewardBundleEntryZoneFlag (reached through thunk 0103da30): base writer 0103d0d0, the length-prefixed string at +0x44 (00911c30), then the d… |

**SoeUtil::Array<ClaimItem,0,1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a01a30` | Array_ClaimItem_0_ScalarDeletingDestructor | confirmed | Scalar deleting destructor of SoeUtil::Array<ClaimItem,0,1>: stores the class vtable, truncates the elements, frees the backing store and then frees this when … |
| `00a007a0` | Array_ClaimItem_0_FreeBackingStore | likely | Free wrapper (18 bytes: mov eax,[esp+4]; push 8; push eax; call 00401c00; add esp,8; ret 8): releases the array buffer with alignment 8. |
| `00a007c0` | Array_ClaimItem_0_Swap | likely | Array swap slot: when the flag param_3 is non-zero it tail-jumps into virtual slot 3 (offset 0xc) of the object in param_2 passing this object as the argument … |
| `00a00700` | — | unknown | Capacity policy of a 0x168-byte-element array: grows the request by 5/4 when it exceeds the current capacity, shrinks it by 6/5 when it is below three quarters… |

**SoeUtil::Array<ClaimItem,32,1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a00830` | Array_ClaimItem_32_FreeBackingStore | likely | Releases param_2 with alignment 8 unless it points at the object own inline storage ((this+0x17) & ~7), in which case nothing is freed; 28 bytes, ret 8. |
| `00a014d0` | Array_ClaimItem_32_GetBuffer | likely | Buffer request of SoeUtil::Array<ClaimItem,32,1>: requests of at most 0x20 elements return the inline storage at (this+0x17) & ~7 with *param_3 = 0x20, larger … |
| `00a02080` | Array_ClaimItem_32_ScalarDeletingDestructor | likely | Scalar deleting destructor of SoeUtil::Array<ClaimItem,32,1>: runs the 32-inline destructor body 00a01400 and frees this when bit 0 of the flag is set. |
| `00a027f0` | Array_ClaimItem_32_Swap | likely | Content exchange of two 32-inline claim item arrays through a stack temporary: temp.Append(src.data, src.count) with ecx=&temp, then src.Truncate(src.count) wi… |

**SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureClaimItemId,0>,0,1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a01ad0` | Array_ClaimItemId_0_ScalarDeletingDestructor | confirmed | Scalar deleting destructor of SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureClaimItemId,0>,0,1>: stores the class vtable, resets the count, frees … |
| `00a00960` | Array_ClaimItemId_0_FreeBackingStore | likely | Free wrapper: releases the buffer with 00401c00 at alignment 8 (same body as 00a007a0). |
| `00a008c0` | — | unknown | Capacity policy of an 8-byte-element array: grows the request by 5/4 when it exceeds the current capacity, shrinks it by 6/5 when it is below three quarters of… |
| `00a00980` | — | unknown | Array swap slot (same body as 00a007c0): with the flag set it tail-jumps into virtual slot 3 of param_2 passing this and 0, otherwise it xor-swaps the data poi… |

**SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureClaimItemId,0>,16,1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a009f0` | Array_ClaimItemId_16_FreeBackingStore | likely | Releases param_2 with alignment 8 unless it equals the object own inline storage ((this+0x17) & ~7); same body as 00a00830. |
| `00a015f0` | Array_ClaimItemId_16_GetBuffer | likely | Buffer request of SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureClaimItemId,0>,16,1>: requests of at most 0x10 elements return the inline storage … |
| `00a020a0` | Array_ClaimItemId_16_ScalarDeletingDestructor | likely | Scalar deleting destructor of SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureClaimItemId,0>,16,1>: runs the 16-inline destructor body 00a01530 and … |
| `00a02220` | Array_ClaimItemId_16_Swap | likely | Content exchange of two 16-inline claim item id arrays through a stack temporary (decoded call sites: temp.Append(src.data, src.count) with ecx=&temp at 00a022… |

**SoeUtil::HashList<ClientClaimManager::ContentTracker,8,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a03760` | HashList_ContentTracker_ScalarDeletingDestructor | likely | Scalar deleting destructor: runs the tracker-map teardown body 00a03420 (stores the HashListMap<int,ClientClaimManager::ContentTracker,8,-1> vtable and drains … |

**SoeUtil::HashListMap<int,ClientClaimManager::ContentTracker,8,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a032e0` | HashListMap_ContentTracker_ScalarDeletingDestructor | confirmed | Scalar deleting destructor of the tracker map: stores the HashListMap<int,ClientClaimManager::ContentTracker,8,-1> vtable, drains the map through 00a02150 and … |
| `00a008a0` | HashListMap_ContentTracker_FreeStorage | likely | Free wrapper: releases storage with 00401c00 at alignment 8 (same body as 00a007a0). |
| `00a00860` | — | unknown | Allocator hook (bytes: call 006fb9c0; mov ecx,eax; call 006fb860; test eax,eax; push 4; push 400; je fallback; mov edx,[eax]; mov ecx,eax; mov eax,[edx+4]; cal… |

**SoeUtil::List<bool_(__thiscall_ClientClaimManager::*)(void)const_,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a02400` | List_Callback_0_ScalarDeletingDestructor | confirmed | Scalar deleting destructor of SoeUtil::List<bool (ClientClaimManager::*)(void)const,-1>: stores the class vtable, drains the list through 00a01370 and frees th… |
| `00a006a0` | List_Callback_AllocNode | likely | Allocator hook (bytes: call 006fb9c0; mov ecx,eax; call 006fb860; test eax,eax; push 4; push 0xc; je fallback; mov edx,[eax]; mov ecx,eax; mov eax,[edx+4]; cal… |
| `00a006e0` | List_Callback_FreeNode | likely | Free wrapper (18 bytes: mov eax,[esp+4]; push 4; push eax; call 00401c00; add esp,8; ret 4): releases the node with the engine FreeMemory helper at alignment 4. |

**SoeUtil::List<bool_(__thiscall_ClientClaimManager::*)(void)const_,8>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a03100` | AdjustThisPlus0x10AndJump0070de20 | confirmed | 8-byte adjustor thunk: adds 0x10 to ECX and jumps to 0070de20. |
| `00a03110` | AdjustThisPlus0x10AndJump0070de90 | confirmed | 8-byte adjustor thunk: adds 0x10 to ECX and jumps to 0070de90. |
| `00a03120` | List_Callback_8_ScalarDeletingDestructor | likely | Scalar deleting destructor of SoeUtil::List<bool (ClientClaimManager::*)(void)const,8>: runs the hashed-list destructor body 00a03050 and frees this when bit 0… |

**(no class)** (47)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a013d0` | Array_ClaimItem_0_Destruct | confirmed | Destructor body of SoeUtil::Array<ClaimItem,0,1>: stores the class vtable, truncates the element count to 0 through 009fff10, frees the backing store with alig… |
| `00a01400` | Array_ClaimItem_32_Destruct | confirmed | Destructor body of SoeUtil::Array<ClaimItem,32,1>: stores the 32-inline class vtable, truncates the elements and frees the store unless it is the inline buffer… |
| `00a01500` | Array_ClaimItemId_0_Destruct | confirmed | Destructor body of SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureClaimItemId,0>,0,1>: stores the class vtable, sets the element count to 0, frees … |
| `00a01530` | Array_ClaimItemId_16_Destruct | confirmed | Destructor body of SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureClaimItemId,0>,16,1>: stores the 16-inline class vtable, resets the count and fre… |
| `00a01f60` | Array_ClaimItem_32_Construct | confirmed | Constructor of SoeUtil::Array<ClaimItem,32,1>: stores the Array<ClaimItem,0,1> base vtable, zeroes the three data words (+4/+8/+0xc) and then stores the 32-inl… |
| `00a01fc0` | Array_ClaimItemId_16_Construct | confirmed | Constructor of SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureClaimItemId,0>,16,1>: stores the 0-inline base vtable, zeroes +4/+8/+0xc and stores t… |
| `00a02140` | List_Callback_0_Destruct | confirmed | Destructor body (11 bytes: mov dword ptr [ecx], 018119e8; jmp 00a01370): stores the SoeUtil::List<bool (ClientClaimManager::*)(void)const,-1> vtable and then t… |
| `00a02fd0` | List_Callback_8_Construct | confirmed | Constructor of SoeUtil::List<bool (ClientClaimManager::*)(void)const,8>: stores the plain-list vtable, zeroes +4/+8/+0xc, stores the hashed (8-bucket) vtable a… |
| `00a03050` | List_Callback_8_Destruct | confirmed | Destructor body of SoeUtil::List<bool (ClientClaimManager::*)(void)const,8>: stores the hashed vtable, drains the nodes through 00a01370, releases the helper o… |
| `00a03140` | HashListMap_ContentTracker_Destruct | confirmed | Destructor body of the tracker map: stores the HashListMap<int,ClientClaimManager::ContentTracker,8,-1> vtable and repeatedly calls 00a02150 on the ordered-lis… |
| `00a03420` | HashListMap_ContentTracker_DestructB | confirmed | Second copy of the tracker-map destructor body: stores the HashListMap<int,ClientClaimManager::ContentTracker,8,-1> vtable inside an SEH frame and drains the m… |
| `00a03570` | HashList_ContentTracker_Construct | confirmed | Constructor of SoeUtil::HashList<ClientClaimManager::ContentTracker,8,-1>: stores the HashListMap<int,ClientClaimManager::ContentTracker,8,-1> base vtable, zer… |
| `00b8a640` | RewardFamilyPacketDispatch 🌐 | confirmed | Reads reward subtype and body, queues bundle presentation, notifies item rewards, accumulates deferred XP/coin deltas, or clears caches and displays granted-ba… |
| `0091c5d0` | GetField8 | likely | Returns the dword at offset +8 of its single argument (the argument is taken from the stack, not from this). |
| `0091c720` | ListAppendAllocEntry | likely | Appends a new entry to the list that lives at this (head at +4, tail at +8, count at +0xc): allocates through the virtual (*this+8) slot, constructs the entry … |
| `00993b20` | InitializeIconRewardNode | likely | Clears a 0x8c-byte member block, stores arguments into object fields, constructs an IconData member and a RewardBundleBase member, and initializes trailing sta… |
| `00a001d0` | ArrayGrowMove360ByteElements | likely | Reallocation helper for 0x168-element arrays: obtains a new block for param_2 elements via virtual slot 1, copy-constructs each old element with 01060d10, dest… |
| `00a00390` | ContentTracker_Construct | likely | Stores param_2 at +0 as the tracker type, param_3 at +4 as the id, copy-constructs the payload at +8 through 01060d10, stores param_5 at +0x170, zeroes the sta… |
| `00a005e0` | ResetPointerAt98AndFlag8 | likely | Reset helper: reads the pointer at this+0x98, clears the byte at this+8, and when that pointer is non-null calls 00c694d0 on it and frees it, then zeroes this+… |
| `00a00610` | NextClaimTrackerId | likely | Id generator: lazily initialises the global counter DAT_01be63a4 to 0 (guarded by the bit-0 flag DAT_01be63a8), increments it, skips the sentinel value DAT_01b… |
| `00a00d70` | ContentTracker_ConstructInline | likely | Constructor wrapper: runs the ContentTracker constructor 00a00390 with the caller's arguments and then stores *param_2 (the tracker id) into this+0x188, the fi… |
| `00a01070` | ArrayResize_360ByteElements | likely | Resize of a 0x168-byte-element array to param_2 elements: grows the backing store through 00a001d0 when the capacity is short, sets the element count, and defa… |
| `00a01620` | ArrayAppendMany_8ByteElements | likely | Appends param_3 copies of 8-byte values from param_2 to an 8-byte-element array: grows through 009fffb0 when needed, advances the count by param_3 and copies t… |
| `00a016d0` | ArrayAppend_8ByteElements | likely | Appends one 8-byte value from param_2 to an 8-byte-element array: grows through 009fffb0 when needed, writes the value at the new slot and returns a pointer to… |
| `00a01760` | ArrayAppend_8ByteElementsB | likely | Appends one 8-byte value from param_2 to an 8-byte-element array (second copy of 00a016d0): grows through 009fffb0, writes the value and returns its address. |
| `00a01a70` | DeletingDtor_01060db0MemberOffset8 | likely | Calls 01060db0 with ECX = this+8; if flag bit 0 is set, calls _free(this), then returns this. |
| `00a01bf0` | Array_ClaimItemId_CopyConstruct | likely | Copy constructor of the claim item id array: sets the destination count to 0 and, when the source has elements, appends all of them through 00a01620; the empty… |
| `00a02480` | Array_ClaimItem_Assign | likely | Assign helper for ClaimItem arrays: truncates this array to zero elements through 009fff10 and then appends all elements of the source (or appends nothing when… |
| `00a03170` | HashListMap_ContentTracker_InsertElement | likely | Adds one ContentTracker to the manager's hashed tracker map: allocates the element through the container's virtual slot 1, constructs it with 00a00d70, links i… |
| `00a04c10` | RequestClaimItemDefinitions 🌐 | likely | For a tracker it builds a ClaimItemsItemDefinitionsRequest (00a02550, sub-opcode 5) whose id array collects the dword at entry+8 of every 0x18-byte entry in th… |
| `00a04f40` | PushClaimItemToTooltipOrRequestDefinitions | likely | Same shape as 00a04ea0 but for the equip case: bounds-check, use the fast path (00a012a0 then 00a03540 to push the item into the tooltip data source) when all … |
| `00a04fd0` | SendClaimItemsRequestForOneId 🌐 | likely | Generates a tracker id, constructs a 16-inline claim-id array, appends the single 8-byte value pointed to by param_2+8, and constructs a ClaimItemsRequestPacke… |
| `00c33910` | ReplaceAndFilterBundleObject | likely | Replaces the owned reward-bundle pointer, allocates and constructs a new RewardBundleBase for nonnull input, filters entries by type and global predicates, rem… |
| `00a00c50` | MemberDtor_01060db0_Offset8B | speculative | SEH-framed cleanup restores the exception chain, adds 8 to the incoming this pointer in ECX, calls 01060db0 on that subobject, and returns. |
| `00a01cc0` | ArrayAppendMany_360ByteElements | speculative | Appends param_3 claim items from param_2 to a 0x168-byte-element array: grows through 00a001d0 when needed, then copy-constructs each element with 01060d10 whi… |
| `00a02910` | DrainTrackerChain | speculative | Drains a chain: while the pointer at this+8 is non-null it calls 00a02150 on that element, which unlinks the element from its bucket chain and ordered list, ru… |
| `00a04ea0` | ClaimItemContentsFastPath | speculative | Claims the item at index param_2 of the manager's claim item array: bounds-check the index, and when all of the item's definitions are already known (00a012a0)… |
| `00993bf0` | — | unknown | Allocates 400 bytes, initializes a record through 00993b20, indexes it via its key at +0x9c, marks two data sources dirty, refreshes them, and returns 1. |
| `009ffb10` | — | unknown | SEH-framed cleanup restores the exception chain, adds 8 to the incoming this pointer in ECX, calls 01060db0 on that subobject, and returns. |
| `009fff10` | — | unknown | Shrinks a dynamic array to param_2 elements: for every element from index param_2 upward it calls virtual slot 1 of the element (destructor) with 0 at stride 0… |
| `00a012a0` | — | unknown | Predicate over a claim item (param_1 = the claim manager, param_2 = a 0x168-byte claim item): reads the item's id list at +0x74 (count at +0x78, entries 0x18 b… |
| `00a01f40` | — | unknown | Small cleanup helper: when the pointer at this+0x14 is non-null it calls that object's virtual slot 0 (a predicate returning a char) and, when the predicate re… |
| `00a02150` | — | unknown | Removes one ClientClaimManager::ContentTracker from the hashed tracker map: finds it in bucket (key & 7) of the eight bucket chains at +0x14..+0x30 (chain link… |
| `00a025e0` | — | unknown | Per-frame watchdog over pending claim content requests: lazily initialises a cached timestamp (globals DAT_01be63b8/bc refreshed at most every ~50 units throug… |
| `00a04a30` | — 🌐 | unknown | When the manager's 'list valid' byte at +9 is clear it constructs a GetAllAvailableClaimItemsRequestPacket (009ff3c0), and when the client object at +4 has a w… |
| `00a05130` | — 🌐 | unknown | Builds a ClaimItemsRequestPacket on the stack (writing the base opcode 0x6c and sub-opcode 3 into the local packet struct under ClaimItemsRequestPacket::vftabl… |
| `00a052e0` | — 🌐 | unknown | When the source's id list pointer at +8 is non-null it generates a tracker id (00a00610), constructs a ClaimItemsRequestPacket (00a00420), sends it through 00a… |

### `ui.collection`

**CollectionCategoryManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009fd4e0` | CollectionCategoryManager::LoadCategories | confirmed | Reads CollectionCategories.txt, parses caret-delimited id/name/icon rows, inserts non-comment rows into linked hash buckets, then refreshes the category table. |
| `009fd850` | CollectionCategoryManager::Dtor | confirmed | Unregisters BaseClient.CollectionCategory, releases its data-source reference, and removes each category node from the owned hash/list table. |
| `009fd9d0` | CollectionCategoryManager::Ctor | likely | Clears manager fields, stores its client context, allocates a category data source, and retains the resulting object reference. |

**CollectionCategoryManager::CollectionCategoryDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009fce30` | CollectionCategoryDataSource::ColumnName | confirmed | Returns one of four fixed headers based on the column index: Collection Category ID, NAME, ICON_ID, or the generic Collection Category label. |
| `009fd0b0` | CollectionCategoryDataSource::GetValue | confirmed | Returns the dword at offset +0xc of the object referenced by this+0x224. |
| `009fd7a0` | CollectionCategoryDataSource::SetValue | confirmed | Gets a selected row, looks up its category node, then writes the selected category ID, localized name, or icon ID for columns 0, 1, or 2. |
| `009fd8d0` | CollectionCategoryManager::CollectionCategoryDataSource::Dt… | confirmed | Restores the data-source vtable at the primary and two embedded offsets, then destroys the embedded user-data table. |
| `009fd930` | CollectionCategoryManager::CollectionCategoryDataSource::Ct… | confirmed | Registers BaseClient.CollectionCategory, initializes the primary and two embedded data-source vtables, and stores its manager pointer at +0x254. |
| `009fd9b0` | CollectionCategoryManager::CollectionCategoryDataSource::Sc… | confirmed | Calls the collection category data-source destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |

**Engine::CollectionAddFilter** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01277730` | Engine::CollectionAddFilter::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionAddFilter::vftable into *this; when the int argument is nonzero calls FU… |
| `012777a0` | Engine::CollectionAddFilter::ActionName | confirmed | Returns the constant C string AddFilter (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionAddFilter::vftable. |
| `012777c0` | Engine::CollectionAddFilter::DebugName | confirmed | Returns the constant C string AddFilter : 0 : ...collectionhandlers.cc:277 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionAddFilter::vft… |
| `012777d0` | Engine::CollectionAddFilter::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |

**Engine::CollectionCloseCollectionManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01277e10` | Engine::CollectionCloseCollectionManager::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionCloseCollectionManager::vftable into *this; when the int argument is non… |
| `01277e80` | Engine::CollectionCloseCollectionManager::ActionName | confirmed | Returns the constant C string CloseCollectionManager (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionCloseCollectionManager::vftable. |
| `01277ea0` | Engine::CollectionCloseCollectionManager::DebugName | confirmed | Returns the constant C string CloseCollectionManager : 0 : ...collectionhandlers.cc:307 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionC… |
| `01277eb0` | Engine::CollectionCloseCollectionManager::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |

**Engine::CollectionOpenCollectionManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01277cb0` | Engine::CollectionOpenCollectionManager::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionOpenCollectionManager::vftable into *this; when the int argument is nonz… |
| `01277d20` | Engine::CollectionOpenCollectionManager::ActionName | confirmed | Returns the constant C string OpenCollectionManager (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionOpenCollectionManager::vftable. |
| `01277d40` | Engine::CollectionOpenCollectionManager::DebugName | confirmed | Returns the constant C string OpenCollectionManager : 0 : ...collectionhandlers.cc:301 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionOp… |
| `01277d50` | Engine::CollectionOpenCollectionManager::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |

**Engine::CollectionOpenPacks** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01278030` | Engine::CollectionOpenPacks::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionOpenPacks::vftable into *this; when the int argument is nonzero calls FU… |
| `012780a0` | Engine::CollectionOpenPacks::ActionName | confirmed | Returns the constant C string OpenPacks (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionOpenPacks::vftable. |
| `012780c0` | Engine::CollectionOpenPacks::DebugName | confirmed | Returns the constant C string OpenPacks : 0 : ...collectionhandlers.cc:127 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionOpenPacks::vft… |
| `012780d0` | Engine::CollectionOpenPacks::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |
| `01278170` | Engine::CollectionOpenPacks::Handle | confirmed | Virtual slot [2] of Engine::CollectionOpenPacks::vftable (79 bytes). Calls FUN_012e0ba0, then FUN_0126db60; when non-null builds ArchetypeID and Quantity keys … |

**Engine::CollectionProcessFulfillment** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01276d10` | Engine::CollectionProcessFulfillment::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string ProcessFulfillment. |
| `01276d30` | Engine::CollectionProcessFulfillment::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string ProcessFulfillment : 0 : ..\common\...\collectionhandlers.cc:160. |
| `01276d40` | Engine::CollectionProcessFulfillment::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01278250` | Engine::CollectionProcessFulfillment::Handle | confirmed | Virtual slot [2] of Engine::CollectionProcessFulfillment::vftable (104 bytes). Calls FUN_012e0ba0, then FUN_0126db60; when non-null builds ArchetypeID and Quan… |
| `01276ca0` | — | unknown | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::CollectionProcessFulfillment::vftable into the object; when the int argument … |

**Engine::CollectionQueryMoz** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012775e0` | Engine::CollectionQueryMoz::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionQueryMoz::vftable into *this; when the int argument is nonzero calls FUN… |
| `01277650` | Engine::CollectionQueryMoz::ActionName | confirmed | Returns the constant C string QueryMoz (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionQueryMoz::vftable. |
| `01277670` | Engine::CollectionQueryMoz::DebugName | confirmed | Returns the constant C string QueryMoz : 0 : ...collectionhandlers.cc:261 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionQueryMoz::vftab… |
| `01277680` | Engine::CollectionQueryMoz::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |
| `01278ff0` | — | unknown | Virtual slot [2] of its vtable (300 bytes). Reads the ArchetypeID value through FUN_012eb290, builds the QueryMoz command with FUN_012ec220, sets ArchetypeID t… |

**Engine::CollectionRedeemItem** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01276bc0` | Engine::CollectionRedeemItem::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string RedeemItem. |
| `01276be0` | Engine::CollectionRedeemItem::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string RedeemItem : 0 : ..\common\...\collectionhandlers.cc:142. |
| `01276bf0` | Engine::CollectionRedeemItem::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `012781c0` | Engine::CollectionRedeemItem::Handle | confirmed | Virtual slot [2] of Engine::CollectionRedeemItem::vftable (129 bytes). Calls FUN_012e0ba0, then FUN_0126db60; when non-null builds ArchetypeID and ChosenArchet… |
| `01276b50` | — | unknown | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::CollectionRedeemItem::vftable into the object; when the int argument is nonze… |

**Engine::CollectionReloadCollectionDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01278aa0` | Engine::CollectionReloadCollectionDisplayAction::Handle | confirmed | Virtual slot [2] of its vtable (169 bytes). Builds the string UpdateCollection and passes it to FUN_012ec220, then posts the result through FUN_012e5730 and FU… |

**Engine::CollectionRemoveFilter** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01277890` | Engine::CollectionRemoveFilter::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionRemoveFilter::vftable into *this; when the int argument is nonzero calls… |
| `01277900` | Engine::CollectionRemoveFilter::ActionName | confirmed | Returns the constant C string RemoveFilter (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionRemoveFilter::vftable. |
| `01277920` | Engine::CollectionRemoveFilter::DebugName | confirmed | Returns the constant C string RemoveFilter : 0 : ...collectionhandlers.cc:283 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionRemoveFilte… |
| `01277930` | Engine::CollectionRemoveFilter::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |

**Engine::CollectionRequestCardData** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01277490` | Engine::CollectionRequestCardData::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionRequestCardData::vftable into *this; when the int argument is nonzero ca… |
| `01277500` | Engine::CollectionRequestCardData::ActionName | confirmed | Returns the constant C string RequestCardData (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionRequestCardData::vftable. |
| `01277520` | Engine::CollectionRequestCardData::DebugName | confirmed | Returns the constant C string RequestCardData : 0 : ...collectionhandlers.cc:245 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionRequestC… |
| `01277530` | Engine::CollectionRequestCardData::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |
| `01278eb0` | — | unknown | Virtual slot [2] of its vtable (300 bytes). Reads the ArchetypeID value through FUN_012eb290, calls FUN_0126db60, builds the SendCardData command with FUN_012e… |

**Engine::CollectionRequestCollection** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01276e60` | Engine::CollectionRequestCollection::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestCollection. |
| `01276e80` | Engine::CollectionRequestCollection::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestCollection : 0 : ..\common\...\collectionhandlers.cc:176. |
| `01276e90` | Engine::CollectionRequestCollection::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01278b50` | Engine::CollectionRequestCollection::Handle | confirmed | Virtual slot [2] of its vtable (308 bytes). Reads the bool ShowUnowned through FUN_012eb310; calls FUN_0126db60 and its vtable slot +0x24; when that returns tr… |
| `01276df0` | — | unknown | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::CollectionRequestCollection::vftable into the object; when the int argument i… |

**Engine::CollectionRequestPacks** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01276fb0` | Engine::CollectionRequestPacks::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestPacks. |
| `01276fd0` | Engine::CollectionRequestPacks::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestPacks : 0 : ..\common\...\collectionhandlers.cc:191. |
| `01276fe0` | Engine::CollectionRequestPacks::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01276f40` | — | unknown | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::CollectionRequestPacks::vftable into the object; when the int argument is non… |
| `01278c90` | — | unknown | Virtual slot [2] of its vtable (267 bytes). Calls FUN_012e0ba0 and FUN_0126db60; when non-null builds the RespondPacks command with FUN_012ec220, attaches a va… |

**Engine::CollectionRequestProductTypes** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01277100` | Engine::CollectionRequestProductTypes::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestProductTypes. |
| `01277120` | Engine::CollectionRequestProductTypes::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestProductTypes : 0 : ..\common\...\collectionhandlers.cc:204. |
| `01277130` | Engine::CollectionRequestProductTypes::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01277090` | — | unknown | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::CollectionRequestProductTypes::vftable into the object; when the int argument… |
| `01279130` | — | unknown | Virtual slot [2] of its vtable (496 bytes). Reads a string at key DAT_0175b400 through FUN_012eb070, reads an integer count through FUN_012eb0a0 and fills a ve… |

**Engine::CollectionRequestTickets** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01277250` | Engine::CollectionRequestTickets::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestTickets. |
| `01277270` | Engine::CollectionRequestTickets::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestTickets : 0 : ..\common\...\collectionhandlers.cc:226. |
| `01277280` | Engine::CollectionRequestTickets::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `012771e0` | — | unknown | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::CollectionRequestTickets::vftable into the object; when the int argument is n… |

**Engine::CollectionResetFilter** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01277b50` | Engine::CollectionResetFilter::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionResetFilter::vftable into *this; when the int argument is nonzero calls … |
| `01277bc0` | Engine::CollectionResetFilter::ActionName | confirmed | Returns the constant C string ResetFilter (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionResetFilter::vftable. |
| `01277be0` | Engine::CollectionResetFilter::DebugName | confirmed | Returns the constant C string ResetFilter : 0 : ...collectionhandlers.cc:295 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionResetFilter:… |
| `01277bf0` | Engine::CollectionResetFilter::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |

**Engine::CollectionRespondTickets** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012773d0` | Engine::CollectionRespondTickets::DebugName | confirmed | Returns the constant C string kUserDataCommandRespondTicketBalanceDisplayAction : 0 : ... (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::Collectio… |
| `012773e0` | Engine::CollectionRespondTickets::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns th… |
| `01278db0` | Engine::CollectionRespondTickets::Handle | confirmed | Virtual slot [2] of its vtable (241 bytes). Reads ValueData entry 0 as an integer (type tag 2, else assert mTypeID == kIntegerTypeID at ValueData.h:0x129 and 0… |

**Engine::CollectionSetSearch** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012779f0` | Engine::CollectionSetSearch::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionSetSearch::vftable into *this; when the int argument is nonzero calls FU… |
| `01277a60` | Engine::CollectionSetSearch::ActionName | confirmed | Returns the constant C string SetSearch (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionSetSearch::vftable. |
| `01277a80` | Engine::CollectionSetSearch::DebugName | confirmed | Returns the constant C string SetSearch : 0 : ...collectionhandlers.cc:289 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionSetSearch::vft… |
| `01277a90` | Engine::CollectionSetSearch::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |

**Engine::CollectionSynchronizationReceiveInstancesDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012783f0` | — | unknown | Virtual slot [2] of its vtable (279 bytes). Reads ValueData argument 1 as an integer (type tag 2, else assert mTypeID == kIntegerTypeID at ValueData.h:0x129 an… |

**Engine::CollectionSynchronizationRemoveInstancesDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012789b0` | — | unknown | Virtual slot [2] of its vtable (233 bytes). Allocates a 0x40-byte object with FUN_00d833a6, builds a RequestCollection command through FUN_012ec110 and posts i… |

**Engine::CollectionSynchronizationStartDeliveryDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01278680` | Engine::CollectionSynchronizationStartDeliveryDisplayAction… | confirmed | Virtual slot [2] of its vtable (802 bytes). Reads ValueData entries 0 to 6 with type checks (integers 0, 1, 3, 5, 6; boolean 2; string 4). Calls FUN_0126db60 a… |

**Engine::CollectionSynchronizationStartInstancesDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01277f30` | Engine::CollectionSynchronizationStartInstancesDisplayActio… | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::CollectionSynchronizationStartInstancesDisplayAction::vftable into *this; when the int … |
| `01277fc0` | Engine::CollectionSynchronizationStartInstancesDisplayActio… | confirmed | Returns the constant C string kSynchronizationStartInstancesDisplayAction : 0 : ... (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `01277fd0` | Engine::CollectionSynchronizationStartInstancesDisplayActio… | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns th… |
| `012782c0` | — | unknown | Virtual slot [2] of its vtable (297 bytes). Reads ValueData argument 2 as an integer (type tag 2, else assert mTypeID == kIntegerTypeID at ValueData.h:0x129 an… |

**Engine::CollectionView** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01515d00` | — | unknown | Virtual slot 3 shared by Engine::CollectionView and Engine::FRTWCollectionView. Looks up an entry with FUN_012b8ff0 and returns without action when it is missi… |

**Engine::CollectionViewBase** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014356b0` | Engine::CollectionViewBase::CollectionViewBase | confirmed | Constructor (ECX = object, returns this). Stores Engine::CollectionViewBase::vftable at +0, constructs the Engine::Filter sub-object at +4 with 014384b0, the E… |
| `01435720` | Engine::CollectionViewBase::~CollectionViewBase | confirmed | Destructor body (ECX = object). Stores Engine::CollectionViewBase::vftable at +0, destroys the ArchetypeView sub-object at +0x78 with 0143b8a0, the PageFilter … |
| `014357d0` | Engine::CollectionViewBase::ScalarDeletingDestructor | confirmed | Thiscall with one byte flag argument. Calls the destructor body 01435720 on the object, calls _free on the object when bit 0 of the flag is set, and returns th… |
| `01435a20` | Engine::CollectionViewBase::WriteArchetypeListXml | confirmed | Virtual slot [1] (+0x4) of Engine::CollectionViewBase (ECX = object; param_1 is the output, param_2 the second argument, param_3 the list structure with begin … |
| `014357f0` | Engine::CollectionViewBase::WriteEntryListXml | likely | Virtual slot [2] (+0x8) of Engine::CollectionViewBase (ECX = object; param_4 is a list pointer whose begin is at +4 and end at +8 of the caller structure). Wal… |
| `01435fd0` | — | unknown | Virtual slot [3] (+0xc) of Engine::CollectionViewBase, called from 01435b80 with the element, this and the list. Calls 012b8f80 and 012ae910 and 012b8ff0, allo… |

**Engine::EngineCollectionSynchronizationCompleteInstancesDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01278130` | Engine::EngineCollectionSynchronizationCompleteInstancesDis… | confirmed | Virtual slot [2] of Engine::EngineCollectionSynchronizationCompleteInstancesDisplayAction::vftable (50 bytes). Calls FUN_012e0ba0, then FUN_0126db60; when that… |

**Engine::FRTWCollectionView** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01515c50` | Engine::FRTWCollectionView::~FRTWCollectionView | confirmed | Stores Engine::FRTWCollectionView::vftable into *this, calls FUN_01435720 (base destructor), and returns; called from EH unwind funclets. |
| `01515ca0` | Engine::FRTWCollectionView::ScalarDeletingDtor | confirmed | Virtual slot 0 of Engine::FRTWCollectionView. Stores the vtable, calls FUN_01435720, then calls _free on this when bit 0 of the second argument is set. Returns… |
| `01515bf0` | Engine::FRTWCollectionView::FRTWCollectionView | likely | Calls FUN_014356b0 on this, then stores Engine::FRTWCollectionView::vftable into *this and returns this. |

**Engine::Filter** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014381c0` | Engine::Filter::~Filter | confirmed | Destructor body (ECX = object). Stores Engine::Filter::vftable at +0, calls 014374b0 with flag 0 (clears both owned pointer vectors, deleting the elements), 01… |
| `014384b0` | Engine::Filter::Filter | confirmed | Constructor (ECX = object). Stores Engine::Filter::vftable at +0, zeroes the dwords at +8, +0xc, +0x10, +0x18, +0x1c, +0x20, +0x28, +0x2c, +0x30, +0x3c, +0x40 … |
| `01438540` | Engine::Filter::ScalarDeletingDestructor | confirmed | Thiscall with one byte flag argument. Calls the destructor body 014381c0 on the object, calls _free on the object when bit 0 of the flag is set, and returns th… |

**Engine::UICollectionSynchronizationCompleteInstancesDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01278510` | Engine::UICollectionSynchronizationCompleteInstancesDisplay… | confirmed | Virtual slot [2] of its vtable (359 bytes). When FUN_00f9c010 returns non-null posts a CloseProgressDialog command. Then allocates a 0x40-byte object with FUN_… |

**UI::CollectionManagerScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f80a0` | UI::CollectionManagerScreen::CollectionManagerScreen | confirmed | Constructor (fastcall, ECX = this): calls UI::FRTWCollectionManagerScreen::Ctor 013ff4f0 (which stores its own vtable), then stores UI::CollectionManagerScreen… |
| `012f8100` | UI::CollectionManagerScreen::DtorBody | confirmed | Non-deleting destructor body: stores UI::CollectionManagerScreen::vftable at *this, then calls 013ff550 (UI::FRTWCollectionManagerScreen::Dtor). |
| `012f8150` | UI::CollectionManagerScreen::~CollectionManagerScreen | confirmed | Scalar deleting destructor (vtable slot [0]): stores UI::CollectionManagerScreen::vftable, calls the destructor body 013ff550, then calls _free on this when bi… |

**UI::CollectionManagerScreenBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01511d00` | UI::CollectionManagerScreenBase::~CollectionManagerScreenBa… | confirmed | Stores UI::CollectionManagerScreenBase::vftable into *this, then calls FUN_012f8480 (base destructor) and returns. |
| `01511d50` | UI::CollectionManagerScreenBase::ScalarDeletingDtor | confirmed | Virtual slot 0 of UI::CollectionManagerScreenBase. Stores the vtable, calls FUN_012f8480, then calls _free on this if flag bit 0 of the second argument is set.… |
| `01511ca0` | UI::CollectionManagerScreenBase::CollectionManagerScreenBase | likely | Calls FUN_012f8420 on this, then stores UI::CollectionManagerScreenBase::vftable into *this and returns this. |

**UI::FRTWCollectionManagerScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013ff4f0` | UI::FRTWCollectionManagerScreen::Ctor | confirmed | Fastcall constructor: calls base constructor 01511ca0, then stores UI::FRTWCollectionManagerScreen::vftable at this. |
| `013ff550` | UI::FRTWCollectionManagerScreen::Dtor | confirmed | Fastcall destructor body: stores UI::FRTWCollectionManagerScreen::vftable, then calls base destructor body 01511d00. |
| `013ff5c0` | UI::FRTWCollectionManagerScreen::~FRTWCollectionManagerScre… | confirmed | Scalar deleting destructor: stores UI::FRTWCollectionManagerScreen::vftable into *this, calls 01511d00 on this, frees this through _free when bit 0 of the stac… |

**(no class)** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009fd050` | FindCollectionCategoryByKey | confirmed | Looks in the bucket selected by param_2 modulo 10, follows bucket links until key param_2 matches, then scans equivalent-key records and returns the exact-key … |
| `00a78040` | CollectionTooltip_BuildRows | confirmed | Method with this in ECX and an entry argument: sets the row count to 10 through 00a76870 or 00a75d50, resets the state fields at +0x280 to +0x294, calls 008356… |
| `00a93c80` | NotifyCollectionStarted | confirmed | Looks up CollectionStarted, builds Lua parameters from the supplied collection record, updates two tagged fields and posts StartCollection with arguments 6000,… |
| `00a93f40` | NotifyCollectionUpdated | confirmed | Builds Lua parameters from collection and locale fields, replaces tagged values, and posts UpdateCollection with arguments 6000,0. |
| `00a94380` | NotifyCollectionCompleted | confirmed | Looks up CollectionCompleted, builds Lua parameters from the supplied record, updates tagged fields and posts CompleteCollection with arguments 6000,0. |
| `00c0a5f0` | RegisterCollectionScreenModule | confirmed | Registration wrapper: pushes nup=0, the 1-entry table 0x0182a798 and the 'CollectionScreen' module-name pointer, calls FUN_00e39e40 and returns 0. |
| `009fd2a0` | InitializeCollectionCategoryNode | likely | Stores constructor arguments at +0, +4, and +0x18; installs the IString vtable at +8 and shared empty-buffer pointer at +0x0c; clears six dwords at +0x10, +0x1… |
| `009fd390` | ScalarDeletingDtor_009fd390 | likely | Calls the embedded string cleanup, frees the node when flag bit 0 is set, and otherwise returns the node pointer. |
| `00c32a70` | SendCollectionSelectionEvent | likely | Builds the HandlerCollectionScreen:SelectCollection Lua event name, allocates a one-parameter inline Lua argument array, stores param_1, and dispatches it when… |
| `00c32c20` | SendCollectionEntrySelectionEvent | likely | Builds the HandlerCollectionScreen:SelectCollectionEntry event name, puts param_1 in a one-argument inline Lua array, and dispatches it when the handler is reg… |
| `009fd0c0` | PopulateCollectionCategoryRows | speculative | Calls a data-source virtual method, iterates rows from the table at +0x254+4 using each row's +0x28 link, and submits every row's first dword through another v… |
| `009fd3e0` | — | unknown | Takes the ordered-list head at this+4, hashes node+0x20 modulo 10, and unlinks the bucket link at +0x1c when found. It repairs ordered neighbors at +0x24/+0x28… |
| `01435b80` | — | unknown | Thiscall (ECX = object, param_2 = list structure with begin at +4 and end at +8). Calls this vtable slot [14] (+0x38) with a temporary out vector, then walks t… |
| `01435ca0` | — | unknown | Thiscall (ECX = object, two args). Calls 01435b80 with the third argument, then calls the object's vtable slot [1] (+0x4) through the stored member at this+4 (… |
| `01435dd0` | — | unknown | Thiscall (ECX = object, two args). Same shape as 01435ca0: calls 01435b80 with the third argument, calls the member object's vtable slot +4 into a local vector… |
| `01435f00` | — | unknown | Two arguments. Calls 0130e6c0 (XmlBuilder-referencing helper) on a stack object, calls 01435ca0 with a 180-byte local as receiver (ECX, assumed thiscall) and t… |
| `01439700` | — | unknown | Thiscall (ECX = object, two arguments). Opens an XmlBuilder scope through 0130e6c0, calls the object's vtable slot [1] (+0x4) with a 172-byte local buffer and … |

### `ui.reward`

**ClientRewardBundle** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b8a390` | ClientRewardBundle::ClientRewardBundle | confirmed | Constructor of the reward bundle object: runs the base constructor FUN_0091c110(0,0,0), installs ClientRewardBundle::vftable and stores param_2 at +0xb8. |
| `00b8a400` | ClientRewardBundle::ScalarDeletingDtor | confirmed | Deleting destructor variant of the reward bundle: destroys the bundle through FUN_0103d010 under an SEH frame and frees it through _free when bit 0 of param_2 … |
| `00b8a460` | ClientRewardBundle_CopyCtor | confirmed | Copy constructor of the reward bundle: runs the base copy FUN_009b9a70(param_2), installs ClientRewardBundle::vftable and copies the field at source+0xb8 into … |

**ClientRewardManager** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b88ec0` | ClientRewardManager::ClientRewardManager | confirmed | Constructor: runs the base constructor FUN_008cf6e0, installs ClientRewardManager::vftable, stores param_2 at +0x38 and zeroes the fields at +0x3c and +0x40. |
| `00b88f30` | ClientRewardManager::~ClientRewardManager | confirmed | Destructor body: reinstalls ClientRewardManager::vftable and runs the base destructor FUN_008cf7a0. |
| `00b890f0` | ClientRewardManager::ScalarDeletingDtor | confirmed | Vtable slot 5 of ClientRewardManager: reinstalls the class vtable, runs the base destructor FUN_008cf7a0 and frees the manager through _free when bit 0 of para… |
| `00b899f0` | ClientRewardManager::PlayItemReceivedNotification | confirmed | Item notification: when the notification kind param_10 equals 0x10 it looks up the reward definition in the hash table at (this+0x38)->+0x20c (buckets at +0x64… |
| `00b88850` | ClientRewardManager::FindReward | speculative | Hash lookup by id: starts at bucket (param_2 % 10) of the table at this+0x14, walks the chain through node+0x1c and returns the first node whose key at node+0x… |

**PlayerRewardEvent** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b8a4d0` | PlayerRewardEvent::PlayerRewardEvent | confirmed | Constructor of the player reward event: runs the base constructor FUN_008b8160, installs PlayerRewardEvent::vftable and constructs the embedded ClientRewardBun… |
| `00b8a560` | PlayerRewardEvent::~PlayerRewardEvent | confirmed | Destructor body of the player reward event: resets to PlayerRewardEvent::vftable, destroys the embedded ClientRewardBundle via FUN_0103d010, resets to PlayerEv… |
| `00b8a5d0` | PlayerRewardEvent_ScalarDeletingDtor | confirmed | Deleting destructor of the player reward event (vtable slot 3): runs the destructor body 00b8a560 and frees the event through _free when bit 0 of param_2 is se… |
| `00b8a5f0` | — | unknown | Vtable slot 1 of PlayerRewardEvent: with bit 0 of the argument set it resolves the local player through FUN_00ae2470, and when the player exists and has a non-… |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b886a0` | — | unknown | Cleanup helper that destroys the embedded ClientRewardBundle at the caller's object by invoking FUN_0103d010 (the bundle base destructor) under an SEH frame. |
| `00b896b0` | — | unknown | Fills seven notification rows of a 0xa0-stride table (state field at +0x140 of each row set to 1, value field at +0x148 = the passed values, the time-based val… |
| `00b89ca0` | — | unknown | Reward bundle presentation: builds a 'target' string, reads the bundle's XP amount (virtual slot 0x3c of param_2, adjusted by 00be8980 through the player objec… |

## Source shards

[[S0242 0091af50-009200d0|S0242]] (41), [[S0265 0098ec00-00995660|S0265]] (3), [[S0284 009f8470-009feec0|S0284]] (14), [[S0285 009fef30-00a059e0|S0285]] (73), [[S0306 00a75b70-00a7b4d0|S0306]] (1), [[S0311 00a91bf0-00a97400|S0311]] (3), [[S0357 00b84e90-00b8a5f0|S0357]] (15), [[S0358 00b8a640-00b906a0|S0358]] (1), [[S0365 00bac0d0-00bb1110|S0365]] (2), [[S0383 00c05780-00c0db00|S0383]] (1), [[S0388 00c2df40-00c34100|S0388]] (4), [[S0421 00ce0340-00ce4c30|S0421]] (6), [[S0568|S0568]] (127), [[S0572|S0572]] (10), [[S0665 01271690-01277340|S0665]] (24), [[S0666 012773d0-0127d160|S0666]] (57), [[S0680|S0680]] (21), [[S0689|S0689]] (3), [[S0721|S0721]] (5), [[S0732|S0732]] (43), [[S0738|S0738]] (2), [[S0739|S0739]] (1), [[S0749|S0749]] (14), [[S0790|S0790]] (7)
