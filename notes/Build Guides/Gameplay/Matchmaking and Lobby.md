---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Matchmaking and Lobby

Part of [[Build Guides]] › Gameplay. Related: [[Minigame Framework]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`, `unknown`) are the accepted native-map labels for a function. *Unlabelled* means the function has no accepted record, so the line comes from pseudocode or disassembly read in this pass. Source tags: *client* = the shipped executable (labels, pseudocode, bytes, vtables, reader and writer order, shard findings); *client Lua* = the decompiled UI script `ScriptsBase.lua` kept in the lab (the `Lobby`, `QuickMatch`, `MiniGameStateManager`, `ActivityPortal` and `ActivityCalendar` tables); *client locale* = the shipped string table; *client resource* = a file in the baseline client's `Resources` folder; *video* = a reviewed frame record; *wiki* = the fan wiki. Precedence is client, then video, then wiki. *Reconstructed* (R) is our own server policy, not original behaviour. OSFR-held packet captures are not used. Community emulator code and tables are not used either. No captured lobby, queue or launch frame is cited: every layout below is what the client's own readers and writers do, and every statement about what the original server sent or decided is a hypothesis built from that. Addresses are hex without `0x`; offsets are bytes; the client reads wire values straight out of memory, so integers are little-endian. No player names, chat or player-chosen names are recorded. Related: [[Minigame Framework]], [[Kart Racing]], [[Demo Derby]], [[Soccer]], [[Chess]], [[Trading Card Game]], [[Social (Friends, Guilds, Chat)]], [[Quests and Objectives]], [[Game Rules and State]].

### Overview

**The word "lobby" covers five different things in this client.** Each has its own packet family and its own server job. Keep them apart.

| # | What the player sees | Family | Runs in |
|---|---|---|---|
| 1 | The game lobby: open games for karts, derby, soccer, chess and checkers, with Create New Game, Join! and START! | 101 (lobby), 102 (lobby catalogue) | main client |
| 2 | Matchmaking: a list of queues, a Join button, a small "waiting" tracker | 141 | main client |
| 3 | Group launch: an invitation to an activity, a member list with accepted and ready states, kick, owner launch | 175 | main client |
| 4 | Activity lists: the portal, the calendar, a Play button per activity | 147, 167 (and 143 for daily counters) | main client |
| 5 | The card-game lobby (Treasure War) | its own link, serialisable commands | embedded engine inside the exe |

- **Game lobby (client Lua, video).** The server opens a lobby and three panes slide in: a top pane with the lobby title, a Create New Game button and the game list; a bottom pane with the selected game's title, options and one big button; a left pane with up to six player slots, each with an invite button. A list row shows the host, the track or encounter name, the other options and "n / max Players". Games that have already started are appended as "In Progress" rows with an elapsed time and cannot be joined. An empty list shows "No Games Found / You should create one!"; before data arrives it shows "Retrieving Game List...". Create New Game opens a form with one control per option (Max Players, Difficulty, Track, Laps, Enable AI, Auto-Start, Total Time and similar); two-value Yes/No options are toggles and the rest are choice lists. "Create!" posts the game; the panes then read "Game has been created", the creator sees START! and everyone else sees Join!, which turns into Leave Game after joining. START! puts every member into a "game is starting" state. In kart and derby lobbies a "Click Here to Switch Karts" link opens the vehicle browser. (client Lua `Lobby`; video frames of the kart lobby show the same labels, with "Waiting For Player" slots and "IN PROGRESS" rows.)
- **Who is in a lobby game.** The left-hand roster is not a lobby packet. It is the player's group (family 40): the creator is the group leader (the status "Solo" counts as leader at creation), joiners must appear as group members, and if the client finds no leader it leaves the game. Friends are invited from a slot's invite button through the ordinary group invite, and only the leader can do that (client Lua `Lobby:GroupDataUpdated`, `FriendPicker`; client locale "Only the group leader can invite friends."). See [[Social (Friends, Guilds, Chat)]].
- **Matchmaking (client Lua, client locale).** The Matchmaking button in the activity map's navigation bar (Calendar, Quests, Card Game, Matchmaking) opens the same lobby window in queue mode. It lists queues with their average wait. Selecting a queue shows its description, icon, minimum and maximum players, how many are waiting and the average wait. Solo play is one click. A group can only join a queue whose maximum is at least the group size, and the group leader can invite friends into the group from the window ("Room for N more"). Queues the player has joined show in a small tracker window with the queue name, time waited, average wait and a leave button. When a group-mate queues, the others get an event with listener, queue and player arguments (`PlayerQueuedGroup`); the locale line "[player] has put you in line for [queue]." fits it. The server can also open the window on a chosen queue (event `Lobby:JoinQueueWithID`), and the Pirates entry hard-codes queue 41 (client Lua `Pirates:Play`). The locale has the texts "Matchmaking Temporarily Unavailable", "Matchmaking is now Available" and "You have been removed from the matchmaking queue."; they fit the server-down, server-up and removed events below, but the link is not traced (client locale).
- **Invites and ready checks (client Lua, client).** For activities that need other players the server starts a launch request. Each invitee gets a pop-up from the Game Action Queue with accept and decline (title key `Minigame.GaqActivityLaunchInvite`; the client's fallback text is "%s has invited you to join an activity launch"). Members are listed with a state: invited, accepted, declined, ready, and three validation flags. The owner can invite and kick, every member can mark ready or quit, and the owner presses launch. While an invitation is unanswered the client refuses world interaction ([[Minigame Framework]]). A matchmaking invitation uses the same pop-up path with its own reply packet (141/9 in, 141/10 out). The auto-refuse options decline silently ([[Social (Friends, Guilds, Chat)]]).
- **Activity finder, portal and calendar (client Lua, wiki).** The Activity Portal is the front page with news, the item of the day and the daily wheel. The Calendar is a month grid of scheduled activities; a day lists each activity with its start time and a Play button that is enabled while it is active and joinable. Every Play button found in the script ends in `Ui.RequestToJoinActivity`. The wiki says the Map button shows "Matchmaking, your Calendar, and the Activity Finder" with the card game and quests. In the shipped scripts the Activity Finder mode is a stub (Open questions).
- **Not covered here.** What happens after a launch (the 39/16 info record, the results screen, the prize wheel) is [[Minigame Framework]]. Kart, derby and soccer play is in their own guides. The card game's own lobby is only summarised here and detailed in [[Trading Card Game]].

### How the client implements it

#### Which code belongs to which system

The generated tables below mix all five systems. Use this key.

| Generated group | Belongs to | Notes |
|---|---|---|
| `engine.lobby`: `ClientLobby` (`00bb…`), `Lobby` (`010499…`), `LobbyGame` (`008fe…`, `010488…`) | game lobby (101/102) | confirmed or likely classes |
| `engine.lobby`: `Engine::FRTWLobbyService`, `FRTWTournamentService`, `TournamentServiceBase`, `LobbyService` (`012c…`), `LobbyView` | card game | runs on the embedded engine |
| `ui.lobby`: `LobbyDataSourceManager::*`, `PopulateLobbyGameOptions`, `JoinLobbyGame` | game lobby | `00a2…`, `00c9…` |
| `ui.lobby`: `Engine::LobbyViewBase`, `UI::LobbyScreen*`, `UI::TournamentScreen`, `013b…` writers | card game | Scaleform screens |
| `ui.matchmaking`: `Matchmaking*DataSource`, the Lua `Matchmaking` handlers (`00c08…`) | matchmaking (141) | |
| `ui.matchmaking`: `Engine::FRTW*`, `Engine::Lobby*DisplayAction`, `Engine::MatchViewBase`, `UI::MatchmakingScreen*` | card game | |
| `game.matchmaking`: `EncounterMatchmaking::MatchmakingRequest` | matchmaking (141) | |
| `game.matchmaking`: `Match::*` (`014025b0…`) | card game | class id 104 |
| `net.lobby`, `net.lobby_command`, `net.match`, `unknown.lobby` | card game | `LobbyCommand_*`, `MatchCommand_*` classes |
| `ui.leaderboard`, `ui.player_positions` | neighbours | one paragraph each below |

The card-game rows are classified from their class names, their `Engine::` and Scaleform bases, and the fact that Treasure War runs inside `FreeRealms.exe` on its own engine (client; [[Trading Card Game]]). Group launch (family 175) and the activity lists (147, 167) sit in other shards (`S0393`, `S0394`, `S0376`, `S0581`), so their functions are in the [[Minigame Framework]] table, not this one.

#### Main objects (game lobby, matchmaking, group launch)

| Object | Role | Evidence |
|---|---|---|
| `BaseClient::HandleLobbyPacket` (`00923210`, vtable slot 72) | Receives family 101. Reads a 3-byte header, switches on the sub-op. | confirmed |
| Client dispatcher `009471a0` | Routes message ids 101 to slot 72, 102 to slot 77 (`0091e060`), 141 to the command processor (`00aa2980`), 143 to `00adbf00`, 147 and 167 to `00be7c50`, 175 to `009c3590`. | label unknown; cases read in this pass |
| `ClientLobby` (vtable `01824f6c`; ctor `00bb0aa0`, 0x354 bytes; dtor body `00bb0780`, likely) | The open lobby, held at client +0x6f4 (null when no lobby is open). Extends `Lobby` (`01049900`, confirmed): a 100-bucket game table at +0x14 keyed by game id, the lobby type id at +0x1a4. Adds the current game id at +0x350, a `LobbyMember` map (100 buckets) at +0x1a8 and a pair table at +0x1bc (`00bb03d0`, `00bb0b30`). | ctor unknown, dtor likely; offsets read in this pass |
| `LobbyGame` (ctor `01048940`, confirmed, 0x110 bytes) | One open game. Layout under Data and formats. The "current players" getter is vtable slot 5, `008fe190` (speculative). | |
| `LobbyDataSourceManager` (ctor `00a29500`, dtor `00a29370`, both likely), at client +0x10c | Owns the Lua data sources `BaseClient.LobbyGameList` (ctor `00a27190`, confirmed) and `BaseClient.EncounterGameList` (ctor `00a27250`, confirmed), and forwards the Lua lobby calls. | |
| `BaseClient.LobbyGameOption0` to `…9` (`PopulateLobbyGameOptions` `00c91670`, confirmed; ctor `00c91580`, confirmed) | One data source per configurable option; up to ten are built. | |
| Lobby definitions object at client +0x110 | Holds the five catalogue tables from 102/2 (parser `00919b60`, confirmed). | |
| `MatchmakingQueuesDataSource`, `MatchmakingRequestsDataSource`, `MatchmakingAvailableBuddiesDataSource` (ctors `00a830a0` unknown, `00a83d70` confirmed, `00a831a0` confirmed) | Lua tables `BaseClient.Matchmaking.Queues`, `.Requests`, `.AvailableBuddies`. | |
| `ClientCommandProcessor::ParseMatchmakingPacket` (`00aa2980`) | Receives family 141. | likely |
| `ClientActivityLaunchRequest` (ctor `00c4d2a0`) and the launch manager at client +0x218 | One object per live launch request: id at +0xa4, second header word at +0xa8, member list at +0x98, cancel flag +0xb4. Listed in a chain with links at +0xb8. | ctor unknown (`00c4d2a0`); fields from the likely labels `00c4c070`, `00c4c1a0`, `009c17c0` |
| `ClientMiniGameActivityLaunchProcessor` (`00bb3660`, likely) | Turns launch events into Lua events and joins the multiplayer mini-game. | |
| `ClientActivityManager` (ctor `00be70b0`, confirmed) | Owns the activity tables (1024-bucket maps of `ClientActivityDefinition` and `ClientScheduledActivityDefinition`, `00bdf230`, `00bdf2a0`, both confirmed) and loads `ActivityCategories.txt`, `ActivityPortalSections.txt` and `ActivityUserData.ini` (`00be6d30`). | |

#### The game lobby (families 101 and 102)

1. **Catalogue request.** The client asks for the catalogue itself. `00a27820` (speculative) runs every frame from `BaseClient::FrameTick` (`0094a970`, likely): while a one-shot flag is clear and more than 10 time units have passed since the last try, it sends 102/1 if the games table of the definitions object is empty (its row count is read at client +0x110 + 0x13c), otherwise it sets the flag. `WaitForWorldReady` (`00936d00`, confirmed) also sends it. The sender is `00a27790` (likely) through `00a273f0` (likely), which writes two 16-bit values (family 102, sub-op 1) into a world-tunnel packet. A server that never answers is asked again every 10 time units until a games row exists.
2. **Catalogue reply.** `0091e060` (confirmed) reads two int16 (family, sub-op, which must be 2), then an i32 length and that many body bytes, and runs the five table parsers in a fixed order (`00919b60`, confirmed): games `008e5450`, options `008e5580`, values `008e56c0`, `0090a070`, settings `008e5800`. A second, file-based loader (`0102a4d0`, confirmed) can load `LobbyGames`, `LobbyGameOptions`, `LobbyGameOptionValues`, `LobbyGameOptionMap` and `LobbyGameOptionValueMap` text files named with a `%s` prefix; none of those files is in the baseline client (searched by name and in the asset manifest), so the catalogue has to come from the server.
3. **Opening.** The server sends 101/6 (`SendLobbyToClient`). `00923210` frees any old lobby, allocates a 0x354-byte `ClientLobby`, reads the whole game table plus the lobby type id from the packet body (`00921740`, speculative; `0091a7e0`, unknown), and raises `Lobby:Show` (`00a243e0`, speculative). 101/12 (`ShowLobbyUi`) raises `Lobby:Show` only. 101/7 destroys the lobby and sets client +0x6f4 to null. Nothing in the client says what makes the server send 101/6 (Open questions).
4. **Rows.** The game list data source (`BaseClient.LobbyGameList`) has one row per visible game. Its columns are: 0 the game id (`+0x38`), 1 the current player count (`LobbyGame` slot 5, `008fe190`), 2 the host string, and 3 onwards one column per option of the lobby type, found through the game's option pairs (`00a25870`, likely). `00a223d0` (likely) reports the column count as options + 3. A row is visible when it is the player's own current game or when its flag byte `+0x46` is clear and its player count is below the maximum resolved from the game's option pairs (`00a22fa0`, unknown; behaviour read in this pass). Full or flagged games therefore disappear from other players' lists.
5. **Column names.** Column 0 uses string id `0xbeb`, column 1 `0xbea`, column 2 `0x19263`, and later columns the option's label id looked up through the catalogue (`00a24a60`, confirmed; `0102a690`, likely). The Lua file maps those label ids to names: 3031 Max Players, 3035 Difficulty, 3038, 416462 and 427573 Track, 3039 Required Profile Type, 3041 Enable AI, 3042 Laps, 3046 Total Time, 3213 Auto-Start, 3218 Checkers, 3219 Chess, 3222 Player skill (client Lua `Lobby.stringMap`).
6. **Create.** `Lobby:CreateNewGameStart` reads data sources `LobbyGameOption0` to `7`. An option with one row is skipped; two rows named Yes and No become a toggle; any other count becomes a choice list; the row flagged default (column 2) is preselected. Each pick calls `Ui.SubmitLobbyGameOptions` (table entry `0182e260` → `00c21610`, whose accepted label is `LoadLobbyGameUiConfig`, speculative): the Lua passes a count and then (data-source name, value index) pairs. "Create!" calls `Ui.CreateNewLobbyGame` (`00c15aa0`, unlabelled) → `00c90ec0` (unknown), which fills a record at the data-source manager +0x50 from the catalogue (`0102a290`, unknown) and calls `ClientLobby::SendCreateGame` (`00bb1cd0`, likely, vtable slot 11) with that record and the lobby type id. That sends 101/5. The client then waits for its own game to appear (`waitingForMyGameInDS`).
7. **Join, leave, start.** `Ui.JoinLobbyGame(id)` (`00c15ac0`, likely) → `00a22570` (speculative) → `SendJoinGame` (`00bb1890`, confirmed, slot 12) sends 101/1. `Ui.LeaveLobbyGame` → `00a22590` (likely) → `SendLeaveGame` (`00bb1920`, confirmed, slot 13) sends 101/2. `Ui.StartLobbyGame` → `00a225c0` (likely) → `SendStartGame` (`00bb19b0`, confirmed, slot 14) sends 101/3. `Ui.LeaveLobby` → `00a225e0` (likely) → `SendLeaveLobby` (`00bb1a40`, likely, slot 16) sends 101/9. All four use the lobby type id at +0x1a4 and (for the first three) the current game id; they send only when the client's connection field (client +0x280) is set. All go in a world-tunnel packet (gateway opcode 6, `0089ae70`, confirmed).
8. **Server answers.** 101/1 and 101/2 call `Lobby_OnJoinedGame` (`00baff30`, confirmed) with the game id or 0; it stores the id at +0x350 and raises `Lobby:JoinedGame`. 101/3 raises `Lobby:GameIsStarting` (`00a241e0`, speculative). 101/4 (`UpdateLobbyGame`) carries one full game record: the client builds a `LobbyGame`, removes any game with the same id, inserts the new one, and if the record's `+0x50/+0x54` pair equals the local player's guid pair it calls `OnJoinedGame` with that game id. That is how a creator learns its game exists. 101/8 removes a game and calls `OnJoinedGame(0)` if it was the current one. 101/11 passes an error code to `ClientLobby` slot 15 (`00bb0c30`, unknown): codes 4, 5 and 9 pick message ids `0xc90`, `0xc8f` and `0xc91`, anything else `0xc8e`, and the event `Lobby:ErrorMessage` is raised. The shipped Lua handler for that event is empty (client Lua), so where the text is shown is unresolved.
9. **Launch.** `Lobby:GameIsLaunching` is raised by `LoadZone` (`009386f0`, confirmed) and by `HandleMiniGameGameStarted` (`009b9550`, confirmed), i.e. when the zone changes or the 39/17 start arrives. The Lua then hides the lobby. `LoadZone` also raises `Lobby:Show`.
10. **Lobby type.** `Ui.GetLobbyGameType` returns the lobby type id (`00c15c40`, unlabelled). The Lua treats 1 as the kart lobby (mini-game type 10, forced profile type 5) and 2 as the derby lobby (type 11, profile 7), opens the vehicle browser for both, and uses the wider soccer-style left pane for types 3 and 6 and in matchmaking mode (client Lua `Lobby:Show`). The lobby title comes from a string id chosen by `00a22640` (likely): type 1 → `0xbec` (3052), 2 → `0xbed` (3053), 3 → `0xc8c` (3212), 4 → `0xc8b` (3211), 5 → `0xc8a` (3210), otherwise `0xbee` (3054). Which text each id names is not resolved here (hashed locale ids).
11. **In-progress games.** Rows for started games come from `BaseClient.EncounterGameList` (columns GameId, NumberOfPlayers, OwnerName, EncounterName, Difficulty, TimeElapsed; `00a224a0`, confirmed). The client fills it by sending the running-list request 41/129 (`00a22300`, unknown; `00a28d00`, likely) and reading the 41/130 response (dispatcher case 0x82 of `00aa36c0`, confirmed; `00aa3500`, confirmed).

#### Matchmaking queues (family 141)

- **Lua surface.** The module `Matchmaking` (registrar `00c08710`, confirmed; table `0182a2b0`) has five entries: `PopulateQueues` `00c08ae0`, `JoinQueue` `00c08e30`, `LeaveMatchRequest` `00c08b90`, `LeaveQueue` `00c08730` (all four unknown) and `UpdateAllQueueStats` `00c08c50` (unlabelled; read in this pass, it calls `PopulateQueues` and the queue-stats sender `00a83b80`, speculative). The Lobby window calls `UpdateAllQueueStats` then `PopulateQueues` when it opens in queue mode.
- **Sends.** All go in world-tunnel packets. `PopulateQueues` builds `ListQueuesRequest` (141/1, `00a7fb90`, confirmed) with the player's two connection words (client +0x3a0/+0x3a4). `JoinQueue(queueId)` builds `AddMatchRequest` (141/3, `00c08790`, unknown) with the queue id, those two words and one dword from the queue row, and sends it through `00c08cf0` (likely). `LeaveQueue(queueId)` calls `CancelMatchRequestsForPlayer` (`00a84150`, confirmed): for every request of this player in that queue it sends `CancelMatchRequest` (141/6, `00a7fc60`, confirmed) carrying the request's 64-bit id. `LeaveMatchRequest(id)` sends the same packet for one request id. The queue-stats request is 141/13 (`00a7fd30`, confirmed). The invitation reply is 141/10 (`00a87ad0`, confirmed), sent from `GroupEncounterRespondToInviteMatchmaking` (`00a979a0`, confirmed) with an accept flag and two ids.
- **Receives** (`00aa2980`, likely, cases 2, 4, 5, 9, 0xc, 0xe, 0xf, 0x12).

| Sub | Class (ctor) | Client action |
|---|---|---|
| 2 | `ListQueuesResponse` (`00a9f1d0`, confirmed) | Replaces the queue list with the received definitions and refreshes the queue table. |
| 4 | `AddMatchRequestResponse` (`00aa1140`, confirmed) | `OnPlayerQueuedGroup` (`00a83ed0`, confirmed): drops this queue's earlier requests, appends the request, and for a request made by someone else raises the event `PlayerQueuedGroup` with listener, queue and player. |
| 5 | `ClearMatchRequest` (`00aa1200`, confirmed) | `RemoveMatchmakingRequestByIds` (`00a835a0`, likely): removes requests by id. |
| 9 | `MatchInvitationRequest` (`00a9a580`, likely) | Looks up a string by id, queues a "group announce encounter" pop-up (`Gaq_AddGroupAnnounceEncounter`, `00a0d550`, likely) and raises `ActivateEncounterRequest` on the group screen (`00a7d830`, confirmed). The pop-up's responders send 141/10 or the encounter reply. |
| 12 (0xc) | `SelectQueueForUser` (`00a99010`, likely) | Raises `Lobby:JoinQueueWithID` with the queue id from the body. |
| 14 (0xe) | `QueueStatsResponse` (`00a9df30`, confirmed) | Assigns two int lists (`00a81040`, likely) that feed the PendingRequests and AverageWait columns. |
| 15 (0xf) | `MatchmakingServerStatus` (`00a990c0`, likely) | `00a83c00` (unknown label): on "up" it sends 141/1 and raises `MatchmakingServerUp`; on "down" it clears the queue list and raises `MatchmakingServerDown`. In both cases `00a839d0` (likely) then empties the request list. |
| 18 (0x12) | `RemoveParticipantFromQueueResponse` (`00a9a4b0`, likely) | `OnPlayerMatchmakingRemoved` (`00a83490`, confirmed): removes this player's requests for that queue id and raises `PlayerMatchmakingRemoved`. |

- **What the client does not show.** Nothing here says how a match forms or what packet starts the game once enough players are queued. The two candidates the client can handle are the invitation pair 141/9 and 141/10 and the group-launch events `OnMatchmakingStart`, `OnMatchmakingLaunch` and `OnMatchmakingCancel` (175/14 to 16); both end in the ordinary mini-game entry ([[Minigame Framework]]).
- **The Quick Match tracker.** `QuickMatch` (client Lua) lists the rows of `BaseClient.Matchmaking.Requests` (column 1 the queue id, column 4 the creation time) and shows each queue's name and average wait. Its leave button calls `Matchmaking.LeaveQueue`.

#### Group launch, invites and ready (family 175)

- **One object per request.** The first message of a request is sub-op 1 (`InviteDetails`). The family entry point `009c3590` (confirmed) reads the header, sends sub-op 1 to `HandleActivityLaunchRequest` (`009c3270`, confirmed) and sub-op 6 to `HandleActivityLaunchCancellation` (`009c29d0`, confirmed). Every other sub-op finds the request by id (`009c17c0`, likely) and goes to `HandleActivityLaunchMessage` (`00c4e490`, confirmed). For a new request `009c3270` builds a `ClientActivityLaunchRequest`, applies the details, and on a decline path sends an invite response at once (`00c4dfe0`); otherwise the request is shown through the Game Action Queue (`00bb38c0`, confirmed).
- **Invite details.** `00c4d980` (likely) applies the details and, if the local player is not in the roster, logs that it is "not part of the invite" and cancels (flag +0xb4). So the roster in `InviteDetails` must include every recipient.
- **Request fields** (the 15 columns of the request data source, `00c4e6b0`, confirmed): Id, NameId, DescriptionId, ImageSetId, SystemHashkey, SystemDefinitionId, Requestor, Members, MembersOnly, MinPlayers, MaxPlayers, AllowedToModifyMembersList, MembersAutoAccept, MembersInstantlyReady, DataBlobBufferSize. **Member columns** (`00c4e790`, confirmed): ActivityId, PlayerName, PlayerGuid, IsFoundingMember, InSameMinigame, InviteStatus.
- **Member state byte** (`00c4ce40`, likely, log format read in this pass): bit 7 validating, 6 validated, 5 failed validation, 4 invited, 3 accepted, 2 declined, 1 ready; bit 0 is not used by the log.
- **Events to Lua** (`00bb36d0`…`00bb4ca0`, mostly confirmed): `ActivityEvents:OnInviteSent`, `OnInviteResult`, `OnInviteAutoDeclined`, `OnKicked`, `OnMemberKicked`, `OnMemberRemoved`, `OnMatchmakingStart`, `OnMatchmakingLaunch`, `OnMatchmakingCancel`, `OnLaunchCanceled` (which also raises `GameEvents:OnMiniGameFailed`). `MiniGameStateManager` listens to five of them; a cancel with reason 3 closes the start screen, and a kick closes it and shows an error message built from the kicker's name and the string `ui_social_GameRemoved` (client Lua).
- **Launch.** `ActivityLaunched` (sub-op 5, handler `00c4c580`, unknown) carries the participant list; the client tests whether the local guid is in it, logs "You are part of the launch = n", and the processor joins the multiplayer mini-game (`00bb4f50`, likely: `MiniGameStateManager:joinMultiplayerMiniGame`).
- **Lua entry points** (table addresses read in this pass): `Ui.InviteToActivity` `00c1b060` → `SendPlayerInvite` (`00c4e2d0`, likely); `Ui.KickFromActivity` `00c1b0e0` → `SendPlayerRemove` (`00c4e3b0`, likely); `Ui.ReadyForActivity` `00c1b1c0` → `SendReadyNotification` (`00c4e1b0`, confirmed); `Ui.QuitActivity` `00c16e80` → `SendQuitNotification` (`00c4e240`, likely). The GAQ responders `AcceptActivityLaunchInvite` (`009c19a0`) and `DeclineActivityLaunchInvite` (`009c1c60`, decline reason 3), both confirmed, call `SendInviteResponse` (`00c4dfe0`, likely). `SendOwnerLaunchRequest` (`00c4e090`) and `SendOwnerMatchmakingRequest` (`00c4e120`) have no mapped caller.
- **Not a lobby packet.** A multiplayer mini-game is also created from the start screen with `Ui.CreateMultiplayerMiniGame` (`00c174e0`, `MiniGame_RequestCreateMiniGame` `009bd590`, likely), which sends 39/3 or 39/4 ([[Minigame Framework]]).

#### Activity lists, portal and calendar (families 147, 167, 143)

- **Receive.** `00be7c50` (likely) dispatches family 147 to `00be7360` (unknown) and 167 to `00be7b30` (likely) / `00be7660` (likely). 147 sub-op 1 applies an activity profile list (six dwords per node) and sub-op 2 raises `ActivityPortal:OnJoinError`. 167 starts with a 16-bit service number: service 1 carries sub-op 1 (a list of activities, `00be79d0`, likely) and sub-op 5 (a featured-status update, `00be61d0`, speculative); service 2 carries the scheduled-activity list (`00be7200`, likely, 0x80-byte records). After a list arrives the client rebuilds its All, Featured, Preferred, MemberOnly and Scheduled lists (`00cea720`, `00ceaa40`), logs "calling zoning complete from minigame manager" and calls `009b79d0(0)`.
- **Send.** `ActivityJoinRequest_SendByType` (`00be7d80`, likely) looks the activity up in a 1024-bucket table and sends `ActivityPacketJoinActivityRequest` (167 service 1, sub-op 2, `00bdefa0`, likely) through the world tunnel for entry type 1 or the client tunnel for type 2. A list refresh request (167 sub-op 4) is `008a2330` (confirmed).
- **Lua tables.** `Activities.All` (19 columns, client Lua `Activities_col`): Id, DisplayNameId, DisplayDescriptionId, NameId, DescriptionId, Category, ImageSetId, ActivityPositionId, CanPlayerJoin, IsFeatured, IsPreferred, FeaturedRewardTooltipStringId, TutorialActivityId, AppSystemId, MembersOnly, DetailImageFilename, ThumbnailImageFilename, Difficulty, MysteryChestIcon. `Activities.Category<N>` has the same columns. `Activities.Scheduled` (columns 1 to 30, client Lua `DsCol`) has the GMT and local start and end dates and times, epochs, SecondsUntilStart and SecondsUntilEnd, display times and non-localised dates (columns 1 to 16), followed by the activity columns ActivityId to AppSystemId (17 to 30). The calendar groups rows by the non-localised start date, shows a Play button when `SecondsUntilStart <= 0 <= SecondsUntilEnd` and `CanPlayerJoin` is set, and treats `AppSystemId` 3 as a social activity that is only launchable on its scheduled day.
- **Repeating activities** (143, daily counters and the wheel) are only touched here for the portal's wheel counter (`Ui.GetRepeatingActivityCount`); see [[Minigame Framework]].

#### The card-game lobby (Treasure War)

The card client has its own lobby on a separate server link. It speaks serialisable command objects, not families 101 to 175 (client; [[Trading Card Game]] for transport and framing). The lobby and match commands, with class id, version and `Read` address (from the binary's command registry, read in this pass):

| Class (id) | Ver | Read | Body as the client reads it |
|---|---|---|---|
| `LobbyCommand` base (91) | 1 | `013b2910` (confirmed) | class header, then AccountID (+4) |
| `MatchCommand` base (98) | 1 | `013b2530` (confirmed) | `LobbyCommand` fields, then Match ID (+8) |
| `LobbyCommand_Join` (92) | 5 | `013b36a0` (unknown) | fields +8 and +0xc, then version-gated strings and objects (+0x10, +0x2c, +0x3c, +0x58, +0x5c, +0x78). The engine builds it from three values: group id, role, and a string (`Engine::LobbyService` slot 3 `013b12d0`, speculative), which matches the Scaleform `JoinLobby` action values GroupID, Role and Password (likely). |
| `LobbyCommand_Leave` (93) | 3 | `013d9bd0` (unlabelled) | not decoded here |
| `LobbyCommand_AddGroups` (94), `RemoveGroups` (95), `ConfigureGroup` (96) | 1 | `0143f3d0` (likely), `014c23e0` (confirmed), `014c5c70` (speculative) | a counted dword list (+8) for add and remove; two dwords (+0xc, +0x10) for configure |
| `LobbyCommand_SendText` (97) | 3 | `013b3e80` (confirmed) | ints +8, +0xc, +0x10, string +0x14, optional bool +0x30 and int +0x34 |
| `LobbyCommand_BulkJoin` (106), `ChangeStatus` (137), `ChangeBulkStatus` (279), `ChangeLobbyDisplay` (291), `DisplayStats` (292), `UpdateLobbyStats` (412) | 1 to 4 | `014cc7f0` (unknown), `013d82d0` (unlabelled), `014c7c00` (speculative), `014c6100` (speculative), `014c5500` (unknown), `014ba470` (confirmed) | `UpdateLobbyStats` v3 reads three containers (+8, +0x14, +0x20), an int +0x2c (v2+) and a container +0x30 (v3) (confirmed) |
| `MatchCommand_SetReady` (99) | 1 | `013b3a40` (confirmed) | base, then one int stored as a bool (+0xc) |
| `MatchCommand_SelectDeck` (101), `SelectStartPosition` (102), `SetBid` (100), `BidPlaced` (125) | 1 to 2 | `013b1e20` (unknown), `014b4c20`, `014b4430`, `014b5840` (all confirmed) | deck: string +0xc, bool +0x28, nested object +0x2c; start position: int +0xc and an int list +0x10; bid: int +0xc |
| `MatchCommand_LaunchGame` (116), `CWMatchCommand_LaunchGame` (140008) | 1 | `014b4f50` (confirmed), `01459860` (unlabelled) | base only; the client then builds its game object |
| `MatchCommand_SetMatchState` (124), `SetTimer` (295), `SetDeckArchetype` (470) | 1 | `014b3970`, `014b34e0`, `014b2f80` (all confirmed) | base, then one int at +0xc (state, duration, archetype) |
| `MatchCommand_SetBulkMatchState` (294) | 1 | `014b4140` (confirmed) | base, then two containers: group ids (+0xc) and match states (+0x1c) |
| `MatchCommand_InitiateChallenge` (486) | 2 | `014b53f0` (confirmed) | base, then ints +0xc, +0x10, +0x14 |
| `EventCommand_RequestJoinMatchingQueue` (570), `NextMatchingTime` (514), `NotifyMatchEnded` (531) | 1 to 2 | `01446f70`, `01535470`, `01534fb0` (all confirmed) | event-service queue commands; not decoded here |

Each level of a command's class hierarchy reads and writes its own class header, so a leaf command carries one header per level before its own fields (client; the base readers `013b2910` and `013b2530` each call the header slot at their own level; project reviewer notes on `013aede0`). The card UI side uses Scaleform actions: lobby actions `QueryCreateMatch`, `RequestCreateMatch`, `QueryMatch`, `CancelQueryMatch`, `CancelCreateMatch`, `RequestQuickJoin`, `StartSinglePlayerGame`, `StartCampaignGame`, `DeckSelected`, `PerformSetMap` (`Engine::FRTW…` classes at `012795…` to `0127bb70`), and display actions `AddLobbyGroup`, `AddLobbyParticipant`, `BulkAddLobbyParticipant`, `StartBulkAddLobbyGroup`, `EndBulkAddLobbyGroup`, `RemoveLobbyGroup`, `RemoveLobbyParticipant`, `ChangeLobbyStatus`, `SetDisplayLobby`, `DisplayMOTD`, `UpdateLobbyStats`, `LaunchGame` (`0127c1c0` to `0127d100`). The match list record has GroupID, ParentGroupID, LobbyType, GameID, MatchState, IsLaunched, Players (role 2) and Observers (role 3); the filter flags are IsBuddyListOnly, IsPasswordProtected, IsCanBeWatched and IsGuildOnly (`MatchViewBase` `013afb10`, confirmed; `013af7c0`, likely). The quick-join handler (`0127b170`, unknown) builds a debug "playtest" match with a test deck when a playtest flag is set; that is developer behaviour.

#### Neighbours on the generated table

- **Leaderboards (web).** `ClientLeaderBoardManager` reads `WebResources/GameLeaderBoardUrl` (`00b85d50`, likely; the baseline `ClientConfig.ini` has the key), sends an HTTP request (`00b85e80`, confirmed), and parses an XML `result` with a `totalPages` attribute and `entry` nodes carrying `guid`, `rank`, `id`, `score`, `type` and a name (`00b86720`, `00b86390`, both confirmed). The data source is `BaseClient.LeaderBoard` (`00cdf050`, confirmed). The service is gone; see [[Minigame Framework]].
- **Player positions.** `BaseClient.PlayerPositions` (`00a86050`, confirmed) has columns Guid, Name, X Location, Z Location, Is Friend, Is Group Member, Is In Encounter (`00a85900`, confirmed), filled from `CommandPacketRequestPlayerPositionsReply` (command family 26, sub-op 35, `00aa1880`, confirmed). It feeds the map and is not part of lobby play.

### Data and formats

**Client resources.** From the baseline client's `Resources` folder.

| File | Contents | Notes |
|---|---|---|
| `ActivityCategories.txt` | `ID^CONTENT_ID^NAME_ID^DESCRIPTION_ID^THUMBNAIL_IMAGE_FILENAME^`, 16 rows | loaded by `00be5b30` (unknown) via the manager (`00be6d30`, confirmed) |
| `ActivityPortalSections.txt` | `ID^NAME_ID^DESCRIPTION_ID^`, 10 rows | same loader |
| `FirstTimeEvents.txt` | row 86 `FtesActivityFinder`, row 88 `FtesMatchmaking` | first-time-event tutorials; the Lua handler `FtesMatchmaking` is empty (client Lua) |
| `ClientConfig.ini` | `GameLeaderBoardUrl` | leaderboard endpoint |
| `ActivityUserData.ini` | empty in the baseline; written per character by the activity manager (keys `Character.%llu`, `Tutorial.%d`; load `00be6d30`, confirmed; save in the destructor body `00be6bb0`, unknown) | client-side only |
| `%sLobby*.txt` (five names) | read by `0102a4d0`; not present | see above |

**Wire headers.** Every field is read from the packet body by the family's reader. The families differ:

| Family | Header | Reader / writer |
|---|---|---|
| 101 lobby | i16 family, i8 sub-op, then body | `008d90b0` (likely) / `00bb0e90` (likely) |
| 102 lobby catalogue | i16 family, i16 sub-op; the reply then has an i32 length and body | `008d67a0` / `00a273f0` (likely) |
| 141 matchmaking | i16 family, i16 sub-op | `00a9d4f0` (likely) / `00a82a90` (likely) |
| 175 launch | i16 family, i32 sub-op, i32 request id, i32 second id | `009c1e00` (likely) / `00c4cd20` (unknown) |
| 147, 167 activity | i16 family, then a 16-bit sub-op (147) or a 16-bit service number and a sub-op (167); exact widths not verified | `00be7360`, `00be7c50` |

The request id and second id in 175 are the request's `+0xa4` and `+0xa8` (`00c4dfe0`, likely). Which of the two is "activity id" is unknown.

**Lobby packets (family 101).**

| Sub | Name | Direction | Body after the header |
|---|---|---|---|
| 1 | JoinLobbyGame | both | u32 game id, u32 lobby type id (send: `008a3da0` confirmed). Inbound the client uses the first word as the joined game id. |
| 2 | LeaveLobbyGame | both | u32 game id, u32 lobby type id (`008a3e60`, confirmed). Inbound: leave acknowledged. |
| 3 | StartLobbyGame | both | u32 game id, u32 lobby type id (`008a3f20`, confirmed). Inbound: raises `Lobby:GameIsStarting`. |
| 4 | UpdateLobbyGame | S>C | i32 length and a blob holding one `LobbyGame` record (`008f9650`, likely) |
| 5 | CreateLobbyGame | C>S | see below (`00bb0b80`, confirmed; writer `00bb1820`, likely) |
| 6 | SendLobbyToClient | S>C | i32 length and a blob: game table, then i32 lobby type id |
| 7 | SendLeaveLobbyToClient | S>C | none (lobby destroyed) |
| 8 | RemoveLobbyGame | S>C | u32 game id (`008f9710`, likely) |
| 9 | LeaveLobby | C>S | u32 lobby type id (`00bafe60`, confirmed; writer `00bb11b0`, likely) |
| 11 | LobbyErrorMessage | S>C | u32 error code (`008f98b0`, likely) |
| 12 | ShowLobbyUi | S>C | u32, u32, ignored by the client (`008f9870`, unknown) |

Sub-ops 5, 9, 10 and every value above 12 are not handled on receive: `00923210` falls into its default case and returns 0 (confirmed). On the receive side the 1, 2, 3 and 12 readers take the header plus two u32 (`008d9210`, `008d9280`, `008d92f0`, likely).

**`CreateLobbyGame` body (C>S, sub-op 5).** After the 3-byte header: an 8-byte guid pair (the local player, `008fb810`, unknown); u32 `+0` of the record; u32 option count, then that many (u32, u32) pairs (option id, value id; `00bb1260`, unknown; the pair meaning is read from the lookup `00a25870`, likely); three u32; two length-prefixed byte strings (`00a0e8f0`, likely; each is a u32 length and the bytes, `00911c30`); and a final u32 (`+0xb0`, the lobby type id passed to `SendCreateGame`, inferred). The first u32 and the two strings are not decoded further. Treat the strings as untrusted text.

**`LobbyGame` record** (blob of 101/4, and one entry per game in 101/6 after a u32 key). Read by `00911130` (unknown; read in this pass). Offsets are in the client object.

| Order | Width | Field | Notes |
|---|---|---|---|
| 1 | i32 | `+0x38` | game id: the table key, the argument of `OnJoinedGame`, the "GameId" column (likely) |
| 2 | i32 | `+0x3c` | unknown |
| 3 | i32 | `+0x60` | unknown |
| 4 | i32 count, then (u32, u32) pairs | `+0x64` list | option id and value id (`ApiLobbyGameOption` list; `008e7c50`, unknown) |
| 5 | info block (`008e7290`, speculative) at `+0x74` | | three i32, then two length-prefixed byte strings; the first string is the host column (`00a25870`) |
| 6 | u8 | `+0x46` | closed flag. When set the game is hidden from other players' lists (`00a22fa0`) and `Ui.GetLobbyGameId` returns 0 for the player's own current game (`00a22610`, likely) |
| 7 | i32 | `+0x48` | current player count (vtable slot 5, `008fe190`, speculative); the constructor defaults it to 1 |
| 8 | 2 x u32 | `+0x50/+0x54` | creator guid pair: compared with the local guid on 101/4, and returned by `Ui.GetLobbyGameCreatorGuid` (`00a23120`, likely), which the Lua uses to show START! to the host |
| 9 | u32, u32, u32 | `+0xfc`, `+0x100`, `+0x104` | `+0xfc` unknown; `+0x100` is the game's description string id (`00a226a0`, likely; default `0xbee`); `+0x104` is its icon id (`00a226e0`, speculative). The Lua reads them through `Ui.GetLobbyGameDescription` and `Ui.GetLobbyGameIcon` |
| 10 | u8 | `+0x108` | unknown |

The 101/6 body is: a clear of the old table (`00912310`, unknown), a signed count, then per game a u32 key and the record above (`0091a7e0`, unknown), then an i32 stored at `ClientLobby +0x1a4` (the lobby type id, `00921740`, speculative).

**Catalogue (102/2 body after the length).** Five tables, each a u32 row count followed by rows that start with a u32 key (parse order and widths from the row readers `008cd0f0`, `008cd1d0`, `008cd280`, `008cd360`, all confirmed, and `0090a070`, unknown).

| Table | Parser | Row after the key | Likely role |
|---|---|---|---|
| 1 | `008e5450` (likely) | 4 x u32 | games. The client finds the row whose second field equals the lobby type id, takes its first field as an id and looks that id up as a key in the same table (`0102a5a0`, likely) |
| 2 | `008e5580` (likely) | 3 x u32 | options |
| 3 | `008e56c0` (likely) | 3 x u32 | option values |
| 4 | `0090a070` (unknown) | 2 x i32, then a length-prefixed byte string | unknown; 10 buckets |
| 5 | `008e5800` (likely) | 5 x u32, read in the order fields 0, 1, 2, 4, 3 | unknown (settings) |

What the client builds from them (client Lua + `0102a690`): for the open lobby type, up to eight (Lua) or ten (native) option data sources; each option has a label string id, and each value has a label, an id and a default flag. The column counts and the lookups above fix what the server must be able to express, but which row field is the option id, the value id, the label id or the default flag is **not** resolved. Never copy any retail catalogue contents from memory; design them to satisfy the Lua (Max Players, Difficulty, Track, Laps, Enable AI, Auto-Start, Total Time).

**Matchmaking queue row** (`00a9d9c0`, likely: 19 dwords at `+4` to `+0x4c` in wire order, then two bytes at `+0x50`, `+0x51`; columns by `00a81e90`, confirmed, and `00a7f2f0`, confirmed):

| Wire dword | Column | Meaning as the client uses it |
|---|---|---|
| 1 | 0 QueueId | key |
| 2 | 1 Name | string id (resolved through the client string table) |
| 3 | 2 MatchType | value cell; meaning unknown |
| 4, 5 | 3 MinPlayers, 4 MaxPlayers | shown in the window; MaxPlayers gates group joins |
| 6, 7 | 5 MinTeams, 6 MaxTeams | unknown |
| 8 | 7 MaxGameStartDelay | unknown |
| 9 to 15 | 8 to 14 Param1 to Param7 | unknown |
| 16 | 17 EncounterDescription | string id shown in the window |
| 17 | 18 EncounterIcon | image id shown in the window |
| 18, 19 | no column | read but unused by the getter |
| byte 1 | 19 MemberOnly | |
| byte 2 | no column | unused |

Columns 15 PendingRequests and 16 AverageWait do not come from the row: they come from the two int lists of 141/14, indexed by row (`00a81e90` fetches them through `0079db00`, speculative). The Lua shows average wait in seconds as m:ss and treats 0 as "-".

**Matchmaking request record** (141/4 and 141/5 body after the header; reader `00aa16f0`, likely; the add-request writer `00c08910`, unknown, mirrors it). Offsets in the request object.

| Order | Width | Field | Use |
|---|---|---|---|
| 1 | 2 x u32 | `+8/+0xc` | request id, 64-bit (the RequestId column, `%llu`) |
| 2 | u32 | `+0x10` | unknown |
| 3 | u32 | `+0x14` | queue id |
| 4 | 2 x u32 | `+0x18/+0x1c` | requesting player's connection pair; compared with client +0x3a0/+0x3a4 |
| 5 | u32 x3 | `+0x20`, `+0x50`, `+0x54` | unknown |
| 6 | u32 count, then per member a u32 key and a participant record (`00aa0550`, likely; `00a24750`, likely: a 32-bit pair plus flags) | `+0x24` list | group members |
| 7 | 2 x u32 | `+0x58/+0x5c` | unknown (a time value in the constructor) |
| 8 | u8 | `+0x61` | unknown |
| 9 | info block (`008e7290`) | | three i32 and two length-prefixed strings |

The Requests data source columns are RequestId, QueueId, RequestingPlayerGuid, IsGroupRequest and CreationTime (`00a7f430`, confirmed). The AvailableBuddies columns are BuddyName and BuddyGuid (`00a7f4c0`, confirmed); the rows are built from the Guild members and Friends lists whose status string matches one value (`00a82590`, unknown; the value is `DAT_0180c498`, not recovered).

**Group launch packets (family 175).** Sub-op constructors from the base `009c0c30` (confirmed).

| Sub | Name | Direction | Body after the 14-byte header |
|---|---|---|---|
| 1 | InviteDetails | S>C | u8 `+0x2c`; u32 pair `+0x30/+0x34`; length-prefixed string; u32 `+0x6c`; optional member roster (u32 count, then per member u32 id, 8-byte guid, length-prefixed name, two flag bytes: `00c4ccb0`, `009c24e0`); optional extra record (`009c1f40`) (`00c4d060`, likely) |
| 2 | InviteMemberReply | S>C | 8-byte guid, length-prefixed name, u8 (invite sent), u32 (code) (`00c4acd0`, likely) |
| 3 | InviteMemberResult | S>C | same layout, u8 means accepted (`00c4adb0`, likely) |
| 4 | PlayerHasBeenAccepted | S>C | header only (`00c4c240`, unknown) |
| 5 | ActivityLaunched | S>C | list of 8-byte guids (`00c4a810`, likely) |
| 6 | ActivityCanceled | S>C | header (carries the request id) and one more value that the handler passes to the processor as the cancel reason (`009c29d0`, confirmed; body parser `009c2780`, likely) |
| 7 | AddMember | S>C | one member record (`009c24e0`, speculative) |
| 8 | RemoveMember | S>C | two u32 (`00c4aeb0`, likely) |
| 9 | UpdateMemberState | S>C | u32 member id, u8 state byte (`00c4b9e0`, likely) |
| 13 | KickMemberEvent | S>C | kicker guid and name, kicked guid and name (`00c4af20`, likely) |
| 14, 15, 16 | OnMatchmakingStart, Launch, Cancel | S>C | header only (`00c49ba0`, `00c49c60`, `00c49d20`, confirmed) |
| 17 | InviteResponse | C>S | u8 accepted, u32 decline reason (`00c4d160`, likely) |
| 18 | MemberReady | C>S | header only (`00c498a0`, confirmed) |
| 19 | MemberQuit | C>S | header only (`00c49960`, confirmed) |
| 20 | InviteMemberRequest | C>S | 8-byte guid and name (`00c4de00`, unknown) |
| 21 | KickMemberRequest | C>S | 8-byte guid and name (ctor `00c4b470`, confirmed; the sender body `00c4def0` was not read, so this assumes the sub-op 20 layout) |
| 22 | OwnerLaunchRequest | C>S | header only (`00c49a20`, confirmed) |
| 23 | OwnerMatchmakingRequest | C>S | header only (`00c49ae0`, confirmed) |

### Network

**Transport.** Everything the client sends in these families goes in a world-tunnel packet (`PacketTunneledClientWorldPacket`, gateway opcode 6, built by `0089ae70`, confirmed) through `00926360` (likely): 101 (`00bb1350`, likely, and its siblings), 102/1 (`00a273f0`, likely), 141 (`00a83670` unknown, `00a83790` confirmed, `00a838b0` confirmed, `00c08cf0` likely), 175 (`00ace110`, unknown, through `008a9140`, likely), and the activity join for entry type 1 (`00be77b0`, confirmed). Entry type 2 uses the client tunnel instead (`00be78c0`, confirmed). Inbound packets reach the handlers through `009471a0` as listed above. No packet is sent while the connection field at client +0x280 is zero. The datagram framing under the tunnel is unverified here.

**Client to server (lobby, matchmaking, launch).**

| Family/sub | Sender | Trigger |
|---|---|---|
| 102/1 | `00a27790` | catalogue missing, or world ready |
| 101/1, 2, 3, 5, 9 | `00bb1890`, `00bb1920`, `00bb19b0`, `00bb1cd0`, `00bb1a40` | Join!, Leave, START!, Create!, close lobby |
| 41/129 | `00a28d00` | refresh in-progress games |
| 141/1 | `00c08ae0` (unknown), `00a83c00` (unknown) | queue window opens; matchmaking comes up |
| 141/3 | `00c08cf0` | Join on a queue |
| 141/6 | `00a84150`, `00c08b90` | leave queue, leave request |
| 141/10 | `00a979a0` | accept or reject a match invitation |
| 141/13 | `00a83b80` | refresh queue stats |
| 175/17, 18, 19, 20, 21 | `00c4dfe0`, `00c4e1b0`, `00c4e240`, `00c4e2d0`, `00c4e3b0` | accept or decline, ready, quit, invite, kick |
| 167 sub 2, sub 4 | `00be7d80`, `008a2330` | Play button, list refresh |

**Server to client.** 101/1, 2, 3, 4, 6, 7, 8, 11, 12; 102/2; 41/130; 141/2, 4, 5, 9, 12, 14, 15, 18; 175/1 to 9, 13 to 16; 147/1, 2; 167 (service 1: sub-ops 1, 5; service 2: scheduled list). The bodies are in Data and formats.

**Lua events these packets raise** (the names are string literals in the cited handlers, passed to the Lua call helper `ClientLuaCallByName`, `009797a0`): `Lobby:Show`, `Lobby:JoinedGame`, `Lobby:GameIsStarting` (speculative), `Lobby:GameIsLaunching` (speculative), `Lobby:ErrorMessage`, `Lobby:JoinQueueWithID`, `ActivityPortal:OnJoinError`, `ActivityEvents:*` (above), `MiniGameStateManager:joinMultiplayerMiniGame`. Client-side events `PlayerQueuedGroup`, `PlayerMatchmakingRemoved`, `MatchmakingServerUp` and `MatchmakingServerDown` are raised through the client event system and no shipped Lua handler listens to them by those names (client Lua search).

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour.

1. **Answer 102/1 with a complete 102/2.** Without a games row the client asks again every ten time units. Provide at least the kart and derby lobby types with the options the Lua expects. Send the five tables in order with the widths above; keep key and value ids unique per table. Which row field is which stays unresolved, so test against the real client and the Lua (R).
2. **Open and close lobbies.** Send 101/6 with the current game table and the lobby type id when the player enters a lobby gate; the full table replaces the client's old one. Send 101/7 to close it. Use the lobby type ids the Lua knows (1 kart, 2 derby, 3 and 6 soccer-style pane) and map the rest through the catalogue (client Lua). Everything about what triggers this on the world side is open.
3. **Create.** On 101/5 validate the option pairs against the catalogue (the client only sends what it was given, but the server must not trust it), allocate a non-zero game id, store the creator, then send **101/4** to the creator with the record's creator pair set to the creator's guid pair. That one packet is how the client learns its game exists and calls `OnJoinedGame`; without it the creator waits forever (client). Also create the creator's group with the creator as leader, because the left pane is the group (R, from the Lua). Treat the two strings in the create body as untrusted text and filter them.
4. **Join, leave, start.** On 101/1 check the game exists, is not flagged, and is below its maximum; add the player to the creator's group and send 101/1 back with the game id, and 101/4 to the lobby with the new player count. On 101/2 remove the player from the group and the game, send 101/2 back (the client then clears its current game) and 101/4 with the new count (R). On 101/3 accept only from the creator with enough players, send 101/3 to every member, then drive the entry sequence (encounter and 39/16, see [[Kart Racing]], [[Minigame Framework]]); only after that resend the game record with its flag byte set so it leaves other players' lists (set earlier, `Ui.GetLobbyGameId` returns 0 for members and the Lua may drop them; the Lua ignores that once the start message has arrived, `gameIsStarted`). Do not use 101/8 on a running game: it also clears the current game of every member that has it selected (client, `00923210`). On 101/9 drop the player from the lobby and any game in it. Report refusals with 101/11 using codes 4, 5 or 9 where the message fits and anything else for a general error (R).
5. **Keep the list honest.** The client shows a game only if it is the player's own or its flag byte is clear and `+0x48` is below the maximum resolved from its option pairs. Update `+0x48` with every join and leave and set the flag when the game starts or fills (client, `00a22fa0`).
6. **In-progress rows.** Answer 41/129 with a 41/130 list of running encounters for the lobby's game types (columns GameId, NumberOfPlayers, OwnerName, EncounterName, Difficulty, TimeElapsed). The record body is the encounter details record (`00a29120`, confirmed); its field meanings are partly unresolved ([[Kart Racing]]).
7. **Matchmaking.** Send 141/15 "up" when the service is ready (the client then requests the queue list itself), and "down" to clear. Answer 141/1 with 141/2 (queue rows as above). Answer 141/13 with 141/14 (pending counts and average waits, one entry per queue row in the same order; the row alignment is inferred from the column getter). On 141/3 check the queue exists, the group size is within the maximum, the membership rule holds, then send 141/4 to every group member with the same request record (the client treats a record made by someone else as "put you in line"). On 141/6 send 141/5 (and 141/18 when the player is removed by the system). Enforce the one-invite-per-person-per-30-seconds rule from the locale on the server (R; the text exists but the enforcement point is not found in the client). Allow a server-initiated 141/12 to open the window on a queue. How a full queue becomes a game is open (R: use 141/9 and 141/10 or the group-launch path, then the normal 39/16 entry).
8. **Group launch.** Create a launch request with a non-zero id. Send **175/1** to each invitee with a roster that includes them (otherwise the client cancels). Then send 175/2 and 175/3 to the inviter for invite results, 175/7, 175/8 and 175/9 to keep every member list in step, 175/13 for kicks and 175/6 to cancel (reason 3 closes the start screen). Accept 175/17 to 175/21 only from members of that request; the owner alone may kick, invite or launch (R; the request also carries `AllowedToModifyMembersList`). When all required members are accepted and ready, send 175/5 with the participant guids, then the mini-game entry. Support `MembersAutoAccept` and `MembersInstantlyReady` flags if a game wants them.
9. **Activity lists.** After login and after zoning send the 147/1 profile list and the 167 activity and scheduled lists. The calendar and Play buttons need `CanPlayerJoin`, `MembersOnly`, the schedule epochs and the server-time flag; the schedule record is the 0x80-byte layout (`00be7200`). Answer the join request (167 sub 2) by starting the activity ([[Minigame Framework]]) or send 147/2 to raise `OnJoinError` (R for the reasons).
10. **Card game.** Treat Treasure War as a separate service with its own commands ([[Trading Card Game]]). Do not mix its lobby with the game lobby.
11. **Leaderboards.** Serve or leave unreachable the web endpoint; never accept client-computed scores (R).
12. **Safety.** Host and any free-text strings in lobby games come from players; filter them, never log them in public notes (R).

### Video and wiki evidence

- **Kart lobby (video).** Two recordings show the kart lobby: the list with a Create New Game button and a "Wildwood" row marked IN PROGRESS (`eCjjzP3ef0E/00036`); the Create New Game form with Max Players, Difficulty, Track, Enable AI, Laps, Auto-Start and a Create! button, plus the "Click Here to Switch Karts" link (`eCjjzP3ef0E/00022`, `00024`); the created game with "Game has been created", the host slot, "Waiting For Player" slots and a START! button, with option lines for Max Players 5 or 2, Difficulty easy, Enable AI No and Laps 5 or 2 (`eCjjzP3ef0E/00028`, `kNZtSg_Qrsw/00016`); a list with Join! and IN PROGRESS rows (`eCjjzP3ef0E/00086`); and a list whose rows include Thunder Falls Demo Derby and Thunder Falls Kart Racing (`kNZtSg_Qrsw/00132`). These agree with the Lua labels. Player names in the frames are not recorded.
- **Matchmaking, queues, activity finder (video).** No reviewed frame shows the matchmaking window, the Quick Match tracker, a launch invitation or the Activity Finder. A search for those terms returned nothing. Everything on those screens is from the client only.
- **Card lobby (video).** The "Free Realms Trading Card Game Lobby" panel with a "Casual Games Lobby" list (`4pioItLqlU8/00106`), the Treasure War panel with Quickplay, Campaign, Tournament and Collection (`JA4jxTzqdOg/00086`, a French-language recording), the loading text "Please wait while we load the Free Realms Trading Card Game / Waiting for server..." (`4h2eVoje3Eg/00070`), and a Trade Lobby panel (`UFsVcZ3X5iA/00046`). Details are on the [[Trading Card Game]] page.
- **Right-click menu (video).** A player menu offering Invite to Group, Trade and Duel (`IYzJU-RjbPE/00126`).
- **Client locale.** Lobby names include Kart Race Lobby, Demo Derby Lobby, Chess Lobby, Checkers Lobby, Soccer Lobby and per-location race and derby lobbies (Wildwood, Thunder Falls, Frost Ridge). Matchmaking text includes "Use matchmaking to find a game with other players.", "See which matchmaking queues you are in.", "Your current wait time for this queue.", "Average wait time for this queue.", "Leave this queue.", "Matchmaking failed to find a valid match." and the thirty-second invite limit. Lobby error titles: "Unable to Create / Join / Start Lobby Game" and "General Lobby Error Occurred". A kart text offers "Quick Play" (random players) or "Invite Friends". The numeric string ids used by the native code and Lua are not the hashed ids of the locale export, so most could not be matched to text.
- **Wiki.** The Interface page: the Map button shows "Matchmaking, your Calendar, and the Activity Finder" with the card game and quests. The Treasure War page says a new player starts in the card lobby and lists Quickplay, Campaign, Tournament and Collection; the Community Events page mentions players using the casual card lobby to play, trade and watch. No wiki page describes queues, lobby packets or launch rules.

### Open questions and contradictions

- **What opens a lobby.** The client only reacts to 101/6 and 101/12. Which world interaction makes the server send them, and how the lobby type is chosen, is not in the client. The Kart and Minigame pages describe a gate or portal that leads into the lobby; the client itself shows no such trigger.
- **Catalogue tables.** Roles of the fields in tables 1 to 5 are unresolved. The client has two loaders (five text files with a `%s` prefix, and 102/2) and the files are not shipped; how they relate is unknown. Lobby types 3 to 5 map to description string ids but not to named lobbies; the Lua uses 3 and 6 for the soccer-style pane while the description switch covers 1 to 5 only.
- **Create body.** The first dword, the three dwords before the strings, the two strings and the last dword are not named. The last dword is inferred to be the lobby type id from the call in `00bb1cd0`.
- **`LobbyGame` fields.** `+0x3c`, `+0x60`, `+0xfc`, `+0x108`, the info-block integers and the second string are unknown. The `+0x46` flag is read by two functions (list filter and current-game id) and is never set by client code in the lobby path, so "closed" is the reading that fits both.
- **Join/Leave/Start inbound words.** The client reads two words but uses one. The second is probably the lobby type id (it is the second word it sends), unverified.
- **Lobby errors.** The Lua `Lobby:ErrorMessage` handler is empty, so the visible path for the "Unable to …" titles is unknown. Codes 4, 5 and 9 pick distinct message ids; the code-to-text pairing is unresolved because of the hashed locale ids.
- **Matchmaking record.** Request fields `+0x10`, `+0x20`, `+0x50`, `+0x54`, `+0x58`, `+0x61` and the participant record flags are unnamed. The `JoinQueue` pseudocode puts the queue id and the two connection words at packet offsets that do not line up cleanly with the reader's request offsets (the embedded request may start at a different base); treat the slot mapping as unverified until a live test. Queue columns MatchType, MinTeams, MaxTeams, MaxGameStartDelay and Param1 to Param7 are unnamed in use.
- **Queue to game.** Nothing in the client says how a satisfied queue starts a game. 141/9 and 141/10 and 175/14 to 16 are both plausible; the Pirates queue id 41 is hard-coded in the Lua.
- **141/5 versus 141/18.** Both remove requests (by request ids, and by queue id for one player). Which the original used for a leave is unknown.
- **Group launch.** `OwnerLaunchRequest` and `OwnerMatchmakingRequest` have no mapped sender. Sub-op 6 is handled at the family entry (`009c3590`), not in `00c4e490`, so a case-6 entry is rightly absent from the per-request dispatcher; its body beyond the request id and one reason value is unread. The bit 0 of the member state byte is unused by the log. The request's "SystemHashkey" and "SystemDefinitionId" suggest launches are keyed to a system definition; their link to `MiniGameData` rows is unproven. The invite pop-up path for a lobby-less friend invite (`Gaq_AddFriendInviteToGame`, `00a0dc20`, likely, and a five-dword invite packet sender `00a97cc0`, likely) is a different family that was not resolved.
- **Activity Finder.** The Lua calls `ActivityFinderHandler:ShowFromNav()` but no such table exists in the shipped script (the name occurs only as calls), and the navigation bar adds no Activity Finder button. The wiki still lists it on the Map button. A data source `BaseClient.ActivityFinder` exists and is used by the job browser to start a job's quest.
- **Activity packet widths.** Families 147 and 167 widths and bodies are mostly unread; list nodes of 147/1 have six dwords by `00be7360` (unknown label). 167 sub-op 5's walk is unused.
- **Earlier notes.** The Minigame Framework page says the constructor for 175 sub-op 6 was not found. It is not constructor-dispatched: `009c3590` (confirmed) sends sub-op 6 straight to `009c29d0` (confirmed), and `00c4e490` has no case 6 for that reason. The Social page defers the 175 bodies to this page; they are in the tables above. The 141 constants used here are the client's own (`00a7fd30` stores 13, `00a87ad0` stores 10).
- **Card service.** The bodies of `LobbyCommand_Join` (v5) and the long-tail lobby, league, draft and tournament commands are not decoded here; see [[Trading Card Game]].

### Key functions

Confidence is the accepted label's, in the same order as the addresses ("unlabelled" = no accepted record).

| Address | Role | Confidence |
|---|---|---|
| `009471a0` | Inbound dispatcher: routes 101, 102, 141, 143, 147, 167, 175 | unknown |
| `00923210` | `BaseClient::HandleLobbyPacket`, family 101 | confirmed |
| `0091e060`, `00919b60` | Lobby catalogue 102/2 handler; five-table parser | confirmed, confirmed |
| `008e5450`, `008e5580`, `008e56c0`, `0090a070`, `008e5800` | Catalogue table parsers 1 to 5 | likely, likely, likely, unknown, likely |
| `00a27820`, `00a27790`, `00a273f0` | Catalogue request timer, sender, packet writer | speculative, likely, likely |
| `00bb0aa0`, `00bb0780`, `01049900` | `ClientLobby` ctor and dtor body; `Lobby::Ctor` | unknown, likely, confirmed |
| `00bb1890`, `00bb1920`, `00bb19b0`, `00bb1a40`, `00bb1cd0` | `SendJoinGame`, `SendLeaveGame`, `SendStartGame`, `SendLeaveLobby`, `SendCreateGame` | confirmed, confirmed, confirmed, likely, likely |
| `00bb1820`, `00bb12e0`, `00bb1260`, `00a0e8f0` | Create-body writers | likely, likely, unknown, likely |
| `00baff30`, `00bb0c30` | `Lobby_OnJoinedGame`; lobby error handler | confirmed, unknown |
| `00911130`, `0091a7e0`, `00921740`, `00912310` | `LobbyGame` record reader; game-table reader; snapshot reader; clear-all | unknown, unknown, speculative, unknown |
| `01048940`, `008fe190`, `00a22fa0` | `LobbyGame` ctor; player-count getter; row-visibility filter | confirmed, speculative, unknown |
| `00a29500`, `00a27190`, `00a27250`, `00a25870`, `00a24a60`, `00a223d0`, `00a224a0` | Data-source manager ctor; `LobbyGameList` ctor; `EncounterGameList` ctor; game-list cell reader; column-name getter; column count; encounter column names | likely, confirmed, confirmed, likely, confirmed, likely, confirmed |
| `00c91670`, `00c21610`, `00c90ec0`, `0102a4d0`, `0102a690` | Build option data sources; submit options; build create record; file loader; option lookup | confirmed, speculative, unknown, confirmed, likely |
| `00a22570`, `00a22590`, `00a225c0`, `00a225e0`, `00a22610` | Manager forwarders: join, leave game, start, leave lobby, current game id | speculative, likely, likely, likely, likely |
| `00c15ac0`, `00c15aa0`, `00c15b20`, `00c15b40`, `00c15b60`, `00c15c40` | Lua `JoinLobbyGame`, `CreateNewLobbyGame`, `LeaveLobbyGame`, `StartLobbyGame`, `LeaveLobby`, `GetLobbyGameType` | likely, then unlabelled for the other five |
| `00a241e0`, `00a242e0`, `00a243e0`, `009386f0`, `009b9550` | `Lobby:GameIsStarting`, `GameIsLaunching`, `Show` events; `LoadZone`; `HandleMiniGameGameStarted` | speculative, speculative, speculative, confirmed, confirmed |
| `00aa2980` | Matchmaking receive dispatcher | likely |
| `00a9f1d0`, `00aa1c20`, `00a9d9c0`, `00aa0460` | Queue list response ctor; parse entry; queue-row reader; queue-list reader | confirmed, likely, likely, likely |
| `00aa1140`, `00aa2460`, `00aa16f0`, `00a83ed0` | Add-match response ctor; parse entry; request-field reader; queued-group handler | confirmed, likely, likely, confirmed |
| `00a83c00`, `00a83490`, `00a835a0`, `00a84150`, `00a81040` | Server status; request removed; clear by ids; cancel for player; stats lists | unknown, confirmed, likely, confirmed, likely |
| `00c08710`, `00c08ae0`, `00c08e30`, `00c08b90`, `00c08730`, `00c08c50` | `Matchmaking` Lua module; `PopulateQueues`; `JoinQueue`; `LeaveMatchRequest`; `LeaveQueue`; `UpdateAllQueueStats` | confirmed, unknown, unknown, unknown, unknown, unlabelled |
| `00c08790`, `00c08910`, `00c08cf0`, `00a979a0`, `00a87ad0` | Add-match request ctor; body writer; send template; invitation reply; invitation response ctor | unknown, unknown, likely, confirmed, confirmed |
| `009c3590`, `009c3270`, `009c29d0`, `00c4e490`, `009c1e00`, `009c0c30` | Group-launch family entry; new request; cancel; per-request dispatcher; header reader; base ctor | confirmed, confirmed, confirmed, confirmed, likely, confirmed |
| `00c4d980`, `00c4ce40`, `00c4c580`, `00bb3660`, `00bb4f50` | Invite details; member state; launched; processor ctor; join mini-game | likely, likely, unknown, likely, likely |
| `00c4dfe0`, `00c4e1b0`, `00c4e240`, `00c4e2d0`, `00c4e3b0`, `009c19a0`, `009c1c60` | Invite response; ready; quit; invite; kick senders; accept and decline responders | likely, confirmed, likely, likely, likely, confirmed, confirmed |
| `00be7c50`, `00be7360`, `00be7b30`, `00be7660`, `00be7d80`, `00be6d30` | Activity receive (147/167); join send; manager load | likely, unknown, likely, likely, likely, confirmed |
| `00cea720`, `00ceaa40`, `00cfb090`, `00cfb1a0` | Activity list rebuilds; activity and scheduled column names | speculative, unknown, confirmed, confirmed |

<!-- NARRATIVE END -->

## Mapped functions (generated)

459 accepted functions: 244 confirmed, 95 likely, 75 speculative, 45 unknown. Network-linked (`wire`): 4.

Subsystems: `ui.matchmaking` (139), `engine.lobby` (90), `ui.lobby` (56), `net.match` (55), `game.matchmaking` (30), `unknown.lobby` (30), `ui.leaderboard` (25), `ui.player_positions` (19), `net.lobby_command` (13), `net.lobby` (2)

### `engine.lobby`

**ClientLobby** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb0b10` | ClientLobby_ScalarDeletingDtor | confirmed | Scalar deleting destructor (ClientLobby vtable slot [0]): calls the destructor body 00bb0780 on ECX and calls _free when the delete flag bit 0 is set. |
| `00bb03d0` | ClientLobby::ContainsPairKey | likely | Virtual slot [10] of ClientLobby: looks up the pair (param_2, param_3) in the 100-bucket table at this+0x1bc, bucket = (param_3 ^ param_2) % 100; walks the cha… |
| `00bb0780` | ClientLobby_DtorBody | likely | Destructor body: stores the ClientLobby vtable (01824f6c) at +0, stores the LobbyMember map vtable (01824f58) at +0x1a8 and clears that map through 00bb0550, t… |
| `00bb0b30` | ClientLobby::AddPairIfAbsent | likely | Virtual slot [8] of ClientLobby: calls virtual slot [10] (ContainsPairKey) with (param_2, param_3); when the pair is absent it inserts it through 00bb0920 and … |
| `00bb0450` | ClientLobby::FindPairNode | speculative | Virtual slot [17] of ClientLobby: the same xor-hash bucket walk as ClientLobby::ContainsPairKey over the table at this+0x1bc, returning the matching node point… |
| `00bb0aa0` | — | unknown | Constructor: stores the ClientLobby vtable (01824f6c), constructs the LobbyMember hash-list member through 00bb07f0, calls 01049900, and zeroes the dword at th… |
| `00bb0c30` | — | unknown | Virtual slot [15] of ClientLobby: formats 'Received lobby error code: %d' and appends ' (ErrorCode = %d)' into a local string, chooses a localized message id f… |

**Engine::FRTWLobbyService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013dedb0` | Engine::FRTWLobbyService::Ctor | confirmed | Fastcall constructor (106 bytes, returns this): calls 013b1c60 (base constructor), stores vtable 018e259c into *this, sets the std::string at +0x148 to empty (… |
| `013dee20` | Engine::FRTWLobbyService::~FRTWLobbyService | confirmed | Destructor body (137 bytes, fastcall, no free of this): stores vtable 018e259c, frees the string buffer at +0x134 when its capacity (+0x148) exceeds 0xf, reset… |
| `013deeb0` | Engine::FRTWLobbyService::ScalarDeletingDtor | confirmed | Vtable slot [0] of Engine::FRTWLobbyService (30 bytes, ret 4): calls the destructor body 013dee20, then if bit 0 of the argument is set calls _free on this and… |

**Engine::FRTWTournamentService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013df660` | Engine::FRTWTournamentService::Ctor | confirmed | Calls the TournamentServiceBase constructor 013ea9b0 on this, then stores the Engine::FRTWTournamentService vftable 018e264c at [this] and returns this. |
| `013df6c0` | Engine::FRTWTournamentService::Dtor | confirmed | Stores the Engine::FRTWTournamentService vftable 018e264c at [this], then calls the TournamentServiceBase destructor body 013ea860. |
| `013df890` | Engine::FRTWTournamentService::ScalarDeletingDtor | confirmed | Stores the FRTW tournament vftable 018e264c at [this], calls the TournamentServiceBase destructor body 013ea860, then frees this through _free (00d82463) only … |

**Engine::LobbyService** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013deef0` | Engine::LobbyService::SetScenarioCampaignParams | confirmed | Virtual slot [8] of Engine::LobbyService and Engine::FRTWLobbyService. Allocates a 0x4c0-byte object (00d833a6) and builds it through 012b8f80, 012ae910 and 01… |
| `013b12d0` | Engine::LobbyService::Vslot3 | speculative | Virtual slot 3 of LobbyService with three stack arguments: constructs a 0x7C-byte LobbyCommand_Join-layout local through 013b3350, sets its AccountID from 012d… |
| `013b13b0` | Engine::LobbyService::Vslot4 | speculative | Virtual slot 4 of LobbyService: takes one stack argument, passes it to 012cfde0, then calls 012cfe50 with the returned value as ECX and returns. |
| `013b13d0` | Engine::LobbyService::Vslot5 | speculative | Virtual slot 5 of LobbyService with four stack arguments: looks up param_3 through 012aab80 and 012aaeb0; when a source object is found it builds a 0x7C lobby-… |
| `013b1540` | Engine::LobbyService::Vslot7 | speculative | Virtual slot 7 of LobbyService: reads the dword at this+0x120, stores 0 there, calls 012cfde0 with the saved value and then 012cfe50 with the result as ECX; re… |
| `013b1970` | — | unknown | Virtual slot 6 of LobbyService: performs setup calls (012d46a0, 012d08f0, 014025b0, 01301ef0(param_1), 013022c0(1, param_2), 0143f180, 013b2900(value)), builds… |

**Engine::LobbyServiceBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013b1bf0` | Engine::LobbyServiceBase::DtorBody | confirmed | Stores Engine::LobbyServiceBase::vftable at *this, calls 012a4570, then 0143f490 and 01230d30; frees nothing. |
| `013b1c60` | Engine::LobbyServiceBase::LobbyServiceBase | confirmed | Calls 01230d10, stores Engine::LobbyServiceBase::vftable at *this, calls 0143f430, zeroes the dwords at this+0x11C and this+0x120, calls 012a4630 and returns t… |
| `013b1cf0` | Engine::LobbyServiceBase::~LobbyServiceBase | confirmed | Scalar deleting destructor in vtable slot 0: calls 013b1bf0 (destructor body), then calls _free(this) when bit 0 of the flag argument is set; returns this. |

**Engine::LobbyView** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0143f430` | Engine::LobbyView::LobbyView | confirmed | Constructor (fastcall, returns this): calls FUN_013afe80 on this (base constructor, in another shard), stores Engine::LobbyView::vftable into *this and returns… |
| `0143f490` | Engine::LobbyView::~LobbyView | confirmed | Destructor body with EH frame: stores Engine::LobbyView::vftable, sets the EH local, calls FUN_013aff20 (base destructor) on this and returns. No free. Called … |
| `0143f4e0` | Engine::LobbyView::~LobbyView | confirmed | Scalar deleting destructor (virtual slot 0): stores Engine::LobbyView::vftable, calls FUN_013aff20 on this, then if param_2 bit 0 is set calls _free on this; r… |

**Engine::TournamentServiceBase** (25)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013e2fd0` | Engine::TournamentServiceBase::QuerySelectDeck | confirmed | Virtual slot [26] of Engine::TournamentServiceBase only. Allocates a 0x40-byte action (00d833a6), creates the named action QuerySelectDeck (012ec110), writes t… |
| `013e3b10` | Engine::TournamentServiceBase::HandleJoinMatch | confirmed | Virtual slot [38] of the tournament vftables. Reads SelectedDeckID through 012eb250; when it is nonempty it builds a JoinMatch action (0x40-byte object, 012ec1… |
| `013e3d70` | Engine::TournamentServiceBase::HandleLeaderboardType | confirmed | Virtual slot [40] of the tournament vftables. Reads LeaderboardType through 012eb290 (default 1), raises a DataFinishedInitializing action (012ec220, hand-off … |
| `013e3e40` | Engine::TournamentServiceBase::RaiseCurrentSession | confirmed | Virtual slot [42] of the tournament vftables. Reads the session value at this+0x54; when the session branch is taken it creates a currentSession action (012ec2… |
| `013e40b0` | Engine::TournamentServiceBase::RaiseUpdatedSessionsForDay | confirmed | Virtual slot [45] of the tournament vftables with six arguments. Creates an updatedSessionsForDay action (012ec220) with the Month value, builds an xmlSessions… |
| `013e4630` | Engine::TournamentServiceBase::RaiseUpdatedReadyStatus | confirmed | Virtual slot [46] of the tournament vftables. Creates an updatedReadyStatus action (012ec220), writes the Ready value from its argument through 012ebe80, and h… |
| `013e46f0` | Engine::TournamentServiceBase::RaiseTournamentPlayerSelecte… | confirmed | Virtual slot [47] of the tournament vftables, taking a string argument. Creates a tournamentPlayerSelectedDeck action (012ec220), writes DeckID from the argume… |
| `013e47e0` | Engine::TournamentServiceBase::RaiseCurrentEventTicketCount | confirmed | Virtual slot [48] of the tournament vftables. Creates a currentEventTicketCount action (012ec220), writes the Count value from its argument through 012ebe40, a… |
| `013e3930` | Engine::TournamentServiceBase::HandleActionName | likely | Virtual slot [36] shared by the three tournament vftables. Reads the ActionName value through 012eb250, copies it into a local string, and compares it with a 4… |
| `013e3a20` | Engine::TournamentServiceBase::HandlePageName | likely | Virtual slot [37] shared by the three tournament vftables. Reads the PageName value through 012eb250 and compares it with the literal Bracket (7 bytes, 00f942a… |
| `013dfe90` | Engine::TournamentServiceBase::GetByte58 | speculative | Virtual slot [13] shared by the three tournament service vftables. Returns the byte stored at this+0x58 (a byte that virtual slot [27] at 013dfc90 writes). |
| `013e0eb0` | Engine::TournamentServiceBase::TimerExpired | speculative | Virtual slot [41] shared by the three tournament vftables. Switches on the int at argument+0: type 1 uses a 60 second limit, types 2 and 3 use 1440 seconds, ty… |
| `013dfbe0` | — | unknown | Virtual slot [15] shared by the three tournament service vftables. Calls the gateway singleton getter 012d46a0, moves its result into ECX and tail-jumps to 012… |
| `013dfc00` | — | unknown | Virtual slot [19] shared by the three tournament service vftables. Stores argument 1 at this+0x54. Calls virtual slot [6] (+0x18) with (argument 1, 0). If that… |
| `013dfc90` | — | unknown | Virtual slot [27] shared by the three tournament service vftables. Calls the gateway singleton getter 012d46a0, then compares the value returned by 012d08f0 (E… |
| `013dfcc0` | — | unknown | Virtual slot [28] shared by the three tournament service vftables. Calls virtual slot [6] (+0x18) with (this+0x54, 0). When that returns nonzero it calls 01447… |
| `013dfda0` | — | unknown | Virtual slot [35] shared by the three tournament service vftables. Loads the vftable of this and tail-calls its slot at +0xc8 (slot [50]) with the same this an… |
| `013dfdb0` | — | unknown | Virtual slot [39] shared by the three tournament service vftables. Calls virtual slot [58] (+0xe8) on this, passes its result to virtual slot [57] (+0xe4), the… |
| `013dfe50` | — | unknown | Virtual slot [5] shared by the three tournament service vftables. Calls virtual slot [6] (+0x18) of this with (this+0x54, 0) and discards the result. |
| `013dfe60` | — | unknown | Virtual slot [9] shared by the three tournament service vftables. Forwards its two arguments as (argument 1, 0, argument 2) to virtual slot [7] (+0x1c) of this… |
| `013dfea0` | — | unknown | Virtual slot [55] shared by the three tournament service vftables. Returns 0 unless 012cfde0 returns nonzero and 012cc6b0 on argument 1 returns nonzero; in tha… |
| `013e2410` | — | unknown | Virtual slot [6] of the three tournament vftables. Picks the map header at this+0x20 when the bool argument is nonzero, otherwise this+0x14; when that header i… |
| `013e32a0` | — | unknown | Virtual slot [31] shared by the three tournament vftables, taking one string argument. Assigns it to the member string at this+0x5c, copies it into a local str… |
| `013e3360` | — | unknown | Virtual slot [34] of the tournament vftables. Reads the ActionName value through 012eb250 and branches on it with 00f942a0 comparisons. Its branches create nam… |
| `013e3f80` | — | unknown | Virtual slot [43] of the tournament vftables. Under a condition Ghidra leaves unresolved (the this+0x54 session value), creates an updatedSession action (012ec… |

**Lobby** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01049900` | Lobby::Ctor | confirmed | Fastcall, ECX = this. Stores Lobby::vftable (0187f65c) at *this, zeroes this+8, +0xc, +0x10 and +4, memsets 400 bytes at this+0x14 to zero, and returns nothing… |
| `010499a0` | Lobby::CtorWithArg | confirmed | Thiscall, one stack arg (param_2). Stores Lobby::vftable (0187f65c) at *this, zeroes this+8, +0xc, +0x10 and +4, memsets 400 bytes at this+0x14, and stores par… |
| `01049a20` | Lobby::EhCleanupSetVtable | confirmed | Fastcall, ECX = this. EH-frame prologue and epilogue around a single store: mov [ecx], Lobby::vftable (0187f65c). Nothing else is done; it returns. |
| `01049ab0` | Lobby::FindThenSlot3 | confirmed | Thiscall, one stack arg (uint key). Bytes 56 8b 74 24 08 .. mul by 0x51eb851f (key % 100 computed as key - (key/100)*100) .. 8b 44 81 14 = bucket head at this+… |
| `01049af0` | Lobby::ScalarDeletingDtor | confirmed | Thiscall with a flag byte param_2. Stores Lobby::vftable (0187f65c) at *this, then if bit 0 of the flag is set calls _free on the object. Returns the object po… |
| `01049ba0` | Lobby::ReleaseNode | confirmed | Thiscall, one stack arg (node pointer). Bytes 56 8b 74 24 08 85 f6 74 19 56 83 c1 04 e8 .. = if the node is non-null: calls 008c0220 with ECX = this+4 and the … |

**LobbyGame** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008fe100` | LobbyGame::Dtor | confirmed | fastcall (ECX=this, EH frame). Stores LobbyGame::vftable at this+0, calls 008eb9c0 on this (accepted label ListDtor: destroys the embedded ApiLobbyGameOption l… |
| `010488d0` | LobbyGame::RemoveId64 | confirmed | Thiscall, two stack args (id low dword, id high dword). Walks the list at this+0x2c for the node whose two dwords equal the arguments. On a match unlinks it (h… |
| `01048940` | LobbyGame::Ctor | confirmed | Fastcall, ECX = this. Stores LobbyGame::vftable (01803f28) at *this, zeroes the dwords at +0x10..+0x1c, stores the SoeUtil::List<StrongType<uint64,...>,-1> vta… |
| `01048a00` | LobbyGame::CtorWithArgs | confirmed | Thiscall with four stack args (param_2..param_5; Ghidra). Same base initialisation as LobbyGame::Ctor (01048940): stores LobbyGame::vftable at *this, the list … |
| `01048860` | LobbyGame::RequeueId64 | likely | Thiscall, one stack arg (param_2, a record with a 64-bit id at +0x10/+0x14). If param_2 is non-null: walks the list at this+0x2c (next at node+8) and unlinks t… |
| `008fe180` | LobbyGame::ByteAt45IsZero | speculative | Eight-byte fastcall body: xor eax,eax; cmp byte ptr [ecx+0x45], al; sete al; ret. Returns 1 when the byte at this+0x45 is zero, otherwise 0 (bool). |
| `008fe190` | LobbyGame::GetDword48 | speculative | Four-byte body: mov eax, dword ptr [ecx+0x48]; ret. Returns the dword at this+0x48. |
| `008fe1a0` | LobbyGame::SetDword48 | speculative | Ten-byte thiscall body with one stack argument (ret 4): mov eax, [esp+4]; mov [ecx+0x48], eax; ret 4. Stores the argument dword at this+0x48. |
| `008fe1e0` | LobbyGame::SetByte44ReturnFalse | speculative | Seven-byte body: mov byte ptr [ecx+0x44], 1; xor al, al; ret. Sets the byte at this+0x44 to 1 and returns 0 (false). |

**LobbyService** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012cfd00` | LobbyService::LobbyService | confirmed | Constructor body: stores LobbyService::vftable at +0, calls 012ce960 with ECX=this+4 (header init of the id set), zeroes the vector words at +0x14..+0x1c, +0x2… |
| `012cfdc0` | LobbyService::ScalarDeletingDtor | confirmed | Scalar deleting destructor (LobbyService vtable slot 0): calls the destructor body 012cf910 on this, then calls _free on this when bit 0 of the flag argument i… |
| `012cd490` | LobbyService::EraseEntryById | likely | Asserts that its entry argument is non-null (LobbyService.cpp line 0x134), reads the entry's int key through getter 01301f30 (field +0x18), erases that key fro… |
| `012ce120` | LobbyService::InsertEntryId | likely | Thiscall with one stack argument: asserts the argument is non-null (LobbyService.cpp line 0x12c), reads its int key through getter 01301f30 (+0x18), and calls … |
| `012cf2b0` | LobbyService::RemoveFromList10 | likely | Thiscall with two stack arguments: forwards them with ECX = this+0x10 to 012cf1e0 (vector remove by key) and returns nothing. |
| `012cf760` | LobbyService::ClearAllLists | likely | Clears the state held by the LobbyService: erases the 0x38-element vectors at this+0x14 and this+0x24 via 012cef90, destroys the tree rooted at the header at t… |
| `012cfde0` | LobbyService::Instance | likely | Lazily creates the process-wide LobbyService: when DAT_01cbd6a0 is null it allocates 0x70 bytes, constructs the object through 012cfd00 and stores it in DAT_01… |
| `012cf120` | CopyListAt10Into | speculative | Thiscall with one stack argument (the destination): calls 012cedd0 with ECX set to the destination and the source this+0x10 (copy-construct of the 0x38-element… |
| `012cf180` | CopyListAt20Into | speculative | Thiscall with one stack argument (the destination): calls 012cedd0 with ECX set to the destination and the source this+0x20 (copy-construct of the 0x38-element… |
| `012cf2d0` | — | unknown | Thiscall with two stack arguments: forwards them with ECX = this+0x20 to 012cf1e0 (vector remove by key) and returns nothing. |
| `012cf910` | — | unknown | Destructor body: stores LobbyService::vftable at +0, clears DAT_01cbd6a0 when it equals this, walks the node list at this+8 calling vtable slot 0 (with flag 1)… |

**(no class)** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00baff30` | Lobby_OnJoinedGame | confirmed | Thiscall: stores param_2 at this+0x350 and calls 009797a0 with the event name 'Lobby:JoinedGame' and two zero arguments. |
| `0127ecb0` | InitializeLobbyMatches | likely | When param_1 is nonzero and param_2 is nonzero: assigns the 22-char string InitializeLobbyMatches, calls FUN_012ec220 with it, then FUN_01301f30, FUN_012ebe40 … |
| `0127ede0` | InitializeLobbyPlayers | likely | When param_1 is nonzero: passes the string Players to FUN_0130df10, allocates 0x40 bytes (FUN_00d833a6); if the allocation is nonnull assigns the 22-char tag I… |
| `013b03a0` | ForEachResolvedIdFiltered | speculative | Fills a 4-byte-element id vector via 0130a060(2, buffer); for each id calls 012cfde0 and 012cc710, and when 012cc710 returns nonzero tests a temporary std::str… |
| `013b1560` | GetDwordAt0x11c | speculative | Returns the dword at this+0x11C (mov eax,[ecx+0x11c]; ret); no call or store. |
| `013b1570` | SetDwordAt0x11c | speculative | Stores the stack argument into the dword at this+0x11C and returns (ret 4). |
| `013b1580` | EqualsDwordAt0x11c | speculative | Returns 1 when the stack argument equals the dword at this+0x11C, else 0 (cmp edx,[ecx+0x11c]; sete al; ret 4). |
| `013b15b0` | SetDwordAt0x120 | speculative | Stores the stack argument into the dword at this+0x120 and returns (ret 4). |
| `013b15f0` | Wrapper_012cc6b0_Of_012cfde0 | speculative | Pushes its argument, calls 012cfde0 with it, moves the result into ECX and tail-jumps to 012cc6b0; returns 012cc6b0's EAX (ret 4). Ghidra's void return is wron… |
| `013b1630` | Wrapper_012cc710_Of_012cfde0 | speculative | Pushes its argument, calls 012cfde0, moves the result into ECX and calls 012cc710; returns 012cc710's EAX (ret 4). |
| `013b1800` | TreeFindStringValue | speculative | Lower-bounds the int key taken from its stack argument in the tree at this+0x124 via 013b1720; when the result is not the sentinel at this+0x128 it assigns the… |
| `013b30b0` | AssignStringAt0x10 | speculative | Assigns its string argument (0, npos) to the std::string at this+0x10 through 00420ff1 and returns. |
| `0127f010` | — | unknown | Runs only when param_2 equals FUN_012d08f0() and FUN_013b15f0(param_1) is nonzero: allocates 0x40 bytes (FUN_00d833a6), builds the tag OnJoinedGroup and passes… |
| `013b1ac0` | — | unknown | Looks up the int key from the stack argument in the tree at this+0x124 (012a2830, then the sentinel test against this+0x128). When the key is absent it builds … |

### `game.matchmaking`

**EncounterMatchmaking::MatchmakingRequest** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a82b30` | EncounterMatchmaking::MatchmakingRequestDtor | confirmed | Destructor body for one MatchmakingRequest: stores the MatchmakingRequest vtable, calls 0070a790 with ECX set to the embedded member at this+0x64, then calls 0… |
| `00a82ba0` | EncounterMatchmaking::MatchmakingRequestScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]) of MatchmakingRequest: runs the destructor body 00a82b30 and frees the object with _free when the delete flag bit … |
| `00a83360` | EncounterMatchmaking::MatchmakingRequestCopyCtor | confirmed | Copy constructor of one matchmaking request node: stores the MatchmakingRequest vtable, copies the scalar fields at +8..+0x20 from the source, copy-constructs … |
| `00a82f10` | MatchmakingRequestDeletingDtorCopy | likely | Second deleting-destructor-shaped copy of the MatchmakingRequest teardown: an SEH-framed wrapper that calls the destructor body 00a82b30 and then frees the obj… |
| `00a83620` | MatchmakingRequestCopyCtorWrapper | likely | Thin SEH-framed wrapper that copy-constructs a MatchmakingRequest at param_1 from the source param_2 by calling 00a83360 and returns param_1; used by the list … |

**LobbyCommand_Join** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013b2be0` | — | unknown | Virtual slot 15: looks up the group (012cc6b0 of +8) and the account (012df9f0 of +4); when +0x50 is nonzero it validates the property set and account, builds … |

**Match** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014025b0` | Match::Match | confirmed | Constructor: calls 0130bd70 on the object, stores Match::vftable (018e3ef4) and Match::vftable (018e3ee0) into the object's first two words, zeroes the dword a… |
| `01402640` | Match::TypeName | confirmed | Virtual slot [11] of Match. Returns the constant C string 'Match'. |
| `01402680` | Match::DtorBody | confirmed | Non-deleting destructor body: stores both Match vtables into the object, calls 0130b6a0 on it and returns. Does not free. |
| `01402940` | Match::~Match | confirmed | Scalar deleting destructor: calls 01402680 (Match::DtorBody) on the object and frees it through _free when bit 0 of the stack byte is set; returns this. |
| `01402650` | Match::GetStaticDword | likely | Virtual slot [12] of Match. Returns the dword at global DAT_01cbe224. |
| `01402660` | Match::SetStaticDword | likely | Virtual slot [13] of Match. Stores its stack argument into DAT_01cbe224 (ret 4). |
| `014028b0` | Match::AdjustorDeleteDtor | likely | Secondary-vtable entry (Match::vftable +0x4, slot [0]): subtracts 4 from ECX (adjustor) and jumps to 01402940 (Match::~Match). |
| `014028c0` | Match::Create | likely | Allocates 0x7c bytes with operator new (00d833a6); if the allocation is non-null it calls the Match constructor 014025b0 on it and returns the EAX left after t… |

**MatchCommand_SetReady** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013b38e0` | MatchCommand_SetReady::Vslot15 | speculative | Virtual slot 15: builds a 0x14-byte event object (00d833a6, 012f9eb0), sets its type id to 0x7B (012f8c70), stores this+4 and this+8 through 012fa2e0 and the b… |

**MatchmakingQueuesDataSource** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a83c00` | — 🌐 | unknown | Reacts to a matchmaking server status change (param_2 non-zero = up): when up it first constructs a ListQueuesRequestPacket (00a7fb90), reads the client's curr… |

**MatchmakingRequestsDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a83490` | OnPlayerMatchmakingRemoved | confirmed | Walks the request list at param_1[0x98] (next link at +0xf0) and for every request whose field +0x14 equals param_2 removes it with 00a832c0 and fires the UI e… |
| `00a83ed0` | OnPlayerQueuedGroup | confirmed | Handles one queued-group notification: first drops the player's previous requests (00a83490), then appends the new request (00a83a80); if the request's handle … |
| `00a84150` | CancelMatchRequestsForPlayer 🌐 | confirmed | Walks the request list at param_1[0x98] and for every request whose field +0x14 equals param_2 builds a CancelMatchRequestPacket in a 24-byte stack object - vt… |
| `00a835a0` | RemoveMatchmakingRequestByIds | likely | Walks the request list at param_1[0x98] and removes every node whose fields +8 and +0xc equal the two id arguments param_2/param_3 using 00a832c0, then notifie… |

**(no class)** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a82c80` | EhCleanupCallMatchmakingRequestDtor | likely | SEH-framed one-shot cleanup that calls the MatchmakingRequest destructor body 00a82b30 on its this argument (saved in the SEH frame) and returns; it is referen… |
| `00a832c0` | MatchmakingRequestListRemoveNode | likely | Unlinks node param_2 from the doubly-linked request list: head is param_1[1], tail param_1[2], count param_1[3], next link node+0xf0 and previous link node+0xf… |
| `00a83600` | MatchmakingRequestListClear | likely | Repeatedly takes the head node at param_1+4 and destroys it with 00a832c0 until the head is null, emptying the request list. |
| `00a83a80` | MatchmakingRequestListAppend | likely | Allocates a node through owner vtable byte offset +8, copy-constructs the request into it with 00a83620, and appends it: node+0xf4 gets the old tail, node+0xf0… |
| `00a83b80` | BuildAndSendQueueStatsRequest 🌐 | speculative | Constructs a QueueStatsRequestPacket (callee 00a7fd30, class 141 / sub-opcode 0xd per its own constructor) in a 24-byte stack object and, when the matchmaking … |
| `014026e0` | GetField78 | speculative | Fastcall getter: returns the dword at this+0x78 (ECX). |
| `01402700` | GetField70 | speculative | Fastcall getter: returns the dword at this+0x70 (ECX). |
| `01402720` | SetFlag74ActivatesField78 | speculative | Thiscall with one byte argument: stores it at this+0x74; if it is nonzero and the dword at this+0x78 is zero it sets this+0x78 to 4. Returns void. |
| `01402740` | GetFlag74 | speculative | Fastcall getter: returns the byte at this+0x74 (ECX). |
| `012ce620` | — | unknown | Finds param_2 in the set header at this+0x30 (012cc4d0) and erases the found node through 012cdc90 when the iterator differs from this+0x34; the param_3 argume… |

### `net.lobby`

**LobbyCommand_UpdateLobbyStats** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ba120` | LobbyCommand_UpdateLobbyStats::Write | confirmed | Calls vftable slot 2 (header write), then writes trace text (no-op), calls 013b2960, writes the three containers at +8, +0x14, +0x20 through 01351620, writes t… |
| `014ba470` | LobbyCommand_UpdateLobbyStats::Read | confirmed | Calls vftable slot 4 (header read); then reads three containers at +8, +0x14, +0x20 through 0135d6e0, the int at +0x2c through 013c24d0 only when the register … |

### `net.lobby_command`

**LobbyCommand_InfoEnumMessage** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0142f7b0` | LobbyCommand_InfoEnumMessage::~LobbyCommand_InfoEnumMessage | confirmed | Scalar deleting destructor: calls the destructor body FUN_0142f260 on this, then if bit 0 of the flag argument is set it calls _free on this; returns this. |
| `0142fbb0` | LobbyCommand_InfoEnumMessage::AppendName | confirmed | Loads its stack argument into ECX and calls FUN_01241650 with the literal text InfoEnumMessage Lobby Command and length 0x1d; returns with ret 4. |
| `0142f4e0` | LobbyCommand_InfoEnumMessage::Clone | likely | Allocates 0x14 bytes with FUN_00d833a6; if that succeeds it constructs the object with FUN_0142f1a0 and keeps its result, otherwise the result is 0. It then ca… |
| `0142f550` | LobbyCommand_InfoEnumMessage::CloneInto | likely | Dynamic-casts the argument to LobbyCommand_InfoEnumMessage with __RTDynamicCast (FUN_00d8d382, source RTTI NetworkCommand). If the cast fails it calls FUN_012f… |
| `0142f730` | LobbyCommand_InfoEnumMessage::Create | likely | Allocates 0x14 bytes with FUN_00d833a6; if the allocation is non-null it calls FUN_0142f1a0 with the new object as this and returns that result, otherwise it r… |
| `0142f7d0` | LobbyCommand_InfoEnumMessage::ReadFields | likely | Calls virtual slot 4 (+0x10) of this with the stream and its own stack address (result ignored). Then FUN_013b2910 with the stream as a gate; if nonzero, FUN_0… |
| `0142f660` | LobbyCommand_InfoEnumMessage::AppendDebugText | speculative | Calls virtual slot 2 (+0x8) of this with the stream argument, appends the literal Starting LobbyCommand_InfoEnumMessage with a newline, calls FUN_013b2960 on t… |
| `0142f400` | — | unknown | Returns 1 on every path. If the dword at +0xac of the object returned by FUN_012d46a0(this) is nonzero (read via FUN_012d0920), it allocates 0x14 bytes with FU… |
| `0142fa20` | — | unknown | Writes the literal Info message Enum display Lobby Command via FUN_01241650 after calling FUN_013b2af0 with the argument. It writes Message and the decimal thi… |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012caf00` | SendLobbyCommand_ChangeStatus 🌐 | likely | Stdcall with one stack argument (the status value). Gets ClientApplication (012d46a0). When the object at its +4 is non-null it constructs a LobbyCommand_Chang… |
| `0142f840` | SetField0cString | likely | If this+0xc is null, allocates 0xc bytes with FUN_00d833a6 and constructs a ValueData with FUN_012fa910 and stores it at this+0xc. It measures the NUL-terminat… |
| `0142f930` | SetField10String | likely | If this+0x10 is null, allocates 0xc bytes with FUN_00d833a6 and constructs a ValueData with FUN_012fa910 and stores it at this+0x10. It measures the NUL-termin… |
| `0142f790` | RegisterCreator_Id0x128 | speculative | Calls FUN_013ce130 with 0x128 and the address of FUN_0142f730, then calls FUN_013ce230 with the value returned by the first call in ECX. |

### `net.match`

**MatchCommand_SetBulkMatchState** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b3d60` | MatchCommand_SetBulkMatchState::WriteBody | confirmed | Virtual slot 10 (+0x28). Calls slot 2 (+0x8) with the argument (header write), writes the text Starting MatchCommand_SetBulkMatchState through 013c26e0 (no-op)… |
| `014b3ea0` | MatchCommand_SetBulkMatchState::TypeName | confirmed | Virtual slot 11 (+0x2c). Returns the constant C string MatchCommand_SetBulkMatchState (mov eax, literal; ret). Reads no fields. |
| `014b3ee0` | MatchCommand_SetBulkMatchState::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Calls 014b3df0 (the SetBulk body destructor), then calls _free(this) only when bit 0 of the byte argument is set. R… |
| `014b3f00` | MatchCommand_SetBulkMatchState::Dump | confirmed | Virtual slot 19 (+0x4c). One stack argument (the output string). Appends Set Bulk Match State Match Command plus newline (35 bytes) via 01241650, then calls 01… |
| `014b3f30` | MatchCommand_SetBulkMatchState::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Loads the stack argument (std::string pointer) into ECX, pushes length 0x22 and the literal Set Bulk Match State Match Command, calls … |
| `014b3f50` | MatchCommand_SetBulkMatchState::ctor | confirmed | Constructor (this in ECX). Calls 013b2320 (the MatchCommand base constructor), stores MatchCommand_SetBulkMatchState::vftable, zeroes the dwords at +0x10, +0x1… |
| `014b3eb0` | MatchCommand_SetBulkMatchState::ReadGlobal01cbe710 | likely | Virtual slot 12 (+0x30). Returns the 32-bit global at 01cbe710 (mov eax,[01cbe710]; ret). Reads no object fields. |
| `014b3ec0` | MatchCommand_SetBulkMatchState::WriteGlobal01cbe710 | likely | Virtual slot 13 (+0x34). One stack argument (ret 4): stores it into the global at 01cbe710. |
| `014b3bb0` | MatchCommand_SetBulkMatchState::QueueEvent | speculative | Virtual slot 15 (+0x3c). Walks two lists in step (dword items from the list at +0x10..+0x14 and from the list at +0x20..+0x24). For each pair it allocates 0x14… |
| `014b3df0` | — | unknown | Stores MatchCommand_SetBulkMatchState::vftable into *this, frees the buffer at +0x20 through _free when non-null and zeroes +0x20, +0x24 and +0x28, frees the b… |
| `014b3fc0` | — | unknown | Virtual slot 17 (+0x44). Calls 00d833a6 (operator new) with size 0x2c. If the block is non-null it constructs a SetBulk command in it with 014b3f50. Then, unco… |

**MatchCommand_SetDeckArchetype** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b2d90` | MatchCommand_SetDeckArchetype::ctor | confirmed | Constructor (this in ECX). Calls 013b2320 (the MatchCommand base constructor, which references MatchCommand::vftable), stores MatchCommand_SetDeckArchetype::vf… |
| `014b2e10` | MatchCommand_SetDeckArchetype::TypeName | confirmed | Virtual slot 11 (+0x2c). Returns the constant C string MatchCommand_SetDeckArchetype (mov eax, literal; ret). Reads no fields. |
| `014b2ec0` | MatchCommand_SetDeckArchetype::Clone | confirmed | Virtual slot 17 (+0x44). Calls 00d833a6 (operator new) with size 0x10. If the block is non-null it constructs a SetDeckArchetype command in it with 014b2d90. T… |
| `014b2f30` | MatchCommand_SetDeckArchetype::CopyFieldsTo | confirmed | Virtual slot 18 (+0x48). Argument is a NetworkCommand pointer; dynamic_cast to SetDeckArchetype (00d8d382). On failure reports via 012f5a60 (tag clone, MatchCo… |
| `014b2f80` | MatchCommand_SetDeckArchetype::ReadBody | confirmed | Virtual slot 9 (+0x24). Calls slot 4 (+0x10) on this with the argument and &param_2 (header read; tested as bool), then 013b2530 with the argument (the MatchCo… |
| `014b2fd0` | MatchCommand_SetDeckArchetype::WriteBody | confirmed | Virtual slot 10 (+0x28). Calls slot 2 (+0x8) with the argument (header write), writes the text Starting MatchCommand_SetDeckArchetype through 013c26e0 (no-op),… |
| `014b30c0` | MatchCommand_SetDeckArchetype::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores MatchCommand_SetDeckArchetype::vftable into *this, calls 013b23e0 (MatchCommand-level teardown), then calls … |
| `014b3120` | MatchCommand_SetDeckArchetype::Dump | confirmed | Virtual slot 19 (+0x4c). Appends Set Deck Archetype Match Command plus newline (33 bytes) via 01241650, calls 013b26c0 with the output string (the MatchCommand… |
| `014b3220` | MatchCommand_SetDeckArchetype::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Loads the stack argument (std::string pointer) into ECX, pushes length 0x20 and the literal Set Deck Archetype Match Command, calls 01… |
| `014b2e20` | MatchCommand_SetDeckArchetype::ReadGlobal01cbe704 | likely | Virtual slot 12 (+0x30). Returns the 32-bit global at 01cbe704 (mov eax,[01cbe704]; ret). Reads no object fields. |
| `014b2e30` | MatchCommand_SetDeckArchetype::WriteGlobal01cbe704 | likely | Virtual slot 13 (+0x34). One stack argument (ret 4): stores it into the global at 01cbe704. |
| `014b2e50` | MatchCommand_SetDeckArchetype::DtorBody | likely | Stores MatchCommand_SetDeckArchetype::vftable into *this, then calls 013b23e0 (MatchCommand-level teardown) with the EH cookie. Not a vtable slot. |
| `014b3040` | MatchCommand_SetDeckArchetype::New | likely | No arguments. Calls 00d833a6 (operator new) with size 0x10. If the block is non-null it passes it to 014b2d90 (the SetDeckArchetype constructor) and returns th… |

**MatchCommand_SetMatchState** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b36c0` | MatchCommand_SetMatchState::ctor | confirmed | Constructor (this in ECX). Calls 013b2320 (the MatchCommand base constructor, which references MatchCommand::vftable), stores MatchCommand_SetMatchState::vftab… |
| `014b3740` | MatchCommand_SetMatchState::TypeName | confirmed | Virtual slot 11 (+0x2c). Returns the constant C string MatchCommand_SetMatchState (mov eax, literal; ret). Reads no fields. |
| `014b38b0` | MatchCommand_SetMatchState::Clone | confirmed | Virtual slot 17 (+0x44). Calls 00d833a6 (operator new) with size 0x10. If the block is non-null it constructs a SetMatchState command in it with 014b36c0. Then… |
| `014b3920` | MatchCommand_SetMatchState::CopyFieldsTo | confirmed | Virtual slot 18 (+0x48). Argument is a NetworkCommand pointer; dynamic_cast to SetMatchState (00d8d382). On failure reports via 012f5a60 (tag clone, MatchComma… |
| `014b3970` | MatchCommand_SetMatchState::ReadBody | confirmed | Virtual slot 9 (+0x24). Calls slot 4 (+0x10) on this with the argument and &param_2 without testing its result, then calls 013b2530 with the argument (MatchCom… |
| `014b39c0` | MatchCommand_SetMatchState::WriteBody | confirmed | Virtual slot 10 (+0x28). Calls slot 2 (+0x8) with the argument (header write), writes the text Starting MatchCommand_SetMatchState through 013c26e0 (no-op), ca… |
| `014b3aa0` | MatchCommand_SetMatchState::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores MatchCommand_SetMatchState::vftable into *this, calls 013b23e0 (MatchCommand-level teardown), then calls _fr… |
| `014b3b00` | MatchCommand_SetMatchState::Dump | confirmed | Virtual slot 19 (+0x4c). Appends Set Match State Match Command plus newline (30 bytes) via 01241650, calls 013b26c0 with the output string (the MatchCommand-le… |
| `014b3b90` | MatchCommand_SetMatchState::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Loads the stack argument (std::string pointer) into ECX, pushes length 0x1d and the literal Set Match State Match Command, calls 01241… |
| `014b3750` | MatchCommand_SetMatchState::ReadGlobal01cbe70c | likely | Virtual slot 12 (+0x30). Returns the 32-bit global at 01cbe70c (mov eax,[01cbe70c]; ret). Reads no object fields. |
| `014b3760` | MatchCommand_SetMatchState::WriteGlobal01cbe70c | likely | Virtual slot 13 (+0x34). One stack argument (ret 4): stores it into the global at 01cbe70c. |
| `014b3a20` | MatchCommand_SetMatchState::New | likely | No arguments. Calls 00d833a6 (operator new) with size 0x10. If the block is non-null it passes it to 014b36c0 (the SetMatchState constructor) and returns that … |
| `014b37f0` | MatchCommand_SetMatchState::QueueEvent | speculative | Virtual slot 15 (+0x3c). Allocates 0x14 bytes with 00d833a6; if non-null builds an object with 012f9eb0 (otherwise 0). Calls 012f8c70 with 0x7e, then 012fa2e0 … |
| `014b3780` | — | unknown | Stores MatchCommand_SetMatchState::vftable into *this, then calls 013b23e0 (MatchCommand-level teardown) with the EH cookie. Not a vtable slot. |

**MatchCommand_SetTimer** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b3240` | MatchCommand_SetTimer::ctor | confirmed | Constructor (this in ECX). Calls 013b2320 (the MatchCommand base constructor, which references MatchCommand::vftable), stores MatchCommand_SetTimer::vftable, z… |
| `014b32c0` | MatchCommand_SetTimer::TypeName | confirmed | Virtual slot 11 (+0x2c). Returns the constant C string MatchCommand_SetTimer (mov eax, literal; ret). Reads no fields. |
| `014b3420` | MatchCommand_SetTimer::Clone | confirmed | Virtual slot 17 (+0x44). Calls 00d833a6 (operator new) with size 0x10. If the block is non-null it constructs a SetTimer command in it with 014b3240. Then, unc… |
| `014b3490` | MatchCommand_SetTimer::CopyFieldsTo | confirmed | Virtual slot 18 (+0x48). Argument is a NetworkCommand pointer; dynamic_cast to SetTimer (00d8d382). On failure reports via 012f5a60 (tag clone, MatchCommandSet… |
| `014b34e0` | MatchCommand_SetTimer::ReadBody | confirmed | Virtual slot 9 (+0x24). Calls slot 4 (+0x10) on this with the argument and &param_2 without testing its result, then calls 013b2530 with the argument (the Matc… |
| `014b3530` | MatchCommand_SetTimer::WriteBody | confirmed | Virtual slot 10 (+0x28). Calls slot 2 (+0x8) with the argument (header write), writes the text Starting MatchCommand_SetTimer through 013c26e0 (no-op), calls 0… |
| `014b3610` | MatchCommand_SetTimer::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores MatchCommand_SetTimer::vftable into *this, calls 013b23e0 (MatchCommand-level teardown), then calls _free(th… |
| `014b3670` | MatchCommand_SetTimer::Dump | confirmed | Virtual slot 19 (+0x4c). One stack argument (the output string). Appends Bid Placed Match Command plus newline (25 bytes) via 01241650, then calls 013b26c0 wit… |
| `014b36a0` | MatchCommand_SetTimer::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Loads the stack argument (std::string pointer) into ECX, pushes length 0x18 and the literal Bid Placed Match Command, calls 01241650 (… |
| `014b32d0` | MatchCommand_SetTimer::ReadGlobal01cbe708 | likely | Virtual slot 12 (+0x30). Returns the 32-bit global at 01cbe708 (mov eax,[01cbe708]; ret). Reads no object fields. |
| `014b32e0` | MatchCommand_SetTimer::WriteGlobal01cbe708 | likely | Virtual slot 13 (+0x34). One stack argument (ret 4): stores it into the global at 01cbe708. |
| `014b3590` | MatchCommand_SetTimer::New | likely | No arguments. Calls 00d833a6 (operator new) with size 0x10. If the block is non-null it passes it to 014b3240 (the SetTimer constructor) and returns that objec… |
| `014b3370` | MatchCommand_SetTimer::QueueEvent | speculative | Virtual slot 15 (+0x3c). Allocates 0x14 bytes with 00d833a6; if non-null builds an object with 012f9eb0 (otherwise 0). Calls 012f8c70 with 0x49, then 012fa2e0 … |
| `014b3300` | — | unknown | Stores MatchCommand_SetTimer::vftable into *this, then calls 013b23e0 (MatchCommand-level teardown) with the EH cookie. Not a vtable slot; the deleting destruc… |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b30a0` | RegisterFactoryKey1d6 | likely | No arguments. Pushes the address 014b3040 (the SetDeckArchetype New factory) and key 0x1d6, calls 013ce130, moves its EAX into ECX and calls 013ce230. Same sha… |
| `014b35f0` | RegisterFactoryKey127 | likely | No arguments. Pushes the address 014b3590 (the SetTimer New factory) and key 0x127, calls 013ce130, moves its EAX into ECX and calls 013ce230. Same shape as 01… |
| `014b3a80` | RegisterFactoryKey7c | likely | No arguments. Pushes the address 014b3a20 (the SetMatchState New factory) and key 0x7c, calls 013ce130, moves its EAX into ECX and calls 013ce230. Same shape a… |

### `ui.leaderboard`

**ClientLeaderBoardManager** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b85e80` | ClientLeaderBoardManager::RequestLeaderBoard | confirmed | Builds the leaderboard request URL: with a non-null URL string it initializes an IString (FUN_006f9e60), appends the passed string (FUN_00402770), logs it as '… |
| `00b865b0` | ClientLeaderBoardManager::ClientLeaderBoardManager | confirmed | Constructor: installs TinyHttp::HttpManagerHandler::vftable then ClientLeaderBoardManager::vftable, stores param_2 at +4, zeroes +8/+0xc/+0x10, initializes the… |
| `00b86840` | ClientLeaderBoardManager::ScalarDeletingDtor | confirmed | Vtable slot 0 of ClientLeaderBoardManager: runs the destructor body 00b86670 and frees the manager object through the _free thunk when bit 0 of param_2 is set. |
| `00b86880` | ClientLeaderBoardManager::OnGetResult | confirmed | HTTP result callback (vtable slot 1): clears the embedded response fields at +0x14/+0x18/+0x1c and the entry array through 00b86200, and when a body buffer is … |
| `00b85d50` | ClientLeaderBoardManager::LoadLeaderBoardUrl | likely | Reads the WebResources/GameLeaderBoardUrl configuration string into a 512-byte buffer through FUN_006febe0 and assigns it via SoeUtil::IStringAssign, then crea… |
| `00b86670` | — | unknown | Destructor body: reinstalls ClientLeaderBoardManager::vftable, frees the heap object at +8 (FUN_00d32f60 then _free), releases and clears the ref-counted data-… |

**LeaderBoardDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cdeef0` | LeaderBoardDataSource::DestructorBody | confirmed | Restores three LeaderBoardDataSource vtables, conditionally releases the stored reference at +0x258, then destroys the embedded GUI table. |
| `00cdf050` | LeaderBoardDataSource::Constructor | confirmed | Initializes the BaseClient.LeaderBoard path, installs three vtables, stores its context, allocates a 600-byte entries object, and installs that object through … |
| `00cdf110` | LeaderBoardDataSource::ScalarDeletingDestructor | confirmed | Calls the destructor body, frees this only when flag bit 0 is set, and otherwise returns this. |

**LeaderBoardEntryDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cdee80` | LeaderBoardEntryDataSource::DestructorBody | confirmed | Restores three LeaderBoardEntryDataSource vtables and destroys the embedded GUI user-data table at its adjusted subobject. |
| `00cdefb0` | LeaderBoardEntryDataSource::Constructor | confirmed | Initializes the leaderboard entries path, installs three vtable pointers, and stores the supplied context at +0x254. |
| `00cdf030` | LeaderBoardEntryDataSource::ScalarDeletingDestructor | confirmed | Calls the destructor body, frees this only when flag bit 0 is set, and otherwise returns this. |

**LeaderBoardResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b86720` | LeaderBoardResponse_ParseXml | confirmed | Parses a leaderboard XML result document into a response object: reads the 'totalPages' attribute into this+0x1c, constructs a temporary entry (00b85ce0), then… |
| `00b864d0` | LeaderBoardResponse_Ctor | likely | Constructor of the response wrapper that owns the entry array at +0xc: puts the SoeUtil::Array<LeaderBoardResponse::EntryLine,0,1> vtable at this+0xc, zeroes t… |
| `00b86540` | LeaderBoardResponse_Destroy | likely | Destructor body of the response wrapper: resets the entry-array vtable at this+0xc, destroys the elements counted at this+0x14 through 00b86200, frees the elem… |

**LeaderBoardResponse::EntryLine** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b86390` | LeaderBoardEntry_ParseAttributes | confirmed | Populates one entry record from a DOM reader: reads the 64-bit 'guid' attribute into the entry key at +0 (00b85b20), then the named ints 'rank' into +0x60, 'id… |
| `00b856f0` | LeaderBoardEntry_Id | likely | Bounds-checked element getter on a 0x70-byte-stride array: returns 0 when index param_2 is negative or >= the count at this+0x14, otherwise returns the dword a… |
| `00b85710` | LeaderBoardEntry_Type | likely | Bounds-checked element getter on a 0x70-byte-stride array: returns 0 when index param_2 is negative or >= this+0x14, otherwise the dword at element*0x70 + 0x64… |
| `00b85730` | LeaderBoardEntry_Score | likely | Bounds-checked element getter on a 0x70-byte-stride array: returns 0 when index param_2 is out of range for the count at this+0x14, otherwise the dword at elem… |
| `00b85750` | LeaderBoardEntry_Rank | likely | Bounds-checked element getter on a 0x70-byte-stride array: returns 0 when index param_2 is out of range for the count at this+0x14, otherwise the dword at elem… |
| `00b85770` | LeaderBoardEntry_Guid | likely | Bounds-checked 8-byte element getter: when index param_3 is in range it copies the two dwords at element*0x70 + 0 and +4 from the base at this+0x10 into the ou… |
| `00b857c0` | LeaderBoardEntry_NameField | likely | Bounds-checked element getter returning the dword at element*0x70 + 0xc (base this+0x10, count this+0x14), or 0 when the index is out of range. |
| `00b85ce0` | LeaderBoardEntryLine_Ctor | likely | Constructor for a 0x70-byte entry record: runs StringFixed64Ctor (FUN_00402a10) on this+8 (the entry name string member), zeroes the dwords at +0x5c, +0x60, +0… |
| `00b86040` | LeaderBoardEntry_CopyCtor | likely | Copy-constructs a 0x70-byte entry into param_1 from param_2: copies the 8-byte key at +0/+4, copies the fixed-string member at +8 through FUN_00964840, and cop… |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090d5c0` | FormatLeaderboardTimeEntry | speculative | Parses an input record, formats a time as hours/minutes/seconds and fractional digits, appends a Personal Best marker for record type 4, and displays the resul… |

### `ui.lobby`

**Engine::LobbyViewBase** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013afe80` | Engine::LobbyViewBase::LobbyViewBase | confirmed | Stores Engine::LobbyViewBase::vftable at *this, then calls 014384b0, 013c1a50, 0143d160, 014384b0, 013c1a50 and 0143d050 in that order; returns this. |
| `013aff20` | Engine::LobbyViewBase::DtorBody | confirmed | Stores Engine::LobbyViewBase::vftable at *this, then calls 0143d0b0, 013c1a70, 014381c0, 0143d1c0, 013c1a70 and 014381c0 in that order; frees nothing. |
| `013b0020` | Engine::LobbyViewBase::~LobbyViewBase | confirmed | Scalar deleting destructor: calls 013aff20 (destructor body), then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `013b0170` | Engine::LobbyViewBase::Vslot3 | speculative | Virtual slot 3 of LobbyViewBase: when param_2 == 0x1075 and 0143ba20 is nonzero, builds the string Z_ (or A_ when 012d08f0 equals 013a3480), appends the text f… |
| `013b0d80` | Engine::LobbyViewBase::Vslot2 | speculative | Virtual slot 2 of LobbyViewBase: allocates 0x18 bytes (00d833a6), constructs an object with 0143b950 when allocation succeeds, calls 0143ba10 with this+4 (or 0… |
| `013b0e60` | Engine::LobbyViewBase::Vslot1 | speculative | Virtual slot 1 of LobbyViewBase: allocates 0x18 bytes (00d833a6), constructs an object with 0143b950, passes it to 0143ba10(param_2), then calls this->vtable s… |

**LobbyDataSourceManager::EncounterListDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a224a0` | EncounterListDataSource::ColumnName | confirmed | Returns the literal column-name string for the index argument when the encounter chain (this+0x224 -> +0x44 -> +0x6f4) is non-null: 0 "GameId", 1 "NumberOfPlay… |
| `00a23940` | EncounterListDataSource::RefreshRows | confirmed | Clears the UI list through virtual slot 0xc of the object at this+0x34 and, when the encounter object (this+0x254 -> +0x44 -> +0x6f4) exists, walks the list at… |
| `00a25a40` | LobbyDataSourceManager::EncounterListDataSource::GetItemCol… | confirmed | Returns 1 on its paths. For a found encounter row, writes the requested column value through param_4: column 0 writes the row value, column 1 sums linked-entry… |
| `00a26fd0` | LobbyDataSourceManager::EncounterListDataSource::Dtor | confirmed | Restores data-source vtables, releases its shared string and destroys embedded GUI user data. |
| `00a27250` | LobbyDataSourceManager::EncounterListDataSource::Ctor | confirmed | Registers BaseClient.EncounterGameList, initializes vtables, stores owner and initializes display string storage. |
| `00a272f0` | LobbyDataSourceManager::EncounterListDataSource::ScalarDele… | confirmed | Calls data-source destructor and frees object when deletion flag bit zero is set. |
| `00a22560` | EncounterListDataSource::ClearFlagAndForward | likely | Stores 0 in the byte at this+0x258 and tail-calls 010caf20, forwarding this; byte-for-byte the same shape as the game-list variant at 00a22470. |
| `00a22520` | EncounterListDataSource::ValueOrZero | speculative | Identical in shape to the game-list variant: if this+0x228 is clear it calls virtual slot 0x38 on the object at this-0x30, then returns 0 when the encounter ch… |

**LobbyDataSourceManager::GameConfigOptionDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c91120` | GameConfigOptionDataSource::ColumnName | confirmed | Returns the literal column names OptionValueId, IsDefaultValue, and OptionLabelStringId for indices one through three; index zero reads a localized option labe… |
| `00c911c0` | GameConfigOptionDataSource::CountValues | confirmed | Builds a temporary integer list from the configured option mapping and returns the count stored in its resulting list object. |
| `00c91310` | GameConfigOptionDataSource::GetColumnValue | confirmed | Returns the localized option label, numeric option value, default-value flag, or formatted option string according to the requested column index. |
| `00c914b0` | GameConfigOptionDataSource::Dtor | confirmed | Restores class and string vtables, releases two owned strings, and destroys the embedded GUI user-data table. |
| `00c91580` | GameConfigOptionDataSource::Ctor | confirmed | Initializes the base GUI table, class vtables, two empty owned strings, option data-source fields, and the supplied option configuration values. |
| `00c91650` | GameConfigOptionDataSource::ScalarDeletingDtor | confirmed | Calls the class cleanup routine and frees the object when the deleting flag's low bit is set; otherwise returns the original pointer. |

**LobbyDataSourceManager::GameListDataSource** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a24a60` | LobbyDataSourceManager::GameListDataSource::GetColumnValue | confirmed | Selects a config identifier by column, resolves later indices, assigns display text and returns its string buffer. |
| `00a26f30` | LobbyDataSourceManager::GameListDataSource::Dtor | confirmed | Restores data-source vtables, releases its shared string and destroys embedded GUI user data. |
| `00a27190` | LobbyDataSourceManager::GameListDataSource::Ctor | confirmed | Registers BaseClient.LobbyGameList, initializes vtables, stores owner and initializes display string storage. |
| `00a27230` | LobbyDataSourceManager::GameListDataSource::ScalarDeletingD… | confirmed | Calls the data-source destructor and frees object when deletion flag bit zero is set. |
| `00a223d0` | GameListDataSource::CountPlus3 | likely | Reads the game object through this+0x224 -> +0x44 -> +0x6f4 and returns 0 when that chain is empty; otherwise calls 0102a5a0 with the game's state field at +0x… |
| `00a22470` | GameListDataSource::ClearFlagAndForward | likely | Stores 0 in the byte at this+0x258 and then tail-calls 010caf20 (a 12-byte function), passing the incoming this through unchanged. |
| `00a25870` | LobbyDataSourceManager::GameListDataSource::GetItemColumn | likely | Returns 1 on its paths. For a valid row and column, writes the selected value or formatted string through param_4; columns 0, 1 and 2 take distinct paths, whil… |
| `00a22400` | GameListDataSource::ValueOrZero | speculative | When the byte at this+0x228 is clear it calls virtual slot 0x38 on the object at this-0x30; then, if the game chain (this+0x224 -> +0x44 -> +0x6f4) is non-null… |
| `00a22fa0` | — | unknown | Clears the UI list through its virtual slot +0xc, then, when the referenced game object exists, walks the list at game+8 using each node?s +0x14 link. For each… |

**UI::LobbyScreen** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f7a40` | UI::LobbyScreen::LobbyScreen | confirmed | Constructor (fastcall, ECX = this): calls UI::LobbyScreenBase::Ctor 013fe3a0, then stores UI::LobbyScreen::vftable at *this. |
| `012f7aa0` | UI::LobbyScreen::DtorBody | confirmed | Non-deleting destructor body: stores UI::LobbyScreen::vftable at *this, then calls UI::LobbyScreenBase::Dtor 013fe400. |
| `012f7af0` | UI::LobbyScreen::~LobbyScreen | confirmed | Scalar deleting destructor (vtable slot [0]): stores UI::LobbyScreen::vftable, calls the destructor body 013fe400, then calls _free on this when bit 0 of the f… |
| `013fe4c0` | UI::LobbyScreen::Slot1 | likely | Virtual slot 1 of UI::LobbyScreen::vftable and UI::LobbyScreenBase::vftable (shared body). Creates two 0x40-byte callback objects through 00d833a6 and binds th… |
| `013fe650` | UI::LobbyScreen::Slot3 | likely | Virtual slot 3 of UI::LobbyScreen::vftable and UI::LobbyScreenBase::vftable (shared body). Builds the 'JoinLobby' and 'QuerySelectDeck' callback objects (0x40-… |

**UI::LobbyScreenBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013fe3a0` | UI::LobbyScreenBase::Ctor | confirmed | Fastcall constructor: calls base constructor 012f8420, then stores UI::LobbyScreenBase::vftable at this. |
| `013fe400` | UI::LobbyScreenBase::Dtor | confirmed | Fastcall destructor body: stores UI::LobbyScreenBase::vftable, then calls base destructor body 012f8480. |
| `013fe460` | UI::LobbyScreenBase::ScalarDeletingDtor | confirmed | Virtual slot 0 of UI::LobbyScreenBase::vftable: stores the vtable, calls base destructor body 012f8480, frees the object when the flag argument has bit 0 set, … |

**UI::TournamentScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f7d70` | UI::TournamentScreen::TournamentScreen | confirmed | Constructor (fastcall, ECX = this): calls UI::TournamentScreenBase::Ctor 013fea30, then stores UI::TournamentScreen::vftable at *this. |
| `012f7dd0` | UI::TournamentScreen::DtorBody | confirmed | Non-deleting destructor body: stores UI::TournamentScreen::vftable at *this, then calls UI::TournamentScreenBase::Dtor 013fea90. |
| `012f7e20` | UI::TournamentScreen::~TournamentScreen | confirmed | Scalar deleting destructor (vtable slot [0]): stores UI::TournamentScreen::vftable, calls the destructor body 013fea90, then calls _free on this when bit 0 of … |

**(no class)** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c91670` | PopulateLobbyGameOptions | confirmed | Builds up to ten BaseClient.LobbyGameOption%d data sources, appends each valid option object to the dynamic array, and stops when allocation or construction fa… |
| `00a29370` | LobbyDataSourceManager::Dtor | likely | Unregisters both lobby list strings, destroys encounter map and child services, then releases base state. |
| `00a29500` | LobbyDataSourceManager::Ctor | likely | Initializes manager state, game and encounter list data sources, encounter hash list and timestamp fields. |
| `00c15ac0` | JoinLobbyGame | likely | Lua C function bound as JoinLobbyGame: when at least one argument is present it reads argument 1 (FUN_00e3a770 + FUN_00d83c90) and calls FUN_00a22570(serverId)… |
| `0102a040` | CollectDisplayOrderPairs10 | likely | Walks the 10-bucket table at this+0x60 (bucket param_2 % 10, chain entries whose key at +0x10 and first dword equal param_2) and, for each matching entry, inse… |
| `0102a160` | CollectDisplayOrderValuePairs10 | likely | Same walk as 0102a040 over the 10-bucket table at this+0x98 (bucket param_2 % 10, entries keyed at +0x10 and first dword), inserting pairs into a stack-local L… |
| `0102a5a0` | ResolveKeyThenCollectPairs | likely | Finds the id for key param_2: scans the list at this+0x134 for a node whose key at +0x14 equals param_2 and takes its +0x10 value; if no node matches, that val… |
| `0102a690` | LookupCollectedPairValue | likely | Resolves key param_2 through the list at this+0x134 to its +0x10 value, collects the display-order pairs for that value with 0102a040 into a stack-local List<i… |
| `013b0640` | WritePlayersList | likely | Calls 013b0530 (ForEachResolvedId) with param_3 and a local, fetches an id vector through the vtable slot 1 of the object at this+0x8c, writes a Players key (0… |
| `013b0810` | WriteMatchesList | likely | Builds the id list through the vtable call on the object at param_1+4, writes a Matches key (0130df10), then for each nonzero id calls 013af410; then clears th… |
| `013b09f0` | WriteLobbyGroup | likely | Writes a Lobby key (0130df10) with an ID entry (value from 01301f30 through 0130e7b0), then calls 013b0640(param_1, param_2) for Players and 013b0810(param_1, … |
| `00a27820` | MaybeRequestLobbyDefinitionsAfterDelay | speculative | Checks elapsed time and, after a threshold, invokes the definition request or sets a one-shot flag based on owner state. |
| `013b0b10` | SubmitLobbyGroupPayload | speculative | Builds a 180-byte payload with 013b09f0(payload, param_2), then calls 0130e200, 0130d600(param_1) and 0130d9b0; param_1 is the id passed to 0130d600. |
| `013b0be0` | SubmitPlayersPayload | speculative | Builds a 180-byte payload with 013b0640(payload, param_2), then calls 0130e200, 0130d600(param_1) and 0130d9b0. |
| `013b0cb0` | SubmitMatchesPayload | speculative | Builds a 180-byte payload with 013b0810(payload, param_2), then calls 0130e200, 0130d600(param_1) and 0130d9b0. |
| `0102a290` | — | unknown | Stores param_1 into *param_2, calls 0102a040 with param_1 and the address of a stack-local List<int,-1>, then walks the temporary list it was given: each id is… |

### `ui.matchmaking`

**Engine::FRTWEngineLobbyDeckSelected** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01279e90` | Engine::FRTWEngineLobbyDeckSelected::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWEngineLobbyDeckSelected::vftable into *this; when the int argument is nonzero … |
| `01279f00` | Engine::FRTWEngineLobbyDeckSelected::ActionName | confirmed | Returns the constant C string DeckSelected (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWEngineLobbyDeckSelected::vftable. |
| `01279f20` | Engine::FRTWEngineLobbyDeckSelected::DebugName | confirmed | Returns the constant C string DeckSelected : 1 : .\engine\lobby\frtwlobbyhandlers.cc:378 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWEngine… |
| `01279f30` | Engine::FRTWEngineLobbyDeckSelected::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |
| `0127aaf0` | — | unknown | Virtual slot [2] of Engine::FRTWEngineLobbyDeckSelected::vftable (335 bytes). Reads a string through FUN_012eb250 and the MatchID value through FUN_012eb290; l… |

**Engine::FRTWEngineLobbyStartSinglePlayerGame** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01279d40` | Engine::FRTWEngineLobbyStartSinglePlayerGame::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWEngineLobbyStartSinglePlayerGame::vftable into *this; when the int argument is… |
| `01279db0` | Engine::FRTWEngineLobbyStartSinglePlayerGame::ActionName | confirmed | Returns the constant C string StartCampaignGame (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWEngineLobbyStartSinglePlayerGame::vftable. |
| `01279dd0` | Engine::FRTWEngineLobbyStartSinglePlayerGame::DebugName | confirmed | Returns the constant C string StartCampaignGame : 0 : .\engine\lobby\frtwlobbyhandlers.cc:367 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWE… |
| `01279de0` | Engine::FRTWEngineLobbyStartSinglePlayerGame::ScalarDeletin… | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |
| `0127a640` | Engine::FRTWEngineLobbyStartSinglePlayerGame::Handle | confirmed | Virtual slot [2] of Engine::FRTWEngineLobbyStartSinglePlayerGame::vftable (33 bytes). Calls FUN_012e0ba0, then ThreadOwnedLookup_0127a580; when that returns no… |

**Engine::FRTWLobbyCancelCreateMatch** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01279800` | Engine::FRTWLobbyCancelCreateMatch::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWLobbyCancelCreateMatch::vftable into *this; when the int argument is nonzero c… |
| `01279870` | Engine::FRTWLobbyCancelCreateMatch::ActionName | confirmed | Returns the constant C string CancelCreateMatch (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWLobbyCancelCreateMatch::vftable. |
| `01279890` | Engine::FRTWLobbyCancelCreateMatch::DebugName | confirmed | Returns the constant C string CancelCreateMatch : 0 : .\engine\lobby\frtwlobbyhandlers.cc:134 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWL… |
| `012798a0` | Engine::FRTWLobbyCancelCreateMatch::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |
| `0127a610` | Engine::FRTWLobbyCancelCreateMatch::Handle | confirmed | Virtual slot [2] of Engine::FRTWLobbyCancelCreateMatch::vftable (33 bytes). Calls FUN_012e0ba0, then ThreadOwnedLookup_0127a580; when that returns non-null cal… |

**Engine::FRTWLobbyCancelQueryMatch** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012796b0` | Engine::FRTWLobbyCancelQueryMatch::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWLobbyCancelQueryMatch::vftable into *this; when the int argument is nonzero ca… |
| `01279720` | Engine::FRTWLobbyCancelQueryMatch::ActionName | confirmed | Returns the constant C string CancelQueryMatch (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWLobbyCancelQueryMatch::vftable. |
| `01279740` | Engine::FRTWLobbyCancelQueryMatch::DebugName | confirmed | Returns the constant C string CancelQueryMatch : 0 : .\engine\lobby\frtwlobbyhandlers.cc:120 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWLo… |
| `01279750` | Engine::FRTWLobbyCancelQueryMatch::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |
| `0127a880` | Engine::FRTWLobbyCancelQueryMatch::Handle | confirmed | Virtual slot [2] of Engine::FRTWLobbyCancelQueryMatch::vftable (434 bytes). Reads the GroupID value, builds a CancelCreateMatch command through FUN_012ec110 an… |

**Engine::FRTWLobbyQueryCreateMatch** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127a2b0` | Engine::FRTWLobbyQueryCreateMatch::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWLobbyQueryCreateMatch::vftable into *this; when the int argument is nonzero ca… |
| `0127a320` | Engine::FRTWLobbyQueryCreateMatch::ActionName | confirmed | Returns the constant C string QueryCreateMatch (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWLobbyQueryCreateMatch::vftable. |
| `0127a340` | Engine::FRTWLobbyQueryCreateMatch::DebugName | confirmed | Returns the constant C string QueryCreateMatch : 0 : .\engine\lobby\frtwlobbyhandlers.cc:49 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWLob… |
| `0127a350` | Engine::FRTWLobbyQueryCreateMatch::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |
| `0127bb70` | Engine::FRTWLobbyQueryCreateMatch::Handle | confirmed | Virtual slot [2] of Engine::FRTWLobbyQueryCreateMatch::vftable (1423 bytes). Reads the current map entries from a list of *.csv files through FUN_012f2fb0, FUN… |

**Engine::FRTWLobbyQueryMatch** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01279560` | Engine::FRTWLobbyQueryMatch::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWLobbyQueryMatch::vftable into *this; when the int argument is nonzero calls FU… |
| `012795d0` | Engine::FRTWLobbyQueryMatch::ActionName | confirmed | Returns the constant C string QueryMatch (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWLobbyQueryMatch::vftable. |
| `012795f0` | Engine::FRTWLobbyQueryMatch::DebugName | confirmed | Returns the constant C string QueryMatch : 0 : .\engine\lobby\frtwlobbyhandlers.cc:87 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWLobbyQuer… |
| `01279600` | Engine::FRTWLobbyQueryMatch::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |
| `0127a670` | Engine::FRTWLobbyQueryMatch::Handle | confirmed | Virtual slot [2] of Engine::FRTWLobbyQueryMatch::vftable (517 bytes). Reads the GroupID value, looks the lobby object up through ThreadOwnedLookup_0127a580 and… |

**Engine::FRTWLobbyRequestCreateMatch** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01279950` | Engine::FRTWLobbyRequestCreateMatch::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWLobbyRequestCreateMatch::vftable into *this; when the int argument is nonzero … |
| `012799c0` | Engine::FRTWLobbyRequestCreateMatch::ActionName | confirmed | Returns the constant C string RequestCreateMatch (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWLobbyRequestCreateMatch::vftable. |
| `012799e0` | Engine::FRTWLobbyRequestCreateMatch::DebugName | confirmed | Returns the constant C string RequestCreateMatch : 0 : .\engine\lobby\frtwlobbyhandlers.cc:145 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTW… |
| `012799f0` | Engine::FRTWLobbyRequestCreateMatch::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |
| `0127ac70` | — | unknown | Virtual slot [2] of Engine::FRTWLobbyRequestCreateMatch::vftable (1260 bytes). Builds a ValueData argument list with the keys 0xfad (string), 0xfaa, 0xfa8, 0xf… |

**Engine::FRTWUILobbyPerformSetMap** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01279fe0` | Engine::FRTWUILobbyPerformSetMap::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWUILobbyPerformSetMap::vftable into *this; when the int argument is nonzero cal… |
| `0127a050` | Engine::FRTWUILobbyPerformSetMap::ActionName | confirmed | Returns the constant C string PerformSetMap (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWUILobbyPerformSetMap::vftable. |
| `0127a070` | Engine::FRTWUILobbyPerformSetMap::DebugName | confirmed | Returns the constant C string PerformSetMap : 0 : .\engine\lobby\frtwlobbyhandlers.cc:400 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWUILob… |
| `0127a080` | Engine::FRTWUILobbyPerformSetMap::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |

**Engine::FRTWUILobbyRequestQuickJoin** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01279aa0` | Engine::FRTWUILobbyRequestQuickJoin::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWUILobbyRequestQuickJoin::vftable into *this; when the int argument is nonzero … |
| `01279b10` | Engine::FRTWUILobbyRequestQuickJoin::ActionName | confirmed | Returns the constant C string RequestQuickJoin (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWUILobbyRequestQuickJoin::vftable. |
| `01279b30` | Engine::FRTWUILobbyRequestQuickJoin::DebugName | confirmed | Returns the constant C string RequestQuickJoin : 0 : .\engine\lobby\frtwlobbyhandlers.cc:206 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWUI… |
| `01279b40` | Engine::FRTWUILobbyRequestQuickJoin::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |
| `0127b170` | — | unknown | Virtual slot [2] of Engine::FRTWUILobbyRequestQuickJoin::vftable (2506 bytes). Builds the string /debug/debug_mode and calls the unlabelled helpers FUN_012e441… |

**Engine::FRTWUILobbyStartSinglePlayerGame** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01279bf0` | Engine::FRTWUILobbyStartSinglePlayerGame::ctor | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWUILobbyStartSinglePlayerGame::vftable into *this; when the int argument is non… |
| `01279c60` | Engine::FRTWUILobbyStartSinglePlayerGame::ActionName | confirmed | Returns the constant C string StartSinglePlayerGame (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWUILobbyStartSinglePlayerGame::vftable. |
| `01279c80` | Engine::FRTWUILobbyStartSinglePlayerGame::DebugName | confirmed | Returns the constant C string StartSinglePlayerGame : 0 : .\engine\lobby\frtwlobbyhandlers.cc:354 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::F… |
| `01279c90` | Engine::FRTWUILobbyStartSinglePlayerGame::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; retur… |
| `0127aa40` | Engine::FRTWUILobbyStartSinglePlayerGame::Handle | likely | Virtual slot [2] of Engine::FRTWUILobbyStartSinglePlayerGame::vftable (171 bytes). Calls FUN_012e0ba0 and FUN_00f9c130; when the UI object exists calls FUN_012… |

**Engine::LobbyAddLobbyGroupDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127c460` | Engine::LobbyAddLobbyGroupDisplayAction::ctor | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyAddLobbyGroupDisplayAction::vftable into *this; when the int argument is nonzero c… |
| `0127c4f0` | Engine::LobbyAddLobbyGroupDisplayAction::DebugName | confirmed | Returns the constant C string kAddLobbyGroupDisplayAction : 0 : ... lobbyhandlers.cc:187 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127c500` | Engine::LobbyAddLobbyGroupDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns th… |

**Engine::LobbyAddLobbyParticipantDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127c5b0` | Engine::LobbyAddLobbyParticipantDisplayAction::ctor | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyAddLobbyParticipantDisplayAction::vftable into *this; when the int argument is non… |
| `0127c640` | Engine::LobbyAddLobbyParticipantDisplayAction::DebugName | confirmed | Returns the constant C string kAddLobbyParticipantDisplayAction : 0 : ... lobbyhandlers.cc:212 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127c650` | Engine::LobbyAddLobbyParticipantDisplayAction::ScalarDeleti… | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns th… |

**Engine::LobbyBulkAddLobbyParticipantDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127c700` | Engine::LobbyBulkAddLobbyParticipantDisplayAction::ctor | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyBulkAddLobbyParticipantDisplayAction::vftable into *this; when the int argument is… |
| `0127c790` | Engine::LobbyBulkAddLobbyParticipantDisplayAction::DebugName | confirmed | Returns the constant C string kBulkAddLobbyParticipantDisplayAction : 0 : ... lobbyhandlers.cc:243 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtab… |
| `0127c7a0` | Engine::LobbyBulkAddLobbyParticipantDisplayAction::ScalarDe… | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns th… |

**Engine::LobbyChangeLobbyStatusDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127c850` | Engine::LobbyChangeLobbyStatusDisplayAction::ctor | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyChangeLobbyStatusDisplayAction::vftable into *this; when the int argument is nonze… |
| `0127c8e0` | Engine::LobbyChangeLobbyStatusDisplayAction::DebugName | confirmed | Returns the constant C string kChangeLobbyStatusDisplayAction : 0 : ... lobbyhandlers.cc:273 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127c8f0` | Engine::LobbyChangeLobbyStatusDisplayAction::ScalarDeleting… | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns th… |

**Engine::LobbyDisplayMOTDDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127cb00` | Engine::LobbyDisplayMOTDDisplayAction::ctor | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyDisplayMOTDDisplayAction::vftable into *this; when the int argument is nonzero cal… |
| `0127cb90` | Engine::LobbyDisplayMOTDDisplayAction::DebugName | confirmed | Returns the constant C string kDisplayMOTDDisplayAction : 0 : ... lobbyhandlers.cc:303 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127cba0` | Engine::LobbyDisplayMOTDDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns th… |

**Engine::LobbyEndBulkAddLobbyGroupDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127cf00` | Engine::LobbyEndBulkAddLobbyGroupDisplayAction::ctor | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyEndBulkAddLobbyGroupDisplayAction::vftable into *this; when the int argument is no… |
| `0127cf90` | Engine::LobbyEndBulkAddLobbyGroupDisplayAction::DebugName | confirmed | Returns the constant C string kEndBulkAddLobbyGroupDisplayAction : 0 : ... lobbyhandlers.cc:342 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127cfa0` | Engine::LobbyEndBulkAddLobbyGroupDisplayAction::ScalarDelet… | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns th… |

**Engine::LobbyLaunchGameDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127d060` | Engine::LobbyLaunchGameDisplayAction::ctor | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyLaunchGameDisplayAction::vftable into *this; when the int argument is nonzero call… |
| `0127d0f0` | Engine::LobbyLaunchGameDisplayAction::DebugName | confirmed | Returns the constant C string kLaunchGameDisplayAction : 0 : ... lobbyhandlers.cc:348 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127d100` | Engine::LobbyLaunchGameDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns th… |

**Engine::LobbyRemoveLobbyGroupDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127c1c0` | Engine::LobbyRemoveLobbyGroupDisplayAction::ctor | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyRemoveLobbyGroupDisplayAction::vftable into *this; when the int argument is nonzer… |
| `0127c250` | Engine::LobbyRemoveLobbyGroupDisplayAction::DebugName | confirmed | Returns the constant C string kRemoveLobbyGroupDisplayAction : 0 : ... lobbyhandlers.cc:141 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127c260` | Engine::LobbyRemoveLobbyGroupDisplayAction::ScalarDeletingD… | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns th… |

**Engine::LobbyRemoveLobbyParticipantDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127c310` | Engine::LobbyRemoveLobbyParticipantDisplayAction::ctor | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyRemoveLobbyParticipantDisplayAction::vftable into *this; when the int argument is … |
| `0127c3a0` | Engine::LobbyRemoveLobbyParticipantDisplayAction::DebugName | confirmed | Returns the constant C string kRemoveLobbyParticipantDisplayAction : 0 : ... lobbyhandlers.cc:159 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtabl… |
| `0127c3b0` | Engine::LobbyRemoveLobbyParticipantDisplayAction::ScalarDel… | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns th… |

**Engine::LobbySetDisplayLobbyDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127c9a0` | Engine::LobbySetDisplayLobbyDisplayAction::ctor | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbySetDisplayLobbyDisplayAction::vftable into *this; when the int argument is nonzero… |
| `0127ca30` | Engine::LobbySetDisplayLobbyDisplayAction::DebugName | confirmed | Returns the constant C string kSetDisplayLobbyDisplayAction : 0 : ... lobbyhandlers.cc:297 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127ca40` | Engine::LobbySetDisplayLobbyDisplayAction::ScalarDeletingDt… | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns th… |

**Engine::LobbyStartBulkAddLobbyGroupDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127cda0` | Engine::LobbyStartBulkAddLobbyGroupDisplayAction::ctor | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyStartBulkAddLobbyGroupDisplayAction::vftable into *this; when the int argument is … |
| `0127ce30` | Engine::LobbyStartBulkAddLobbyGroupDisplayAction::DebugName | confirmed | Returns the constant C string kStartBulkAddLobbyGroupDisplayAction : 0 : ... lobbyhandlers.cc:336 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtabl… |
| `0127ce40` | Engine::LobbyStartBulkAddLobbyGroupDisplayAction::ScalarDel… | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns th… |

**Engine::LobbyUpdateLobbyStatsDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127cc50` | Engine::LobbyUpdateLobbyStatsDisplayAction::ctor | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyUpdateLobbyStatsDisplayAction::vftable into *this; when the int argument is nonzer… |
| `0127cce0` | Engine::LobbyUpdateLobbyStatsDisplayAction::DebugName | confirmed | Returns the constant C string kUpdateLobbyStatsDisplayAction : 0 : ... lobbyhandlers.cc:323 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127ccf0` | Engine::LobbyUpdateLobbyStatsDisplayAction::ScalarDeletingD… | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns th… |

**Engine::MatchViewBase** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013aeff0` | Engine::MatchViewBase::MatchViewBase | confirmed | Calls 0143cfd0, stores Engine::MatchViewBase::vftable at *this, then calls 0143d020 eight times with 0xfa6, 0xfad, 0xfaa, 0xfb0, 0xfa9, 0xfa7, 0xfb1 and 0x273;… |
| `013af0b0` | Engine::MatchViewBase::DtorBody | confirmed | Stores Engine::MatchViewBase::vftable at *this, calls 0143c1e0 and frees nothing; returns nothing. |
| `013af130` | Engine::MatchViewBase::~MatchViewBase | confirmed | Scalar deleting destructor in vtable slot 0: stores Engine::MatchViewBase::vftable, calls 0143c1e0, then calls _free(this) when bit 0 of the flag argument is s… |
| `013afb10` | Engine::MatchViewBase::WriteMatchRecord | confirmed | Virtual slot 5 of MatchViewBase: writes GroupID, ParentGroupID, LobbyType, GameID, MatchState and IsLaunched (via 0130e750) into the output table, then Players… |
| `013af7c0` | Engine::MatchViewBase::ReadFilterFlagList | likely | Virtual slot 1 of MatchViewBase: reads key 0xfa5 of its value argument through its vtable slot 1; requires type code 6 (integer list, assert 'mTypeID == kInteg… |
| `013af340` | Engine::MatchViewBase::Vslot4 | speculative | Virtual slot 4 of MatchViewBase: when param_3 is nonzero, builds a std::string 'Match', passes it to 0130df10, then calls this->vtable slot 5 (offset 0x14) wit… |

**MatchmakingAvailableBuddiesDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7f4c0` | MatchmakingAvailableBuddiesDataSource::GetColumnName | confirmed | Column-name table of the available-buddies list: BuddyName for index 0, BuddyGuid for index 1 and <error> otherwise; it never touches this. |
| `00a82bc0` | MatchmakingAvailableBuddiesDataSource::GetField | confirmed | Lua/UI data-source field getter (vtable slot [4]): triggers the lazy rebuild (00a82590) when this+0x228 is clear, walks the buddy list from this+0x230 with str… |
| `00a82c60` | MatchmakingAvailableBuddiesDataSource::GetRowCount | confirmed | Row-count getter (vtable slot [2]): if the loaded flag at this+0x228 is clear it first runs the rebuild routine 00a82590, then returns the buddy count stored a… |
| `00a82e30` | MatchmakingAvailableBuddiesDataSourceDtor | confirmed | Destructor body of the Matchmaking.AvailableBuddies data source: stores the three class vtables, releases the two observer objects at param_1[0x9b] and param_1… |
| `00a831a0` | MatchmakingAvailableBuddiesDataSourceCtor | confirmed | Constructor of the Matchmaking.AvailableBuddies data source: registers the name "BaseClient.Matchmaking.AvailableBuddies" through 00922890, stores the three cl… |
| `00a832a0` | MatchmakingAvailableBuddiesDataSourceScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]) of MatchmakingAvailableBuddiesDataSource: runs the destructor body 00a82e30 and frees the object with _free when t… |
| `00a82590` | — | unknown | Rebuild routine for the available-buddies table: clears the node list at this+0x260, then for every row of the UI data source "BaseClient.GuildMembers" (row co… |

**MatchmakingAvailableBuddiesDataSource::FriendsListObserver** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7f560` | MatchmakingAvailableBuddiesDataSource::FriendsListObserver:… | confirmed | Reads the owner data source from this+0x1c, clears that object's data-changed byte at +0x258, and tail-jumps (ff e2 in the machine code) to the owner's virtual… |
| `00a7fe60` | MatchmakingAvailableBuddiesDataSource::FriendsListObserver:… | confirmed | Deleting destructor of the friend-list observer: calls the observer destructor body 00a7f500 and frees the object with _free when bit 0 of param_2 is set. |
| `00a80930` | MatchmakingAvailableBuddiesDataSource::FriendsListObserver:… | confirmed | Constructor: runs the base construction FUN_0081b470, installs the FriendsListObserver vtable, stores the owner argument at +0x1c (the field the change notific… |
| `00a7f500` | MatchmakingAvailableBuddiesDataSource::FriendsListObserver:… | likely | Destructor body: rewrites the observer's vtable pointer, clears its name registration with FUN_008356b0(&DAT_0175b400) and runs the base destructor FUN_0083561… |

**MatchmakingAvailableBuddiesDataSource::GuildMemberObserver** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7f5e0` | MatchmakingAvailableBuddiesDataSource::GuildMemberObserver:… | confirmed | Same notification as 00a7f560 for the guild observer: takes the owner at this+0x1c, clears the owner's data-changed byte at +0x258 and tail-jumps to the owner'… |
| `00a7fe80` | MatchmakingAvailableBuddiesDataSource::GuildMemberObserver:… | confirmed | Deleting destructor of the guild-member observer: calls the observer destructor body 00a7f580 and frees the object with _free when bit 0 of param_2 is set. |
| `00a809a0` | MatchmakingAvailableBuddiesDataSource::GuildMemberObserver:… | confirmed | Constructor: runs the base construction FUN_0081b470, installs the GuildMemberObserver vtable, stores the owner argument at +0x1c and registers the name BaseCl… |
| `00a7f580` | MatchmakingAvailableBuddiesDataSource::GuildMemberObserver:… | likely | Destructor body: rewrites the guild-member observer's vtable pointer, clears its name registration with FUN_008356b0(&DAT_0175b400) and runs the base destructo… |

**MatchmakingQueuesDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7f2f0` | MatchmakingQueuesDataSource::GetColumnName | confirmed | Column-name table of the matchmaking queue list: QueueId, Name, MatchType, MinPlayers, MaxPlayers, MinTeams, MaxTeams, MaxGameStartDelay, Param1..Param7, Pendi… |
| `00a81e90` | MatchmakingQueuesDataSource::GetField | confirmed | Lua/UI data-source field getter (vtable slot [4]): lazily loads rows (calls the loader through this+4 slot 3 once, setting flag this+0x228), walks the row list… |
| `00a82d50` | MatchmakingQueuesDataSourceDtor | confirmed | Destructor body of the Matchmaking.Queues data source: stores the three MatchmakingQueuesDataSource vtables (main object plus two sub-objects at +0x30 and +0x3… |
| `00a83180` | MatchmakingQueuesDataSourceScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]) of MatchmakingQueuesDataSource: runs the destructor body 00a82d50 and frees the object with _free when the delete … |
| `00a7fe00` | MatchmakingQueuesDataSource::GetRowCount | likely | Lazily builds the rows once: when the byte at this+0x228 is clear it calls virtual slot 3 (offset 0xc) on the sink object at this+4 and sets that byte to 1, th… |
| `00a830a0` | — | unknown | Constructor of the Matchmaking.Queues data source: registers the UI data source name through 00922890(PTR_s_BaseClient.Matchmaking.Queues), stores the three cl… |

**MatchmakingRequestsDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7f430` | MatchmakingRequestsDataSource::GetColumnName | confirmed | Column-name table of the matchmaking request list: RequestId, QueueId, RequestingPlayerGuid, IsGroupRequest and CreationTime for indices 0..4, <error> otherwis… |
| `00a82420` | MatchmakingRequestsDataSource::GetField | confirmed | Lua/UI data-source field getter (vtable slot [4]): lazily loads rows (one call through this+4 slot 3, then flag this+0x228 = 1), walks the request list from th… |
| `00a83d70` | MatchmakingRequestsDataSourceCtor | confirmed | Constructor of the Matchmaking.Requests data source: registers the name through 00922890(PTR_s_BaseClient.Matchmaking.Requests), stores the three class vtables… |
| `00a83e30` | MatchmakingRequestsDataSourceDtor | confirmed | Destructor body of the Matchmaking.Requests data source: stores the three class vtables, stores the SoeUtil::List<EncounterMatchmaking::MatchmakingRequest,-1> … |
| `00a84260` | MatchmakingRequestsDataSourceScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]) of MatchmakingRequestsDataSource: runs the destructor body 00a83e30 and frees the object with _free when the delet… |
| `00a7fe30` | MatchmakingRequestsDataSource::GetRowCount | likely | Byte-identical sibling of 00a7fe00: when the byte at this+0x228 is clear it calls virtual slot 3 on the sink at this+4, sets that byte to 1, and returns the co… |
| `00a839d0` | MatchmakingRequestsDataSourceClearRequests | likely | Empties the request list at param_1[0x98] by repeatedly destroying the head node with 00a832c0 until it is null, then notifies the data source through its vtab… |

**UI::MatchmakingScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f82c0` | UI::MatchmakingScreen::MatchmakingScreen | confirmed | Constructor (fastcall, ECX = this): calls UI::MatchmakingScreenBase::MatchmakingScreenBase 013ff750 (which runs 012f8420 and stores its base vtable), then stor… |
| `012f8320` | UI::MatchmakingScreen::DtorBody | confirmed | Non-deleting destructor body: stores UI::MatchmakingScreen::vftable at *this, then calls 013ff7b0 (UI::MatchmakingScreenBase::DtorBody). |
| `012f8370` | UI::MatchmakingScreen::~MatchmakingScreen | confirmed | Scalar deleting destructor (vtable slot [0]): stores UI::MatchmakingScreen::vftable, calls the destructor body 013ff7b0, then calls _free on this when bit 0 of… |

**UI::MatchmakingScreenBase** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013ff750` | UI::MatchmakingScreenBase::MatchmakingScreenBase | confirmed | Constructor: calls 012f8420 on this, stores UI::MatchmakingScreenBase::vftable (018e3e2c) into *this, returns this. |
| `013ff7b0` | UI::MatchmakingScreenBase::DtorBody | confirmed | Non-deleting destructor body: stores UI::MatchmakingScreenBase::vftable into *param_1, calls 012f8480 on the object, returns. Does not free. |
| `013ff810` | UI::MatchmakingScreenBase::~MatchmakingScreenBase | confirmed | Scalar deleting destructor: stores UI::MatchmakingScreenBase::vftable into *this, calls 012f8480 on this, frees this through _free when bit 0 of the stack byte… |
| `013ff870` | UI::MatchmakingScreenBase::VSlot3 | speculative | Virtual slot [3] shared by UI::MatchmakingScreen and UI::MatchmakingScreenBase. Calls 012e0ba0 with ECX=this, passes its return value in ECX to 00f9c010; if th… |
| `013ff890` | — | unknown | Virtual slot [1] shared by UI::MatchmakingScreen and UI::MatchmakingScreenBase. Calls 012f8510, allocates 0x40 bytes (00d833a6); if the allocation is non-null … |

**(no class)** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a811a0` | FindFriendsListDataSourceByName | confirmed | Hashes param_2 with 00705bc0, follows bucket hash%1000 in the table at this+0xaea4 via entry+4, compares the stored hash at entry+8, and RTTI-casts a found obj… |
| `00a822e0` | FormatUint64ToIString | confirmed | Formats the 64-bit unsigned value given by the two dwords of param_2 as "%llu" into the SoeUtil wide string at param_1 (via 0082ecd0) and returns the string's … |
| `00a82310` | FormatInt64ToIString | confirmed | Formats the 64-bit signed value given by the two dwords of param_2 as "%lld" into the SoeUtil wide string at param_1 (via 0082ecd0) and returns the string's da… |
| `00c08710` | RegisterMatchmakingModule | confirmed | Registration wrapper: pushes nup=0, the 5-entry table 0x0182a2b0 and the 'Matchmaking' module-name pointer, calls the Lua module registrar FUN_00e39e40 and ret… |
| `013af410` | Engine::MatchViewBase::WriteMatchEntry | likely | Calls 0143ba20 and, when its result is nonzero and not 4, builds the 'Match' key (0130df10), then calls the vtable slot 5 of param_1 with (param_2, result-4) a… |
| `013af4f0` | WriteAccountListByRole | likely | Iterates the id list returned by 01308740 for role param_3; for each id nonzero it calls 012dfbc0 and 012de760, and when the lookup returns an object writes a … |
| `00a7f410` | NotifyRowsChanged_00a7f410 | speculative | Calls slot 3 (vtable offset 0xc) on the row-sink subobject embedded at this+0x34 (the machine code is lea ecx,[esi+0x34]; call [eax+0xc]) and then sets the dat… |
| `00a7f4a0` | NotifyRowsChanged_00a7f4a0 | speculative | Byte-identical sibling of the requests-list notification 00a7f410: calls slot 3 (offset 0xc) on the row-sink subobject at this+0x34 and sets the changed byte a… |
| `00c08730` | — | unknown | Lua 'LeaveQueue' handler: with one numeric argument (FUN_00e3a5f0 gate, FUN_00e3a770 + FUN_00d83c90 read) it calls FUN_00a84150(queue id) on the matchmaking ma… |
| `00c08ae0` | — | unknown | Lua 'PopulateQueues' handler: resets the queue state (FUN_00a39000), builds a matchmaking request object (FUN_00a7fb90), copies the local player's two actor wo… |
| `00c08b90` | — | unknown | Lua 'LeaveMatchRequest' handler: with one argument it reads an eight-byte value (FUN_00bfea90), builds the leave-request object (FUN_00a7fc60) carrying it and,… |
| `00c08e30` | — | unknown | Lua 'JoinQueue' handler: with one argument it reads the queue id, resolves the queue record through FUN_00a80300, builds an AddMatchRequestPacket (FUN_00c08790… |

### `ui.player_positions`

**CommandPacketRequestPlayerPositionsReply::PlayerPosition** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a85ae0` | PlayerPositionDtorBody | likely | Destructor body of one PlayerPosition row: rewrites the embedded SoeUtil::IString<char> at this+8 to its static vtable value and, when the string holds a heap … |
| `00a85ba0` | PlayerPositionCtor | likely | Default constructor body of a PlayerPosition row: stores the two default dwords from DAT_01be79c0/DAT_01be79c4 at +0 and +4 (the default 64-bit guid), installs… |
| `00a85c20` | PlayerPositionCtorWrapper | likely | SEH-framed constructor wrapper that runs the PlayerPosition default-constructor body 00a85ba0 on the object in ecx and returns that object. |
| `00a85c70` | PlayerPositionScalarDeletingDtor | likely | Deleting destructor wrapper for a PlayerPosition row: runs the destructor body 00a85ae0 and frees the node with _free when the delete flag bit 0 is set. |

**PlayerPositionsDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a85900` | PlayerPositionsDataSource::GetColumnName | confirmed | Column-title getter (vtable slot [1]): switches on the column index and returns the literals "Guid" (0), "Name" (1), "X Location" (2), "Z Location" (3), "Is Fr… |
| `00a85a70` | PlayerPositionsDataSource::GetRowCount | confirmed | Row-count getter (vtable slot [2]): a 7-byte field getter that loads the dword at this+0x234 and returns it (the current number of position rows). |
| `00a86050` | PlayerPositionsDataSourceCtor | confirmed | Constructor of the positions data source: registers the name through 00922890(PTR_s_BaseClient.PlayerPositions), stores the three class vtables, saves the owne… |
| `00a860f0` | PlayerPositionsDataSourceDtor | confirmed | Destructor body of the positions data source: installs the SoeUtil::List<...PlayerPosition,-1> vtable into the list sub-object at this+0x258 and drains the row… |
| `00a86170` | PlayerPositionsDataSourceScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]) of PlayerPositionsDataSource: runs the destructor body 00a860f0 and frees the object with _free when the delete fl… |
| `00a85df0` | PlayerPositionsDataSourceAppendRow | likely | Copies one PlayerPosition from param_2 into the table: allocates a node via 00a85cd0, copies the 64-bit guid from param_2[0]/[1], assigns the name string from … |
| `00a85fb0` | PlayerPositionsDataSourceClear | likely | Empties the positions table: destroys every node of the list at this+0x25c with 00a85d60 (list object at this+0x258) and then tail-calls the data source's vtab… |
| `00a85e70` | — | unknown | Lua/UI field getter (vtable slot [4]): walks the row list from this+0x22c (next link at +0x40, count at +0x234) to row param_2 and, by column param_3, writes t… |

**SoeUtil::List** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a85cd0` | PlayerPositionsListAppend | likely | Allocates a node through owner vtable byte offset +8, constructs its PlayerPosition with 00a85c20, and appends it using node+0x40 as next and node+0x44 as prev… |
| `00a85d60` | PlayerPositionsListRemoveNode | likely | Unlinks node param_2 from the PlayerPosition list: head is owner[1], tail owner[2], count owner[3], next link node+0x40 and previous link node+0x44. It repairs… |
| `00a85f90` | PlayerPositionListClear | likely | Repeatedly takes the head node at param_1+4 and destroys it with 00a85d60 until the head is null, emptying the PlayerPosition list. |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d1d20` | NamePlatesInternals::NamePlateManagerCtor | likely | Initializes name-plate colors and flags, constructs two maps and a debug-node list, then generates and hashes twenty NamePlate%d keys. |
| `009d1eb0` | AllocateNamePlateManager | likely | Allocates 0x2a8 bytes, constructs the object with FUN_009d1d20 when allocation succeeds, stores the pointer, and returns the output address. |
| `009d1f20` | NamePlatesInternals::NamePlateManagerDtor | likely | Drains DebugNode entries, restores the integer-list vtable, releases associated storage, and destroys two embedded map objects. |
| `00a85b50` | EhCleanupCallPlayerPositionDtor | likely | SEH-framed one-shot cleanup that calls the PlayerPosition destructor body 00a85ae0 on its this argument (saved in the SEH frame) and returns; it is referenced … |

### `unknown.lobby`

**(no class)** (30)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c4670` | Ctor_DwordZero_EmptyStringAt8 | speculative | Default-constructs an object: stores 0 at this+0, sets this+0x1c to 0xf and this+0x18 to 0, and clears the byte at this+8, the buffer of an empty MSVC std::str… |
| `014c46c0` | Copy_DwordAndStringAt8 | speculative | Copy-constructs an object from param_2: copies the DWORD at +0, sets the empty header fields at this+0x18/+0x1c, clears the byte at +8, then calls std::basic_s… |
| `014c48c0` | ListNodeStringOrEmpty | speculative | Uses 013b1720 to get an iterator (piVar5); if its node is the list end (compared with this+0x4c) it assigns the empty string to param_2, otherwise it assigns t… |
| `014c4c30` | PackTripleCallSubobject | speculative | Thiscall with three stack DWORD arguments (ret 0xc). Copies the three arguments into a 12-byte local, then calls 013098c0 on this+0xc with the address of that … |
| `014c5140` | CopyAssign_SelfCheck | speculative | Thiscall with one stack argument (the other object), returns this. If other is not this, calls 012a3c30 with this and this+4 list references, then 012a3f30 wit… |
| `014c54c0` | CopyAssign_SelfCheck | speculative | Byte-identical in logic to 014c5140: thiscall with one stack argument, returns this, and calls 012a3c30 then 012a3f30 when the two objects differ. |
| `014c57b0` | AllocAndConstruct_0x54 | speculative | Allocates 0x54 bytes with 00d833a6 and, when the allocation is non-null, runs 014c55d0 on it and returns that object; returns 0 when the allocation fails. |
| `014c5810` | RegisterCreator_0x124 | speculative | Pushes the id 0x124 and the address of 014c57b0, calls 013ce130 with them, then calls 013ce230 with the returned value in ECX (bytes 68b0574c01 6824010000 e8..… |
| `014c5d70` | AllocAndConstruct_0x14 | speculative | Allocates 0x14 bytes with 00d833a6; when non-null runs 014c5830 on it and returns that object, else returns 0. |
| `014c5dd0` | RegisterCreator_0x60 | speculative | Pushes the id 0x60 and the address of 014c5d70, calls 013ce130 with them, then calls 013ce230 with the returned value (same pattern as RegisterCreator_0x124 at… |
| `014c61b0` | AllocAndConstruct_0xc | speculative | Allocates 0xc bytes with 00d833a6; when non-null runs 014c5e60 on it and returns the object, else 0. |
| `014c6210` | RegisterCreator_0x123 | speculative | Pushes the id 0x123 and the address of 014c61b0, calls 013ce130 with them, then calls 013ce230 with the returned value (same pattern as RegisterCreator_0x124 a… |
| `014c6ef0` | WrapCopy014c68d0_3a | speculative | Stdcall with three stack arguments (ret 0xc). Pushes seven DWORDs (param_1 low byte cleared into a local, plus the arguments) and calls 014c68d0 (cdecl, cleans… |
| `014c7040` | FillWrapper_014c6be0 | speculative | Thiscall with three stack arguments. Calls 014c6be0 with the same argument pattern and returns param_2 + param_3*4 (end pointer after param_3 DWORDs). |
| `014c70f0` | WrapCopy014c68d0_5a | speculative | Stdcall with five stack arguments (ret 0x14). Pushes seven DWORDs including a cleared local and calls 014c68d0 (cdecl, cleans 0x1c); no return value. |
| `014c7120` | WrapCopy014c68d0_3b | speculative | Stdcall with three stack arguments (ret 0xc). Pushes seven DWORDs and calls 014c68d0 (cdecl, cleans 0x1c); no return value. |
| `014c7b50` | AllocAndConstruct_0x28 | speculative | Allocates 0x28 bytes with 00d833a6; when non-null runs 014c7900 (ChangeBulkStatus constructor) on it and returns the object, else returns 0. |
| `014c7bb0` | RegisterCreator_0x117 | speculative | Pushes the factory address 014c7b50 and the id 0x117, calls 013ce130 with them, then calls 013ce230 with the returned value in ECX (bytes 68507b4c01 6817010000… |
| `014c7bd0` | PushPairIntoTwoMembers | speculative | Thiscall with two stack arguments (ret 8). Calls 0042c155 on this+8 with &arg1 (its accepted label is ArrayPushBackI, likely) and calls 014c7a40 on this+0x18 w… |
| `014c8220` | AllocAndConstruct_0x3c | speculative | Allocates 0x3c bytes with 00d833a6; when non-null runs 014c8040 on it and returns the object, else returns 0. |
| `014c8280` | RegisterCreator_0x153 | speculative | Pushes the id 0x153 and the address of 014c8220, calls 013ce130 with them, then calls 013ce230 with the returned value (same pattern as RegisterCreator_0x124). |
| `014c8820` | AllocAndConstruct_0x18 | speculative | Allocates 0x18 bytes with 00d833a6; when non-null runs 014c8420 on it and returns the object, else returns 0. |
| `014c8880` | RegisterCreator_0x152 | speculative | Pushes the factory address 014c8820 and the id 0x152, calls 013ce130 with them, then calls 013ce230 with the returned value in ECX (bytes 68208848 01 685201000… |
| `014c8c00` | Wrapper_005f15f0_At10_RetArg | speculative | Thiscall with one stack argument. Calls 005f15f0 on this+0x10 (its result is not used) and returns its stack argument (pseudocode return param_2; ret count not… |
| `014c8d40` | AllocAndConstruct_0x24 | speculative | Allocates 0x24 bytes with 00d833a6; when non-null runs 014c8b90 on it and returns the object, else returns 0. |
| `014c8da0` | RegisterCreator_0x157 | speculative | Pushes the factory address 014c8d40 and the id 0x157, calls 013ce130 with them, then calls 013ce230 with the returned value (same pattern as RegisterCreator_0x… |
| `014ca4c0` | FillWrapper_014ca070 | speculative | Thiscall with three stack arguments. Calls 014ca070 (fill of n DWORDs through 014c9b20) with the same argument pattern as the other wrappers, and returns param… |
| `014ca570` | WrapCopy9b50_5a | speculative | Thiscall-shaped wrapper with five stack arguments (bytes 51 8b542418 c6042400 8b042450 8b44241c 52 8b542414 51 8b4c2420 505152 e8.. 83c41c c21400): pushes seve… |
| `014ca270` | — | unknown | Stdcall with three stack arguments (ret 0xc; bytes 51 8b542410 c60424 00 ... e8.. 83c41c c20c00). Pushes seven DWORDs (including a cleared local) and calls 014… |
| `014ca600` | — | unknown | Stdcall with three stack arguments (ret 0xc; bytes 51 8b542410 c60424 00 ... e8.. 83c41c c20c00). Pushes seven DWORDs and calls 014c9b50; no return value. |

## Source shards

[[S0236|S0236]] (5), [[S0239 0090a890-009105b0|S0239]] (1), [[S0276 009cdf80-009d3fe0|S0276]] (3), [[S0290 00a1de60-00a243e0|S0290]] (8), [[S0291 00a244a0-00a29500|S0291]] (12), [[S0307 00a7b930-00a81420|S0307]] (16), [[S0308 00a81470-00a87b30|S0308]] (48), [[S0357 00b84e90-00b8a5f0|S0357]] (18), [[S0365 00bac0d0-00bb1110|S0365]] (8), [[S0383 00c05780-00c0db00|S0383]] (5), [[S0384 00c0dcd0-00c171d0|S0384]] (1), [[S0406 00c8f830-00c95560|S0406]] (7), [[S0420 00cdb370-00ce0140|S0420]] (6), [[S0565 01025bf0-0102b850|S0565]] (5), [[S0571 01044480-0104a2b0|S0571]] (10), [[S0666 012773d0-0127d160|S0666]] (85), [[S0667 0127d1b0-01281de0|S0667]] (3), [[S0681|S0681]] (1), [[S0682|S0682]] (12), [[S0689|S0689]] (9), [[S0724|S0724]] (40), [[S0732|S0732]] (3), [[S0733|S0733]] (29), [[S0738|S0738]] (5), [[S0739|S0739]] (17), [[S0748|S0748]] (12), [[S0750|S0750]] (3), [[S0772|S0772]] (55), [[S0774|S0774]] (2), [[S0776|S0776]] (30)
