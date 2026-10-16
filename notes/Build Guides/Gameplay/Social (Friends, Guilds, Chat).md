---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Social (Friends, Guilds, Chat)

Part of [[Build Guides]] › Gameplay. Related: [[Command System]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`) are the native-map labels for a function. Source tags: *client* = read from the shipped executable in a shard or in this pass; *client Lua* = the client's own UI script bytecode, losslessly decompiled in the lab (not redistributed); *client resource* = a data file from the shipped client's `Resources` folder; *client locale* = the shipped text table and the `CodeStringMappings.txt` message keys; *video* = a reviewed frame record (no player names, guild names or chat text are recorded); *wiki* = the fan wiki; *vault* = a project research note, used here only for facts it derived from the client or from our own native runs. OSFR-held data (packet captures and server JSON) is not used. Precedence is client, then video, then wiki. *(R)* marks reconstructed policy that is ours, not original behaviour. No original server code exists, so every statement about what the server did is a hypothesis built from what the client requires. The community emulators' friend, guild, group and chat tables, packet writers, role ids and catalogue rows are not used anywhere in this guide; family and sub-opcode numbers below are the ones the client's own constructors and dispatchers show. **Privacy:** this guide records mechanics and client strings only. It contains no player, guild or house names and no chat text. Message-key text is cited by key name or paraphrased. Related: [[Matchmaking and Lobby]], [[Trade]], [[Housing]], [[Quests and Objectives]], [[Trading Card Game]], [[Minigame Framework]], [[Combat and Abilities]], [[Character and Customization]], [[Coin Store and Purchases]], [[Gateway and Transport]], [[Command System]], [[Login and Account]], [[Scripting (Lua)]], [[Scaleform and Flash UI]], [[Text and Localization]], [[Zones and Terrain]].

### Overview

The word "social" covers eight separate player-facing systems. They are separate in the client's code and on the wire, so they are listed apart.

1. **The Social window** (the Friends button on the game dock; the welcome screen also shows a friends list). Four lists: Friends, Acquaintances (recently met players), Ignored, and Guild. A friend row shows name, online state, job badge and rank, popularity, and whether the friend is in an encounter. Row actions seen in the script: whisper, locate on the atlas, teleport to the friend, invite to house, go to house, invite to group, remove, ignore (client Lua `SocialHandler`; wiki Interface page; video shows the "My Friends" list open in several recordings).
2. **Invite pop-ups.** Friend, group, guild, house, trade, duel, encounter and mini-game invitations arrive as small timed pop-ups with accept and decline, handled by the "Game Action Queue" (GAQ) described under Invites below.
3. **Groups** (a party). A leader invites players by name; members appear in a HUD panel with job badge, rank, status text and, in combat, health, energy and lives. The leader can kick; anyone can leave. Groups also gate group chat, group quick chat and encounter announcements. Mini-game lobbies use the same invite call from a friend picker (client Lua `FriendPicker`, `Ui.InviteToGroup`).
4. **Guilds.** Created from the Social window by a player who has any job at level 15 (client locale tutorial text; wiki). One guild per player. Four default ranks, a message of the day, invitations, promote and demote, guild chat, a guild filter in the chat log and a guild marker on the minimap. A custom name needs a paid rename (client `PaidNameChange` flow, wiki).
5. **Chat.** A chat log with filters (All, Group, Guild, Area, Trade, Looking-for-group, Members-only), chat bubbles over heads, whispers (`/tell`), a quick-chat phrase wheel, an emote menu and slash commands. Accounts are either typed-chat or quick-chat-only (see Chat safety).
6. **Player actions on other players:** add friend, ignore, report as offensive, inspect, invite to group, invite to guild, invite to house, trade, duel.
7. **A second, separate social stack inside the card-game executable.** The embedded card client (Treasure War) has its own friend list, ignore list, guild, lobby chat and polls over a different TCP link. It is described briefly here and in [[Trading Card Game]].
8. **Adjacent surfaces mapped in the same subsystems but not social features:** a regional social-network posting integration (`net.sns`, a localized build), a YouTube video uploader (`net.youtube`), polls and ratings (`net.poll`, `net.rating`, the housing rating system), portrait photo upload (`net.portrait`) and the social-share manager (`game.social_share`). They are listed under Adjacent below and not specified further.

**Not found in the client.** There is no player-to-player mail: the only "mail" strings belong to the Postman job's sorting game and a keyboard media-key name (client locale, client Lua). There is no voice chat on the PC client: the voice-chat strings are tagged `_PS3` (client locale keys). Emotes exist, but they are rows of the quick-chat table, not a separate system.

### How the client implements it

#### Main classes

| Class or object | Role | Evidence |
|---|---|---|
| `ClientChatProcessor` (ctor `00b15450`, command handler `00b164b0`) | Console command handler for chat. Registers 15 command names against 9 command ids; owns a 256-entry spam list and a chat-history map; ten channel senders. | confirmed ctor; S0336 |
| `BaseChatPacket` family 15 (dispatcher `00b14350`) | Five payload modes: 1 `PacketChat`, 2 `ChatPacketEnterArea`, 3 `ChatPacketDebugChat`, 4 `ChatPacketFromStringId`, 5 `TellEchoPacket`. | confirmed; S0335 |
| `ClientQuickChatManager` (ctor `00b96db0`) | Owns the quick-chat table (`BaseClient.QuickChatData`), the emote category table, the autocomplete source and a text index; receives family 67 (`00b966e0`); sends phrases to channels or as tells. | confirmed; S0359, S0360 |
| Friend store: `BaseClient.FriendsList` (bind `00cb52c0`), `ClientSocialData` (`00c7e640`) and the handlers `00cb47a0`, `00cb4950`, `00cb4da0`, `00cb4ee0`, `00cb4fc0` | Sorted friend rows keyed by a 64-bit guid; mirrors server pushes; raises Lua callbacks `SocialHandler:OnFriendOnline` and `OnFriendOffline`. | confirmed; S0412 |
| `IgnoreListDataSource` (ctor `00cf9c90`), add and remove handlers `00cf8e90`, `00cf8b10` | Ignore list keyed by the XOR of the two guid halves; 64 buckets. | confirmed ctor; likely handlers |
| Acquaintance list (`List<Acquaintance::Record,20>`, applier `00a1bb70`) | Capped at 20 entries; add, remove, online-flag update. | likely; S0289 |
| `ClientGuildManager` (ctor `00b1e3b0`), guild record (ctor `0106e6f0`) | Guild data sources `BaseClient.Guilds` and `BaseClient.GuildMembers`; a 100-bucket guild-name cache; a 32-bucket pending-name-request cache; per guild a 100-bucket member map (0xe8-byte nodes keyed by 64-bit character guid) and a 10-bucket role map. | confirmed; S0336, S0337, S0577 |
| Group state: group object at client `+0x6ec`, pending invite record at `+0x6e8`; `GroupDataSource` and `GroupMemberDataSource` (ctors `00a7dc80`, `00a7dbe0`) | Group membership and the HUD rows. Dispatcher `0093daf0` (BaseClient vtable slot 71). | confirmed; S0249, S0307 |
| `GaqManager` and `GaqItem*` classes (64-bucket map) | The pop-up queue; typed items for guild, house, friend, group, trade, duel, encounter, mini-game and claim actions. | confirmed; S0286, S0287 |
| `InspectProxy` (ctor `00ad8880`) and data sources `BaseClient.Inspect`, `.Inspect.Jobs`, `.Inspect.Items` | Pedestal view and the inspect panel data. | confirmed ctor; S0324 |
| Client Lua objects `SocialHandler`, `ChatLog`, `chatHandler`, `ChatBubbles`, `GroupHandler`, `Gaq`, `GuildDialog`, `FriendPicker`, `PlayerInspect`, `AboutPlayer`, `EmoteHandler` | The presentation layer; native code reaches it through `009797a0`. | client Lua |
| Card-service stack: `UChatCommand_*`, `GuildCommand_*`, `LobbyCommand_*`, `PollCommand_*`, `Engine::Account*` | Separate command protocol over a direct TCP link in the card executable. | confirmed; S0724 to S0795 |

#### Two social stacks

The Free Realms world connection carries families 15, 22, 26, 40, 67, 68, 72, 74, 112, 129, 131 and 149, all tunnelled through the gateway (see [[Gateway and Transport]]). The card executable has its own connection to a card service, with command objects identified by class id and framed with three routing words (see [[Trading Card Game]], "Network"). Nothing in the card stack's friend, guild or chat classes uses the world families: its guild commands are class ids 417 to 429 and 461 to 468, not family 129, and S0778 states so explicitly. The two stacks do not share state in the client. Whether the card service shared a friend or guild database with the world server is unknown.

#### Friends: add, confirm, presence (world connection)

1. **Add.** The Social window, the chat-log name menu and the typed `Ui.AddFriend` call feed the add-friend sender `CheckAndSubmitFriendRequest` (`0093b9b0`; the Lua-to-native hop was not traced). It refuses locally when the candidate equals the player's own profile name (message key `FriendYourself`) or is already in the list (`FriendAlreadyOnList`); a second predicate blocks silently. Otherwise it sends `CommandPacketAddFriendRequest` (family 26, sub 14) through the world tunnel (client; S0248).
2. **Confirm request.** The target receives family 26 sub 16 (`CommandPacketConfirmFriendRequest`), parsed in the command dispatcher `00aa2560` case 0x10 and turned into a GAQ friend-invite pop-up (`00a0bd80` to `00a0ba90`) unless the Friend auto-refuse option is on, in which case the client calls the item's response routine at once (client).
3. **Response.** Accept and decline send `CommandPacketConfirmFriendResponse` (26 sub 17): three arguments (a guid pair and a status) and a trailing name member (`00a0a500`, senders `00a0f460`, `00a0f500`, `00a0f5a0`, which are byte-identical apart from their exception labels, so which status each one carries is not visible in the copies).
4. **Result to the requester.** The server answers with family 74 sub 8 (`FriendMessagePacket`): a status byte, a guid and name data. The client maps the byte to a notice (`ShowFriendStatusMessage` `00cb3fd0`): 1 accepted, 2 declined, 3 timed out, 4 cannot teleport (target in combat), 5 request sent, 6 cannot teleport (friend offline), 7 teleported (combat), 8 "no friend spam", 9 friend renamed, 10 cannot teleport to that player, 11 cannot teleport (target in a mini-game) (client; message keys named in Data and formats). On acceptance the new row arrives as 74 sub 6 (`FriendAdd`, handler `00cb4da0`).
5. **Remove.** `Ui.RemoveFriend` builds `CommandPacketRemoveFriendRequest` (26 sub 15) with a two-string name payload (`00c24a00`). The server answers with 74 sub 7 (`FriendRemove`, a guid); the client shows the "friend removed" notice and drops the row (`00cb4ee0`).
6. **Presence.** 74 sub 2 (online: name data, guid and a flag), sub 3 (offline: guid), sub 9 (status: name, guid, seven integers) and sub 10 (rename) update a row in place and raise the Lua callbacks (`00cb47a0`, `00cb4950`, `00cb41d0`, `00cb3f60`). 74 sub 1 replaces the whole list (`00cb4fc0`).
7. **Positions.** The atlas asks for friend positions with 26 sub 21 (`00c23e50`); 74 sub 5 returns 0x28-byte entries that update three fields of rows the client already holds and add nothing (`00cb37d0`; per-field meaning not recovered).
8. **Teleport to friend.** The Social window calls `Ui.TeleportToFriend(guid)` and the atlas can locate a friend. The refusal notices above (codes 4, 6, 7, 10, 11) prove a teleport request exists, but the packet that carries it was not identified (see Open questions).

All of families 72, 74, 112, 129, 131 and 192 are dropped by the main dispatcher until the player-data object (client `+0x3e0`) exists, i.e. until the self-data message (id 12) has been read (client; `009471a0` cases 0x48, 0x4a, 0x70, 0x81, 0x83, 0xc0 guard on that field).

#### Ignore list and acquaintances

The ignore list arrives as 112 sub 1 (count and records), grows with 112 sub 2 (one record: a guid and a name string) and shrinks with 112 sub 3 (a guid) (`00c81250`, handlers `00cf8e30`, `00cf8e90`, `00cf8b10`). The client sends family 26 sub 30 (`CommandPacketIgnoreRequest`) with a flag (1 add, 0 remove) and a two-string name payload (`00c24b00`, `00c24c00`). The client itself uses the list in at least two places: a guild invitation notice from an ignored player is declined without a pop-up (`00b1e7e0` asks `00a12720`, which reaches the 64-bucket XOR-key lookup `00cf8700`), and the trade dispatcher asks the same question (`00c6df10`, likely). The chat receive path (`00b11810`, `00b11ab0`, `00b14350`) holds no ignore check that we found, so hiding an ignored player's chat is the server's job (R); the wiki says an ignored player's typing, whispers and yells are not shown and that ignoring also removes them from your house or lot.

Acquaintances are a recent-players list of at most 20 entries: the client drops the oldest while the count is above 20 and replaces an existing entry for the same guid (`00a1bb70`). Family 72 has sub 1 add, sub 2 remove (matched by name) and sub 3 online-flag update; the client only builds these as receive targets (`00a1bb70`, `00a19170`, `00a16bb0`, `00a12160`), so all three are server to client. The data source `BaseClient.Acquaintances` has five columns: Guid, Name, Type, HowLongAgo, IsOnline (client Lua). What makes someone an acquaintance, and what "Type" holds, are not in the client.

#### Invites and the Game Action Queue (GAQ)

The Quests guide found GAQ is a queue of pop-ups, not a quest system. This pass ties it to the social invites. Every GAQ item is created locally by the client in response to another message, then shown through the Lua `Gaq:AddGameAction` (client Lua; `00a0c3a0` to `00a0e7f0`). **The guard that the earlier pass left unrecovered (`00917b60`) is the client's own AutoRefuse option:** a seven-entry table of option names in the user configuration section `AutoRefuse` (client; `00917b20`, `00917b60`; names read from the binary at `01b3aa74`).

| Index | Option name | Used by | Account-visible label |
|---|---|---|---|
| 0 | FriendInvitation | friend-invite item (`00a0ba90`) | options label "auto refuse friend" |
| 1 | DuelInvitation | duel item (`00a0d840`) | none |
| 2 | GuildInvitation | guild-invite item (`00a0b5b0`) | options label "auto refuse guild" |
| 3 | HideUi | not an invite; default **true**; the Lua inverts it for an option (`GetAutoRefuseF10`) | none |
| 4 | TradeRequest | trade item (`00a0d320`) | none |
| 5 | HousingInvitation | house item (`00a0b830`) | none |
| 6 | GroupInvitation | group-invite item (`00a0c170`) | options label "auto refuse group" |

When the option is on the builder calls the new item's decline routine immediately, so the sender still gets a decline. Pop-ups show for 60 s and live 300 s for friend, group, guild, house, rename and friend-invite-to-game items; duel, encounter and mini-game announcements use 40 s and 40 s (client Lua `Gaq`). Items are suppressed during the new-player tutorial (`Ui.IsInNPXTutorial`), and an interaction request is refused while a mini-game invitation is pending (`0092efe0`, message key `Interactions.PendingGaqInvitationWarning`).

| Invite | Arrives as | Pop-up responders | Wire reply |
|---|---|---|---|
| Friend | 26/16 | `00a0f460`, `00a0f500`, `00a0f5a0` | 26/17 with a status |
| Group | 40/1 (stores the inviter record at client `+0x6e8`, then `00a87050` to `00a0c390` to `00a0c170`) | `00a090d0` (status 0, decline) and `00a090e0` (status 1, accept) to `00a983b0` | 40/4 with an accept byte; the client then prints the "response accepted" or "response rejected" notice itself |
| Guild | 129/24 (`00b1e7e0`) | `00a0f130` (decline), `00a0f1d0` (accept, then purges the other queued guild invites with `00a0df80`), `00a0f290` (timeout) | 129/9, 129/8, 129/10, each with the guild guid |
| House | housing family (see [[Housing]]) | `00a09550`, `00a09580` | decline 127/16, accept 127/19 (from the Quests guide) |
| Trade, duel, encounter, mini-game | their own families | `00c6dbe0`, `00a97720`, `00a979a0`, `00a97c10`, `00a97cc0` | see [[Trade]] and [[Matchmaking and Lobby]] |

The Lua command `Guild.InviteReply` builds accept and decline packets through a different constructor (`00c0e8f0`, `00c0e950`) that leaves the guild sub-opcode at 0. No shipped script calls it; the pop-up path above is the live one (client Lua grep, `00c10430`).

#### Groups (party)

1. **Invite.** `Ui.InviteToGroup(name)` sends `GroupPacketGroupInvite` (family 40, sub 1) with the target name (`00c24d20`). The Social window passes the friend's name, the mini-game lobby picker passes the chosen friend's name.
2. **Result to the inviter.** 40/2 carries a dword result that the client turns into a notice (`0093daf0` case 2): 0 invite sent, 1 target not found, 3 already in a group, 4 already has an invite, 5 inviter is not the leader, 6 already invited, 7 group full. Code 2 shows nothing (message keys listed in Data and formats). A separate locale line says only the group leader can invite friends.
3. **Invite to the target.** 40/1 arrives, the client stores the inviter's record and raises the GAQ group-invite pop-up (above).
4. **Answer.** Accept or decline sends 40/4 with an accept byte. The server answers the answerer with 40/5, a dword result: 1 no such group, 2 no room, 3 not invited (message keys `GroupAcceptFailNoGroup`, `...NoRoom`, `...NotInvited`). Members are told with 40/4 (accept flag true: the new member record is added, `GroupInviteOtherAccepted`; flag false: `GroupInviteRejected`).
5. **Membership changes.** 40/8 (group update) and 40/9 (member update, one `GroupMemberRecord`) refresh the HUD rows. 40/3 (leave) names the leaving member; if that guid is the local player the group state is torn down and all four group HUD panels are closed (`CombatGroupHandler`, `SoccerGroupHandler`, `VehicleRaceGroupHandler`, `VehicleDemolitionDerbyGroupHandler`, `00a899f0` to `00a89b30`), otherwise a "player left" notice is shown; a group that falls to one member is dissolved. 40/6 (kick) is the same for the kicked member, with "you were removed" or "other removed" notices. The kicker sends 40/6 with the target's guid pair (`00a7e1c0`, `00a7e2e0`); the server answers with 40/7: 1 not in a group, 2 not the owner, 3 target not in group.
6. **Leave.** `GroupScreen.DepartGroup` sends 40/3 (`00a7e110`).
7. **Extras.** 40/11 propagates a renamed player; 40/13 announces an encounter to the group (GAQ item, 40 s, pulsing); 40/14 (client to server, world tunnel variant) tells the group a member is changing jobs and 40/15 relays it with a flag; 40/16 is a map ping (four dwords, drawn by Lua); 40/17 flags that a member is in a mini-game.

HUD columns, 17 per member: PlayerName, PlayerGuid, PlayerJobId, PlayerJobBackgroundIconId, PlayerJobIconId, PlayerJobRank, PlayerStatusMessage, IsGroupLeader (a string: "Solo", leader or member, from locale keys `GroupStatusSolo`, `GroupStatusLeader`, `GroupStatusMember`), InSameMinigame, six numeric parameters (current and maximum health, energy and lives), IsChangingJob, InviteStatus (client Lua `GroupMemberData_DS`). `BaseClient.GroupData` has one column, MinigameType.

Two grouping systems must not be confused. Family 40 is the persistent party. Family 175 (activity launch: invites, member state, ready, kick, owner launch, matchmaking start) is the per-activity lobby roster; it is specified in [[Matchmaking and Lobby]] (vault Piece 020 read its 20 sub-opcodes from the client constructors). A party's members can be invited into an activity, which is why the lobby friend picker and `Ui.InviteToGroup` meet.

#### Guilds

1. **Eligibility and creation.** The server pushes 129/32 (`GuildCanCreateGuildPacket`, one byte) and the data source column `CanCreateGuild` (column 3 of `BaseClient.Guilds`). The Social window shows a sign-up page (`ShowGuildSignup`) with a Create Guild button when the player is in no guild. `Guild.Create()` sends 129/1 with three string fields (the optional name, a locale string read from the client, and a third left at its default; `00c0ff00`) and then formats a local message that substitutes the player's own name (which message was not traced). The creation dialog says the new guild starts with a default name built from the player's name and offers a paid custom name (client Lua `GuildDialog`; a placeholder key `TempGuildName` exists). Client locale gives the rules: any job at level 15 or higher (several text variants), one guild at a time, and error lines for already in a guild, pending invite and maximum size. The wiki says the cap is 100 players and that a new guild's custom name was originally approved by staff.
2. **Naming and approval.** `RequestGuildName` (`00b1e960`) fills a 100-bucket name cache; on a miss it sends 129/14 (`GuildNameRequestPacket`, a guid pair), throttled by a short per-guid age check (the unit is not established), and the server answers 129/18 (`GuildNameUpdatePacket`, guid pair plus name). 129/19 and 129/20 are name accepted and rejected notifications; on a rejection the client flags the cached guild and raises a `GuildRenameRequest` GAQ item (`00b1ee10` case 0x14 to `00a0bd60`), which leads to the rename dialog (`GuildDialog:ShowNameDenied`). A paid rename first sends 129/16 (`GuildPaidRenameCheckRequestPacket`, a name check) and gets 129/30; the rename itself is 129/13 (guild guid, new name, a locale string, and a flag that is false in the China environment).
3. **Invitation.** From the Social window `Guild.Invite(guildGuid, playerGuid, name)` sends 129/3 (two guid pairs and a name string); either the guid or the name is used, the other is 0. The server tells the invitee with 129/24 (`GuildInviteNotificationPacket`) and may cancel with 129/26. The invitee's pop-up answers with 129/8, 129/9 or 129/10 (above). The inviter is told with 129/27 (accepted notice, key `GuildInviteAccepted`) or 129/28 (a boolean: true shows `GuildInviteTimedOut`, false `GuildInviteDeclined`); the client handles both (`00b1d5b0`, `00b1d6c0`) but no constructor sets those ids, so the numbers come from the dispatcher alone.
4. **Membership events.** 129/17 (`GuildMemberStatusUpdatePacket`) carries a status code the client maps to notices and cache edits (`00b1d7d0`): 1 member joined, 2 removed, 3 quit, 4 promoted (with the new rank name), 5 demoted, 6 online or offline by a flag, 7 silent data update. Each of 1 to 5 has a "you" and an "other" notice depending on whether the member is the local player. 129/31 (`GuildPlayerStatusUpdatePacket`, two guid pairs and a flag) refreshes the local player's guild data. 129/22 (`GuildDataFullPacket`) replaces a whole guild record (name cache, role map, member array); 129/25 (`GuildMemberLocationUpdatePacket`) updates 0x18-byte entries (guid, a flag, two coordinates, NaN-checked). 129/21 renames a role. 129/29 announces that the guild was deleted.
5. **Actions.** Promote 129/4, demote 129/5, remove or kick 129/6 (two guid pairs, guild and player; the order inside the body was not verified), quit 129/7, delete 129/2. The Social window quits instead of kicking when the target is the local player. Message of the day: `Guild.SetMotd(guid, text)` sends 129/11; the update comes back as 129/11 (client dispatcher case 0x0b).
6. **Locations.** `00b1ea80` builds 129/15 (`GuildMemberLocationRequest`) from cached members, excluding the local player and only rows with the location flag, and the server answers with 129/25.
7. **Errors.** 129/23 (`GuildErrorPacket`) carries one **string**: the key of a locale message. The client looks the key up and prints it; a missing key prints "GuildString NOT FOUND" (`00b1cd30`). The keys the client ships for this purpose are listed under Data and formats.
8. **Ranks.** The client seeds four default role names from locale keys (`GuildDefaultRankNameLeader`, `...Officer`, `...Member`, `...Recruit`; client locale) and holds roles in a 10-bucket map keyed by integer. The wiki describes Leader and Officer powers; rank numbering and the permission matrix are not in the client (the `GuildNotHighRankTo*` error keys show which actions are rank-gated: delete, rename guild, rename rank, update message, invite, promote, demote, remove).
9. **Guild chat.** Channel 9 on the chat packet, with the guild guid written into the packet (`00b160b0`); the `/guildsay` and `/gu` commands. The chat log gains a Guild filter only while the player has a guild (`SetGuildFilter`).
10. **Admin family.** 131 (`BaseAdminGuildPacket`) has one packet, sub 41, a server-version string (`00b19d70`, dispatcher case 0x29).

#### Chat: sending

1. **Typing.** The chat box (Lua `ChatLog`, `GameDock`) sends the line through `Ui.ProcessChatCommand`. Both paths refuse to send unless `Ui.IsUnderage()` is false, or `Ui.IsOpenChatEnabled()` is true, or the build is the China environment (client Lua). The two account flags are bytes the client reads out of the self-data message (message id 12) just before the membership value (`+0x460` underage, `+0x461` open chat, `+0x464` membership; reader `00944fe0`). Underage accounts use the phrase wheel (`chatHandler:ShowCannedChat`) instead; the client locale has two chat-tutorial lines, one for choosing a phrase and one for typing. The wiki calls these Quick Chat and Safe Chat and says Safe Chat text went through a server-side white-list filter that also blocked digits (wiki, rev 2013).
2. **Command parse.** `ClientChatProcessor` registers 15 names against 9 ids (`00b15450`): say 0; groupsay 1; tell and whisper 2; yell 3; lfg and the number 3 as 4; area and 1 as 5; tradechat and 2 as 6; guildsay and gu as 7; mo and 4 as 8 (client). A plain line is "say". `/tell "Name" text` and `/whisper "Name" text` take a quoted name. The chat-log name menu pre-fills the tell form.
3. **Sender channels.** Each command builds a `PacketChat` (`00b0f8f0`) and writes the channel number the server sees (client; S0336): say 0, tell 1 (`/tell` and `/whisper` both reach it), whisper 3 (a sender exists, `00b16350`, but no command reaches it), group 4, yell 5, trade 6, lfg 7, area 8, guild 9, members-only 10. The members-only sender is also gated on a field of the client object at `+0xc58` (its meaning was not traced; the dispatcher refuses the command when it equals 1).
4. **Client spam gate.** Before sending, the processor applies four limits read from the `ChatSpeed` configuration section: Any 330 ms, Dup 1000 ms, Dup3 2000 ms, Max 30000 ms (`00b15450`; key names read from the binary). A message within Any of the previous send is dropped. Otherwise it compares the text against the remembered recent sends with a character edit distance (`00b12b70`, normalised by length, threshold 0.2); a similar text is counted and is dropped when sent within Dup of the earlier one, within Dup3 once it has matched three times, or within Max once it has matched four times; entries older than Max are forgotten. A dropped message sends nothing and asks the text interface for the key `SpamChatBlocked` (`00b164b0`, client); that key is not in the shipped `CodeStringMappings.txt`, so the notice may be silent (open question). This is advisory: it lives in the client only and is trivially bypassed.
5. **Send path.** `00b15340` serialises the packet (`00b14fa0`) and wraps it in `PacketTunneledClientPacket` (gateway id 5, `0089adc0`), then hands it to the shared submit helper `00925640`. The client does not write the sender's own line into the log at send time (the senders only build and send); own lines come back from the server through the normal receive path, where the Lua color code explicitly tests for the local guid (client Lua `OnReceiveSay`) (likely).
6. **Colors.** `Ui.SetChatColorFgBgSz(fg, bg, size)` stores the sender's text color, bubble color and size and sends a chat-bubble color command (`0092f380`) that also writes the three values into client fields `+0x71c`, `+0x720`, `+0x724`. When a line arrives, `OnReceiveEmote` reads the same three fields from the speaker's entity object (`00901a50`), and `OnReceiveSay` lets the receiving player override a speaker's colors and size separately for friends and others unless "use their colors" is on (user options `frUseTheirColors`, `dfUseTheirColors`). So the server has to keep each player's chosen colors on the entity (R: how they reach other clients is not traced).

#### Chat: receiving

`00b14350` reads the base header (a 16-bit family and a mode) and switches on the mode:

- **Mode 1, `PacketChat`.** Decoded by `00b140c0` to `00b11600`. The receive channel map is: 0 say (`WorldSay`), 1 tell (`OnReceiveWhisper`, `00b11ab0`), 2 system line (`ChatShowSystemMessage` `008edac0`, a "System Message" block), 3 generic channel line (`OnReceiveChannel`, `00b11810`), 4 group (`GroupSay`), 5 yell (`WorldShout`), 6 trade (`WorldTrade`), 7 lfg (`WorldLfg`), 8 area (`WorldArea`), 9 guild (`GuildSay`), 10 members-only (`WorldMembersOnly`) (client). So all eleven channel numbers are pinned by the client's own dispatcher, which supersedes the vault note that pinned only four. After dispatch every line is stored in a per-speaker chat history of the last ten lines (`00b14200`, keyed by a hash of the speaker's name).
- **Mode 2, `ChatPacketEnterArea`.** One dword (an area id). The client prints the "entering chat area" notice into the Area channel (`00b10250`).
- **Mode 3, `ChatPacketDebugChat`.** Length-prefixed fields and a flag; a debug line. Not decoded further.
- **Mode 4, `ChatPacketFromStringId`.** A locale string id with a source entity and a target: the client looks up the string, substitutes "player" and "target" names and up to five arguments, and prints it, routing to `OnReceiveEmote` (`00b0e190` to `00901a50`). It is the only path by which the server can send a localized, composed line (likely).
- **Mode 5, `TellEchoPacket`.** The confirmation to the sender of a tell: the client formats "you whisper to [target]: [message]" locally (`00b120e0`, key `YouWhisperToPlayer`) and logs it as a whisper.

Presentation: `OnReceiveSay` shows a bubble for say, yell, group and quick-chat-say lines and keeps area, trade, lfg, guild and members-only lines log-only; whispers go to the log with a sound (`TellReceived`) (client Lua). A bubble lives 5 s, at most 29 bubbles share a free list. Family 22 (`PacketTargetClientNotOnline`, a name string) makes the client print the "player not online" notice with that name (`009471a0` case 0x16); it is most likely the answer to a tell for an absent target (R). User options in `UserOptions.ini` section `chatOptions` hold per-channel visibility (Area default on, Trade and LFG default off, Members-only on for members), the three color/size/portrait sets (mine, friends, others), log-to-file, auto-complete and language flags (client Lua `LoadChatSetting`). The `BaseClient.Language` data source (columns ID, Location, Language, flags, a Filtered column) feeds the language picker, and the chat packet carries a language integer (meaning of the filter not traced).

#### Quick chat and emotes

Family 67 (`BaseQuickChatPacket`) has three sub-packets (client; `00b966e0`): sub 1 delivers the phrase table, sub 2 is a tell by phrase, sub 3 is chat to a channel by phrase. The table is a hash of rows keyed by integer (1000 buckets, 0x3c-byte nodes; counted-record reader `00b90750`). Client Lua `ChatLog:AddQuickChatPhrases` reads, per row, column 0 as row id, column 1 as menu parent, column 2 as the display string id, column 3 as an optional "say" string id and column 4 as an animation id (the row data is server-supplied and not in the client, so the row count, the menu tree and the animation ids are unknown). The client sends only a row id and a channel: `ProcessQuickChatCommand` sends channel 0, the group variant channel 4, the guild variant channel 9 with two extra dwords (the guild guid), and `ProcessQuickChatChannelCommand` maps `WorldArea` 8, `WorldTrade` 6, `WorldLfg` 7, `WorldMembersOnly` 10 (`00b97380`, `00b97480`, `00b97580`, `00b978b0`). A phrase tell (`00b976a0`) sends sub 2 and echoes locally with `ChatLog:EchoWhisperFromQuickChat`.

**Emotes are quick-chat rows.** The Lua `EmoteHandler` binds 72 slash commands (`/wave`, `/dance`, `/cheer` and so on; one command, "curious", is bound twice and the later id wins) to `Ui.ProcessQuickChatCommand(id)`; the emote menu button toggles the same phrase wheel in emote mode (`toggleEmotes`). The animation comes from column 4 of the row and the animation slot named in `AnimationTypes.xml` (`Emotes` branch; vault 093). Restrictions that the client data carries: the wielded item is hidden while an emote plays, two emotes freeze movement, one disables bone masking, and the emote button is disabled while mounted (vault 093, from the client Lua and `AnimationTypes.xml`). The shipped `EmoteItemCategories.txt` has a header and no rows, so the item-gated emote categories are filled from server data (client resource).

#### Reporting, inspection, other surfaces

- **Report.** The chat-log name menu and the console command `/report username` call `ReportPlayerRequest` (`0093b730`, console `00a9ea40`). It looks up the reported player's stored chat history (the last ten lines, up to ten entries, each a pair of integers and a string), builds `ReportPacketReportPlayer` (family 68, sub 1: the reported name string plus the history lines) and sends it through the client tunnel. The client then prints "player reported", or "player not reported" when it holds no history for that name. A house report is 68 sub 3.
- **Inspect.** Choosing inspect on another player makes the server send 149/1 (`StartInspectPacket`), which makes the client build an `InspectProxy` pedestal for the target; 149/4 (`WorldInspectReplyPacket`) delivers the data behind `BaseClient.Inspect` (30 columns: name, jobs unlocked, levels gained, active profile, VIP rank, coins, collections and quests counts, maximum health and energy, defense, star and coin boosts, wallet, title, achievement points, plus China-only fields), `.Inspect.Jobs` and `.Inspect.Items`. The panel has Equipment, Misc and About tabs. Closing it sends 149/2 (`StopInspect`, `009347d0`). The request that starts an inspect from the client side was not identified (open question).
- **Telemetry.** Typed chat lines are also passed to `Ui.SetWallOfData("ChatLog", "sendChatMessage", text)` (client Lua), which feeds the "Wall of Data" UI-event telemetry (family 194; `WallOfDataEventTypes.txt` enables four event types). Whether the `ChatLog` table is one of the enabled types was not traced.

#### The card-service stack

Friends, ignores, guild and chat in the card executable are command objects over a direct TCP link (client; S0724, S0726, S0727, S0751, S0778, S0779, S0785, S0794, S0795). Friend flow: `UChatCommand_RequestAddFriend` (489), `RequestFriendConfirm` (491), the receive events `OnReceiveFriendConfirmRequest` (497) and `...Response` (498), `RequestRemoveFriend` (494), `RequestSetFriendComment` (496), presence `OnReceiveFriendLogin` (499) and `...Logout` (500), and a "reciprocate" set (509 to 513). Status lists are `RequestFriendStatus` (492) and `RequestIgnoreStatus` (493); the friend list is sent **zlib-compressed with a bound-sized zero-padded buffer** (`012`-prefixed serializer `013ba330`; the real compressed length is ignored), the ignore list uncompressed. Elements are polymorphic (`FriendStatus` 505, `IgnoreStatus` 506). Chat is `LobbyCommand_SendText` (97, lobby) or `LobbyCommand_SendUChatText` (446, to a conversation id; `Chat::ChatChatInput::SubmitChatLine` `01282970` sends 97 when the conversation id is below 1 and 446 otherwise, and lines starting with `/` run local commands); `LobbyCommand_SendUChatError` (445) and ten `Tcg_error.uChat*` messages cover delivery errors (address missing, destination avatar missing, ignoring, remote server down, timeout, unknown). Lobby ignores have their own commands (110, 111, 281, 285, 286). The guild commands are classes 417 to 429 and 461 to 468 plus the cached `Guild` object (class 388, 0x54 bytes, pushed by id), `GuildCommand_Chat` (464) and `AskToJoinGuild` / `AskToJoinGuildResponse` (424, 425), all routed with the guild routing word. Polls (391 to 394) and user data (542 and up: player info, awards, honors, avatars, news) ride the same link. See [[Trading Card Game]] for framing and routing.

### Data and formats

**Shared encodings (client; readers `00c7ef80`, `008e7290`, `00911c30`).** Strings are an `i32` length followed by the bytes. A guid is a pair of dwords (a 64-bit value). Name data (`NameData`) is three `i32` followed by two length-prefixed strings (first and last name). Most readers are bounded and set a failure flag instead of reading past the end. Family headers differ: chat, quick chat, guild and group read a 16-bit family and a 16-bit sub-opcode; friends and ignore read a 16-bit family and a signed **byte** sub-opcode (`00c7ef80`).

**`PacketChat` (family 15, mode 1), in writer order (`00b14fa0`).** `u16` family 15 and `u16` mode 1; `u16` channel; sender guid (`u64`); target guid (`u64`); sender name data; target name data; message (`i32` length and bytes); four dwords (a position vector, the constructor seeds them from a global default); guild guid (`u64`); language (`i32`); and, only when the channel is 8 (area), one more `i32` (area id). Field names are ours; the order and widths are from the writer. The receive reader (`00b11600`) is the mirror (client).

**Friend row (`FriendData`, family 74 sub 1 rows and sub 6; reader `00c7fd50`).** Name data; guid (`u64`); byte A; if A is 0, a `u64`; otherwise byte B, a seven-dword block (reader `00c7f460`, the status block), when B is 0 two floats (X and Z, NaN-checked), a dword and a byte; then, for both branches, a final byte. The `FriendsList` data source has 18 columns: FriendName, LocationX, LocationZ, Guid, Online, Online_Status, Profile_Name, Profile_Rank, Profile_Icon_ID, Profile_Background_Image_ID, Popularity, In_Encounter, Is_Local, Status, HighScore, Current_Mini_Game_Type, Current_Mini_Game_Name, Is_In_Static_Zone (client Lua). Which wire field fills which column is not recovered. By shape, A is most plausibly the online flag (an offline row carries only a `u64`), B the in-encounter flag (position only when not in an encounter), the dword the popularity and the last two bytes the local and static-zone flags (R).

**Other family 74 and 112 bodies (readers `00c7efe0` to `00c80500`).** Sub 2 (online): name string, guid, flag. Sub 3 (offline) and 7 (remove): guid. Sub 10 (rename): name string and guid. Sub 9 (status): name string, guid, seven dwords. Sub 8 (message): a signed byte code, a guid and two name-data blocks. Sub 5 (positions): count and 0x28-byte entries (fields not recovered). Ignore: sub 1 an `i32` count and records, sub 2 one record, sub 3 a guid; an ignore record is a guid (two dwords) followed by a name string (`00c7f590`).

**Group records.** Invite: the group id, the inviter's guid pair and names, and trailing dwords (member record built by `008dcd60`, fields `+0x90` to `+0xa8`). Reply codes are one dword (`00908fd0`). Accept carries an accept byte (default 1). Kick and leave carry a guid pair and the member name data. Change-job: one byte. Map ping: four dwords. In-mini-game and changing-job update: two dwords and a byte (`008b75d0`, `008b7400`). Widths of the member record fields beyond these are not decoded.

**Guild bodies (family 129).** Create: three strings. Invite: two guid pairs and a string. Promote, demote, remove: two guid pairs. Quit, accept, decline, timeout, name request: one guid pair. Motd: guild guid and a string. Rename: guild guid, two strings and a flag byte. Paid-rename check: guild guid and a name. Member location request: a guid pair and an array of 64-bit guids. Server side: member status (guild guid, member guid, name data, status and rank fields), name update (guid pair, string), role rename (guid pair, two dwords), player status (two guid pairs and a flag), can-create (one byte), error (one string), invite notification (inviter query then pop-up), admin version (one string). Full data: a leading identifier pair, bounded fields, a counted list, and the role map (10 buckets, 0x28-byte nodes); member entries are 0x18 bytes (guid, flag, two dwords). The exact field names for the full record are not recovered.

**Data source columns (client Lua).** `BaseClient.Guilds`: GuildGuid, Name, MessageOfTheDay, CanCreateGuild (the binary knows a fifth, `CanRenameGuild?`). `BaseClient.GuildMembers` (15): Guid, Name, Rank, Online, WorldId, ProfileName, ProfileIconID, ProfileRank, ProfileBackgroundImageID, InEncounter, HasLocation, XLocation, ZLocation, IsLocal, IsFriend. `BaseClient.Acquaintances` (5) and `BaseClient.IgnoreList` (one name column used by the Lua). `BaseClient.GroupMemberData` (17, above). `BaseClient.QuickChatData`, `BaseClient.EmoteItemCategories` (ID, STRING_ID) and `BaseClient.QuickChat.AutoComplete` (QuickChatNodeId, StringId, String, IsExactMatch). `BaseClient.Language` (the `LocaleDsColumn` set: ID, LanguageID, Location, Language, LocationFlag, LanguageFlag, LocationCode, LanguageCode, Filtered, IsDefault).

**Status and result tables (client).**

| Where | Value | Meaning (message key) |
|---|---|---|
| 74/8 code | 1 to 11 | `FriendAddRequestAccepted`, `FriendAddRequestDeclined`, `FriendAddRequestTimedOut`, `NoTeleportToFriendCombat`, `FriendAddRequested`, `NoTeleportFriendOffline`, `TeleportedCombat`, `NoFriendSpam`, `FriendNameChanged`, `NoTeleportToPlayer`, `NoTeleportToFriendMiniGame` |
| 40/2 result | 0, 1, 3 to 7 | `GroupInviteSent`, `GroupInviteFailNotFound`, `...AlreadyGrouped`, `...AlreadyHasInvite`, `...NotLeader`, `...AlreadyInvited`, `...GroupFull` |
| 40/5 result | 1, 2, 3 | `GroupAcceptFailNoGroup`, `...NoRoom`, `...NotInvited` |
| 40/7 result | 1, 2, 3 | `GroupKickFailNotInGroup`, `...NotOwner`, `...TargetNotInGroup` |
| 129/17 status | 1 to 7 | joined, removed, quit, promoted, demoted, online or offline (flag), silent update |
| 129/23 key strings the client ships | | `GuildNameNotValid`, `GuildNameAlreadyExists`, `GuildNameRequestPending`, `GuildCantRenameAtThisTime`, `GuildMotdTooLong`, `GuildMemberCountExceeded`, `GuildInMaxGuilds`, `GuildInviteeInMaxGuilds`, `GuildInvitePlayerNotFound`, `GuildCreateRequirementsNotMet`, `GuildPendingInviteTimedOut`, `GuildPromoteAtMaxRank`, `GuildPromoteAtMinRank`, `GuildPromoteCantPromoteAbove`, `GuildPromoteCantDemoteAbove`, `GuildRemoveCantRemoveAbove`, and the `GuildNotHighRankTo...` set for Delete, Rename guild, Rename rank, UpdateMotd, Invite, Promote, Demote and Remove |

**Message keys by group (client locale, `CodeStringMappings.txt`).** Friends: `FriendAdded`, `FriendRemoved`, `FriendOnline`, `FriendOffline`, `FriendAlreadyOnList`, `FriendYourself`, `NoOnlineFriends`. Ignore: `IgnoreAdded`, `IgnoreRemoved`, `Ignored`. Chat: `ChatChannelSelected`, `NoChatChannelSelected`, `EnteringChatArea`, `UsageWhisper`, `YouWhisperToPlayer`, `PlayerNotOnline`, `PlayerReported`, `PlayerNotReported`, `GuildChat0001...` and `GuildChatSuffix0001...` (phrase fragments). The code-string ids in that file are not keys into the locale table's id column (the table uses its own ids), so key-to-text pairing in this guide is by key name and wording only.

**Client resources.** `EmoteItemCategories.txt` (header only), `CodeStringMappings.txt` (the keys above), the `chatbubble_border`, `_fill` and `_innershadow` textures (nine styles), `SocialProfileLikesDefinitions.txt`, `SocialProfileReputationDefinitions.txt`, `SocialWallEntryDefinitions.txt`, `SocialWallTagIcons.txt`, `SocialWallTagStrings.txt` (all header only: the social-profile and social-wall feature ships with empty tables), `WallOfDataEventTypes.txt` (four rows). Chat color names in the script are White, Black, Pink, Green, Blue, Red, Purple, Yellow, mapped to bubble styles.

### Network

Unless stated, packets are tunnelled; a sender that wraps a body in `PacketTunneledClientPacket` uses gateway id 5 (`0089adc0`) and one that uses `PacketTunneledClientWorldPacket` uses gateway id 6 (`0089ae70`) (see [[Gateway and Transport]]). Directions are proven when a sender builds the packet (C>S) or a dispatcher case parses it (S>C). The packet catalogue the validator uses lacks most of these classes, so the map's `wire` field is set on only 92 of the 1,142 functions.

| Fam/sub | Name (client RTTI) | Dir | Body | Handler or sender |
|---|---|---|---|---|
| 15/1 | PacketChat | both | see Data and formats | send `00b15340` (tunnel 5); read `00b140c0`; dispatcher `00b14350` |
| 15/2 | ChatPacketEnterArea | S>C | one `u32` area id | `00b0db30`, `00b109b0`, `00b10250` |
| 15/3 | ChatPacketDebugChat | S>C | length-prefixed fields, flag | `00b0f9d0`, `00b10a20` |
| 15/4 | ChatPacketFromStringId | S>C | string id, entity refs, scalar fields | `00b0e0f0`, `00b10a90`, `00b0e190` |
| 15/5 | TellEchoPacket | S>C | target name data, message | `00b0fa40`, `00b14130`, `00b120e0` |
| 22 | PacketTargetClientNotOnline | S>C | name string (32 bytes max) | `008dc2f0`, `009471a0` case 0x16 |
| 26/14 | AddFriendRequest | C>S | candidate name | `0093b9b0` (tunnel 6) |
| 26/15 | RemoveFriendRequest | C>S | two-string name payload | `00c24a00` |
| 26/16 | ConfirmFriendRequest | S>C | requester record, a byte | `00a9de90`, `00aa13b0`, `00aa2560` case 0x10 |
| 26/17 | ConfirmFriendResponse | C>S | guid pair, status, trailing name | `00a0a500`, `00a0f330` |
| 26/21 | FriendsPositionRequest | C>S | none | `00c23e50` |
| 26/30 | IgnoreRequest | C>S | flag byte, two-string name payload | `00c24b00`, `00c24c00` |
| 40/1 | GroupInvite | both | by name (C>S); inviter record (S>C) | `00c24d20`; `0093daf0` case 1 |
| 40/2 | GroupInviteReply | S>C | `u32` result | `0093daf0` case 2 |
| 40/3 | GroupLeave | both | member guid pair and name data | `00a7e110`; case 3 |
| 40/4 | GroupAccept | both | accept byte, member record | `00a983b0`; case 4 |
| 40/5 | GroupAcceptReply | S>C | `u32` result | case 5 |
| 40/6 | GroupKick | both | target guid pair, member data | `00a7e1c0`, `00a7e2e0`; case 6 |
| 40/7 | GroupKickReply | S>C | `u32` result | case 7 |
| 40/8, 40/9 | GroupUpdate, GroupMemberUpdate | S>C | group data; one member record | cases 8, 9 |
| 40/11 | RenamePlayer | S>C | member record | case 0xb |
| 40/13 | AnnounceEncounterReply | S>C | two dwords | case 0xd |
| 40/14, 40/15 | GroupChangingJobPacket, GroupChangingJobUpdatePacket | C>S (tunnel 6), S>C | a byte; two dwords and a byte | `00c0b300`, `00c0b1d0`; case 0xf |
| 40/16 | GroupMapPingPacket | both | four dwords | `00938650`; case 0x10 |
| 40/17 | GroupInMinigamePacket | S>C | two dwords and a byte | case 0x11 |
| 67/1 | QuickChatSendDataPacket | S>C | count, then keyed rows | `00b8ae40`, `00b90750`, `00b966e0` |
| 67/2 | QuickChatSendTellPacket | both | base fields, target, phrase row | `00b90970`, `00b976a0`, `00b97210`; case in `00b966e0` |
| 67/3 | QuickChatSendChatToChannelPacket | both | base fields, `i16` channel, row dwords | `00b90a10`, `00b97100`, `00b95450` |
| 68/1, 68/3 | ReportPlayer, ReportHouse | C>S | reported name, up to ten history lines; house report | `0093b730` (tunnel 5); `00ac71a0` |
| 72/1, 72/2, 72/3 | Acquaintance add, remove, online | S>C | record; name string; guid and flag | `00a1bb70`, `00a19170`, `00a16bb0`, `00a12160` |
| 74/1 to 74/10 | FriendList, Online, Offline, Positions, Add, Remove, Message, Status, Rename | S>C | see Data and formats (sub 4 unused) | `00c814d0` |
| 112/1, 112/2, 112/3 | IgnoreList, IgnoreAdd, IgnoreRemove | S>C | see Data and formats | `00c81250` |
| 129/1 to 129/7 | GuildCreate, Delete, Invite, Promote, Demote, Remove, Quit | C>S (tunnel 6) | see Data and formats | Lua module `Guild` (`00c0ff00` to `00c107c0`), wrappers `00c0f3b0` to `00c0f830` |
| 129/8, 129/9, 129/10 | InviteAccept, Decline, TimeOut | C>S | guild guid pair | `00a0f1d0`, `00a0f130`, `00a0f290` via `00a0ee70`, `00a0ed50`, `00a0ef90` |
| 129/11 | MotdUpdate | both | guid, string | `00b19560`, `00c10250` |
| 129/13, 129/14, 129/15, 129/16 | Rename, NameRequest, MemberLocationRequest, PaidRenameCheckRequest | C>S | see Data and formats | `00c10130`, `00b1e960`, `00b1ea80`, `00b1e600` |
| 129/17 to 129/22, 129/24 to 129/26, 129/29 to 129/32, 129/23, 129/27, 129/28 | server notifications | S>C | see Data and formats | dispatcher `00b1ee10` |
| 131/41 | AdminGuildServerVersionPacket | S>C | string | `00b19d70`, `00b1abb0` |
| 149/1, 149/4 | StartInspectPacket, WorldInspectReplyPacket | S>C | target guid and data; inspect data | `009471a0` case 0x95; `008b7790` |
| 149/2 | StopInspectPacket | C>S | none | `009347d0` |
| 194/4 | Wall-of-data UI event | C>S | event row and text | `00adcd60` to `00ade7f0` |

The card-service link uses command classes instead (listed in the card-service paragraph above and in [[Trading Card Game]]). The activity-launch family 175 and the housing invites are specified in their own guides.

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour. None of this has been run against the shipped client; the vault's friend, chat, party, guild and emote pieces all state that no native acceptance exists.

1. **Own all social state per character:** friend list (rows keyed by guid), ignore list, acquaintance list (cap 20 on the client; server cap (R)), guild membership (one guild at a time), group membership (volatile), chat colors and sizes, quick-chat table, auto-refuse is client-side only.
2. **Login order.** The social families are ignored until the self-data message (id 12) is read, so send that first. It must include the two account flags (underage and open-chat; bytes before the membership value in the player-data reader `00944fe0`). Then send the friend list (74/1), the ignore list (112/1), the guild record (129/22 and 129/32) and the quick-chat table (67/1). The order of these lists after the self-data is our choice (R); the client was not shown to require it. Whether the original sent any other family-129 reply during login is unknown.
3. **Friends.** On 26/14: reject self, duplicates, ignored requesters (R) and floods (reply 74/8 code 8); reply code 5 to the requester; send 26/16 to the target; on 26/17 accept send 74/6 to both sides and code 1 to the requester, on decline code 2, on no answer code 3 (the pop-up lives 300 s, R). On 26/15 remove and send 74/7. Send 74/2 and 74/3 presence changes to online friends only (R), 74/9 and 74/10 for status and rename, 74/5 for position requests. Teleport refusals use codes 4, 6, 7, 10, 11. Friend cap and mutual-removal semantics are unknown (R).
4. **Ignore.** On 26/30 update the list and send 112/2 or 112/3. Enforce the list on the server: drop chat and whispers from ignored senders, auto-decline their invites (the client already does this for guild and trade), keep them out of your house (wiki).
5. **Groups.** Implement the leader rules the error table implies: only the leader invites (code 5), one group per player (3), one pending invite per target (4, 6), size limit (7, the number is not in the client (R)), accept validity (5/1 to 5/3), kick by the leader only (7/1 to 7/3). Fan out 40/3, 40/4, 40/6, 40/8, 40/9 to members; keep 40/9 cadence modest (health and energy rows; R). Relay 40/14 to 40/15, 40/16 and 40/17. Groups are volatile: dissolve at one member. Expire invites (R).
6. **Guilds.** Enforce level 15 in any job and one guild per player (client locale). Create flow: validate, answer 129/32, push 129/22 to the founder and 129/17 code 1 to members. Name approval and paid rename: 129/14 to 129/18, 129/19 and 129/20, 129/16 to 129/30, 129/13 (R: queue, review state, token). Invite flow: 129/3 to 129/24 (and 129/26 on cancel), 129/8 to join (129/17 code 1 to members, 129/22 to the joiner), 129/9 or 129/10 to close the invite (129/27 and 129/28 notify the inviter). Rank actions 129/4 to 129/7 with the `GuildNotHighRankTo...` rules and the "can't act above" rules; answer failures with 129/23 and one of the shipped key strings (a key not in the client's table prints "GuildString NOT FOUND"). Cap at 100 members (wiki) and say so with `GuildMemberCountExceeded`. Provide member locations (129/15 to 129/25), MOTD (129/11) and delete (129/2 to 129/29). Rank names and numbering are ours (R) but must be four ranks to match the shipped names.
7. **Chat routing (R for every recipient rule).** Accept 15/1 only from a connected, spawned player. Validate the channel: 0 say to nearby players, 5 yell to the zone, 4 group members, 9 guild members using the claimed guild guid checked against membership, 10 members only (check membership), 6 trade and 7 lfg to the players the channel scope covers (scope unknown (R); the show or hide toggle for each channel is a local user option, `setShowChannel`, so the server never learns it), 8 area with the area id in the trailing field. Never trust the sender guid, sender name or guild guid in the packet; rebuild them from the session. Echo the sender's line back through the normal path (the client does not print it itself, likely). Clamp message length (the packet has none) (R).
8. **Tell.** Resolve the target name; if offline or unknown, send family 22 with the name; otherwise deliver 15/1 channel 1 to the target and 15/5 to the sender (the client builds the sender's "you whisper" line from it). Honour ignore: an ignored sender gets no delivery (R: and whether the sender is told is unknown).
9. **Chat safety the client expects.** (a) For accounts with the underage flag and no open-chat flag, refuse typed 15/1 on the server, because the client check is trivially bypassed; allow only quick chat (67/2, 67/3) (R, matches wiki: Quick Chat shows no user text and Safe Chat is white-listed). (b) Run a server-side white-list filter on typed text, including blocking digits as the wiki says; the client contains no word list (wiki; R for the exact rules). (c) Mirror the client spam gate (330/1000/2000/30000 ms, similarity 0.2) as a floor, not a substitute (client). (d) Keep a per-sender history of the last ten lines so that a report (68/1) can be checked against it; the client also sends the lines. (e) Treat 68/1 reports as moderation input: store the reported name and lines, do not act automatically (R). (f) Do not store chat text received through the telemetry family 194; accept and discard it (R). (g) Filter guild names and rename requests; the client expects 129/19 and 129/20 and a pending state (`GuildNameRequestPending`).
10. **Quick chat and emotes.** Send the phrase table (67/1) with the rows our own content defines (the emote rows with their animation ids). On 67/3 and 67/2 validate the row id against the table and the channel against the rules above, then relay a 67/3 or 67/2 to recipients; the client alone resolves text and animation from the row. Enforce emote restrictions server-side where they matter (mounted, frozen-movement emotes) (vault 093, R). Do not invent a motion for the one animation id that has no slot (open question).
11. **Colors.** Store the colors and size from the chat-bubble color command per character and make them visible to other clients with the entity data the client reads (`+0x71c` to `+0x724` of the speaker entity). How the original made them visible is not recovered.
12. **Mode 2 and mode 4.** Send 15/2 when the player enters a named chat area (the client prints the notice and uses the area channel); use 15/4 for any server-composed localized line.
13. **Report and inspect.** Accept 68/1 and 68/3; for inspect, answer the unknown start request with 149/1 and 149/4 carrying the player's public data (the columns above) and accept 149/2. Public data policy (what a stranger may see) is ours (R).
14. **Invites.** Never send an invite the client cannot display: friend (26/16), group (40/1), guild (129/24). Expect the client to answer on its own timers; make the server's timeouts longer than the pop-up's 60 s display (R). Do not send invites during a player's tutorial (the client drops them).
15. **Card service.** If the card feature is revived it needs its own listener for the command stack (friend, ignore, chat, guild, polls) as described in [[Trading Card Game]]; the world server does not implement it.

### Evidence

- **Client.** 1,142 accepted functions in 52 shards (S0233 to S0795): 593 confirmed, 283 likely, 143 speculative, 123 unknown, 92 with a `wire` link. Subsystems: `net.guild` 324, `net.chat` 271, `net.youtube` 109, `net.sns` 71, `net.poll` 48, `game.friends` 46, `net.friends` 50, `ui.friends` 35, `game.guild` 32, `ui.chat` 30, `ui.group` 29, `net.rating` 27, `ui.quickchat` 24, `game.social_share` 23, `net.portrait` 9, `net.group` 8, `net.acquaintance` 4, `net.profile` 2. The table also holds the card-service friend, ignore, guild and chat commands (S0724 to S0795), which are a separate stack. Reads in this pass beyond the table: the main dispatcher `009471a0` cases for 15, 22, 40, 72, 74, 112, 129, 131, 149; the friend, ignore and group dispatchers `00c814d0`, `00c81250`, `0093daf0` (shards S0249, S0402, S0403, which the table does not include); the command dispatcher `00aa2560` case 0x10; the chat receive switch `00b14350` and every channel branch; the spam gate `00b164b0`; the writer `00b14fa0`; the friend, ignore and group result tables; the AutoRefuse table; the guild status mapper `00b1d7d0`; the report builder `0093b730`; the player-data reader fragment that sets the underage and open-chat bytes.
- **Client Lua and resources.** `ScriptsBase.lua` supplies the data-source columns, `SocialHandler`, `ChatLog`, `Gaq`, `GroupHandler`, `EmoteHandler`, `PlayerInspect`, the chat options and the underage gate; `CodeStringMappings.txt`, `EmoteItemCategories.txt`, the empty social-profile tables and `WallOfDataEventTypes.txt` are shipped files.
- **Client locale.** The guild eligibility text (level 15, one guild, maximum size, pending invite), the chat tutorial lines, the group-leader rule and the guild rank-change lines were read with the content query tool.
- **Video.** The reviewed frame records show the friends overlay ("My Friends"), the guild panel with "not in a guild" and a Create Guild button, a group-invite prompt during the kart tutorial, and the quick-chat wheel; player names, friend names and chat were not recorded. The substring query for "Friends List" returned the tool's 40-record limit, "Ignore" 18, "Emote" 22, "Quick Chat" 7, "Group Invite" 5, "Create Guild" 1, "Add Friend" 2 and "Whisper" 1 (not curated). Video does not show message wording or timings, so it adds no protocol evidence.
- **Wiki.** Chatting (rev 2013-07-11: Quick Chat versus Safe Chat, white-list filter, digits filtered, slash commands, channel list), In-Game Commands (rev 2021-11-20), Guilds (rev 2018-01-24: level 15, 100 players, staff approval of names, paid rename scroll), Groups (rev 2021-11-21), Ignore Button (rev 2013-07-11), Interface (rev 2017-10-01). The wiki has no Friends article.
- **Vault.** Used only for client-derived facts: Pieces 087 (friend and command sub-opcodes read from the client constructors), 088 (channel names, command registration table, chat spam values), 089 (the 15 group sub-opcodes from the client constructors), 090 (guild sub-opcodes and constructor widths, data source columns), 093 (emote command table from the client Lua, animation slots from `AnimationTypes.xml`), and 020 (activity-launch sub-opcodes). Not used: piece 096 (login ordering) and the Kart entry note's 5-byte family-129 block, whose only evidence is recorded server captures; every byte layout, role id, recipient rule and reconstructed registry in those notes whose source is a community packet model, the community enum numbers for guild sub-opcodes 23, 27 and 28 (the client handles them but its constructors do not set them), and the legacy gateway oracle.
- **Verified versus reconstructed.** Verified from client bytes: family and sub-opcode numbers, packet directions where a sender or dispatcher shows them, the chat channel numbers, the field order of `PacketChat` and the friend, ignore and group result tables, the AutoRefuse table, the spam-gate numbers, the Lua-visible data source columns, the guild status mapper and the GAQ responders. Reconstructed (R): every server rule, cap, timeout, recipient set, rank permission and the meaning of unlabelled fields.
- **Native acceptance.** None. No friend, group, guild, chat or emote traffic has been exercised against the shipped client, and no original-server traffic of these families is used.

### Open questions and contradictions

- **No original server code.** Every server rule above is inferred from the client's requirements. Caps (friends, ignores, group size, 100 guild members from the wiki), timeouts, recipient rules and permission matrices are not in the client.
- **GAQ guard resolved.** The earlier note that the guard `00917b60` is unrecovered is superseded: it is the client AutoRefuse option read from the user configuration (seven categories, defaults off except index 3). Quests guide statement refined, not contradicted.
- **Friend-message sub-opcode.** The vault saw the `FriendMessagePacket` constructor set sub 0 (an outside enum number it also quoted is not used). The client dispatcher routes sub 8 to that class (`00c814d0` case 8) and the constructor's 0 is just its default, so 8 is what the client expects on the wire.
- **Guild accept and decline numbers.** The Lua command constructors leave the sub-opcode at 0 (S0386 flagged a contradiction with 8 and 9). The pop-up responders (`00a0f1d0`, `00a0f130`) use ids 8 and 9 and are the path the shipped script uses. Sub-opcodes 27 and 28 (acceptance and decline notices) are handled by the dispatcher but no constructor sets them.
- **Chat channel numbers.** All eleven (0 to 10) are now pinned by the receive switch; the vault's "only four are proven" is outdated. Whether the server may send channel 2 and channel 3 lines is open: the client handles them (system line, generic channel line) but no sender uses them.
- **Mode 3 and mode 4 chat bodies** are only structurally known; the exact fields of `ChatPacketDebugChat` and `ChatPacketFromStringId` are not decoded.
- **Teleport to friend.** The refusal codes in 74/8 prove a teleport request, and `Ui.TeleportToFriend` is called, but the sending packet was not identified. A command with sub-opcode 45 (`00c23a20`, a Lua guid argument) is a candidate.
- **Inspect request.** The client-side start of an inspect (menu item, packet) is unidentified; only 149/1, 149/2 and 149/4 are known.
- **Chat-bubble color delivery.** The client reads the speaker's colors from the speaker's entity; how other clients get them is not traced.
- **Wall-of-data telemetry.** Typed chat text is passed to the telemetry call; whether the `ChatLog` events are among the four enabled types is open. Do not log the text.
- **Spam notice.** The key `SpamChatBlocked` used by the client spam gate is absent from `CodeStringMappings.txt`; whether it resolves from another table is open.
- **Message length.** The packet has no length limit and the chat box limit lives in the Flash movie; the maximum is not recovered.
- **Safe Chat filter.** The client has no word list that we found, and the white-list filter and digit blocking come from the wiki. Whether any filtering runs in the chat input Flash movie (which we cannot read as script) is open.
- **Language field.** The chat packet carries a language integer and the language list has a Filtered column; their effect on delivery is not traced.
- **`isRef` flag.** The chat callbacks receive `isNPC`, `isRef` and a friend flag; `isRef` is passed through to the log and its meaning (possibly a staff marker) is not recovered.
- **Name-change family.** The guild paid-rename flow reaches the shared `PaidNameChange` UI (`NameChange` family 192); its sub-opcodes belong to [[Character and Customization]] and were not re-derived.
- **Acquaintance rules** (who is added, the "Type" and "how long ago" fields) are unknown.
- **Card-service questions.** Whether the card service shared friends and guilds with the world server; the UChat routing word for friend commands (the default word is used by every UChat class); the unused copies of `FriendStatus` and `IgnoreStatus` vtable slots.
- **Speculative and mis-grouped rows.** `net.group` holds three unlabelled tunnel senders (`00a7dd50`, `00a7dea0`, `00a7dfc0`). `net.youtube`, `net.sns`, `net.poll`, `net.rating`, `net.portrait` and `game.social_share` are not part of friends, guilds or chat and stay unspecified here. The SNS integration targets a regional microblog and a hard-coded host (S0366) and should be stubbed, not implemented. The YouTube uploader speaks a retired API (S0583).
- **Stub and duplicate code.** Three identical `ConfirmFriendResponse` senders and six byte-identical empty guild destructors (`00b172e0` to `00b17470`) cannot be tied to owners from their own bytes.

### Key functions

| Address | Role |
|---|---|
| `009471a0` | Main inbound dispatcher; routes 15, 22, 40, 67, 72, 74, 112, 129, 131, 149 |
| `00b14350`, `00b140c0`, `00b11600` | Chat receive dispatcher; `PacketChat` reader |
| `00b14fa0`, `00b15340` | `PacketChat` writer; send helper with tunnel |
| `00b15450`, `00b164b0` | `ClientChatProcessor` constructor; command and spam-gate handler |
| `00b15710`, `00b16220`, `00b15f50`, `00b15870`, `00b159d0`, `00b15b30`, `00b15df0`, `00b160b0`, `00b15c90` | Channel senders (say, tell, group, yell, trade, lfg, area, guild, members-only) |
| `00b11ab0`, `00b11810`, `00b120e0`, `00b14200`, `008edac0` | Whisper and channel receive, tell echo, chat history, system line |
| `00b10250`, `00b0e0f0`, `00b0e190`, `00901a50` | Enter-area notice, from-string-id chat, emote bridge |
| `00b96db0`, `00b966e0`, `00b97380`, `00b976a0`, `00b978b0` | Quick-chat manager, receive handler, channel and tell senders |
| `00c814d0`, `00c81250`, `00a1bb70` | Friend, ignore, acquaintance dispatchers |
| `00cb47a0`, `00cb4950`, `00cb4da0`, `00cb4ee0`, `00cb4fc0`, `00cb3fd0` | Friend online, offline, add, remove, list; status message map |
| `0093b9b0`, `00c24a00`, `00c23e50`, `00c24b00`, `00c24c00` | Add friend, remove friend, positions, ignore add, ignore remove |
| `00aa2560`, `00a9de90`, `00a0f330` | Command dispatcher (26/16), confirm request, confirm response send |
| `00cf9c90`, `00cf8700`, `00cf8e90`, `00cf8b10` | Ignore data source, lookup, add, remove |
| `00917b20`, `00917b60` | AutoRefuse setter and guard |
| `00a0c170`, `00a0b5b0`, `00a0ba90`, `00a0d320`, `00a0d840`, `00a0b830` | GAQ group, guild, friend, trade, duel, house item builders |
| `00a090d0`, `00a090e0`, `00a983b0` | Group invite responders and the 40/4 sender |
| `0093daf0` | Group packet dispatcher |
| `00c24d20`, `00a7e110`, `00a7e1c0`, `00a7e2e0`, `00c0b300`, `00938650` | Group invite, leave, kick, change-job, map-ping senders |
| `00a7dc80`, `00a7dbe0`, `00a7c9f0`, `00a899f0` to `00a89b30` | Group data sources, row refresh, HUD open and close |
| `00b1ee10`, `00b1d7d0`, `00b1cd30`, `00b1e7e0`, `00b1e960`, `00b1ea80`, `00b1ec10` | Guild dispatcher, member status, error, invite notification, name request, location request, full data |
| `00a0f130`, `00a0f1d0`, `00a0f290`, `00a0ed50`, `00a0ee70`, `00a0ef90` | Guild invite decline, accept, timeout and their senders |
| `00c0ff00`, `00c10250`, `00c10330`, `00c10130`, `00c10430` | Lua `Guild` module: create, set motd, invite, rename, invite reply |
| `00b1e3b0`, `0106e6f0`, `00b19d70` | Guild manager, guild record, admin version |
| `0093b730`, `00a9ea40` | Player report and `/report` console command |
| `009347d0`, `00ad8880`, `008c6030` | Stop inspect, inspect proxy, show-inspect state |
| `008dc2f0` | `PacketTargetClientNotOnline` constructor |
| `00944fe0` | Self-data reader (underage, open-chat, membership bytes) |
| `0092f380` | Chat-bubble color command |
| `01282970`, `013ba330`, `013c0800` | Card-service chat submit, friend-status serializer, confirm-reciprocate serializer |
<!-- NARRATIVE END -->

## Mapped functions (generated)

1142 accepted functions: 593 confirmed, 283 likely, 143 speculative, 123 unknown. Network-linked (`wire`): 92.

Subsystems: `net.guild` (324), `net.chat` (271), `net.youtube` (109), `net.sns` (71), `net.friends` (50), `net.poll` (48), `game.friends` (46), `ui.friends` (35), `game.guild` (32), `ui.chat` (30), `ui.group` (29), `net.rating` (27), `ui.quickchat` (24), `game.social_share` (23), `net.portrait` (9), `net.group` (8), `net.acquaintance` (4), `net.profile` (2)

### `game.friends`

**Engine::AccountAcceptFriendRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128fad0` | Engine::AccountAcceptFriendRequest::Handle | likely | Gates on FUN_0127ebb0 and FUN_013bdf40 (which returns the first element of the vector at +0x20, or 0); builds a local UChatCommand_RequestFriendConfirmReciproc… |

**Engine::AccountRejectFriendRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128fc00` | Engine::AccountRejectFriendRequest::Handle | likely | Same body as AcceptFriendRequest::Handle (0128fad0): gates on FUN_0127ebb0 and FUN_013bdf40, builds a UChatCommand_RequestFriendConfirmReciprocate with FUN_013… |

**Engine::AccountRemoveFriend** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128f830` | Engine::AccountRemoveFriend::Handle | likely | Reads and converts the AccountID integer argument, then proceeds only when that conversion is nonzero and FUN_0127ebb0() succeeds; copies the two strings retur… |

**Engine::AccountRemoveIgnore** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128f980` | Engine::AccountRemoveIgnore::Handle | likely | Reads and converts the AccountID integer argument, then proceeds only when that conversion is nonzero and FUN_0127ebb0() succeeds; copies the two strings retur… |

**FriendStatus** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0143f7c0` | FriendStatus::FriendStatus | confirmed | Fastcall constructor (this in ECX, returns this in EAX). Stores the PersistentBase, PersistentComponent and FriendStatus vtables in that order (the last store … |
| `0143f8c0` | FriendStatus::~FriendStatus | confirmed | Destructor body (thiscall, no deleting part, no flag test). Sets FriendStatus::vftable, then for the string fields at +0x40, +0x24 and +0x8 in that order frees… |
| `0143f990` | FriendStatus::Clone | confirmed | Virtual slot 16 (fastcall, this in ECX). Allocates 0x60 bytes with operator new (00d833a6); when non-null it default-constructs the block with 0143f7c0 (which … |
| `0143fa80` | FriendStatus::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (thiscall, one stack argument, ret 4). Calls the destructor body 0143f8c0; when bit 0 of the argument is set calls _free on this; re… |
| `0143fc90` | FriendStatus::CopyFieldsTo | confirmed | Virtual slot 17 (thiscall: this is the source, one stack argument is the destination). Copies the int at +0x4 and the int at +0x5c into the destination, and as… |
| `0143fa00` | FriendStatus::Create | likely | Factory with no arguments. Calls operator new with 0x60 bytes (00d833a6); returns 0 when that returns null, otherwise default-constructs the block with 0143f7c… |

**IgnoreStatus** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0143fdc0` | IgnoreStatus::IgnoreStatus | confirmed | Fastcall constructor (this in ECX). Stores the PersistentBase, PersistentComponent and IgnoreStatus vtables in that order (the last store leaves IgnoreStatus::… |
| `0143fea0` | IgnoreStatus::~IgnoreStatus | confirmed | Destructor body (no deleting part). Sets IgnoreStatus::vftable, then frees the buffer of the string at +0x24 through _free when its capacity is above 15 and re… |
| `0143ff50` | IgnoreStatus::Clone | confirmed | Virtual slot 16 (fastcall, this in ECX). Allocates 0x40 bytes with operator new (00d833a6); when non-null default-constructs the block with 0143fdc0; then call… |
| `01440040` | IgnoreStatus::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (thiscall, one stack argument, ret 4). Calls the destructor body 0143fea0; when bit 0 of the argument is set calls _free on this; re… |
| `014401b0` | IgnoreStatus::CopyFieldsTo | confirmed | Virtual slot 17 (thiscall: this is the source, one stack argument is the destination). Copies the int at +0x4 and assigns the strings at +0x8 and +0x24 from so… |
| `0143ffc0` | IgnoreStatus::Create | likely | Factory with no arguments. Calls operator new with 0x40 bytes (00d833a6); returns 0 when that returns null, otherwise default-constructs the block with 0143fdc… |

**UChatCommand_RequestAddFriendReciprocate** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01441080` | UChatCommand_RequestAddFriendReciprocate::UChatCommand_Requ… | confirmed | Fastcall constructor (this in ECX, returns this). Calls the UChatCommand base constructor 013bd2e0, stores the UChatCommand_RequestAddFriendReciprocate vtable,… |
| `01441250` | UChatCommand_RequestAddFriendReciprocate::Clone | confirmed | Virtual slot 17 (fastcall, this in ECX). Allocates 0x9c bytes with operator new (00d833a6); when non-null default-constructs the block with 01441080; then call… |
| `01441350` | UChatCommand_RequestAddFriendReciprocate::ScalarDeletingDes… | confirmed | Scalar deleting destructor (thiscall, one stack argument, ret 4). Calls the destructor body 01441180; when bit 0 of the argument is set calls _free on this; re… |
| `014412c0` | UChatCommand_RequestAddFriendReciprocate::Create | likely | Factory with no arguments. Calls operator new with 0x9c bytes (00d833a6); returns 0 when that returns null, otherwise default-constructs the block with 0144108… |
| `01441180` | — | unknown | Destructor body (fastcall, this in ECX). Stores the AddFriendReciprocate vtable, frees the string at +0x84 (capacity at +0x94 above 15) and resets it, then the… |

**UChatCommand_RequestAddIgnore** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014417c0` | UChatCommand_RequestAddIgnore::UChatCommand_RequestAddIgnore | confirmed | Fastcall constructor (this in ECX, returns this). Calls the UChatCommand base constructor 013bd2e0, stores the UChatCommand_RequestAddIgnore vtable, and initia… |
| `01441890` | UChatCommand_RequestAddIgnore::~UChatCommand_RequestAddIgno… | confirmed | Destructor body (fastcall, this in ECX). Stores the AddIgnore vtable, frees the string at +0x64 through _free when its capacity at +0x78 is above 15 and resets… |
| `01441930` | UChatCommand_RequestAddIgnore::Clone | confirmed | Virtual slot 17 (fastcall, this in ECX). Allocates 0x7c bytes with operator new (00d833a6); when non-null default-constructs the block with 014417c0; then call… |
| `01441a20` | UChatCommand_RequestAddIgnore::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (thiscall, one stack argument, ret 4). Calls the destructor body 01441890; when bit 0 of the argument is set calls _free on this; re… |
| `014419a0` | UChatCommand_RequestAddIgnore::Create | likely | Factory with no arguments. Calls operator new with 0x7c bytes (00d833a6); returns 0 when that returns null, otherwise default-constructs the block with 014417c… |

**UChatCommand_RequestRemoveFriendReciprocate** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01440370` | UChatCommand_RequestRemoveFriendReciprocate::UChatCommand_R… | confirmed | Fastcall constructor (this in ECX). Calls the UChatCommand base constructor 013bd2e0 on this, then stores the UChatCommand_RequestRemoveFriendReciprocate vtabl… |
| `01440440` | UChatCommand_RequestRemoveFriendReciprocate::~UChatCommand_… | confirmed | Destructor body (fastcall, this in ECX). Stores the UChatCommand_RequestRemoveFriendReciprocate vtable, then frees the string at +0x64 (through _free when its … |
| `014404e0` | UChatCommand_RequestRemoveFriendReciprocate::Clone | confirmed | Virtual slot 17 (fastcall, this in ECX). Allocates 0x7c bytes with operator new (00d833a6); when non-null default-constructs the block with 01440370; then call… |
| `014405d0` | UChatCommand_RequestRemoveFriendReciprocate::ScalarDeleting… | confirmed | Scalar deleting destructor (thiscall, one stack argument, ret 4). Calls the destructor body 01440440; when bit 0 of the argument is set calls _free on this; re… |
| `01440550` | UChatCommand_RequestRemoveFriendReciprocate::Create | likely | Factory with no arguments. Calls operator new with 0x7c bytes (00d833a6); returns 0 when that returns null, otherwise default-constructs the block with 0144037… |

**UChatCommand_RequestRemoveIgnore** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014409b0` | UChatCommand_RequestRemoveIgnore::UChatCommand_RequestRemov… | confirmed | Fastcall constructor (this in ECX). Calls the UChatCommand base constructor 013bd2e0 on this, stores the UChatCommand_RequestRemoveIgnore vtable, and initialis… |
| `01440a80` | UChatCommand_RequestRemoveIgnore::~UChatCommand_RequestRemo… | confirmed | Destructor body (fastcall, this in ECX). Stores the RemoveIgnore vtable, frees the string at +0x64 through _free when its capacity at +0x78 is above 15 and res… |
| `01440b20` | UChatCommand_RequestRemoveIgnore::Clone | confirmed | Virtual slot 17 (fastcall, this in ECX). Allocates 0x7c bytes with operator new (00d833a6); when non-null default-constructs the block with 014409b0; then call… |
| `01440c10` | UChatCommand_RequestRemoveIgnore::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (thiscall, one stack argument, ret 4). Calls the destructor body 01440a80; when bit 0 of the argument is set calls _free on this; re… |
| `01440b90` | UChatCommand_RequestRemoveIgnore::Create | likely | Factory with no arguments. Calls operator new with 0x7c bytes (00d833a6); returns 0 when that returns null, otherwise default-constructs the block with 014409b… |

**(no class)** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00917b20` | SetAutoRefuseSetting | confirmed | Thiscall with a category index (param_2) and a value (param_3). When this+0x6d8 is non-null and the index is below 7 and the 7-entry name table (PTR_s_FriendIn… |
| `00917b60` | IsAutoRefuseEnabled | confirmed | Thiscall with a category index (param_2). Returns false when this+0x6d8 is null or the index is 7 or more or has no table name. Otherwise returns FUN_00910880(… |
| `0093b9b0` | CheckAndSubmitFriendRequest 🌐 | confirmed | Compares the candidate name with the current profile name; self matches emit FriendYourself. Two further candidate predicates block submission; only the first … |
| `00cb3fd0` | ShowFriendStatusMessage | confirmed | Maps status codes to friend-related message keys, optionally resolves a renamed friend, then displays the selected message for the record or supplied context. |
| `00cb47a0` | HandleFriendOnline | confirmed | Finds the friend record by key, marks it online and local-state fields, sorts rows, updates the displayed index if changed, then dispatches a Lua callback with… |
| `00cb4da0` | HandleFriendAdded | confirmed | Adds or resolves the friend row, sorts the list, emits FriendAdded UI notification, updates the related cache, and conditionally performs a location-dependent … |
| `00cb4ee0` | HandleFriendRemoved | confirmed | Notifies the associated handler, shows FriendRemoved for a cached record, removes the friend from the sorted row array, and triggers a local event when an inde… |
| `00cb4170` | CopyRecordAndSetIndex | likely | Copies a source record into the destination, sets destination field +0xd8 to -1, and returns the destination pointer. |
| `00cb4450` | CopyRecordAndStoreField | likely | Copies the supplied record into a temporary field and stores one additional 32-bit value at destination offset +0xe8. |
| `00cb4950` | HandleFriendOffline | likely | Finds the friend record by key, clears its online and local-state bytes, sorts rows, updates the displayed index when needed, then dispatches a Lua callback. |

### `game.guild`

**ClientGuildManager::PendingGuildNameRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a12cd0` | PendingGuildNameRequestMap_Dtor | confirmed | Map destructor: stores the SoeUtil::HashMap<int,ClientGuildManager::PendingGuildNameRequest,32,-1> vtable and calls FUN_00a11090 to free all 32 buckets. |

**Guild** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f8900` | Guild::TypeName | confirmed | Returns the C string 'Guild' (no arguments, no side effects). |
| `014f8af0` | Guild::Guild | confirmed | Fastcall with the new object in ECX. Stores PersistentBase and PersistentComponent vtables, then Guild::vftable at +0 and +4; zeroes +0x8, +0xc, +0x10, +0x34, … |
| `014f8be0` | Guild::~Guild | confirmed | Scalar deleting destructor: calls the Guild dtor body 014f8800, then frees this when bit 0 of the flag argument is set. Returns this. |
| `014f8580` | GuildPropertyCodeDispatchThunk | likely | Thunk on the secondary Guild base: adjusts ECX by -4 to reach the primary object and jumps to 014f7740 (PropertyCodeDispatch). Nothing else. |
| `014f8800` | Guild::DtorBody | likely | Stores Guild::vftable at this+0 and +4, calls the object at +0x4c (if any) through its virtual slot 0 with flag 1 and nulls the pointer, runs the string or mem… |
| `014f8910` | Guild::GetClassStatic | likely | Returns the dword at DAT_01cbe970 (no arguments). The word sits just before the global guild tree at DAT_01cbe974. |
| `014f8920` | Guild::SetClassStatic | likely | Stores its argument into DAT_01cbe970 (one stack argument, void). The matching getter is 014f8910. |
| `014f8940` | GuildSecondaryDtorThunk | likely | Thunk on the secondary Guild base: subtracts 4 from ECX and jumps to 014f8be0 (the scalar deleting destructor). Nothing else. |
| `014f7a60` | — | unknown | Thiscall with a code (param_2) and an output string object (param_3, written at *(param_3+8)). For code 0xff1 builds a temporary from this+0xc via 012df3f0 and… |

**GuildMemberDataSource** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cb6b80` | — | unknown | For special update types, delegates to the full member update path; otherwise validates guild context, resolves the incoming member record and updates the matc… |

**SoeUtil::HashList<GuildName,100,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b1e330` | GuildNameHashListScalarDeletingDtor | confirmed | Scalar deleting destructor: runs the destructor body FUN_00a1c0f0 and then, when bit 0 of param_2 is set, frees the object through the _free thunk at 00d82463;… |

**SoeUtil::HashListMap<int,GuildName,100,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b1e2b0` | GuildNameHashListMapCtor | confirmed | Installs the HashListMap and HashList vtables, zeros the map state, clears 400 bytes for 100 bucket heads, and returns the map pointer. |

**SoeUtil::HashMap<int,ClientGuildManager::PendingGuildNameRequest,32,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a13ed0` | HashMap_PendingGuildNameRequest_DtorWithEH | confirmed | Installs the PendingGuildNameRequest map vftable and calls FUN_00a11090 with this object in ECX to drain the 32-bucket map, then returns without freeing the ob… |
| `00a13660` | HashMap_PendingGuildNameRequest_ScalarDeletingDtor | likely | Scalar deleting destructor (vtable slot 0) of the guild manager's pending name-request map: writes the class vftable into the object, runs the 32-bucket teardo… |

**(no class)** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b1bba0` | TouchPairTimeEntry | likely | Looks up the (param_1,param_2) pair in a 32-bucket hash map; allocates an entry when absent, writes the pair, then stamps the current time pair on every call, … |
| `00b1bc10` | InitHashListNodeKey | likely | Runs FUN_00b1afa0 on the node (zeroing its first two dwords) and then stores the dword pointed to by param_2 in the node field at +0x48; returns the node point… |
| `00b1ca10` | HandleGuildInviteCancel 🌐 | likely | Guards param_2, builds a reader over (param_2, param_3), parses the cancel record with FUN_00b1ac50 and then calls FUN_00a0ebc0 (clearing the pending invite st… |
| `00b1ce80` | HandleGuildPlayerStatusUpdate 🌐 | likely | Constructs GuildPlayerStatusUpdatePacket and parses its body. Looks up the referenced player; if found, calls interface slot 0x10 at object+0x54 and compares t… |
| `00b1d030` | UpdateGuildNameCacheEntry | likely | Looks up and unlinks an existing pair-keyed entry, searches the 100-bucket name table by xor of the two key dwords, allocates and stores the pair if absent, as… |
| `00b1e3b0` | GuildManagerCtor | likely | Guild client-manager constructor: stores the owner pointer at +0x8c, zeroes +0x90 through +0xac, constructs the 100-bucket name table at +0xb0 (00b1e2b0), clea… |
| `00b1e960` | RequestGuildName 🌐 | likely | Walks the manager's 100-bucket name table and returns node+0xc on a key match. On a miss, checks the pair request cache; if stored time age is below 0x3d it re… |
| `0106e4e0` | DtorBodyGuildRoleAndMemberMaps | likely | Destructor body for a guild record object. It erases the head of the list at this+0x98 through 00b19170 until empty, runs 00b1a730 (GuildRole map destruct) wit… |
| `014f82b0` | GuildDisplayName | likely | Output string (param_1) and guild id (param_2) as stack arguments. Looks up the id in the global tree through 014f6c00; when found, assigns the string at the e… |
| `014f8d80` | GuildLookupOrCreate | likely | One argument on the stack (a guild record whose id is at +8). Looks the id up in the global guild table with 014f6c00; when found, copies fields with 014f8980,… |
| `014f8e50` | GuildUpsertName | likely | Two arguments: a guild id (param_1) and a string (param_2). Looks the id up in the global table with 014f6c00; when absent, creates a Guild (0x54 bytes, 014f8a… |
| `00b1e7e0` | HandleGuildInviteNotification 🌐 | speculative | Constructs and parses a GuildInviteNotificationPacket when param_2 is non-null. On successful parse, queries the inviter through FUN_00a12720. If that returns … |
| `00b1c110` | — | unknown | Allocates a node through the container's virtual allocator (slot 1 of *param_1), links it as the new tail of the intrusive list (node+0x44 = old tail at param_… |
| `00b1d6c0` | — 🌐 | unknown | Guards param_1, builds a reader over (param_1, param_2), parses with FUN_00b1d1b0 (two 16-bit fields, a skip and one boolean into the local record) and, when t… |
| `00b1d7d0` | — 🌐 | unknown | Builds a GuildMemberStatusUpdatePacket object (FUN_00b1b280), parses the status record with FUN_00b1c840, looks the member up (FUN_00b18df0) and fetches or cre… |
| `00b1ec10` | — 🌐 | unknown | Constructs and parses GuildDataFullPacket; on a valid bounded parse, removes an existing matching guild entry from the four-bucket map, allocates and links a r… |
| `014f8980` | — | unknown | Thiscall (ECX = target object) with one stack argument (source object). Copies dwords +0x8, +0xc and +0x10 from the source; assigns the string at +0x14; calls … |

### `game.social_share`

**SocialShareManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae0c00` | SocialShareManager::ScalarDeletingDtor | confirmed | Calls manager destruction body, frees this when flag bit 0 is set, and returns this otherwise. |
| `00ae09b0` | — | unknown | Installs SocialShareManager vtable, initializes its HTTP base, string fields and three provider states, sets timeout/config values, allocates a request buffer,… |
| `00ae0b10` | — | unknown | Installs manager vtable, shuts down HTTP support, destroys three provider records and the manager string; frees its allocated request object when present, then… |

**SocialShareManager::MessageTrackingNode** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00adf030` | MessageTrackingNodeBaseCtor | likely | Initialises the 0x38-byte base of a tracking-map node: zeroes +0/+4/+8, installs SoeUtil::IString<char>::vftable at +0xc with the static empty buffer 0x1be809c… |
| `00adf0b0` | MessageTrackingNodeBaseDtor | likely | Destructor body for the node base: frees the owned buffer pointer at +0x30 through _free, restores the IString vtable at +0xc, and when the string length at +0… |

**(no class)** (18)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00adfc00` | HandleHttpProviderStatus | confirmed | For HTTP 200, copies the response body, maps known textual statuses to numeric outcomes, clears authorization state for missing/revoked authorization, invokes … |
| `00adebb0` | SocialShareMessageTrackingMap_Construct | likely | Installs the SoeUtil::HashListMap<int,SocialShareManager::MessageTrackingNode,64,-1> vtable, zeros dwords at +4,+8,+0xc,+0x10, and clears 0x100 bytes at +0x14 … |
| `00adfef0` | RequestSocialShareAuthorization | likely | Rejects provider indexes outside 1Ã¢â‚¬â€œ2; for a valid unauthorized provider checks cached authorization, creates a tracking entry, calls the callback interf… |
| `00adffe0` | SubmitSocialSharePayload | likely | For provider indexes 1Ã¢â‚¬â€œ2 with authorization set, allocates a request node, copies the caller's payload bytes into owned storage, records callback fields… |
| `00ae03d0` | HandleProviderStatusResponse 🌐 | likely | Parses a response body, locates its request by identifier, updates the provider status byte on HTTP 200 and invokes the callback, reports callback error on oth… |
| `00ae0850` | SocialShareProviderStateCtor | likely | Initializes two embedded IString objects to the static empty buffer, constructs the tracking map, and clears the provider state's status byte. |
| `00ae08e0` | SocialShareProviderStateDtor | likely | Removes all tracked request nodes, destroys the map, then decrements shared-buffer references and destroys both embedded IString fields. |
| `00ae0f60` | HandleSocialSharePublishHttp 🌐 | likely | When authorization and ID checks pass, constructs a publish packet, fills provider/string/payload fields, checks the active app context, sends it through the g… |
| `00ae1060` | ProcessSocialShareRequests | likely | Walks a linked request queue at manager+0x28; dispatches entries with kind byte 1 to status handling, kind 2 to text/payload handling, and kind 3 to publish ha… |
| `00ae10b0` | PollSocialShareManager | likely | If enabled flag at this+0x20 is set, processes two provider queues beginning at this+0x15c with stride 0x138, then calls the shared HTTP polling helper. |
| `00adf620` | SetFields3d0AndEnable | speculative | Calls FUN_00402ce0 with the second argument, stores the next two arguments at this+0x3d0/+0x3d4, sets byte this+0x20 to 1, and returns. |
| `00adfe10` | ClearTrackedRequests | speculative | Repeatedly takes the node at object+8 and passes it to the node-removal helper until the list head becomes null. |
| `00adee10` | — | unknown | Hash lookup over two 64-bucket tables: for each of the two table bases at this+400 and this+0x538 it walks the bucket chain selected by key & 0x3f (next pointe… |
| `00adf650` | — | unknown | When this+0x3d8 is clear and the supplied pair matches DAT_01be80f8/DAT_01be80fc, builds an IString, copies up to 0x400 characters from the supplied text, assi… |
| `00adfe50` | — | unknown | For provider indexes 1 or 2, allocates a tracking node, records index/state/callback data, and increments the request count; index 0 recursively requests both.… |
| `00ae00b0` | — | unknown | If the refresh flag is set, scans provider indexes 1 and 2 for state-2 entries, removes the first whose stored identifiers differ from the current identifiers,… |
| `00ae0520` | — 🌐 | unknown | Parses two 32-bit response values, finds the tracked request, clears local authorization for result 3, dispatches the callback with result/provider and stored … |
| `00ae0ea0` | — | unknown | When supplied IDs match manager IDs, constructs a status request, checks the active app context, serializes/sends it through the gateway, then replaces the cac… |

### `net.acquaintance`

**RemoveAcquaintancePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a16c30` | RemoveAcquaintancePacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): delegates to the destructor body 00a15150, which releases the IString member at +0xc, and frees the packet with _fr… |
| `00a16bb0` | — 🌐 | unknown | Constructor: calls the acquaintance base constructor FUN_00a10210 with sub-opcode 2 (base opcode 0x48), stores RemoveAcquaintancePacket::vftable, zeroes the fi… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a15150` | — | unknown | Destructor body for a packet whose IString lives at +0xc: stores the IString vftable at +0xc, decrements the shared-buffer refcount at buffer-4 and calls the I… |
| `00c24a00` | — 🌐 | unknown | Reads one input value, constructs a two-string payload, constructs CommandPacketRemoveFriendRequest through 00c1db80, and passes it to 00c24670 when the client… |

### `net.chat`

**Chat::ChatSendTextToLobbyDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01281de0` | — | unknown | Virtual slot [2]. Reads entry 2 as the ChatDisposition integer and entry 3 as the ConversationID integer (type checks assert and fall back to 0), entry 1 as a … |

**ClientChatProcessor** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b15450` | ClientChatProcessor::ClientChatProcessor | confirmed | ClientChatProcessor constructor: installs CommandProcessor::CommandEventHandler::vftable and then ClientChatProcessor::vftable, stores the owner pointer at +4,… |
| `00b156d0` | ClientChatProcessor::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot [1] of ClientChatProcessor::vftable: calls the destructor body 00b151d0 and frees the object with _free when bit 0 of the fl… |
| `00b160b0` | ClientChatProcessor::CommandGuildSay 🌐 | confirmed | Chat command handler for command id 7 with channel number 9: constructs a PacketChat with 00b0f8f0, writes channel 9 into the packet channel slot (local_224), … |
| `00b16350` | ClientChatProcessor::CommandWhisper 🌐 | speculative | Chat line sender for channel number 3: constructs a PacketChat with 00b0f8f0, writes channel 3 into the packet channel slot (local_224), copies the text from p… |
| `00b151d0` | — | unknown | ClientChatProcessor destructor: installs ClientChatProcessor::vftable, tears down the base through 00986c10, drains the pending chat-node list at +0x488 throug… |
| `00b15710` | — 🌐 | unknown | Chat command handler for command id 0 with channel number 0: constructs a PacketChat on the stack with 00b0f8f0, writes channel 0 into the packet channel slot … |
| `00b15870` | — 🌐 | unknown | Chat command handler for command id 3 with channel number 5: constructs a PacketChat with 00b0f8f0, writes channel 5 into the packet channel slot (local_224), … |
| `00b159d0` | — 🌐 | unknown | Chat command handler for command id 6 with channel number 6: constructs a PacketChat with 00b0f8f0, writes channel 6 into the packet channel slot (local_224), … |
| `00b15b30` | — 🌐 | unknown | Chat command handler for command id 4 with channel number 7: constructs a PacketChat with 00b0f8f0, writes channel 7 into the packet channel slot (local_224), … |
| `00b15c90` | — 🌐 | unknown | Chat command handler for command id 8 with channel number 10: constructs a PacketChat with 00b0f8f0, writes channel 10 into the packet channel slot (local_224)… |
| `00b15df0` | — 🌐 | unknown | Chat command handler for command id 5 with channel number 8: constructs a PacketChat with 00b0f8f0, writes channel 8 into the packet channel slot (local_224), … |
| `00b15f50` | — 🌐 | unknown | Chat command handler for command id 1 with channel number 4: constructs a PacketChat with 00b0f8f0, writes channel 4 into the packet channel slot (local_224), … |
| `00b16220` | — 🌐 | unknown | Chat command handler for command id 2 with channel number 1: constructs a PacketChat with 00b0f8f0, writes channel 1 into the packet channel slot (local_224), … |
| `00b164b0` | — | unknown | Slot [0] virtual of ClientChatProcessor::vftable, the implementation of the CommandEventHandler pure virtual: first runs the chat spam gate using the ChatSpeed… |

**Engine::PendingFriendRequest** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013c0e20` | Engine::PendingFriendRequest::PendingFriendRequest | confirmed | Constructor body (fastcall, this in ecx): stores the Engine::PendingFriendRequest vtable at +0, then initialises four std::string objects at +0x04, +0x20, +0x3… |
| `013c0eb0` | Engine::PendingFriendRequest::~PendingFriendRequest | confirmed | Destructor body: re-stores the Engine::PendingFriendRequest vtable at +0, then for each of the four string objects (capacity at +0x70, +0x54, +0x38, +0x1c) rel… |
| `013c0f90` | Engine::PendingFriendRequest::ScalarDeletingDtor | confirmed | Virtual slot [0] of the class (bytes 56 8b f1 / e8 / f6 44 24 08 01 / 74 09 / 56 / e8 / 83 c4 04 / 8b c6 / 5e c3): saves esi, calls the destructor body FUN_013… |
| `013c1350` | Engine::PendingFriendRequest::SetField04 | likely | Assigns the argument std::string to the string member at this+0x04 (assign(src, 0, npos); bytes 8b 44 24 04 / 6a ff 6a 00 50 / 83 c1 04 / e8 / c2 04 00, this i… |
| `013c1370` | Engine::PendingFriendRequest::SetField20 | likely | Assigns the argument std::string to the string member at this+0x20 (assign(src, 0, npos)). |
| `013c1390` | Engine::PendingFriendRequest::SetField3C | likely | Assigns the argument std::string to the string member at this+0x3c (assign(src, 0, npos)). |
| `013c13b0` | Engine::PendingFriendRequest::SetField58 | likely | Assigns the argument std::string to the string member at this+0x58 (assign(src, 0, npos)). |

**LobbyCommand_SendText** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013b3e80` | LobbyCommand_SendText::ReadFields | confirmed | Virtual slot 9: calls this->vtable slot 4 with (stream, &stream); then 013b2910 (LobbyCommand read), reads int into +8, int into +0xC, a third int into +0x10 (… |
| `013b3f50` | LobbyCommand_SendText::WriteFields | confirmed | Virtual slot 10: calls this->vtable slot 2, then 013b2960 (LobbyCommand base write), writes ints at +8, +0xC and +0x10 through 013c23f0, the string at +0x14 th… |
| `013b3fc0` | LobbyCommand_SendText::LobbyCommand_SendText | confirmed | Calls 013b2760 (LobbyCommand constructor), stores LobbyCommand_SendText::vftable at *this, sets the capacity dwords at +0x2C to 0xF and the length dwords at +0… |
| `013b4060` | LobbyCommand_SendText::GetTypeName | confirmed | Virtual slot 11: returns the static C string LobbyCommand_SendText. |
| `013b40a0` | LobbyCommand_SendText::DtorBody | confirmed | Stores LobbyCommand_SendText::vftable at *this, frees the string buffer at +0x18 when the capacity at +0x2C is above 0xF, resets the string words, then calls 0… |
| `013b4210` | LobbyCommand_SendText::~LobbyCommand_SendText | confirmed | Scalar deleting destructor in vtable slot 0: calls 013b40a0 (destructor body), then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `013b4250` | LobbyCommand_SendText::CopyTo | confirmed | Virtual slot 18: dynamic-casts param_2 to LobbyCommand_SendText (00d8d382 with NetworkCommand and LobbyCommand_SendText RTTI descriptors); on failure asserts l… |
| `013b42d0` | LobbyCommand_SendText::AppendDescription | confirmed | Virtual slot 19: appends the text Send Text Lobby Command newline, calls 013b2af0 (LobbyCommand description), then appends lines formatted with _sprintf for Gr… |
| `013b4460` | LobbyCommand_SendText::AppendDisplayName | confirmed | Virtual slot 20: appends the 23-byte text Send Text Lobby Command to its std::string argument through 01241650. |
| `013b4070` | LobbyCommand_SendText::GetClassId | likely | Virtual slot 12: returns the dword in the global DAT_01cbdd20. |
| `013b4080` | LobbyCommand_SendText::SetClassId | likely | Virtual slot 13: stores its stack argument into the global DAT_01cbdd20. |
| `013b4120` | LobbyCommand_SendText::Clone | likely | Virtual slot 17: allocates 0x38 bytes (00d833a6), constructs a LobbyCommand_SendText with 013b3fc0 when the allocation succeeds, then calls this->vtable slot 1… |
| `013b4190` | LobbyCommand_SendText::CreateInstance | likely | Allocates 0x38 bytes (00d833a6); when the allocation succeeds constructs a LobbyCommand_SendText with 013b3fc0 and returns it, else returns 0. |
| `013b3d40` | LobbyCommand_SendText::Vslot15 | speculative | Virtual slot 15: builds a 0x14-byte event object (00d833a6, 012f9eb0); when this+0x10 is zero it sets type id 0xF (012f8c70) and stores +0xC, the string at +0x… |
| `013b41f0` | — | unknown | Pushes creator 013b4190 and id 0x61, calls 013ce130, then calls 013ce230 with ECX set to 013ce130 return value and no stack arguments; returns void. |

**LobbyCommand_SendUChatText** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013b45d0` | LobbyCommand_SendUChatText::ReadFields | confirmed | Virtual slot 9: calls this->vtable slot 4 with (stream, &stream); then 013b2910 (LobbyCommand read), reads five strings into +8, +0x24, +0x40, +0x5C and +0x78 … |
| `013b4680` | LobbyCommand_SendUChatText::WriteFields | confirmed | Virtual slot 10: calls this->vtable slot 2, then 013b2960 (LobbyCommand base write), writes five strings at +8, +0x24, +0x40, +0x5C and +0x78 through 013c2860,… |
| `013b46f0` | LobbyCommand_SendUChatText::LobbyCommand_SendUChatText | confirmed | Calls 013b2760 (LobbyCommand constructor), stores LobbyCommand_SendUChatText::vftable at *this, zeroes five std::string members (capacity 0xF, length 0, first … |
| `013b44f0` | LobbyCommand_SendUChatText::Vslot15 | speculative | Virtual slot 15: builds a 0x14-byte event object (00d833a6, 012f9eb0), sets its type id to 0xA6 (012f8c70), stores an id from 01301f30 (guarded by 012d0920) an… |

**PacketChat** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b11600` | DeserializePacketChatBody 🌐 | confirmed | Reads the two 16-bit header fields, bounded scalar and pair fields, a length-counted byte span, a float4, and trailing fields into the packet object; truncatio… |
| `00b14fa0` | — 🌐 | unknown | Writes the PacketChat fields through the writer: delegates the first two 16-bit values, writes the field at +0xc, serializes members at +0x10, +0x18, +0x130, +… |

**UChatCommand** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013bcf60` | UChatCommand::Deserialize | confirmed | Read (vtable slot [9]) of the UChatCommand base: reads the header through slot [4], calls 0140c2a0, reads two serial integers into +4 and +8, reads two strings… |
| `013bd090` | UChatCommand::~UChatCommand | confirmed | Destructor body: stores UChatCommand::vftable at +0, frees the string buffer at +0x28 when its capacity exceeds 15 and the one at +0xc likewise, resets both st… |
| `013bd150` | UChatCommand::GetName | confirmed | Virtual slot [11] of the base: returns the literal UChatCommand (6 bytes: mov eax, literal; ret). |
| `013bd160` | UChatCommand::GetSharedDword | confirmed | Virtual slot [12] of the base: returns the 32-bit global DAT_01cbdd70. |
| `013bd170` | UChatCommand::SetSharedDword | confirmed | Virtual slot [13] of the base: stores its one stack argument into the global DAT_01cbdd70 (ret 4). |
| `013bd2c0` | UChatCommand::ScalarDeletingDtor | confirmed | Virtual slot [0] of the base: calls the destructor body 013bd090, then calls _free(this) only when bit 0 of the argument is set; returns this. |
| `013bd2e0` | UChatCommand::UChatCommand | confirmed | Constructor body of the UChat base: runs 0140c090 (NetworkCommand base constructor) on this, stores UChatCommand::vftable at +0, then sets the string members t… |
| `013bd3f0` | UChatCommand::TraceDump | confirmed | Virtual slot [19] of the base: formats AccountID: %d, Result: %d and Source: %s@%s trace lines (snprintf/sprintf into a stack buffer) and appends them with 012… |
| `013bd500` | UChatCommand::TraceAppendName | confirmed | Virtual slot [20] of the base: appends the literal UChat Command with 01241650 and returns. |
| `013bd680` | UChatCommand::CopyFieldsTo | confirmed | Virtual slot [18] of the base: dynamic-casts param_2 from NetworkCommand to UChatCommand through 00d8d382 (assert uchatCommand on failure), copies the integer … |
| `013bd600` | UChatCommand::CreateDefault | likely | Allocates an object with operator new (00d833a6); on success constructs it with 013bd2e0 and returns it, otherwise returns 0. |
| `013bd660` | UChatCommand::RegisterFactory | likely | Pushes the factory 013bd600 and the key 0x1f8, calls 013ce130 with them, then 013ce230 with the registry result; returns. |
| `013bd190` | — | unknown | Virtual slot [15] of the base: builds a 0x14-byte event with 012f9eb0 (allocated with 00d833a6), writes its type through 012f8c70, writes the base fields throu… |
| `013bd380` | — | unknown | Virtual slot [17] of the base: allocates 0x54 bytes with 00d833a6 when possible, constructs the object with 013bd2e0 and returns it (0 on allocation failure). |

**UChatCommand_OnReceiveFriendConfirmReciprocateRequest** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01529710` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::Ctor | confirmed | Constructor body: runs the UChatCommand base constructor 013bd2e0 and stores this class vftable; it adds no fields of its own. |
| `01529790` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::GetN… | confirmed | Virtual slot 11: returns the address of the literal 'UChatCommand_OnReceiveFriendConfirmReciprocateRequest'. |
| `015297d0` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::Dest… | confirmed | Stores this class vftable into *this, then calls 013bd090 (UChatCommand base teardown, a helper outside this shard). |
| `01529930` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::Copy… | confirmed | Virtual slot 18: dynamic-casts the target from NetworkCommand to this class (00d8d382); on failure reports the 'uchatCommand' assertion via 012f5a60; then call… |
| `01529980` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::Read | confirmed | Virtual slot 9: reads the header through this->vtable[4], then the base body 013bcf60, then the leaf body through this->vtable[5]. It reads no fields of its ow… |
| `015299c0` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::Write | confirmed | Virtual slot 10: calls this->vtable[2] (header write), the base body 013bcfe0, then this->vtable[3] (body write). It writes no fields of its own. |
| `01529a70` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::Scal… | confirmed | Virtual slot 0: stores the class vftable, calls base teardown 013bd090, then calls _free(this) only when bit 0 of the argument is set; returns this. |
| `01529ad0` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::Dump | confirmed | Virtual slot 19 (debug dump, ret 4): appends 'Send OnReceiveFriendConfirmRequest UChat Command' with a newline to the output string, then calls base dump 013bd… |
| `01529b00` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::Appe… | confirmed | Virtual slot 20: moves its stack argument (a string object) into ECX and appends the 48-character literal 'Send OnReceiveFriendConfirmRequest UChat Command' wi… |
| `01529820` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::Queu… | likely | Virtual slot 15: allocates 0x14 bytes with 00d833a6 and, when non-null, initialises the block with 012f9eb0; writes tag 0xc5 with 012f8c70; calls 012fa160 for … |
| `015298c0` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::Clone | likely | Virtual slot 17: allocates 0x44 bytes with 00d833a6; when non-null constructs the object with 01529710; then calls this->vtable[0x48/4] (CopyFieldsTo) with the… |
| `015299f0` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::Crea… | likely | Allocates 0x44 bytes with 00d833a6; when non-null constructs the object with 01529710 and returns it, otherwise returns 0. |
| `015297a0` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::GetS… | speculative | Virtual slot 12: returns the global dword DAT_01cbec28. |
| `015297b0` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::SetS… | speculative | Virtual slot 13: stores its stack argument into the global DAT_01cbec28. |

**UChatCommand_OnReceiveFriendConfirmReciprocateResponse** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01528d70` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::Ctor | confirmed | Constructor body: runs the UChatCommand base constructor 013bd2e0, stores this class vftable and clears the flag byte at this+0x44. |
| `01528df0` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::Get… | confirmed | Virtual slot 11: returns the address of the literal 'UChatCommand_OnReceiveFriendConfirmReciprocateResponse'. |
| `01528e30` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::Des… | confirmed | Stores this class vftable into *this, then calls 013bd090 (UChatCommand base teardown, a helper outside this shard). |
| `01528fc0` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::Cop… | confirmed | Virtual slot 18: dynamic-casts the target from NetworkCommand to this class (00d8d382); on failure reports the 'uchatCommand' assertion through 012f5a60; calls… |
| `01529010` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::Read | confirmed | Virtual slot 9: reads the header through this->vtable[4], the base body 013bcf60, then one integer (013c24d0) which it stores as a bool at this+0x44 (non-zero … |
| `01529070` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::Wri… | confirmed | Virtual slot 10: calls this->vtable[2] (header write), the base body 013bcfe0, appends the flag byte at this+0x44 as one integer with 013c23f0, then calls this… |
| `01529130` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::Sca… | confirmed | Virtual slot 0: stores the class vftable, calls base teardown 013bd090, then calls _free(this) only when bit 0 of the argument is set; returns this. |
| `01529190` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::Dump | confirmed | Virtual slot 19 (debug dump): appends 'Send OnReceiveFriendConfirmResponse UChat Command' with a newline, calls base dump 013bd3f0 with this and the output, th… |
| `01529220` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::App… | confirmed | Virtual slot 20: moves its stack argument (a string object) into ECX and appends the 49-character literal 'Send OnReceiveFriendConfirmResponse UChat Command' w… |
| `01528ea0` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::Que… | likely | Virtual slot 15: allocates 0x14 bytes with 00d833a6 and, when non-null, initialises the block with 012f9eb0; writes tag 0xc6 into it with 012f8c70; calls 012fa… |
| `01528f50` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::Clo… | likely | Virtual slot 17: allocates 0x48 bytes with 00d833a6; when non-null constructs the object with 01528d70; then calls this->vtable[0x48/4] (CopyFieldsTo) with the… |
| `015290b0` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::Cre… | likely | Allocates 0x48 bytes with 00d833a6; when non-null constructs the object with 01528d70 and returns it, otherwise returns 0. |
| `01528e00` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::Get… | speculative | Virtual slot 12: returns the global dword DAT_01cbec20. |
| `01528e10` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::Set… | speculative | Virtual slot 13: stores its stack argument into the global DAT_01cbec20. |

**UChatCommand_OnReceiveFriendConfirmRequest** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01529b20` | UChatCommand_OnReceiveFriendConfirmRequest::UChatCommand_On… | confirmed | Constructor body: runs the UChatCommand base constructor 013bd2e0 and stores this class vftable; it adds no fields of its own. |
| `01529ba0` | UChatCommand_OnReceiveFriendConfirmRequest::GetName | confirmed | Virtual slot 11: returns the address of the literal 'UChatCommand_OnReceiveFriendConfirmRequest'. |
| `01529be0` | UChatCommand_OnReceiveFriendConfirmRequest::DestructorBody | confirmed | Stores this class vftable into *this, then calls 013bd090 (UChatCommand base teardown, a helper outside this shard). |
| `01529d40` | UChatCommand_OnReceiveFriendConfirmRequest::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target from NetworkCommand to this class (00d8d382); on failure reports the 'uchatCommand' assertion via 012f5a60; then call… |
| `01529d90` | UChatCommand_OnReceiveFriendConfirmRequest::Read | confirmed | Virtual slot 9: reads the header through this->vtable[4], then the base body 013bcf60, then the leaf body through this->vtable[5]. It reads no fields of its ow… |
| `01529dd0` | UChatCommand_OnReceiveFriendConfirmRequest::Write | confirmed | Virtual slot 10: calls this->vtable[2] (header write), the base body 013bcfe0, then this->vtable[3] (body write). It writes no fields of its own. |
| `01529e80` | UChatCommand_OnReceiveFriendConfirmRequest::ScalarDeletingD… | confirmed | Virtual slot 0: stores the class vftable, calls base teardown 013bd090, then calls _free(this) only when bit 0 of the argument is set; returns this. |
| `01529ee0` | UChatCommand_OnReceiveFriendConfirmRequest::Dump | confirmed | Virtual slot 19 (debug dump, ret 4): appends 'Send OnReceiveFriendConfirmRequest UChat Command' with a newline to the output string, then calls base dump 013bd… |
| `01529f10` | UChatCommand_OnReceiveFriendConfirmRequest::AppendDisplayNa… | confirmed | Virtual slot 20: appends the 48-character literal 'Send OnReceiveFriendConfirmRequest UChat Command' with 01241650 (the output string argument is not used by G… |
| `01529c30` | UChatCommand_OnReceiveFriendConfirmRequest::QueueClientEvent | likely | Virtual slot 15: allocates 0x14 bytes with 00d833a6 and, when non-null, initialises the block with 012f9eb0; writes tag 0xc5 with 012f8c70; calls 012fa160 for … |
| `01529cd0` | UChatCommand_OnReceiveFriendConfirmRequest::Clone | likely | Virtual slot 17: allocates 0x44 bytes with 00d833a6; when non-null constructs the object with 01529b20; then calls this->vtable[0x48/4] (CopyFieldsTo) with the… |
| `01529e00` | UChatCommand_OnReceiveFriendConfirmRequest::Create | likely | Allocates 0x44 bytes with 00d833a6; when non-null constructs the object with 01529b20 and returns it, otherwise returns 0. |
| `01529bb0` | UChatCommand_OnReceiveFriendConfirmRequest::GetSharedDword | speculative | Virtual slot 12: returns the global dword DAT_01cbec2c. |
| `01529bc0` | UChatCommand_OnReceiveFriendConfirmRequest::SetSharedDword | speculative | Virtual slot 13: stores its stack argument into the global DAT_01cbec2c. |

**UChatCommand_OnReceiveFriendConfirmResponse** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01529240` | UChatCommand_OnReceiveFriendConfirmResponse::UChatCommand_O… | confirmed | Constructor body: runs the UChatCommand base constructor 013bd2e0, stores this class vftable and clears the flag byte at this+0x44. |
| `015292c0` | UChatCommand_OnReceiveFriendConfirmResponse::GetName | confirmed | Virtual slot 11: returns the address of the literal 'UChatCommand_OnReceiveFriendConfirmResponse'. |
| `01529300` | UChatCommand_OnReceiveFriendConfirmResponse::DestructorBody | confirmed | Stores this class vftable into *this, then calls 013bd090 (UChatCommand base teardown, a helper outside this shard). |
| `01529490` | UChatCommand_OnReceiveFriendConfirmResponse::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target from NetworkCommand to this class (00d8d382); on failure reports the 'uchatCommand' assertion via 012f5a60; calls bas… |
| `015294e0` | UChatCommand_OnReceiveFriendConfirmResponse::Read | confirmed | Virtual slot 9: reads the header through this->vtable[4], the base body 013bcf60, then one integer (013c24d0) stored as a bool at this+0x44 (non-zero means tru… |
| `01529540` | UChatCommand_OnReceiveFriendConfirmResponse::Write | confirmed | Virtual slot 10: calls this->vtable[2] (header write), the base body 013bcfe0, appends the flag byte at this+0x44 as one integer with 013c23f0, then calls this… |
| `01529600` | UChatCommand_OnReceiveFriendConfirmResponse::ScalarDeleting… | confirmed | Virtual slot 0: stores the class vftable, calls base teardown 013bd090, then calls _free(this) only when bit 0 of the argument is set; returns this. |
| `01529660` | UChatCommand_OnReceiveFriendConfirmResponse::Dump | confirmed | Virtual slot 19 (debug dump): appends 'Send OnReceiveFriendConfirmResponse UChat Command' with a newline, calls base dump 013bd3f0 with this and the output, th… |
| `015296f0` | UChatCommand_OnReceiveFriendConfirmResponse::AppendDisplayN… | confirmed | Virtual slot 20: moves its stack argument (a string object) into ECX and appends the 49-character literal 'Send OnReceiveFriendConfirmResponse UChat Command' w… |
| `01529370` | UChatCommand_OnReceiveFriendConfirmResponse::QueueClientEve… | likely | Virtual slot 15: allocates 0x14 bytes with 00d833a6 and, when non-null, initialises the block with 012f9eb0; writes tag 0xc6 with 012f8c70; calls 012fa160 for … |
| `01529420` | UChatCommand_OnReceiveFriendConfirmResponse::Clone | likely | Virtual slot 17: allocates 0x48 bytes with 00d833a6; when non-null constructs the object with 01529240; then calls this->vtable[0x48/4] (CopyFieldsTo) with the… |
| `01529580` | UChatCommand_OnReceiveFriendConfirmResponse::Create | likely | Allocates 0x48 bytes with 00d833a6; when non-null constructs the object with 01529240 and returns it, otherwise returns 0. |
| `015292d0` | UChatCommand_OnReceiveFriendConfirmResponse::GetSharedDword | speculative | Virtual slot 12: returns the global dword DAT_01cbec24. |
| `015292e0` | UChatCommand_OnReceiveFriendConfirmResponse::SetSharedDword | speculative | Virtual slot 13: stores its stack argument into the global DAT_01cbec24. |

**UChatCommand_OnReceiveFriendLogin** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152a340` | UChatCommand_OnReceiveFriendLogin::UChatCommand_OnReceiveFr… | confirmed | Constructor body: runs the UChatCommand base constructor 013bd2e0 and stores this class vftable; it adds no fields of its own. |
| `0152a3c0` | UChatCommand_OnReceiveFriendLogin::GetName | confirmed | Virtual slot 11: returns the address of the literal 'UChatCommand_OnReceiveFriendLogin'. |
| `0152a400` | UChatCommand_OnReceiveFriendLogin::DestructorBody | confirmed | Stores this class vftable into *this, then calls 013bd090 (UChatCommand base teardown, a helper outside this shard). |
| `0152a4f0` | UChatCommand_OnReceiveFriendLogin::Clone | confirmed | Virtual slot [17]. Allocates 0x44 bytes with FUN_00d833a6; if that returns non-null it constructs the block with FUN_0152a340 and uses the result, otherwise us… |
| `0152a560` | UChatCommand_OnReceiveFriendLogin::CopyFieldsTo | confirmed | Virtual slot [18]. Dynamic-casts param_2 from NetworkCommand to UChatCommand_OnReceiveFriendLogin through FUN_00d8d382; when the cast fails it reports through … |
| `0152a5b0` | UChatCommand_OnReceiveFriendLogin::Deserialize | confirmed | Virtual slot [9]. Calls this object's slot [4] (+0x10) with the stream and the address of the argument; if that returns true it calls the base reader FUN_013bc… |
| `0152a5f0` | UChatCommand_OnReceiveFriendLogin::Serialize | confirmed | Virtual slot [10]. Calls this object's slot [2] (+0x8) with the stream, then the base writer FUN_013bcfe0 (UChatCommand::vftable slot [10]) with the stream, th… |
| `0152a6a0` | UChatCommand_OnReceiveFriendLogin::ScalarDeletingDtor | confirmed | Virtual slot [0]. Stores UChatCommand_OnReceiveFriendLogin::vftable into *this, calls base destructor body FUN_013bd090, then calls _free(this) when bit 0 of t… |
| `0152a700` | UChatCommand_OnReceiveFriendLogin::Dump | confirmed | Virtual slot [19]. Appends the 40-byte text Send OnReceiveFriendLogin UChat Command plus newline to the output string (FUN_01241650), then calls base dump FUN_… |
| `0152a730` | UChatCommand_OnReceiveFriendLogin::DumpTitle | confirmed | Virtual slot [20]. Appends the 39-byte text Send OnReceiveFriendLogin UChat Command to the output string passed as argument (FUN_01241650, count 0x27). Returns… |
| `0152a450` | UChatCommand_OnReceiveFriendLogin::QueueClientEvent | likely | Virtual slot 15: allocates 0x14 bytes with 00d833a6 and, when non-null, initialises the block with 012f9eb0; writes the decimal tag 199 (0xc7) with 012f8c70; c… |
| `0152a620` | UChatCommand_OnReceiveFriendLogin::Create | likely | Allocates 0x44 bytes with FUN_00d833a6. If the allocation returns 0 it returns 0; otherwise it constructs the block with FUN_0152a340 and returns the construct… |
| `0152a680` | UChatCommand_OnReceiveFriendLogin::RegisterFactory 🌐 | likely | Registers factory 0152a620 under key 499 (0x1f3) by calling FUN_013ce130 with (499, FUN_0152a620), then FUN_013ce230 with the same key and factory. Returns not… |
| `0152a3d0` | UChatCommand_OnReceiveFriendLogin::GetSharedDword | speculative | Virtual slot 12: returns the global dword DAT_01cbec34. |
| `0152a3e0` | UChatCommand_OnReceiveFriendLogin::SetSharedDword | speculative | Virtual slot 13: stores its stack argument into the global DAT_01cbec34. |

**UChatCommand_OnReceiveFriendLogout** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01529f30` | UChatCommand_OnReceiveFriendLogout::UChatCommand_OnReceiveF… | confirmed | Constructor body: runs the UChatCommand base constructor 013bd2e0 and stores this class vftable; it adds no fields of its own. |
| `01529fb0` | UChatCommand_OnReceiveFriendLogout::GetName | confirmed | Virtual slot 11: returns the address of the literal 'UChatCommand_OnReceiveFriendLogout'. |
| `01529ff0` | UChatCommand_OnReceiveFriendLogout::DestructorBody | confirmed | Stores this class vftable into *this, then calls 013bd090 (UChatCommand base teardown, a helper outside this shard). |
| `0152a150` | UChatCommand_OnReceiveFriendLogout::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target from NetworkCommand to this class (00d8d382); on failure reports the 'uchatCommand' assertion via 012f5a60; then call… |
| `0152a1a0` | UChatCommand_OnReceiveFriendLogout::Read | confirmed | Virtual slot 9: reads the header through this->vtable[4], then the base body 013bcf60, then the leaf body through this->vtable[5]. It reads no fields of its ow… |
| `0152a1e0` | UChatCommand_OnReceiveFriendLogout::Write | confirmed | Virtual slot 10: calls this->vtable[2] (header write), the base body 013bcfe0, then this->vtable[3] (body write). It writes no fields of its own. |
| `0152a290` | UChatCommand_OnReceiveFriendLogout::ScalarDeletingDtor | confirmed | Virtual slot 0: stores the class vftable, calls base teardown 013bd090, then calls _free(this) only when bit 0 of the argument is set; returns this. |
| `0152a2f0` | UChatCommand_OnReceiveFriendLogout::Dump | confirmed | Virtual slot 19 (debug dump, ret 4): appends 'Send OnReceiveFriendLogout UChat Command' with a newline to the output string, then calls base dump 013bd3f0 with… |
| `0152a320` | UChatCommand_OnReceiveFriendLogout::AppendDisplayName | confirmed | Virtual slot 20: moves its stack argument (a string object) into ECX and appends the 40-character literal 'Send OnReceiveFriendLogout UChat Command' with 01241… |
| `0152a040` | UChatCommand_OnReceiveFriendLogout::QueueClientEvent | likely | Virtual slot 15: allocates 0x14 bytes with 00d833a6 and, when non-null, initialises the block with 012f9eb0; writes the decimal tag 200 (0xc8) with 012f8c70; c… |
| `0152a0e0` | UChatCommand_OnReceiveFriendLogout::Clone | likely | Virtual slot 17: allocates 0x44 bytes with 00d833a6; when non-null constructs the object with 01529f30; then calls this->vtable[0x48/4] (CopyFieldsTo) with the… |
| `0152a210` | UChatCommand_OnReceiveFriendLogout::Create | likely | Allocates 0x44 bytes with 00d833a6; when non-null constructs the object with 01529f30 and returns it, otherwise returns 0. |
| `01529fc0` | UChatCommand_OnReceiveFriendLogout::GetSharedDword | speculative | Virtual slot 12: returns the global dword DAT_01cbec30. |
| `01529fd0` | UChatCommand_OnReceiveFriendLogout::SetSharedDword | speculative | Virtual slot 13: stores its stack argument into the global DAT_01cbec30. |

**UChatCommand_RequestAddFriend** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152bd30` | UChatCommand_RequestAddFriend::Deserialize | confirmed | Virtual slot [9]. Calls slot [4] with the stream; then base reader FUN_013bcf60; then strings into +0x44, +0x60 and +0x7c (FUN_013c28d0); then an integer into … |
| `0152bdc0` | UChatCommand_RequestAddFriend::Serialize | confirmed | Virtual slot [10]. Calls slot [2] with the stream, the base writer FUN_013bcfe0, the strings at +0x44, +0x60 and +0x7c (FUN_013c2860), the Confirm byte at +0x9… |
| `0152be20` | UChatCommand_RequestAddFriend::Ctor | confirmed | Runs base constructor FUN_013bd2e0, stores UChatCommand_RequestAddFriend::vftable at +0, sets the three string objects at +0x44, +0x60 and +0x7c to empty, and … |
| `0152bee0` | UChatCommand_RequestAddFriend::GetName | confirmed | Virtual slot [11]. Returns the address of the literal UChatCommand_RequestAddFriend (mov eax, 0x01914bf4; ret). |
| `0152bf20` | UChatCommand_RequestAddFriend::DtorBody | confirmed | Stores UChatCommand_RequestAddFriend::vftable into *this, releases the heap buffers of the strings at +0x7c, +0x60 and +0x44 when their capacity exceeds 15, re… |
| `0152bff0` | UChatCommand_RequestAddFriend::Clone | confirmed | Virtual slot [17]. Allocates 0x9c bytes (FUN_00d833a6); if non-null constructs the block with FUN_0152be20 and uses it, otherwise 0. Calls this object's slot [… |
| `0152c0f0` | UChatCommand_RequestAddFriend::ScalarDeletingDtor | confirmed | Virtual slot [0]. Calls the destructor body 0152bf20, then _free(this) when bit 0 of the flag argument is set. Returns this. |
| `0152c220` | UChatCommand_RequestAddFriend::CopyFieldsTo | confirmed | Virtual slot [18]. Dynamic-casts param_2 from NetworkCommand to UChatCommand_RequestAddFriend through FUN_00d8d382; asserts through FUN_012f5a60 (file UChatCom… |
| `0152c2b0` | UChatCommand_RequestAddFriend::Dump | confirmed | Virtual slot [19]. Appends Send RequestAddFriend UChat Command plus newline (count 0x24), calls base dump FUN_013bd3f0, then appends Dest: %s@%s (strings at +0… |
| `0152c3c0` | UChatCommand_RequestAddFriend::DumpTitle | confirmed | Virtual slot [20]. Appends the 35-byte text Send RequestAddFriend UChat Command (count 0x23) through FUN_01241650. |
| `0152c060` | UChatCommand_RequestAddFriend::Create | likely | Allocates 0x9c bytes with FUN_00d833a6. If the allocation returns 0 it returns 0; otherwise it constructs the block with FUN_0152be20 and returns it. |
| `0152c0d0` | UChatCommand_RequestAddFriend::RegisterFactory 🌐 | likely | Registers factory 0152c060 under key 0x1e9 (489) with FUN_013ce130 and then FUN_013ce230 using the same pair. Returns nothing. |
| `0152bc60` | UChatCommand_RequestAddFriend::EncodeMessage 🌐 | speculative | Virtual slot [15]. Allocates a 0x14-byte block (FUN_00d833a6); when that succeeds it calls FUN_012f9eb0 to zero it. It writes type id 0xc9 with FUN_012f8c70, a… |
| `0152c150` | UChatCommand_RequestAddFriend::FormatDestination | speculative | Builds a temporary string from the text at +0x44 followed by the separator at DAT_01800bfc (FUN_01249400), passes it with the text at +0x60 to FUN_012325d0 wit… |

**UChatCommand_RequestFriendConfirm** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152a870` | UChatCommand_RequestFriendConfirm::Deserialize | confirmed | Virtual slot [9]. Calls slot [4] with the stream; then base reader FUN_013bcf60; then an integer into +0x44 (FUN_013c24d0), strings into +0x4c and +0x68 (FUN_0… |
| `0152a900` | UChatCommand_RequestFriendConfirm::Serialize | confirmed | Virtual slot [10]. Calls slot [2] with the stream, the base writer FUN_013bcfe0, an integer from +0x44 (FUN_013c23f0), strings from +0x4c and +0x68 (FUN_013c28… |
| `0152a960` | UChatCommand_RequestFriendConfirm::Ctor | confirmed | Runs base constructor FUN_013bd2e0, stores UChatCommand_RequestFriendConfirm::vftable at +0, sets the two string objects at +0x4c and +0x68 to empty (capacity … |
| `0152aa00` | UChatCommand_RequestFriendConfirm::GetName | confirmed | Virtual slot [11]. Returns the address of the literal UChatCommand_RequestFriendConfirm (mov eax, 0x019148dc; ret). |
| `0152aa40` | UChatCommand_RequestFriendConfirm::DtorBody | confirmed | Stores UChatCommand_RequestFriendConfirm::vftable into *this, releases the heap buffers of the strings at +0x4c and +0x68 when their capacity exceeds 15, reset… |
| `0152aaf0` | UChatCommand_RequestFriendConfirm::Clone | confirmed | Virtual slot [17]. Allocates 0x84 bytes (FUN_00d833a6); if non-null constructs the block with FUN_0152a960 and uses it, otherwise 0. Calls this object's slot [… |
| `0152abf0` | UChatCommand_RequestFriendConfirm::ScalarDeletingDtor | confirmed | Virtual slot [0]. Calls the destructor body 0152aa40, then _free(this) when bit 0 of the flag argument is set. Returns this. |
| `0152ad00` | UChatCommand_RequestFriendConfirm::CopyFieldsTo | confirmed | Virtual slot [18]. Dynamic-casts param_2 from NetworkCommand to UChatCommand_RequestFriendConfirm through FUN_00d8d382 and asserts through FUN_012f5a60 (file U… |
| `0152ad80` | UChatCommand_RequestFriendConfirm::Dump | confirmed | Virtual slot [19]. Appends Send RequestFriendConfirm UChat Command plus newline, calls base dump FUN_013bd3f0, then formats and appends Dest AccountID: %d (int… |
| `0152ae70` | UChatCommand_RequestFriendConfirm::DumpTitle | confirmed | Virtual slot [20]. Appends the 39-byte text Send RequestFriendConfirm UChat Command (count 0x27) through FUN_01241650. |
| `0152ab60` | UChatCommand_RequestFriendConfirm::Create | likely | Allocates 0x84 bytes with FUN_00d833a6. If the allocation returns 0 it returns 0; otherwise it constructs the block with FUN_0152a960 and returns it. |
| `0152abd0` | UChatCommand_RequestFriendConfirm::RegisterFactory 🌐 | likely | Registers factory 0152ab60 under key 0x1eb (491) with FUN_013ce130 and then FUN_013ce230 using the same pair. Returns nothing. |
| `0152aa10` | UChatCommand_RequestFriendConfirm::GetSharedDword | speculative | Virtual slot [12]. Returns the 32-bit global DAT_01cbec38 (mov eax,[0x01cbec38]; ret). |
| `0152aa20` | UChatCommand_RequestFriendConfirm::SetSharedDword | speculative | Virtual slot [13]. Stores its 32-bit stack argument into the global DAT_01cbec38 (mov eax,[esp+4]; mov [0x01cbec38],eax; ret 4). |
| `0152ac50` | UChatCommand_RequestFriendConfirm::FormatDestination | speculative | Builds a temporary string from the text at +0x4c followed by the separator string at DAT_01800bfc (FUN_01249400), passes it with the text at +0x68 to FUN_01232… |
| `0152a7b0` | — 🌐 | unknown | Virtual slot [15]. Allocates a 0x14-byte block (FUN_00d833a6); when that succeeds it calls FUN_012f9eb0 to zero the block. It writes type id 0xd2 with FUN_012f… |

**UChatCommand_RequestFriendConfirmReciprocate** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013c0860` | UChatCommand_RequestFriendConfirmReciprocate::UChatCommand_… | confirmed | Constructor body: runs FUN_013bd2e0 for the base part, stores the UChatCommand_RequestFriendConfirmReciprocate vtable at +0, zeroes the two string objects at +… |
| `013c0900` | UChatCommand_RequestFriendConfirmReciprocate::GetName | confirmed | Virtual slot [11] of the class: returns the literal UChatCommand_RequestFriendConfirmReciprocate (bytes b8 ac 02 8e 01 c3: mov eax, 0x018e02ac; ret). |
| `013c0af0` | UChatCommand_RequestFriendConfirmReciprocate::ScalarDeletin… | confirmed | Virtual slot [0] of the class (bytes 56 8b f1 / e8 / f6 44 24 08 01 / 74 09 / 56 / e8 / 83 c4 04 / 8b c6 / 5e c3): saves esi, calls the destructor body FUN_013… |
| `013c0c00` | UChatCommand_RequestFriendConfirmReciprocate::CopyFieldsTo | confirmed | Virtual slot [18] of the class: dynamic-casts param_2 from NetworkCommand to UChatCommand_RequestFriendConfirmReciprocate via FUN_00d8d382 (RTTI_Type_Descripto… |
| `013c0c80` | UChatCommand_RequestFriendConfirmReciprocate::TraceDestinat… | confirmed | Virtual slot [19] of the class: writes the trace line Send RequestFriendConfirmReciprocate UChat Command (level 0x33) and calls FUN_013bd3f0(param_2); then for… |
| `013c0d70` | UChatCommand_RequestFriendConfirmReciprocate::TraceSend | confirmed | Virtual slot [20] of the class: calls FUN_01241650 with the string Send RequestFriendConfirmReciprocate UChat Command and the level 0x32, then returns (bytes 8… |
| `013c06b0` | UChatCommand_RequestFriendConfirmReciprocate::EncodeMessage | likely | Virtual slot [15] of the class. Allocates a 0x14-byte block (FUN_00d833a6) and, when it succeeds, initialises it with FUN_012f9eb0. It then writes the constant… |
| `013c0800` | UChatCommand_RequestFriendConfirmReciprocate::Serialize | likely | Virtual slot [10] of the class. Calls base slot [2] with param_2, then FUN_013bcfe0(param_2), appends the integer at +0x44 (FUN_013c23f0), the strings at +0x4c… |
| `013c0910` | UChatCommand_RequestFriendConfirmReciprocate::GetSharedDword | speculative | Virtual slot [12] of the class: returns the 32-bit global DAT_01cbdd74 (bytes a1 74 dd cb 01 c3: mov eax,[0x01cbdd74]; ret). Slot [13] writes the same global. |
| `013c0920` | UChatCommand_RequestFriendConfirmReciprocate::SetSharedDword | speculative | Virtual slot [13] of the class: stores its 32-bit argument into the global DAT_01cbdd74 (bytes 8b 44 24 04 a3 74 dd cb 01 c2 04 00: mov eax,[esp+4]; mov [0x01c… |
| `013c0770` | — | unknown | Virtual slot [9] of the class. Calls base slot [4] with (param_2, &local), then FUN_013bcf60(param_2), then reads in order: an integer into +0x44 (FUN_013c24d0… |
| `013c0940` | — | unknown | Destructor body: stores the class vtable at +0, releases the two string buffers of the strings at +0x4c and +0x68 (heap block when their capacity fields at +0x… |

**UChatCommand_RequestFriendStatus** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013ba740` | UChatCommand_RequestFriendStatus::~UChatCommand_RequestFrie… | confirmed | Destructor body: stores UChatCommand_RequestFriendStatus::vftable at +0, calls 013ba660 on the vector at +0x44 (destroying non-null elements through their vtab… |
| `013ba7e0` | UChatCommand_RequestFriendStatus::GetName | confirmed | Virtual slot [11] of the class: returns the literal string UChatCommand_RequestFriendStatus (mov eax, literal; ret; 6 bytes). |
| `013ba7f0` | UChatCommand_RequestFriendStatus::GetSharedDword | confirmed | Virtual slot [12] of the class: returns the 32-bit global DAT_01cbdd68 (mov eax,[DAT_01cbdd68]; ret). |
| `013ba800` | UChatCommand_RequestFriendStatus::SetSharedDword | confirmed | Virtual slot [13] of the class: stores its one stack argument into the global DAT_01cbdd68 (ret 4). |
| `013ba890` | UChatCommand_RequestFriendStatus::ScalarDeletingDtor | confirmed | Virtual slot [0] of the class: calls the destructor body 013ba740 on this, then calls _free(this) only when bit 0 of the argument is set; returns this. |
| `013ba8b0` | UChatCommand_RequestFriendStatus::TraceDump | confirmed | Virtual slot [19] of the class: appends the text Send RequestFriendStatus UChat Command with 01241650, calls the base trace 013bd3f0 with the out argument, the… |
| `013ba950` | UChatCommand_RequestFriendStatus::TraceSend | confirmed | Virtual slot [20] of the class: calls 01241650 with the literal Send RequestFriendStatus UChat Command and length 0x26, then returns. |
| `013baf40` | UChatCommand_RequestFriendStatus::Clone | confirmed | Virtual slot [17] of the class: allocates 0x54 bytes with operator new (00d833a6); when that succeeds it constructs the object with 013baee0; then calls this o… |
| `013bafb0` | UChatCommand_RequestFriendStatus::CopyFieldsTo | confirmed | Virtual slot [18] of the class: dynamic-casts param_2 from NetworkCommand to UChatCommand_RequestFriendStatus through 00d8d382; when the cast fails it reports … |
| `013bb100` | UChatCommand_RequestFriendStatus::CreateDefault | likely | Allocates 0x54 bytes with operator new (00d833a6); when the allocation succeeds it constructs the command with 013baee0 and returns it, otherwise returns 0. |
| `013bb160` | UChatCommand_RequestFriendStatus::RegisterFactory | likely | Calls 013ce130 with the classID 0x1ec and the factory 013bb100, then 013ce230 with the returned registry object; returns nothing. |
| `013baee0` | — | unknown | Constructor body: runs 013bd2e0 (UChatCommand base constructor) on this, stores UChatCommand_RequestFriendStatus::vftable at +0, and zeroes the words at +0x48,… |

**UChatCommand_RequestIgnoreStatus** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013bc260` | UChatCommand_RequestIgnoreStatus::~UChatCommand_RequestIgno… | confirmed | Destructor body: stores UChatCommand_RequestIgnoreStatus::vftable at +0, calls 013bc180 on the vector at +0x44 (element teardown), releases the buffer at +0x48… |
| `013bc300` | UChatCommand_RequestIgnoreStatus::GetName | confirmed | Virtual slot [11] of the class: returns the literal UChatCommand_RequestIgnoreStatus (6 bytes: mov eax, literal; ret). |
| `013bc310` | UChatCommand_RequestIgnoreStatus::GetSharedDword | confirmed | Virtual slot [12] of the class: returns the 32-bit global DAT_01cbdd6c. |
| `013bc320` | UChatCommand_RequestIgnoreStatus::SetSharedDword | confirmed | Virtual slot [13] of the class: stores its one stack argument into the global DAT_01cbdd6c (ret 4). |
| `013bc3b0` | UChatCommand_RequestIgnoreStatus::ScalarDeletingDtor | confirmed | Virtual slot [0] of the class: calls the destructor body 013bc260 on this, then calls _free(this) only when bit 0 of the argument is set; returns this. |
| `013bc3d0` | UChatCommand_RequestIgnoreStatus::TraceDump | confirmed | Virtual slot [19] of the class: appends the text Send RequestIgnoreStatus UChat Command with 01241650, calls the base trace 013bd3f0, then formats Ignore List:… |
| `013bc470` | UChatCommand_RequestIgnoreStatus::TraceSend | confirmed | Virtual slot [20] of the class: calls 01241650 with the literal Send RequestIgnoreStatus UChat Command and length 0x26, then returns. |
| `013bca00` | UChatCommand_RequestIgnoreStatus::UChatCommand_RequestIgnor… | confirmed | Constructor body: runs 013bd2e0 (UChatCommand base constructor) on this, stores UChatCommand_RequestIgnoreStatus::vftable at +0, and zeroes the words at +0x48,… |
| `013bca60` | UChatCommand_RequestIgnoreStatus::Clone | confirmed | Virtual slot [17] of the class: allocates 0x54 bytes with operator new (00d833a6); on success constructs the object with 013bca00 and calls this object's slot … |
| `013bcad0` | UChatCommand_RequestIgnoreStatus::CopyFieldsTo | confirmed | Virtual slot [18] of the class: dynamic-casts param_2 from NetworkCommand to this class through 00d8d382; on failure reports the uchatCommand assert (012f5a60)… |
| `013bcc20` | UChatCommand_RequestIgnoreStatus::CreateDefault | likely | Allocates 0x54 bytes with operator new (00d833a6); on success constructs the command with 013bca00 and returns it; returns 0 when allocation fails. |
| `013bcc80` | UChatCommand_RequestIgnoreStatus::RegisterFactory | likely | Calls 013ce130 with the classID 0x1ed and the factory 013bcc20, then 013ce230 with the registry result; returns nothing. |

**UChatCommand_RequestRemoveFriend** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152af70` | UChatCommand_RequestRemoveFriend::Deserialize | confirmed | Virtual slot [9]. Calls slot [4] with the stream; then base reader FUN_013bcf60; then strings into +0x44 and +0x60 (FUN_013c28d0); then calls slot [5] and retu… |
| `0152afd0` | UChatCommand_RequestRemoveFriend::Serialize | confirmed | Virtual slot [10]. Calls slot [2] with the stream, the base writer FUN_013bcfe0, the strings at +0x44 and +0x60 (FUN_013c2860), then slot [3]. |
| `0152b010` | UChatCommand_RequestRemoveFriend::Ctor | confirmed | Runs base constructor FUN_013bd2e0, stores UChatCommand_RequestRemoveFriend::vftable at +0, and sets the two string objects at +0x44 and +0x60 to empty (capaci… |
| `0152b0a0` | UChatCommand_RequestRemoveFriend::GetName | confirmed | Virtual slot [11]. Returns the address of the literal UChatCommand_RequestRemoveFriend (mov eax, 0x019149fc; ret). |
| `0152b0e0` | UChatCommand_RequestRemoveFriend::DtorBody | confirmed | Stores UChatCommand_RequestRemoveFriend::vftable into *this, releases the heap buffers of the strings at +0x44 and +0x60 when their capacity exceeds 15, resets… |
| `0152b180` | UChatCommand_RequestRemoveFriend::Clone | confirmed | Virtual slot [17]. Allocates 0x7c bytes (FUN_00d833a6); if non-null constructs the block with FUN_0152b010 and uses it, otherwise 0. Calls this object's slot [… |
| `0152b270` | UChatCommand_RequestRemoveFriend::ScalarDeletingDtor | confirmed | Virtual slot [0]. Calls the destructor body 0152b0e0, then _free(this) when bit 0 of the flag argument is set. Returns this. |
| `0152b380` | UChatCommand_RequestRemoveFriend::CopyFieldsTo | confirmed | Virtual slot [18]. Dynamic-casts param_2 from NetworkCommand to UChatCommand_RequestRemoveFriend through FUN_00d8d382; asserts through FUN_012f5a60 (file UChat… |
| `0152b3f0` | UChatCommand_RequestRemoveFriend::Dump | confirmed | Virtual slot [19]. Appends Send RequestRemoveFriend UChat Command plus newline (count 0x27), calls base dump FUN_013bd3f0, then appends Dest: %s@%s plus newlin… |
| `0152b4b0` | UChatCommand_RequestRemoveFriend::DumpTitle | confirmed | Virtual slot [20]. Appends the 38-byte text Send RequestRemoveFriend UChat Command (count 0x26) through FUN_01241650. |
| `0152b1f0` | UChatCommand_RequestRemoveFriend::Create | likely | Allocates 0x7c bytes with FUN_00d833a6. If the allocation returns 0 it returns 0; otherwise it constructs the block with FUN_0152b010 and returns it. |
| `0152b250` | UChatCommand_RequestRemoveFriend::RegisterFactory 🌐 | likely | Registers factory 0152b1f0 under key 0x1ee (494) with FUN_013ce130 and then FUN_013ce230 using the same pair. Returns nothing. |
| `0152b2d0` | UChatCommand_RequestRemoveFriend::FormatDestination | speculative | Builds a temporary string from the text at +0x44 followed by the separator at DAT_01800bfc (FUN_01249400), passes it with the text at +0x60 to FUN_012325d0 wit… |
| `0152aeb0` | — 🌐 | unknown | Virtual slot [15]. Allocates a 0x14-byte block (FUN_00d833a6); when that succeeds it calls FUN_012f9eb0 to zero it. It writes type id 0xcd with FUN_012f8c70, a… |

**UChatCommand_RequestSetFriendComment** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152b5c0` | UChatCommand_RequestSetFriendComment::Deserialize | confirmed | Virtual slot [9]. Calls slot [4] with the stream; then base reader FUN_013bcf60; then strings into +0x44, +0x60 and +0x7c (FUN_013c28d0); then calls slot [5] a… |
| `0152b630` | UChatCommand_RequestSetFriendComment::Serialize | confirmed | Virtual slot [10]. Calls slot [2] with the stream, the base writer FUN_013bcfe0, the strings at +0x44, +0x60 and +0x7c (FUN_013c2860), then slot [3]. |
| `0152b680` | UChatCommand_RequestSetFriendComment::Ctor | confirmed | Runs base constructor FUN_013bd2e0, stores UChatCommand_RequestSetFriendComment::vftable at +0, and sets the three string objects at +0x44, +0x60 and +0x7c to … |
| `0152b730` | UChatCommand_RequestSetFriendComment::GetName | confirmed | Virtual slot [11]. Returns the address of the literal UChatCommand_RequestSetFriendComment (mov eax, 0x01914ac4; ret). |
| `0152b770` | UChatCommand_RequestSetFriendComment::DtorBody | confirmed | Stores UChatCommand_RequestSetFriendComment::vftable into *this, releases the heap buffers of the strings at +0x7c, +0x60 and +0x44 when their capacity exceeds… |
| `0152b840` | UChatCommand_RequestSetFriendComment::Clone | confirmed | Virtual slot [17]. Allocates 0x98 bytes (FUN_00d833a6); if non-null constructs the block with FUN_0152b680 and uses it, otherwise 0. Calls this object's slot [… |
| `0152b940` | UChatCommand_RequestSetFriendComment::ScalarDeletingDtor | confirmed | Virtual slot [0]. Calls the destructor body 0152b770, then _free(this) when bit 0 of the flag argument is set. Returns this. |
| `0152ba70` | UChatCommand_RequestSetFriendComment::CopyFieldsTo | confirmed | Virtual slot [18]. Dynamic-casts param_2 from NetworkCommand to UChatCommand_RequestSetFriendComment through FUN_00d8d382; asserts through FUN_012f5a60 (file U… |
| `0152baf0` | UChatCommand_RequestSetFriendComment::Dump | confirmed | Virtual slot [19]. Appends Send RequestSetFriendComment UChat Command plus newline (count 0x2b), calls base dump FUN_013bd3f0, then appends Dest: %s@%s (string… |
| `0152bbf0` | UChatCommand_RequestSetFriendComment::DumpTitle | confirmed | Virtual slot [20]. Appends the 42-byte text Send RequestSetFriendComment UChat Command (count 0x2a) through FUN_01241650. |
| `0152b8b0` | UChatCommand_RequestSetFriendComment::Create | likely | Allocates 0x98 bytes with FUN_00d833a6. If the allocation returns 0 it returns 0; otherwise it constructs the block with FUN_0152b680 and returns it. |
| `0152b920` | UChatCommand_RequestSetFriendComment::RegisterFactory 🌐 | likely | Registers factory 0152b8b0 under key 0x1f0 (496) with FUN_013ce130 and then FUN_013ce230 using the same pair. Returns nothing. |
| `0152b9a0` | UChatCommand_RequestSetFriendComment::FormatDestination | speculative | Builds a temporary string from the text at +0x44 followed by the separator at DAT_01800bfc (FUN_01249400), passes it with the text at +0x60 to FUN_012325d0 wit… |
| `0152b500` | — 🌐 | unknown | Virtual slot [15]. Allocates a 0x14-byte block (FUN_00d833a6); when that succeeds it calls FUN_012f9eb0 to zero it. It writes type id 0xcf with FUN_012f8c70, a… |

**(no class)** (35)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0092f380` | SendChatBubbleColor 🌐 | confirmed | Constructs CommandPacketSetChatBubbleColor with three arguments and two global values, sends it through 00929560, then if param_1+200 exists stores the three s… |
| `00b117a0` | DeserializeTellEchoBody 🌐 | confirmed | Reads the bounded 16-bit header fields, a signed byte count, and that many bytes when they fit; otherwise sets the reader error flag and clamps the cursor to t… |
| `00b12ad0` | WriteTwoU16Fields 🌐 | confirmed | Copies up to two bytes from the low 16 bits of this+4 into the writer, advances its cursor, then repeats for the low 16 bits of this+8. |
| `00b140c0` | ReadPacketChatPayload 🌐 | confirmed | Builds a bounded reader over param_2 and param_3, calls 00b11600 to decode the body, then checks reader error state and optional exact end-of-buffer consumptio… |
| `00b14130` | ReadTellEchoPayload 🌐 | confirmed | Builds a bounded reader over param_2 and param_3, calls 00b117a0 to decode the body, then checks reader error state and optional exact end-of-buffer consumptio… |
| `00b14350` | BaseChatPacketReceiveDispatcher 🌐 | confirmed | Constructs a BaseChatPacket, parses the bounded input, switches on payload modes 1 through 5 to construct and decode concrete packets, routes parsed messages t… |
| `00b15340` | SendChatPacket 🌐 | speculative | Chat send helper called by every chat command handler: allocates a 0x203c-byte stack frame, initializes a local serialization buffer (008d4a40 with 0x10000000)… |
| `00be83c0` | ChatWStringFieldUpperProcess | speculative | When 008f11e0 returns 1, copies the wide string at param_1+4 into a local SoeUtil::IString<unsigned_short> through 00701290, calls 00836c70 on that string, cal… |
| `013b4230` | AssignStringAt0x14 | speculative | Assigns its string argument (0, npos) to the std::string at this+0x14 through 00420ff1 and returns. |
| `013bd520` | SetStringAt0C | speculative | Assigns the argument std::string to the string at this+0x0c (assign(src, 0, npos)). |
| `013bd540` | SetStringAt28 | speculative | Assigns the argument std::string to the string at this+0x28 (assign(src, 0, npos)). |
| `013bd560` | ConcatFieldsWithSep | speculative | Builds a temporary string through 01249400 from the string at this+0x0c and the constant DAT_01800bfc, then writes the concatenation with the string at this+0x… |
| `013c0230` | NameLookupOrInsert | speculative | When the object at param_2 has no text in +0x14, it takes the text from FUN_013a3a30(); otherwise it copies param_2 itself. It then tests that text with FUN_01… |
| `013c06a0` | SetFlagByte48 | speculative | Stores the 1-byte argument param_2 at this+0x48 and returns (bytes 8a 44 24 04 / 88 41 48 / c2 04 00: mov al,[esp+4]; mov [ecx+0x48],al; ret 4). |
| `013c0ad0` | RegisterFactory0x1fe | speculative | Bytes 68 60 0a 3c 01 / 68 fe 01 00 00 / e8 / 8b c8 / e8 / c3: pushes the address of FUN_013c0a60 and the key 0x1fe, calls FUN_013ce130(0x1fe, FUN_013c0a60), mo… |
| `013c0b10` | SetStringAt4C | speculative | Assigns the argument string to the std::string at param_1+0x4c (assign(src, 0, npos); bytes 8b 44 24 04 / 6a ff 6a 00 50 / 83 c1 4c / e8 / c2 04 00). |
| `013c0b30` | SetStringAt68 | speculative | Assigns the argument string to the std::string at param_1+0x68 (assign(src, 0, npos); bytes 8b 44 24 04 / 6a ff 6a 00 50 / 83 c1 68 / e8 / c2 04 00). |
| `013c0d90` | GetSubobjectAt4 | speculative | Returns this+4 (bytes 8d 41 04 c3: lea eax,[ecx+4]; ret). |
| `013c0da0` | GetSubobjectAt20 | speculative | Returns this+0x20 (bytes 8d 41 20 c3: lea eax,[ecx+0x20]; ret). |
| `013c0db0` | GetSubobjectAt3C | speculative | Returns this+0x3c (bytes 8d 41 3c c3: lea eax,[ecx+0x3c]; ret). |
| `013c0dc0` | GetSubobjectAt58 | speculative | Returns this+0x58 (bytes 8d 41 58 c3: lea eax,[ecx+0x58]; ret). |
| `013c0dd0` | SetFlagByte74 | speculative | Stores the 1-byte argument at this+0x74 and returns (bytes 8a 44 24 04 / 88 41 74 / c2 04 00: mov al,[esp+4]; mov [ecx+0x74],al; ret 4). |
| `013c13d0` | FieldPairMatchEitherOrder | speculative | Returns 1 when (this+0x04 matches param_2 and this+0x3c matches param_4) or the swapped pairing (this+0x04 with param_4 and this+0x3c with param_2), using FUN_… |
| `013c1480` | SetByteAt3D | speculative | Stores the 1-byte argument at this+0x3d (bytes 8a 44 24 04 / 88 41 3d / c2 04 00: mov al,[esp+4]; mov [ecx+0x3d],al; ret 4). |
| `0152b0b0` | UChatCommand_RequestRemoveFriend::GetSharedDword | speculative | Virtual slot [12]. Returns the 32-bit global DAT_01cbec3c (mov eax,[0x01cbec3c]; ret). |
| `0152b0c0` | UChatCommand_RequestRemoveFriend::SetSharedDword | speculative | Virtual slot [13]. Stores its 32-bit stack argument into the global DAT_01cbec3c (mov eax,[esp+4]; mov [0x01cbec3c],eax; ret 4). |
| `0152b740` | UChatCommand_RequestSetFriendComment::GetSharedDword | speculative | Virtual slot [12]. Returns the 32-bit global DAT_01cbec40 (mov eax,[0x01cbec40]; ret). |
| `0152b750` | UChatCommand_RequestSetFriendComment::SetSharedDword | speculative | Virtual slot [13]. Stores its 32-bit stack argument into the global DAT_01cbec40 (mov eax,[esp+4]; mov [0x01cbec40],eax; ret 4). |
| `0152bef0` | UChatCommand_RequestAddFriend::GetSharedDword | speculative | Virtual slot [12]. Returns the 32-bit global DAT_01cbec44 (mov eax,[0x01cbec44]; ret). |
| `0152bf00` | UChatCommand_RequestAddFriend::SetSharedDword | speculative | Virtual slot [13]. Stores its 32-bit stack argument into the global DAT_01cbec44 (mov eax,[esp+4]; mov [0x01cbec44],eax; ret 4). |
| `013bfce0` | — | unknown | Walks the list of entries whose begin and end pointers are at the object's +0x20 and +0x24 and tests each entry with FUN_013c13d0 (FieldPairMatchEitherOrder) a… |
| `013c0590` | — | unknown | Allocates a 0x78-byte block with FUN_00d833a6 and, on success, constructs the record with FUN_013c0e20 (ecx = record). It then calls the record setters FUN_013… |
| `013c09f0` | — | unknown | Allocates a 0x84-byte block with FUN_00d833a6; if the allocation succeeds it constructs a UChatCommand_RequestFriendConfirmReciprocate with FUN_013c0860 (other… |
| `013c0a60` | — | unknown | Allocates a 0x84-byte block with FUN_00d833a6 and, when that succeeds, constructs a UChatCommand_RequestFriendConfirmReciprocate with FUN_013c0860 and returns … |
| `013c0b50` | — | unknown | Builds a temporary std::string with FUN_01249400(temp, this+0x4c, &DAT_01800bfc, cookie), then calls FUN_012325d0(param_2, temp, this+0x68) to combine it with … |

### `net.friends`

**FriendStatus** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0143f6e0` | FriendStatus::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls slot 4 on this with (param_2, &param_2) and returns 0 when that fails; then SerialReadInteger on +0x4 (013… |
| `0143f760` | FriendStatus::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls slot 2 on this with param_2, SerialAppendInteger on +0x4 (013c23f0), SerialAppendString on +0x8, +0x24 an… |
| `0143f880` | FriendStatus::TypeName | confirmed | Virtual slot 11 with no arguments. Returns the address of the literal "FriendStatus" (018ea114). |
| `0143fb00` | FriendStatus::DumpText | confirmed | Virtual slot 14 (thiscall, this in ECX, one stack argument: the std::string that receives text). Appends the line FriendStatus, then lines formatted with __snp… |
| `0143fc70` | FriendStatus::AppendTypeName | confirmed | Virtual slot 15 (one stack argument, ret 4). Loads ECX from the argument (the std::string that receives text) and appends the 12-byte literal FriendStatus (018… |
| `0143f890` | FriendStatus::GetStaticDword | speculative | Virtual slot 12 with no arguments. Returns the dword stored in the global DAT_01cbe3d0. |
| `0143f8a0` | FriendStatus::SetStaticDword | speculative | Virtual slot 13 with one argument. Stores the argument into the global dword DAT_01cbe3d0 (the value read by slot 12). |

**IgnoreStatus** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0143fd20` | IgnoreStatus::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls slot 4 on this with (param_2, &param_2) and returns 0 when that fails; then SerialReadInteger on +0x4 (013… |
| `0143fd80` | IgnoreStatus::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls slot 2 on this with param_2, SerialAppendInteger on +0x4 (013c23f0), SerialAppendString on +0x8 and +0x24… |
| `0143fe60` | IgnoreStatus::TypeName | confirmed | Virtual slot 11 with no arguments. Returns the address of the literal IgnoreStatus (018ea1a4). |
| `014400a0` | IgnoreStatus::DumpText | confirmed | Virtual slot 14 (thiscall, this in ECX, one stack argument: the std::string that receives text). Appends the line IgnoreStatus, then __snprintf lines: AccountI… |
| `01440190` | IgnoreStatus::AppendTypeName | confirmed | Virtual slot 15 (one stack argument, ret 4). Loads ECX from the argument (the std::string that receives text) and appends the 12-byte literal IgnoreStatus (018… |
| `0143fe70` | IgnoreStatus::GetStaticDword | speculative | Virtual slot 12 with no arguments. Returns the dword stored in the global DAT_01cbe3d4. |
| `0143fe80` | IgnoreStatus::SetStaticDword | speculative | Virtual slot 13 with one argument. Stores the argument into the global dword DAT_01cbe3d4 (the value read by slot 12). |

**UChatCommand_RequestAddFriendReciprocate** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01440f90` | UChatCommand_RequestAddFriendReciprocate::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls slot 4 on this with the reader and an address of a local (fails with 0 when false); calls base-class read … |
| `01441020` | UChatCommand_RequestAddFriendReciprocate::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls slot 2 on this with param_2, calls base-class write 013bcfe0, SerialAppendString on +0x44, +0x60 and +0x7… |
| `01441140` | UChatCommand_RequestAddFriendReciprocate::TypeName | confirmed | Virtual slot 11 with no arguments. Returns the address of the literal UChatCommand_RequestAddFriendReciprocate (018ea46c). |
| `01441480` | UChatCommand_RequestAddFriendReciprocate::CopyFieldsTo | confirmed | Virtual slot 18 (thiscall: this is the source, one stack argument is the destination NetworkCommand). Casts the destination with the RTTI dynamic_cast 00d8d382… |
| `01441510` | UChatCommand_RequestAddFriendReciprocate::DumpText | confirmed | Virtual slot 19 (thiscall, one stack argument: the std::string that receives text). Appends the line Send RequestAddFriend UChat Command with a newline, calls … |
| `01441620` | UChatCommand_RequestAddFriendReciprocate::AppendDisplayName | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument (the std::string receiving text) and appends the 35-byte literal Send RequestAddFriend… |
| `01440ec0` | UChatCommand_RequestAddFriendReciprocate::Handle | likely | Virtual slot 15 (fastcall, this in ECX). Allocates a 0x14-byte block with operator new (00d833a6); when non-null builds it with 012f9eb0. Then calls 012f8c70 w… |
| `01441150` | UChatCommand_RequestAddFriendReciprocate::GetStaticDword | speculative | Virtual slot 12 with no arguments. Returns the dword stored in the global DAT_01cbe3e0. |
| `01441160` | UChatCommand_RequestAddFriendReciprocate::SetStaticDword | speculative | Virtual slot 13 with one argument. Stores the argument into the global dword DAT_01cbe3e0. |

**UChatCommand_RequestAddIgnore** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01441720` | UChatCommand_RequestAddIgnore::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls slot 4 on this with (param_2, &param_2) and returns 0 when that fails; calls base-class read 013bcf60 with… |
| `01441780` | UChatCommand_RequestAddIgnore::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls slot 2 on this with param_2, calls base-class write 013bcfe0, SerialAppendString on +0x44 and +0x60 (013c… |
| `01441850` | UChatCommand_RequestAddIgnore::TypeName | confirmed | Virtual slot 11 with no arguments. Returns the address of the literal UChatCommand_RequestAddIgnore (018ea5b4). |
| `01441b30` | UChatCommand_RequestAddIgnore::CopyFieldsTo | confirmed | Virtual slot 18 (thiscall: this is the source, one stack argument is the destination NetworkCommand). Casts the destination with the RTTI dynamic_cast 00d8d382… |
| `01441ba0` | UChatCommand_RequestAddIgnore::DumpText | confirmed | Virtual slot 19 (thiscall, this in ECX, one stack argument: the std::string that receives text). Appends the line Send RequestAddIgnore UChat Command with a ne… |
| `01441c50` | UChatCommand_RequestAddIgnore::AppendDisplayName | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument (the std::string receiving text) and appends the 35-byte literal Send RequestAddIgnore… |
| `01441660` | UChatCommand_RequestAddIgnore::Handle | likely | Virtual slot 15 (fastcall, this in ECX). Allocates a 0x14-byte block with operator new (00d833a6); when non-null builds it with 012f9eb0. Then calls 012f8c70 w… |
| `01441860` | UChatCommand_RequestAddIgnore::GetStaticDword | speculative | Virtual slot 12 with no arguments. Returns the dword stored in the global DAT_01cbe3e4. |
| `01441870` | UChatCommand_RequestAddIgnore::SetStaticDword | speculative | Virtual slot 13 with one argument. Stores the argument into the global dword DAT_01cbe3e4. |

**UChatCommand_RequestRemoveFriendReciprocate** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014402d0` | UChatCommand_RequestRemoveFriendReciprocate::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls slot 4 on this with (param_2, &param_2) and returns 0 when that fails; calls 013bcf60 with the reader (bas… |
| `01440330` | UChatCommand_RequestRemoveFriendReciprocate::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls slot 2 on this with param_2, calls base-class write 013bcfe0 with the writer, SerialAppendString on the s… |
| `01440400` | UChatCommand_RequestRemoveFriendReciprocate::TypeName | confirmed | Virtual slot 11 with no arguments. Returns the address of the literal UChatCommand_RequestRemoveFriendReciprocate (018ea224). |
| `014406e0` | UChatCommand_RequestRemoveFriendReciprocate::CopyFieldsTo | confirmed | Virtual slot 18 (thiscall: this is the source, one stack argument is the destination NetworkCommand). Casts the destination with the RTTI dynamic_cast 00d8d382… |
| `01440750` | UChatCommand_RequestRemoveFriendReciprocate::DumpText | confirmed | Virtual slot 19 (thiscall, this in ECX, one stack argument: the std::string that receives text). Appends the line Send RequestRemoveFriend UChat Command with a… |
| `01440810` | UChatCommand_RequestRemoveFriendReciprocate::AppendDisplayN… | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument (the std::string receiving text) and appends the 38-byte literal Send RequestRemoveFri… |
| `01440210` | UChatCommand_RequestRemoveFriendReciprocate::Handle | likely | Virtual slot 15 (fastcall, this in ECX). Allocates a 0x14-byte block with operator new (00d833a6); when non-null builds it with 012f9eb0. Then calls 012f8c70 w… |
| `01440410` | UChatCommand_RequestRemoveFriendReciprocate::GetStaticDword | speculative | Virtual slot 12 with no arguments. Returns the dword stored in the global DAT_01cbe3d8. |
| `01440420` | UChatCommand_RequestRemoveFriendReciprocate::SetStaticDword | speculative | Virtual slot 13 with one argument. Stores the argument into the global dword DAT_01cbe3d8. |

**UChatCommand_RequestRemoveIgnore** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01440910` | UChatCommand_RequestRemoveIgnore::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls slot 4 on this with (param_2, &param_2) and returns 0 when that fails; calls base-class read 013bcf60 with… |
| `01440970` | UChatCommand_RequestRemoveIgnore::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls slot 2 on this with param_2, calls base-class write 013bcfe0 with the writer, SerialAppendString on the s… |
| `01440a40` | UChatCommand_RequestRemoveIgnore::TypeName | confirmed | Virtual slot 11 with no arguments. Returns the address of the literal UChatCommand_RequestRemoveIgnore (018ea354). |
| `01440d20` | UChatCommand_RequestRemoveIgnore::CopyFieldsTo | confirmed | Virtual slot 18 (thiscall: this is the source, one stack argument is the destination NetworkCommand). Casts the destination with the RTTI dynamic_cast 00d8d382… |
| `01440d90` | UChatCommand_RequestRemoveIgnore::DumpText | confirmed | Virtual slot 19 (thiscall, this in ECX, one stack argument: the std::string that receives text). Appends the line Send RequestRemoveIgnore UChat Command with a… |
| `01440e50` | UChatCommand_RequestRemoveIgnore::AppendDisplayName | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument (the std::string receiving text) and appends the 38-byte literal Send RequestRemoveIgn… |
| `01440850` | UChatCommand_RequestRemoveIgnore::Handle | likely | Virtual slot 15 (fastcall, this in ECX). Allocates a 0x14-byte block with operator new (00d833a6); when non-null builds it with 012f9eb0. Then calls 012f8c70 w… |
| `01440a50` | UChatCommand_RequestRemoveIgnore::GetStaticDword | speculative | Virtual slot 12 with no arguments. Returns the dword stored in the global DAT_01cbe3dc. |
| `01440a60` | UChatCommand_RequestRemoveIgnore::SetStaticDword | speculative | Virtual slot 13 with one argument. Stores the argument into the global dword DAT_01cbe3dc. |

### `net.group`

**(no class)** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7e1c0` | SendGroupKickPacketForMemberRow 🌐 | likely | Gets the selected row value from the datasource at (this+0x254)+0x5d4, resolves its entity through 0102d6b0, and proceeds only when that entity and the group a… |
| `00a7e2e0` | SendGroupKickPacketForGuid 🌐 | likely | Constructs a GroupPacketGroupKick temporary through 008dd060. If the group at owner+0x6ec exists, resolves the supplied guid pair through 0102d620; for a resol… |
| `00c24d20` | SendGroupInvite | likely | Reads one indexed string, constructs GroupPacketGroupInvite, sets a second string from static data, invokes a local helper, and wraps the packet when service i… |
| `00a7e110` | SendGroupLeavePacketIfConnected 🌐 | speculative | Constructs a GroupPacketGroupLeave temporary through 008dcfe0. When (this+0x254)+0x280 is non-null, passes the temporary and flag 1 to sender 00a7dea0; then de… |
| `00938650` | — | unknown | Constructs GroupMapPingPacket and, when the gateway pointer at this+0x280 is non-null, serializes and submits it through the gateway wrapper. |
| `00a7dd50` | — | unknown | Writes the record's two sub-objects at source+0x10 and source+0xa0 into a byte writer (FUN_0090ae60 begins it, FUN_008fb810 writes each sub-object, FUN_00a0e8f… |
| `00a7dea0` | — 🌐 | unknown | Initializes a stack writer, serializes the source subobject at +0x10, finalizes the payload, and constructs a ClientGateway::PacketTunneledClientPacket wrapper… |
| `00a7dfc0` | — 🌐 | unknown | Initializes a stack writer, serializes the source record through 00a7dd50, finalizes the payload, and constructs a ClientGateway::PacketTunneledClientPacket wr… |

### `net.guild`

**BaseGuildPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a151d0` | — | unknown | Destructor body of a guild packet: resets the IString at +0x18, releases its shared buffer when the refcount at buffer-4 drops to zero via virtual slot 2, then… |

**Guild** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f8f80` | Guild::Create | likely | No arguments. Allocates 0x54 bytes with operator new (00d833a6); when the allocation succeeds it calls the Guild constructor 014f8af0 on the block and returns … |
| `014f8fe0` | Guild::RegisterFactory | likely | Pushes the factory 014f8f80 and the constant 0x184, calls 013ce130, moves its EAX into ECX and calls 013ce230, which stores the factory under key 0x184 (ret 8)… |

**GuildCommand** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d6640` | GuildCommand::GuildCommand | confirmed | Constructor body: calls 0140c090 (base-chain setup, also used by the NetworkCommand/PersistentBase constructors), installs GuildCommand::vftable and sets this+… |
| `014d66f0` | GuildCommand::GetName | confirmed | Returns the address of the string literal GuildCommand (mov eax, imm32; ret). |
| `014d6730` | GuildCommand::DtorBody | confirmed | Sets the vtable to GuildCommand::vftable, then calls 0140c160 (the NetworkCommand/PersistentBase destructor chain) and returns. It is the cleanup target of 46 … |
| `014d67e0` | GuildCommand::Clone | confirmed | Allocates 8 bytes (00d833a6). When that succeeds it constructs the new object with 014d6640. It then calls virtual slot [18] (0x48) on this with the new pointe… |
| `014d6850` | GuildCommand::CopyFieldsTo | confirmed | Calls 0140c280 (a stub that only returns) on its argument, dynamic-casts the argument from NetworkCommand to GuildCommand (00d8d382), and stores this+4 into th… |
| `014d6880` | GuildCommand::Deserialize | confirmed | Calls vtable slot 4 (0x10) on this with the buffer (header read) and returns 0 on failure; calls 0140c2a0 on the buffer and returns 0 on failure; reads an int … |
| `014d68d0` | GuildCommand::Serialize | confirmed | Calls vtable slot 2 (0x08) with the buffer (header write), calls 013c26e0 (no-op returning 1) with the text Starting GuildCommand, calls 0140c2d0, calls 013c26… |
| `014d69b0` | GuildCommand::ScalarDeletingDtor | confirmed | Sets the vtable to GuildCommand::vftable, calls 0140c160 (the destructor chain), then calls _free(this) only when bit 0 of the flag argument is set. Returns th… |
| `014d6a10` | GuildCommand::TraceFields | confirmed | Calls 0140c290 (a stub that only returns) on the output argument, formats AccountID: %d with this+4 into a 1000-byte local with _sprintf, and appends the forma… |
| `014d6a90` | GuildCommand::TraceName | confirmed | Appends the 13-character text Guild Command (01241650, length 0xd) to the output string and returns (ret 4). |
| `014d6700` | GuildCommand::GetSharedDword | likely | Returns the dword in global DAT_01cbe840 (mov eax, [0x01cbe840]; ret). |
| `014d6710` | GuildCommand::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe840 (ret 4). |
| `014d67a0` | GuildCommand::GetFrameWords | likely | Writes 0x3b9aca03 to the first out-pointer and 0 to the second and third (stdcall, ret 12). It is the shared slot-21 implementation for the GuildCommand_* clas… |
| `014d6930` | GuildCommand::FactoryCreate | likely | Allocates 8 bytes (00d833a6) and, when the allocation is non-null, constructs a GuildCommand with 014d6640. Returns the object, or 0 when the allocation failed. |
| `014d6990` | GuildCommand::RegisterFactory | likely | Pushes the creator 014d6930 and the classID 0x1a1 and calls 013ce130; then passes the value it returned in ECX to 013ce230. |

**GuildCommand_AskToJoinGuild** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d6150` | GuildCommand_AskToJoinGuild::GuildCommand_AskToJoinGuild | confirmed | Constructor body: calls 014d6640 (GuildCommand base constructor), installs GuildCommand_AskToJoinGuild::vftable and zeroes the ints at +0x8, +0xc and +0x10. Re… |
| `014d61d0` | GuildCommand_AskToJoinGuild::GetName | confirmed | Returns the address of the string literal GuildCommand_AskToJoinGuild (mov eax, imm32; ret). |
| `014d6210` | GuildCommand_AskToJoinGuild::DtorBody | confirmed | Sets the vtable to GuildCommand_AskToJoinGuild::vftable, calls 014d6730 (base destructor body) and returns. |
| `014d6380` | GuildCommand_AskToJoinGuild::Clone | confirmed | Allocates 0x14 bytes (00d833a6). When that succeeds it constructs the new object with 014d6150. It then calls virtual slot [18] (0x48) on this with the new poi… |
| `014d63f0` | GuildCommand_AskToJoinGuild::CopyFieldsTo | confirmed | Dynamic-casts its argument from NetworkCommand to GuildCommand_AskToJoinGuild (00d8d382); on failure reports the clone assertion (012f5a60, line 0x54). It then… |
| `014d6590` | GuildCommand_AskToJoinGuild::ScalarDeletingDtor | confirmed | Sets the vtable to GuildCommand_AskToJoinGuild::vftable, calls 014d6730 (base destructor body), then calls _free(this) only when bit 0 of the flag argument is … |
| `014d65f0` | GuildCommand_AskToJoinGuild::TraceFields | confirmed | Appends the text Ask To Join Guild Guild Command followed by a newline (01241650, length 0x20) to the output string, then calls the base trace helper 014d6a10 … |
| `014d6620` | GuildCommand_AskToJoinGuild::TraceName | confirmed | Appends the 31-character text Ask To Join Guild Guild Command (01241650, length 0x1f) to the string passed in and returns (ret 4). |
| `014d61e0` | GuildCommand_AskToJoinGuild::GetSharedDword | likely | Returns the dword in global DAT_01cbe83c (mov eax, [0x01cbe83c]; ret). |
| `014d61f0` | GuildCommand_AskToJoinGuild::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe83c (ret 4). |
| `014d62c0` | GuildCommand_AskToJoinGuild::EncodeMessage | likely | Calls 012d46a0 (returns the ClientApplication singleton), calls 012d08e0 on it and, when that is non-zero, calls 013a3480 on the result and compares it with th… |
| `014d6510` | GuildCommand_AskToJoinGuild::FactoryCreate | likely | Allocates 0x14 bytes (00d833a6) and, when the allocation is non-null, constructs the object with 014d6150. Returns the object, or 0 when the allocation failed. |
| `014d6570` | GuildCommand_AskToJoinGuild::RegisterFactory | likely | Pushes the creator 014d6510 and the classID 0x1a8 and calls 013ce130; then passes the value it returned in ECX to 013ce230. |
| `014d6450` | — | unknown | Reads the class header through vtable slot 4 (0x10) with an out-pointer to the buffer argument, calls 014d6880 and returns 0 when it fails. It then reads three… |
| `014d64c0` | — | unknown | Calls vtable slot 2 (0x08) with the buffer (header write), calls 014d68d0 (base trace helper), writes the three ints at +0x8, +0xc and +0x10 through 013c23f0, … |

**GuildCommand_AskToJoinGuildResponse** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d5d00` | GuildCommand_AskToJoinGuildResponse::GuildCommand_AskToJoin… | confirmed | Constructor body: calls 014d6640 (the GuildCommand base constructor), installs GuildCommand_AskToJoinGuildResponse::vftable, zeroes the ints at +0x8 and +0xc a… |
| `014d5d80` | GuildCommand_AskToJoinGuildResponse::GetName | confirmed | Returns the address of the string literal GuildCommand_AskToJoinGuildResponse (mov eax, imm32; ret). |
| `014d5dc0` | GuildCommand_AskToJoinGuildResponse::DtorBody | confirmed | Sets the vtable to GuildCommand_AskToJoinGuildResponse::vftable, calls 014d6730 (base destructor body) and returns. |
| `014d5e80` | GuildCommand_AskToJoinGuildResponse::Clone | confirmed | Allocates 0x14 bytes (00d833a6). When that succeeds it constructs the new object with 014d5d00. It then calls virtual slot [18] (0x48) on this with the new poi… |
| `014d5ef0` | GuildCommand_AskToJoinGuildResponse::CopyFieldsTo | confirmed | Dynamic-casts its argument from NetworkCommand to GuildCommand_AskToJoinGuildResponse (00d8d382); on failure it reports the clone assertion (012f5a60). It then… |
| `014d60a0` | GuildCommand_AskToJoinGuildResponse::ScalarDeletingDtor | confirmed | Sets the vtable to GuildCommand_AskToJoinGuildResponse::vftable, calls 014d6730 (base destructor body), then calls _free(this) only when bit 0 of the flag argu… |
| `014d6100` | GuildCommand_AskToJoinGuildResponse::TraceFields | confirmed | Appends the text Ask To Join Guild Response Guild Command followed by a newline (01241650, length 0x29) to the output string, then calls the base trace helper … |
| `014d6130` | GuildCommand_AskToJoinGuildResponse::TraceName | confirmed | Appends the 40-character text Ask To Join Guild Response Guild Command (01241650, length 0x28) to the string passed in and returns (ret 4). |
| `014d5d90` | GuildCommand_AskToJoinGuildResponse::GetSharedDword | likely | Returns the dword in global DAT_01cbe838 (mov eax, [0x01cbe838]; ret). |
| `014d5da0` | GuildCommand_AskToJoinGuildResponse::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe838 (ret 4). |
| `014d6020` | GuildCommand_AskToJoinGuildResponse::FactoryCreate | likely | Allocates 0x14 bytes (00d833a6) and, when the allocation is non-null, constructs the object with 014d5d00. Returns the object, or 0 when the allocation failed. |
| `014d6080` | GuildCommand_AskToJoinGuildResponse::RegisterFactory | likely | Pushes the creator 014d6020 and the classID 0x1a9 and calls 013ce130; then passes the value it returned in ECX to 013ce230. |
| `014d5f50` | — | unknown | Reads the class header through vtable slot 4 (0x10) with an out value; calls 014d6880 and returns 0 when it fails. It then reads an int into +0x8 (013c24d0), r… |
| `014d5fd0` | — | unknown | Calls vtable slot 2 (0x08) with the buffer (header write), calls 014d68d0 (base trace helper), writes the int at +0x8, the byte at +0x10 (as an int) and the in… |

**GuildCommand_Chat** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d55e0` | GuildCommand_Chat::EncodeMessage | confirmed | Allocates a 0x14-byte block (00d833a6); when it is non-null, zeroes it (012f9eb0) and writes the word 0xb5 into it (012f8c70). It then appends nine arguments i… |
| `014d5820` | GuildCommand_Chat::GuildCommand_Chat | confirmed | Constructor body: calls 014d6640 (the GuildCommand base constructor), installs GuildCommand_Chat::vftable, sets the capacity words of its three std::string mem… |
| `014d58e0` | GuildCommand_Chat::GetName | confirmed | Returns the address of the string literal GuildCommand_Chat (mov eax, 0x01904ac4; ret). |
| `014d5920` | GuildCommand_Chat::DtorBody | confirmed | Sets the vtable to GuildCommand_Chat::vftable, frees each of the three std::string buffers whose capacity is above 15 (_free), resets them to empty, calls 014d… |
| `014d59e0` | GuildCommand_Chat::Clone | confirmed | Allocates 0x70 bytes (00d833a6). When that succeeds it constructs the new object with 014d5820. It then calls virtual slot [18] (0x48) on this with the new poi… |
| `014d5ad0` | GuildCommand_Chat::ScalarDeletingDtor | confirmed | Calls the destructor body 014d5920 on this, then calls _free(this) only when bit 0 of the flag argument is set (test byte [esp+8],1). Returns this (ret 4). |
| `014d5b50` | GuildCommand_Chat::CopyFieldsTo | confirmed | Dynamic-casts its argument from NetworkCommand to GuildCommand_Chat (00d8d382); on failure it reports the clone assertion (012f5a60). It then calls the base co… |
| `014d5bf0` | GuildCommand_Chat::TraceFields | confirmed | Appends the text Guild Chat Guild Command and a newline (01241650), calls the base trace helper 014d6a10 (GuildCommand::TraceFields, the AccountID line) with t… |
| `014d5ce0` | GuildCommand_Chat::TraceName | confirmed | Pushes the length 0x18 and the literal Guild Chat Guild Command, then calls 01241650 with the string object taken from the single stack argument (ECX). Returns… |
| `014d58f0` | GuildCommand_Chat::GetSharedDword | likely | Returns the dword in global DAT_01cbe834 (mov eax, [0x01cbe834]; ret). |
| `014d5900` | GuildCommand_Chat::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe834 (mov eax,[esp+4]; mov [0x01cbe834],eax; ret 4). |
| `014d5a50` | GuildCommand_Chat::FactoryCreate | likely | Allocates 0x70 bytes (00d833a6) and, when the allocation is non-null, constructs a GuildCommand_Chat in it with 014d5820. Returns the object, or 0 when the all… |
| `014d5ab0` | GuildCommand_Chat::RegisterFactory | likely | Pushes the creator 014d5a50 and the classID 0x1d0 and calls 013ce130; then passes the value it returned in ECX to 013ce230 with no further pushes. |
| `014d56c0` | — | unknown | Reads the class header through vtable slot 4 (0x10) and returns 0 on failure. It then reads ints into +0x8, +0xc, +0x10 and +0x14 (013c24d0), strings into +0x1… |
| `014d57a0` | — | unknown | Calls vtable slot 2 (0x08) on this with the buffer argument (class header write), calls 014d68d0 (base trace helper), writes the ints at +0x8, +0xc, +0x10 and … |

**GuildCommand_CreateGuild** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d5120` | GuildCommand_CreateGuild::Read | confirmed | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4), the base read 014d6880, reads the title std::string into this+8 (013c28d0), then calls slot … |
| `014d5170` | GuildCommand_CreateGuild::Write | confirmed | Virtual slot 10 (+0x28), write body. Calls the header write (slot 2), the base write 014d68d0, writes the title string at this+8 (013c2860), then calls slot 3 … |
| `014d51c0` | GuildCommand_CreateGuild::Ctor | confirmed | Fastcall constructor: calls base constructor 014d6640 with ECX=this, stores GuildCommand_CreateGuild::vftable into *this, sets the std::string at this+8 to emp… |
| `014d5250` | GuildCommand_CreateGuild::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the C string literal GuildCommand_CreateGuild. |
| `014d5290` | GuildCommand_CreateGuild::DtorBody | confirmed | Fastcall destructor body: stores GuildCommand_CreateGuild::vftable into *this, frees the title buffer at this+0xc when capacity (this+0x20) exceeds 15 (_free),… |
| `014d5310` | GuildCommand_CreateGuild::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 0x24 bytes through 00d833a6; when non-null it constructs the object with 014d51c0, then calls this->vtable[+0x48] (CopyTo) o… |
| `014d5400` | GuildCommand_CreateGuild::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x00). Calls the destructor body 014d5290 on this, frees this when bit 0 of the flag argument is set (_free), and returns this. |
| `014d5440` | GuildCommand_CreateGuild::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. RTTI-casts the destination to CreateGuild (00d8d382); on null calls the assert helper… |
| `014d54a0` | GuildCommand_CreateGuild::Dump | confirmed | Virtual slot 19 (+0x4c). Appends Create Guild Guild Command newline (0x1b bytes) through 01241650, appends Title: label, then the title string at this+8 throug… |
| `014d54f0` | GuildCommand_CreateGuild::AppendDisplayName | confirmed | Virtual slot 20 (+0x50). Appends the literal Create Guild Guild Command (0x1a bytes) to the std::string in ECX through 01241650 (stack argument, ret 4). |
| `014d5380` | GuildCommand_CreateGuild::Factory | likely | Allocates 0x24 bytes through 00d833a6; when non-null it constructs the object with 014d51c0 and returns it, otherwise returns 0. |
| `014d53e0` | GuildCommand_CreateGuild::RegisterFactory | likely | Registers the factory 014d5380 under classID 0x1a6 through 013ce130 and 013ce230. |
| `014d5260` | GuildCommand_CreateGuild::ReadGlobal01cbe830 | speculative | Virtual slot 12 (+0x30). No arguments; returns the 32-bit global DAT_01cbe830. |
| `014d5270` | GuildCommand_CreateGuild::WriteGlobal01cbe830 | speculative | Virtual slot 13 (+0x34). One stack argument: stores it into the global DAT_01cbe830 (ret 4). |

**GuildCommand_EndGuilds** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d4d20` | GuildCommand_EndGuilds::Ctor | confirmed | Fastcall constructor: calls base constructor 014d6640 with ECX=this and stores GuildCommand_EndGuilds::vftable into *this. The command has no payload fields. |
| `014d4da0` | GuildCommand_EndGuilds::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the C string literal GuildCommand_EndGuilds. |
| `014d4de0` | GuildCommand_EndGuilds::DtorBody | confirmed | Fastcall destructor body: stores GuildCommand_EndGuilds::vftable into *this, then calls the base destructor body 014d6730 on this. |
| `014d4eb0` | GuildCommand_EndGuilds::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 8 bytes through 00d833a6; when non-null it constructs the object with 014d4d20, then calls this->vtable[+0x48] (CopyTo) on t… |
| `014d4f20` | GuildCommand_EndGuilds::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. Calls the base copy 014d6850 on the destination, then RTTI-casts it to EndGuilds (00d… |
| `014d4f50` | GuildCommand_EndGuilds::Read | confirmed | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4) with a local out-pointer, then the base read 014d6880, then calls slot 5 (+0x14) and returns … |
| `014d4f90` | GuildCommand_EndGuilds::Write | confirmed | Virtual slot 10 (+0x28), write body. Calls the header write (slot 2), trace text (013c26e0, no-op), the base write 014d68d0, trace text again, then slot 3 (+0x… |
| `014d5060` | GuildCommand_EndGuilds::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x00). Stores the vftable into *this, calls the base destructor 014d6730, frees this when bit 0 of the flag argument is set (_free), and retur… |
| `014d50c0` | GuildCommand_EndGuilds::Dump | confirmed | Virtual slot 19 (+0x4c). Appends End Guilds Guild Command newline (0x19 bytes) through 01241650, then calls the base dump 014d6a10. |
| `014d50f0` | GuildCommand_EndGuilds::AppendDisplayName | confirmed | Virtual slot 20 (+0x50). Appends the literal End Guilds Guild Command (0x18 bytes) to the std::string in ECX through 01241650 (stack argument, ret 4). |
| `014d4e30` | GuildCommand_EndGuilds::PostClientEvent | likely | Virtual slot 15 (+0x3c). Calls 012d46a0 and allocates 0x14 bytes, constructs the block with 012f9eb0 when allocated, sets its type word to 0x97 (012f8c70), pus… |
| `014d4fe0` | GuildCommand_EndGuilds::Factory | likely | Allocates 8 bytes through 00d833a6; when non-null it constructs the object with 014d4d20 and returns it, otherwise returns 0. |
| `014d5040` | GuildCommand_EndGuilds::RegisterFactory | likely | Registers the factory 014d4fe0 under classID 0x1a2 through 013ce130 and 013ce230. |
| `014d4db0` | GuildCommand_EndGuilds::ReadGlobal01cbe82c | speculative | Virtual slot 12 (+0x30). No arguments; returns the 32-bit global DAT_01cbe82c (mov eax,[01cbe82c]; ret). |
| `014d4dc0` | GuildCommand_EndGuilds::WriteGlobal01cbe82c | speculative | Virtual slot 13 (+0x34). One stack argument: stores it into the global DAT_01cbe82c (ret 4). |

**GuildCommand_GetGuildsByPartialName** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d4530` | GuildCommand_GetGuildsByPartialName::Write | confirmed | Virtual slot 10 (+0x28), write body. Calls the header write (slot 2), trace text (013c26e0, no-op), the base write 014d68d0, writes the partial name string at … |
| `014d4700` | GuildCommand_GetGuildsByPartialName::Dump | confirmed | Virtual slot 19 (+0x4c). Appends the header newline through 01241650, calls the base dump 014d6a10, formats Partial Guild Name: and the string at this+0x2c or … |
| `014d47e0` | GuildCommand_GetGuildsByPartialName::AppendDisplayName | confirmed | Virtual slot 20 (+0x50). Appends the literal Get Guild By Partial Name Guild Command (0x27 bytes) to the std::string in ECX through 01241650 (stack argument, r… |
| `014d4800` | GuildCommand_GetGuildsByPartialName::Ctor | confirmed | Fastcall constructor: calls base constructor 014d6640 with ECX=this, stores GuildCommand_GetGuildsByPartialName::vftable into *this, zeroes the vector headers … |
| `014d48a0` | GuildCommand_GetGuildsByPartialName::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the C string literal GuildCommand_GetGuildsByPartialName. |
| `014d48e0` | GuildCommand_GetGuildsByPartialName::DtorBody | confirmed | Fastcall destructor body: stores the class vftable into *this, frees the string buffer at this+0x2c when capacity (this+0x40) exceeds 15, resets the string, de… |
| `014d49b0` | GuildCommand_GetGuildsByPartialName::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 0x48 bytes through 00d833a6; when non-null it constructs the object with 014d4800, then calls this->vtable[+0x48] (CopyTo) o… |
| `014d4aa0` | GuildCommand_GetGuildsByPartialName::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x00). Calls the destructor body 014d48e0 on this, frees this when bit 0 of the flag argument is set (_free), and returns this. |
| `014d4ac0` | GuildCommand_GetGuildsByPartialName::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. RTTI-casts the destination to GetGuildsByPartialName (00d8d382); on null calls the as… |
| `014d4490` | GuildCommand_GetGuildsByPartialName::PostClientEvent | likely | Virtual slot 15 (+0x3c). Allocates 0x14 bytes and constructs the block with 012f9eb0, sets the event type word to 0xb2 (012f8c70), stores the int at this+0x44 … |
| `014d4a20` | GuildCommand_GetGuildsByPartialName::Factory | likely | Allocates 0x48 bytes through 00d833a6; when non-null it constructs the object with 014d4800 and returns it, otherwise returns 0. |
| `014d4a80` | GuildCommand_GetGuildsByPartialName::RegisterFactory | likely | Registers the factory 014d4a20 under classID 0x1cd through 013ce130 and 013ce230. |
| `014d48b0` | GuildCommand_GetGuildsByPartialName::ReadGlobal01cbe828 | speculative | Virtual slot 12 (+0x30). No arguments; returns the 32-bit global DAT_01cbe828. |
| `014d48c0` | GuildCommand_GetGuildsByPartialName::WriteGlobal01cbe828 | speculative | Virtual slot 13 (+0x34). One stack argument: stores it into the global DAT_01cbe828 (ret 4). |
| `014d4b70` | — | unknown | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4) with a local, the base read 014d6880, reads the partial name string into this+0x28 (013c28d0)… |

**GuildCommand_Kick** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d4010` | GuildCommand_Kick::Read | confirmed | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4), the base read 014d6880, reads one integer into this+8 (013c24d0), reads a std::string into t… |
| `014d4070` | GuildCommand_Kick::Write | confirmed | Virtual slot 10 (+0x28), write body. Calls the header write (slot 2), the base write 014d68d0, writes the integer this+8 (013c23f0) and the string at this+0xc … |
| `014d40b0` | GuildCommand_Kick::Ctor | confirmed | Fastcall constructor: calls base constructor 014d6640 with ECX=this, stores GuildCommand_Kick::vftable into *this, sets the std::string at this+0xc to empty (s… |
| `014d4140` | GuildCommand_Kick::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the C string literal GuildCommand_Kick (mov eax,01904634; ret). |
| `014d4180` | GuildCommand_Kick::DtorBody | confirmed | Fastcall destructor body: stores GuildCommand_Kick::vftable into *this, frees the string buffer at this+0x10 when its capacity (this+0x24) exceeds 15 (_free), … |
| `014d4200` | GuildCommand_Kick::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 0x28 bytes through 00d833a6; when non-null it constructs the object with 014d40b0, then calls this->vtable[+0x48] (CopyTo) o… |
| `014d42f0` | GuildCommand_Kick::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x00). Calls the destructor body 014d4180 on this, frees this when bit 0 of the flag argument is set (_free), and returns this. |
| `014d43a0` | GuildCommand_Kick::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. RTTI-casts the destination to Kick (00d8d382); on null calls the assert helper 012f5a… |
| `014d4400` | GuildCommand_Kick::Dump | confirmed | Virtual slot 19 (+0x4c). Appends Guild Kick Guild Command newline (0x19 bytes) through 01241650, calls the base dump 014d6a10, appends Name: through 01241650, … |
| `014d4440` | GuildCommand_Kick::AppendDisplayName | confirmed | Virtual slot 20 (+0x50). Appends the literal Guild Kick Guild Command (0x18 bytes) to the std::string in ECX through 01241650 (stack argument, ret 4). |
| `014d4270` | GuildCommand_Kick::Factory | likely | Allocates 0x28 bytes through 00d833a6; when non-null it constructs the object with 014d40b0 and returns it, otherwise returns 0. |
| `014d42d0` | GuildCommand_Kick::RegisterFactory | likely | Registers the factory 014d4270 under classID 0x1aa through 013ce130 and 013ce230. |
| `014d4150` | GuildCommand_Kick::ReadGlobal01cbe824 | speculative | Virtual slot 12 (+0x30). No arguments; returns the 32-bit global DAT_01cbe824 (mov eax,[01cbe824]; ret). |
| `014d4160` | GuildCommand_Kick::WriteGlobal01cbe824 | speculative | Virtual slot 13 (+0x34). One stack argument: stores it into the global DAT_01cbe824 (ret 4). |

**GuildCommand_LeaveGuild** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d3c20` | GuildCommand_LeaveGuild::Ctor | confirmed | Fastcall constructor: calls base constructor 014d6640 with ECX=this, stores GuildCommand_LeaveGuild::vftable into *this and zeroes the dword at this+8. |
| `014d3ca0` | GuildCommand_LeaveGuild::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the C string literal GuildCommand_LeaveGuild. |
| `014d3ce0` | GuildCommand_LeaveGuild::DtorBody | confirmed | Fastcall destructor body: stores GuildCommand_LeaveGuild::vftable into *this, then calls the base destructor body 014d6730 on this. |
| `014d3d60` | GuildCommand_LeaveGuild::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 0xc bytes through 00d833a6; when non-null it constructs the object with 014d3c20, then calls this->vtable[+0x48] (CopyTo) on… |
| `014d3dd0` | GuildCommand_LeaveGuild::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. RTTI-casts the destination to LeaveGuild (00d8d382); on null calls the assert helper … |
| `014d3e20` | GuildCommand_LeaveGuild::Read | confirmed | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4), the base read 014d6880, reads one integer into this+8 (013c24d0), then calls slot 5 (+0x14) … |
| `014d3e70` | GuildCommand_LeaveGuild::Write | confirmed | Virtual slot 10 (+0x28), write body. Calls the header write (slot 2), the base write 014d68d0, writes the integer this+8 (013c23f0), then calls slot 3 (+0xc). |
| `014d3f30` | GuildCommand_LeaveGuild::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x00). Stores the vftable into *this, calls the base destructor 014d6730, frees this when bit 0 of the flag argument is set (_free), and retur… |
| `014d3f90` | GuildCommand_LeaveGuild::Dump | confirmed | Virtual slot 19 (+0x4c). Appends Leave Guild Guild Command newline (0x1a bytes) through 01241650, then calls the base dump 014d6a10. |
| `014d3fc0` | GuildCommand_LeaveGuild::AppendDisplayName | confirmed | Virtual slot 20 (+0x50). Appends the literal Leave Guild Guild Command (0x19 bytes) to the std::string in ECX through 01241650 (stack argument, ret 4). |
| `014d3eb0` | GuildCommand_LeaveGuild::Factory | likely | Allocates 0xc bytes through 00d833a6; when non-null it constructs the object with 014d3c20 and returns it, otherwise returns 0. |
| `014d3f10` | GuildCommand_LeaveGuild::RegisterFactory | likely | Registers the factory 014d3eb0 under classID 0x1a7 through 013ce130 and 013ce230. |
| `014d3cb0` | GuildCommand_LeaveGuild::ReadGlobal01cbe820 | speculative | Virtual slot 12 (+0x30). No arguments; returns the 32-bit global DAT_01cbe820. |
| `014d3cc0` | GuildCommand_LeaveGuild::WriteGlobal01cbe820 | speculative | Virtual slot 13 (+0x34). One stack argument: stores it into the global DAT_01cbe820 (ret 4). |

**GuildCommand_ModifyGuildStatus** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d3720` | GuildCommand_ModifyGuildStatus::Ctor | confirmed | Fastcall constructor: calls base constructor 014d6640 with ECX=this, stores GuildCommand_ModifyGuildStatus::vftable into *this and zeroes the dwords at this+8,… |
| `014d37a0` | GuildCommand_ModifyGuildStatus::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the C string literal GuildCommand_ModifyGuildStatus. |
| `014d37e0` | GuildCommand_ModifyGuildStatus::DtorBody | confirmed | Fastcall destructor body: stores GuildCommand_ModifyGuildStatus::vftable into *this, then calls the base destructor body 014d6730 on this. |
| `014d38a0` | GuildCommand_ModifyGuildStatus::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 0x14 bytes through 00d833a6; when non-null it constructs the object with 014d3720, then calls this->vtable[+0x48] (CopyTo) o… |
| `014d3910` | GuildCommand_ModifyGuildStatus::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. RTTI-casts the destination to ModifyGuildStatus (00d8d382); on null calls the assert … |
| `014d3970` | GuildCommand_ModifyGuildStatus::Read | confirmed | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4), the base read 014d6880, reads three integers into this+8, +0xc and +0x10 (013c24d0), then ca… |
| `014d39e0` | GuildCommand_ModifyGuildStatus::Write | confirmed | Virtual slot 10 (+0x28), write body. Calls the header write (slot 2), the base write 014d68d0, writes the integers this+8, +0xc and +0x10 (013c23f0), then call… |
| `014d3ab0` | GuildCommand_ModifyGuildStatus::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x00). Stores the vftable into *this, calls the base destructor 014d6730, frees this when bit 0 of the flag argument is set (_free), and retur… |
| `014d3b10` | GuildCommand_ModifyGuildStatus::Dump | confirmed | Virtual slot 19 (+0x4c). Appends the header Guild ModifyGuildStatus Guild Command newline through 01241650, calls the base dump 014d6a10, formats this+0xc with… |
| `014d3c00` | GuildCommand_ModifyGuildStatus::AppendDisplayName | confirmed | Virtual slot 20 (+0x50). Appends the literal Guild ModifyGuildStatus Guild Command (0x25 bytes) to the std::string in ECX through 01241650 (stack argument, ret… |
| `014d3a30` | GuildCommand_ModifyGuildStatus::Factory | likely | Allocates 0x14 bytes through 00d833a6; when non-null it constructs the object with 014d3720 and returns it, otherwise returns 0. |
| `014d3a90` | GuildCommand_ModifyGuildStatus::RegisterFactory | likely | Registers the factory 014d3a30 under classID 0x1d1 through 013ce130 and 013ce230. |
| `014d37b0` | GuildCommand_ModifyGuildStatus::ReadGlobal01cbe81c | speculative | Virtual slot 12 (+0x30). No arguments; returns the 32-bit global DAT_01cbe81c. |
| `014d37c0` | GuildCommand_ModifyGuildStatus::WriteGlobal01cbe81c | speculative | Virtual slot 13 (+0x34). One stack argument: stores it into the global DAT_01cbe81c (ret 4). |

**GuildCommand_Notify** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d2ca0` | GuildCommand_Notify::Ctor | confirmed | Fastcall constructor: calls base constructor 014d6640 with ECX=this, stores GuildCommand_Notify::vftable into *this, zeroes the dword at this+8, the byte at th… |
| `014d2d30` | GuildCommand_Notify::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the C string literal GuildCommand_Notify. |
| `014d2d70` | GuildCommand_Notify::DtorBody | confirmed | Fastcall destructor body: stores GuildCommand_Notify::vftable into *this; when the pointer at this+0x14 is set calls its vtable slot 0 with flag 1 (deleting de… |
| `014d2ff0` | GuildCommand_Notify::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 0x1c bytes through 00d833a6; when non-null it constructs the object with 014d2ca0, then calls this->vtable[+0x48] (CopyTo) o… |
| `014d3060` | GuildCommand_Notify::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. RTTI-casts the destination to Notify (00d8d382); on null calls the assert helper 012f… |
| `014d3180` | GuildCommand_Notify::Write | confirmed | Virtual slot 10 (+0x28), write body. Calls the header write (slot 2), the base write 014d68d0, writes the trace literal Message through 013c26e0 (no-op), write… |
| `014d32c0` | GuildCommand_Notify::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x00). Calls the destructor body 014d2d70 on this, frees this when bit 0 of the flag argument is set (_free), and returns this. |
| `014d32e0` | GuildCommand_Notify::Read | confirmed | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4), the base read 014d6880, then reads an integer into this+8 (013c24d0), a bool into this+0xc f… |
| `014d3570` | GuildCommand_Notify::Dump | confirmed | Virtual slot 19 (+0x4c). Appends the header Info message Enum display Guild Command newline through 01241650, calls the base dump 014d6a10, appends Message: an… |
| `014d3700` | GuildCommand_Notify::AppendDisplayName | confirmed | Virtual slot 20 (+0x50). Appends the literal Notify Guild Command (0x14 bytes) to the std::string in ECX through 01241650 (stack argument, ret 4). |
| `014d3240` | GuildCommand_Notify::Factory | likely | Allocates 0x1c bytes through 00d833a6; when non-null it constructs the object with 014d2ca0 and returns it, otherwise returns 0. |
| `014d32a0` | GuildCommand_Notify::RegisterFactory | likely | Registers the factory 014d3240 under classID 0x1ab through 013ce130 and 013ce230. |
| `014d2d40` | GuildCommand_Notify::ReadGlobal01cbe818 | speculative | Virtual slot 12 (+0x30). No arguments; returns the 32-bit global DAT_01cbe818. |
| `014d2d50` | GuildCommand_Notify::WriteGlobal01cbe818 | speculative | Virtual slot 13 (+0x34). One stack argument: stores it into the global DAT_01cbe818 (ret 4). |
| `014d2f30` | — | unknown | Virtual slot 15 (+0x3c). Allocates 0x14 bytes and constructs the block with 012f9eb0; sets the event type to 0x9b when the byte at this+0xc is zero (0x9c other… |

**GuildCommand_Remove** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d2860` | GuildCommand_Remove::Ctor | confirmed | Fastcall constructor: calls base constructor 014d6640 with ECX=this, stores GuildCommand_Remove::vftable into *this and zeroes the dword at this+8. |
| `014d28e0` | GuildCommand_Remove::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the C string literal GuildCommand_Remove. |
| `014d2920` | GuildCommand_Remove::DtorBody | confirmed | Fastcall destructor body: stores GuildCommand_Remove::vftable into *this, then calls the base destructor body 014d6730 on this. |
| `014d2a20` | GuildCommand_Remove::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 0xc bytes through 00d833a6; when non-null it constructs the object with 014d2860, then calls this->vtable[+0x48] (CopyTo) on… |
| `014d2a90` | GuildCommand_Remove::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. RTTI-casts the destination to Remove (00d8d382); on null calls the assert helper 012f… |
| `014d2ae0` | GuildCommand_Remove::Read | confirmed | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4) with the stream, then the base read 014d6880, reads one integer into this+8 through 013c24d0,… |
| `014d2b30` | GuildCommand_Remove::Write | confirmed | Virtual slot 10 (+0x28), write body. Calls the header write (slot 2), the base write 014d68d0, writes the integer this+8 through 013c23f0, then calls slot 3 (+… |
| `014d2bf0` | GuildCommand_Remove::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x00). Stores the vftable into *this, calls the base destructor 014d6730, frees this when bit 0 of the flag argument is set (_free), and retur… |
| `014d2c50` | GuildCommand_Remove::Dump | confirmed | Virtual slot 19 (+0x4c). Appends Remove Guild Command newline (0x15 bytes) through 01241650, then calls the base dump 014d6a10. |
| `014d2c80` | GuildCommand_Remove::AppendDisplayName | confirmed | Virtual slot 20 (+0x50). Appends the literal Remove Guild Command (0x14 bytes) to the std::string in ECX through 01241650 (stack argument, ret 4). |
| `014d2990` | GuildCommand_Remove::PostClientEvent | likely | Virtual slot 15 (+0x3c). Calls 014f83f0 with the int at this+8 (its role is not labelled), allocates 0x14 bytes and constructs the block with 012f9eb0, sets th… |
| `014d2b70` | GuildCommand_Remove::Factory | likely | Allocates 0xc bytes through 00d833a6; when non-null it constructs the object with 014d2860 and returns it, otherwise returns 0. |
| `014d2bd0` | GuildCommand_Remove::RegisterFactory | likely | Registers the factory 014d2b70 under classID 0x1ac through 013ce130 and 013ce230. |
| `014d28f0` | GuildCommand_Remove::ReadGlobal01cbe814 | speculative | Virtual slot 12 (+0x30). No arguments; returns the 32-bit global DAT_01cbe814. |
| `014d2900` | GuildCommand_Remove::WriteGlobal01cbe814 | speculative | Virtual slot 13 (+0x34). One stack argument: stores it into the global DAT_01cbe814 (ret 4). |

**GuildCommand_RequestGuilds** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d2450` | GuildCommand_RequestGuilds::Ctor | confirmed | Fastcall constructor: calls base constructor 014d6640 with ECX=this, stores GuildCommand_RequestGuilds::vftable into *this and zeroes the dword at this+8. |
| `014d24d0` | GuildCommand_RequestGuilds::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the C string literal GuildCommand_RequestGuilds. |
| `014d2510` | GuildCommand_RequestGuilds::DtorBody | confirmed | Fastcall destructor body: stores GuildCommand_RequestGuilds::vftable into *this, then calls the base destructor body 014d6730 on this. |
| `014d2590` | GuildCommand_RequestGuilds::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 0xc bytes through 00d833a6; when non-null it constructs the object with 014d2450, then calls this->vtable[+0x48] (CopyTo) on… |
| `014d2600` | GuildCommand_RequestGuilds::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. Calls the base copy 014d6850 on the destination, RTTI-casts it to RequestGuilds (00d8… |
| `014d2630` | GuildCommand_RequestGuilds::Read | confirmed | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4) with a local out-pointer, then the base read 014d6880; the integer at this+8 is read through … |
| `014d2690` | GuildCommand_RequestGuilds::Write | confirmed | Virtual slot 10 (+0x28), write body. Calls the header write (slot 2), the base write 014d68d0, writes the integer at this+8 (013c23f0) unconditionally, then ca… |
| `014d2750` | GuildCommand_RequestGuilds::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x00). Stores the vftable into *this, calls the base destructor 014d6730, frees this when bit 0 of the flag argument is set (_free), and retur… |
| `014d27b0` | GuildCommand_RequestGuilds::Dump | confirmed | Virtual slot 19 (+0x4c). Appends Request Guilds Guild Command newline through 01241650, calls the base dump 014d6a10, then formats GuildID: %d newline with thi… |
| `014d2840` | GuildCommand_RequestGuilds::AppendDisplayName | confirmed | Virtual slot 20 (+0x50). Appends the literal Request Guilds Guild Command (0x1c bytes) to the std::string in ECX through 01241650 (stack argument, ret 4). |
| `014d26d0` | GuildCommand_RequestGuilds::Factory | likely | Allocates 0xc bytes through 00d833a6; when non-null it constructs the object with 014d2450 and returns it, otherwise returns 0. |
| `014d2730` | GuildCommand_RequestGuilds::RegisterFactory | likely | Registers the factory 014d26d0 under classID 0x1a3 through 013ce130 and 013ce230. |
| `014d24e0` | GuildCommand_RequestGuilds::ReadGlobal01cbe810 | speculative | Virtual slot 12 (+0x30). No arguments; returns the 32-bit global DAT_01cbe810. |
| `014d24f0` | GuildCommand_RequestGuilds::WriteGlobal01cbe810 | speculative | Virtual slot 13 (+0x34). One stack argument: stores it into the global DAT_01cbe810 (ret 4). |

**GuildCommand_SetListenStatus** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d2080` | GuildCommand_SetListenStatus::Ctor | confirmed | Fastcall constructor: calls base constructor 014d6640 with ECX=this, stores GuildCommand_SetListenStatus::vftable into *this and zeroes the byte at this+8. |
| `014d2100` | GuildCommand_SetListenStatus::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the C string literal GuildCommand_SetListenStatus. |
| `014d2140` | GuildCommand_SetListenStatus::DtorBody | confirmed | Fastcall destructor body: stores GuildCommand_SetListenStatus::vftable into *this, then calls the base destructor body 014d6730 on this. |
| `014d21c0` | GuildCommand_SetListenStatus::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 0xc bytes through 00d833a6; when non-null it constructs the object with 014d2080, then calls this->vtable[+0x48] (CopyTo) on… |
| `014d2230` | GuildCommand_SetListenStatus::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. RTTI-casts the destination to SetListenStatus (00d8d382); on null calls the assert he… |
| `014d2280` | GuildCommand_SetListenStatus::Read | confirmed | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4) on the object, the base read 014d6880, reads one integer through 013c24d0 into a local and st… |
| `014d22e0` | GuildCommand_SetListenStatus::Write | confirmed | Virtual slot 10 (+0x28), write body. Calls the header write (slot 2), the base write 014d68d0, writes the byte at this+8 as one integer through 013c23f0, then … |
| `014d23a0` | GuildCommand_SetListenStatus::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x00). Stores the vftable into *this, calls the base destructor 014d6730, frees this when bit 0 of the flag argument is set (_free), and retur… |
| `014d2400` | GuildCommand_SetListenStatus::Dump | confirmed | Virtual slot 19 (+0x4c). Appends Set Listen Status Guild Command newline (0x20 bytes) through 01241650, then calls the base dump 014d6a10. |
| `014d2430` | GuildCommand_SetListenStatus::AppendDisplayName | confirmed | Virtual slot 20 (+0x50). Appends the literal Set Listen Status Guild Command (0x1f bytes) to the std::string in ECX through 01241650 (stack argument, ret 4). |
| `014d2320` | GuildCommand_SetListenStatus::Factory | likely | Allocates 0xc bytes through 00d833a6; when non-null it constructs the object with 014d2080 and returns it, otherwise returns 0. |
| `014d2380` | GuildCommand_SetListenStatus::RegisterFactory | likely | Registers the factory 014d2320 under classID 0x1ad through 013ce130 and 013ce230. |
| `014d2110` | GuildCommand_SetListenStatus::ReadGlobal01cbe80c | speculative | Virtual slot 12 (+0x30). No arguments; returns the 32-bit global DAT_01cbe80c. |
| `014d2120` | GuildCommand_SetListenStatus::WriteGlobal01cbe80c | speculative | Virtual slot 13 (+0x34). One stack argument: stores it into the global DAT_01cbe80c (ret 4). |

**GuildCommand_SetMOTD** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d1c60` | GuildCommand_SetMOTD::Read | confirmed | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4) on the object, the base read 014d6880, reads an integer into this+8 (013c24d0), a std::string… |
| `014d1ce0` | GuildCommand_SetMOTD::Write | confirmed | Virtual slot 10 (+0x28), write body. Calls the header write (slot 2), the base write 014d68d0, writes the integer this+8 (013c23f0), the string at this+0xc (01… |
| `014d1d30` | GuildCommand_SetMOTD::Ctor | confirmed | Fastcall constructor: calls base constructor 014d6640 with ECX=this, stores GuildCommand_SetMOTD::vftable into *this, sets the std::string at this+0xc to empty… |
| `014d1dc0` | GuildCommand_SetMOTD::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the C string literal GuildCommand_SetMOTD. |
| `014d1e00` | GuildCommand_SetMOTD::DtorBody | confirmed | Fastcall destructor body: stores GuildCommand_SetMOTD::vftable into *this, frees the text buffer at this+0x10 when its capacity (this+0x24) exceeds 15 (_free),… |
| `014d1e80` | GuildCommand_SetMOTD::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 0x2c bytes through 00d833a6; when non-null it constructs the object with 014d1d30, then calls this->vtable[+0x48] (CopyTo) o… |
| `014d1f70` | GuildCommand_SetMOTD::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x00). Calls the destructor body 014d1e00 on this, frees this when bit 0 of the flag argument is set (_free), and returns this. |
| `014d1fb0` | GuildCommand_SetMOTD::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. RTTI-casts the destination to SetMOTD (00d8d382); on null calls the assert helper 012… |
| `014d2020` | GuildCommand_SetMOTD::Dump | confirmed | Virtual slot 19 (+0x4c). Appends Guild SetMOTD Guild Command newline (0x1c bytes) through 01241650, calls the base dump 014d6a10, appends Text: through 0124165… |
| `014d2060` | GuildCommand_SetMOTD::AppendDisplayName | confirmed | Virtual slot 20 (+0x50). Appends the literal Guild SetMOTD Guild Command (0x1b bytes) to the std::string in ECX through 01241650 (stack argument, ret 4). |
| `014d1ef0` | GuildCommand_SetMOTD::Factory | likely | Allocates 0x2c bytes through 00d833a6; when non-null it constructs the object with 014d1d30 and returns it, otherwise returns 0. |
| `014d1f50` | GuildCommand_SetMOTD::RegisterFactory | likely | Registers the factory 014d1ef0 under classID 0x1d2 through 013ce130 and 013ce230. |
| `014d1dd0` | GuildCommand_SetMOTD::ReadGlobal01cbe808 | speculative | Virtual slot 12 (+0x30). No arguments; returns the 32-bit global DAT_01cbe808. |
| `014d1de0` | GuildCommand_SetMOTD::WriteGlobal01cbe808 | speculative | Virtual slot 13 (+0x34). One stack argument: stores it into the global DAT_01cbe808 (ret 4). |

**GuildCommand_SetMOTDResponse** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d1780` | GuildCommand_SetMOTDResponse::Ctor | confirmed | Fastcall constructor: calls base constructor 014d6640 with ECX=this, stores GuildCommand_SetMOTDResponse::vftable into *this, zeroes the dwords at this+8 and +… |
| `014d1800` | GuildCommand_SetMOTDResponse::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the C string literal GuildCommand_SetMOTDResponse. |
| `014d1840` | GuildCommand_SetMOTDResponse::DtorBody | confirmed | Fastcall destructor body: stores GuildCommand_SetMOTDResponse::vftable into *this, then calls the base destructor body 014d6730 on this. |
| `014d1950` | GuildCommand_SetMOTDResponse::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 0x10 bytes through 00d833a6; when non-null it constructs the object with 014d1780, then calls this->vtable[+0x48] (CopyTo) o… |
| `014d19c0` | GuildCommand_SetMOTDResponse::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. RTTI-casts the destination to SetMOTDResponse (00d8d382); on null calls the assert he… |
| `014d1a20` | GuildCommand_SetMOTDResponse::Read | confirmed | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4) on the object, the base read 014d6880, reads an integer into this+8 (013c24d0), a second inte… |
| `014d1a90` | GuildCommand_SetMOTDResponse::Write | confirmed | Virtual slot 10 (+0x28), write body. Calls the header write (slot 2), the base write 014d68d0, writes integers this+8 and this+0xc (013c23f0), then calls slot … |
| `014d1b50` | GuildCommand_SetMOTDResponse::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x00). Stores the vftable, calls the base destructor 014d6730, frees this when bit 0 of the flag argument is set, and returns this. |
| `014d1bb0` | GuildCommand_SetMOTDResponse::Dump | confirmed | Virtual slot 19 (+0x4c). Appends Guild Set MOTD Response Guild Command newline (0x26 bytes) through 01241650, then calls the base dump 014d6a10. |
| `014d1be0` | GuildCommand_SetMOTDResponse::AppendDisplayName | confirmed | Virtual slot 20 (+0x50). Appends the literal Guild Set MOTD Response Guild Command (0x25 bytes) to the std::string in ECX through 01241650 (stack argument, ret… |
| `014d18b0` | GuildCommand_SetMOTDResponse::PostClientEvent | likely | Virtual slot 15 (+0x3c). Allocates 0x14 bytes and constructs the block with 012f9eb0, sets its type word to 0xb8 (012f8c70), stores this+8 and this+0xc (012fa2… |
| `014d1ad0` | GuildCommand_SetMOTDResponse::Factory | likely | Allocates 0x10 bytes through 00d833a6; when non-null it constructs the object with 014d1780 and returns it, otherwise returns 0. |
| `014d1b30` | GuildCommand_SetMOTDResponse::RegisterFactory | likely | Registers the factory 014d1ad0 under classID 0x1d4 through 013ce130 and 013ce230. |
| `014d1810` | GuildCommand_SetMOTDResponse::ReadGlobal01cbe804 | speculative | Virtual slot 12 (+0x30). No arguments; returns the 32-bit global DAT_01cbe804. |
| `014d1820` | GuildCommand_SetMOTDResponse::WriteGlobal01cbe804 | speculative | Virtual slot 13 (+0x34). One stack argument: stores it into the global DAT_01cbe804 (ret 4). |

**GuildCommand_ShowMOTD** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d12a0` | GuildCommand_ShowMOTD::Read | confirmed | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4) on the object, the base read 014d6880, then reads three integers into this+8, +0xc, +0x10 (01… |
| `014d1350` | GuildCommand_ShowMOTD::Write | confirmed | Virtual slot 10 (+0x28), write body. Calls the header write (slot 2), the base write 014d68d0, writes integers this+8, +0xc, +0x10 (013c23f0), two strings at t… |
| `014d13c0` | GuildCommand_ShowMOTD::Ctor | confirmed | Fastcall constructor: calls base constructor 014d6640 with ECX=this, stores GuildCommand_ShowMOTD::vftable, sets the two std::string members at this+0x14 and t… |
| `014d1470` | GuildCommand_ShowMOTD::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the C string literal GuildCommand_ShowMOTD. |
| `014d14b0` | GuildCommand_ShowMOTD::DtorBody | confirmed | Fastcall destructor body: stores GuildCommand_ShowMOTD::vftable into *this, resets the two std::string members at this+0x30 and this+0x14 (buffer freed when he… |
| `014d1550` | GuildCommand_ShowMOTD::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 0x50 bytes through 00d833a6; when non-null it constructs the object with 014d13c0 and then calls this->vtable[+0x48] (CopyTo… |
| `014d1640` | GuildCommand_ShowMOTD::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x00). Calls the destructor body 014d14b0 on this, frees this when bit 0 of the flag argument is set, returns this (ret 4). |
| `014d16a0` | GuildCommand_ShowMOTD::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. RTTI-casts the destination from NetworkCommand to ShowMOTD (00d8d382), calls the base… |
| `014d1720` | GuildCommand_ShowMOTD::Dump | confirmed | Virtual slot 19 (+0x4c). Appends the text Guild ShowMOTD Guild Command newline through 01241650, calls the base dump 014d6a10, appends Message: through 0124165… |
| `014d1760` | GuildCommand_ShowMOTD::AppendDisplayName | confirmed | Virtual slot 20 (+0x50). Appends the literal Guild ShowMOTD Guild Command (0x1c bytes) to the std::string passed as its stack argument (ECX) through 01241650. |
| `014d11e0` | GuildCommand_ShowMOTD::PostClientEvent | likely | Virtual slot 15 (+0x3c). Allocates 0x14 bytes, constructs the block with 012f9eb0, sets its type word to 0xb6 (012f8c70), stores the three ints this+8, +0xc, +… |
| `014d15c0` | GuildCommand_ShowMOTD::Factory | likely | Allocates 0x50 bytes through 00d833a6; when non-null it constructs the object with 014d13c0 and returns it, otherwise returns 0. |
| `014d1620` | GuildCommand_ShowMOTD::RegisterFactory | likely | Registers the factory 014d15c0 under classID 0x1d3 through 013ce130 and 013ce230. |
| `014d1480` | GuildCommand_ShowMOTD::ReadGlobal01cbe800 | speculative | Virtual slot 12 (+0x30). No arguments; returns the 32-bit global DAT_01cbe800 (mov eax,[01cbe800]; ret). |
| `014d1490` | GuildCommand_ShowMOTD::WriteGlobal01cbe800 | speculative | Virtual slot 13 (+0x34). One stack argument, ret 4: stores it into the global DAT_01cbe800. |

**GuildCommand_StartGuilds** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d0d20` | GuildCommand_StartGuilds::Ctor | confirmed | Fastcall constructor: calls the base constructor 014d6640 with ECX=this, stores GuildCommand_StartGuilds::vftable into *this and zeroes the dword at this+8 (th… |
| `014d0da0` | GuildCommand_StartGuilds::GetClassName | confirmed | Virtual slot 11 (+0x2c). No arguments; returns the C string literal GuildCommand_StartGuilds (6-byte body: mov eax,01903c64; ret). |
| `014d0de0` | GuildCommand_StartGuilds::DtorBody | confirmed | Fastcall destructor body: stores GuildCommand_StartGuilds::vftable into *this, then calls the base destructor body 014d6730 on this. No free. |
| `014d0ed0` | GuildCommand_StartGuilds::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 0xc bytes through 00d833a6; when non-null it constructs the object with 014d0d20 and then calls this->vtable[+0x48] (CopyTo)… |
| `014d0f40` | GuildCommand_StartGuilds::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. Calls the base copy 014d6850 on the destination, RTTI-casts it from NetworkCommand to… |
| `014d0f70` | GuildCommand_StartGuilds::Read | confirmed | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4, +0x10) on the object, the base read 014d6880, reads one integer into this+8 through 013c24d0,… |
| `014d0fc0` | GuildCommand_StartGuilds::Write | confirmed | Virtual slot 10 (+0x28), write body. Calls the header write (slot 2, +0x8) on the object, trace text through 013c26e0 (no-op), the base write 014d68d0, one int… |
| `014d1090` | GuildCommand_StartGuilds::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x00). Stores GuildCommand_StartGuilds::vftable into *this, calls the base destructor 014d6730, and calls _free(this) when bit 0 of the flag a… |
| `014d10f0` | GuildCommand_StartGuilds::Dump | confirmed | Virtual slot 19 (+0x4c). Appends the text Start Guilds Guild Command newline through 01241650, calls the base dump 014d6a10, then formats ' Guild Count: %d' wi… |
| `014d1180` | GuildCommand_StartGuilds::AppendDisplayName | confirmed | Virtual slot 20 (+0x50). One stack argument, ret 4, ECX = that std::string: appends the literal Start Guilds Guild Command (0x1a bytes) through 01241650. |
| `014d0e40` | GuildCommand_StartGuilds::PostClientEvent | likely | Virtual slot 15 (+0x3c). Calls 012d46a0 with ECX=this and uses its result as the receiver of 012d3550 (list push). Allocates 0x14 bytes, constructs the block w… |
| `014d1010` | GuildCommand_StartGuilds::Factory | likely | Allocates 0xc bytes through 00d833a6; when non-null it constructs the object with 014d0d20 and returns it, otherwise returns 0. |
| `014d1070` | GuildCommand_StartGuilds::RegisterFactory | likely | Registers the factory 014d1010 under classID 0x1a4 through 013ce130 and 013ce230. |
| `014d0db0` | GuildCommand_StartGuilds::ReadGlobal01cbe7fc | speculative | Virtual slot 12 (+0x30). No arguments; returns the 32-bit global DAT_01cbe7fc (mov eax,[01cbe7fc]; ret). |
| `014d0dc0` | GuildCommand_StartGuilds::WriteGlobal01cbe7fc | speculative | Virtual slot 13 (+0x34). One stack argument, ret 4: stores the argument into the global DAT_01cbe7fc. |

**GuildCommand_TransferGuild** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d0850` | GuildCommand_TransferGuild::Ctor | confirmed | Fastcall constructor: calls the base constructor 014d6640, stores GuildCommand_TransferGuild::vftable into *this, zeroes the dwords at this+0xc, +0x10, +0x14 a… |
| `014d08c0` | GuildCommand_TransferGuild::Clone | confirmed | Virtual slot 17 (+0x44). Allocates 0x1c bytes through 00d833a6; if the allocation succeeded it constructs the object with 014d0850 and then calls this->vtable[… |
| `014d0b30` | GuildCommand_TransferGuild::CopyTo | confirmed | Virtual slot 18 (+0x48), this = source, one stack argument = destination. Calls base copy 014d6850 on the destination, RTTI-casts the destination from NetworkC… |
| `014d0ca0` | GuildCommand_TransferGuild::Read | confirmed | Virtual slot 9 (+0x24), read body. Calls the header read (slot 4, +0x10) on the object, the base read 014d6880, then the count-prefixed value list into the vec… |
| `014d0a20` | GuildCommand_TransferGuild::Factory | likely | Allocates 0x1c bytes with 00d833a6; when the allocation is non-null it constructs the object with 014d0850 and returns it, otherwise returns 0. |
| `014d0a80` | GuildCommand_TransferGuild::RegisterFactory | likely | Registers the TransferGuild factory 014d0a20 under classID 0x1a5 through 013ce130 and 013ce230 (both take the id and the factory pointer). |

**GuildPaidRenameCheckRequestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a16c50` | GuildPaidRenameCheckRequestPacket_Ctor 🌐 | confirmed | Constructor: calls the guild packet base constructor FUN_00a09b70 with sub-opcode 0x10, stores GuildPaidRenameCheckRequestPacket::vftable, copies the two argum… |
| `00a16ce0` | GuildPaidRenameCheckRequestPacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): delegates to the destructor body 00a151d0, which releases the IString member at +0x18, and frees the packet with _f… |

**(no class)** (51)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b1a510` | ResolveGuildRoleName | confirmed | Resolves a guild role display name: looks the role up with 0106dd60 and assigns the empty default DAT_0181d9ec when it is missing; a custom name already on the… |
| `00b1e600` | SendGuildMemberLocationTunnel 🌐 | confirmed | Initializes a byte buffer, serializes the record header and counted pair array, constructs PacketTunneledClientWorldPacket, copies the serialized payload lengt… |
| `00b1e730` | ReadGuildDataFullPayload 🌐 | confirmed | Reads the two-short guild header, copies an eight-byte identifier pair into the destination (or zeros it and marks reader error if truncated), advances the bou… |
| `00b1ea80` | BuildGuildMemberLocationRequest 🌐 | confirmed | Looks up the guild pair and, if found, constructs a GuildMemberLocationRequest, walks its member entries, excludes the pair matching client fields +0x3a0/+0x3a… |
| `00c0e490` | RegisterGuildLuaModule | confirmed | Registers the Lua module "Guild": calls FUN_00e39e40(param_1, "Guild", &table_at_0182b090, 0) where the table maps the names Create, Delete, Rename, SetMotd, I… |
| `00b175f0` | DeletingDtor_00b175f0 | likely | Deleting destructor for a guild record object: calls 0106e4e0, which drains the record chain at +0x98 through 00b19170 and releases the embedded role map and n… |
| `00b17d80` | InitEntryAndSetField20 | likely | Initializes a five-dword record through 0106d8e0 (which writes 0,0,0,0xffffffff,0) and then stores the first dword of param_2 into the record field at +0x20, r… |
| `00b18380` | LookupGuidPairByIndex | likely | Fills the two-dword output in param_2 with the guid pair at entry+0x20/entry+0x24 of the chain rooted at this+0x94, advancing param_3 links; when the chain end… |
| `00b183d0` | ChainAt94_GetIndex | likely | Returns the chain entry at index param_2 by walking the links at entry+0xc from the head at this+0x94 and stopping when the chain ends or the index runs out; t… |
| `00b18400` | ContainsPairIn258 | likely | Predicate: builds an 8-byte pair from the two arguments on the stack, forwards it to 00cb57c0 on the object at this+0x258 and returns whether the result is non… |
| `00b188f0` | DeletingDtor_00b188f0 | likely | Deleting destructor for a guild name record object: runs the cleanup body 0106dfb0, which releases the embedded name record through 0070a790, and then frees th… |
| `00b18df0` | FindByGuidPair | likely | Looks up an entry by the two-dword key (param_2, param_3): with param_4 non-null it walks the chain at this+0x94 comparing the pair at entry+0x20/entry+0x24 an… |
| `00b18ed0` | FindEntry32Buckets | likely | Looks up an entry in the 32-bucket table at this+0xc keyed by the xor of the two arguments: bucket = (param_3 xor param_2) & 0x1f, the chain runs through entry… |
| `00b1b060` | CopyPlayerIdentity | likely | Copies a player identity: three name-record sub-fields of the source at +0, +0x24 and +0x48 are copied through 006f9e60 and the guid trio at source+0x7c, +0x80… |
| `00b1d1b0` | ReadTwoU16ThenBoolAt94 | likely | Calls the bounded two-short reader, skips one bounded length-prefixed field, then reads one byte as a boolean into destination offset +0x94; on insufficient re… |
| `00b1d280` | WriteCountedPairArray | likely | Writes the array count from param_2+8 through 00911420, then iterates from zero to count-1 and serializes each 8-byte element at param_2+4 plus index times eig… |
| `00b1d370` | ReadBoundedGuildPayload 🌐 | likely | Rejects null destination or input, initializes a bounded reader over the supplied buffer and length, calls 00b1c770 to parse the body, and returns one only whe… |
| `00b1d3e0` | ReadTwoU16AndSkipField | likely | For non-null input and output, initializes a bounded reader, reads two 16-bit values through 00b1a810, skips one length-prefixed field through 008e7290, and re… |
| `00b1e160` | ReadGuildFullDataFields 🌐 | likely | Reads a two-dword identifier into destination +0x20/+0x24, consumes two bounded length-prefixed fields, reads booleans into +0x68/+0x69, delegates counted list… |
| `00c0f710` | SendGuildSetMotd | likely | Serialises the guild message-of-the-day packet (param_1) into a 0x2018-byte stack buffer: initialises the buffer (FUN_008d4a40(0x10000000), FUN_00a0de80), appe… |
| `00c0f830` | SendGuildInvite 🌐 | likely | Serialises a GuildInvitePacket (param_1) into a 0x2018-byte stack buffer: initialises the buffer (FUN_008d4a40(0x10000000), FUN_00a0de80), appends the 8-byte a… |
| `00c0f960` | SendGuildKick 🌐 | likely | Serialises a GuildRemovePacket (param_1) into a 0x2018-byte stack buffer: initialises the buffer (FUN_008d4a40(0x10000000), FUN_00a0de80), appends the 8-byte a… |
| `00c0fa80` | SendGuildQuit 🌐 | likely | Serialises a GuildQuitPacket (param_1) into a 0x2018-byte stack buffer: initialises the buffer (FUN_008d4a40(0x10000000), FUN_00a0de80), appends the u64 field … |
| `00c0fba0` | SendGuildPromote 🌐 | likely | Serialises a GuildPromotePacket (param_1) into a 0x2018-byte stack buffer: initialises the buffer (FUN_008d4a40(0x10000000), FUN_00a0de80), appends the 8-byte … |
| `00c0fcc0` | SendGuildDemote 🌐 | likely | Serialises a GuildDemotePacket (param_1) into a 0x2018-byte stack buffer: initialises the buffer (FUN_008d4a40(0x10000000), FUN_00a0de80), appends the 8-byte a… |
| `014d09a0` | ReadCountPrefixedValues4 | likely | Reads an element count through 013c2690 on the object whose pointer is the first dword of ECX, reserves the destination through 014d0410, then calls 014000c0 o… |
| `00c10640` | GuildQuit 🌐 | speculative | Lua C function bound as Guild.Quit: requires exactly 1 truthy argument, constructs GuildQuitPacket (FUN_00c0e820), stores the arg1 int64 (FUN_00bfea90) as the … |
| `014d0aa0` | AppendNewElement54 | speculative | Thiscall with one stack argument. Allocates 0x54 bytes through 00d833a6 and constructs the element with 014f8af0 (or 0 on failure), copies the stack argument i… |
| `014d2e10` | SetOwnedValueAt14Int | speculative | Thiscall with one stack argument. When the pointer at this+0x14 is null it allocates 0xc bytes through 00d833a6 and constructs the block with 012fa910; then ca… |
| `014d2e90` | SetOwnedValueAt18Int | speculative | Thiscall with one stack argument. Same as 014d2e10 but for the pointer at this+0x18: allocates the 0xc block when null (00d833a6, 012fa910), calls 01300680 wit… |
| `014d3390` | SetOwnedString14 | speculative | Thiscall with a C string argument. Creates the 0xc block at this+0x14 when null (00d833a6, 012fa910), copies the C string into a local std::string through 0042… |
| `014d3480` | SetOwnedString18 | speculative | Thiscall with a C string argument. Same as 014d3390 but for the pointer at this+0x18: creates the 0xc block when null, copies the C string through a local std:… |
| `014d4310` | CopyStringFrom0c | speculative | Thiscall (ECX = object) with one stack argument, the destination std::string. Sets the destination to empty (capacity 0xf, size 0, first byte 0) and assigns th… |
| `014d4b40` | VectorPushBackThenCall | speculative | Two stack arguments. Calls 0042c155 with the address of its first argument (push_back shape: 4-byte elements, begin/end at receiver +4/+8) and then 01233d70 wi… |
| `00b17ac0` | — | unknown | Cleanup wrapper with an exception frame whose whole body is a call to 0106dfb0, itself a wrapper around 0070a790 that releases a name-record structure; it has … |
| `00b17d20` | — | unknown | Constructs the record that lives at param_1 by running 0106e000 (which initializes the embedded strings and the fields up to +0xcc) and then stores the first d… |
| `00b18e90` | — | unknown | Finds the entry for the given key with 00b18df0 and, when the lookup returns non-null, passes it to 00cb5520 (the release/detach routine); it has no recorded c… |
| `00b19350` | — 🌐 | unknown | Reads one 0x18-byte guild entry from a bounded reader: takes an 8-byte guid pair from the cursor at reader+8, setting the overflow flag at reader+0x10 and clam… |
| `00b1b340` | — | unknown | Applies a freshly parsed member record (param_2) to the cached roster row (param_3): for each field pair it compares them and, when they differ, copies the new… |
| `00c0f3b0` | — 🌐 | unknown | Serialises a GuildCreatePacket (param_1) into a 0x2018-byte stack buffer: initialises the buffer with FUN_008d4a40(0x10000000) and FUN_00a0de80, appends the th… |
| `00c0f4e0` | — | unknown | Serialises a GuildDeletePacket (param_1) into a 0x2018-byte stack buffer: initialises the buffer (FUN_008d4a40(0x10000000), FUN_00a0de80), appends the u64 fiel… |
| `00c0f600` | — 🌐 | unknown | Serialises a GuildRenameGuildPacket into a 0x2018-byte stack buffer by delegating the body to FUN_00c0eef0 (u64 at +0x10, two strings at +0x18/+0x28, flag byte… |
| `00c0ff00` | — 🌐 | unknown | Lua C function bound as Guild.Create: constructs a GuildCreatePacket (FUN_00c0ea50), takes the guild name from arg1 (arg1 truthy test FUN_00e3a630 with index -… |
| `00c10080` | — | unknown | Lua C function bound as Guild.Delete: requires arg1 truthy (FUN_00e3a630 with -1), constructs a GuildDeletePacket (FUN_00c0e4b0), stores the arg1 int64 (FUN_00… |
| `00c10130` | — 🌐 | unknown | Lua C function bound as Guild.Rename: requires exactly 2 arguments (FUN_00e3a390 == 2) that are both truthy; constructs a GuildRenameGuildPacket (FUN_00c0ece0)… |
| `00c10250` | — | unknown | Lua C function bound as Guild.SetMotd: requires exactly 2 truthy arguments, constructs the motd packet with FUN_00b19560, stores the arg1 int64 (FUN_00bfea90) … |
| `00c10330` | — 🌐 | unknown | Lua C function bound as Guild.Invite: requires exactly 3 truthy arguments, constructs a GuildInvitePacket (FUN_00c0ebd0), stores the arg1 int64 (FUN_00bfea90) … |
| `00c10430` | — 🌐 | unknown | Lua C function bound as Guild.InviteReply: requires exactly 2 arguments; arg1 must be truthy and arg2 must be a boolean (FUN_00e3a580 == 1, value read with FUN… |
| `00c10570` | — 🌐 | unknown | Lua C function bound as Guild.Kick: requires exactly 2 truthy arguments, constructs GuildRemovePacket (FUN_00c0e740), stores arg1 (FUN_00bfea90) and arg2 (FUN_… |
| `00c106f0` | — 🌐 | unknown | Lua C function bound as Guild.Promote: requires exactly 2 truthy arguments, constructs GuildPromotePacket (FUN_00c0e580), stores the two int64 arguments (FUN_0… |
| `00c107c0` | — 🌐 | unknown | Lua C function bound as Guild.Demote: requires exactly 2 truthy arguments, constructs GuildDemotePacket (FUN_00c0e660), stores the two int64 arguments (FUN_00b… |

### `net.poll`

**PollChoice** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01442ab0` | PollChoice::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls slot 4 on this with (param_2, &param_2) and returns 0 when that fails; reads an int into +0x4 and an int i… |
| `01442b30` | PollChoice::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls slot 2 on this with param_2, calls 013c26e0 with Starting PollChoice (a no-op that returns 1, so the text… |
| `01442c40` | PollChoice::TypeName | confirmed | Virtual slot 11 with no arguments. Returns the address of the literal PollChoice (018ea920). |
| `01442c50` | PollChoice::GetStaticDword | speculative | Virtual slot 12 with no arguments. Returns the dword stored in the global DAT_01cbe3f4. |
| `01442c60` | PollChoice::SetStaticDword | speculative | Virtual slot 13 with one argument. Stores the argument into the global dword DAT_01cbe3f4. |

**PollCommand** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01442df0` | PollCommand::PollCommand | confirmed | Fastcall constructor (this in ECX, returns this). Calls the base constructor 0140c090 on this, then stores the PollCommand vtable at +0. |
| `01442e70` | PollCommand::TypeName | confirmed | Virtual slot 11 with no arguments. Returns the address of the literal PollCommand (018ea98c). |
| `01442eb0` | PollCommand::~PollCommand | confirmed | Destructor body (fastcall, this in ECX, no deleting part). Stores PollCommand::vftable at +0, calls the base destructor 0140c160 on this and returns. |
| `01442f10` | PollCommand::Clone | confirmed | Virtual slot 17 (fastcall, this in ECX). Allocates 4 bytes with operator new (00d833a6); when non-null default-constructs the block with 01442df0; then calls s… |
| `01442f80` | PollCommand::CopyFieldsTo | confirmed | Virtual slot 18 (one stack argument, the destination NetworkCommand, ret 4). Casts the destination with the RTTI dynamic_cast 00d8d382 from NetworkCommand to P… |
| `01442fe0` | PollCommand::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls slot 4 on this with (param_2, &param_2) and returns 0 when that fails; calls base-class read 0140c2a0 with… |
| `01443020` | PollCommand::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls slot 2 on this with param_2, calls 013c26e0 with Starting PollCommand (a no-op that returns 1, so the tex… |
| `014430f0` | PollCommand::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (thiscall, one stack argument, ret 4). Stores PollCommand::vftable at +0, calls the base destructor 0140c160 on this, then calls _fr… |
| `01443150` | PollCommand::AppendDisplayName | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument (the std::string receiving text) and appends the 12-byte literal Poll Command (018ea9f… |
| `01443070` | PollCommand::Create | likely | Factory with no arguments. Calls operator new with 4 bytes (00d833a6); returns 0 when that returns null, otherwise default-constructs the block with 01442df0 a… |
| `01442e80` | PollCommand::GetStaticDword | speculative | Virtual slot 12 with no arguments. Returns the dword stored in the global DAT_01cbe3f8. |
| `01442e90` | PollCommand::SetStaticDword | speculative | Virtual slot 13 with one argument. Stores the argument into the global dword DAT_01cbe3f8. |

**PollCommand_AddPoll** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b2810` | PollCommand_AddPoll::ctor | confirmed | Constructor (this in ECX). Calls 01442df0 (the PollCommand constructor, which references PollCommand::vftable), stores PollCommand_AddPoll::vftable, zeroes the… |
| `014b2890` | PollCommand_AddPoll::TypeName | confirmed | Virtual slot 11 (+0x2c). Returns the constant C string PollCommand_AddPoll (mov eax, literal; ret). Reads no fields. |
| `014b29c0` | PollCommand_AddPoll::Clone | confirmed | Virtual slot 17 (+0x44). Calls 00d833a6 (operator new) with size 8. If the block is non-null it constructs an AddPoll command in it with 014b2810. Then, uncond… |
| `014b2a30` | PollCommand_AddPoll::CopyFieldsTo | confirmed | Virtual slot 18 (+0x48). Argument is the target NetworkCommand pointer; dynamic_cast to PollCommand_AddPoll (00d8d382). On failure reports via 012f5a60 (tag lo… |
| `014b2a80` | PollCommand_AddPoll::WriteBody | confirmed | Virtual slot 10 (+0x28). Calls slot 2 (+0x8) with the argument (header write), writes the text Starting PollCommand_AddPoll through 013c26e0 (no-op), calls 014… |
| `014b2c10` | PollCommand_AddPoll::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores PollCommand_AddPoll::vftable into *this, calls 01442eb0 (PollCommand-level teardown), then calls _free(this)… |
| `014b2c70` | PollCommand_AddPoll::ReadBody | confirmed | Virtual slot 9 (+0x24). Calls slot 4 (+0x10) on this with the argument and &param_2 (header read; tested as bool), then 01442fe0 with the argument (PollCommand… |
| `014b2cc0` | PollCommand_AddPoll::Dump | confirmed | Virtual slot 19 (+0x4c). Appends the header Poll Command Add Poll plus newline via 01241650, calls thunk 01442fd0 (stub) with the output string, then appends P… |
| `014b2d70` | PollCommand_AddPoll::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Pushes length 0x15 and the literal Poll Command Add Poll, calls 01241650 (append), returns. The argument is loaded into ECX before the… |
| `014b28a0` | PollCommand_AddPoll::ReadGlobal01cbe700 | likely | Virtual slot 12 (+0x30). Returns the 32-bit global at 01cbe700 (mov eax,[01cbe700]; ret). Reads no object fields. |
| `014b28b0` | PollCommand_AddPoll::WriteGlobal01cbe700 | likely | Virtual slot 13 (+0x34). One stack argument (ret 4): stores it into the global at 01cbe700. |
| `014b28d0` | PollCommand_AddPoll::DtorBody | likely | Stores PollCommand_AddPoll::vftable into *this, then calls 01442eb0 (PollCommand-level teardown) with the EH cookie. Not a vtable slot; the deleting destructor… |
| `014b2b90` | PollCommand_AddPoll::New | likely | No arguments. Calls 00d833a6 (operator new) with size 8. If the block is non-null it passes it to 014b2810 (the AddPoll constructor) and returns that object; o… |
| `014b2930` | PollCommand_AddPoll::QueueEvent | speculative | Virtual slot 15 (+0x3c). Reads the pointer at this+0x4; when it is non-null it calls 012cadf0 and 012caed0 with it, then allocates 0x14 bytes with 00d833a6, bu… |

**PollCommand_PollResponse** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b2350` | PollCommand_PollResponse::ctor | confirmed | Constructor (this in ECX). Calls 01442df0 (the PollCommand constructor, which references PollCommand::vftable), stores PollCommand_PollResponse::vftable, and s… |
| `014b23d0` | PollCommand_PollResponse::TypeName | confirmed | Virtual slot 11 (+0x2c). Returns the constant C string PollCommand_PollResponse (mov eax, literal; ret). Reads no fields. |
| `014b24b0` | PollCommand_PollResponse::Clone | confirmed | Virtual slot 17 (+0x44). Calls 00d833a6 (operator new) with size 0xc. If the block is non-null it constructs a PollResponse command in it with 014b2350. Then, … |
| `014b2520` | PollCommand_PollResponse::CopyFieldsTo | confirmed | Virtual slot 18 (+0x48). Argument is a NetworkCommand pointer; dynamic_cast to PollResponse (00d8d382). On failure reports via 012f5a60 (pollCommand, PollComma… |
| `014b2580` | PollCommand_PollResponse::ReadBody | confirmed | Virtual slot 9 (+0x24). Calls slot 4 (+0x10) on this with the argument and &param_2 (header read; tested as bool), then 01442fe0 with the argument (the PollCom… |
| `014b25e0` | PollCommand_PollResponse::WriteBody | confirmed | Virtual slot 10 (+0x28). Calls slot 2 (+0x8) with the argument (header write), writes the text Starting PollCommand_PollResponse through 013c26e0 (no-op), call… |
| `014b26c0` | PollCommand_PollResponse::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores PollCommand_PollResponse::vftable into *this, calls 01442eb0 (PollCommand-level teardown), then calls _free(… |
| `014b2720` | PollCommand_PollResponse::Dump | confirmed | Virtual slot 19 (+0x4c). Appends the header Poll Command Poll Response plus newline via 01241650, calls thunk 01442fd0 (stub) with the output string, then form… |
| `014b27f0` | PollCommand_PollResponse::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Loads the stack argument (std::string pointer) into ECX, pushes length 0x1a and the literal Poll Command Poll Response, calls 01241650… |
| `014b23e0` | PollCommand_PollResponse::ReadGlobal01cbe6fc | likely | Virtual slot 12 (+0x30). Returns the 32-bit global at 01cbe6fc (mov eax,[01cbe6fc]; ret). Reads no object fields. |
| `014b23f0` | PollCommand_PollResponse::WriteGlobal01cbe6fc | likely | Virtual slot 13 (+0x34). One stack argument (ret 4): stores it into the global at 01cbe6fc. |
| `014b2640` | PollCommand_PollResponse::New | likely | No arguments. Calls 00d833a6 (operator new) with size 0xc. If the block is non-null it passes it to 014b2350 (the PollResponse constructor) and returns that ob… |
| `014b2410` | — | unknown | Stores PollCommand_PollResponse::vftable into *this, then calls 01442eb0 with the EH cookie (the PollCommand-level teardown, not labelled in this shard). Not a… |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b26a0` | RegisterFactoryKey18a | likely | No arguments. Pushes the address 014b2640 (the PollResponse New factory) and key 0x18a, calls 013ce130, moves its EAX into ECX and calls 013ce230. Same shape a… |
| `014b2af0` | ReadOptionalSubObject | likely | Two stack arguments (the stream and a pointer to the slot at +0x4). Reads a flag integer with 013c24d0; when 0 returns 0. Releases the object previously held i… |
| `014b2bf0` | RegisterFactoryKey188 | likely | No arguments. Pushes the address 014b2b90 (the PollCommand_AddPoll New factory) and key 0x188, calls 013ce130, moves its EAX into ECX and calls 013ce230. Same … |
| `012ca880` | ClearOwnedListAndSendPollControlAdd 🌐 | speculative | Fastcall on an owned-pointer vector at this. Deletes each element through its virtual slot 0 and compacts the vector as 012ca710 does, then builds a PollComman… |

### `net.portrait`

**FotomatPortraitDataManager** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bd32b0` | FotomatDefaultPortraitCheck | confirmed | Takes one pointer argument. When the pointer is non-null and the dword at +8 of its sub-object is nonzero it calls 007CC400 and returns; otherwise it logs thro… |
| `00bd3300` | FotomatLoadDefaultPortrait | confirmed | Thiscall with the manager as ECX. If the dword at this+0x21C is zero, assigns the string icon_UI_empty_portrait_64.dds to a local IString (00402CE0), obtains t… |

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b4f7d0` | AppendU16ThenU32 | likely | Thiscall (ECX = source record; param_2 = writer object on the stack). Appends the low 16 bits of the dword at record+4, then the dword at record+8, to the writ… |
| `00b4fc30` | BuildAndSendTunneledRecord16_32 🌐 | likely | Stack arguments: two dwords; the second is passed to the send helper as its mode. Calls 008d4a40(0x10000000) to initialise a writer, appends the 16-bit and 32-… |
| `00bd2730` | FotomatPortraitCacheFindTouch | likely | Thiscall with two dword id halves (param_2, param_3). Key = param_3 XOR param_2; searches the 8-bucket chain at this+0x1D0 (bucket = key & 7; key at node+0x211… |
| `00bd27d0` | HandlerListBroadcastSlot0 | likely | Thiscall with three stack arguments. Walks the singly linked list at this+0x1F8 (node: handler pointer at +0, next at +4) and calls slot 0 of each handler obje… |
| `00bd2820` | HandlerListBroadcastSlot1 | likely | Thiscall with three stack arguments. Walks the list at this+0x1F8 (handler at node+0, next at node+4) and calls each handler's vtable slot 1 (+4) with the thre… |
| `00bd2870` | HandlerListBroadcastSlot2 | likely | Thiscall with three stack arguments. Walks the list at this+0x1F8 (handler at node+0, next at node+4) and calls each handler's vtable slot 2 (+8) with the thre… |
| `00bd2b20` | FotomatCurrentIdTouch | speculative | Gets the global object through 00769710, reads the two-dword id at its sub-object +0x3A0/+0x3A4 and calls 00bd2730 (FotomatPortraitCacheFindTouch) with it, dis… |

### `net.profile`

**PlayClient** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0092ef50` | PlayClient::SetProfileByItemDefinitionId | confirmed | Constructs CommandPacketSetProfileByItemDefinitionId, stores the supplied parameter in its payload, and invokes sender 00925960 only when client field +0x280 i… |
| `0092f260` | PlayClient::SendActivateProfileFailed | confirmed | Constructs CommandPacketActivateProfileFailed, copies two words from offsets +0x504/+0x508 of the object at client offset +0x3e0, then calls sender 00925b80. |

### `net.rating`

**BaseRatingPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c86790` | BaseRatingPacketWithListDtor | confirmed | Destroys the embedded RatingDataEntry list, releases the owned string buffer, and restores BaseRatingPacket then BasePacket vtables. |
| `00c869e0` | — 🌐 | unknown | Reads a BaseRatingPacket subopcode and dispatches rating data, candidate-info, and other rating replies; housing-related cases update local lookup or housing p… |

**RatingPacketAddCandidate** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab09a0` | RatingPacketAddCandidateScalarDeletingDtor | confirmed | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00ac1540` | SendRatingPacketAddCandidate | likely | Transport wrapper for the rating add-candidate packet: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abcfa0 over the caller… |

**RatingPacketAddVote** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab09e0` | RatingPacketAddVoteScalarDeletingDtor | confirmed | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00ac1890` | SendRatingPacketAddVote | likely | Transport wrapper for the rating add-vote packet: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abd070 over the caller's pa… |

**RatingPacketCandidateInfoReply** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c86830` | RatingPacketCandidateInfoReplyScalarDeletingDtor | confirmed | Runs the candidate-info reply destructor, frees the object when flag bit 0 is set, and otherwise returns it. |

**RatingPacketCandidateInfoRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab0a20` | RatingPacketCandidateInfoRequestScalarDeletingDtor | confirmed | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00abfe70` | — | unknown | Transport wrapper for the rating candidate-info request: opens the shared transport scratch scope with FUN_008d4a40(0x10000000), runs the rating family header … |

**RatingPacketClientDataRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abd470` | RatingPacketClientDataRequest::RatingPacketClientDataRequest 🌐 | confirmed | Calls FUN_00aa5ca0 with inner opcode 3, installs the request and IString vtables, initializes the string at +0xc, zeroes +0x10,+0x14,+0x18, then calls FUN_00ab… |
| `00abd4f0` | RatingPacketClientDataRequest::ScalarDeletingDtor | confirmed | Calls FUN_00abab40, frees this only when flag bit 0 is set, returns this in EAX and returns with ret 4. |

**RatingPacketDataReply** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c86670` | RatingPacketDataReplyCtor | confirmed | Constructs BaseRatingPacket with subtype 5, installs the data-reply vtable, initializes an owned string, and initializes rating data storage. |
| `00c866f0` | RatingPacketDataReplyScalarDeletingDtor | confirmed | Runs the packet destructor, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |

**RatingPacketRemoveCandidate** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab09c0` | RatingPacketRemoveCandidateScalarDeletingDtor | confirmed | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00ac1650` | SendRatingPacketRemoveCandidate | likely | Transport wrapper for the rating remove-candidate packet: opens the scratch scope with FUN_008d4a40(0x10000000), emits the rating family header with FUN_00aba8… |

**RatingPacketRequestFeatured** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab0980` | RatingPacketRequestFeaturedScalarDeletingDtor | confirmed | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00ac1770` | SendRatingPacketRequestFeatured | likely | Transport wrapper for the rating featured-request packet: opens the scratch scope with FUN_008d4a40(0x10000000), emits the rating family header with FUN_00aba8… |

**RatingPacketSearchReply** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c86710` | RatingPacketSearchReplyCtor | confirmed | Constructs BaseRatingPacket with subtype 0x0d, installs the search-reply vtable, and initializes its owned string fields. |
| `00c868e0` | RatingPacketSearchReplyScalarDeletingDtor | confirmed | Runs the search-reply destructor, frees the object when flag bit 0 is set, and otherwise returns it. |

**RatingPacketSearchRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab0a00` | RatingPacketSearchRequestScalarDeletingDtor | confirmed | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00ac1ae0` | SendRatingPacketSearchRequest | likely | Transport wrapper for the rate-limited rating search request: opens the scratch scope with FUN_008d4a40(0x10000000), emits the rating family header with FUN_00… |

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00abcfa0` | RatingPacketAddCandidate_SerializeBody | likely | Calls FUN_00aba870 for the rating header, serializes seven length-prefixed strings at source+0xc,+0x1c,+0x2c,+0x3c,+0x4c,+0x5c,+0x6c and eight bytes at source+… |
| `00abd070` | RatingPacketAddVote_SerializeBody | likely | Calls FUN_00aba870 for the rating header, serializes four length-prefixed strings at source+0xc,+0x1c,+0x2c,+0x3c, then writes the dword at source+0x4c through… |
| `00abd280` | SerializeLinkedRatingEntries | likely | Writes the dword at list+0x10 through FUN_00911420, then walks nodes from list+8 through each node+0x10; for each node writes node+0x18 and serializes the node… |
| `00abf6f0` | RatingPacketClientDataRequest_SerializeBody | likely | Body writer for the rating client-data request: emits the rating family header with FUN_00aba870 (base opcode u16 plus one subtype byte), writes the string mem… |
| `00c865a0` | ReadRatingDataReply | likely | Reads two leading values, parses nested counted collections, then reads one additional scalar into the output object. |
| `00c864f0` | — | unknown | Reads two leading values, parses a counted nested collection, bounds-checks its length, and appends its contents to output fields. |

### `net.sns`

**BaseSnsIntegrationPacket** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb5710` | BaseSnsIntegrationPacket::~BaseSnsIntegrationPacket | confirmed | Destructor body: installs BaseSnsIntegrationPacket::vftable and then BasePacket::vftable into this+0 and returns; it is called from fifteen MSVC unwind funclet… |
| `00bb5760` | BaseSnsIntegrationPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at vtable slot 0: installs BaseSnsIntegrationPacket::vftable then BasePacket::vftable and, when bit 0 of the flag argument is set, r… |
| `00bb5830` | BaseSnsIntegrationPacket::~BaseSnsIntegrationPacket_2 | likely | Second base-destructor body of the SNS family (same shape as 00bb5710, 76 bytes): installs BaseSnsIntegrationPacket::vftable then BasePacket::vftable into this… |
| `00bb58e0` | BaseSnsIntegrationPacket::~BaseSnsIntegrationPacket_3 | likely | Third base-destructor body of the SNS family (same shape): installs BaseSnsIntegrationPacket::vftable then BasePacket::vftable into this+0; invoked from the un… |
| `00bb5640` | — 🌐 | unknown | Constructor body: installs BasePacket::vftable, stores the base opcode 0xc5 (197) at +4, zeroes the sub-opcode at +8 and the payload slot at +0xc, then install… |
| `00bb56a0` | — 🌐 | unknown | Initialisation used by every SNS sub-packet constructor: installs BasePacket::vftable, stores the base opcode 0xc5 at +4, installs BaseSnsIntegrationPacket::vf… |

**PlayerTitleSNSUpdatePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb6ba0` | PlayerTitleSNSUpdatePacket::PlayerTitleSNSUpdatePacket 🌐 | confirmed | Constructor: runs the BasePlayerTitlePacket family constructor 00b491a0 with sub-opcode 7 (which stores base opcode 0x98 = 152), installs PlayerTitleSNSUpdateP… |
| `00bb6c10` | PlayerTitleSNSUpdatePacket::~PlayerTitleSNSUpdatePacket | confirmed | Destructor: installs PlayerTitleSNSUpdatePacket::vftable, rewrites the SoeUtil::List<int,-1> vtable at +0x18 and clears that list with 007798c0, then installs … |
| `00bb6c80` | PlayerTitleSNSUpdatePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at vtable slot 0: runs the destructor body 00bb6c10 and, when bit 0 of the flag argument is set, releases this through the CRT free … |

**SnsIntegrationCommon** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01080ce0` | SnsBuildMultipartPost | confirmed | Virtual slot 6 implementation shared with SnsIntegrationProcessor. When the flag at this+0x10 is set it builds a multipart/form-data body (fields pact=xml, req… |
| `01081c60` | SnsIntegrationCommonDtorBody | confirmed | Destructor body of SnsIntegrationCommon: stores the SnsIntegrationCommon vftable, drains the two MessageTrackingNode lists at this+0x24 and this+0x138 through … |
| `01081f70` | SnsIntegrationCommonScalarDeletingDtor | likely | Virtual slot 0 scalar deleting destructor: calls the destructor body 01081c60 on this, then frees this with _free when flag bit 0 of the argument is set, and r… |
| `0107fc80` | ReleaseMember4IfNonNull | speculative | Virtual slot 5 (shared by SnsIntegrationProcessor and SnsIntegrationCommon): if the pointer at this+0x4 is non-null it tail-jumps to 00d31890 with that pointer… |
| `01081e10` | — | unknown | Constructor body: stores TinyHttp::HttpManagerHandler then SnsIntegrationCommon vftables at this+0x0, installs the IString<char> vftable at +0x8 with the stati… |

**SnsIntegrationFirstTimeLogin** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb5be0` | SnsIntegrationFirstTimeLogin::ScalarDeletingDtor | confirmed | Scalar deleting destructor at vtable slot 0: installs BaseSnsIntegrationPacket::vftable then BasePacket::vftable and, when bit 0 of the flag argument is set, r… |
| `00bb57d0` | — 🌐 | unknown | Constructor of the SNS first-time-login packet: runs the family initialiser 00bb56a0 with sub-opcode 2 and installs SnsIntegrationFirstTimeLogin::vftable. |

**SnsIntegrationGuildCreated** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb6320` | SnsIntegrationGuildCreated::~SnsIntegrationGuildCreated | confirmed | Destructor: rewrites the IString vtable at +0x10 and, when its capacity word is positive, decrements the shared buffer refcount and releases it through IString… |
| `00bb64b0` | SnsIntegrationGuildCreated::ScalarDeletingDtor | confirmed | Scalar deleting destructor at vtable slot 0: runs the destructor body 00bb6320 and, when bit 0 of the flag argument is set, releases this through the CRT free … |
| `00bb62b0` | — 🌐 | unknown | Constructor of the SNS guild-created packet: runs the family initialiser 00bb56a0 with sub-opcode 3, installs SnsIntegrationGuildCreated::vftable and initialis… |

**SnsIntegrationMaxLevelReached** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb5880` | SnsIntegrationMaxLevelReached::SnsIntegrationMaxLevelReached 🌐 | confirmed | Constructor calls the SNS packet base initializer with subtype 5 and installs SnsIntegrationMaxLevelReached::vftable. |
| `00bb5c50` | SnsIntegrationMaxLevelReached::ScalarDeletingDtor | confirmed | Scalar deleting destructor at vtable slot 0: installs BaseSnsIntegrationPacket::vftable then BasePacket::vftable and, when bit 0 of the flag argument is set, r… |

**SnsIntegrationProcessor** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb5cd0` | SnsIntegrationProcessor::SnsIntegrationProcessor | confirmed | Constructor: runs the SnsIntegrationCommon/TinyHttp base setup 01081e10, installs SnsIntegrationProcessor::vftable, stores the client argument at +0x244 and in… |
| `00bb5d50` | SnsIntegrationProcessor::ScalarDeletingDtor | confirmed | Scalar deleting destructor at vtable slot 0: installs SnsIntegrationProcessor::vftable, calls the base teardown 01081c60 and, when bit 0 of the flag argument i… |
| `00bb7790` | SnsIntegrationProcessor::HandleInspectReply | confirmed | Parses the result and gender, birthday, signature, interests, city, and about-me fields; marks missing values and copies the values to the inspect proxy before… |
| `00bb8220` | SnsIntegrationProcessor::OnUiCancelShare | confirmed | Searches the pending-message bucket for the requested tracking ID; logs and returns if absent, otherwise removes the matching entry. |
| `00bb84f0` | SnsIntegrationProcessor::SendSecureSignatureRequest | confirmed | Finds the pending tracking record, copies its message, and when the interface at owner offset 0x280 exists builds and sends a secure-signature request through … |
| `00bb85d0` | SnsIntegrationProcessor::HandleAboutMeReply 🌐 | confirmed | Parses gender, birthday, signature, interests, city, and about-me fields; marks missing values, updates character data, reads titles, builds PlayerTitleSNSUpda… |
| `00bb8b30` | SnsIntegrationProcessor::HandleFirstTimeLoginPacket | confirmed | Parses the packet fields, requests the SnsFirstTimeLogin translation, builds a postAppWeibo XML body, tracks the request, and asks the secure-signature path to… |
| `00bb8d90` | SnsIntegrationProcessor::HandleJobMaxLevelReachedPacket | confirmed | Parses packet fields, requests SnsJobMaxLevelReached translation, builds a postAppWeibo XML body, tracks the request, and sends it through the secure-signature… |
| `00bb92c0` | SnsIntegrationProcessor::StartInspectRequest | confirmed | Obtains account and player identifiers, formats getUserInfoForFr XML, allocates a pending record with type 8, logs the request, and sends it through the secure… |
| `00bb94c0` | SnsIntegrationProcessor::StartAboutMeRequest | confirmed | Obtains the account identifier, formats getUserInfoForFr XML, allocates a pending record with type 10, logs the request, and sends it through the secure-signat… |
| `00bb9650` | SnsIntegrationProcessor::OnPostResult | confirmed | Logs non-200 and empty responses; for HTTP 200 parses the body, finds a tracking ID, routes type 8 to HandleInspectReply and type 10 to HandleAboutMeReply, the… |
| `00bb5db0` | SnsIntegrationProcessor::ThrottleRemaining | likely | Throttle check for outgoing SNS posts: reads the current time with 00702670, and when the stored last-send pair at +0x248/+0x24c is not the null pair (globals … |
| `00bb9930` | SnsIntegrationProcessor::DispatchBaseSnsIntegrationPacket 🌐 | likely | Initializes the base SNS packet, parses its leading fields, then dispatches subtype 1 to the secure-signature response handler and subtype 2 to the first-time-… |
| `00bb5970` | — | unknown | Base constructor fragment of SnsIntegrationProcessor: installs SnsIntegrationProcessor::vftable and calls the SnsIntegrationCommon/TinyHttp base constructor 01… |
| `00bb6ca0` | — | unknown | Posts one SNS message: gates on the throttle 00bb5db0 (logging 'SnsIntegrationCommon::SnsSendMessage - failed. Attempted to send message before throttle time e… |
| `00bb8100` | — | unknown | Parses a secure-signature response body, finds the pending tracking ID, stores the returned pair, posts the prepared message, updates the tracking record, and … |

**SnsIntegrationSecureSignatureRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb61b0` | SnsIntegrationSecureSignatureRequest::SnsIntegrationSecureS… 🌐 | confirmed | Constructor of the SNS secure-signature-request packet: runs the family initialiser 00bb56a0 with sub-opcode 6, installs SnsIntegrationSecureSignatureRequest::… |
| `00bb6220` | SnsIntegrationSecureSignatureRequest::~SnsIntegrationSecure… | confirmed | Destructor: rewrites the IString vtable at +0x10 and, when its capacity word is positive, decrements the shared buffer refcount and releases it through IString… |
| `00bb6490` | SnsIntegrationSecureSignatureRequest::ScalarDeletingDtor | confirmed | Scalar deleting destructor at vtable slot 0: runs the destructor body 00bb6220 and, when bit 0 of the flag argument is set, releases this through the CRT free … |

**SnsIntegrationSecureSignatureResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb6120` | SnsIntegrationSecureSignatureResponse::~SnsIntegrationSecur… | confirmed | Destructor: rewrites the IString vtable at +0x10 and, when its capacity word is positive, decrements the shared buffer refcount and releases the buffer through… |
| `00bb6470` | SnsIntegrationSecureSignatureResponse::ScalarDeletingDtor | confirmed | Scalar deleting destructor at vtable slot 0: runs the destructor body 00bb6120 and, when bit 0 of the flag argument is set, releases this through the CRT free … |
| `00bb60b0` | — 🌐 | unknown | Constructor of the SNS secure-signature-response packet: runs the family initialiser 00bb56a0 with sub-opcode 1, installs SnsIntegrationSecureSignatureResponse… |

**SoeUtil::HashList<SnsIntegrationCommon::MessageTrackingNode,64,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01081d70` | HashListSnsCtor | likely | Constructor body: stores the HashListMap<int,SnsIntegrationCommon::MessageTrackingNode,64,-1> vftable, zeroes this+0x10, +0xc, +0x8 and +0x4, clears the 64-buc… |
| `01081df0` | HashListSnsScalarDeletingDtor | likely | Virtual slot 0 scalar deleting destructor: calls the destructor body 01081bf0 on this, then frees this with _free when flag bit 0 of the argument is set, and r… |

**SoeUtil::HashListMap<int,SnsIntegrationCommon::MessageTrackingNode,64,-1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0107fc90` | HashListMap64Ctor | likely | Constructor body: stores the HashListMap<int,SnsIntegrationCommon::MessageTrackingNode,64,-1> vftable, zeroes this+0x10, +0x8, +0xc and +0x4, and clears the 64… |
| `01081a40` | HashListMapSnsDtorBody | likely | Destructor body: stores the HashListMap<int,SnsIntegrationCommon::MessageTrackingNode,64,-1> vftable, removes nodes from the list head through 00bb8000 until t… |
| `01081bb0` | HashListMapSnsScalarDeletingDtor | likely | Virtual slot 0 scalar deleting destructor: stores the vftable, removes list nodes through 00bb8000 until the head is null, frees this with _free when flag bit … |
| `01081bf0` | HashListMapSnsDtorBody | likely | Destructor body (EH variant): stores the HashListMap<int,SnsIntegrationCommon::MessageTrackingNode,64,-1> vftable and removes list nodes through 00bb8000 until… |

**YouTube::FRYouTubeUploaderHandler** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d2e0e0` | OnVideoUploadAuthenticate | confirmed | Virtual slot 0 of FRYouTubeUploaderHandler: when the function pointer at this+4 is non-null, calls it with the string 'VideoHandler:OnVideoUploadAuthenticate' … |
| `00d2e100` | OnVideoUploadPost | confirmed | Virtual slot 1 of FRYouTubeUploaderHandler: when the callback at this+4 is non-null, calls it with the string 'VideoHandler:OnVideoUploadPost' and the third st… |
| `00d2e130` | OnVideoUploadProgress | confirmed | Virtual slot 3 of FRYouTubeUploaderHandler: when the callback at this+4 is non-null, calls it with the string 'VideoHandler:OnVideoUploadProgress' and the seco… |
| `00d2e170` | Ctor | confirmed | Stores the FRYouTubeUploaderHandler vptr at +0 and the stack argument at +4, then returns this in EAX (ret 4). |
| `00d2e190` | DeleteIfFlagBit0 | speculative | Virtual slot 4 of FRYouTubeUploaderHandler: stores the vptr at +0, frees this through _free when bit 0 of the byte argument is set, and returns this (ret 4). I… |

**(no class)** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb74d0` | Dtor_TwoStringsAndList | confirmed | Destructor of the SNS message object the reply handlers build: rewrites the IString vtable at +0x28 and releases that buffer through IString vtable slot 2, rew… |
| `00bb7670` | Ctor_TwoStringsAndList | confirmed | Constructor of the SNS message object: initialises the IString at +8 (IString vtable + static empty string, zeroed length/capacity), the SoeUtil::List<SoeUtil:… |
| `00bb63b0` | ReadChildElementList | likely | Iterates the children of a named node into a list: selects the child named param_2 with 01017500, and when found clears the target list (007798c0) and then rep… |
| `00bb6560` | ReadU16U16U32Into_4_8_c | likely | Reads three fields from the byte reader at param_2 into the destination at this+4/+8/+0xc: two little-endian 16-bit values and one 32-bit value; every read com… |
| `00bb65f0` | ReadU16U16U32StringAndPair | likely | Decodes a packet body into the destination: the three fields via 00bb6560, then a length-prefixed byte string whose length is read from the cursor and whose by… |
| `00bb6690` | ReadU16U16U32AndString | likely | Decodes a packet body into the destination: the three fields via 00bb6560 and then a length-prefixed byte string assigned into the destination's IString with 0… |
| `00bb7080` | ReadU16U16U32AndU32 | likely | Decodes a packet body: the three fields via 00bb6560 and then one more 32-bit value into the destination's +0x10, with the same end-of-buffer check (0 stored, … |
| `00bb7380` | DeserializeSnsGuildCreated | likely | Stack deserialiser: constructs a SnsIntegrationGuildCreated with 00bb62b0, initialises a reader state over the (param_1, param_1+param_2) byte range and consum… |
| `00bb7410` | DeserializeSnsGuildCreated_2 | likely | Byte-identical twin of 00bb7380 (only the SEH frame label differs): constructs a SnsIntegrationGuildCreated with 00bb62b0, decodes the (param_1, param_2) range… |
| `00bb7590` | WriteU16U16U32 | likely | Serialises a source record into the packet writer: appends the low 16 bits of source+4, the low 16 bits of source+8 and the dword at source+0xc, each copy clam… |
| `00bb7710` | CopyAssign_TwoStringsAndList | likely | Copies the first two dwords, copy-assigns the IString at +8, and when the source and destination list subobjects at +0x18 differ clears the destination list th… |
| `00bb90c0` | StartPostAppWeiboRequest | likely | Formats a postAppWeibo XML request from the supplied message and account fields, allocates a tracking record with type 2, and passes it to the secure-signature… |
| `00bb9250` | CopyPendingRecordAndSend | likely | Allocates a pending record, copies the supplied record into it when allocation succeeds, and passes its identifier to the secure-signature sender. |
| `00bb9280` | CopyTypedPendingRecordAndSend | likely | Allocates a pending record, stores type 3, copies the supplied record when allocation succeeds, and passes its identifier to the secure-signature sender. |
| `00bb5fd0` | SnsFindInterfaceAA264C6F | speculative | Walks the interface list at [[[this+0x244]+500]+0x11d4] (next pointer at node+0x14) looking for the entry whose key at +0x10 equals -0x55d9b391 (0xAA264C6F) an… |
| `01081880` | CounterAssignAndIncrement | speculative | Fastcall on this (bytes: lea esi,[ecx+0x18]; push esi; add ecx,0x1c; call 00bb7eb0): calls 00bb7eb0 with receiver this+0x1c and the address this+0x18 pushed; w… |
| `01081a00` | HashListClearViaRemoveSns | speculative | Loop: while the list head at this+0x8 is non-null, calls 00bb8000 on the head; empties the list through that removal routine. |

### `net.youtube`

**YouTube::FunctionObject** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0108d4e0` | YouTube::FunctionObject::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (flag byte argument): stores the YouTube::FunctionObject vftable at this, then calls _free on this when bit 0 of the flag is set. Ret… |
| `0108d4c0` | FunctionObjectDtor | likely | Fastcall: stores the YouTube::FunctionObject vftable at this and returns. Nothing else is done. |

**YouTube::FunctionObjectT<YouTube::PostRequest,YouTube::PostResponse,YouTube::PostCallback>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0108fe40` | PostRequestFunctionObject_Ctor | confirmed | Constructor: stores YouTube::FunctionObject::vftable and then the FunctionObjectT<PostRequest,PostResponse,PostCallback> vftable at *this, stores param_2 at +4… |
| `0108ff90` | PostRequestFunctionObject_IsTerminal | confirmed | Virtual slot [2]: returns 1 when the status at +0x3f0 is 5 or 6, otherwise 0. |
| `01090250` | PostRequestFunctionObject_ScalarDeletingDtor | confirmed | Virtual slot [0]: calls the dtor body 01090120 on this, then frees the block with _free when bit 0 of the argument byte is set; returns this. |
| `01090270` | PostRequestFunctionObject_Step | confirmed | Virtual slot [1]: switches on status +0x3f0: 0 calls 01090060, 1 calls 0108fff0, 2 calls 0108e540, 3 calls 0108f200, 4 sets status 5 and calls callback +0x3c8 … |
| `0108ffb0` | PostRequestFunctionObject_MarkTimeout | likely | Virtual slot [3]: stores 4 into the status at +0x3f0 and returns. The step 01090270 calls this slot (vtbl+0xc) when the elapsed-time limit at +0x404 has passed. |
| `01090120` | — | unknown | Destructor body: restores FunctionObjectT<PostRequest>::vftable, releases the refcounted object at +0x238 (LOCK decrement; vtbl+4 at zero), calls 007034d0 and … |

**YouTube::FunctionObjectT<YouTube::StatusRequest,YouTube::StatusResponse,YouTube::StatusCallback>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01090480` | StatusRequestFunctionObject_IsTerminal | confirmed | Virtual slot [2]: returns 1 when the status at +0x378 is 5 or 6, otherwise 0. |
| `01090730` | StatusRequestFunctionObject_ScalarDeletingDtor | confirmed | Virtual slot [0]: calls the dtor body 01090600 on this, then frees the block with _free when bit 0 of the argument byte is set; returns this. |
| `01090750` | StatusRequestFunctionObject_Step | confirmed | Virtual slot [1]: switches on status +0x378: 0 calls 01090540, 1 calls 010904e0, 2 calls 0108e990, 3 calls 0108f370, 4 sets status 5 and calls callback +0x350 … |
| `010904a0` | StatusRequestFunctionObject_MarkTimeout | likely | Virtual slot [3]: stores 4 into the status at +0x378 and returns. |
| `01090330` | — | unknown | Constructor: stores YouTube::FunctionObject::vftable then FunctionObjectT<StatusRequest,StatusResponse,StatusCallback>::vftable at *this, stores param_2 at +4,… |
| `01090600` | — | unknown | Destructor body: restores FunctionObjectT<StatusRequest>::vftable, releases the refcounted object at +0x1c0 (LOCK decrement; vtbl+4 at zero), calls 007034d0 an… |

**YouTube::FunctionObjectT_SSL<YouTube::AuthenticateRequest,YouTube::AuthenticateResponse,YouTube::AuthenticateCallback>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01090810` | AuthenticateRequestFunctionObject_Ctor | confirmed | Constructor: stores YouTube::FunctionObject::vftable then FunctionObjectT_SSL<AuthenticateRequest,AuthenticateResponse,AuthenticateCallback>::vftable at *this,… |
| `010909c0` | AuthenticateRequestFunctionObject_IsTerminal | confirmed | Virtual slot [2]: returns 1 when the status at +0x3a0 is 5 or 6, otherwise 0. |
| `01090ce0` | AuthenticateRequestFunctionObject_DtorBody | confirmed | Destructor body: restores FunctionObjectT_SSL<AuthenticateRequest>::vftable, calls 00f99910, 00f91de0, 00f99670, 0108e0f0, then 01091980 and 01091830 (member t… |
| `01090d90` | AuthenticateRequestFunctionObject_ScalarDeletingDtor | confirmed | Virtual slot [0]: calls the dtor body 01090ce0 on this, then frees the block with _free when bit 0 of the argument byte is set; returns this. |
| `010909e0` | AuthenticateRequestFunctionObject_MarkTimeout | likely | Virtual slot [3]: stores 4 into the status at +0x3a0 and returns. |
| `01091200` | — | unknown | Virtual slot [1]: switches on status +0x3a0: 0 calls 01090b20, 1 calls 01090a70, 2 calls 01090db0, 3 calls 01090fb0, 4 sets status 5 and calls callback +0x39c … |

**YouTube::HttpHandler** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0108e000` | YouTubeHttpHandlerCtor | likely | Fastcall: stores the HttpLibrary::HttpConnectionHandler vftable at this and the TcpLibrary::TcpConnectionHandler vftable at this+4, then installs the YouTube::… |
| `0108e0a0` | YouTube::HttpHandler::SetWord8 | likely | Virtual slot [0] of the primary vtable (10 bytes, not a destructor): stores its third argument at this+8. Returns no value. |
| `0108e0f0` | YouTube::HttpHandler::DtorBody | likely | Fastcall, SEH framed, called from EH funclets and from the derived destructors: stores the HttpHandler vftable at this and this+4, installs the SoeUtil::Array<… |
| `0108e190` | HttpHandler::AppendBytes | likely | Virtual slot [1] of the secondary vtable at this+4 (64 bytes): computes size+n at this+0x154; grows the char buffer at this+0x150 through 00f601e0(size+n, 0) w… |
| `0108f1e0` | YouTubeHttpHandler_Slot1SetFlag160 | speculative | Virtual slot [1] of YouTube::HttpHandler: calls FUN_00f62650 with the single argument, then sets the byte at this+0x160 to 1. |
| `0108e0e0` | — | unknown | Virtual slot [0] of the secondary vtable at this+4 (8 bytes): calls 0108e1d0 with no arguments and returns. ECX is passed through unchanged. |

**YouTube::YouTubeUploader** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0108c610` | YouTubeUploader::ForwardVirtual2 | likely | Fastcall virtual slot [2] (19 bytes): if the pointer at this+8 is nonzero it calls that object's vtable slot [2] (+8) with no arguments; otherwise it returns. |
| `0108c630` | YouTubeUploader::ForwardVirtual3 | likely | Fastcall virtual slot [3] (19 bytes): if the pointer at this+8 is nonzero it calls that object's vtable slot [3] (+0xc) with no arguments; otherwise it returns. |
| `0108cb30` | YouTubeUploader::ForwardVirtual0 | likely | Virtual slot [0] (54 bytes, not a destructor: no vtable store, no free): when the fourth argument is zero it calls 006f9e60 on param_3; then, when the pointer … |
| `0108c5f0` | — | unknown | Fastcall virtual slot [1] (19 bytes): if the pointer at this+8 is nonzero it calls that object's vtable slot [1] (+4) with no arguments; otherwise it returns. |
| `0108c790` | — | unknown | Fastcall, SEH framed: stores the YouTube::YouTubeUploader vftable at this, calls 0108f6d0 with ECX = this, then zeroes this+8, stores PTR_01884620 at this+0xc,… |
| `0108c810` | — | unknown | Fastcall, SEH framed: stores the YouTube::YouTubeUploader vftable at this, installs the SoeUtil::IString<char> vftable at this+0x10, releases the embedded stri… |

**YouTube::YouTubeUploaderImpl_SingleUse** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0108c8a0` | YouTubeUploaderImplSingleUseCtor | likely | Fastcall, SEH framed: stores the YouTube::YouTubeUploaderImpl_SingleUse vftable at this, calls 0108f6d0, then sets this+8 to PTR_01884620, installs the SoeUtil… |
| `0108c940` | YouTubeUploaderImplSingleUse::SetCode4Or5 | likely | Virtual slot [1] (18 bytes): stores (param_4 != 0) + 4 at this+0x1c, that is 5 when the fourth argument is nonzero and 4 otherwise. Returns no value. |
| `0108c9f0` | YouTubeUploaderImplSingleUseFactory | likely | Fastcall (ECX = out pointer): allocates 0x20 bytes with 00d833a6 (operator new); on success constructs the object with 0108c8a0 and stores the result in *param… |
| `0108cf20` | YouTubeUploaderImplSingleUse::SetCode2Or5 | likely | Virtual slot [0] (45 bytes, not a destructor): when the fourth argument is zero it calls 006f9e60 on param_3 and stores 2 at this+0x1c; otherwise it stores 5 a… |
| `0108c960` | — | unknown | Fastcall, SEH framed: stores the YouTube::YouTubeUploaderImpl_SingleUse vftable at this, installs the SoeUtil::IString<char> vftable at this+0xc, releases the … |

**(no class)** (72)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0108fbc0` | PostRequestCopyCtor | confirmed | Thiscall copy constructor: copies the request record from the argument through 0108f9b0, then initialises seven SoeUtil::IString<char> fields at +0x1a0, +0x1b0… |
| `0108fd90` | AuthRequestCopyCtor | confirmed | Thiscall copy constructor: copies the request record from the argument through 0108f9b0, then initialises two SoeUtil::IString<char> fields at +0x1a0 and +0x1b… |
| `01091a50` | DtorSevenIStringsPostRecord | confirmed | Destructor body for seven IString fields at +0x1a0, +0x1b0, +0x1c0, +0x1d0, +0x1e0, +0x1f0 and +0x200 (the layout written by 0108fbc0): stores the IString vtab… |
| `01091dc0` | ClientLoginBodyBuild | confirmed | Thiscall (ECX = the record, one stack argument = the output SoeUtil::IString; byte-verified: the output is loaded from [esp+8] into ESI and passed as ECX to 00… |
| `010920f0` | UploadHeadersBuild | confirmed | Thiscall (ECX=this): appends GoogleLogin auth= and the string at this+0x1a4 through 00402770, writes the header name Authorization: through 00f98b70, stores th… |
| `010922b0` | PostResponseDump | confirmed | Thiscall debug dump: reserves through 00401df0 when the global count is positive, appends the line PostResponse_BEG, the response text at this+4 and the line P… |
| `010923e0` | StatusResponseDump | confirmed | Thiscall debug dump, same shape as 010922b0: appends StatusResponse_BEG, the response text at this+4 and StatusResponse_END through 00402770 after an optional … |
| `010924f0` | MultipartBoundaryString | confirmed | One-time initialiser: on first call with the flag at DAT_01cb8504 clear it sets the flag, runs 00f94b80 and registers an atexit cleanup; then if DAT_01ba59c4 i… |
| `01092580` | YouTubeClientLoginStaticsInit | confirmed | One-time initialiser: when the flag DAT_01ba59c5 is set it creates the static ClientLogin URL https://www.google.com/youtube/accounts/ClientLogin (00f95440), s… |
| `01092710` | AuthResponseDump | confirmed | Thiscall debug dump of a parsed ClientLogin response: appends AuthenticateResponse_BEG, then the Auth= value (fields at +8) through 0071be40, and two further k… |
| `01092b40` | YouTubeUploadStaticsInit | confirmed | One-time initialiser: when the flag DAT_01ba59c6 is set it creates the static upload URL http://uploads.gdata.youtube.com/feeds/api/users/default/uploads (00f9… |
| `01092d00` | UploadAuthHeadersBuild | confirmed | Fastcall-style header builder with two stack arguments: sets the dword at this (ESI) to 1, sets close and soe_dev, appends X-GData-Client and X-GData-Key (thro… |
| `01092ed0` | YouTubeAtomEntryBuild | confirmed | Thiscall: zeroes the string object at +4..+0x10, reserves 0x2000 bytes, calls 010924f0 and 01092960 on a local, sets the content type application/atom+xml; cha… |
| `0108ca60` | ScalarDeletingDtorForSingleUse | likely | Thiscall (ECX = object, flag byte): calls 0108c960 (the single-use uploader destructor body) on this, then calls _free on this when bit 0 of the flag is set. R… |
| `0108ca80` | DeleteOwnedSingleUse | likely | Fastcall (ECX = holder): if the pointer at *holder is nonzero, calls the single-use destructor body 0108c960 on it and frees it with _free. Returns no value. |
| `0108cc70` | VideoUploadMetadataSubmit | likely | Thiscall (ECX = object, param_2 = description, param_3 = trailing string), SEH framed, 528-byte stack buffer: returns 0 when the dword at this+0x18 is zero. Ot… |
| `0108d020` | VideoUploadMetadataSubmitGuarded | likely | Thiscall (ECX = object with state at +0x1c and a pointer at +0x14; param_2, param_3), SEH framed, 528-byte stack buffer: runs only when the state at +0x1c is 2… |
| `0108d5f0` | ReportProgressPercent | likely | Fastcall (ECX = object): when the 64-bit total at +0x3e0 is nonzero, computes ((total - base) * 100) / total with __allmul and __alldiv, where base is the 64-b… |
| `0108e1d0` | HttpHandlerScalarDeletingDtorViaSecondary | likely | Thiscall (flag byte argument): calls 0108e0f0 on this, then calls _free on this when bit 0 of the flag is set. Returns this. |
| `0108f200` | HttpResponseFinishPath_0x220 | likely | Thiscall (ECX=this): returns early unless the int at +0x248 equals 2. With the flag at +0x3a0 clear it sets +0x3f0=5 and calls vtbl+4 of the callback at +0x3c8… |
| `0108f370` | HttpResponseFinishPath_0x1a8 | likely | Thiscall: returns early unless +0x1d0 equals 2. With +0x328 clear it sets +0x378=5 and calls vtbl+8 of the callback at +0x350 with (this+4, this+0x1a8, -8). Ot… |
| `0108f9b0` | RequestRecordCopy | likely | Thiscall copy of the request record: copies the dword at +0, the byte at +4, and the dwords at +8, +0x10 and +0x14, then calls 0108ea60 (range copy) on 19 stri… |
| `01090060` | PostRequestOpenConnectionStep | likely | Fastcall step (state 0 of the PostRequest state machine): calls 00f98f10 on +0x244 (or passes 0 when +0x240 is null) and 00f98f20 on +0x240, picks port 0x50, o… |
| `010904e0` | StatusResponseCodeStep | likely | Fastcall step (state 1 of the StatusRequest machine): reads the int at (+0x1c0 object)+0x1c. When it is 1 it sets status +0x378 to 2. When it is another non-ze… |
| `01090540` | StatusRequestOpenConnectionStep | likely | Fastcall step (state 0 of the StatusRequest machine): calls 00f98f10 on +0x1cc (or passes 0 when +0x1c8 is null) and 00f98f20 on +0x1c8, picks port 0x50 or 0x1… |
| `01090a70` | AuthRequestPollStep | likely | Fastcall poll step: calls 00f91e00 to get a state code and 00f92720 to get a ready flag. State 0: returns when the ready flag is set, otherwise falls to the fa… |
| `01090b20` | AuthRequestOpenStep | likely | Fastcall state 0 of the AuthenticateRequest machine: calls 00f98f10 on +0x214 (0 when +0x210 is null) and 00f98f20 on +0x210, reads the data pointers of the st… |
| `01090db0` | AuthRequestSendClientLogin | likely | Fastcall state 2 of the AuthenticateRequest machine: builds the form body with 01091dc0 into a local SoeUtil::IString and stores its length sign-extended at +0… |
| `01090fb0` | AuthResponseHandler | likely | Fastcall state 3 of the AuthenticateRequest machine: requires state +0x218 == 2 (else 00f92720 must be true or status 5 with callback -7); when the elapsed fie… |
| `010912c0` | QueueAuthenticateRequest | likely | Thiscall: returns 0 unless the flag byte at +0x60 is set and 01091700 holds (both length fields +0x1a8 and +0x1b8 non-zero). It then allocates 0x3b8 bytes, con… |
| `01091380` | QueuePostRequest | likely | Thiscall: returns 0 unless the flag byte at +0x60 is set and 010917c0 holds (seven length fields +0x1a8 to +0x208 all non-zero). It then allocates 0x408 bytes,… |
| `01091440` | QueueStatusRequest | likely | Thiscall: returns 0 unless the flag byte at +0x60 is set. Then it allocates 0x390 bytes, constructs a StatusRequest with 01090330 (arguments from +0x40, +0x50,… |
| `01091e70` | ClientLoginResponseParse | likely | Thiscall parser over a response text between two pointers: skips whitespace and matches three keywords from the table at 01ba5980 (each entry a pointer and len… |
| `010927c0` | AuthResponseDumpLocal | likely | Builds a local SoeUtil::IString, calls 01092710 to dump the auth response into it, releases the string through the vtable slot 2 when its count is positive, an… |
| `01092a00` | MultipartBinaryPartCtor | likely | Thiscall constructor: zeroes the dwords at +4, +8, +0xc and +0x10, calls 010924f0 (boundary string) and 01092960 (five range strings) on this, then calls 00f63… |
| `010933b0` | UploadRecordCtor | likely | Fastcall constructor: copies the static upload record returned by 01092b40 through 0108f9b0, then initialises seven SoeUtil::IString<char> fields at +0x1a0, +0… |
| `010934e0` | ClientLoginRecordCtor | likely | Fastcall constructor: copies the static ClientLogin record returned by 01092580 through 0108f9b0, then initialises two SoeUtil::IString<char> fields at +0x1a0 … |
| `0108cb70` | RunStepTimeout30000 | speculative | Thiscall (ECX = object with an IString at +0x10): releases the IString at +0x10 (count before the buffer at +0x14; vtable slot [2] when it reaches zero), reset… |
| `0108cf50` | RunStepTimeout30000Alt | speculative | Thiscall (ECX = object, param_2, param_3), SEH framed, 448-byte stack buffer: sets the dword at this+0x1c to 1, calls 010934e0, applies 006f9e60 to param_3 and… |
| `0108d2f0` | Wrapper_0108d2f0 | speculative | Bare forwarding call: calls 0108d020 with no arguments and returns (7 bytes). |
| `0108d5b0` | SetStatus5AndNotify | speculative | Thiscall (ECX = object, param_2 = flag): when the flag is nonzero stores 5 at this+0x3f0; then, when the observer pointer at this+0x3c8 is nonzero, calls its v… |
| `0108d660` | SetStatus5AndNotifyB | speculative | Thiscall (ECX = object, param_2 = flag): when the flag is nonzero stores 5 at this+0x378; then, when the observer pointer at this+0x350 is nonzero, calls its v… |
| `0108e540` | UploadChunkStep | speculative | Thiscall state step on the object in ECX. When +0x3cc is set it finishes (status 3 at +0x3f0) or calls the thunk FUN_00f99920 (tail-jumps to vtable slot 0 of t… |
| `0108e990` | StatusRequestSend | speculative | Thiscall method: builds request text through FUN_00f96090 into a local SoeUtil::IString, passes its begin pointer and length to the thunk FUN_00f99920 (vtable … |
| `0108edd0` | OneTimeInitTwoStringsHandle | speculative | Thiscall: when the flag byte at +0x60 is clear and (param_2 or param_3 is non-zero) and param_4 is non-zero: allocates 0xf8 bytes (FUN_00d833a6), constructs th… |
| `0108efa0` | FunctionQueueDrainStep | speculative | Thiscall: when +0x34 (count) is non-zero, takes a time value via FUN_007027f0, sets a limit with FUN_010eb0f0, then for each element: takes the head element (b… |
| `0108f140` | AdjustorThunk_0108efa0 | speculative | Seven-byte adjustor thunk (8b 09 / e9 rel32): loads ECX from the dword at this and jumps to 0108efa0. |
| `0108f740` | Dtor_DequeTwoIStrings_Scalar | speculative | Scalar deleting destructor of the 0108f570 class: calls the dtor body 0108f570 on this and frees the block with _free when bit 0 of the argument byte is set; r… |
| `0108f760` | Dtor_OwnedTwoIStringsPtr | speculative | Destructor helper for a pointer member: when *this is non-null it calls the 0108f570 dtor body on that object and frees it with _free; no action for a null poi… |
| `0108fd40` | RequestRecordCopyWrap | speculative | Thiscall wrapper (byte-verified, same pattern as 0108f960): calls 0108f9b0 with ECX still holding this and its stack argument pushed as the source, then return… |
| `0108fff0` | PostResponseCodeStep | speculative | Fastcall: reads the int at (+0x238 object)+0x1c. When it is 1 it sets the status at +0x3f0 to 2. When it is another non-zero value it sets +0x3f0 to 5 and call… |
| `010909f0` | AuthRequestFinish | speculative | Thiscall: when the argument is non-zero it sets status +0x3a0 to 5. Then, when the callback object at +0x39c is non-null, calls its vtable slot 0 with (this+4,… |
| `010914f0` | AdjustorThunk_010912c0 | speculative | Seven-byte adjustor thunk (8b 09 / e9 rel32): loads ECX from the dword at this and jumps to 010912c0. |
| `01091500` | AdjustorThunk_01091380 | speculative | Seven-byte adjustor thunk (8b 09 / e9 rel32): loads ECX from the dword at this and jumps to 01091380. |
| `01091700` | AuthFieldsNonEmpty | speculative | Thiscall predicate: returns 1 when the dwords at +0x1a8 and +0x1b8 are both non-zero, otherwise 0 (byte-verified: cmp [ecx+1a8],0; je; cmp [ecx+1b8],0; je; mov… |
| `010917a0` | PredicateField8OrField28 | speculative | Thiscall predicate (byte-verified): returns 1 when (+0x8 != 0 and +0x18 != 0) or +0x28 != 0, else 0. |
| `010917c0` | PostFieldsNonEmpty7 | speculative | Thiscall predicate (byte-verified): returns 1 only when the dwords at +0x1a8, +0x1b8, +0x1c8, +0x1d8, +0x1e8, +0x1f8 and +0x208 are all non-zero, else 0. |
| `01091810` | FieldAt8NonZero | speculative | Thiscall predicate (byte-verified: cmp dword [ecx+8],0; setne al; ret): returns 1 when the dword at +0x8 is non-zero, else 0. |
| `01091820` | FieldAt8NonZero_Alt | speculative | Thiscall predicate, byte-identical to 01091810 (cmp dword [ecx+8],0; setne al; ret): returns 1 when the dword at +0x8 is non-zero. |
| `01091830` | DtorTwoIStringsAt1A0 | speculative | Destructor body for two IString fields: stores SoeUtil::IString<char>::vftable at +0x1b0 and releases its buffer (LOCK decrement; vtbl+8 when the count reaches… |
| `01092880` | TableReplaceAppend | speculative | Thiscall: walks the bytes between this+4 and this+4+this+8 and, at each position, compares against entries of a pattern table (PTR_DAT_01ba5994 array of 4-dwor… |
| `01092960` | CopyFiveRangeStrings | speculative | Thiscall: calls 0108ea60 (range copy) on the argument object and on its four following 0x14-byte string slots at +0x14, +0x28, +0x3c and +0x50, then returns th… |
| `0108d2e0` | — | unknown | Bare forwarding call: calls 0108cf50 with no arguments and returns (7 bytes). |
| `0108f570` | — | unknown | Destructor body of the class with two IString members and a Deque: calls vtbl[0] with 1 on each element of the deque (block table +0x14, index +0x38, count +0x… |
| `010916b0` | — | unknown | Calls 00f98190 with the address of a local cookie-protected stack slot (MSVC /GS check bookkeeping as its argument), then returns. Called from two EH unwind fu… |
| `01092000` | — | unknown | Cookie-guarded wrapper: calls 00f633b0 with the MSVC /GS cookie local as its argument (stack bookkeeping, not a real argument), then returns. |
| `01092050` | — | unknown | Cookie-guarded wrapper, byte-identical in shape to 01092000: calls 00f633b0 with the stack cookie local as its argument, then returns. |
| `010920a0` | — | unknown | Cookie-guarded wrapper, byte-identical in shape to 01092000: calls 00f633b0 with the stack cookie local as its argument, then returns. |
| `010921e0` | — | unknown | Cleanup routine: calls 00f633b0 three times (first with the stack cookie local, then with no argument, then again) and returns. |
| `01092ab0` | — | unknown | Thiscall constructor: zeroes the dwords at +4, +8, +0xc and +0x10, calls 010924f0 (boundary string) and 01092960 (five range strings) on this, then calls 00f96… |
| `01092e60` | — | unknown | Thiscall wrapper: calls 00f98960 with the stack cookie local (MSVC /GS bookkeeping), then calls 01092d00 with its two stack arguments and returns the first arg… |
| `01093340` | — | unknown | Thiscall: calls 01092ed0 (Atom entry builder) on the argument, then 01092a00 (binary part constructor) on the argument, then 01092ab0 (part constructor) on the… |

### `ui.chat`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00902730` | BaseClient::SetChatText | confirmed | Virtual with one stack argument param_2. Gated on FUN_00977740("ChatHandler"), appends ':SetChatText' to an IString, builds a one-entry Lua argument array (FUN… |

**Chat::ChatDisplayUChatTextDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01282210` | Chat::ChatDisplayUChatTextDisplayAction::PushUpdateChatLog | confirmed | Virtual slot [2] (ret 8). Reads display-action string arguments 1, 2 and 3 through 012f8f60 with kStringTypeID checks (012f5a60 assert on mismatch). Argument 1… |

**Chat::ChatGetConversationID** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01282670` | Chat::ChatGetConversationID::RespondGetConversationID | confirmed | Virtual slot [2] (ret 8 path). Reads the string argument named ConversationName through 012eb250, copies it into a local std::string, builds a key with 012dfbc… |

**Chat::DisplayEasyChatTextDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012827d0` | Chat::DisplayEasyChatTextDisplayAction::PushLogEasyChat | confirmed | Virtual slot [2] (ret 8). Reads argument 1 with 012f8f60 as an integer (kIntegerTypeID check, value 0 if the tag differs) and argument 2 as a string (kStringTy… |

**ClientQuickChatManager::InternalDs** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b8af00` | QuickChatInternalDs::ColumnName | confirmed | Maps column indices 0 through 11 to Id, ParentId, MenuText, ChatText, AnimationId, MenuIconId, ItemId, ItemCategory, ItemCount, IsFavorite, IsRecentlyUsed, and… |
| `00b8c390` | — | unknown | Calls a data-source virtual method, iterates rows in the nested collection, conditionally invokes row cleanup for entries with valid IDs, then calls another vi… |

**ClientQuickChatManager::InternalEmoteItemCategoriesDs** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b8afc0` | QuickChatCategoryDs::ColumnName | confirmed | Returns Id for index zero, CategoryName for index one, and the error marker for other indices. |
| `00b8c400` | InternalEmoteItemCategoriesDs::GetNestedFieldFC0 | confirmed | Returns the dword at nested object offset 0xfc0 reached through this object's field at offset 0x224. |

**QuickChatManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0107c580` | QuickChatManager::QuickChatManager | confirmed | Fastcall constructor (this in ECX, returns this): stores QuickChatManager::vftable; zeroes the dwords at +4, +8, +0xc and +0x10; clears the 4000 bytes at +0x14… |
| `0107c680` | QuickChatManager::Destruct | confirmed | Fastcall destructor body under an EH frame: stores QuickChatManager::vftable, then calls 00b8fca0 on the object (the chain-removal routine labelled accepted un… |
| `0107c6e0` | QuickChatManager::ScalarDeletingDtor | confirmed | Thiscall with one stack byte argument (ret 4): calls 0107c680 (QuickChatManager::Destruct) on this, then calls _free(this) through thunk 00d82463 when bit 0 of… |

**(no class)** (19)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008edac0` | ChatShowSystemMessage | confirmed | Thiscall with an unused second argument and a message record (param_3). Formats System Message:, newline, then the string at param_3+4 in quotes, via 00402fd0,… |
| `00901140` | StartChatLuaCall | confirmed | Gated on FUN_00977740("ChatHandler"), appends ':StartChat' to an IString, builds a one-entry Lua argument array (FUN_0070ec40) holding a constant string via FU… |
| `00901a50` | OnReceiveEmote | confirmed | Thiscall with five stack arguments (param_2..param_6; param_6 is a byte). Formats a line with FUN_00402fd0 from DAT_0175b2f8. Calls FUN_0102d0a0 and FUN_0102d1… |
| `00b11810` | OnReceiveChannel | confirmed | Queries the OnReceiveChannel hook; on its zero-result path, formats a channel line and passes it with resolved sender fields to a UI callback. |
| `00b11ab0` | OnReceiveWhisper | confirmed | Builds callback parameters from the sender and message, formats a local whisper line, queries OnReceiveWhisper, and sends the resulting text through the UI cal… |
| `00b120e0` | TellEchoHandler 🌐 | confirmed | Formats the localized tell-to-target confirmation, builds callback parameters from the target and message, then invokes OnReceiveWhisper. |
| `00b12b70` | CharSequenceEditDistance | confirmed | Builds a nested float dynamic-programming matrix, initializes its boundary costs, compares input characters per cell, uses the diagonal on equality and otherwi… |
| `00be89f0` | AIAlertNotification_Handle | confirmed | Logs 'HandleNotification: time %d:%2d, notification id %d' to ClientCais.log through 00704db0 (minutes and hours from the dword at param_2+0xc divided by 0x3c … |
| `008edba0` | DisplaySystemMessageDeduped | likely | Keeps the last shown text in a global IString (DAT_01be4920) and the time of its display in DAT_01be4918. When the text equals the cached text and the seconds … |
| `008f0cc0` | ShowFlaggedSystemMessage | likely | Thiscall with a message argument (param_2). When the byte at this+0x8e is nonzero, calls 008c6b60 with ECX = this and the arguments (1.0f = 0x3f800000, the flo… |
| `00b14200` | StoreChatHistoryEntry | likely | Finds or creates the keyed ChatNode list, removes its first node when the count is 10, allocates and appends a new node, copies two payload dwords, and stores … |
| `00c167f0` | SetChatColorFgBgSz | likely | Lua C function bound as SetChatColorFgBgSz: requires exactly three numeric arguments; it rounds each of arguments 3, 2 and 1 to integers (FUN_00e3a770 + ROUND)… |
| `00c16d80` | ProcessQuickChatChannelCommand | likely | Lua C function bound as ProcessQuickChatChannelCommand: requires more than one argument; when argument 1 is numeric and argument 2 is truthy it reads argument … |
| `0107c5f0` | ChainedFieldTransfer9 | likely | Thiscall with one pointer argument (buffer cursor base). Calls 0072d900(cursor, field, 0x5e) for nine fields of this (+0x0, +0x10, +0x4, +0x8, +0xc, +0x14, +0x… |
| `0107c700` | ParseKeyedRecordsFromIString | likely | Thiscall with one argument (source string). Clears the existing records through 00b8fca0, reads the string through 00fbf4a0, then for each token reads an IStri… |
| `00b1b5a0` | SendGuildSayChat | speculative | Assigns the C-string argument param_3 into a temporary SoeUtil::IString (SoeUtil::IStringAssign 00402ce0), builds a stack frame holding the caller's (param_1, … |
| `009012b0` | — | unknown | Nine stack arguments, no thiscall binding shown. Looks up an object with FUN_00984960(param_1, param_2); when found it reads the values at +0x71c, +0x720, +0x7… |
| `00c16d00` | — | unknown | Lua C function bound as ProcessQuickChatCommand: when argument 1 is numeric it reads it (FUN_00e3a7b0) and calls FUN_00b97380(index); returns 0. |
| `00c16d40` | — | unknown | Lua C function bound as ProcessQuickGroupChatCommand: when argument 1 is numeric it reads it and calls FUN_00b97480(index); returns 0. |

### `ui.friends`

**Engine::AccountAcceptFriendRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128f160` | Engine::AccountAcceptFriendRequest::GetActionName | confirmed | Returns the address of the constant string AcceptFriendRequest (6 bytes, mov eax,imm32 then ret). |
| `0128f180` | Engine::AccountAcceptFriendRequest::GetSourceTag | confirmed | Returns the address of the constant string AcceptFriendRequest : 0 : (accounthandlers.cc line 1086); 6 bytes, mov eax,imm32 then ret. |
| `0128f0f0` | — | unknown | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountAcceptFriendRequest::vftable into *this with EH state 0; when the int argument is nonzer… |

**Engine::AccountAddFriend** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128ec20` | Engine::AccountAddFriend::GetActionName | confirmed | Returns the address of the constant string AddFriend (6 bytes, mov eax,imm32 then ret). |
| `0128ec40` | Engine::AccountAddFriend::GetSourceTag | confirmed | Returns the address of the constant string AddFriend : 0 : (accounthandlers.cc line 948); 6 bytes, mov eax,imm32 then ret. |
| `01295220` | Engine::AccountAddFriend::AddFriendOrShowError | likely | Returns 1. Reads AccountID with default zero, passes it to FUN_012dfbc0 and FUN_012de760, and returns early if either FUN_012de760 or FUN_0127ebb0 is zero. Oth… |
| `0128ebb0` | — | unknown | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountAddFriend::vftable into *this with EH state 0; when the int argument is nonzero calls FU… |

**Engine::AccountAddIgnore** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128eec0` | Engine::AccountAddIgnore::GetActionName | confirmed | Returns the address of the constant string AddIgnore (6 bytes, mov eax,imm32 then ret). |
| `0128eee0` | Engine::AccountAddIgnore::GetSourceTag | confirmed | Returns the address of the constant string AddIgnore : 0 : (accounthandlers.cc line 1024); 6 bytes, mov eax,imm32 then ret. |
| `01295650` | Engine::AccountAddIgnore::AddIgnoreOrShowError | likely | Returns 1. Reads AccountID with default zero, passes it to FUN_012dfbc0 and FUN_012de760, and proceeds only when FUN_012de760 and FUN_0127ebb0 are both nonzero… |
| `0128ee50` | — | unknown | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountAddIgnore::vftable into *this with EH state 0; when the int argument is nonzero calls FU… |

**Engine::AccountOnReceiveFriendConfirmRequestUC** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128fda0` | — 🌐 | unknown | Reads string arguments 0 and 1 and integer argument 2 (type-checked), formats a prompt string, and opens a GenericPromptDialog through OpenDialog: TitleText Tc… |

**Engine::AccountOnReceiveFriendConfirmResponseUC** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01290210` | — 🌐 | unknown | Type-checks string arguments 0 and 1, integer argument 2 and boolean argument 3; on the decline result opens a GenericInfoDialog titled Friend Declined with th… |

**Engine::AccountOnReceiveFriendLoginUC** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01290c90` | Engine::AccountOnReceiveFriendLoginUC::Handle 🌐 | speculative | Type-checks string arguments 0 and 1 and integer argument 2, then runs the friend-list refresh: builds an UpdateFriends command (13-character literal) with Lis… |

**Engine::AccountOnReceiveFriendLogoutUC** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01290f70` | Engine::AccountOnReceiveFriendLogoutUC::Handle 🌐 | likely | Type-checks string arguments 0 and 1 and integer argument 2, then runs the friend-list refresh: builds an UpdateFriends command (13-character literal) with Lis… |

**Engine::AccountOnRequestAddFriendUC** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01291250` | Engine::AccountOnRequestAddFriendUC::ApplyAddFriendResult | likely | Returns 1. Reads args 0,1,3,4,5 as strings, arg 2 as an integer, and only type-checks arg 6 as boolean through FUN_012f8f60. If FUN_012d08e0 is nonzero, replac… |

**Engine::AccountOnRequestAddIgnoreUC** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01291c70` | Engine::AccountOnRequestAddIgnoreUC::ApplyAddIgnoreResult | likely | Returns 1. Reads args 0,1,3,4 as strings and arg 2 as an integer through FUN_012f8f60. On code 0, calls FUN_012cfde0 and FUN_012d0110 with args 3 and 4; if FUN… |

**Engine::AccountOnRequestFriendConfirmUC** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012928f0` | Engine::AccountOnRequestFriendConfirmUC::ApplyFriendConfirm… | likely | Returns 1. Reads args 0,1,3,4 as strings, arg 2 as an integer, and arg 5 as a boolean through FUN_012f8f60. When arg 2 is zero, arg 5 false bypasses the succes… |

**Engine::AccountOnRequestRemoveFriendUC** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01293230` | Engine::AccountOnRequestRemoveFriendUC::ApplyRemoveFriendRe… | likely | Returns 1. Reads args 0,1,3,4 as strings and arg 2 as an integer through FUN_012f8f60. On code 0, calls FUN_012cfde0 and FUN_012cf2b0 with args 3 and 4; if FUN… |

**Engine::AccountOnRequestRemoveIgnoreUC** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01293e90` | Engine::AccountOnRequestRemoveIgnoreUC::ApplyRemoveIgnoreRe… | likely | Returns 1. Reads args 0,1,3,4 as strings and arg 2 as an integer through FUN_012f8f60. On code 0, calls FUN_012cfde0 and FUN_012cf2d0 with args 3 and 4; if FUN… |

**Engine::AccountRejectFriendRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128f2b0` | Engine::AccountRejectFriendRequest::GetActionName | confirmed | Returns the address of the constant string RejectFriendRequest (6 bytes, mov eax,imm32 then ret). |
| `0128f2d0` | Engine::AccountRejectFriendRequest::GetSourceTag | confirmed | Returns the address of the constant string RejectFriendRequest : 0 : (accounthandlers.cc line 1127); 6 bytes, mov eax,imm32 then ret. |
| `0128f240` | — | unknown | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountRejectFriendRequest::vftable into *this with EH state 0; when the int argument is nonzer… |

**Engine::AccountRemoveFriend** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128ed00` | Engine::AccountRemoveFriend::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountRemoveFriend::vftable into *this with EH state 0; when the int argument is nonzero calls… |
| `0128ed70` | Engine::AccountRemoveFriend::GetActionName | confirmed | Returns the address of the constant string RemoveFriend (6 bytes, mov eax,imm32 then ret). |
| `0128ed90` | Engine::AccountRemoveFriend::GetSourceTag | confirmed | Returns the address of the constant string RemoveFriend : 0 : (accounthandlers.cc line 997); 6 bytes, mov eax,imm32 then ret. |

**Engine::AccountRemoveIgnore** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128f010` | Engine::AccountRemoveIgnore::GetActionName | confirmed | Returns the address of the constant string RemoveIgnore (6 bytes, mov eax,imm32 then ret). |
| `0128f030` | Engine::AccountRemoveIgnore::GetSourceTag | confirmed | Returns the address of the constant string RemoveIgnore : 0 : (accounthandlers.cc line 1058); 6 bytes, mov eax,imm32 then ret. |
| `0128efa0` | — | unknown | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountRemoveIgnore::vftable into *this with EH state 0; when the int argument is nonzero calls… |

**Engine::AccountShowFriends** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128ea60` | Engine::AccountShowFriends::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountShowFriends::vftable into *this with EH state 0; when the int argument is nonzero calls … |
| `0128ead0` | Engine::AccountShowFriends::GetActionName | confirmed | Returns the address of the constant string ShowFriends (6 bytes, mov eax,imm32 then ret). |
| `0128eaf0` | Engine::AccountShowFriends::GetSourceTag | confirmed | Returns the address of the constant string ShowFriends : 0 : (accounthandlers.cc line 927); 6 bytes, mov eax,imm32 then ret. |
| `01295090` | Engine::AccountShowFriends::OpenFriendsListDialog | confirmed | Returns 1. If FUN_0127a580 and then FUN_013b15e0 are nonzero, creates OpenDialog, sets GroupID to 2 and its XML value from FUN_013b1200, calls FUN_013afff0 bef… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013b1060` | WriteFriendsList | likely | Fills a vector through 013b0ef0 and the vtable call on the object at +0x8c, writes a Friends key (0130df10), then for each nonzero id calls 0143d650(out, id, 2… |
| `013b1200` | SubmitFriendsPayload | speculative | Builds a 180-byte payload with 013b1060(payload), then calls 0130e200, 0130d600(param_1) and 0130d9b0. |

### `ui.group`

**GroupDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7c960` | GroupDataSource::GetColumnName | confirmed | Column-name table of the group data source: returns the literal MinigameType when the index is 0 and <error> for every other index; it never touches this. |
| `00a7db70` | GroupDataSource::~GroupDataSource | confirmed | Destructor body: rewrites the three GroupDataSource vtable pointers (at +0, +0x30 and +0x34) and runs the shared base destructor FUN_0091bf60. |
| `00a7dc80` | GroupDataSource::GroupDataSource | confirmed | Constructor: registers the name BaseClient.GroupData through FUN_00922890, installs the three GroupDataSource vtables (+0, +0x30, +0x34) and stores the owner a… |
| `00a7dd00` | GroupDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the destructor body 00a7db70 and frees the object with _free when bit 0 of param_2 is set. |
| `00a7c980` | GroupDataSource::GetRowCount | likely | Reads the group pointer at this+0x224 (primary object +0x254), then returns 1 when that object's field +0x6ec is non-null and 0 otherwise (the compiled body is… |
| `00a7cd10` | GroupDataSource::GetColumnValue | likely | Row-value getter of the group data source. When the group pointer (this+0x224)->+0x6ec is null it returns 0; otherwise it returns 0 for any column other than 0… |

**GroupMemberDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7c810` | GroupMemberDataSource::GetColumnName | confirmed | Pure column-name table for the group member list: returns PlayerName, PlayerGuid, PlayerJobId, PlayerJobBackgroundIconId, PlayerJobIconId, PlayerJobRank, Playe… |
| `00a7cd60` | GroupMemberDataSource::GetColumnValue | confirmed | Row-value getter of the group member list (vtable slot 4). When the byte at +0x228 is clear it first rebuilds the rows through 00a7c9f0, resolves the group obj… |
| `00a7db00` | GroupMemberDataSource::~GroupMemberDataSource | confirmed | Destructor body: rewrites the three GroupMemberDataSource vtable pointers (at +0, +0x30 and +0x34) and then runs the shared base destructor FUN_0091bf60. |
| `00a7dbe0` | GroupMemberDataSource::GroupMemberDataSource | confirmed | Constructor: registers the name BaseClient.GroupMemberData through FUN_00922890, installs the three class vtables (+0, +0x30, +0x34), stores the owner argument… |
| `00a7dc60` | GroupMemberDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (slot 0 of the class vtable): calls the destructor body 00a7db00 and, when bit 0 of param_2 is set, releases the object with _free, … |
| `00a7c9d0` | GroupMemberDataSource::GetRowCount | likely | Reads the group pointer at this+0x224 (primary object +0x254) and, when its field +0x6ec is non-null, returns the count stored at group+0xc; when there is no g… |
| `00a7c9f0` | GroupMemberDataSource::RefreshMemberRows | likely | Rebuilds the member rows of the object at +0: calls virtual slot 3 (offset 0xc) and then slot 1 (offset 4) with 0 on the row-sink object at +0x34 to clear it, … |

**(no class)** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7ca90` | RaiseGroupMemberDataUpdatedEvent | confirmed | Refreshes the datasource at (this+0x254)+0x5d4 through virtual slot 4 and clears its byte at +0x258. It looks up HandlerGroupScreen and, only when that lookup … |
| `00a7cbd0` | RaiseGroupDataUpdatedEvent | confirmed | Refreshes the datasource at (this+0x254)+0x5d4 through virtual slot 4 and clears its byte at +0x258. It looks up HandlerGroupScreen and, only when that lookup … |
| `00a7d3b0` | RaiseSetGroupMemberSelectingJob | confirmed | Initializes the row index to 0. If param_2/param_3 differ from the owner fields +0x3a0/+0x3a4 it scans datasource rows starting at index 1, requiring a row val… |
| `00a7d660` | RaiseGroupMemberRowDataUpdatedEvent | confirmed | Refreshes the datasource at (this+0x254)+0x5d4 through virtual slot 4 and clears its byte at +0x258. It looks up HandlerGroupScreen and, only when registered, … |
| `00a7d830` | RaiseActivateOrDeactivateEncounterRequest | confirmed | Looks up HandlerGroupScreen. If registered, param_1<1 selects :DeactivateEncounterRequest with no parameters; otherwise it selects :ActivateEncounterRequest an… |
| `00a81130` | FindGuildMemberDataSourceByName | confirmed | Hashes param_2 with 00705bc0, follows bucket hash%1000 in the table at this+0xaea4 via entry+4, compares the stored hash at entry+8, and RTTI-casts a found obj… |
| `00a899f0` | CombatGroupHandlerEnsureOpen | confirmed | If flag 0x40 at this+0xc is clear, returns unless the gate passes ((*(this+8)+0x778)+0x10 non-zero or FUN_008a91d0 returns non-zero); otherwise calls 009797a0 … |
| `00a89a40` | CombatGroupHandlerClose | confirmed | If flag 0x40 at this+0xc is set, calls 009797a0 with CombatGroupHandler:Close (with ECX loaded from DAT_01be1090) and clears flag 0x40; returns otherwise. Byte… |
| `00a89a70` | VehicleRaceGroupHandlerClose | confirmed | If flag 0x20 at this+0xc is set, calls 009797a0 with VehicleRaceGroupHandler:Close (ECX from DAT_01be1090) and clears flag 0x20; returns otherwise. Bytes match… |
| `00a89aa0` | SoccerGroupHandlerEnsureOpen | confirmed | If flag 4 at this+0xc is clear and the object at [this+8]+200 is non-null with bit 12 of its dword at +0x6f8 set, calls 009797a0 with SoccerGroupHandler:Init a… |
| `00a89b00` | SoccerGroupHandlerClose | confirmed | If flag 4 at this+0xc is set, calls 009797a0 with SoccerGroupHandler:Close (ECX from DAT_01be1090) and clears flag 4; returns otherwise. |
| `00a89b30` | VehicleDemolitionDerbyGroupHandlerClose | confirmed | If flag 0x10 at this+0xc is set, calls 009797a0 with VehicleDemolitionDerbyGroupHandler:Close and clears flag 0x10; returns otherwise. |
| `00c0b080` | RegisterGroupScreenModule | confirmed | Registration wrapper: pushes nup=0, the 5-entry table 0x0182a988 and the 'GroupScreen' module-name pointer, calls FUN_00e39e40 and returns 0. |
| `00a7da40` | RefreshMemberRowForGuid | likely | Initializes the row index to 0. If param_2/param_3 differ from the owner fields +0x3a0/+0x3a4, it scans datasource rows from index 1, requiring a row value and… |
| `00c0b020` | Group_RemovePlayerFromGroup | speculative | Lua 'RemovePlayerFromGroup' handler: with one numeric argument it calls FUN_00a7e1c0(player id) on the group manager; returns 0. |
| `00c0b300` | — | unknown | Lua 'SetChangingJob' handler: requires exactly one argument (FUN_00e3a580(L,-1)==1 boolean); when the group client object at +0x6ec exists it reads the argumen… |

### `ui.quickchat`

**ClientQuickChatManager** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b96050` | ClientQuickChatManager_RefreshPhraseData | confirmed | Virtual method of the quick-chat manager: calls the virtual function at slot 0x38 of the object referenced by this+0xff0, then rebuilds the phrase row labels w… |
| `00b96db0` | ClientQuickChatManager_Construct | confirmed | Constructor of the quick-chat manager: runs the QuickChatManager base constructor FUN_0107c580, stores ClientQuickChatManager::vftable, zeroes the data source … |
| `00b96f60` | ClientQuickChatManager_DeletingDestruct | confirmed | Scalar deleting destructor: calls the manager destructor 00b965d0 and then _free(this) when bit 0 of param_2 is set. |
| `00b95140` | — | unknown | Walks the singly linked row list at this+8 (next pointer at +0x38): for every row it asks the client object at this+0xfec for the row display text (virtual cal… |
| `00b965d0` | — | unknown | Destructor of the quick-chat manager: reinstalls ClientQuickChatManager::vftable, destroys the embedded text-index object through 00b96540, releases the three … |

**ClientQuickChatManager::InternalDs** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b933f0` | ClientQuickChatManager::InternalDs::GetField | confirmed | Checks row index against the source count, fetches the row and indexed entry, then switches over field IDs to return or format values; invalid IDs emit the err… |
| `00b96070` | ClientQuickChatManager_InternalDs_Destruct | confirmed | Destructor body of the quick-chat data source: stores the InternalDs vtable into the three interface slots at +0, +0x30 and +0x34 and calls the base GuiUserDat… |
| `00b96c80` | ClientQuickChatManager_InternalDs_DeletingDestruct | confirmed | Scalar deleting destructor: calls the data source destructor 00b96070 and then _free(this) when bit 0 of param_2 is set. |
| `00b96c00` | ClientQuickChatManager_InternalDs_Construct | likely | Constructor of the quick-chat data source: runs the base IDataAccessTable/GuiUserDataTable constructor FUN_00922890 with param_2, then stores the InternalDs vt… |

**ClientQuickChatManager::InternalEmoteItemCategoriesDs** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b935c0` | ClientQuickChatManager::InternalEmoteItemCategoriesDs::GetF… | confirmed | Finds the category row by key, then formats its numeric field or resolves the string identifier through the client string table; invalid field IDs emit the err… |
| `00b960f0` | ClientQuickChatManager_InternalEmoteItemCategoriesDs_Destru… | confirmed | Destructor body of the emote item categories data source: stores the InternalEmoteItemCategoriesDs vtable into the three interface slots at +0, +0x30 and +0x34… |
| `00b96ca0` | ClientQuickChatManager_InternalEmoteItemCategoriesDs_Constr… | confirmed | Constructor of the emote item categories data source: runs the base IDataAccessTable/GuiUserDataTable constructor FUN_00922890 with param_2, stores the Interna… |
| `00b96d20` | ClientQuickChatManager_InternalEmoteItemCategoriesDs_Deleti… | confirmed | Scalar deleting destructor: calls the data source destructor 00b960f0 and then _free(this) when bit 0 of param_2 is set. |

**QuickChatAutoCompleteDataSource** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cdf760` | QuickChatAutoCompleteDataSource::Constructor | confirmed | Initializes the BaseClient.QuickChat.AutoComplete path, stores its context, installs three quick-chat data-source vtables, initializes the list, and initialize… |
| `00cdf8c0` | QuickChatAutoCompleteDataSource::ScalarDeletingDestructor | confirmed | Calls the autocomplete data-source destructor body, frees this when flag bit 0 is set, and otherwise returns this. |

**(no class)** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b976a0` | QuickChat_SendWhisperAndEcho 🌐 | confirmed | Whisper path of the quick-chat manager: looks up the row param_2, builds a QuickChatSendTellPacket temporary (00b90970) with the string param_3 copied into a S… |
| `00b90c70` | ParseCaretSeparatedResource | likely | Loads supplied text, skips comment or short records, splits entries at caret delimiters, allocates entry nodes, and inserts them into the object's bucketed lin… |
| `00b95280` | Thunk_ThisPlusFF8_00b93f70 | likely | Adjustor thunk of 11 bytes: adds 0xff8 to this and tail-jumps to FUN_00b93f70, the string routine that references SoeUtil::IStringFixed<char,16>. |
| `00b96170` | QuickChat_BuildEmoteFavoritesRows | likely | Walks the row list at this+8 (next pointer at +0x38): for each row whose byte at +0x24 is set it appends the row text into a local IString through FUN_00402ff0… |
| `00b97480` | QuickChatSendChannel4 🌐 | likely | Same data source column setter as 00b97380 but with the channel number 4 and only the two value slots row+8 and row+0xc checked; sends the chat-to-channel pack… |
| `00cdf820` | ReleaseAutocompleteStringAndLists | likely | Restores an embedded IString vtable, decrements and releases its owned buffer when required, destroys the capacity-256 list, and destroys the embedded GUI tabl… |
| `00b97380` | QuickChatSendChannel0 🌐 | speculative | Data source column setter: looks the row param_2 up in the bucket chain at this+0x14 (key at row+0x30, next at row+0x2c) and, when the row exists and has any o… |
| `00b97580` | — 🌐 | unknown | Same data source column setter as 00b97380 but with the channel number 9 and two extra dwords (param_3, param_4) stored in the packet; sends the chat-to-channe… |
| `00b978b0` | — 🌐 | unknown | Channel toggle setter of the chat window: looks the row param_2 up in the bucket chain at this+0x14 (key at row+0x30, next at row+0x2c) and maps the requested … |

## Source shards

[[S0233 008ec2a0-008f0cc0|S0233]] (3), [[S0237 008ff3d0-00904e70|S0237]] (4), [[S0241 009162f0-0091aa10|S0241]] (2), [[S0245 0092b170-0092f380|S0245]] (3), [[S0247 00934740-00939200|S0247]] (1), [[S0248 009392c0-0093d740|S0248]] (1), [[S0287 00a0c3a0-00a13140|S0287]] (1), [[S0288 00a13190-00a18a10|S0288]] (8), [[S0307 00a7b930-00a81420|S0307]] (26), [[S0309 00a87ba0-00a8cdc0|S0309]] (6), [[S0316 00aae6e0-00ab2c20|S0316]] (6), [[S0319 00abcb90-00ac2280|S0319]] (12), [[S0324 00ad8880-00adee10|S0324]] (2), [[S0325 00adee60-00ae4e70|S0325]] (21), [[S0335 00b109b0-00b150b0|S0335]] (12), [[S0336 00b150e0-00b1b340|S0336]] (30), [[S0337 00b1b5a0-00b21f20|S0337]] (23), [[S0345 00b4b3e0-00b50190|S0345]] (2), [[S0358 00b8a640-00b906a0|S0358]] (4), [[S0359 00b90750-00b94a30|S0359]] (3), [[S0360 00b94bf0-00b9a300|S0360]] (18), [[S0366 00bb11b0-00bb7710|S0366]] (39), [[S0367 00bb7790-00bbdbc0|S0367]] (14), [[S0371|S0371]] (7), [[S0376 00be5b30-00bea020|S0376]] (2), [[S0383 00c05780-00c0db00|S0383]] (3), [[S0384 00c0dcd0-00c171d0|S0384]] (24), [[S0386 00c1f190-00c26aa0|S0386]] (2), [[S0404 00c84f00-00c89db0|S0404]] (9), [[S0412 00cb0bf0-00cb6b80|S0412]] (8), [[S0420 00cdb370-00ce0140|S0420]] (3), [[S0437 00d297c0-00d2f870|S0437]] (5), [[S0577|S0577]] (1), [[S0579|S0579]] (5), [[S0580 0107d0c0-010829c0|S0580]] (13), [[S0582|S0582]] (30), [[S0583|S0583]] (79), [[S0667 0127d1b0-01281de0|S0667]] (1), [[S0668|S0668]] (3), [[S0670 0128c350-01290f70|S0670]] (29), [[S0671 01291250-01295650|S0671]] (8), [[S0681|S0681]] (1), [[S0724|S0724]] (22), [[S0726|S0726]] (41), [[S0727 013be8e0-013c3b90|S0727]] (36), [[S0751|S0751]] (99), [[S0772|S0772]] (30), [[S0778|S0778]] (217), [[S0779|S0779]] (59), [[S0785|S0785]] (15), [[S0794|S0794]] (76), [[S0795|S0795]] (73)
