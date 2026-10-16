---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Quests and Objectives

Part of [[Build Guides]] › Gameplay. Related: [[Rewards and Collections]], [[Actors and NPCs]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`) are the native-map labels for a function. Source tags: *client* = read from the shipped executable in a shard or in this pass; *client Lua* = the client's own UI script bytecode, losslessly decompiled in the lab (not redistributed); *client locale* = the shipped text table; *video* = a reviewed frame record (48 batch files, 5,760 records, 656 videos); *wiki* = the fan wiki; *vault* = a project research note, used here only for its client-derived facts. Precedence is client, then video, then wiki. *(R)* marks reconstructed server policy that is ours, not original behaviour. No original server code exists, so every statement about what the server did is a hypothesis built from what the client requires. No player names or chat are recorded; NPC names are. The community emulators' quest, hub and sticker tables are not used anywhere in this guide. OSFR-held data (packet captures and server JSON) is not used. The retail hub, region and sticker set is therefore left open (see Open questions). Related: [[Rewards and Collections]], [[Actors and NPCs]], [[Items and Inventory]], [[Housing]], [[Minigame Framework]], [[Matchmaking and Lobby]], [[Social (Friends, Guilds, Chat)]], [[Scripting (Lua)]], [[Text and Localization]], [[Scaleform and Flash UI]], [[Gateway and Transport]], [[Zones and Terrain]], [[Content]].

### Overview

Five separate player-facing systems share this guide. They are separate in the client's code and on the wire, so they are listed apart.

1. **Quests** (family 49): offered by NPCs, tracked in the quest helper, listed in the Hero's Journal.
2. **Goal panels and objective targets** (family 47) and **objectives** (family 45): the "Goals" list for activities and minigames, the map pin, and the tutorial's scripted client-side goals.
3. **The Adventurer's Journal** (family 209): the data behind the Hero's Journal book (chapters, hub map, stickers).
4. **The tutorial** (the "NPX" new-player flow and the directed play prompts).
5. **The Game Action Queue** ("Gaq"): the pop-up queue for invites and notices. It is not a quest system (see Open questions).

What the player sees:

- **Quest givers.** An NPC with a quest shows a green exclamation mark over its head (wiki; client locale tutorial text says the same and "interact with me to start the quest"). On a world frame the interaction prompt reads "Press X or mouse" (video). Quest kinds named on the wiki are Combat, Explorations, Minigames, Collections, Brain Teasers and Scavenger Hunt.
- **The offer.** Interacting opens a start screen with the NPC's intro text and two buttons, accept ("Yes, I accept!") and decline ("No! I decline."). In the tutorial the decline button is removed (client Lua `QuestHandler:AddDialogResponseNodes`; client locale ids 71313 and 71314; vault Pieces 071). The server can also add a quest with no offer at all (vault: the reconstructed opening does this and the client accepts it).
- **Quest started.** A "Quest Started!" banner and chat line, then a "New Goal" line for each objective as it becomes active, a "Goal Complete" banner with a check mark when one finishes, and a "Quest Completed!" banner with floating reward lines such as "You receive 12 stars and 54 coins" or "You receive 1 Blast Speedster" (client locale; video: completion banners in 22 videos, 174 "Goal Complete" frames in 58 videos).
- **The quest tracker (quest helper).** A panel at the right edge shows the selected quest's title, description and a "Goals" list with ticks and counts (video: "Harvest Bumbleberry - 20/25"). Arrows scroll between active quests, the panel can be minimized, and a link opens the Hero's Journal (client Lua). The wiki calls this the Mini Hero's Journal.
- **Active Quest markers and Take Me There.** The quest target is shown as a green marker labelled "Active Quest!" with a distance in metres over the objective NPC or place, and the tracker gets a "Take Me There" button (video: 147 frames in 27 videos). The client's tutorial text says the button makes the character run to the quest location automatically (client locale). The world map has a "Show Active Quest" toggle, a "Make Active Quest" action and the text "No Active Quests!" (client locale; video, 83 videos).
- **The Hero's Journal.** A main-menu tile and the L key open a book (wiki; the original key table confirms physical L, per [[Native Quest Journal UI]]). Chapter tabs run along the edge; each chapter shows a map with hub icons, each hub showing its current quest; a sticker page shows collectible tiles; an "Other Quests" tab lists every active quest that belongs to no hub, with an "n / limit" count (client Lua; our own native run showed "1/30" only because our server sent 30, R). The first quest of a new character is itself called "Hero's Journal": enter the caverns, find the book, interact with it, close it (video, one session, goals ticking off in order).
- **Terms.** In the client's own words a *chapter* is a region tab, a *hub* is a map node with a current quest, and a *sticker* is a tile tied to a quest id. Some quest titles are also chapter-numbered, for example a "Chapter 1:" prefix (client locale has 49 entries containing "Chapter "; video: 15 frames in 5 videos). Which retail quests were chapters, hubs or stickers is not in the client.
- **Quest limit.** A non-member who hits the limit gets a red chat line, and a member gets a different line (client Lua, client locale: "You can't start any more quests! Become a member to raise your quest limit.").
- **Job quests.** The job browser's "start quest" button and the world map's breadcrumb button both begin a breadcrumb quest (client Lua), which is how new jobs are unlocked.
- **Rewards.** Stars (the experience currency label), coins, items and "Mystery Gift" boxes (video: a "You receive" line appears in 221 frames across 99 videos). Reward amounts per quest are not recoverable from the client, only from video samples.
- **Series and requirements on the wiki.** Quest pages carry previous, next and series fields and a coin amount. Requirements are: a previous quest, an active membership, a combat job level, a glide-trainable ride, a one-day cooldown after a prior quest, or an event window (vault Pieces 067 classifies 36 pages; examples re-read here: Horrible Hiccups needs membership, Glide Training: Basic Training needs a day after Getting Started, Winter Party runs only during the event).
- **Directed play prompts.** First-time events named `DPPHousingTakeMeThere`, `DPPBattlesTakeMeThere`, `DPPPetsTakeMeThere` and `DPPMerigoldTakeMeThere` teach a feature by handing out a quest and pointing at its Take Me There button (client resource `FirstTimeEvents.txt`, client locale tutorial lines).

### How the client implements it

#### Main classes

| Class or object | Role | Evidence |
|---|---|---|
| Quest manager (data store; constructor `00c7ca80`) | Lives at the player object plus `0x1add0` (thunk `00a107f0` adds that offset before calling the dispatcher `00c7cd40`). Holds a 256-bucket quest map keyed by the low byte of the quest id (`00c7a610`; node key `+0x190`, chain `+0x194`), the selected quest id (`+0x10f4`), the auto-switch flag (`+0x10f8`), the completed-quest count (`+0x10fc`) and two more dwords (`+0x1100`, `+0x1104`, see Open questions). | confirmed constructor; offsets from `00940c60`, `00c7cd40` |
| `ClientQuestData` runtime record | One accepted quest. Id at `+0`, locale ids for title and description at `+4` and `+8`, members-only byte `+0x10`, profile reference `+0x20`, overall progress as a 32-bit float `+0x28`, an embedded reward bundle at `+0x30` and the objective list at `+0xe8` (16 buckets), icon id `+0x150`, system-quest byte `+0x154`, three resolved strings (`+0x158`, `+0x168`, `+0x178`), hash-node key `+0x190` and chain `+0x194`. The map names the quest-level copy constructor `ObjectiveData_CopyCtor` (`00baf380`), so "ObjectiveData" in the map is used for both levels. | `00baf490`, `0091c940`, `00a43240`, `00a43270`; field names from [[Native Quest Journal UI]] column proofs |
| Objective node | State `+0xd0` (1 active, 2 complete, 3 failed), current counter `+0xd4`, required count `+0xd8`, one more dword `+0xdc`, runtime key `+0xf0`, embedded reward bundle at `+0x18`. | `00c7a910`, `00c7a990`, `00c7aa10`, `00c7aa90`, `00922810`, `0091c8b0` (confirmed or likely) |
| `RewardBundleBase` / `ClientRewardBundle` | The reward object embedded in quest packets and records: XP/stars at `+0x48`, coins at `+0x50`, kind at `+0x54`, an item list at `+0x38`. | `0091c110`, getters `0091c290`, `0091c2e0`, `0091c380` (S0242) |
| Quest, objective and journal data sources | The tables the Lua panels bind to. Quest: `BaseClient.QuestData` (13 columns), `BaseClient.QuestObjectiveData`, `BaseClient.MiscQuestObjectiveDataBook`. Journal: `BaseClient.AdventurersJournal.RegionData`, `.HubData`, `.StickerData`, `.MiscQuestData`. Goals: `BaseClient.ObjectiveHelper`, `BaseClient.ObjectiveTarget`. Columns are listed under Data and formats. | constructors confirmed (`00cf81f0`, `00c98ad0`, `00c98b80`, `00c98c30`, `00c98ce0`, `00cb83d0`, `00cb89a0`) |
| `ClientObjective` list and its four subclasses | Client-side scripted goals for the tutorial: look-at (`00cf35e0`, `00cf3890`), UI-event (`00cf3920`), first-movement (`00cf3a00`), housing-fixture-move (`00cf3aa0`). A 100-bucket registry keyed by objective id, entry flags enabled (`+0x1c`) and completed (`+0x1d`). | confirmed (S0392, S0424) |
| `ClientTutorialManager` (constructor `00a46cd0`) | Created when the tutorial flag turns on. It only registers itself and preloads the asset list `GamePreLoadAssets.txt`; it does no hotkey, camera or job-switch gating. | confirmed constructor; [[Original Tutorial Mode Exit]] |
| `GaqManager` and `GaqItem*` classes | The Game Action Queue: a 64-bucket map of item objects (`00a091e0`) with typed items for guild, house, friend, group, trade, duel, encounter, mini-game and claim actions. | confirmed (S0286, S0287) |
| Lua objects `QuestHandler`, `QuestHelper`, `StoryBook`, `HerosJournalButton`, `Atlas`, `MiniMap`, `TutorialArrow`, `FirstTimeEvent` | The presentation layer. Native code calls them by name through the Lua bridge `009797a0`. | client Lua; `009797a0` confirmed |
| Native Lua module `QuestScreen` (registrar `00c0a020`, table `0182a6f8`, 11 entries) | The only native entry points the quest UI uses; decoded from the binary's own table. Listed next. | table read from the binary |

The 11 `QuestScreen` entries are `AcceptQuest` (`00c0a040`), `DeclineQuest` (`00c0a060`), `SelectQuest` (`00c0a080`), `AbandonQuest` (`00c0a500`), `SelectQuestInHelper` (`00c0a0e0`), `EndQuest` (`00c0a1a0`), `GetSelectedQuest` (`00c0a1c0`), `SetAutoSwitchSelectedQuest` (`00c0a200`), `GetAutoSwitchSelectedQuest` (`00c0a260`), `GetQuestLimit` (`00c0a2a0`) and `SelectMiscQuestFromBook` (`00c0a140`) (client).

#### Offer, accept and start

1. **Offer.** The client does not decide that a quest is offered. The server sends **49/1** (QuestInfo; reader `00c7bb60`, constructor `00c7be00`). The dispatcher `00c7cd40` case 1 calls `ShowQuestStartScreen` (`00a94be0`), which fills Lua fields, formats the `QuestNpcIntroTextMessage` string with quest name, player and message, and emits `QuestHandler:ShowStartScreen`. The packet object carries six dwords, two bytes, an embedded reward bundle and trailing fields; the wire order is not decoded (Open questions).
2. **Accept or decline.** Lua calls `QuestScreen.AcceptQuest` or `DeclineQuest`, which call the senders on the UI processor object (client `+0x274`). Accept builds a QuestReply (**49/2**: `i32` quest id, `u8` accept = 1; `00a97d70`, constructor `00a86400`, body writer `00a91630`). Decline builds the same packet with accept = 0 and also resets the handler state (`00a97e10`; the zero is a pushed literal in the bytes). The id is read from `+0x1c` of that object; that it is the offered quest id is inferred from the packet it builds, because the writer of that field was not traced. The packet is sent only when the client's connected gate (`+0x280`) is set, wrapped in a tunneled client packet (`00a96360`).
3. **Add.** The server answers with **49/3** (QuestAdd: a full quest record plus a select bool). The dispatcher inserts it through `ApplyQuestAddPacket` (`00c7bd10`), then calls `NotifyQuestStarted` (`00a92680`), which posts the `StartQuest` Lua event (arguments 6000 and 0) and, when the select bool is set, `QuestHandler:PopulateHelper` (`00a8f2b0`). If the quest id is also a hub-quest definition in the journal, `00a44bb0` additionally raises `CallHandler:ShowAdvJournal`, which shows the Hero's Journal button unless the tutorial is on.
4. **Key normalisation (important).** The full record carries a raw key per objective. When the client copies the decoded quest into the manager (`009225d0` to `0091bb40` to `00baf490` to `00baf380`), it re-keys every objective by its **definition id**. Every later objective message must therefore use the definition id, not the position in the list. This is proven in the bytes (`push edi; push edi; call 00baba60` at `00baf458`, then `mov [esi+0xf0]` at `00bab990`) and in [[Native Journal Live Objective Keys]].
5. **Login.** The self-data block sent at login holds the same records (readers `00940c60`, `00940e60`; the first clears and rebuilds the list and writes `+0x10fc`, `+0x10f4`, `+0x10f8`, `+0x1100`, `+0x1104`; the second writes only `+0x1100`).

#### Progress, completion, failure

| Sub | Client effect (dispatcher `00c7cd40`) |
|---|---|
| 7 ObjectiveAdded | `ApplyAddedObjective` (`00c7c660`) inserts one objective into an existing quest; `NotifyNewQuestObjective` (`00a92ee0`) formats the `QuestNewObjective` text (the "New Goal" line in video) and posts `QuestHandler:UpdateObjective`. |
| 8 ObjectiveActivated | `ApplyObjectiveActivated` (`00c7a910`): finds the quest and the objective by key, sets state 1 and the required count. |
| 9 ObjectiveUpdate | `ApplyObjectiveUpdate` (`00c7a990`): writes the counter to the objective (`+0xd4`) and the progress float to the quest (`+0x28`). If the objective key is not found the quest progress is still written. `NotifyQuestObjectiveUpdated` (`00a931b0`) builds the chat message and refreshes the helper. |
| 10 ObjectiveComplete | `ApplyObjectiveComplete` (`00c7aa10`): sets state 2 and the progress; `NotifyQuestObjectiveCompleted` (`00a936e0`) posts `CompleteObjective` (the green "Goal Complete"). |
| 11 ObjectiveFailed | `ApplyObjectiveFailed` (`00c7aa90`): state 3, no progress change; `NotifyQuestObjectiveFailed` (`00a939b0`). |
| 12 CompletedQuestCountUpdate | Assigns the manager's completed count (`+0x10fc`). |
| 4 QuestComplete | Looks the quest up, removes the record (`00c7c130`), fixes the selection (`00c7ab20`), `NotifyQuestCompleted` (`00a928b0`, "Quest Completed!"). |
| 5 QuestFailed / 6 QuestAbandoned | Same removal path (`00c7c190`, `00c7c1f0`); notify functions `00a92ac0` and `00a92cd0`. Abandoned reuses the `FailQuest` event. |
| 13 QuestEnd | Calls `ShowQuestEndScreen` (`00a95420`): formats `QuestNpcEndTextMessage` and emits `QuestHandler:ShowEndScreen`. The end screen has one button, "Thanks!". |
| 16 QuestEndBlocked | Logs "Received QuestEndBlockedPacket, text id %d, quest id %d" (to `ClientQuest.log`), looks the text id up in the locale and, if found, shows it in a message window. |

Every notify function has an optional "silent" argument: the trailing bools of 49/9 and 49/10 and the select bool of 49/3 decide whether the banner, chat line or helper refresh is produced (client: each notify function skips its string work when that argument is non-zero; the bool-to-argument mapping is the vault's reading; for 49/4 the argument is a byte stored in the quest record at `+0x24`, read in `00c7cd40` case 4, whose source on the wire is not traced). Subtypes 2, 14 and 15 have no dispatcher case, so they are client-to-server (S0402).

The turn-in path is therefore: server sends 49/13 (end screen), the player presses "Thanks!", Lua calls `QuestScreen.EndQuest(true)`, the client sends **49/14** (QuestEndReply: `i32` quest id; `00a97ef0`, constructor `00a864c0`), then the server grants rewards and sends 49/4 and 49/12. The order of those last steps is our reading of the client (R); the client handles each message independently.

#### Selection, the helper and the limit

- **Selection.** `SelectQuest` and `SelectMiscQuestFromBook` select a record locally by id (`00c7a890`). `SelectQuestInHelper` selects locally and, when the selection changed, sends **47/12** SelectQuest (`i32` quest id; `00c7c540`, writer `00c7c250`, constructor `00c79e40`), gated on the connected flag. The auto-switch flag is sent as **47/13** SelectedQuestLocked (one byte, inverted: `00c7c5d0`, writer `00c7c360`). When a quest is removed and it was selected, the client picks the remaining record with the largest progress (`00c7ab20`).
- **Limit.** `QuestScreen.GetQuestLimit` returns `player+0x1bed4` when the dword at `player+0x464` is non-zero, otherwise `player+0x1bed0` (`00a10890`, bytes read directly). That field is set by the inbound `PacketMembershipActivation` (id 202 / `0xca`; constructor `0089bab0`), whose handler stores one body value there through `00a12470` and then raises `MembershipHandler:OnMembershipActivated` (`009471a0`). The manager is at `player+0x1add0`, so `+0x1bed0` and `+0x1bed4` are exactly the quest manager's `+0x1100` and `+0x1104`, the last two integers of the self-data quest block. They are therefore most likely the non-member and member quest limits (client bytes). The original values are unknown: the only source for them was recorded server payloads, which are not used, and our native run's "1/30" display just shows the client reads the member limit our server sent (R). This corrects the vault's earlier note (Pieces 070) that nothing reads them back.
- **Limit message.** Server message **47/15** carries one byte, 1 or 2: 1 prints a red `QuestLimitMember` line; 2 prints a red `QuestLimitNonmember` line and raises `QuestHandler:NotifyNonmemberQuestLimitReached` (`00a91bf0` case 15). The numeric limit itself is not in that packet.

#### Objective targets, Take Me There and breadcrumbs

- **Target pin.** **47/14** ObjectiveTargetUpdate (`00a88080`, handler `00a8fab0`) feeds the `BaseClient.ObjectiveTarget` data source (columns Active, Guid, Name, Location X, Location Z, Zone ID, Target Type, Screen Angle). When it updates, Lua refreshes the atlas quest path and `MiniMap:SetObjectiveTarget`, which shows or hides the "Take Me There" button and drops the minimap pin (client Lua, unless the tutorial is on). The wire order of the 47/14 body is not decoded; the packet object holds a byte, three dwords, an 8-byte guid, a dword and a float.
- **Take Me There.** The helper's `takeMeThere` event calls `Ui.TakeMeToObjective` (`00c17c70`), raises the client objective event `onTakeMeThere` and hides the tutorial arrow. Natively `TakeMeToObjective` restores a stored target vector into the point-path controller at `player+0x1ec` (`008a8e50` to `009cef10`); it builds no quest packet. The controller may issue its own click-to-move path request (the main dispatcher logs click-to-move "Reply for ... target" lines); that was not traced. The vector is stored by the same controller (`009ceee0`, `009ce260`, and `Ui.TakeMeToPointOfInterest` for map points); where the objective pin writes it was not traced.
- **Breadcrumbs.** `Ui.StartBreadcrumbQuest(questId)` (`00c18aa0`) sends **49/15** QuestStartBreadcrumb (`i32` quest id; `00a97f90`, constructor `00a86580`). Callers are the job browser's start-quest button (the quest id is column 4 of the activity-finder row), and the world map's "showPOIBreadcrumbs" button (point-of-interest column `BreadcrumbQuestId`). The client draws a path from the `BaseClient.Breadcrumb` data source (X, Y, Z rows; 20 are drawn on the minimap), and `SetBreadcrumbTarget` (`009cd6c0`) issues a path request toward a target vector. Which packet fills that data source was not found in this pass.

#### The three goal systems

The client keeps three goal lists. They look alike on screen (a "Goals" panel with ticks) but are fed differently; which one a given video frame shows is not always provable.

| System | Fed by | Held in | Used for |
|---|---|---|---|
| Quest objectives | 49/3, 49/7 to 49/11 | the quest manager | Quest helper and Hero's Journal |
| Task goals ("HUD objectives") | **47/1** TaskAdd, **47/2** TaskUpdate, **47/3** TaskComplete, **47/4** TaskFail, **47/5** window hide, **47/10** StartTimer | `BaseClient.ObjectiveHelper` rows (`00a88220`, `00cb7e50`, `00cb7f20`, `00cb7ff0`, `00cb7e20`) | Activity and timed goals (likely; the "Coin Clean-up" countdown goal in video fits but is not proven); columns include category prefix, goal count and total |
| Minigame goals | **45/1** Activate, **45/2** Update, **45/3** Complete, **45/4** Fail, **45/5** Unhide | the minigame state's goal array (0x110-byte rows; `00c42870`, `009b6700`, `009b6950`) | Minigame results and scores; only handled while a minigame state exists |

Task goals show a category prefix chosen by a category number: Primary, Secondary, Job, Bonus or Elite (`00a872f0`, localized keys `ObjectiveCategoryPrefix*`) and a status text with or without count and total (`00a8b9a0`). The minigame goal formatter has six statuses: NotStarted, InProgress, Completed, Failed, failure-condition, MembershipRequired (`00c42440`).

Quest objectives and the 45-family objectives are different systems. Do not reuse the quest objective-key rules for 45/x, or the reverse (client; S0392).

#### Client-side scripted objectives and the tutorial

- **Client objectives.** The server registers a client-evaluated goal and the client reports completion. Registration packets (family 45): **45/6** LookAt (actor guid or world position, plus range and angle; `00c435c0`, `00c436b0`), **45/10** UIEvent (objective id and an event-name string; `00c437a0`), **45/11** FirstMovement (id and an is-mouse bool; `00c43870`), **45/12** HousingFixtureMove (id, fixture id, action id; `00c43940`, zero means wildcard). The server controls them with **45/8** ClientClear (disable) and **45/9** ClientCompleteFailed (re-arm so it can be retried). The client sends **45/7** ClientComplete (`u16` 45, `u8` 7, `i32` objective id, 7 bytes) when a match occurs (`00cffba0`). There is no duplicate check on registration (client, `00c43140` and the add paths); the wire formats are from [[Original Opening Microtutorial Recovery]] (vault, native readers).
- **UI events that can complete a UI-event objective.** The client Lua raises seven event names through `Ui.OnObjectiveEvent`: `onMarketplaceOpen` (two sites), `onTakeMeThere`, `onStoryBookClose` (only when the tutorial flag is on), `onPlayMoreHide`, `onMinigameClose`, `onHousingBrowserOpen`, `onAskForHelpOpen`.
- **The tutorial flag.** `Ui.IsInNPXTutorial` reads the byte at `player+0x780`. The 43/31 zone message sets it (`008a89a0`, which also creates the manager at `player+0x77c`) and never clears it; encounter-enter 41/2 writes it from a body byte and disposes the manager (`00aa38b8`, `00aa38ce`); `Ui.TutorialCancelled` clears only the flag and its one Lua call site is the minigame lobby close ([[Original Tutorial Mode Exit]]).
- **Scripted prompts.** Message **107/5** carries a script name, integer parameters, an event id and a clear flag; the client splits `object.method` and calls it through the Lua bridge with no whitelist (`00c893f0`). The `FirstTimeEvent` Lua table holds the tutorial steps: `FtesNpxDppStart`, `Camera1` to `3`, `Movement1` to `3`, `Enemies`, `Combat1`, `Potions`, `Abilities`, `Combat2`, `Interact`, `Tome`, `BookQuest`, `BookSticker`, `BookTab`, `BookExit`, `DppQuests`, `DppReturnHud`, `DppEnd`, `DppHide` (client Lua; the order of execution was not recovered). `FtesNpxBookExit` shows the journal and allows it to be closed; `FtesNpxDppEnd` restores hotkeys and job switching.
- **Not the same tutorial.** `GameTutorialData.txt` (18 rows) lists minigame tutorial movies (artifact, farming, forging, mailsorting, mining, cooking, music, smelting, sword, chess, checkers, pet gestures, mahjong, Halloween, soccer, hidden object, tower defense, racing). `TutorialDB` (`01406d50`) is a rule-set store for the card game; the map labelled it `game.tutorial` by name only (see Open questions).

#### The Adventurer's Journal

- **Messages.** Family 209 (`0xd1`) is dispatched to `00a45d80` on the object at client `+0x23c` (message id 209 resolves through the jump table of `009471a0`). The layout is `u16 id`, `u32 sub-kind`, body. Sub-kind 1 builds a new 0x250-byte definition block (`00a459c0`), freeing the old one; sub-kind 2 replaces a 32-bucket integer map (`00a45680`) and then re-picks the selection. Both are whole-snapshot replacements, not deltas. The body must be consumed exactly (`00a45d10`, `00a45bf0`).
- **Definitions.** Four tables, each a count followed by entries of one `u32` key and a fixed number of `u32` fields: region 7, hub 10, hub-quest 3, sticker 9 (readers `00a45740`, `00a457e0`, `00a45880`, `00a45920`; field readers `00a442c0`, `00a443d0`, `00a44540`, `00a445c0`). The hub-quest fields are stored out of order, at `+4`, `+0` and `+8` of the node.
- **Selection logic.** The selection code (`00a44020`, `00a44100`, `00a447d0`, `00a43f00`; all `likely`) walks the hub-quest list for entries whose first stored field equals the selected chapter id and looks each entry's second stored field up in the sub-kind 2 set. It keeps the lowest-order entry (third field) that is not in the set, requiring a live quest record when asked, and falls back to the highest-order entry that is. The labels call the stored result a "region id", so what exactly is selected (a hub, a quest or a chapter) is unresolved. The reading that fits is: the hub-quest list is an ordered chain per chapter, and the sub-kind 2 set holds the keys already completed. The value paired with each key is not used in anything traced.
- **Views.** Region data feeds the chapter tabs (name, description, tab image, chapter-map image, completed text, should-be-hidden). Hub data feeds map icons (image set, map X and Y, current quest title and description, completed description, active, complete). Sticker data feeds tiles (name, description, image set, owning quest id, complete). Misc-quest data is every live quest absent from the hub-quest map (`00c97d70`, `00c97de0`): it does not test members-only, system-quest, profile or state ([[Native Quest Journal UI]]).
- **Opening the book.** `StoryBook:Show` enables script events on the hub, sticker and misc data sources and rebuilds all pages. The "Hero's Journal" button is shown when the hub data source has at least one row, and also by the show-journal call raised when a hub quest is added (client Lua). Closing it restores movement and, in the tutorial, raises `onStoryBookClose`.
- **Area link.** Zone areas can carry an "AdvJournal" property (`ClientAreaAdvJournalProperty`, `00bea660`), which suggests an area can bind to a journal region; what it stores was not traced.

#### The Game Action Queue

Every GAQ item is created locally by the client in response to some other message, then shown through the Lua `Gaq:AddGameAction` (`ScriptsBase.lua`, `GaqItemManager`). The client Lua list of types is: GuildRenameRequest, GuildInvite, HouseInvite, FriendInvite, ChallengeNotification, ChallengeResult, ClaimNotification, TextNotification, GroupInvite, TradeInvite, GroupAnnounceEncounter, DuelRequest, FriendInviteToGame, GameTopScore, PersonalHighScore, FriendBeatHighScore, ItemReceived, NameChange, NameChangePending, Claim and MOTD. Invites become pop-up items with accept and decline responders (status 1 and 0): for example the house invite's decline sends the housing decline `127/16` and its accept sends the housing enter `127/19` (`00a09550`, `00a09580`); trade, group, encounter, duel and mini-game invites forward to their own senders (`00c6dbe0`, `00a983b0`, `00a97720`, `00a979a0`, `00a97c10`, `00a97cc0`). Four builders (guild, house, friend, group) auto-decline by calling the new item's decline responder when the guard `00917b60` returns non-zero; what that guard tests is unrecovered. Scores go to the communicator and notices go to the system-message line. Items can also expire (`00a0df40`) or be purged by key (`00a0df80`, `00a0e030`, `00a0e770`). `ItemReceived` is created from the coin-store gift handler and the transaction-complete handler (`00a0c3a0` via `00c764f0`, `00c78220`). No GAQ packet family of its own was found; the queue is fed by the social, housing, trade, guild, encounter and coin-store messages.

#### Progressive quests (adjacent)

Family 207 (`0xcf`) drives a timed key-and-chest program shown by the Mystery Reward Browser. The client holds a definition (id, name, seconds remaining, active slot, three objective string ids, button text ids), a slot list (item, key, "can do quest", key bundle id and price, redeem) and a prize list (progress percent, redeem). Inbound (dispatcher `00bde9c0`, wire sub-opcode is the u32 after the u16 family): **1** client data (`00bde8a0`: eight dwords plus a slot array and a prize array), **5** show the window (`00bde3d0`, Lua `MysteryRewardBrowser:Show`), **6** order response (`00bde510`, Lua `CallHandler:OnOrderResponse`), **7** reward item notice (`00bde610`, Lua `UnifiedMessageWindow:ShowItemPanel`). Outbound requests start, claim a slot and claim a prize (Lua handlers `00c24400`, `00c244d0`, `00c245a0`, two numbers each, packet classes `ProgressiveQuestRequestStartQuest`, `...ClaimSlot`, `...ClaimPrize`; their sub-opcodes were not read). It shares the name "quest" but not the quest runtime.

### Data and formats

**What the client ships and what it does not.** The client has **no quest, objective, hub, region or sticker definition table**. Titles, descriptions and objective text arrive as locale ids inside the server's records and are resolved through the locale manager. The client locale (113,352 entries) holds quest titles and quest text as separate entries with no link between them, and each title appears twice under two tags (`ucdt`, `ugdt`); the meaning of the two tags is not recovered. The [[Kart Driver]] content page notes the same missing link. Resource files that do exist and touch this area:

| File | Contents | Source |
|---|---|---|
| `FirstTimeEvents.txt` | 92 rows of `ID^NAME^`. Quest-related names: `GamedockMyQuests` (6), `FtesQuestHelperTut` (44), `DPPClickToMove` (107), the `DPP*TakeMeThere` and `DPP*Completed` family (108 to 122), `DPPFailed` (120). | client resource |
| `GameTutorialData.txt` | 18 rows of minigame tutorial movies (not the new-player flow). | client resource |
| `ActivityCategories.txt`, `ActivityPortalSections.txt` | 16 and 10 data rows; activity-finder categories (content id, name id, description id, thumbnail) and portal sections. | client resource |

**Quest data source columns (13).** Quest Id, Quest Title, Quest Description, Time Started, Completed Percentage, Job Background Badge Id, Job Icon Id, Job Level, Objective Count, Members Only, Helper Text, Icon Id, System Quest (`00cf7a10`, client Lua `QuestDsColumn`). Objective source (6): Name, Description, Status, two unused columns and Id (`00cf7ae0` names the unused ones "NotUsed"; the Lua names them RewardName and RewardIcon).

**Journal data source columns.** Region (8): Region Id, Name, Description, Tab Image Id, Chapter Map Image Id, Is Selected, Completed String Id, Should Be Hidden. Hub (11): Hub Id, Hub Name, Image Set Id, Current Quest Id, Current Quest Title, Current Quest Description, Completed Description, Is Active, Is Complete, Map X, Map Y. Sticker (6): Sticker Id, Sticker Quest Id, Sticker Name, Sticker Description, Image Set Id, Is Complete. Misc quest (10): Quest Id, Title, Description, Time Started, Completed Percentage, Job Badge, Job Icon, Job Level, Objective Count, Members Only (client Lua; `00c97710`, `00c977a0`, `00c97850`). The wire fields are not the columns: for example a region entry is one key plus seven dwords against eight columns, and a hub entry is one key plus ten dwords against eleven columns. Which wire field is which column is not recovered; the live-quest columns (current quest, active, complete, selected) are most likely derived on the client from the quest manager and the sub-kind 2 set.

**Goal data sources.** `BaseClient.ObjectiveHelper` (9 columns): ID, Text, Icon ID, Is Bonus, Status, MembersOnly, Category Prefix, Goal Count, Goal Total; each row is 0x2c bytes (`00cb7710`, `00cb7cf0`). `BaseClient.ObjectiveTarget` (8 columns) is listed under Take Me There.

**Full quest record (wire order, vault, cross-checked with the client reader `0093c840`).** Field names are ours.

| Field | Type | Notes |
|---|---|---|
| quest id, title locale id, description locale id | `i32` x3 | runtime `+0`, `+4`, `+8` |
| helper-text locale id | `i32` | column "Helper Text" |
| members-only | `bool` | runtime `+0x10` |
| time started | `i64` | zero in the vault's records; the data source has a "Time Started" column |
| profile reference | `i32` | job badge, icon and level lookup; zero gives a neutral look; runtime `+0x20` |
| one more flag | `bool` | meaning not recovered |
| overall progress | `f32` | runtime `+0x28` (`Completed Percentage` column) |
| reward bundle | variable | see below |
| objective count, then per objective: key, definition id, name locale id, description locale id (`i32` x4); one `bool`; reward bundle; state, current, required, one extra (`i32` x4); one `bool`; one `i32` | | key is re-keyed to the definition id at runtime |
| icon id | `i32` | runtime `+0x150` |
| system quest, one more flag | `bool` x2 | runtime `+0x154` for the first |

The self-data quest block is `i32 quest count`, the records, `i32 completed count`, `i32 selected quest id`, `bool auto-switch`, then `i32 non-member limit` and `i32 member limit` (see Selection above). Incremental packets use a six-byte header (`u16 49`, `i32 subtype`).

**Reward bundle.** An empty bundle is 69 bytes: a success bool, coins (`i32`), XP/stars (`i32`), one unresolved `i32`, a reward kind (`i32`), more fields (a multiplier and guids), an entry count, and a trailing `i32`. Coins, stars and kind are the offsets the reward toast reads (`00b89ca0`, `008e7930`); the vault finds the same 69-byte empty size inside quest and objective records. See [[Rewards and Collections]].

**Reward text.** `RewardPatternOne` is "You receive" plus an item; `RewardPatternTwo` is "You receive" plus stars and coins (client locale via `CodeStringMappings.txt`, vault: Reward Toast note).

### Network

All quest traffic is tunneled. Client-to-server packets are written by body writers and wrapped by the gateway (`PacketTunneledClientPacket`, client gateway id 5; see [[Gateway and Transport]]). Headers differ by family: 49, 209 and 207 use `u16` family plus `u32` subtype; 45, 47 and 50 use `u16` family plus `u8` subtype. Directions are proven when a sender builds the packet (C>S) or a dispatcher case parses it (S>C). The packet catalogue the map validates against lacks most of these classes, so the map's `wire` field is set on only 26 of the 381 functions.

| Fam/sub | Name | Dir | Body | Handler or sender |
|---|---|---|---|---|
| 49/1 | QuestInfo | S>C | quest offer; see Open questions | `00c7bb60`, `00a94be0` |
| 49/2 | QuestReply | C>S | `i32` quest id, `u8` accept | `00a97d70`, `00a97e10`, `00a86400` |
| 49/3 | QuestAdd | S>C | quest record, `bool` select | `00c7cc80`, `00c7ccd0`, `00c7bd10` |
| 49/4 | QuestComplete | S>C | `i32` quest id | `00c7b420` |
| 49/5 | QuestFailed | S>C | `i32` quest id | `00c7b490` |
| 49/6 | QuestAbandoned | S>C | `i32` quest id | `00c7b500` |
| 49/7 | QuestObjectiveAdded | S>C | quest id, objective record | `00c7bd30`, `00c7c660` |
| 49/8 | ObjectiveActivated | S>C | quest id, key, required (`i32` x3), `bool` (19 bytes with header) | `00c7b570`, `00c7a910` |
| 49/9 | ObjectiveUpdate | S>C | quest id, key, current, `f32` progress, `bool`, `i32` (27 bytes) | `00c7b5e0`, `00c7a990` |
| 49/10 | ObjectiveComplete | S>C | quest id, key, `f32` progress, `i32`, `bool` (23 bytes) | `00c7b650`, `00c7aa10` |
| 49/11 | ObjectiveFailed | S>C | quest id, objective key | `00c7b6c0`, `00c7aa90` |
| 49/12 | CompletedQuestCountUpdate | S>C | `i32` count (10 bytes) | `00c7b730` |
| 49/13 | QuestEnd | S>C | quest id, four scalars, reward bundle, trailing field | `00c7bbd0`, `00c7c030`, `00a95420` |
| 49/14 | QuestEndReply | C>S | `i32` quest id | `00a97ef0`, `00a864c0` |
| 49/15 | QuestStartBreadcrumb | C>S | `i32` quest id | `00a97f90`, `00a86580`, `00c18aa0` |
| 49/16 | QuestEndBlocked | S>C | four `i32` (text id and quest id are logged) | `00c7b3b0`, `00c7a470` |
| 26/0x17 | QuestAbandon (command) | C>S | `i32` quest id | `00c09f60`, `00c0a500` |
| 47/1 to 4 | TaskAdd, TaskUpdate, TaskComplete, TaskFail | S>C | goal id and fields; layouts not decoded | `00a91bf0` cases 1 to 4 |
| 47/5 | Window hide | S>C | none | `00a89b60` |
| 47/10 | StartTimer | S>C | timer fields | `00a8eff0` |
| 47/12 | SelectQuest | C>S | `i32` quest id | `00c7c250`, `00c79e40` |
| 47/13 | SelectedQuestLocked | C>S | `u8` flag | `00c7c360`, `00c79f00` |
| 47/14 | ObjectiveTargetUpdate | S>C | active, guid, name, X, Z, zone, type, angle | `00a8fab0`, `00a88080` |
| 47/15 | Quest limit message | S>C | value 1 member, 2 non-member | `00a91bf0` case 15 |
| 45/1 to 4 | Objective Activate, Update, Complete, Fail | S>C | after the `u16`/`u8` header: Activate 2 x `i32`, Update 2 x `i32`, Complete 3 x `i32`, Fail 1 x `i32` | `009be130`, readers `009b56d0`, `009b5740`, `009b57b0`, `009b5820` |
| 45/5 | Objective Unhide | S>C | extended record | `009ba240`, `009bd970` |
| 45/6 | LookAt | S>C | id, guid or position, offset, range, angle | `009b5890`, `00c435c0`, `00c436b0` |
| 45/7 | ClientComplete | C>S | `i32` objective id (7 bytes) | `00cffba0`, `00cffa70` |
| 45/8, 45/9 | ClientClear, ClientCompleteFailed | S>C | `i32` id | `00c43090`, `00c430e0` |
| 45/10 | UIEvent | S>C | `i32` id, `i32` length, string | `00c437a0` |
| 45/11 | FirstMovement | S>C | `i32` id, `bool` is-mouse | `00c43870` |
| 45/12 | HousingFixtureMove | S>C | `i32` x3 (id, fixture, action) | `00c43940` |
| 209/1 | Journal definitions | S>C | four tables (key + 7, 10, 3, 9 `u32`) | `00a45d80`, `00a45d10` |
| 209/2 | Journal set | S>C | `u32` count, pairs of `u32` | `00a45bf0`, `00a45680` |
| 207/1, 5, 6, 7 | Progressive quest | S>C | see above | `00bde9c0` |
| 107/5 | FirstTimeEventScript | S>C | script name, int list, event id, clear flag | `00c897b0`, `00c893f0` |
| 202 | MembershipActivation | S>C | one value, stored at `player+0x464`; rest of the body not decoded here | `009471a0`, `0089bab0`, `00a12470` |

Other families this guide leans on, covered in their own guides: NPC dialog commands (family 26: sub-opcode 3 show dialog `00a99bd0`, 4 end dialog `009f5760`, 6 dialog response `009f5890`, 28 dialog effect `00a9ad20`; vault Pieces 071), reward toasts (family 50: sub 1 bundle, 2 item, 6 and 7 deferred bank), encounter enter 41/2 (tutorial flag) and zone details 43/31 (tutorial byte).

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour.

1. **Own all quest content.** The client has no definitions. Keep a table per quest: id, title, description and helper-text locale ids, giver NPC, requirements (previous quest, series, membership, combat job level, ride, cooldown, event window; wiki kinds), profile reference, icon, members-only and system flags, objectives (definition id, text ids, required count, category: primary, secondary, job, bonus or elite, trigger and target), rewards (stars, coins, items), and which hub, region or sticker it belongs to. Our own content pages supply the text; do not use the community tables.
2. **Persist per character.** Active quests with accept time, objective state, current and required counts, overall progress; completed quests (needed for prerequisites and the absolute count); selected quest and the auto-switch flag; the completed hub-quest key set; sticker completion; tutorial progress.
3. **Login.** Send the self-data quest block (records, count, selection, auto-switch, non-member limit, member limit) and the journal definitions 209/1 followed by 209/2. Send `PacketMembershipActivation` (id 202; it sets `player+0x464`, body not decoded here) so the client picks the right limit.
4. **Offer.** On NPC interaction check requirements and the limit. If the limit blocks the offer, send 47/15 (1 member, 2 non-member); otherwise send 49/1. Remember the offered quest id per player (R). A quest may also be added directly with 49/3 and no offer (tutorial and scripted starts).
5. **Accept.** On 49/2 with accept = 1 verify the id equals the offered quest and the limit, then send 49/3 with the select bool and objective states. Treat accept = 0 as a decline (no state change). Do not trust the client's id beyond the match with the pending offer.
6. **Progress.** The client reports nothing about kills, gathers, crafts or visits. The server derives progress from game events and sends 49/9 then 49/10 using the **objective definition id** as the key and a progress float; use 49/8 to activate later objectives, 49/7 to add one, 49/11 to fail one.
7. **Turn-in.** Send 49/13, wait for 49/14, grant rewards exactly once in one transaction, then send 49/4 and 49/12 (absolute count), then the next offer or add. Show rewards through the reward family ([[Rewards and Collections]]). If the turn-in cannot proceed send 49/16 with a locale text id and the quest id. Order is (R).
8. **Abandon and fail.** On command 26/0x17 remove the quest, send 49/6; send 49/5 for a failure. Neither grants rewards.
9. **Selection.** Accept 47/12 and 47/13, persist them (the vault found no proof the original persisted manual selection), and send the select bool on 49/3.
10. **Targets and Take Me There.** For the selected quest's active objective send 47/14 with position, zone id, target guid and type; send inactive when none. Handle 49/15: the client asks the server to start the breadcrumb for a quest id; the original reply (an offer, path data, or both) is unrecovered, so design it from the two consumers the client has, the 49/1 offer screen and the `BaseClient.Breadcrumb` rows (R).
11. **Goals panels.** For timed or activity goals send 47/1 to 47/4 and 47/10; for minigames send 45/1 to 45/5 only after the minigame state exists.
12. **Tutorial.** Register client objectives with 45/6, 45/10, 45/11, 45/12 exactly once per id; accept 45/7 only for an enabled, not completed id, and only once; use 45/8 to retire and 45/9 to re-arm. Never send duplicates (the client does not de-duplicate). Set and clear the tutorial flag only through the zone and encounter messages (43/31 true, 41/2 clears). Use 107/5 only with a fixed list of the known `FirstTimeEvent` names (R).
13. **Journal.** Send 209/1 and 209/2 with the exact widths (key + 7, 10, 3, 9 `u32`); the body must be consumed exactly. Send new snapshots when the hub or sticker state changes. Chapter and hub content is ours to define.
14. **Locale.** Every string id used by a quest, objective, hub, sticker or region must exist in the locale table we ship, or the client prints a "not found" marker.
15. **No repeats.** The client has no traced duplicate handling for 49/3, 49/7 or any 45/x registration (45/x is known not to de-duplicate), so the server must track what it has already sent and never repeat it. Removal messages for a missing quest are harmless: the lookup returns null and the notices are skipped.

### Evidence

- **Client.** 381 accepted functions across 31 shards (S0221 to S0740): 131 confirmed, 171 likely, 39 speculative, 40 unknown. Subsystems: `game.gaq` 66, `net.quest` 64, `ui.quest` 56, `game.quest` 53, `ui.gaq` 33, `net.objective` 31, `ui.objectives` 22, `game.objectives` 22, `ui.questjournal` 17, `game.tutorial` 17. Extra reads in this pass: the main dispatcher jump table (`009471a0`: ids 45, 47, 49, 207, 209), the `QuestScreen` and `Ui` method tables (`TakeMeToObjective`), quest-limit bytes, the quest reply senders, the 45-family and 47-family dispatchers, the journal readers. The map's serializable command-class registry holds no quest, objective, journal or tutorial class (its families are account, trade, guild, chat and card-game services).
- **Client Lua and locale.** `ScriptsBase.lua` supplies the data source column names, the `StoryBook`, `QuestHelper`, `Atlas` and `FirstTimeEvent` behaviour, the seven objective UI events and the 21 GAQ types. The locale supplies the banner, limit and tutorial strings.
- **Video.** Among 5,760 frame records in 656 videos, substring counts (not curated) are: "Show Active Quest" 96 frames (83 videos), "Active Quest" 571 (144), "Take Me There" 147 (27), "Goals" 768 (140), "New Goal" 134 (34), "Goal Complete" 174 (58), "Quest Started" 150 (27), "Quest Completed" 105 (22), "You receive" 221 (99), "Hero's Journal" 12 (5), "Chapter " titles 15 (5). 179 distinct quest titles appear. Examples used above are in the opening quest (find, use and close the Hero's Journal), a farming quest with counted goals, a timed coin goal, and several "Take Me There" trackers. Reward lines seen: 12 stars and 54 coins with an armour item; 11 stars and 13 coins; 12 stars and 22 coins; 30 stars and 16 coins; items such as Blast Speedster; "Mystery Gift" boxes.
- **Wiki.** The Quests page (rev 2021-11-20), the Interface page (rev 2017-10-01), and the Category:Quest list of 155 pages (51 "Quest:" pages). Verbatim requirement and step lines were re-read for Your First Weapon, Scaredy Pants, Horrible Hiccups, Glide Training: Basic Training, Winter Party and The Spring Eggstravaganza.
- **Vault.** [[Native Quest Journal UI]], [[Native Opening Quest Journal]] (our native accept, counter and turn-in runs only; its byte oracle is built from recorded server JSON and is not used), [[Native Journal Live Objective Keys]], [[Original Opening Microtutorial Recovery]], [[Original Tutorial Mode Exit]], [[Reward Toast and Deferred Bank Findings]], and three pieces: 067 Quest prerequisite engine (wiki requirement classes only), 070 Quest journal wire (only the client reader's view of the self-data tail, corrected here; the captured bytes and the 10/30 limit pair in that note come from recorded server payloads and are not used) and 071 NPC dialogue branching (client Lua response nodes and locale ids only). Not used: [[Original Quest Coverage]] and Pieces 066, 068 and 069, whose hub, sticker and registration rows come from community reconstructions, and every reconstructed-server result (the 17-stage local opening, ledgers, route tables).

### Open questions and contradictions

- **No original server code.** Offers, limits, rewards and ordering are inferred from the client. The vault's native runs prove the client accepts the reconstructed messages, not that the original sent them.
- **GAQ is not quests.** The brief calls GAQ "Group Activity Quests". Nothing in the client says that. The Lua entry point is `Gaq:AddGameAction`, the items are invites and notices, and the 66+33 mapped functions are queue and UI code. This guide treats GAQ as a game action queue; the expansion of the letters is our reading.
- **Quest-limit pair.** The vault (Pieces 070) states that nothing reads the last two self-data integers back. Bytes at `00a10890` and `00c0a2a0` read them as the quest limits (non-member and member, chosen by `player+0x464`). This is our correction. That `player+0x464` means "member" is inferred from the packet's class name and the `MembershipHandler:OnMembershipActivated` call beside the write; the id 202 body is not decoded.
- **`TutorialDB` is mis-grouped.** The map's `game.tutorial` subsystem holds `TutorialDB` (`01406d50` and neighbours), which stores keyed rule sets (`deck__1`, `rule__%d`) under `data/tutorial` for the card game. Only `ClientTutorialManager` and the three `+0x780` helpers (`008a89a0`, `008a8a30`, `008a9200`) belong to the new-player tutorial. 10 of the 17 rows in that subsystem are card-game code.
- **Unmapped bodies.** 49/1 and 49/13 (offer and end-screen records), 49/16 (only the text id and quest id are known to be logged), 47/1 to 47/4, 47/10, 47/14, 45/1 to 45/5 exact layouts, 209/1 per-field meaning (which `u32` is which column, ordering, hidden flag), the value half of 209/2, 207 sub-opcode bodies, the packet that fills the breadcrumb data source, and the message that sets the quest-giver exclamation mark.
- **Exclamation mark and "Active Quest!" label.** The client tutorial text and wiki say NPCs with a quest show an exclamation mark; the code that decides it was not traced. The green "Active Quest!" label with a metre distance is seen in video on the objective NPC; its HUD code path was not traced.
- **Where Take Me There's vector is set.** `Ui.TakeMeToObjective` restores a stored vector; the write that stores the objective's position (`009ce260`, `009ceee0`) was not tied to 47/14.
- **What the 6000 means.** The notify functions post events with arguments 6000 and 0; 6000 looks like a display time in milliseconds, unproven.
- **Quest records.** Several flags in the full quest record are unnamed (one `bool` after the profile reference, one `bool` and `i32` in each objective, two trailing `bool`s). The vault says none is needed for visibility. The 25-record sample the vault uses is canned character data, so counts and ids in it are not retail content.
- **Series, order and requirements.** The client has no prerequisite table. The wiki lists requirements on 36 quest pages, and six prerequisite pages are missing from the corpus (vault Pieces 067). Numeric quest ids for wiki pages are not known, so wiki series cannot be tied to client ids.
- **How an interaction becomes an offer.** The NPC interaction list and the dialog family (26/3, 4, 6, 28) are only partly decoded, so the exact trigger for 49/1 (a dialog response, an interaction entry, or both) is open.
- **Hubs, regions, stickers.** The retail set (names, order, which quest is which) is not in the client. The video chapter titles (for example a chapter-numbered quest in the Queensfields area) show chapter-style quest names, and the locale has 49 "Chapter " entries and completed-summary lines; whether those are hub or sticker texts is speculative.
- **Locale tags.** `ucdt` against `ugdt` double titles are unexplained.
- **Progressive quests.** The three request senders build named packet classes whose sub-opcodes were not read; what the order-response status values 3, 6, 7 and 8 mean is unknown.
- **Speculative labels in the map.** The 66 `game.gaq` and 33 `ui.gaq` rows are mostly mechanical thunks and inserts (`likely`); `00a0e740` and `00a0e7c0` are `speculative`. The four quest senders `00a97d70`, `00a97e10`, `00a97ef0`, `00a97f90` are `speculative` by label although their bytes are read above. The three progressive-quest data sources (15 rows) are `speculative`.
- **Duplicated functions.** `00c7c130`, `00c7c190` and `00c7c1f0` have identical bodies; the reason for three copies is unknown.

### Key functions

| Address | Role |
|---|---|
| `009471a0` | Main inbound dispatcher; ids 45, 47, 49, 207, 209 route to the handlers below |
| `00c7cd40` | `DispatchQuestPacket`, family 49 (subtypes 1, 3 to 13, 16) |
| `00c7bd10`, `00c7c130` | Quest add (insert) and remove |
| `00c7a910`, `00c7a990`, `00c7aa10`, `00c7aa90` | Objective activate, update, complete, fail |
| `00c7c660`, `00c7ab20` | Objective added; selection repair after removal |
| `00940c60`, `00940e60` | Self-data quest readers (records, counts, selection, limits) |
| `0093c840` | Quest record decoder |
| `009225d0`, `0091bb40`, `00baf490`, `00baf380`, `00baba60` | Runtime quest creation and objective re-keying |
| `00a94be0`, `00a95420` | Quest start and end screens |
| `00a92680`, `00a928b0`, `00a931b0`, `00a936e0`, `00a939b0` | Started, completed, objective updated, completed, failed notices |
| `00a97d70`, `00a97e10`, `00a97ef0`, `00a97f90` | Quest reply (accept, decline), end reply, breadcrumb senders |
| `00c0a020`, `0182a6f8` | `QuestScreen` registrar and its 11-entry table |
| `00c0a500`, `00c09f60` | Abandon (Lua handler and command constructor) |
| `00c7c250`, `00c7c360`, `00c7c540`, `00c7c5d0` | Select quest and lock-flag senders |
| `00a10890`, `00c0a2a0`, `00a12470` | Quest-limit selector, `GetQuestLimit`, membership flag setter |
| `00a91bf0` | Family 47 dispatcher (task goals, timer, target, quest-limit message) |
| `00a8fab0`, `00a88080` | Objective target update and its packet constructor |
| `00cb83d0`, `00cb89a0`, `00cb7cf0`, `00cb8810` | Objective helper and target data sources |
| `00c17c70`, `008a8e50`, `009cef10` | Take Me There native path |
| `00c18aa0`, `009cd6c0` | Start breadcrumb quest; set breadcrumb target |
| `009be130` | Family 45 dispatcher |
| `00c43090`, `00c430e0`, `00c43140` | Client objective clear, re-arm, event fan-out |
| `00c435c0`, `00c436b0`, `00c437a0`, `00c43870`, `00c43940` | Client objective registration paths |
| `00cf36a0`, `00cf3900`, `00cffba0` | Look-at check, UI-event match, completion send |
| `00c42870`, `00c42440`, `009b6700`, `009b6950` | Minigame goal creation, status text, complete, fail |
| `00a45d80`, `00a45d10`, `00a45bf0`, `00a45680` | Journal message handler and bodies |
| `00a459c0`, `00a45740`, `00a457e0`, `00a45880`, `00a45920` | Journal definition block and table readers |
| `00a44020`, `00a44100`, `00a43f00`, `00a447d0`, `00a44b70` | Journal selection logic |
| `00a432d0`, `00a44bb0` | Hub-quest lookup; show-journal on hub quest |
| `00c98ad0`, `00c98b80`, `00c98c30`, `00c98ce0` | Journal data sources (region, hub, sticker, misc) |
| `00c97d70`, `00c97de0` | Misc quest count and rebuild |
| `008a89a0`, `008a9200`, `00a46cd0`, `00a46c40` | Tutorial flag setters and manager |
| `00c893f0`, `00c897b0` | First-time-event script dispatch (107/5) |
| `00a0c170`, `00a0b5b0`, `00a0b830`, `00a0ba90`, `00a0bf80`, `00a0dae0` | GAQ creators and removal |
| `00a0e7f0`, `00a0e9e0` | GAQ response dispatch and Lua flush |
| `00bde9c0`, `00bde8a0` | Progressive quest dispatcher and client-data apply |
<!-- NARRATIVE END -->

## Mapped functions (generated)

381 accepted functions: 131 confirmed, 171 likely, 39 speculative, 40 unknown. Network-linked (`wire`): 26.

Subsystems: `game.gaq` (66), `net.quest` (64), `ui.quest` (56), `game.quest` (53), `ui.gaq` (33), `net.objective` (31), `ui.objectives` (22), `game.objectives` (22), `ui.questjournal` (17), `game.tutorial` (17)

### `game.gaq`

**GaqItemClaimItem** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a094d0` | GaqItemClaimItem_Respond1 | confirmed | Thirty-two-byte virtual method: fetches the global singleton with FUN_00769710, takes the manager pointer at singleton+4 and calls FUN_00a05130 on the object a… |

**GaqItemDuelRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a09710` | GaqItemDuelRequest_ForwardToSlot0 | confirmed | Six-byte tail thunk that jumps to slot 0 of its own vtable (bytes 8b 01 8b 10 ff e2), i.e. to the duel responder that calls the encounter reply builder 00a97c1… |
| `00a09730` | GaqItemDuelRequest_Respond0 🌐 | confirmed | Twenty-seven-byte virtual method calling FUN_00a97c10 with status 0 and the item's two parameters; that callee builds a BaseEncounterPacket reply. |
| `00a09750` | GaqItemDuelRequest_Respond1 🌐 | confirmed | Twenty-seven-byte virtual method calling FUN_00a97c10 with status 1 and the item's two parameters. |

**GaqItemFriendInvite** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a0a410` | GaqItemFriendInvite_Construct | confirmed | Constructor of the friend-invite game action item: installs the GaqItemFriendInvite vtable, stores the two id parameters at +8 and +0xc and initialises the tra… |

**GaqItemGroupAnnounceEncounter** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a09610` | GaqItemGroupAnnounceEncounter_ForwardToSlot0 | confirmed | Six-byte tail thunk that jumps to slot 0 of its own vtable (bytes 8b 01 8b 10 ff e2), i.e. to the encounter responder that runs the 00a97720/00a979a0 join-or-l… |
| `00a09650` | GaqItemGroupAnnounceEncounter_Respond0 | confirmed | Virtual method: fetches the global singleton, clears the dword at client+0x6F0, then compares the item's two parameters with the globals DAT_01be65c0/DAT_01be6… |
| `00a096a0` | GaqItemGroupAnnounceEncounter_Respond1 | confirmed | Virtual method identical to the slot-0 responder but passing status 1 to FUN_00a97720 and FUN_00a979a0. |

**GaqItemGroupInvite** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a090d0` | GaqItemGroupInvite_Respond0 | confirmed | Eleven-byte virtual method that loads the action object at this+4 and calls FUN_00a983b0 with status 0. |
| `00a090e0` | GaqItemGroupInvite_Respond1 | confirmed | Eleven-byte virtual method that loads the action object at this+4 and calls FUN_00a983b0 with status 1. |
| `00a090f0` | GaqItemGroupInvite_ForwardToSlot0 | confirmed | Six-byte tail thunk: loads the object vtable and jumps to slot 0 of the same object (bytes 8b 01 8b 10 ff e2), so the action is the item's first responder, whi… |

**GaqItemGuildInvite** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a09b00` | GaqItemGuildInvite_Construct | confirmed | Constructor of the guild-invite game action item: installs the GaqItemGuildInvite vtable, stores the two id parameters at +8 and +0xc, and initialises the embe… |

**GaqItemHouseInvite** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a09550` | GaqItemHouseInvite_Respond0 🌐 | confirmed | Thirty-seven-byte virtual method: reads the two parameter dwords at +8/+0xc, passes them to FUN_00769710, calls FUN_008a67a0 and then FUN_00ac4e00 with the sam… |
| `00a09580` | GaqItemHouseInvite_Respond1 🌐 | confirmed | Thirty-seven-byte virtual method identical to the slot-0 responder but calling FUN_00ac3fe0 instead of FUN_00ac4e00 with the two item parameters. |

**GaqItemInviteAndStartMiniGame** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a097b0` | GaqItemInviteAndStartMiniGame_ForwardToSlot0 | confirmed | Six-byte tail thunk that jumps to slot 0 of its own vtable (bytes 8b 01 8b 10 ff e2), i.e. to the responder that passes the item parameters to the mini-game in… |
| `00a097d0` | GaqItemInviteAndStartMiniGame_Respond0 | likely | Virtual method passing the five item parameter dwords at +8..+0x18 plus status 0 to FUN_00a97cc0. |
| `00a09800` | GaqItemInviteAndStartMiniGame_Respond1 | likely | Virtual method identical to the slot-0 responder but passing status 1 as the trailing argument to FUN_00a97cc0. |

**GaqItemTradeInvite** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a09140` | GaqItemTradeInvite_ForwardToSlot0 | confirmed | Six-byte tail thunk that jumps to slot 0 of its own vtable (bytes 8b 01 8b 10 ff e2), i.e. to the trade-invite responder that calls 00c6dbe0 with status 0. |
| `00a09120` | GaqItemTradeInvite_Respond0 | likely | Eleven-byte virtual method that forwards to FUN_00c6dbe0 with status 0. |
| `00a09130` | GaqItemTradeInvite_Respond1 | likely | Eleven-byte virtual method that forwards to FUN_00c6dbe0 with status 1. |

**SoeUtil::HashList<GaqItem*,64,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a0a3b0` | HashList_GaqItem_ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the GAQ map teardown 00a0a1f0 and frees this when the delete flag is set. |

**SoeUtil::HashListMap<int,GaqItem*,64,-1>** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a091e0` | HashListMap_GaqItem64_Construct | confirmed | Constructor of the GAQ item map: stores the HashListMap vtable, clears head/tail/count and the extra word and memsets the 64 bucket pointers at +0x14 (0x100 by… |
| `00a098c0` | HashListMap_GaqItem64_NewNode | confirmed | Node allocator of the GAQ item map: requests a 0x14-byte node from the thread/global memory manager, falling back to FUN_006fb950(0x14,4). |
| `00a09900` | HashListMap_GaqItem64_FreeNode | confirmed | Node deallocator of the GAQ item map: routes the node pointer to FreeMemory with alignment 4. |
| `00a0a160` | HashListMap_GaqItem64_Destruct | confirmed | Destructor body of the GAQ item map: stores the HashListMap vtable and repeatedly removes the head node with 00a09840 until the list is empty. |
| `00a0a190` | HashListMap_GaqItem64_ScalarDeletingDtor | confirmed | Scalar deleting destructor: drains the GAQ item map through 00a09840 and frees this when the delete flag is set. |
| `00a0a1f0` | HashListMap_GaqItem64_Destruct_B | confirmed | Second emission of the GAQ map destructor: stores the HashListMap vtable and drains the map through 00a09840 under an SEH frame. |
| `00a0a330` | HashListMap_GaqItem64_Construct_B | confirmed | Full constructor of the GAQ item map: stores the HashListMap vtable, clears head/tail/count and the 64 bucket pointers (0x100 bytes at +0x14), then installs So… |

**(no class)** (38)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a0c170` | GaqManager_AddGroupInvite | confirmed | Creates a group-invite game action: allocates an eight-byte GaqItemGroupInvite item, stores the group id from the arguments at item+4, when the client script g… |
| `00a09840` | HashListMap_GaqItem64_RemoveNode | likely | Unlinks a node from the 64-bucket GAQ item map: bucket selected by the dword at node+0xc masked with 0x3f (buckets from +0x14), fixes the doubly linked head/ta… |
| `00a09ad0` | GaqItemMap_Clear | likely | Loops while the head pointer at this+8 is non-null, calling 00a09840 (this = map, node = head) to remove and free each node. |
| `00a0a080` | GaqItem_DestructMembers | likely | Teardown of the item members: rewrites the embedded IString vtable at this+0x10 and, when the string still owns a heap buffer, decrements the buffer reference … |
| `00a0a260` | ScalarDelete_a0a260 | likely | Scalar deleting destructor shape: calls the item teardown 00a0a080 and frees this when the delete flag is set. |
| `00a0a5a0` | GaqItemMapOwner_Construct | likely | Owner constructor: stores the tag dword 1 at offset 0 and constructs the 64-bucket GAQ item map at +4 through 00a0a330. |
| `00a0a680` | GaqItemMapOwner_Create | likely | Allocating factory: stores 0 at the out pointer, allocates 0x118 bytes with FUN_00d833a6 and constructs the GAQ owner object with 00a0a5a0, storing the result. |
| `00a0a6f0` | ScalarDelete_a0a6f0 | likely | Scalar deleting destructor shape: calls the owner teardown 00a0a600 and frees this when the delete flag is set. |
| `00a0a960` | GaqItemMap_InsertGeneric | likely | Inserts one node into the 64-bucket GAQ item map: allocates through vtable slot 1, stores the value pointer from param_3 at node[0] and the id from param_2 at … |
| `00a0aa00` | GaqItemMap_InsertGuildRenameRequest | likely | Insert emission identical to 00a0a960 (allocate node, store value and id, link into list and bucket, increment count). |
| `00a0aaa0` | GaqItemMap_InsertGuildInvite | likely | Insert emission identical to 00a0a960. |
| `00a0ab40` | GaqItemMap_InsertHouseInvite | likely | Insert emission identical to 00a0a960. |
| `00a0abe0` | GaqItemMap_InsertFriendInvite | likely | Insert emission identical to 00a0a960. |
| `00a0ac80` | GaqItemMap_InsertClaimItem | likely | Insert emission identical to 00a0a960. |
| `00a0ad20` | GaqItemMap_InsertGenericNoResponse | likely | Insert emission identical to 00a0a960. |
| `00a0adc0` | GaqItemMap_InsertGroupInvite | likely | Insert emission identical to 00a0a960. |
| `00a0ae60` | GaqItemMap_InsertTradeInvite | likely | Insert emission identical to 00a0a960. |
| `00a0af00` | GaqItemMap_InsertEncounterInvite | likely | Insert emission identical to 00a0a960. |
| `00a0afa0` | GaqItemMap_InsertDuelRequest | likely | Insert emission identical to 00a0a960. |
| `00a0b040` | GaqItemMap_InsertInviteAndStartMiniGame | likely | Insert emission identical to 00a0a960. |
| `00a0b180` | GaqItemMapOwner_Destroy | likely | Twenty-two-byte destroy routine: loads the pointer at this[0] and, when non-null, calls the owner teardown 00a0a600 and then frees that object. |
| `00a0b5b0` | GaqManager_AddGuildInvite | likely | Creates a guild-invite game action: first scans the existing items and returns early when a guild invite with the same two inviter ids is already pending; othe… |
| `00a0bd50` | GaqManager_AddFriendInviteToGame_Entry | likely | Loads the inner receiver from this[0] and tail-jumps to 00a0b1f0, forwarding into the generic game-action creator. |
| `00a0bd60` | Wrapper_00a0bd60 | likely | Seven-byte thunk that loads the receiver from this[0] and tail-jumps to the guild-rename creator 00a0b380. |
| `00a0bd70` | Wrapper_00a0bd70 | likely | Seven-byte thunk that loads the receiver from this[0] and tail-jumps to the guild-invite creator 00a0b5b0. |
| `00a0bd80` | Wrapper_00a0bd80 | likely | Thirty-eight-byte wrapper that forwards four arguments to the friend-invite creator 00a0ba90 and returns. |
| `00a0bdb0` | Wrapper_00a0bdb0 | likely | Seven-byte thunk that loads the receiver from this[0] and tail-jumps to the house-invite creator 00a0b830. |
| `00a0bf80` | GaqManager_AddGenericNotification | likely | Creates a parameterised notification action: allocates a four-byte GaqGenericNoResponse item, takes the next id, inserts it with 00a0ad20, builds a four-argume… |
| `00a0c390` | Wrapper_00a0c390 | likely | Seven-byte thunk that loads the receiver from this[0] and tail-jumps to the group-invite creator 00a0c170. |
| `00b20420` | AdjustThis8TailJump_0099f000 | likely | mov ecx,[ecx+8], then jumps to 0099f000; forwards this through the adjusted subobject. |
| `00a09a80` | — | unknown | Walks the list whose head is at this+0xc (next link at node+4) and, for each item object at node[0], calls the three virtual predicates at vtable offsets 0x2c,… |
| `00a0a070` | — | unknown | Seven-byte wrapper that calls the item-list scan 00a09a80 with the same receiver and returns. |
| `00a0a600` | — | unknown | Owner teardown: takes the node at this+0xc, saves its item pointer, unlinks and frees the node with 00a09840, frees the saved item, and then destroys the GAQ i… |
| `00a0b1f0` | — | unknown | Generic game-action creator: takes the next id from the counter at this[0], inserts the new item into the GAQ map with 00a0a960, builds a script object with a … |
| `00a0b380` | — | unknown | Creates a guild-rename game action: allocates a four-byte item with the GaqItemGuildRenameRequest vtable, takes the next id from this[0], inserts it into the G… |
| `00a0b830` | — | unknown | Creates a house-invite game action: scans the pending items for one whose slot at +0x18 predicate matches and whose ids match, otherwise allocates a GaqItemHou… |
| `00a0ba90` | — | unknown | Creates a friend-invite game action: allocates a GaqItemFriendInvite with 00a0a410 from the handler arguments, when the client script guard 00917b60 reports re… |
| `00a0bdc0` | — | unknown | Creates a plain text-notification action: allocates a four-byte GaqGenericNoResponse item, takes the next id from this[0], inserts it with 00a0ad20, builds a t… |

### `game.objectives`

**ClientObjectiveFirstMovement** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf3a00` | ClientObjectiveFirstMovement::Ctor | confirmed | Clears base objective fields, stores objective id/type, installs the ClientObjectiveFirstMovement vtable, and stores the supplied input-mode byte. |

**ClientObjectiveHousingFixtureMove** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf3a70` | ClientObjectiveHousingFixtureMove::CheckFixtureAction | confirmed | Calls 00cffba0 only when each of the two stored identifiers either matches its corresponding argument or is zero as a wildcard. |
| `00cf3aa0` | ClientObjectiveHousingFixtureMove::Ctor | confirmed | Clears the objective base, stores the objective id/type, installs the ClientObjectiveHousingFixtureMove vtable, and writes two configured identifiers at +0x20 … |

**ClientObjectiveLookAt** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf35e0` | ClientObjectiveLookAt::Ctor | confirmed | Clears base state, stores the objective id/type and target identifiers, installs the ClientObjectiveLookAt vtable, copies the supplied position vector and stor… |
| `00cf36a0` | ClientObjectiveLookAt::CheckCompletion | confirmed | When the objective target mode is active, resolves the target or stored position, computes the camera-to-target vector, checks configured range and alignment t… |
| `00cf3890` | ClientObjectiveLookAt::Ctor | confirmed | Clears objective base fields, stores the objective id/type and target GUID, installs the ClientObjectiveLookAt vtable, stores the remaining constructor argumen… |

**ClientObjectiveUIEvent** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf3920` | ClientObjectiveUIEvent::Ctor | confirmed | Clears objective base fields, stores objective id/type, installs the ClientObjectiveUIEvent vtable, initializes its string member, and assigns the supplied eve… |
| `00cf3900` | — | unknown | Compares the supplied event value through 00977030 and calls 00cffba0 when the comparison reports a match. |

**(no class)** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009bd970` | AddObjectiveRecord | confirmed | When manager field +0x18 is set, appends the supplied record, marks data dirty, and for record kind 2 emits a New Objective notification using its text. |
| `00c43090` | ClientObjective_ClearDisableById | confirmed | Looks an objective id up in the manager's bucket chains (bucket id % 100 at this+0x14, chain next at node+4, key at node+8) and, on a hit, logs 'Clearing objec… |
| `00c430e0` | ClientObjective_FailById | confirmed | Same id lookup; when the located entry is still enabled (byte +0x1c non-zero) it clears the completed byte at +0x1d (re-arming the objective for a retry) and l… |
| `00c43140` | ClientObjectiveList_OnObjectiveEvent | confirmed | Walks the client-objective list from the head at this+8 following the next pointer at node+0x10 and, for every entry whose enabled byte +0x1c is non-zero and c… |
| `00c45c30` | ObjectiveCategory_GetPrefixKey | confirmed | Maps an objective category argument (param_2 + 1) to the localized category prefix through the string-table object returned by FUN_00769710 (its +0x10 virtual … |
| `00c43180` | ClientObjectiveList_NotifySlot2 | likely | Same list walk and the same enabled/unfinished filter as 00c43140, but calls virtual slot 2 (offset +8) with one argument; the four known ClientObjective class… |
| `00c431c0` | ClientObjectiveList_NotifySlot1 | likely | Same list walk and enabled/unfinished filter, calling virtual slot 1 (offset +4) with two arguments; slots 1/2 are empty stubs in ClientObjectiveUIEvent and ar… |
| `009bd8f0` | — | unknown | Iterates records with 0xe8 stride, finds each goal by key, appends a missing record, or copies count fields into an existing goal and marks its source dirty. |
| `00c43490` | — | unknown | Walks the manager's global list from the head at this+8 (next at +0x10). For every entry whose enabled byte at +0x1c is zero it unlinks the node from its hash … |
| `00c435c0` | — | unknown | Registration path for a LookAt objective aimed at an actor: logs 'Adding LookAt objective %d - target guid: %llu', allocates 0x50 bytes through FUN_00d833a6 an… |
| `00c436b0` | — | unknown | Registration path for a LookAt objective aimed at a world position: logs 'Adding LookAt objective %d - target location: (%.1f, %.1f, %.1f)' with the three floa… |
| `00c437a0` | — | unknown | Registration path for a UI-event objective: logs 'Adding UIEvent objective %d - name %s' with the event name string, allocates 0x30 bytes through FUN_00d833a6 … |
| `00c43870` | — | unknown | Registration path for a first-movement objective: logs 'Adding FirstMovement objective %d - isMouse %d', allocates 0x24 bytes and constructs the objective thro… |
| `00c43940` | — | unknown | Registration path for a housing-fixture-move objective: logs 'Adding HousingFixtureMove objective %d - fixture id %d, action %d', allocates 0x28 bytes and cons… |

### `game.quest`

**ClientQuestData** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0092ac70` | ClientQuestData::UnlinkAndDestroy | likely | Cleans the record, unlinks it from the list using offsets 0x188/0x18c, decrements count, releases string members, and calls the map destroy slot. |

**QuestStartHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a99220` | QuestStartHandler::DismissEndScreen | confirmed | Reads the object at arg+4 and tests a chain of state bytes in sub-objects at +0x11c, +0x130, +0x120, +0x124, +0x134 and +0x128; picks a reason code 1, 6, 7, 8,… |

**SoeUtil::HashListMap** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a98a0` | HashListMap_SignatureGuid_Construct | confirmed | Constructor of the signature-guid hash map: installs the SoeUtil::HashListMap<SoeUtil::StrongType<unsigned __int64,SoeUtilSignatureCharacterGuid,0>, ... ,256,-… |
| `008a9ba0` | HashListMap_ObjectiveData_Construct | likely | Constructor of the objective-data map: installs the SoeUtil::HashListMap<int,ObjectiveData,16,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 16 bucket poi… |

**SoeUtil::HashListMap<int,ClientQuestData,256,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c79fc0` | SoeUtil::HashListMap_int_ClientQuestData_256_neg1::Ctor | confirmed | Constructor: stores the SoeUtil::HashListMap<int,ClientQuestData,256,-1> vtable, zeroes this+8, +0xc, +0x10 and +4, and memsets 0x400 bytes (256 bucket heads) … |

**(no class)** (48)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091bb40` | QuestRuntimeNodeCtor | confirmed | Constructor helper: calls FUN_00baf490 on the third argument (constructs the embedded objective collection) and then copies the first dword of param_2 into thi… |
| `009225d0` | QuestRuntimeNodeCreateAndAppend | confirmed | Calls virtual slot [1] of the object in ECX as a gate; when it returns nonzero it constructs a node through 0091bb40 with the two stack arguments, otherwise us… |
| `00922810` | QuestObjectiveConstructWithKey_F0 | confirmed | Calls 0091c8b0 on this (its label: constructs the reward bundle embedded at this+0x18), then stores the dword at *param_2 into this+0xf0 and returns this. |
| `0092b050` | ClearTemporaryQuestObjectiveRecords | confirmed | Destroys temporary records, iterates a count-limited key list, passes each key to 00926800, and looks up the separate definition ID through 008fd770. |
| `0093c840` | DecodeQuestRecord | confirmed | Reads a bounded record of dwords, flags, paired values, nested data, a checked floating-point field, and trailing status values into a structure. |
| `00940c60` | ReadQuestCollectionAndTail | confirmed | Drains the existing list through 0092ac70, reads a bounded count, constructs a stack-local record with 0091c940 for each positive count, parses it through 0093… |
| `00bab950` | CopyObjectiveWithRuntimeKey | confirmed | Copy-constructs the embedded record through FUN_009ba390, writes *param_2 to the destination +0xf0 key field, and returns the destination pointer. |
| `00baba60` | InsertObjectiveByRuntimeKey | confirmed | Allocates/copies an objective node, links it into a doubly linked list, selects bucket runtimeKey&0xf, prepends it to that bucket chain, and increments the cou… |
| `00c7a910` | ApplyObjectiveActivated | confirmed | Finds a quest by id, searches its 16-bucket objective chain by key, sets objective state at +0xd0 to 1 and required count at +0xd8, and returns the objective o… |
| `00c7a990` | ApplyObjectiveUpdate | confirmed | Finds a quest and objective by their keys, writes the supplied counter to objective+0xd4 and progress to quest+0x28; if the objective is missing, still writes … |
| `00c7aa10` | ApplyObjectiveComplete | confirmed | Finds the quest and objective by id/key, sets matched objective state at +0xd0 to 2, stores progress at quest+0x28, and returns the objective or null. |
| `00c7bd10` | ApplyQuestAddPacket | confirmed | Takes the embedded quest-data pointer from the packet object and passes its address with the packet object to FUN_009225d0 for insertion into the runtime manag… |
| `00c7c130` | RemoveQuestRecord | confirmed | Searches the manager's bucket chain for the quest id; if found, destroys that record, runs removal bookkeeping, and returns 1, otherwise returns 0. |
| `008ad140` | HashListMap256_UnlinkNode_190 | likely | Unlinks one node from a 256-bucket HashListMap-style table (bucket heads at this+0x14 = the HashListMap bucket-array offset): bucket index is the low byte of [… |
| `0091bbb0` | QuestRuntimeNodeCtor_2 | likely | Constructor helper shaped exactly like 0091bb40 but calling FUN_00baf580: constructs the embedded collection from the third argument and copies the first dword… |
| `0091bc30` | Ctor_Call009142f0_Store0x214 | likely | Constructor helper that runs FUN_009142f0 first (constructs the base subobject) and then copies the dword pointed to by param_2 into this+0x214. |
| `0091c8b0` | Ctor_EmbedRewardBundleAt0x18 | likely | Constructor: sets this+0 to -1, zeroes this+4/+8/+0xc, clears the byte at +0x10, constructs a RewardBundleBase at this+0x18 via FUN_0091c110, and zeroes the tr… |
| `0091c940` | Ctor_EmbedRewardBundle30_HashListObjectiveData | likely | Constructor: sets this+0 to -1, zeroes +4..+0x10 and +0x24, copies the 8-byte global pair at 01be48b8/01be48bc into this+0x18/+0x1c, zeroes +0x20, constructs a… |
| `0091c9f0` | Ctor_EmbedRewardBundle_HashListObjectiveData_PairIds | likely | Constructor: sets this+0 to -1, zeroes +4/+8 and the byte at +0xc, copies the 8-byte global pair into this+0x10/+0x14 and again into +0x18/+0x1c, zeroes +0x20,… |
| `00940e60` | ReadCountedQuestRecordsAndTail | likely | Drains the existing list through 0092ad20, reads a bounded count, constructs each stack-local record with 0091c9f0, parses it through 0093ca50, and inserts it … |
| `00a43240` | GetQuestStringAt15C | likely | Resolves a quest record for the id param_1 through the two-step lookup FUN_008a6740 then FUN_00c7a610 and returns the string pointer stored at record+0x15c, or… |
| `00a43270` | GetQuestStringAt16C | likely | Same two-step quest lookup as 00a43240 but returns the string pointer at record+0x16c, with the same &DAT_0175b400 empty-string fallback. |
| `00babc00` | CopyRecordAndObjectives | likely | Copies leading fields, initializes embedded objects through helpers, copies selected trailing fields, adjusts a count if needed, then iterates source records c… |
| `00baf380` | ObjectiveData_CopyCtor | likely | Copy constructor of an ObjectiveData record: copies the dwords at +0..+0xc, +0x18..+0x20 and +0x28 from param_2, copies the bytes at +0x10, +0x11 and +0x24, co… |
| `00baf490` | RuntimeNode_ObjectiveCollectionCtor | likely | Constructor: calls 00baf380 on param_2 to copy the objective record, sets three IString<char> members at this+0x158, +0x168 and +0x178 to the empty string, and… |
| `00c24400` | RequestProgressiveQuestStart | likely | Reads two numeric arguments, constructs ProgressiveQuestRequestStartQuestPacket, and wraps the request when the command service is active. |
| `00c244d0` | RequestProgressiveQuestClaimSlot | likely | Reads two numeric arguments, constructs ProgressiveQuestRequestClaimSlotPacket, and wraps it when the command service is active. |
| `00c245a0` | RequestProgressiveQuestClaimPrize | likely | Reads two numeric arguments, constructs ProgressiveQuestRequestClaimPrizePacket, and wraps it when the command service is active. |
| `00c7a610` | FindQuestRecord | likely | Indexes the bucket array at manager+0x18 with the low byte of the quest id, follows node+0x194 links, compares node+0x190, and returns the matching node or nul… |
| `00c7a670` | AllocateQuestRecord | likely | Initializes the quest-record pool, requests a pooled block, invokes its virtual constructor when present, otherwise invokes the global pool constructor or allo… |
| `00c7a6b0` | FreeQuestRecord | likely | Passes the supplied allocation pointer and alignment 8 to FUN_00401c00, then returns. |
| `00c7a890` | SelectHashRecordPointer | likely | Optionally notifies the datasource, always notifies it again, searches the manager's quest bucket chain for the requested id, and passes record+0x30 or null to… |
| `00c7aa90` | ApplyObjectiveFailed | likely | Finds the quest and objective by id/key, writes state 3 to a match's +0xd0 field, and returns the objective or null without changing quest progress. |
| `00c7ab00` | ReapplyStoredSelection | likely | Reads the stored selection at manager+0x10f4 and, when it is positive, calls FUN_00c7a890 with that id and flag 1. |
| `00c7ab20` | RepairSelectionAfterRecordRemoval | likely | Notifies the journal datasource; if the removed id was selected, scans remaining records for the greatest progress value at +0x28, clears selection, then clear… |
| `00c7c190` | RemoveQuestRecord | likely | Searches the manager's bucket chain for the quest id; when present destroys the record, performs quest-removal bookkeeping and returns 1, otherwise returns 0. |
| `00c7c1f0` | RemoveQuestRecord | likely | Searches the manager's bucket chain for the quest id; when present destroys the record, performs quest-removal bookkeeping and returns 1, otherwise returns 0. |
| `00c7c660` | ApplyAddedObjective | likely | Searches the quest manager bucket chain for the requested id and, when found, passes the decoded objective data to FUN_00baba60. |
| `00c7c8a0` | ReleaseFourReferencesAndStore | likely | Releases reference-counted members at offsets +0x1108/+0x110c/+0x1110/+0x1114 when enabled, clears each pointer, then destroys the embedded quest-data store. |
| `00c7ca80` | ConstructQuestDataStore | likely | Stores the supplied vtable pointer, initializes the quest hash-list, sets selection to -1 and several counters to zero, then creates and assigns reward/objecti… |
| `00a97d70` | SendQuestReplyPacket | speculative | Thiscall. When the word at this+0x1c is positive, builds a QuestReplyPacket through callee 00a86400 with that value and flag 1, and when *(*(this+8)+0x280) is … |
| `00a97e10` | SendQuestReplyPacketAndReset | speculative | Thiscall. Calls 008c5d90 first, then when the word at this+0x1c is positive builds a QuestReplyPacket through 00a86400 and sends it via 00a96360 when *(*(this+… |
| `00a97ef0` | SendQuestEndReplyPacket | speculative | Thiscall. When the word at this+0x1c is positive, builds a QuestEndReplyPacket through callee 00a864c0 with that value, hands it to 00a96470 with flag 1 when *… |
| `00a97f90` | SendQuestStartBreadcrumbPacket | speculative | Thiscall with an argument. When the argument is nonzero, builds a QuestStartBreadcrumbPacket through callee 00a86580 from that argument and, when *(*(this+8)+0… |
| `00cf7b70` | FindRuntimeQuestById | speculative | Indexes one of 16 linked-list buckets with the low four bits of the supplied ID, compares node field 0xf0, and returns the matching node or null. |
| `00922650` | — | unknown | Second copy of 009225d0: calls virtual slot [1] of the object in ECX as a gate, constructs with 0091bbb0 when it returns nonzero (otherwise 0), then calls 008e… |
| `00c7c540` | — | unknown | When selection differs from manager+0x43d, stores the new id, constructs the UI request object, conditionally sends it when vtable+0x280 is nonzero, then appli… |
| `00c7c5d0` | — | unknown | When the byte at this+0x43e changes, stores the new flag, constructs a request carrying the inverted boolean and conditionally transmits it through FUN_00c7c36… |

### `game.tutorial`

**ClientTutorialManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a46c60` | ClientTutorialManager::Preload | confirmed | Thiscall: when the byte at +8 is set and the byte at +9 is clear and the player's byte at +0x959 is clear, it sets +9 and calls 00ae79b0 with GamePreLoadAssets… |
| `00a46cd0` | ClientTutorialManager::ClientTutorialManager | confirmed | Thiscall with one player argument: installs ClientTutorialManager::vftable at +0, stores the argument at +4, clears the bytes at +8 and +9, runs the DelayedAss… |
| `00a46c40` | ClientTutorialManager::Cleanup | likely | Thiscall: when the byte at +8 is zero it sets it to 1, gets the client singleton through 00769710 and passes its field at +0x18 to 00ae3750 (StoreFieldAndDispa… |
| `00a46bf0` | — | unknown | Installs ClientTutorialManager::vftable at +0, then calls 00ae8460 (DelayedAssetLoader::Destroy) on the object; it does not free the object. |

**TutorialDB** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014064e0` | TutorialDB::SerializeDb | confirmed | Thiscall (ECX = TutorialDB this). Zeroes a serial buffer (013c2210), writes this+0x18 as a 32-bit integer (013c23f0) and then the low word of __time64 as an in… |
| `01406cc0` | TutorialDB::DtorBody | confirmed | Stores TutorialDB::vftable into this, calls 01406020(this, 0) (member release), frees the heap buffer at this+0x20 when the string capacity at this+0x34 is gre… |
| `01406d50` | TutorialDB::TutorialDB | confirmed | Stores TutorialDB::vftable into this, calls 0129d350 with ECX = this+8 (lookup-map sub-object constructor), zeroes this+4, the bytes at +0x14, +0x15 and +0x20,… |
| `01406dd0` | TutorialDB::~TutorialDB | confirmed | Scalar deleting destructor: calls 01406cc0 (the destructor body) on this, then calls _free(this) when bit 0 of the second argument is set. Returns this. |
| `01406df0` | TutorialDB::AddRuleSet | confirmed | Asserts the rule-set argument is non-null and that its ID (vtable +0x44, getRuleSetID) is non-zero. Looks that ID up in the lookup map at this+8 through 0129ae… |
| `01406ec0` | TutorialDB::LoadRuleSet | confirmed | Thiscall with one stack argument (rule-set id). Looks the id up in the lookup map at this+8 through 0129aea0 (asserting the container is this+8). If the id is … |
| `01406020` | TutorialDB::ReleaseMembers | likely | Thiscall with one stack argument. If this+4 is non-null it calls 01514ed0(this+4, arg) and zeroes this+4. Calls 01405f60(this) to clear the lookup map at this+… |
| `01405d60` | — | unknown | Cdecl with two stack arguments: the rule-set object and a dbBuffer item pointer. Asserts both are non-null ('ruleSet', 'dbBuffer'). Zeroes a five-word temporar… |
| `01406350` | — | unknown | Thiscall (ECX = TutorialDB) with two stack arguments: param_2 is an output item pointer and param_3 a key suffix string. Builds the key from the std::string at… |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a89a0` | SetTutorialModeAndEnsureManager | confirmed | When param_2 is true, writes 1 to this+0x780. If this+0x77c is null, calls FUN_00d833a6 with the sole explicit argument 0x2ba70; on allocation success it calls… |
| `008a8a30` | DisposeTutorialManagerIfPresent | confirmed | Checks this+0x77c and, when the tutorial-manager pointer is non-zero, tail-jumps to FUN_00a46c40 (the manager cleanup that unregisters it); when it is null the… |
| `008a9200` | SetTutorialFlag780 | confirmed | Byte-verified one-line setter (8a 44 24 04 / 88 81 80 07 00 00 / c2 04 00): stores the byte stack argument at this+0x780. No calls. Caller 00aa36c0 is the enco… |
| `01406950` | TutorialLoadItems | likely | Thiscall with one stack argument. If the flag byte at this+0x14 is not 1: when 013aef60() returns zero the location is the literal 'data/tutorial'; otherwise i… |

### `net.objective`

**BaseObjectivePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aa4c0` | BaseObjectivePacket::BaseObjectivePacket 🌐 | confirmed | Constructor: installs BasePacket::vftable, writes this[1] = 0x2d (family/opcode 45), stores the sub-opcode param_2 at this[2] and installs BaseObjectivePacket:… |
| `009aca40` | DeletingDtor_009aca40 | likely | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object… |

**ObjectiveActivatePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aa750` | ObjectiveActivatePacket::ObjectiveActivatePacket 🌐 | confirmed | Constructor: calls the family constructor FUN_009aa4c0(1) (opcode 45, sub-opcode 1), installs ObjectiveActivatePacket::vftable and stores param_2/param_3 into … |
| `009acb60` | DeletingDtor_009acb60 | likely | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object… |

**ObjectiveClientClearPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aaa20` | ObjectiveClientClearPacket::ObjectiveClientClearPacket 🌐 | confirmed | Constructor: calls the family constructor FUN_009aa4c0(8) (opcode 45, sub-opcode 8), installs ObjectiveClientClearPacket::vftable and stores param_2 at this[3]. |
| `009acd20` | DeletingDtor_009acd20 | likely | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object… |

**ObjectiveClientCompleteFailedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aaaf0` | ObjectiveClientCompleteFailedPacket::ObjectiveClientComplet… 🌐 | confirmed | Constructor: calls the family constructor FUN_009aa4c0(9) (opcode 45, sub-opcode 9), installs ObjectiveClientCompleteFailedPacket::vftable and stores param_2 a… |
| `009acd80` | DeletingDtor_009acd80 | likely | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object… |

**ObjectiveCompletePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aa590` | ObjectiveCompletePacket::ObjectiveCompletePacket 🌐 | confirmed | Constructor: calls the family constructor FUN_009aa4c0(3) (opcode 45, sub-opcode 3), installs ObjectiveCompletePacket::vftable and stores param_2/param_3/param… |
| `009acaa0` | DeletingDtor_009acaa0 | likely | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object… |

**ObjectiveFailPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aa680` | ObjectiveFailPacket::ObjectiveFailPacket 🌐 | confirmed | Constructor: calls the family constructor FUN_009aa4c0(4) (opcode 45, sub-opcode 4), installs ObjectiveFailPacket::vftable and stores param_2 at this[3]. |
| `009acb00` | DeletingDtor_009acb00 | likely | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object… |

**ObjectiveFirstMovementPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aabd0` | ObjectiveFirstMovementPacket::ObjectiveFirstMovementPacket 🌐 | confirmed | Constructor: calls the family constructor FUN_009aa4c0(0xb) (opcode 45, sub-opcode 11), installs ObjectiveFirstMovementPacket::vftable and sets this[3] = 0xfff… |
| `009acdf0` | DeletingDtor_009acdf0 | likely | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object… |

**ObjectiveHousingFixtureMovePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aaca0` | ObjectiveHousingFixtureMovePacket::ObjectiveHousingFixtureM… 🌐 | confirmed | Constructor: calls the family constructor FUN_009aa4c0(0xc) (opcode 45, sub-opcode 12), installs ObjectiveHousingFixtureMovePacket::vftable and sets this[3] = … |
| `009ace50` | DeletingDtor_009ace50 | likely | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object… |

**ObjectiveLookAtPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009acc20` | ObjectiveLookAtPacket::ObjectiveLookAtPacket 🌐 | likely | Constructor: calls the family constructor FUN_009aa4c0(6) (opcode 45, sub-opcode 6), installs ObjectiveLookAtPacket::vftable, sets this[3] = -1, copies the two… |
| `009acca0` | DeletingDtor_009acca0 | likely | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object… |

**ObjectiveUpdatePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aa830` | ObjectiveUpdatePacket::ObjectiveUpdatePacket 🌐 | confirmed | Constructor: calls the family constructor FUN_009aa4c0(2) (opcode 45, sub-opcode 2), installs ObjectiveUpdatePacket::vftable and stores param_2/param_3 into th… |
| `009acbc0` | DeletingDtor_009acbc0 | likely | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object… |

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aa530` | PacketDtor_009aa530 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from 36 MSVC unwind funclets (the largest clean… |
| `009aa600` | PacketDtor_009aa600 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from the objective builder 009be130 and an MSVC… |
| `009aa6f0` | PacketDtor_009aa6f0 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from the objective builder 009be130 and an MSVC… |
| `009aa7c0` | PacketDtor_009aa7c0 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from the objective builder 009be130 and an MSVC… |
| `009aa8a0` | PacketDtor_009aa8a0 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from the objective builder 009be130 and an MSVC… |
| `009aa910` | PacketDtorWithRewardMember_009aa910 | likely | Destructor variant: calls FUN_0103d010 (destructor of a member/base owning RewardBundleBase::vftable and SoeUtil list members) under the stack-cookie check and… |
| `009aa980` | PacketDtor_009aa980 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from the objective builder 009be130 and an MSVC… |
| `009aaa90` | PacketDtor_009aaa90 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from the objective builder 009be130 and an MSVC… |
| `009aab60` | PacketDtor_009aab60 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from the objective builder 009be130 and an MSVC… |
| `009aac30` | PacketDtor_009aac30 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from the objective builder 009be130 and an MSVC… |
| `009aad00` | PacketDtor_009aad00 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from the objective builder 009be130 and an MSVC… |

### `net.quest`

**BaseProgressiveQuestPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bdc8a0` | BaseProgressiveQuestPacket::BaseProgressiveQuestPacket | confirmed | Family constructor of the progressive-quest packet: stores BasePacket::vftable, writes the family opcode 0xcf (207) into the packet opcode field at +4, install… |
| `00bdc920` | BaseProgressiveQuestPacket::~BaseProgressiveQuestPacket | confirmed | Destructor body shared by the progressive-quest packet classes: stores BasePacket::vftable at object+0 and performs no member cleanup; referenced by 22 EH unwi… |
| `00bdce90` | BaseProgressiveQuestPacket::ScalarDeletingDtor | confirmed | Virtual scalar deleting destructor (slot 0 of BaseProgressiveQuestPacket::vftable): stores BasePacket::vftable at object+0 and frees the object through the 00d… |

**ClientProgressiveQuestData** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bddcb0` | ClientProgressiveQuestData_Init | likely | Initialises the two embedded arrays of the progressive-quest data record: installs SoeUtil::Array<ClientProgressiveQuestSlot,0,1>::vftable and zeroes the field… |
| `00bddd20` | ClientProgressiveQuestData::~ClientProgressiveQuestData | likely | Destructor body of the progressive-quest data record: clears the prize array size at +0x48, installs its vtable and frees its buffer at +0x44 with alignment 4,… |
| `00bdddb0` | ClientProgressiveQuestData::ScalarDeletingDtor | likely | Scalar deleting destructor of the progressive-quest data record: calls the destructor body 00bddd20 and then frees the object through the 00d82463 _free thunk … |
| `00bde1f0` | ClientProgressiveQuestData_Read | likely | Reads the client-data record: eight u32 values into +0..+0x1c, then calls 00bddfa0 for the ClientProgressiveQuestSlot array member at +0x30 and 00bde020 for th… |

**ClientProgressiveQuestPrizeSlot** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bddba0` | ClientProgressiveQuestPrizeSlot::Read | likely | Reads one prize record from the packet reader: six u32 values into +0, +4, +8, +0xc, +0x10 and +0x14 and one byte into +0x18 (0x1c bytes in total), setting the… |

**ClientProgressiveQuestSlot** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bdd9c0` | ClientProgressiveQuestSlot::Read | likely | Reads one slot record from the packet reader (cursor at +8, end at +0xc, error byte at +0x10): three u32 values into +0, +4, +8, four single-byte booleans into… |

**CompletedQuestCountUpdatePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c79c50` | CompletedQuestCountUpdatePacket::Construct 🌐 | confirmed | Constructs a CompletedQuestCountUpdatePacket: calls FUN_00a86340(0xc) (BaseQuestPacket base constructor with subtype 12), stores the vtable and the completed-c… |
| `00c7a3f0` | CompletedQuestCountUpdatePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |

**ProgressiveQuestClientDataPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bdcef0` | ProgressiveQuestClientDataPacket::ScalarDeletingDtor | confirmed | Virtual scalar deleting destructor (slot 0 of ProgressiveQuestClientDataPacket::vftable): stores BasePacket::vftable at object+0 and frees the object through t… |
| `00bdc970` | ProgressiveQuestClientDataPacket::Ctor 🌐 | likely | Constructs the packet with FUN_00bdc8a0(1) (family 0xcf, sub-opcode 1), installs ProgressiveQuestClientDataPacket::vftable and stores the argument (the ClientP… |
| `00bdc9e0` | ProgressiveQuestClientDataPacket::~ProgressiveQuestClientDa… | likely | Destructor body: stores BasePacket::vftable at object+0 and performs no member cleanup; referenced by the unwind funclet 015efacb of the handler that builds th… |

**ProgressiveQuestNotifyRewardItemPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bdcfb0` | ProgressiveQuestNotifyRewardItemPacket::ScalarDeletingDtor | confirmed | Virtual scalar deleting destructor (slot 0 of ProgressiveQuestNotifyRewardItemPacket::vftable): stores BasePacket::vftable at object+0 and frees the object thr… |
| `00bdcae0` | ProgressiveQuestNotifyRewardItemPacket::Ctor 🌐 | likely | Constructs the packet with FUN_00bdc8a0(7) (family 0xcf, sub-opcode 7) and installs ProgressiveQuestNotifyRewardItemPacket::vftable. |
| `00bdcb40` | ProgressiveQuestNotifyRewardItemPacket::~ProgressiveQuestNo… | likely | Destructor body: stores BasePacket::vftable at object+0 and performs no member cleanup; referenced by the unwind funclet 015efa60 of 00bde610. |

**ProgressiveQuestShowWindowPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bdcf50` | ProgressiveQuestShowWindowPacket::ScalarDeletingDtor | confirmed | Virtual scalar deleting destructor (slot 0 of ProgressiveQuestShowWindowPacket::vftable): stores BasePacket::vftable at object+0 and frees the object through t… |
| `00bdca90` | ProgressiveQuestShowWindowPacket::~ProgressiveQuestShowWind… | likely | Destructor body: stores BasePacket::vftable at object+0 and performs no member cleanup; referenced by the unwind funclet 015ef9c0 of 00bde3d0. |
| `00bdca30` | ProgressiveQuestShowWindowPacket::Ctor | speculative | Constructs the packet with FUN_00bdc8a0(0) (family 0xcf, sub-opcode 0) and installs ProgressiveQuestShowWindowPacket::vftable. |

**QuestAbandonedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c79730` | QuestAbandonedPacket::Construct 🌐 | confirmed | Constructs a QuestAbandonedPacket: calls FUN_00a86340(6) (BaseQuestPacket base constructor with subtype 6), stores the QuestAbandonedPacket vtable, the quest i… |
| `00c7a260` | QuestAbandonedPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |

**QuestCompletePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7a1a0` | QuestCompletePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c79590` | QuestCompletePacket::Construct 🌐 | likely | Constructs a QuestCompletePacket: calls FUN_00a86340(4) (BaseQuestPacket base constructor with subtype 4), stores the QuestCompletePacket vtable and the quest … |

**QuestEndBlockedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7a470` | QuestEndBlockedPacket::Construct 🌐 | confirmed | Constructs a QuestEndBlockedPacket: calls FUN_00a86340(0x10) (BaseQuestPacket base constructor with subtype 16), stores the vtable and the four 32-bit argument… |
| `00c7a4f0` | QuestEndBlockedPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |

**QuestEndPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c79d20` | — | unknown | Destructor body: calls FUN_0103d010 on the object (reward-bundle member teardown) under an SEH frame, then resets the vtable to BasePacket::vftable. |

**QuestFailedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7a200` | QuestFailedPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c79660` | QuestFailedPacket::Construct 🌐 | likely | Constructs a QuestFailedPacket: calls FUN_00a86340(5) (BaseQuestPacket base constructor with subtype 5), stores the QuestFailedPacket vtable and the quest id a… |

**QuestInfoPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c79470` | — | unknown | Destructor body: calls FUN_0103d010 on the object (teardown of the embedded reward-bundle members) under an SEH frame, then resets the vtable to BasePacket::vf… |

**QuestObjectiveActivatedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c7a2c0` | QuestObjectiveActivatedPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c798a0` | QuestObjectiveActivatedPacket::Construct 🌐 | likely | Constructs a QuestObjectiveActivatedPacket: calls FUN_00a86340(8) (BaseQuestPacket base constructor with subtype 8), stores the vtable and four fields (quest i… |

**QuestObjectiveAddedPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c79810` | QuestObjectiveAddedPacket::Destruct | likely | Destructor body: calls FUN_0103d010 on the object (reward-bundle member teardown) under an SEH frame, then resets the vtable to BasePacket::vftable. |

**QuestObjectiveFailedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c79b70` | QuestObjectiveFailedPacket::Construct 🌐 | confirmed | Constructs a QuestObjectiveFailedPacket: calls FUN_00a86340(0xb) (BaseQuestPacket base constructor with subtype 11), stores the vtable and the quest id and obj… |
| `00c7a390` | QuestObjectiveFailedPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |

**QuestObjectiveUpdatePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c799c0` | QuestObjectiveUpdatePacket::Construct 🌐 | confirmed | Constructs a QuestObjectiveUpdatePacket: calls FUN_00a86340(9) (BaseQuestPacket base constructor with subtype 9), stores the vtable and the quest id, objective… |
| `00c7a320` | QuestObjectiveUpdatePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |

**SelectQuestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c79e40` | SelectQuestPacket::Construct 🌐 | confirmed | Constructs a SelectQuestPacket: calls FUN_00a86640(0xc) (BaseUiPacket base constructor with subtype 12), stores the SelectQuestPacket vtable and the quest id a… |
| `00c7a550` | SelectQuestPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |

**SelectedQuestLockedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c79f00` | SelectedQuestLockedPacket::Construct 🌐 | confirmed | Constructs a SelectedQuestLockedPacket: calls FUN_00a86640(0xd) (BaseUiPacket base constructor with subtype 13), stores the vtable and the byte argument at +0x… |
| `00c7a5b0` | SelectedQuestLockedPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |

**(no class)** (23)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bde610` | LuaCall_UnifiedMessageWindow_ShowItemPanel | confirmed | Constructs the local ProgressiveQuestNotifyRewardItemPacket scratch with 00bdcae0, parses the message body with 00bdde70 (u16 sub-opcode, u32 field plus four f… |
| `00c7cd40` | DispatchQuestPacket | confirmed | Reads the quest-family prefix and dispatches subtype cases 1-13 and 16; parses each body, inserts/removes quest records, mutates objective state and progress, … |
| `00bdde70` | NotifyRewardItemBody_Read | likely | Reads the opcode-207 reward body: calls 00bdddd0 to fill the u16 at +4 and the u32 at +8, then reads four further u32 values into +0xc, +0x10, +0x14 and +0x18,… |
| `00bde020` | PacketRead_PrizeArray | likely | Reads an element count from the packet reader and aborts with the error byte set when it is negative or larger than the remaining bytes; otherwise makes the Cl… |
| `00bde820` | ParseProgressiveQuestClientDataBuffer | likely | When both the buffer and size arguments are non-zero it builds a stack reader over them, calls 00bdddd0 and 00bde1f0 to parse a full client-data record into lo… |
| `00bde8a0` | ProgressiveQuestClientData_ApplyFromPacket | likely | Inbound opcode-207 sub-1 handler: frees the old 0x50-byte ClientProgressiveQuestData record at this+0x10 (destructor 00bddd20 then operator delete), allocates … |
| `00c79600` | TrivialDtorBasePacket_00c79600 | likely | Trivial destructor body: resets the vtable to BasePacket::vftable and returns; no member teardown. |
| `00c796d0` | TrivialDtorBasePacket_00c796d0 | likely | Trivial destructor body: resets the vtable to BasePacket::vftable and returns; no member teardown. |
| `00c797a0` | TrivialDtorBasePacket_00c797a0 | likely | Trivial destructor body: resets the vtable to BasePacket::vftable and returns; no member teardown. |
| `00c79920` | TrivialDtorBasePacket_00c79920 | likely | Trivial destructor body: resets the vtable to BasePacket::vftable and returns; no member teardown. |
| `00c79a50` | TrivialDtorBasePacket_00c79a50 | likely | Trivial destructor body: resets the vtable to BasePacket::vftable and returns; no member teardown. |
| `00c79ae0` | TrivialDtorBasePacket_00c79ae0 | likely | Trivial destructor body: resets the vtable to BasePacket::vftable and returns; no member teardown. |
| `00c79be0` | TrivialDtorBasePacket_00c79be0 | likely | Trivial destructor body: resets the vtable to BasePacket::vftable and returns; no member teardown. |
| `00c79cc0` | TrivialDtorBasePacket_00c79cc0 | likely | Trivial destructor body: resets the vtable to BasePacket::vftable and returns; no member teardown. |
| `00c79dd0` | TrivialDtorBasePacket_00c79dd0 | likely | Trivial destructor body: resets the vtable to BasePacket::vftable and returns; no member teardown. |
| `00c79eb0` | TrivialDtorBasePacket_00c79eb0 | likely | Trivial destructor body: resets the vtable to BasePacket::vftable and returns; no member teardown. |
| `00c79f70` | TrivialDtorBasePacket_00c79f70 | likely | Trivial destructor body: resets the vtable to BasePacket::vftable and returns; no member teardown. |
| `00c7c250` | SendSelectQuestRequest | likely | Initializes a serializer, writes the selection request, constructs a tunneled client packet, fills payload lengths, sends via FUN_00925640 and frees temporary … |
| `00c7c360` | SendSelectedQuestLockedRequest | likely | Initializes a serializer, writes a one-byte selection-lock value, constructs a tunneled client packet, sends it through FUN_00925640 and releases the temporary… |
| `00bde510` | LuaCall_OrderResponse | speculative | Starts a Lua argument frame with 0070ec40; when the frame guard passes it switches on the first argument value and for values 3, 6, 7 and 8 invokes the callbac… |
| `00bddfa0` | — | unknown | Reads an element count from the packet reader and aborts with the error byte set when it is negative or larger than the remaining bytes; otherwise makes the Cl… |
| `00bde3d0` | — | unknown | Constructs the local ProgressiveQuestShowWindowPacket scratch with 00bdca30, parses the message body with 00bddf50 (u16 sub-opcode, u32 field, one byte) and, w… |
| `00bde9c0` | — | unknown | Entry point for opcode 207 (0xcf) messages dispatched by FUN_009471a0: constructs a local BaseProgressiveQuestPacket scratch with 00bdc8a0(0), builds a reader … |

### `ui.gaq`

**ClientRewardManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a0c3a0` | ClientRewardManager_AddItemReceived | likely | Looks up an item definition by id param_3 in the hash table reached through FUN_00769710()+4 (bucket id&0x3ff, chain via +0xb0, key at +0xb4); on a miss logs t… |

**GaqItemGuildRenameRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a09070` | GaqItemGuildRenameRequest_OnNameDenied | confirmed | Twenty-one-byte virtual method that dispatches the script event GuildDialog:ShowNameDenied through the Lua bridge 009797a0 with two zero arguments. |

**(no class)** (31)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a0a770` | CallLuaHandlerWithPlayerAndGuild | confirmed | Resolves a script handler by name through the client's script object (SINGLETON+4+4, slot at +0x14), builds the named parameter list player=<arg2> and guild=<a… |
| `00a0a890` | CallLuaHandlerWithPlayer | confirmed | Same helper as 00a0a770 but with a single named parameter: it looks the handler name up through the client's script object, builds player=<arg> with 0097b540 a… |
| `00a0ca80` | Gaq_AddFriendBeatHighScore | likely | Allocates a GaqGenericNoResponse handler, takes the next GAQ id from *param_1, links (id, handler) with FUN_00a0ad20, then fills a ClientLuaParam array (FUN_00… |
| `00a0cd90` | Gaq_AddGameTopScore | likely | Creates a GaqGenericNoResponse handler, increments the GAQ id counter *param_1, links the entry with FUN_00a0ad20, fills the ClientLuaParam array created by FU… |
| `00a0d090` | Gaq_AddPersonalHighScore | likely | Same shape as the other GAQ builders: allocates a GaqGenericNoResponse handler, bumps the id counter *param_1, links it with FUN_00a0ad20, builds a ClientLuaPa… |
| `00a0d540` | Gaq_AddTradeInviteThunk | likely | Seven-byte MSVC thunk: 'mov ecx,[ecx]' then 'jmp 00a0d320'. The Ghidra body is the single call FUN_00a0d320(); the byte-level form shows it reloads the manager… |
| `00a0d550` | Gaq_AddGroupAnnounceEncounter | likely | Allocates a 0x18-byte GaqItemGroupAnnounceEncounter, stores param_2/param_4/param_5 into it, takes the next GAQ id from *param_1 and links it with FUN_00a0af00… |
| `00a0d810` | Gaq_AddGroupAnnounceEncounterThunk | likely | 38-byte thunk: packs the two incoming 8-byte pairs into temporaries, reloads the manager pointer from ecx ('mov ecx,[ecx]') and calls FUN_00a0d550; the recover… |
| `00a0daa0` | Gaq_AddDuelRequestThunk | likely | 51-byte thunk: packs the two 8-byte argument pairs, reloads ecx ('mov ecx,[ecx]') and calls FUN_00a0d840; the recovered body forwards all five arguments. |
| `00a0dc00` | Gaq_RemoveItemByIdThunk | likely | 17-byte thunk: 'mov eax,[esp+4]; mov ecx,[ecx]; push 1; push eax; call 00a0dae0' - it removes the GAQ entry with the caller's id and the notify flag set to 1. … |
| `00a0dc20` | Gaq_AddFriendInviteToGame | likely | Checks FUN_00984960() and then allocates a 0x20-byte GaqItemInviteAndStartMiniGame filled from param_2..param_7; takes the next GAQ id from *param_1, links the… |
| `00a0df40` | Gaq_RemoveExpiredItems | likely | Walks the GAQ list at this+0xc; for every node whose item virtual predicate at vtable slot +0x10 returns 1 it calls FUN_00a0dae0(node[3],1) to remove that entr… |
| `00a0df80` | Gaq_RemoveItemsExcept | likely | Walks the GAQ list at this+0xc; for each item whose virtual predicate at slot +0x1c is true and whose fields [2] and [3] do not equal (param_2, param_3) it inv… |
| `00a0dfe0` | Gaq_RemoveExpiredItemsThunk | likely | Seven-byte thunk ('mov ecx,[ecx]; jmp 00a0df40'): recovered body is the single forwarding call FUN_00a0df40(). |
| `00a0e020` | Gaq_Thunk_00a0bf80 | likely | Seven-byte thunk ('mov ecx,[ecx]; jmp 00a0bf80'): forwards to FUN_00a0bf80 after dereferencing the manager holder, recovered as a single call. |
| `00a0e030` | Gaq_RemoveByTwoFields | likely | Walks the GAQ list at this+0xc and removes the first entry whose item satisfies virtual slot +0x28 and whose fields [2] and [3] equal (param_2, param_3), via F… |
| `00a0e090` | Gaq_AddClaimNotification | likely | Prepares a 0x18-byte GaqItemClaimItem (vtable + param_2, param_3, param_7, param_8), first purging an existing entry with FUN_00a0e030, then takes the next GAQ… |
| `00a0e670` | Gaq_AddFriendInviteToGameThunk | likely | 56-byte thunk: packs the second 8-byte argument pair, reloads ecx and calls FUN_00a0dc20; recovered body is the single forwarding call with six arguments. |
| `00a0e6c0` | Gaq_AddItemReceivedThunk | likely | 58-byte thunk: packs two 8-byte argument pairs, reloads ecx and calls FUN_00a0c3a0; recovered body forwards eight arguments. |
| `00a0e770` | Gaq_RemoveByItemFields | likely | Walks the GAQ list at this+0xc and removes the first item that satisfies vtable slot +0x1c and whose fields at +8 and +0xc equal (param_2, param_3), via FUN_00… |
| `00a0e7f0` | Gaq_DispatchItemById | likely | Looks up the queue node with id param_2 in the bucket table this+0x18+(id&0x3f)*4 (chain at node[4]) and applies action param_3: 0 calls the item's virtual slo… |
| `00a0e9e0` | Gaq_FlushItemsToLua | likely | Walks the GAQ list at this+0xc; for each item whose status returned by vtable slot +0xc is not 3 it calls FUN_00a0e7f0(id, status) and then calls Lua 'Gaq:Remo… |
| `00a0eb40` | Gaq_FlushItemsToLuaThunk | likely | Seven-byte thunk ('mov ecx,[ecx]; jmp 00a0e9e0'): recovered body is the single forwarding call FUN_00a0e9e0(). |
| `00a0eb50` | Gaq_AddClaimNotificationThunk | likely | 58-byte thunk: packs the second 8-byte argument pair, reloads ecx and calls FUN_00a0e090; recovered body forwards eight arguments. |
| `00a0eb90` | Gaq_RemoveFirstSlot14Thunk | likely | Walks the list at *param_1+0xc (the manager is reached through a pointer-to-pointer) and removes the first item whose vtable slot +0x14 predicate is true, via … |
| `00a0ebc0` | Gaq_RemoveByItemFieldsThunk | likely | 28-byte thunk: packs the incoming 8-byte pair into temporaries, reloads ecx and calls FUN_00a0e770; recovered body forwards the two arguments. |
| `00a0e740` | Gaq_RemoveFirstSlot14 | speculative | Walks the GAQ list at this+0xc and removes the first entry whose item returns true from vtable slot +0x14, by calling FUN_00a0dae0(node[3],1). |
| `00a0e7c0` | Gaq_RemoveFirstSlot20 | speculative | Walks the GAQ list at this+0xc and removes the first item whose vtable slot +0x20 predicate returns true, via FUN_00a0dae0(node[3],1). |
| `00a0d320` | — | unknown | Allocates an 8-byte GaqItemTradeInvite (vtable + the invite id param_2), aborts it via FUN_00917b60(4) when the check fails, takes the next GAQ id from *param_… |
| `00a0d840` | — | unknown | Checks FUN_00984960() for an active state, then allocates a 0x10-byte GaqItemDuelRequest holding param_2/param_3/param_4, aborts it via FUN_00917b60 when the c… |
| `00a0dae0` | — | unknown | Looks up the GAQ entry for id param_2 in the bucket table at this+0x18+(id&0x3f)*4 following the node chain at node[4]; if the notify flag param_3 is set it ca… |

### `ui.objectives`

**ObjectiveHelperDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cb7710` | ObjectiveHelperDataSource::ColumnHeaderText | confirmed | Virtual slot 1 of ObjectiveHelperDataSource: switches on the index argument and returns a static header string for 0 ID, 1 Text, 2 Icon ID, 3 Is Bonus, 4 Statu… |
| `00cb8320` | ObjectiveHelperDataSource::DtorBody | confirmed | Fastcall on receiver p: stores ObjectiveHelperDataSource::vftable at p+0, p+0x30 and p+0x34, then the Array<ObjectiveEntry> vftable at p+0x258; calls 00cb7aa0 … |
| `00cb8460` | ObjectiveHelperDataSource::DeletingDtor | confirmed | Virtual slot 0 of ObjectiveHelperDataSource (thiscall, one byte flag): calls the dtor body 00cb8320 on the receiver, then when flag bit 0 is set calls _free (t… |
| `00cb7cf0` | ObjectiveHelperDataSource::GetCellValue | likely | Virtual slot 4 of ObjectiveHelperDataSource (thiscall: row index, column, output). Row record = receiver+0x22c base plus index*0x2c. Switch on column 0..8: 0 p… |
| `00cb83d0` | ObjectiveHelperDataSource::Ctor | likely | Thiscall with one dword argument: calls 00922890 with the BaseClient.ObjectiveHelper string, stores ObjectiveHelperDataSource::vftable at p+0, p+0x30 and p+0x3… |
| `00cb7800` | ObjectiveHelperDataSource::GetField230 | speculative | Virtual slot 2 of ObjectiveHelperDataSource: bytes 8b 81 30 02 00 00 c3 decode to mov eax,[ecx+0x230]; ret, so it returns the dword at receiver+0x230. |

**ObjectiveTargetDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cb8480` | ObjectiveTargetDataSource::ColumnHeaderText | confirmed | Virtual slot 1 of ObjectiveTargetDataSource: switches on the index argument and returns a static header string for 0 Active, 1 Guid, 2 Name, 3 Location X, 4 Lo… |
| `00cb89a0` | ObjectiveTargetDataSource::Ctor | confirmed | Thiscall with one dword argument: calls 00922890 with the BaseClient.ObjectiveTarget string, stores ObjectiveTargetDataSource::vftable at p+0, p+0x30 and p+0x3… |
| `00cb8b40` | ObjectiveTargetDataSource::DeletingDtor | confirmed | Virtual slot 0 of ObjectiveTargetDataSource (thiscall, one byte flag): calls the dtor body 00cb8ab0 on the receiver, then when flag bit 0 is set calls _free (t… |
| `00cb8810` | ObjectiveTargetDataSource::GetCellValue | likely | Virtual slot 4 of ObjectiveTargetDataSource (thiscall; param_2 float, param_3 column, param_4 output). Returns 0 unless param_2 == 0.0. Column 0 passes one of … |

**(no class)** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a872f0` | GetObjectiveCategoryPrefix | confirmed | Maps an objective category number to its localisation key and resolves it into the caller's string: switch on param_2+1 where categories -1 and 1 both use "Obj… |
| `00a89b60` | ObjectiveWindowHide | confirmed | Calls 00cb7e20 with no arguments, calls 009797a0 with ObjectiveWindow:Hide, clears flag 0x80 at this+0xc, and sets the dword at this+0x18 to 0. |
| `00a8f3b0` | AchievementHandlerUpdateObjective | confirmed | Builds a two-element ClientLuaParam array from the supplied values, calls AchievementHandler:UpdateObjective, then destroys the temporary array. |
| `00a8fab0` | ObjectiveWindowSelectTask | confirmed | Compares incoming task state with cached fields, clears selection on an inactive state, otherwise updates ids and flags, selects the minimap indicator, and inv… |
| `00a8b9a0` | ObjectiveStatusDataBuild | likely | Builds the objective-status data for one objective (1324 bytes): gets the category prefix through 00a872f0 (GetObjectiveCategoryPrefix), then chooses among the… |
| `00cb7e20` | ClearRecordsThenSlot4 | likely | Fastcall on receiver p: calls vtable slot 3 of the sub-object at p+0x34 (ECX = p+0x34, no arguments); calls 00cb7aa0 on the array header at p+0x258 with the co… |
| `00cb7e50` | RowSetField24FormatUpdate | speculative | Thiscall with two stack arguments (key, value): finds the row whose first dword equals key via 00cb7810; if found, stores value at row+0x24, formats the value … |
| `00cb7ff0` | RowSetField20To3UpdateCol4 | speculative | Stack-argument function (key): finds the row whose first dword equals key via 00cb7810; if found, stores the constant 3 at row+0x20, formats that value with 00… |
| `00cb8ab0` | StringMemberReleaseThenSub34Dtor | speculative | Fastcall on receiver p: stores SoeUtil::IString<char>::vftable at p+0x26c; when the count at p+0x278 is positive, atomically decrements the reference count sto… |
| `00cb7f20` | — | unknown | Fastcall-shaped stack-argument function (key): finds the row whose first dword equals key via 00cb7810; if found, stores the constant 2 at row+0x20, formats th… |
| `00cb81e0` | — | unknown | Thiscall with seven stack arguments (ret 0x1c): id (param_2), a value (param_3), a byte flag (param_4), a char flag (param_5) and a dword (param_6) are used; p… |
| `00cb8500` | — | unknown | Fastcall on receiver p: returns 0.0 unless 008a7d50 fills local_20 (two floats plus fStack_18) and the object reached through p+0x254 then +0xec answers a virt… |

### `ui.quest`

**ProgressiveQuestDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce7ea0` | ProgressiveQuestDataSource::ColumnName | speculative | Maps column indices 0..11 to progressive quest id/name/timer/slot/icon/objective/window/button fields; returns <error> otherwise. |
| `00ce7f50` | ProgressiveQuestDataSource::GetHasQuest | speculative | Returns whether the quest data pointer at +0x228 has a nonzero field at offset +0x10. |
| `00ce8460` | ProgressiveQuestDataSource::GetColumn | speculative | Reads the current quest record and returns quest id, localized name, timer, active slot, icon, objective strings, window flag, and button string ids for suppor… |
| `00ce89d0` | ProgressiveQuestDataSource::Ctor | speculative | Constructs ProgressiveQuest.Definition holder, installs source vtables, and stores two supplied arguments. |
| `00ce8ab0` | ProgressiveQuestDataSource::ScalarDeletingDtor | speculative | Calls the embedded holder destructor, conditionally frees the object when flag bit 0 is set, and returns its pointer. |

**ProgressiveQuestPrizeSlotsDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce8020` | ProgressiveQuestPrizeSlotsDataSource::ColumnName | speculative | Maps column indices 0..6 to prize slot, icon, progress, claim eligibility, tooltip, name, and quest id fields. |
| `00ce80d0` | ProgressiveQuestPrizeSlotsDataSource::GetField48 | speculative | Reads nested quest data pointer and returns its field at offset +0x48, or zero when pointer is null. |
| `00ce8c50` | ProgressiveQuestPrizeSlotsDataSource::Ctor | speculative | Constructs ProgressiveQuest.PrizeSlots holder, installs prize slots source vtables, and stores two supplied arguments. |
| `00ce8d30` | ProgressiveQuestPrizeSlotsDataSource::ScalarDeletingDtor | speculative | Calls the embedded holder destructor, conditionally frees the object when flag bit 0 is set, and returns its pointer. |
| `00ce8860` | — | unknown | Bounds-checks prize rows and returns prize slot ids, icon, progress, claim predicate, tooltip/name strings, and quest id. |

**ProgressiveQuestSlotsDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce7f70` | ProgressiveQuestSlotsDataSource::ColumnName | speculative | Maps column indices 0..11 to slot/item/key/quest eligibility, key bundle, claim, tooltip, name, and quest id fields. |
| `00ce80b0` | ProgressiveQuestSlotsDataSource::GetField38 | speculative | Reads nested quest data pointer and returns its field at offset +0x38, or zero when pointer is null. |
| `00ce8b10` | ProgressiveQuestSlotsDataSource::Ctor | speculative | Constructs ProgressiveQuest.Slots holder, installs slots source vtables, and stores two supplied arguments. |
| `00ce8bf0` | ProgressiveQuestSlotsDataSource::ScalarDeletingDtor | speculative | Calls the embedded holder destructor, conditionally frees the object when flag bit 0 is set, and returns its pointer. |
| `00ce85f0` | — | unknown | Bounds-checks slot rows and returns slot id, icon, slot/key/quest predicates, key bundle values, progress, tooltip and localized labels. |

**QuestDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf7a10` | QuestDataSource::GetColumnName | confirmed | Returns the column name for each index from Quest Id through System Quest, or an error string for unsupported indices. |
| `00cf81f0` | QuestDataSource::QuestDataSource | confirmed | Initializes the datasource base with BaseClient.QuestData, installs QuestDataSource vtables, stores two arguments, and clears the population state. |
| `00cf7bb0` | QuestDataSource::GetRowCount | speculative | Returns the value at offset 0x14 of the object referenced by this+0x228. |
| `00cf7ca0` | QuestDataSource::SelectQuest | speculative | Looks up the supplied quest identifier with FUN_00c7a610, stores the returned record at offset 0x25c, and rebuilds quest rows. |
| `00cf7cd0` | QuestDataSource::GetCell | speculative | Ensures rows are populated, resolves the row's runtime quest record, then writes the requested quest identifier, title, description, time, progress, flags, or … |
| `00cf8110` | QuestDataSource::Cleanup | speculative | Restores the QuestDataSource vtables at the object and embedded offsets, then destroys its embedded GUI userdata table. |
| `00cf8280` | QuestDataSource::ScalarDeletingDtor | speculative | Calls the datasource cleanup routine and frees the object when the delete flag's low bit is set. |
| `00cf7c70` | — | unknown | Populates objective rows when the source is marked dirty, then passes the supplied argument to FUN_010c90c0. |

**QuestObjectiveDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf7ae0` | QuestObjectiveDataSource::GetColumnName | confirmed | Returns Name, Description, Status, NotUsed, or Id for indices 0 through 5, and an error string for other indices. |
| `00cf7bc0` | QuestObjectiveDataSource::GetRowCount | speculative | Returns zero when the pointer at this+0x22c is null; otherwise returns the dword at the referenced object plus 0xf8. |
| `00cf8180` | QuestObjectiveDataSource::Cleanup | speculative | Restores the QuestObjectiveDataSource vtables at the object and embedded offsets, then destroys its embedded GUI userdata table. |
| `00cf82a0` | QuestObjectiveDataSource::QuestObjectiveDataSource | speculative | Initializes its base, installs the QuestObjectiveDataSource vtables, stores two constructor arguments, and clears a state byte. |
| `00cf8330` | QuestObjectiveDataSource::ScalarDeletingDtor | speculative | Calls the datasource cleanup routine and frees the object when the delete flag's low bit is set. |
| `00cf7fd0` | — | unknown | Resolves the row's runtime quest, finds its objective by ID, then returns objective name, description, status, or identifier fields. |

**(no class)** (27)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a10870` | GetAutoSwitchSelectedQuest | confirmed | Returns the byte field at this+0x1bec8, the AutoSwitch-selected-quest flag; nothing else is touched. |
| `00a10880` | AutoSwitchSelectedQuest_Set | confirmed | Eleven-byte adjustor thunk ('add ecx,0x1add0; jmp 00c7c5d0'), recovered as the single call FUN_00c7c5d0(); the sub-object at +0x1add0 owns the AutoSwitch flag … |
| `00a8f130` | QuestHandlerUpdateObjective | confirmed | Builds a two-element ClientLuaParam array from the supplied values, calls QuestHandler:UpdateObjective, then destroys the temporary array. |
| `00a8f2b0` | QuestHandlerPopulateHelper | confirmed | Builds a one-element ClientLuaParam array from the supplied value, calls QuestHandler:PopulateHelper, then destroys the temporary array. |
| `00a92680` | NotifyQuestStarted | confirmed | When param_4 is zero, looks up QuestStarted, formats two strings, builds two Lua values and posts StartQuest with arguments 6000,0; param_5 can call 00a8f2b0. |
| `00a928b0` | NotifyQuestCompleted | confirmed | When param_3 is zero, looks up QuestCompleted, formats two strings, builds two Lua values and posts CompleteQuest with arguments 6000,0. |
| `00a92ac0` | NotifyQuestFailed | confirmed | When param_3 is zero, looks up QuestFailed, formats two strings, builds two Lua values and posts FailQuest with arguments 6000,0. |
| `00a92cd0` | NotifyQuestAbandoned | confirmed | Looks up QuestAbandoned, formats a message, stores two Lua values and posts the FailQuest notification with arguments 6000,0. |
| `00a92ee0` | NotifyNewQuestObjective | confirmed | Calls 00a8f130 with objective data, then when param_5 is zero formats QuestNewObjective text and posts UpdateObjective with arguments 6000,0. |
| `00a931b0` | NotifyQuestObjectiveUpdated | confirmed | Calls 00a8f130 with objective data, formats QuestObjectiveUpdateChatMessage from localized fields, posts it, then passes progress data to 00a92570. |
| `00a936e0` | NotifyQuestObjectiveCompleted | confirmed | When param_5 is zero, looks up QuestObjectiveComplete, formats three strings, builds three Lua values and posts CompleteObjective with arguments 6000,0. |
| `00a939b0` | NotifyQuestObjectiveFailed | confirmed | When param_5 is zero, looks up QuestObjectiveFailed, formats three strings, builds three Lua values and posts FailObjective with arguments 6000,0. |
| `00a94be0` | ShowQuestStartScreen | confirmed | Updates quest-related Lua fields, optionally formats QuestNpcIntroTextMessage with questName/player/message values, then emits QuestHandler:ShowStartScreen. |
| `00a95420` | ShowQuestEndScreen | confirmed | Updates quest-related Lua fields, optionally formats QuestNpcEndTextMessage with player/message values, emits QuestHandler:ShowEndScreen, and may call 0096c780. |
| `00c0a020` | RegisterQuestScreenModule | confirmed | Registration wrapper: pushes nup=0, the 11-entry table 0x0182a6f8 and the 'QuestScreen' module-name pointer, calls FUN_00e39e40 and returns 0. |
| `00a10800` | AutoSwitchQuestSetter_Enable | likely | Twenty-one-byte thunk: pushes 1 and the incoming argument, adjusts ecx by +0x1add0 and calls FUN_00c7a890; recovered as FUN_00c7a890(param_1,1). |
| `00a10820` | MethodThunk_00c7c540 | likely | Eleven-byte adjustor thunk ('add ecx,0x1add0; jmp 00c7c540'), recovered as the single call FUN_00c7c540(). |
| `00a10830` | Quest_SelectMiscQuestFromBookThunk | likely | Calls FUN_00c7a890 with the incoming argument and constant 0, then returns. |
| `00c0a080` | Quest_SelectQuest | likely | Lua 'SelectQuest' handler: with one numeric argument, when the local player exists, it calls FUN_00a10800(quest id) on the quest screen object; returns 0. |
| `00c0a140` | Quest_SelectMiscQuestFromBook | likely | Lua 'SelectMiscQuestFromBook' handler: with one numeric argument, when the local player exists, it calls FUN_00a10830(quest id) on the quest book object; retur… |
| `00c0a500` | Quest_AbandonQuest | likely | Lua 'AbandonQuest' handler: with one numeric argument it builds CommandPacketQuestAbandon (FUN_00c09f60) with the quest id and, when the gateway link is up, se… |
| `00ce80f0` | ProgressiveQuestSlotArrayAt | speculative | Returns element address at base plus index times 0x24; for index beyond count, requests growth and stores new count. |
| `00cf7ad0` | QuestDatasourceResetThunk | speculative | Loads a virtual target from this object's vtable, clears byte this+0x25c, then jumps through the target register. |
| `00cf7c30` | BuildQuestRows | speculative | Clears the row builder and, when the active quest collection pointer is nonzero, iterates its linked nodes and adds each row through virtual slot 1. |
| `00c0a0e0` | — | unknown | Lua 'SelectQuestInHelper' handler: with one numeric argument, when the local player exists, it calls FUN_00a10820(quest id) on the quest helper object; returns… |
| `00ce81a0` | — | unknown | Returns element address at base plus index times 0x1c; for index beyond count, requests growth and stores new count. |
| `00cf7be0` | — | unknown | Clears the row builder, iterates nodes from the selected quest's list, adds each node through virtual slot 1, then sets the populated flag. |

### `ui.questjournal`

**(no class)** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a432d0` | HasHubQuestDefinition | confirmed | Chained hash lookup over the map reached through this+0x20: bucket index is param_2 & 0x1f inside the bucket array at map+0x13c, chain next at node+0x18, key a… |
| `00a44bb0` | ShowAdvJournalIfHubQuest | confirmed | Refresh-and-show path: calls FUN_00c97980 and virtual slot 0x10 on the object at +0x34, then looks param_2 up in the map at this+0x20 (bucket index param_2 & 0… |
| `00a43310` | InvokeSlot10OnField34 | likely | Calls FUN_00c97980 and then invokes virtual slot 0x10 on the object held at this+0x34, i.e. a two-step refresh notification. |
| `00a43c90` | JournalData_Ctor | likely | Constructor of the journal data object: stores the owner at +0, zeroes +4..+0x34, sets +0xc = 1, then allocates and constructs four data sources - FUN_00c98ad0… |
| `00a43f00` | AnyListEntryKeyInMap24 | likely | For the list reached through *(this+0x20)+0x130 (entries with an id at +4 and a key at +0, chained through +0xc): for every entry whose +4 equals param_2 it lo… |
| `00a43f60` | MaxNonnegativeListNodeField0 | likely | Walks nodes at *(this+0x20)+8 through each node's +0x20 link. It starts the accumulator at 0, compares each node's first dword as a signed value, and returns m… |
| `00a44020` | RecomputeCurrentRegionId | likely | Recomputes the current selection at this+0x1c from the definition list at *(this+0x20)+0x130: for entries whose +4 equals the selected category at this+0x18 it… |
| `00a44100` | HasSelectableEntryForCategory | likely | Calls 00a43f80(param_2, 1) to obtain a fallback category id, then scans the definition list at *(this+0x20)+0x130 for entries whose +4 equals that id, looking … |
| `00a44790` | SelectChapterAndRefresh | likely | Sets the selected chapter id at this+0x18 to param_2 when it differs, calls 00a44020(0) to recompute the dependent selection, and unless param_3 suppresses the… |
| `00a447d0` | RebuildJournalSelection | likely | Rebuilds the journal's selection: with no explicit chapter (this+8 zero) it takes the chapter recorded at +0x10, or, when that is zero, the first category for … |
| `00a44af0` | SetField8AndRebuild | likely | Stores param_2 at this+8 and immediately calls the selection rebuild 00a447d0. |
| `00a44b10` | TrackSelectedIdAndRebuild | likely | Selection bookkeeping: when param_2 differs from the currently tracked id at +0x10 it resets the counter at +0x14 to zero, then increments +0x14, stores param_… |
| `00a44b30` | UntrackSelectedIdAndRebuild | likely | Releases one reference to the tracked selection: when param_2 equals the tracked id at +0x10 and the counter at +0x14 is positive it decrements the counter, ze… |
| `00a45370` | JournalDefinitionMaps_TeardownB | likely | Teardown body of the object that owns the four journal definition maps: runs the map destructors 00a45300 (sticker), 00a45290 (hub-quest), 00a45220 (hub) and 0… |
| `00a459c0` | JournalDefinitionMaps_Ctor | likely | Constructs the four definition maps of the block at ECX: calls the region HashList constructor 00a453f0 on +0, the hub constructor 00a45470 on +0x94, the hub-q… |
| `00a45a40` | JournalDefinitionMaps_ScalarDeletingDtor | likely | Calls the four-map teardown 00a45370 on ECX, then, when bit 0 of the stack argument is set, frees the block with _free through thunk 00d82463; returns the bloc… |
| `00a44b70` | SelectEntryByField14AndRebuild | speculative | When *(this+0x20) is non-null, walks the list at holder+8 through node[8], comparing each node's field +0x14 with param_2. On a match it stores node[0] at this… |

## Source shards

[[S0221 008a86b0-008b1d80|S0221]] (6), [[S0242 0091af50-009200d0|S0242]] (6), [[S0243 00920130-00925ec0|S0243]] (3), [[S0244 00925fe0-0092b110|S0244]] (2), [[S0248 009392c0-0093d740|S0248]] (1), [[S0249 0093daf0-00942c60|S0249]] (2), [[S0269 009a6cf0-009aceb0|S0269]] (31), [[S0273 009bd650-009c2c50|S0273]] (2), [[S0286 00a05a50-00a0c390|S0286]] (68), [[S0287 00a0c3a0-00a13140|S0287]] (35), [[S0296 00a3f560-00a45650|S0296]] (17), [[S0297|S0297]] (6), [[S0308 00a81470-00a87b30|S0308]] (1), [[S0309 00a87ba0-00a8cdc0|S0309]] (2), [[S0310 00a8ce60-00a91b00|S0310]] (4), [[S0311 00a91bf0-00a97400|S0311]] (10), [[S0312 00a97450-00a9d360|S0312]] (5), [[S0337 00b1b5a0-00b21f20|S0337]] (1), [[S0364 00ba74d0-00bac070|S0364]] (3), [[S0365 00bac0d0-00bb1110|S0365]] (2), [[S0374 00bd9880-00be0090|S0374]] (27), [[S0383 00c05780-00c0db00|S0383]] (5), [[S0386 00c1f190-00c26aa0|S0386]] (3), [[S0392 00c42400-00c482e0|S0392]] (12), [[S0401 00c73bb0-00c7a5b0|S0401]] (35), [[S0402 00c7a610-00c7f590|S0402]] (22), [[S0413 00cb6d20-00cbc7f0|S0413]] (17), [[S0422 00ce4e30-00ceb920|S0422]] (17), [[S0424 00cf1430-00cf68d0|S0424]] (8), [[S0425 00cf69c0-00cfc480|S0425]] (18), [[S0740|S0740]] (10)
