---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Trade

Part of [[Build Guides]] › Gameplay. Related: [[Items and Inventory]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`, `unknown`) are the native-map labels for a function. Source tags: *client* = read from the shipped client in a shard or in this pass (function pseudocode and bytes, the Lua tables decoded from the binary, the decompiled client Lua `ScriptsBase.lua`, the shipped `Resources` files, the client locale); *video* = a reviewed frame record (frame key in brackets); *wiki* = the fan wiki. Precedence is client, then video, then wiki. *(R)* marks our own reconstructed server policy, not original behaviour. "Bytes" means I read the instruction bytes of `FreeRealms.exe` directly because the function is not in the map. OSFR-held data (packet captures and server JSON) is not used. No trade datagram has been decoded for this guide, so every statement about what the server sent or decided is a hypothesis built from what the client requires. The client locale is keyed by hashed ids that do not match the small string ids in `CodeStringMappings.txt`, so a script key is paired with a locale line by meaning only, and each pairing says so. No player names, chat or player-chosen names are recorded. Related: [[Items and Inventory]], [[Trading Card Game]], [[Quests and Objectives]] (the Game Action Queue), [[Social (Friends, Guilds, Chat)]], [[Pets]], [[Coin Store and Purchases]].

### Overview

Two features share the word "trade". They use different code and different wire formats, and the Rust server needs both.

1. **Player trade.** Two players in the world swap items and coins through the "Player Trade" window. The traffic is one packet family (client id `0x3b`, 59) with 18 sub-opcodes, handled by one dispatcher. About 130 of the mapped functions.
2. **Card trade.** Inside the Trading Card Game client, players trade cards directly or through a Trade Lobby of posted offers. This is a separate stack of `TradeCommand_*` network commands (25 rows in the command catalogue) on the card service's own direct TCP connection. About 360 of the 455 `net.trade` functions on this page belong to it. The card game itself is in [[Trading Card Game]].

A third thing is only a name clash: the **Trade chat channel** (`WorldTrade`, channel 6, `/trade` or `/2`) is chat, not trading (client `00b0fcb0`, `00b159d0`; wiki "Chatting"). It is not covered here.

**Player trade, what the player sees.**

- **Starting.** Clicking another player opens a radial menu. Frames show "Invite to Group, Trade, Duel" and, in another, "Add Friend, Duel, Invite to Group, Trade, Ignore, Inspect". (video `IYzJU-RjbPE/00126`, `fjZluIfbUIo/00024`) The client's radial menu is built from a server-sent command (command sub-opcode 9, `HandleCommandCase9` `00aa2360` likely, menu builder `00a38570` unknown label). That the player menu uses this path is likely, not proven; if it does, the server decides which entries exist. How a click on Trade becomes a request is not traced (see Open questions).
- **The invite.** The other player gets a toast in the Game Action Queue (type `TradeInvite`, Lua `Gaq:AddGameAction`, icon image 737, `AddTradeInvite` in the client Lua) with accept and decline. (client Lua; builder `00a0d320`) A video frame shows a "Do you want to accept?" dialog with accept and cancel buttons for a trade request. (video `eqV_FzElXSE/00378`) The locale has "Trade with [*player*]?", which fits the script key `GAQTradeInvite` by wording. Options has a "Block Trade Request" setting (locale, paired with the options script key below by wording); the options script key `setAutoRefuseTrade` calls `Ui.SetAutoRefuse(4, ...)`, and category 4 is the config key `TradeRequest` (bytes of the name table at `01b3aa74`; check `00917b60`, confirmed). The locale also has "You already have a trade request pending. Cancel it before asking for another trade."
- **The window.** Titled "Player Trade" (video `eqV_FzElXSE/00384`, `nmT76ti9kyo/00288`; locale). Three views: your inventory (the same category tree as the Stuff window, housing fixtures hidden), "My Offers" and the partner's offers (the title is the partner name plus a locale suffix, key `TradeUIOtherOffers`). A coin box shows your coins and the partner's coin offer, and a portrait is loaded from the partner's guid. The bottom bar has a red X (cancel) and a green check (the lock step). (client Lua `TradeHandler`, `ScriptsBase.lua:25282` to `25726`) A context entry that cancels the trade is pushed while the window is open, and opening the window while in a mini-game cancels at once. (client Lua)
- **Restrictions on rows.** The inventory view carries an "Available Count" column and a "No Trade Message" column (`00cf5d60`, confirmed). The message is one of three, chosen by `00cf76e0` (confirmed): no-trade, currently equipped, equipped on a pet. The locale has "This item can't be traded", "This item can't be traded because it is currently equipped." and "This item can't be traded because it is equipped on a pet." (paired by wording with `ItemNoTrade`, `ItemEquipped`, `ItemEquippedOnPet`). Item detail panels in other frames also read "Cannot Trade". (video `4pioItLqlU8/00156`, `xC7asgOhJKc/00015`)
- **Lock, confirm, accept.** Pressing the green check blocks your side, shows a confirmation popup and sends a lock. The popup reads "WARNING! Please review your trade before accepting. All trades are final. Waiting for other player." with ACCEPT and CANCEL. (video `nmT76ti9kyo/00288`; client Lua `TradeConfirmation`) When the server sends the confirm message the popup starts a timer, and the Accept button is enabled when it fires: `TradeAcceptTimer` is 10 seconds of in-game time. (client `ClientTimerDefinitions.txt`, Lua)
- **Ending.** Messages in the client: trade completed, canceled, partner left, an offer invalid, an offer over the partner's max stack, entered an instanced zone. The video shows "Trade completed" as a system line over the player. (video `eqV_FzElXSE/00442`, `nmT76ti9kyo/00312`) A removed item shows "Item [name] has been removed from the trade offer." (video `nmT76ti9kyo/00296`; locale)
- **Hooks elsewhere.** The locale has "Complete 5 player to player trades", "Successfully trade with a player." and "Player Trade". Which quest or achievement owns them is not known. A first-time event `FtesTrading` fires when the window first opens (client Lua `Ui.TriggerFirstTimeEvent`, `FirstTimeEvents.txt`); the locale line explaining the trade window and its checkmark is probably its text.
- **Gear that cannot be traded.** The wiki marks many items "Cannot Sell. Cannot Trade." (Station Cash flair shards, starter weapons, promo items) and lists retired Coin Shop gear (retired 2009-12-14) as "only available through trade". (wiki, item pages)

**Card trade, what the player sees.** A Trade Lobby window ("Trade Lobby (1294 Trades)" in one frame) with Trades, Players and Friends tabs, a Create Trade button and filters (Rarity, "Trades I Can Do", "Trades from Friends", "Trades with Virtual Rewards", "Trades without Virtual Rewards", "Trades I created"). A help tip says posted trades are listed on the Trades tab and a direct trade starts from the Players or Friends tabs. A posted trade window has "My Cards" and "Their Cards" tabs and an "I Will Get" panel; a posted trade row has an Examine and Cancel menu. The collection window has a "Show Trade Limits" tab. (video `UFsVcZ3X5iA/00046`, `UFsVcZ3X5iA/00048`, `UFsVcZ3X5iA/00028`, `jtn0uNxCkDQ/00068`, `jtn0uNxCkDQ/00078`, `jtn0uNxCkDQ/00106`) The membership panel lists "Create and accept trades" as a member feature and the locale says Members can trade cards. (video `ip2vKMeCU6U/00078`; locale) Per the wiki, boosters, single cards and loot cards can be traded through the lobby, but cards from the monthly membership pack cannot. (wiki "Trading Card Game")

### How the client implements it

#### Main classes and objects

| Class or object | Role | Evidence |
|---|---|---|
| `TradeDispatchPacket` (`00c6df10`, 2387 bytes) | The only receive switch for player trade. The client message dispatcher `009471a0` sends message id `0x3b` to it through the adjustor thunk `00a10940` (`add ecx,0x1d008`), and only when the local player object (client `+0x3e0`) exists. It reads the header with `00c6beb0`, builds the packet class for the sub-opcode, runs the matching reader, checks the session id and raises script events. | confirmed (`00c6df10`), likely (`00a10940`); switch read in this pass |
| Trade handler object (ctor `00c6c470`, dtor `00c6be30`; label unknown / likely) | The session state, embedded in the game-state object at `+0x1d008`. `+0` owner, `+4` session id (`0xFFFFFFFF` = none), `+8` update counter (see Data and formats), `+0x10/+0x14` partner guid (default pair `DAT_01bee968/6c`), `+0x18` partner name (a string), `+0xa0` pointer to the tracker below. Reached from Lua through the accessor `008a6650`. | ctor read in this pass (bytes and pseudocode) |
| `TradeSessionTracker` (ctor `00cf68d0` confirmed, dtor `00cf65b0` likely) | A 0x18-byte holder of five references: `+4` the GUI value `BaseClient.Trade.PartnerName`, `+8` `BaseClient.Trade.PartnerCoinCount`, `+0xc` the partner item source, `+0x10` your item source, `+0x14` the inventory source. The two GUI values are 0x54 bytes each (state bits at `+0x4c`, value or pointer at `+0x50`). | confirmed ctor; slot use read from pseudocode and bytes in this pass |
| `TradeInventoryDataSource` (ctor `00cf6ed0`, `Initialize` `00cf7860`, `GetCell` `00cf76e0`) | `BaseClient.Trade.Inventory`: the left view. Holds the selected category at `+0x2e4` (setter `00cf6260`, likely) and supplies the Available Count and No Trade Message columns. `Initialize` builds all three item sources the first time a session starts. | confirmed |
| `TradeItemDataSource` (ctor `00cf70d0`, `GetCell` `00cf7200`) | `BaseClient.Trade.PartnerItems` (built with flag 0) and `BaseClient.Trade.MyItems` (flag 1), a byte at `+0x2e8` records the flag. Rows are keyed by the item guid. | `GetCell` confirmed; ctor speculative; flags read from `00cf7860` |
| Lua module `Trade` (registrar `00c05ce0`, table `01829c48`, 11 entries) | The script surface. Table decoded from the binary in this pass; the handlers for the first six entries are not in the map and were read from bytes: `RejectInvite` `00c05d00` (calls `00c6db40`), `Accept` `00c05d60` (`00c6dcf0`), `Cancel` `00c05d80` (`00c6dd70`), `Lock` `00c05d20` (`00c6dc70` with 1), `CancelLock` `00c05d40` (`00c6dc70` with 0), `OfferItems` `00c05da0` (needs two numeric arguments, calls `00c6ddf0`), `OfferCoins` `00c05e20` (`00c6de90`), `GetPartnerCoinOffer` `00c05e60` (`00c6b5f0`), `GetPartnerName` `00c05f00`, `GetPartnerGuid` `00c06020`, `SelectInventoryCategory` `00c05e90`. | `00c05ce0` confirmed; the other handlers per their own labels, bytes where noted |
| Client Lua `TradeHandler` and `TradeConfirmation` | The two windows. `TradeHandler` receives the script events (`Init`, `Destroy`, `OtherAccepted`, `OnUpdatePlayer`, `OnUpdateOther`, `OnUpdateOtherCoin`) and pushes the data-source rows into the movie. | client Lua |
| `GaqItemTradeInvite` (builder `00a0d320`, thunk `00a0d540`, responders `00a09120` / `00a09130`) | The toast. Decline and accept both call `00c6dbe0` with 0 and 1. | `00a0d540` likely; responders likely |
| 18 packet classes on `BaseTradePacket` (family ctor `00c6a7b0`) | The ctor stores a dword `0x3b` at `+4` and the sub-opcode at `+8`; `00c6cc30` writes the low two bytes of `+4` and the one-byte sub-opcode, `00c6beb0` reads them. | constructors confirmed or likely; `00c6a7b0` unknown label |
| Client send path (`00c6db40` to `00c6de90`, gateway paths `00c6d420` to `00c6d970`) | Each request is built, serialised into a stack writer and wrapped in a gateway packet. Invite reply, lock, accept and cancel use `ClientGateway::PacketTunneledClientWorldPacket` (ctor `0089ae70`, gateway opcode 6); item offer and coin offer use `PacketTunneledClientPacket` (`0089adc0`, opcode 5). Nothing is sent unless the client transport at `+0x280` exists. | senders likely; gateway constructors confirmed |
| Card trade: `UI::TradeLobbyScreen` (`012f7b50`), `UI::TradeScreen` (`012f7c60`), `TradeCommand_*` classes, `PostedTrade` (`013d1b50`), `TradeService` singleton (`DAT_01cbd690`, ctor `012c0270`) | The card client's screens, command classes and the local list of posted trades. | confirmed constructors |

#### Starting a trade

1. **The request.** The family has no class for a "please trade with this player" request (sub-opcode 1 has no class and nothing in the client builds one). The request therefore leaves the client by a path outside this family; see Open questions.
2. **Invite received (sub 2).** `00c6df10` acts only when the handler's session id is `0xFFFFFFFF`; if the client is already pending or trading, it drops the invite silently, with no refusal. The reader `00c6cab0` gives a name record, two dwords and the session id. The dispatcher stores the session id, copies the name into `+0x18`, then calls `00a12720` with the two dwords. That is a forwarder to `00c7e260` and `00cf8700`, a 64-bucket hash lookup keyed by the XOR of the two dwords, sitting in the ignore-list data-source code; read together with the two dwords being the sender's guid (`00c6cb40` stores a guid pair in the same place), it is most likely "the sender is on the ignore list" (speculative). If true the client answers with an invite reply of 0 at once. If false it builds the toast with `00a0d540`, and that builder consults the auto-refuse option 4 through `00917b60` before queuing (`00a0d320`, unknown label; `00917b60`, confirmed).
3. **Answer.** `00a09120` / `00a09130` call `00c6dbe0` with 0 / 1: it sends an invite reply with the session id and the accepted byte and, for 0, clears the session id locally. `Trade.RejectInvite` calls `00c6db40`, which does the same refusal, but only while no session has started (counter 0). The mini-game lobby script also calls `Trade.RejectInvite` when a game is starting or launching (client Lua, `ScriptsBase.lua:39694`, `39704`).
4. **Withdrawing a pending invite.** A status message of 2, 7 or 8 while the counter is still 0 (no session yet) removes the toast with `00a0eb90` (likely) and clears the session id.
5. **Session start (sub 4).** `00c6cb40` gives the partner guid pair, a name record and the session id. The dispatcher stores them, sets the counter to 1, asks `00bd4810` (likely, a headshot refresh) for the partner's picture, runs `00cf7860` to create the three item sources and raises `TradeHandler:Init`. Nothing checks that the id matches a pending invite.

#### Inside a session

- **Your offers.** The inventory pane raises `ItemAdded` and the coin box raises `CoinOffer`. `Trade.OfferItems(item, delta)` runs `00c6ddf0`, which first calls `00cf6c90` (speculative, but read in full in this pass): it clamps a positive delta to what the inventory still has available, clamps a negative delta to what is already offered, updates the local offered count and returns the resulting total. The packet carries the item guid and that resulting total, not the delta. `Trade.OfferCoins(n)` sends the number as typed (`00c6de90`); the native code does no range check. The Lua also cancels its own "accepted" display after a coin change.
- **Server echoes.** Coin update (sub `0xb`) calls `00cf61e0`, which stores the partner's coins and refreshes the display only when the side byte is 0. Add item (sub `0xc`) calls `00cf79d0`, which creates a row in the chosen item source. Item count (sub `0xd`) calls `00cf69c0`: a positive quantity sets the row's count, zero or less removes the row. Each of the three also raises the counter to the packet's second dword if that is larger. The side byte (0, or non-zero) picks the source at tracker `+0xc` (partner, built with flag 0) or `+0x10` (yours, flag 1); that pairing is by creation order and the coin check, not by a label.
- **Lock.** The green check shows the confirmation popup and sends `Trade.Lock()`: `00c6dc70` builds a lock packet from the session id, the counter and a locked byte. An incoming lock update (sub `0x11`) with a session match raises `TradeHandler:OtherAccepted` when the byte is set and `TradeConfirmation:Hide` when clear. `TradeConfirmation:Hide` hides the popup, unblocks the window and calls `Trade.CancelLock()`, which sends a lock with 0. The Lua guards against looping (it returns when the popup is not visible).
- **Confirm.** A confirm message (sub `0x12`) raises `TradeConfirmation:StartTradeTimer`: the Lua starts `TradeAcceptTimer` and the movie shows its timer message. When the timer fires the script enables the Accept button. `Trade.Accept()` runs `00c6dcf0`: a packet with the session id and the counter.
- **Accept update (sub `0xe`).** The dispatcher reads and discards it. It has no visible effect on this client.
- **Item removed (sub `0x13`).** After the session match the dispatcher looks up the item (a virtual call on the client object), formats the `TradeItemRemoved` text and shows it in red `0xFF0000` as a system message. Of the three dwords, only the session id is certain; the other two are not traced.

#### Ending a session

- **Status message (sub `0xf`).** One byte. `00c6df10` raises a script event by value and shows the matching text through `00a94ab0` (`FormatPlayerMessage`, likely) using the partner name stored at `+0x18`. It does not close the window, clear the session id or reset the counter, except for the pending-invite case above.

| Value | Script key | Locale line paired by wording |
|---|---|---|
| 1 | `TradeCanceled` | one of "Your trade with [*player*] has been canceled.", "[*player*] canceled the trade", "Your trade has been canceled." (not resolved which) |
| 2 | `TradeCanceledPartnerDisconnected` | "Trade canceled - [*player*] left the game" |
| 3 | `TradeCanceledPartnerOfferInvalid` | "Trade canceled - your partner's offer was invalid" |
| 4 | `TradeCanceledYourOfferInvalid` | "Trade canceled - your offer was invalid" |
| 6 | `TradeCompleted` | "Trade completed" (video `eqV_FzElXSE/00442`) |
| 7 | `TradeCanceledYouEnteredInstancedZone` | no English line found |
| 8 | `TradeCanceledPartnerEnteredInstancedZone` | no English line found |
| 9 | `TradeCanceledYourOfferExceedsPartnersMaxStack` | "Trade canceled - Your offer had an item that would exceed its max stack in your partner's inventory." |
| 10 | `TradeCanceledPartnersOfferExceedsYourMaxStack` | "Trade canceled - Partner's offer had an item that would exceed its max stack in your inventory." |

Values 0, 5 and above 10 do nothing. The mapping of values to cases is read from the dispatcher's own arms (`00c6df10`, confirmed), not from a capture.

- **End session (sub 5).** The only thing that closes the window. With a matching session id the client clears the id, the counter and the partner guid, runs `00cf6be0` (reset partner coins to 0 and empty both item lists) and raises `TradeHandler:Destroy`, which hides the window and the confirmation popup. The packet class is receive-only; the client's own cancel (`Trade.Cancel` → `00c6dd70`) only sends a cancel request (sub 7) and does not clear anything locally. (bytes; the shard labelled `00c6dd70` as clearing state, the review rejected that)
- **Rejected (sub `0x10`).** Carries one name record. The client shows the `TradeRejected` text with that name.

#### Card trade in the client

The card client has no use of family `0x3b`. It builds `TradeCommand_*` objects (a `NetworkCommand` subclass), writes them with `Write` (vtable slot 10), and frames them as `[htonl(length+12)][three htonl routing dwords][body]` through `SendCommandToConnection` (`012d0810`, likely) and `FrameCommandToOutBuf` (`012f0a00`, likely). The base `TradeCommand` routing is `1000000002, 0, TradeSessionID` (`014426a0`, confirmed); `IssueTradeRequest` overrides it to `1000000000, 0, 0` (`014a8560`, speculative). The card service connection (a direct TCP link, not tunneled through the game packet), its gateway step and the class header are described in [[Trading Card Game]]. Handlers do not send replies: each queues a UI event (tag in the table below) and, for most, a system text. `TradeService` (`012c02f0`) keeps the local list of posted trades keyed by id; cancel and accept only send, and the entry is dropped when the server's cancel command arrives (`013d0670` → `012bf9d0`; `012bded0`, `012bdf50`, `012bf340` send, shard S0679).

### Data and formats

**Client resources (read by the client, not sent).** Read in this pass from the client baseline's `Resources` folder.

| File | Contents |
|---|---|
| `ClientTimerDefinitions.txt` | `TradeAcceptTimer^10^1^0^0^TradeConfirmation.EnableAcceptButton^`: 10 seconds, in-game time, no auto start, no repeat, then the Accept button is enabled. |
| `CodeStringMappings.txt` | Script keys and their string ids: `GAQTradeInvite` 2605, `TradeBodyText` 504, `TradeCanceled` 2398, `TradeCanceledPartnerDisconnected` 2395, `TradeCanceledPartnerEnteredInstancedZone` 3097, `TradeCanceledPartnerOfferInvalid` 2397, `TradeCanceledPartnersOfferExceedsYourMaxStack` 1053, `TradeCanceledYouEnteredInstancedZone` 3096, `TradeCanceledYourOfferExceedsPartnersMaxStack` 1052, `TradeCanceledYourOfferInvalid` 2396, `TradeCompleted` 2394, `TradeItemRemoved` 439226, `TradeRejected` 2399, `TradeScamAccept` 514, `TradeScamCancel` 516, `TradeScamReview` 511, `TradeScamWaiting` 509, `TradeScamWarning` 506, `TradeUIMyOffers` 408182, `TradeUIOtherOffers` 408187, `TradeUIWindowTitle` 408186, `ItemNoTrade` 120, `ItemEquipped` 121, `ItemEquippedOnPet` 3601, `FtesTrading` 384899, `2Trade` 401342 (chat tab), `Tcg_contextMenu.trade` 3299, and nine `Tcg_error.*trade*` keys. The `TradeScam*` keys are the likely sources of the confirmation popup lines in the video; the pairing is by wording. |
| `FirstTimeEvents.txt` | Event 26 `FtesTrading`. |
| `CharacterCreate/ItemDefinitions.txt` | The item schema has a `NO_TRADE` column (15th) and `MAX_STACK_SIZE` (11th). The shipped file is only the 302-row character-creator subset: `NO_TRADE` is 1 on 246 rows and 0 on 56; `MAX_STACK_SIZE` is 1 on 221 rows and -1 on 81 (meaning of -1 not established). The full catalogue arrives from the server (see [[Items and Inventory]]). |
| Lua data-source columns (`ScriptsBase.lua:3389` to `3450`, `25293` to `25376`) | `BaseClient.Trade.MyItems` and `PartnerItems` expose 29 columns (item guid, icon, name, description, quantity, members only, tint, equippable state, slot, new, activatable, rarity, equipped, model, item type, tint override, activate enabled, power rating, gender usage, stat id and value, minimum rank, text colour, item id, in storage, in inventory, equipped on any profile, price, no sale). The inventory view adds `tradableQuantity` and `noTrade`. |

**Handler state (the object at game-state `+0x1d008`).**

| Offset | Meaning |
|---|---|
| `+4` | Session id, `0xFFFFFFFF` when none. The server must never use that value. |
| `+8` | Update counter: 0 while only an invite is pending, set to 1 when a session starts, then raised to the second dword of every coin, add-item and item-count update when that is larger. The client sends this value in every lock and accept. The client only shows that it is a running maximum; no name for it is taken from outside the client. |
| `+0x10 / +0x14` | Partner guid, two dwords. Lua formats it with `%llu` for the portrait (`00c06020`, likely). |
| `+0x18` | Partner name (a string), set at invite and at session start. |
| `+0xa0` | Pointer to the `TradeSessionTracker`. |

**Name record** (reader `008e7290`, 33 callers, label speculative; layout read from its pseudocode in this pass). Used for the player name in the invite, session start and rejected packets.

| Order | Width | Client field |
|---|---|---|
| 1 to 3 | 3 x u32 | record `+0x7c`, `+0x80`, `+0x84` (meaning unknown) |
| 4 | i32 length, then bytes | string at record `+0` |
| 5 | i32 length, then bytes | string at record `+0x24` |

The shard summary calls this "a string"; the reader takes three dwords and two strings. `00726400` runs afterwards and builds a display string from the record. What the two strings and three dwords are (first and last name parts is a guess) is not established.

**Item record** (reader `008fa4b0` for the add-item packet, label speculative; the field names come from the `ItemInstance` layout in [[Items and Inventory]], confirmed there by the client's own format string). Widths and order are read from the pseudocode.

| Order | Width | `ItemInstance` field |
|---|---|---|
| 1, 2 | u32, u32 | Definition `+4`, Tint override `+8` |
| 3 | u32 | Guid `+0x10` (also the key the trade rows use, `00cf79d0`) |
| 4, 5 | u32, u32 | Count `+0x14`, ConsumedCount `+0x18` |
| 6 | i32 | Last-cast time as an age: stored as now minus the value; `0x7fffffff` means none (`008cb5a0`) |
| 7 | u8 | Rental flag |
| 8 | 16 bytes if the flag is non-zero | An 8-byte time, then two u32 (rental block `+0x28`; `008cd5e0`) |
| 9 | u32 | Stored count `+0x1c` |

### Network

**Envelope.** Player trade is message id `0x3b` (59). The body starts with a u16 family (the low two bytes of the dword the family ctor stored) and a u8 sub-opcode (`00c6beb0`, `00c6cc30`, likely), then the sub-opcode's fields, little-endian, strings as an i32 length plus bytes. The receive wrappers (`00c6c730`, `00c6c7a0`, `00c6c810`, `00c6c880`, `00c6c8f0`, `00c6c960`, `00c6c9d0`, `00c6ca40`, `00c6ccd0`, `00c6cd40`, `00c6cdb0`, `00c6d350`) accept a body only if the overflow flag stayed clear and the cursor reached the end (unless a strictness argument is set). Client requests are wrapped as described above. The outer SOE layer and the gateway opcodes are outside this guide. Field names below are inferred from use; widths and order are client-read.

**Server to client (handled by `00c6df10`).**

| Sub | Class (reader) | Body after the 3-byte header | Client action |
|---|---|---|---|
| 2 | `TradeInvitePacket` (`00c6cab0`) | name record; u32, u32 (sender guid, likely); u32 session id | Stores the id and name; ignore check; toast. Dropped if a session or invite is open. |
| 4 | `TradeStartSessionPacket` (`00c6cb40`) | u32, u32 partner guid; name record; u32 session id | Opens the window (`TradeHandler:Init`). |
| 5 | `TradeEndSessionPacket` (`00c6c530`) | u32 session id | Closes the window (`TradeHandler:Destroy`). |
| `0xb` (11) | `TradeUpdateCoinCountPacket` (`00c6bf50`) | u32 session; u32 counter; u8 side; u32 coins | Partner coin display when side is 0. |
| `0xc` (12) | `TradeUpdateAddItemPacket` (`00c6ce30`) | u32 session; u32 counter; u8 side; item record | Adds a row. |
| `0xd` (13) | `TradeUpdateItemCountPacket` (`00c6c000`) | u32 session; u32 counter; u8 side; u32 item guid; u32 quantity | Sets or removes a row. |
| `0xe` (14) | `TradeUpdateAcceptPacket` (`00c6c590`) | u32 | Read, then nothing. |
| `0xf` (15) | `TradeMessagePacket` (`00c6c630`) | u8 status (sign-extended) | The table above. |
| `0x10` (16) | `TradeRejectedPacket` (`00c6cdb0`) | name record | `TradeRejected` text. |
| `0x11` (17) | `TradeUpdateLockPacket` (`00c6c110`) | u32 session; u8 locked | `OtherAccepted` or `TradeConfirmation:Hide`. |
| `0x12` (18) | `TradeConfirmPacket` (`00c6c5f0`) | u32 session | Starts the confirm timer. |
| `0x13` (19) | `TradeItemRemovedPacket` (`00c6c180`) | u32 session; u32; u32 | Red "item removed" message. |

Only the packets that carry a session id are compared against the handler's id (`0xb` to `0xd`, `0x11` to `0x13`, and 5). Sub 4 and sub `0xf` are not compared.

**Client to server (senders).**

| Sub | Class (writer) | Body | Sender |
|---|---|---|---|
| 3 | `TradeInviteReplyPacket` (`00c6cee0`) | u32 session; u8 accepted | `00c6dbe0`, `00c6db40` |
| 6 | `TradeAcceptPacket` (`00c6d060`) | u32 session; u32 counter | `Trade.Accept` → `00c6dcf0` |
| 7 | `TradeCancelPacket` (`00c6d100`) | u32 session | `Trade.Cancel` → `00c6dd70` |
| 8 | `TradeLockPacket` (`00c6cf80`) | u32 session; u32 counter; u8 locked | `Trade.Lock` / `CancelLock` → `00c6dc70` |
| 9 | `TradeOfferItemPacket` (`00c6d160`) | u32 session; u32 item guid; u32 resulting total | `Trade.OfferItems` → `00c6ddf0` |
| 10 | `TradeCoinCountPacket` (`00c6d240`) | u32 session; u32 coins | `Trade.OfferCoins` → `00c6de90` |

**Card trade commands.** Body fields come from the `Read` and `Write` labels in the command catalogue; every class also carries the `TradeCommand` base (`01442780`, `014427e0`, confirmed): `AccountID` (int at `+4`) and `TradeSessionID` (int at `+8`). Ints are written by `SerialAppendInteger`. Version gates read a field only above a stated version. Handler tags are the UI event numbers the handler queues. Read/write confidence is `confirmed` unless stated.

| Id | Class (version) | Fields after the base | Handler effect (tag) |
|---|---|---|---|
| 147 (`0x93`) | `IssueTradeRequest` (2) | IssueTo AccountID; GroupID (read only above version 1) | Tag `0x21`; "You issue a trade request to %s." or "%s asks you to trade." (confirmed) |
| 148 (`0x94`) | `AcceptTradeRequest` (1) | IssuedFrom AccountID; TradeLobbyId | Tag `0x22`; "%s accepts your trade request." or "You accept %s's offer to trade." (confirmed) |
| 152 (`0x98`) | `CancelTradeRequest` (2) | IssuedFrom AccountID; OriginAccountID (above version 1) | Tag `0x23`; four texts (cancel by you / by them, request or offer); handler unknown label |
| 272 (`0x110`) | `StartTradeSync` (1) | IssuedFrom AccountID | Tag `0x23`; same accept texts, then "Please wait while collections are synchronized." (confirmed) |
| 149 (`0x95`) | `AddCardToTrade` (1) | card archetype; card instance; OriginAccountID | Tag `0x24`; "You add your %s to the trade." and three variants (handler unknown label) |
| 151 (`0x97`) | `RemoveCardFromTrade` (3) | archetype; instance; OriginAccountID (above 1); error code (above 2) | Tag `0x25`; five removal texts incl. "%s is removed from the trade because an error occurred." (confirmed) |
| 273 (`0x111`) | `LockTrade` (1) | lock flag | Tag `0x37`; "Please wait while collections are resynced." when set (handler unknown label) |
| 150 (`0x96`) | `AcceptTrade` (1) | "Trade Button" flag | Tag `0x26` (speculative) |
| 155 (`0x9b`) | `FinalAcceptTrade` (1) | "Trade Button" flag | Tag `0x29` (likely) |
| 156 (`0x9c`) | `SetupFinalTradeAccept` (2) | founder card list; trader card list (lists read only when a version gate holds) | Tag `0x2a`; walks both lists (unknown label) |
| 157 (`0x9d`) | `ClearAcceptState` (1) | none | Tag `0x2b` (speculative) |
| 154 (`0x9a`) | `ClearTrade` (1) | none | Tag `0x28` (unknown label) |
| 153 (`0x99`) | `EndTradeSession` (1) | none | Tag `0x27` (unknown label) |
| 270 (`0x10e`) | `ChangeTradeSession` (1) | OldTradeSessionID | Tag `0x35` (speculative) |
| 348 (`0x15c`) | `SetTradeAllowed` (1) | a dword-keyed flag tree | Tag `0x5a` per entry (likely) |
| 431 (`0x1af`) | `SetListenStatus` (1) | flag | handler not labelled |
| 401 (`0x191`) | `RequestPostedTrades` (1) | none | The client's own handler is a stub that returns 1 (shard S0679) |
| 402 (`0x192`) | `StartPostedTrades` (1) | count | Tag `0x8e` (likely) |
| 398 (`0x18e`) | `AddPostedTrade` (1) | presence int (1 = no object), then a `PostedTrade` | Tag `0x8b`; stores it in `TradeService` (speculative) |
| 403 (`0x193`) | `TransferPostedTrade` (2) | presence int, `PostedTrade`; DWORD vector (above version 1) | Tag `0x8f` (likely) |
| 400 (`0x190`) | `EndPostedTrades` (1) | none | Tag `0x8d` (unknown label) |
| 397 (`0x18d`) | `AcceptPostedTrade` (1) | posted trade id; response | Tag `0x8a` (speculative) |
| 399 (`0x18f`) | `CancelPostedTrade` (1) | posted trade id | Tag `0x8c`; erases the local entry (speculative) |
| 405 (`0x195`) | `PostedTradeError` (1) | error int; "from add" flag | Tag `0x90` (likely) |

`PostedTrade` (`013d1b50`, reader `013d2ac0` likely, writer `013d1520` likely) is read as: ints at `+0x08`, `+0x0c`; a string at `+0x2c`; ints at `+0x48`, `+0x64`; sub-objects at `+0x4c`, `+0x58`; strings at `+0x84`, `+0xa4`; int at `+0xa0`; then, above version 1, strings at `+0x10`, `+0x68`; above version 2, a sub-object at `+0xf8` (the item list). No field is named by the client. The video shows "Offered" and "Wants" lists, so the sub-objects are probably card lists (video; inference).

**Card trade error codes** (switch in `012e1020`, label unknown, with one rejected claim about other codes; these entries read from its pseudocode): `0x22` tradeLockCardFailure, `0x23` tradeItemLimit, `0x24` tradeBusyAdding, `0x25` tradeSelfTrading, `0x26` tradeOtherTrading, `0x2e` tradeNotAllowed, `0x5b` tradeNotAllowedAtAll, `0x5c` tradeOtherNotAllowedAtAll, `0x79` youAreTradeBanned. Whether these are the codes carried by `PostedTradeError` and `RemoveCardFromTrade` is not traced.

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour. Items 1 to 12 are player trade, 13 to 15 are card trade.

1. **Gate the traffic.** The client ignores family `0x3b` until its local player object exists, so send trade traffic after player spawn (client, `009471a0`).
2. **Offer the entry.** Add "Trade" to the server-built radial menu for other players, and decide when it is shown (not self, not already busy, trade allowed) (R). The click-to-request path is unresolved (Open questions); the server needs a request that names a target player.
3. **Invite.** Create a pending session with a fresh u32 id (never `0xFFFFFFFF`). Send sub 2 to the target with the sender's name record, the sender's guid in the two dwords and the id. Do not expect a refusal from a busy client: it drops sub 2 when it already has a session or a pending invite, so check busy state on the server first. Honour the target's "Block Trade Request" and ignore list on the server as well (R); the client refuses locally for the first and probably the second.
4. **Reply (sub 3).** On accepted: open the session. Send sub 4 to both players, each with the other's guid pair, name record and the session id; the client sets its counter to 1. On refused: drop the session and tell the inviter with sub `0x10` carrying the refuser's name record (the client shows `TradeRejected` with it). To withdraw a pending invite from the server (for example the inviter left), send status 2, 7 or 8 while no session has started; they remove the toast.
5. **Session state (R).** Per session: two players, each side's coin offer, item rows (guid and count), a counter, lock flags and accept flags. Start the counter at 1 and raise it on every offer change; echo it in updates. The client sends its highest seen counter in lock and accept, so a stale value means the player has not seen the latest offer.
6. **Offers (sub 9, 10).** Never trust the client's clamping. Check that the item belongs to the sender, is tradable (definition `NO_TRADE`, not equipped, not equipped on a pet), and that the total is no more than owned minus anything committed elsewhere; check coins against the balance and any cap (R). On success send each player the matching update with the side byte from that player's point of view (0 = partner, 1 = yours): sub `0xc` the first time an item appears (a full item record), sub `0xd` for later count changes (zero removes the row), sub `0xb` for the partner's coins. The client ignores a coin update whose side byte is 1, so there is no need to echo your own coins. Use sub `0x13` when the server must pull an item out (for example it became invalid).
7. **Offer change clears locks (R).** Reset both lock states and send sub `0x11` with locked 0 to the clients that had locked; the client hides its popup and sends its own unlock (harmless, but expect it).
8. **Lock (sub 8).** Record the sender's lock if the counter matches. Send sub `0x11` locked 1 to the other player (it shows the partner's check). When both are locked, send sub `0x12` to both: the client starts its 10-second timer and enables Accept afterwards. A server should also ignore an accept that arrives before 10 seconds have passed (R).
9. **Accept (sub 6).** Only valid when both are locked and the counter matches. When both have accepted, re-check everything: items still owned and tradable, enough bag space and stack room on both sides (statuses 9 and 10 when stacks would overflow, 3 or 4 when an offer is no longer valid), coins. Do the exchange as one atomic step. Then send status 6 (completed) and sub 5 to both.
10. **Cancel and loss (sub 7, disconnect, zone change).** Send the right status (1 canceled, 2 partner left, 7 or 8 for instanced zones) and then sub 5 to both players. Sub 5 is the only thing that closes the window; no status message does. Cancel the trade when either player enters an instanced zone (statuses 7 and 8). The client also cancels by itself if the window would open during a mini-game, and refuses a pending invite when a mini-game starts (client Lua); the full trigger set is R.
11. **Account gates.** The locale has refusal lines for an account not flagged to allow trading, for an account blocked from trades, for a player who is already trading, and for a player who is not allowed to trade. Which of them belong to player trade or to card trade is not traced. Implement the checks as server policy (R) and send them as system messages.
12. **Objectives and counters (R).** Fire a "trade completed" event for the quests or achievements that count player trades ("Complete 5 player to player trades", "Successfully trade with a player.").
13. **Card trade transport.** Serve the `TradeCommand_*` set on the card service connection ([[Trading Card Game]]) with the routing triple `1000000002, 0, TradeSessionID`, and send the current version of each class (client). The base fields are `AccountID` and `TradeSessionID`. A direct trade uses `IssueTradeRequest`, `AcceptTradeRequest` (or `CancelTradeRequest`), `StartTradeSync`, then `AddCardToTrade` and `RemoveCardFromTrade`, `LockTrade`, `AcceptTrade`, `SetupFinalTradeAccept`, `FinalAcceptTrade`, `ClearAcceptState` and `ClearTrade`, and ends with `EndTradeSession` (order inferred from the text each handler shows and from the names, R). `ChangeTradeSession` moves a trade to a new session id.
14. **Posted trades.** The client sends `RequestPostedTrades`, `AcceptPostedTrade` and `CancelPostedTrade`. Answer a request with `StartPostedTrades` (count), the posted trades (`AddPostedTrade` or `TransferPostedTrade`) and `EndPostedTrades`; confirm a cancel with `CancelPostedTrade`, because the client drops its local entry only when that command arrives (client, `013d0670`). Report failures with `PostedTradeError`. Who sends `AddPostedTrade` to create an offer is not traced; accept it from the client (R).
15. **Card rules.** Membership (R, from the locale and the membership panel), the non-tradable flags (`SynchronizationCommand_AddNonTradable` 301 and `RemoveNonTradable` 302 exist in the catalogue; their layouts are not read here), trade limits (the "Show Trade Limits" tab) and `SetTradeAllowed` (a per-key flag tree) are server policy with no client numbers. The membership pack rule comes from the wiki.

### Video and wiki evidence

**Video frames reviewed:** the frames that show a trade request dialog, the Player Trade window, its confirmation popup, trade system messages, the player radial menu, the card Trade Lobby and posted trade windows, and item details that show "Cannot Trade". Player names, partner names, chat and card names are not recorded.

- **Request and window (video).** A request dialog "Do you want to accept?" with accept and cancel (`eqV_FzElXSE/00378`). The Player Trade window with a "Stuff" tab and a "My Offers:" line (`eqV_FzElXSE/00384`). A frame with the request dialog and "Trade completed" together (`eqV_FzElXSE/00414`). The window with the review warning and "Waiting for other player." and a second frame in which the removed-item message appears (`nmT76ti9kyo/00288`, `nmT76ti9kyo/00296`). "Trade completed" appears as a system line over the world (`eqV_FzElXSE/00442`, `nmT76ti9kyo/00312`).
- **Radial menu (video).** "Invite to Group, Trade, Duel" (`IYzJU-RjbPE/00126`) and "Add Friend, Duel, Invite to Group, Trade, Ignore, Inspect" (`fjZluIfbUIo/00024`). The second set implies the entries change with the target (friend or not); the cause is not shown.
- **Item restriction (video, wiki).** Item detail panels show "Cannot Sell" and "Cannot Trade" on kart parts, gear and shards (`P2SusMJ2_tQ/00002`, `4pioItLqlU8/00156`, `xC7asgOhJKc/00015`, `Qg7_4br2H80/00448`). The wiki repeats this on item pages and distinguishes drops that "can be traded". Two wiki examples: a Cupcake Hat version from the 2011 Birthday Bash "can be worn by non-members, traded to other players", which the page contrasts with the quest and Coin Shop versions (implying those cannot be traded); Marketplace flair shards are "Cannot Sell. Cannot Trade."
- **Card trade (video, wiki, client).** See the overview. The wiki says the lobby trades boosters, cards and loot cards, and that cards from the free monthly membership pack are untradable, and that some promo cards from bundles "can be traded". Video also shows a tip that direct trades start from the Players or Friends tabs (`UFsVcZ3X5iA/00046`).
- **Trading Centers (wiki).** A wiki page lists spots where players gather to trade (Seaside, Bristlewood, Sanctuary Plaza, Rumble Ridge, plus holiday spots). It is social colour; nothing in the client depends on it.
- **Missing from wiki and video.** The wiki has no page on the trade window rules, no stack, coin or distance limits, and no statement on whether free players may trade. Among the frames returned by trade-related searches, none shows the invite toast text, the partner offer pane with a coin amount, or a refusal message.

### Open questions and contradictions

- **How a trade is requested.** No family-59 class or client sender asks for a trade. The `Trade` entry in the radial menu is server-sent; what happens on a click is not traced. The likely carrier is the interaction-select command the radial menu code builds (`00a38570`, which builds a `CommandPacketInteractionSelect` in its single-entry shortcut), but the multi-button click path through `Ui.RadialMenuClicked` was not followed. Unconfirmed.
- **What the counter means.** The shard calls the second dword of lock, accept and the updates a "revision"; a review rejected that name because native code only shows a copied dword. The code I read shows a running maximum starting at 1. Whether the original server rejected stale values is unknown.
- **The side byte and coin flag.** Byte 0 = partner, non-zero = you is read from the creation order of the two item sources and from the coin update ignoring non-zero. It is not named by any label (`00cf69c0` and `00cf79d0` are speculative and unknown).
- **Which side a lock update names.** Sub `0x11` locked 1 raises "other accepted". Sub `0x11` locked 0 hides your own popup. Whether the server sends the partner's unlock, your own, or both is not shown.
- **Unused sub-opcodes.** Sub `0xe` is read and ignored; sub 1 has no class; the status message statuses 5 and above 10 do nothing. Whether the original server ever sent them is unknown.
- **Wire width of the status.** The constructor stores the status as a dword while the reader takes one byte (shard S0399). The reader is what the server must match.
- **The invite's two dwords and the ignore check.** They are passed to `00a12720` and onwards. The ignore interpretation rests on where the lookup function sits, not on a label. The same dwords are never named.
- **Item record meaning.** Names for the add-item record come from the `ItemInstance` layout; the reader labels are speculative and the rental block (flag, 16 bytes) is not seen in a trade. A trade server can probably send the flag as 0, unverified.
- **Locale pairing.** `TradeCanceled`, `TradeRejected` and the two instanced-zone keys cannot be paired with one locale line each. The English locale file has no line mentioning an instanced zone together with trade. The `CodeStringMappings.txt` ids do not match the locale file ids, so every pairing here is by wording.
- **Stack and capacity rules.** The client reports "exceeds max stack" outcomes but does not compute them. The client's own clamp uses the item's stack count, not the partner's bag. Server rules for bag space are not recovered.
- **Account gates.** "Not flagged to allow trading", "blocked from participating in trades", "already in a trade session" and "not allowed to trade" exist in the locale. The flows that show them are not traced. `SetTradeAllowed` (348) carries a flag tree for the card side.
- **Card trade order and directions.** Directions and the order in Server item 13 are inferred from handler texts and class names. The class-header repeat per inheritance level is described in [[Trading Card Game]]. `LockTrade`, `AcceptTrade` and `FinalAcceptTrade` handlers are partly unlabelled. `RequestPostedTrades`' own handler is a stub, so the client only sends it. Senders for `IssueTradeRequest`, `AddCardToTrade`, `AddPostedTrade` and the other card actions were not found in the mapped range.
- **Mislabelled functions.** Review rejected the shard's claims that `00c6dd70` clears local state, that `00c6a7b0` is an SOE opcode, and that `00c6c470` allocates the tracker through `00cf68d0`. Other labels are marked as such above. Several lock/accept/offer senders have no mapped caller because they are reached through the Lua table and were read from bytes here.
- **Wiki versus client.** None found for player trade. For card trade the wiki says "Members" can trade; the client locale agrees. Nothing in the client says whether free players may use player trade.

### Key functions

| Address | Role |
|---|---|
| `009471a0`, `00a10940`, `00c6df10` | Client message switch (case `0x3b`), adjustor thunk, trade dispatcher |
| `00c6a7b0`, `00c6beb0`, `00c6cc30` | Family ctor, header reader, header writer |
| `00c6a8a0`, `00c6a980`, `00c6aa50`, `00c6ab10`, `00c6abd0`, `00c6ac90`, `00c6ad60` | Client-request and end-session constructors (subs 3, 5, 8, 6, 7, 9, 10) |
| `00c6ae20`, `00c6af30`, `00c6b040`, `00c6b160`, `00c6b240`, `00c6b300`, `00c6b3e0`, `00c6b4d0`, `00c6c290`, `00c6c340`, `00c6c3f0` | Server-packet constructors (subs `0xb` to `0x13`, 2, 4, `0x10`) |
| `00c6cab0`, `00c6cb40`, `00c6c530`, `00c6bf50`, `00c6ce30`, `00c6c000`, `00c6c590`, `00c6c630`, `00c6cdb0`, `00c6c110`, `00c6c5f0`, `00c6c180` | Field readers |
| `00c6cee0`, `00c6d060`, `00c6d100`, `00c6cf80`, `00c6d160`, `00c6d240` | Body writers |
| `00c6db40`, `00c6dbe0`, `00c6dc70`, `00c6dcf0`, `00c6dd70`, `00c6ddf0`, `00c6de90` | Request senders |
| `00c6d420`, `00c6d530`, `00c6d640`, `00c6d750`, `00c6d860`, `00c6d970` | Gateway send paths |
| `008e7290`, `008fa4b0`, `008cb540`, `008cb5a0`, `008cd5e0` | Name record and item record readers |
| `00c6c470`, `00c6be30`, `00cf68d0`, `00cf65b0` | Handler state and tracker constructors and destructors |
| `00cf7860`, `00cf6be0`, `00cf61e0`, `00cf79d0`, `00cf69c0` | Start-up of the item sources, reset, coin, add item, item count |
| `00cf6c90`, `00cf6850`, `00cf6240`, `00cf6260`, `00cf6f80` | Offer clamp, available count, partner coin read, category setter, partner name read |
| `00cf6ed0`, `00cf70d0`, `00cf76e0`, `00cf7200`, `00cf5d60` | Item data sources and their columns |
| `00c05ce0`, `01829c48`, `00c05d00` to `00c05da0`, `00c05e20` to `00c06020` | Lua `Trade` registrar, table and handlers |
| `00a0d320`, `00a0d540`, `00a09120`, `00a09130`, `00a0eb90`, `00917b60`, `00cf8700` | Invite toast, responders, removal, auto-refuse check, ignore-style lookup |
| `00a94ab0`, `00a94900` | System-message formatters |
| `0089ae70`, `0089adc0` | Gateway tunnel constructors (opcodes 6 and 5) |
| `00aa2360`, `00a38570` | Server radial menu command and its builder |
| `014426a0`, `01442780`, `014427e0`, `012d0810`, `012f0a00` | Card trade routing triple, base read and write, send and framing |
| `012c02f0`, `012bf9d0`, `013d1b50`, `013d2ac0`, `013d1520`, `012e1020` | Posted trade service, erase, `PostedTrade` ctor, reader, writer, error code switch |
<!-- NARRATIVE END -->

## Mapped functions (generated)

520 accepted functions: 242 confirmed, 134 likely, 87 speculative, 57 unknown. Network-linked (`wire`): 64.

Subsystems: `net.trade` (455), `ui.trade` (36), `game.trade` (29)

### `game.trade`

**TradeCommand** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01442520` | TradeCommand::TradeCommand | confirmed | Fastcall constructor (this in ECX, returns this). Calls the base constructor 0140c090 on this, stores the TradeCommand vtable, sets the field at +0x4 to 0 and … |
| `01442610` | TradeCommand::~TradeCommand | confirmed | Destructor body (fastcall, this in ECX, no deleting part). Stores TradeCommand::vftable at +0, calls the base destructor 0140c160 on this and returns. |
| `014428e0` | TradeCommand::ScalarDeletingDestructor | confirmed | Scalar deleting destructor shape (thiscall, one stack argument): stores TradeCommand::vftable, calls the base destructor 0140c160 on this, then calls _free on … |
| `01442860` | TradeCommand::Create | likely | Factory with no arguments. Calls operator new with 0xc bytes (00d833a6); returns 0 when that returns null, otherwise default-constructs the block with 01442520… |

**TradeService** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012c00a0` | TradeService_DtorBody | confirmed | Stores TradeService::vftable into this, clears DAT_01cbd690 when it points at this, clears the owner-level tree (012bf190 with ECX = this), then destroys the t… |
| `012c0270` | TradeService_Ctor | confirmed | Stores TradeService::vftable into this and calls 012c0110 with ECX = this+4 to initialise the owned-value tree object; returns this. |
| `012c02d0` | TradeService_ScalarDeletingDtor | confirmed | Calls the TradeService destructor body 012c00a0 on this, and when bit 0 of the flag argument is set frees this with _free; returns this with ret 4. |
| `012c0110` | TradeService_InitTree | likely | Initialises the tree object passed in ECX (this+4 from TradeService_Ctor): allocates the header node through 012becc0, sets its isnil byte to 1, links the head… |

**TradeSessionTracker** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf68d0` | TradeSessionTracker::Ctor | confirmed | Stores the owner pointer, allocates two 0x54-byte named GUI data objects for PartnerName and PartnerCoinCount, initializes their stored slots and clears remain… |
| `00cf65b0` | TradeSessionTracker::Dtor | likely | Releases five owned interface references at offsets +4 through +0x14 using vtable slot 0x28 and 0081ad80, clearing each reference when released. |

**(no class)** (19)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6df10` | TradeDispatchPacket 🌐 | confirmed | Reads the family and sub-opcode with 00c6beb0, then switches on the sub-opcode: 2 invite (00c6ccd0 reader, sets the partner id at param_1[1]); 4 session start … |
| `00a10940` | MethodThunk_00c6df10 | likely | Eleven-byte adjustor thunk ('add ecx,0x1d008; jmp 00c6df10'), recovered as the single call FUN_00c6df10(). The target is the trade handler whose own log string… |
| `00cf6240` | GetPartnerCoinCountIfPartnerState | likely | Reads state bits from the trade object; if the masked value equals 2, returns the dword at trade object+0x50, otherwise returns zero. |
| `00cf6260` | SetTradeField_2e4 | likely | Writes the supplied value to trade object+0x2e4 only when changed, then invokes the callback at trade object+0x254+0x18. |
| `00cf6850` | ComputeTradeEligibleCount | likely | Gets an eligibility record from a global lookup; if present and inactive, computes the available stack count and subtracts one when the record's category-speci… |
| `012bde90` | GlobalObj01cbd690_Destroy | likely | When DAT_01cbd690 is non-null it calls the object's vtable slot 0 with flag 1 (scalar deleting destructor), then sets DAT_01cbd690 to 0. |
| `012bf190` | OwnedTree_ClearValues_15 | likely | Walks the tree object at this+4 (head read at this+8) from its first node: for each node with a non-null value word at +0x10 it calls the value's vtable slot 0… |
| `012bf250` | OwnedTree_FindIntKeyValue | likely | Finds the node for the int key argument in the tree object at this+4 (through 012bf020); returns the value word at node+0x10 when the found node is not the end… |
| `012bf9d0` | OwnedTree_EraseIntKey | likely | For the owner's tree object at this+4: finds the node for the int key argument through 012befb0; when it is found it calls the stored owned value's vtable slot… |
| `012c02f0` | GlobalObj01cbd690_GetOrCreate | likely | If DAT_01cbd690 is zero, allocates 0x10 bytes; when that succeeds it constructs a TradeService through 012c0270 and stores it in DAT_01cbd690; otherwise leaves… |
| `00cf69c0` | TradeInventoryAdjustmentCallback | speculative | Selects one of two referenced objects, resolves an entry, invokes a method based on the signed quantity, and notifies the selected object. |
| `00cf6be0` | TradeStateCleanup | speculative | Resets a global status field, clears linked nodes from two object lists, and invokes each list object's virtual method at offset 0x10. |
| `00cf6c90` | AdjustTradeItemQuantity | speculative | Validates two objects, computes the current item count, clamps a positive or negative quantity, updates the item count, and notifies datasource callbacks. |
| `012beee0` | TreeCountNodesWithPredicates | speculative | Walks the tree whose head is read at this+8 from its first node to the header; counts the nodes whose value word at +0x10 is non-null, for which 013d0fb0 retur… |
| `012c0360` | OwnedTree_StoreByKey | speculative | When the argument is non-null, derives its key through 013d0f10 and looks it up in the tree object at this+4 (012befb0); an existing non-null owned value is de… |
| `012c0400` | OwnedTree_CollectValues | speculative | Walks the owner's tree (tree object at this+4, head at this+8) from first to last node; for each node whose value word at +0x10 is non-null it passes that valu… |
| `012c0480` | OwnedTree_CollectMatchingKeys | speculative | Walks the owner's tree (head read at this+8) and, for each node with a non-null value at +0x10 that passes a combination of 013d0fb0, 013d0ff0, 013a46b0 and 01… |
| `00cf6800` | — | unknown | Finds a matching map entry; when found subtracts three stored quantity fields from the total at this+0x14, otherwise subtracts two fields, then returns the res… |
| `00cf79d0` | — | unknown | Selects one of two trade-side objects, passes an item pointer and count to FUN_00cf7050, then invokes the selected object's virtual update callback on success. |

### `net.trade`

**BaseTradePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6b650` | BaseTradePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the BaseTradePacket vtable: under an SEH frame it writes BasePacket::vftable at +0 and calls _free on this when bit 0 o… |
| `00c6a7b0` | — 🌐 | unknown | Family constructor for every trade message: installs BasePacket::vftable at +0, writes the family byte 0x3b (59) at +4, then swaps in BaseTradePacket::vftable … |

**PostedTrade** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d1cc0` | PostedTrade::GetClassName | confirmed | Virtual slot 11: returns the address of the literal PostedTrade. |
| `013d1d10` | PostedTrade::~PostedTrade | confirmed | Destructor body: stores both PostedTrade vtables, releases the item-list vector at +0xfc (through 012327d0 and _free), clears the string members, calls 00521a9… |
| `013d2980` | PostedTrade::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 013d1d10, then calls _free on this when bit 0 of the second argument is set; returns this. |
| `013d1520` | PostedTrade::Serialize | likely | Virtual slot 10 of PostedTrade: calls slot 2 with its argument, appends the ints at this+0x08 and this+0x0c and the string at +0x2c with SerialAppendInteger an… |
| `013d28f0` | PostedTrade::Create | likely | Allocates 0x108 bytes with 00d833a6; when non-null constructs a PostedTrade with 013d1b50 and returns it, otherwise returns 0. |
| `013d2ac0` | PostedTrade::Deserialize | likely | Virtual slot 9 of PostedTrade: reads a header through slot 4, then ints at +0x08 and +0x0c, the string at +0x2c, the int at +0x48, 01300190 on +0x4c and +0x58,… |
| `013d1cd0` | PostedTrade::GetGlobalDword_01cbde10 | speculative | Virtual slot 12: returns the process-global dword DAT_01cbde10. |
| `013d1ce0` | PostedTrade::SetGlobalDword_01cbde10 | speculative | Virtual slot 13: stores its argument into the global dword DAT_01cbde10. |
| `013d1d00` | PostedTrade::AdjustorThunk_4 | speculative | Thunk in the secondary PostedTrade vtable (object+4), slot 0: subtracts 4 from ECX and jumps to 013d2980. |
| `013d29a0` | PostedTrade::AdjustorThunk_Minus4 | speculative | Thunk in the secondary PostedTrade vtable (object+4), slot 1: adds -4 to ECX and jumps to 013d2140. |
| `013d1b50` | — | unknown | Constructor: stores PersistentBase and PersistentComponent vtables, then PostedTrade::vftable at +0x00 and +0x04; zeroes the dwords at +0x08, +0x0c and +0x48; … |

**TradeAcceptPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6b810` | TradeAcceptPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6ab10` | TradeAcceptPacket::TradeAcceptPacket 🌐 | likely | Constructor of the client's accept request: calls the family constructor FUN_00c6a7b0(6), installs TradeAcceptPacket::vftable, stores the session id at +0xc an… |
| `00c6d060` | TradeAcceptPacket::Write 🌐 | speculative | Serialises the accept request: writes the header with 00c6cc30, then copies the 32-bit session id at +0xc and the 32-bit second field at +0x10 into the writer … |

**TradeCancelPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6abd0` | TradeCancelPacket::TradeCancelPacket 🌐 | confirmed | Constructor of the client's cancel request: calls the family constructor FUN_00c6a7b0(7), installs TradeCancelPacket::vftable and stores the session id at +0xc. |
| `00c6b870` | TradeCancelPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6d100` | — 🌐 | unknown | Serialises the cancel request: writes the header with 00c6cc30 and copies the single 32-bit session id at +0xc into the writer through FUN_007cc400. |

**TradeCoinCountPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6ad60` | TradeCoinCountPacket::TradeCoinCountPacket 🌐 | confirmed | Constructor of the client's coin offer: calls the family constructor FUN_00c6a7b0(10), installs TradeCoinCountPacket::vftable and stores the session id at +0xc… |
| `00c6b930` | TradeCoinCountPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6d240` | TradeCoinCountPacket::Write 🌐 | likely | Serialises the coin offer: writes the header with 00c6cc30, then copies the 32-bit session id at +0xc and the 32-bit coin count at +0x10 into the writer throug… |

**TradeCommand** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014425d0` | TradeCommand::TypeName | confirmed | Virtual slot 11 with no arguments. Returns the address of the literal TradeCommand (018ea854). |
| `014426a0` | TradeCommand::WriteRoutingTriple | confirmed | Virtual slot 21 (thiscall, three stack arguments, ret 0xc; bytes verified). Stores 0x3b9aca02 (1000000002) through the first pointer argument, 0 through the se… |
| `014426d0` | TradeCommand::Clone | confirmed | Virtual slot 17 (fastcall, this in ECX). Allocates 0xc bytes with operator new (00d833a6); when non-null default-constructs the block with 01442520; then calls… |
| `01442740` | TradeCommand::CopyFieldsTo | confirmed | Virtual slot 18 (thiscall: this is the source, one stack argument is the destination NetworkCommand). Calls the base copy stub 0140c280 (empty) with the destin… |
| `01442780` | TradeCommand::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls slot 4 on this with (param_2, &param_2) and returns 0 when that fails; calls base-class read 0140c2a0 with… |
| `014427e0` | TradeCommand::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls slot 2 on this with param_2, calls 013c26e0 with Starting TradeCommand (a no-op that returns 1, so the te… |
| `01442940` | TradeCommand::DumpText | confirmed | Virtual slot 19 (thiscall, one stack argument: the std::string that receives text). Calls the base stub 0140c290 with the argument (empty), formats AccountID: … |
| `014429f0` | TradeCommand::AppendDisplayName | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument (the std::string receiving text) and appends the 13-byte literal Trade Command (018ea8… |
| `014425e0` | TradeCommand::GetStaticDword | speculative | Virtual slot 12 with no arguments. Returns the dword stored in the global DAT_01cbe3f0. |
| `014425f0` | TradeCommand::SetStaticDword | speculative | Virtual slot 13 with one argument. Stores the argument into the global dword DAT_01cbe3f0. |

**TradeCommand_AcceptPostedTrade** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d09e0` | TradeCommand_AcceptPostedTrade::TradeCommand_AcceptPostedTr… | confirmed | Constructor: calls 01442520, stores TradeCommand_AcceptPostedTrade::vftable, and sets the dwords at +0x0c and +0x10 to 0; returns this. |
| `013d0a60` | TradeCommand_AcceptPostedTrade::CommandName | confirmed | Virtual slot 11: returns the address of the literal TradeCommand_AcceptPostedTrade. |
| `013d0bd0` | TradeCommand_AcceptPostedTrade::Clone | confirmed | Virtual slot 17: allocates 0x14 bytes with 00d833a6, constructs an AcceptPostedTrade (013d09e0) in them, then calls slot 18 of this (013d0c40) with the new obj… |
| `013d0db0` | TradeCommand_AcceptPostedTrade::ScalarDeletingDtor | confirmed | Scalar deleting destructor with the body inlined: stores the vtable, calls 01442610, then calls _free on this when bit 0 of the second argument is set; returns… |
| `013d0e10` | TradeCommand_AcceptPostedTrade::DebugDump | confirmed | Virtual slot 19: writes Accept Posted Trade Trade Command through 01241650, calls 01442940 on the argument, formats Posted Trade ID: %d from this+0x0c and, whe… |
| `013d0ee0` | TradeCommand_AcceptPostedTrade::EmitDebugName | confirmed | Virtual slot 20: passes the literal Accept Posted Trade Trade Command (length 0x21) to 01241650. |
| `013d0c40` | TradeCommand_AcceptPostedTrade::CopyInto | likely | Virtual slot 18: calls 01442740 on the argument, dynamic-casts it to AcceptPostedTrade through 00d8d382, and copies the dwords at this+0x0c and this+0x10 into … |
| `013d0c80` | TradeCommand_AcceptPostedTrade::Deserialize | likely | Virtual slot 9: reads a header through slot 4, calls 01442780 on the argument, reads an integer into this+0x0c with 013c24d0, reads a second integer into this+… |
| `013d0cf0` | TradeCommand_AcceptPostedTrade::Serialize | likely | Virtual slot 10: calls slot 2 with its argument, calls 014427e0, appends the dwords at this+0x0c and this+0x10 with SerialAppendInteger 013c23f0, then calls sl… |
| `013d0d30` | TradeCommand_AcceptPostedTrade::Create | likely | Allocates 0x14 bytes with 00d833a6; when non-null constructs an AcceptPostedTrade with 013d09e0 and returns it, otherwise returns 0. |
| `013d0a70` | TradeCommand_AcceptPostedTrade::GetGlobalDword_01cbde0c | speculative | Virtual slot 12: returns the process-global dword DAT_01cbde0c. |
| `013d0a80` | TradeCommand_AcceptPostedTrade::SetGlobalDword_01cbde0c | speculative | Virtual slot 13: stores its argument into the global dword DAT_01cbde0c. |
| `013d0b30` | TradeCommand_AcceptPostedTrade::PushAcceptEvent | speculative | Virtual slot 15: calls 012d46a0, allocates 0x14 bytes with 00d833a6 and, when non-null, calls 012f9eb0; calls 012f8c70 with 0x8a, 012fa2e0 with the dwords at t… |
| `013d0aa0` | — | unknown | Destructor body: stores the vtable, saves this in a local for the unwind frame and calls 01442610; does not free memory. |

**TradeCommand_AcceptTrade** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ac190` | TradeCommand_AcceptTrade::TradeCommand_AcceptTrade | confirmed | Fastcall constructor (ECX=this). Calls base constructor 01442520, stores TradeCommand_AcceptTrade::vftable, runs the stack-temporary base construct/destroy pai… |
| `014ac230` | TradeCommand_AcceptTrade::GetClassName | confirmed | Virtual slot 11 (no arguments). Returns the address of the literal TradeCommand_AcceptTrade. |
| `014ac270` | TradeCommand_AcceptTrade::DtorBody | confirmed | Fastcall (ECX=this). Stores TradeCommand_AcceptTrade::vftable into *this, then calls the base destructor body 01442610 on this; no free. |
| `014ac3f0` | TradeCommand_AcceptTrade::CopyFieldsInto | confirmed | Thiscall (ECX=source this, one stack argument = destination, ret 4). Calls 01442740(this, dest) for the base fields, runs a dynamic_cast of dest from NetworkCo… |
| `014ac420` | TradeCommand_AcceptTrade::Read | confirmed | Virtual slot 9 (thiscall, one stack argument, ret 4). vtbl[4](buf,&local) header read with a local initialised to this; 01442780 base read; 013c24d0 reads one … |
| `014ac480` | TradeCommand_AcceptTrade::Write | confirmed | Virtual slot 10 (thiscall, one stack argument, ret 4). vtbl[2](buf); no-op trace; 014427e0 base Write; trace Trade Button label; 013c23f0 writes the byte at th… |
| `014ac560` | TradeCommand_AcceptTrade::ScalarDeletingDtor | confirmed | Vtable slot 0 (ECX=this, flag byte on stack). Stores the AcceptTrade vtable, calls base destructor body 01442610 directly, then calls _free(this) when bit 0 of… |
| `014ac5c0` | TradeCommand_AcceptTrade::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument). Appends the literal Accept Trade Trade Command newline first, then calls 01442940 (base dump) on the argument, … |
| `014ac650` | TradeCommand_AcceptTrade::AppendTypeName | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument and appends the 26-byte literal Accept Trade Trade Command with 01241650. |
| `014ac380` | TradeCommand_AcceptTrade::Clone | likely | Virtual slot 17 (fastcall, ECX=this, no stack arguments). Allocates 0x10 bytes with operator new; if non-null, constructs an AcceptTrade on it with 014ac190. C… |
| `014ac240` | TradeCommand_AcceptTrade::GetGlobal_01cbe6bc | speculative | Virtual slot 12 (no arguments). Returns the dword at global DAT_01cbe6bc. |
| `014ac250` | TradeCommand_AcceptTrade::SetGlobal_01cbe6bc | speculative | Virtual slot 13 (one stack argument, ret 4). Stores the argument into global DAT_01cbe6bc. |
| `014ac2e0` | TradeCommand_AcceptTrade::QueueEvent0x26 | speculative | Virtual slot 15 (fastcall, ECX=this, no stack arguments, returns 1). Takes a queue pointer from 012d46a0, allocates 0x14 bytes, constructs the block with 012f9… |

**TradeCommand_AcceptTradeRequest** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014aba90` | TradeCommand_AcceptTradeRequest::TradeCommand_AcceptTradeRe… | confirmed | Fastcall constructor (ECX=this). Calls base constructor 01442520, stores TradeCommand_AcceptTradeRequest::vftable, runs the stack-temporary base construct/dest… |
| `014abb30` | TradeCommand_AcceptTradeRequest::GetClassName | confirmed | Virtual slot 11 (no arguments). Returns the address of the literal TradeCommand_AcceptTradeRequest. |
| `014abb70` | TradeCommand_AcceptTradeRequest::DtorBody | confirmed | Fastcall (ECX=this). Stores TradeCommand_AcceptTradeRequest::vftable into *this, then calls the base destructor body 01442610 on this; no free. |
| `014abc70` | TradeCommand_AcceptTradeRequest::CopyFieldsInto | confirmed | Thiscall (ECX=source this, one stack argument = destination, ret 4). Calls 01442740(this, dest) for the base fields, runs a dynamic_cast of dest from NetworkCo… |
| `014abcb0` | TradeCommand_AcceptTradeRequest::Read | confirmed | Virtual slot 9 (thiscall, one stack argument, ret 4). vtbl[4](buf,&buf) header read; 01442780 base read; 013c24d0 reads an integer into this+0x0c and another i… |
| `014abd10` | TradeCommand_AcceptTradeRequest::Write | confirmed | Virtual slot 10 (thiscall, one stack argument, ret 4). vtbl[2](buf); no-op trace; 014427e0 base Write; trace Issued From AccountID; 013c23f0 writes this+0x0c; … |
| `014abe10` | TradeCommand_AcceptTradeRequest::ScalarDeletingDtor | confirmed | Vtable slot 0 (ECX=this, flag byte on stack). Stores the AcceptTradeRequest vtable, calls base destructor body 01442610, then calls _free(this) when bit 0 of t… |
| `014abe70` | TradeCommand_AcceptTradeRequest::NotifyAccepted | confirmed | Virtual slot 15 (fastcall, ECX=this, no stack arguments). Queues a 0x14-byte event of type 0x22 with this+0x08, +0x0c, +0x04 and +0x10 (012fa2e0 four times, th… |
| `014ac0b0` | TradeCommand_AcceptTradeRequest::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument). Calls 01442940 (base dump), appends the literal Accept Trade Request Trade Command newline, then Issued From: %… |
| `014ac170` | TradeCommand_AcceptTradeRequest::AppendTypeName | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument and appends the 34-byte literal Accept Trade Request Trade Command with 01241650. |
| `014abc00` | TradeCommand_AcceptTradeRequest::Clone | likely | Virtual slot 17 (fastcall, ECX=this, no stack arguments). Allocates 0x14 bytes with operator new; if non-null, constructs an AcceptTradeRequest on it with 014a… |
| `014abb40` | TradeCommand_AcceptTradeRequest::GetGlobal_01cbe6b8 | speculative | Virtual slot 12 (no arguments). Returns the dword at global DAT_01cbe6b8. |
| `014abb50` | TradeCommand_AcceptTradeRequest::SetGlobal_01cbe6b8 | speculative | Virtual slot 13 (one stack argument, ret 4). Stores the argument into global DAT_01cbe6b8. |

**TradeCommand_AddCardToTrade** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ab0d0` | TradeCommand_AddCardToTrade::TradeCommand_AddCardToTrade | confirmed | Fastcall constructor (ECX=this). Calls base constructor 01442520, stores TradeCommand_AddCardToTrade::vftable, constructs the member at this+0x0c with 013d3300… |
| `014ab180` | TradeCommand_AddCardToTrade::GetClassName | confirmed | Virtual slot 11 (no arguments). Returns the address of the literal TradeCommand_AddCardToTrade. |
| `014ab1c0` | TradeCommand_AddCardToTrade::DtorBody | confirmed | Fastcall (ECX=this). Stores TradeCommand_AddCardToTrade::vftable, calls the member destructor 013d34d0 on this+0x0c, then the base destructor body 01442610 on … |
| `014ab340` | TradeCommand_AddCardToTrade::CopyFieldsInto | confirmed | Thiscall (ECX=source this, one stack argument = destination, ret 4). Calls 01442740(this, dest) for the base fields, runs a dynamic_cast of dest from NetworkCo… |
| `014ab380` | TradeCommand_AddCardToTrade::Read | confirmed | Virtual slot 9 (thiscall, one stack argument, ret 4). vtbl[4](buf,&buf) header read; 01442780 base read; then 013c24d0 reads three integers into this+0x10, thi… |
| `014ab3f0` | TradeCommand_AddCardToTrade::Write | confirmed | Virtual slot 10 (thiscall, one stack argument, ret 4). vtbl[2](buf); no-op trace; 014427e0 base Write; then 013c23f0 writes this+0x10 (AddCardID archetype), th… |
| `014ab500` | TradeCommand_AddCardToTrade::ScalarDeletingDtor | confirmed | Vtable slot 0 (ECX=this, flag byte on stack). Calls the DtorBody 014ab1c0, then calls _free(this) when bit 0 of the flag is set; returns this. |
| `014ab970` | TradeCommand_AddCardToTrade::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument, ret 4). Calls 01442940 (base dump) on the argument, then appends the literal Add Card To Trade Trade Command new… |
| `014aba70` | TradeCommand_AddCardToTrade::AppendTypeName | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument and appends the 31-byte literal Add Card To Trade Trade Command with 01241650. |
| `014ab2d0` | TradeCommand_AddCardToTrade::Clone | likely | Virtual slot 17 (fastcall, ECX=this, no stack arguments). Allocates 0x20 bytes with operator new; if non-null, constructs an AddCardToTrade on it with 014ab0d0… |
| `014ab190` | TradeCommand_AddCardToTrade::GetGlobal_01cbe6b4 | speculative | Virtual slot 12 (no arguments). Returns the dword at global DAT_01cbe6b4. |
| `014ab1a0` | TradeCommand_AddCardToTrade::SetGlobal_01cbe6b4 | speculative | Virtual slot 13 (one stack argument). Stores the argument into global DAT_01cbe6b4. |
| `014ab520` | — | unknown | Virtual slot 15 (fastcall, ECX=this, no stack arguments, returns nothing). Takes a queue pointer from 012d46a0, allocates 0x14 bytes, constructs the block with… |

**TradeCommand_AddPostedTrade** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014aabe0` | TradeCommand_AddPostedTrade::TradeCommand_AddPostedTrade | confirmed | Fastcall constructor (ECX=this). Calls base constructor 01442520, stores TradeCommand_AddPostedTrade::vftable, zeroes this+0x0c (owned object pointer) and sets… |
| `014aac60` | TradeCommand_AddPostedTrade::GetClassName | confirmed | Virtual slot 11 (no arguments). Returns the address of the literal TradeCommand_AddPostedTrade. |
| `014aaca0` | TradeCommand_AddPostedTrade::DtorBody | confirmed | Fastcall (ECX=this). Stores TradeCommand_AddPostedTrade::vftable; if the flag byte at this+0x10 is 1 and the owned pointer at this+0x0c is non-null, calls the … |
| `014aae80` | TradeCommand_AddPostedTrade::CopyFieldsInto | confirmed | Thiscall (ECX=source this, one stack argument = destination, ret 4). Calls 01442740(this, dest) for the base fields, runs a dynamic_cast of dest from NetworkCo… |
| `014aaf20` | TradeCommand_AddPostedTrade::Write | confirmed | Virtual slot 10 (thiscall, one stack argument, ret 4). vtbl[2](buf); no-op trace; 014427e0 base Write; if this+0x0c is null writes integer 1 with 013c23f0, oth… |
| `014ab010` | TradeCommand_AddPostedTrade::ScalarDeletingDtor | confirmed | Vtable slot 0 (ECX=this, flag byte on stack). Calls the DtorBody 014aaca0, then calls _free(this) when bit 0 of the flag is set; returns this. |
| `014ab030` | TradeCommand_AddPostedTrade::Read | confirmed | Virtual slot 9 (thiscall, one stack argument, ret 4). vtbl[4](buf,&buf) header read; 01442780 base read; then 014a41c0(buf, &this+0x0c) reads the owned payload… |
| `014ab080` | TradeCommand_AddPostedTrade::Dump | confirmed | Virtual slot 19 (one stack argument, ret 4). Appends the literal Add Posted Trade Trade Command newline first, then calls 01442940 (base dump) on the argument.… |
| `014ab0b0` | TradeCommand_AddPostedTrade::AppendTypeName | confirmed | Virtual slot 20 (one stack argument). Loads ECX from the argument and appends the 30-byte literal Add Posted Trade Trade Command with 01241650. |
| `014aae10` | TradeCommand_AddPostedTrade::Clone | likely | Virtual slot 17 (fastcall, ECX=this, no stack arguments). Allocates 0x14 bytes; if non-null, constructs an AddPostedTrade on it with 014aabe0; calls this->vtbl… |
| `014aac70` | TradeCommand_AddPostedTrade::GetGlobal_01cbe6b0 | speculative | Virtual slot 12 (no arguments). Returns the dword at global DAT_01cbe6b0. |
| `014aac80` | TradeCommand_AddPostedTrade::SetGlobal_01cbe6b0 | speculative | Virtual slot 13 (one stack argument). Stores the argument into global DAT_01cbe6b0. |
| `014aad60` | TradeCommand_AddPostedTrade::QueueEvent0x8b | speculative | Virtual slot 15 (fastcall, ECX=this, returns 1). If this+0x0c is non-null: clears the flag byte at this+0x10, calls 012c02f0 and 012c0360 on the owned object, … |

**TradeCommand_CancelPostedTrade** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d0540` | TradeCommand_CancelPostedTrade::TradeCommand_CancelPostedTr… | confirmed | Constructor: calls 01442520, stores TradeCommand_CancelPostedTrade::vftable and sets the dword at +0x0c to 0; returns this. |
| `013d05c0` | TradeCommand_CancelPostedTrade::CommandName | confirmed | Virtual slot 11: returns the address of the literal TradeCommand_CancelPostedTrade. |
| `013d0710` | TradeCommand_CancelPostedTrade::Clone | confirmed | Virtual slot 17: allocates 0x10 bytes with 00d833a6, constructs a CancelPostedTrade (013d0540) in them, then calls slot 18 of this (013d0780) with the new obje… |
| `013d08d0` | TradeCommand_CancelPostedTrade::ScalarDeletingDtor | confirmed | Scalar deleting destructor with the body inlined: stores the vtable, calls 01442610, then calls _free on this when bit 0 of the second argument is set; returns… |
| `013d0930` | TradeCommand_CancelPostedTrade::DebugDump | confirmed | Virtual slot 19: writes Cancel Posted Trade Trade Command through 01241650, calls 01442940 on the argument, formats Posted Trade ID: %d from this+0x0c with _sp… |
| `013d09c0` | TradeCommand_CancelPostedTrade::EmitDebugName | confirmed | Virtual slot 20: passes the literal Cancel Posted Trade Trade Command (length 0x21) to 01241650. |
| `013d0780` | TradeCommand_CancelPostedTrade::CopyInto | likely | Virtual slot 18: calls 01442740 on the argument, dynamic-casts it to CancelPostedTrade through 00d8d382, and copies the dword at this+0x0c into the cast object… |
| `013d07b0` | TradeCommand_CancelPostedTrade::Deserialize | likely | Virtual slot 9: reads a header through slot 4, calls 01442780 with the argument, reads an integer into this+0x0c with 013c24d0, then calls slot 5 (+0x14); retu… |
| `013d0800` | TradeCommand_CancelPostedTrade::Serialize | likely | Virtual slot 10: calls slot 2 with its argument, writes trace text through 013c26e0 (a no-op), calls 014427e0, appends the dword at this+0x0c with SerialAppend… |
| `013d0850` | TradeCommand_CancelPostedTrade::Create | likely | Allocates 0x10 bytes with 00d833a6; when non-null constructs a CancelPostedTrade with 013d0540 and returns it, otherwise returns 0. |
| `013d05d0` | TradeCommand_CancelPostedTrade::GetGlobalDword_01cbde08 | speculative | Virtual slot 12: returns the process-global dword DAT_01cbde08. |
| `013d05e0` | TradeCommand_CancelPostedTrade::SetGlobalDword_01cbde08 | speculative | Virtual slot 13: stores its argument into the global dword DAT_01cbde08. |
| `013d0670` | TradeCommand_CancelPostedTrade::PushCancelEvent | speculative | Virtual slot 15: calls 012d46a0, reads the dword at this+0x0c and passes it to 012c02f0 and 012bf9d0; allocates 0x14 bytes with 00d833a6 and, when non-null, ca… |
| `013d0600` | — | unknown | Destructor body: stores the vtable, saves this in a local for the unwind frame and calls 01442610; does not free memory. |

**TradeCommand_CancelTradeRequest** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014aa450` | TradeCommand_CancelTradeRequest::TradeCommand_CancelTradeRe… | confirmed | Fastcall constructor (ECX=this). Calls base constructor 01442520, stores TradeCommand_CancelTradeRequest::vftable, runs the stack-temporary base construct/dest… |
| `014aa4f0` | TradeCommand_CancelTradeRequest::GetClassName | confirmed | Virtual slot 11 (no arguments). Returns the address of the literal TradeCommand_CancelTradeRequest. |
| `014aa530` | TradeCommand_CancelTradeRequest::DtorBody | confirmed | Fastcall (ECX=this). Stores TradeCommand_CancelTradeRequest::vftable into *this, then calls the base destructor body 01442610; no free. |
| `014aa630` | TradeCommand_CancelTradeRequest::CopyFieldsInto | confirmed | Thiscall (ECX=source this, one stack argument = destination, ret 4). Calls 01442740(this, dest) for AccountID and TradeSessionID, runs a dynamic_cast of dest f… |
| `014aa670` | TradeCommand_CancelTradeRequest::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). vtbl[4](buf, &buf) header read, which writes the header version back into the stack argument slot; then 01442780… |
| `014aa6e0` | TradeCommand_CancelTradeRequest::Write | confirmed | Virtual slot 10 (thiscall, one stack argument, ret 4). vtbl[2](buf); no-op trace; 014427e0 base Write; trace Issued From AccountID; 013c23f0 writes this+0x0c; … |
| `014aa7e0` | TradeCommand_CancelTradeRequest::ScalarDeletingDtor | confirmed | Vtable slot 0 (ECX=this, flag byte on stack). Stores the CancelTradeRequest vtable, calls base destructor body 01442610, then calls _free(this) when bit 0 of t… |
| `014aab00` | TradeCommand_CancelTradeRequest::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument). Calls 01442940 (base dump) on the argument, appends the literal Cancel Trade Request Trade Command newline, the… |
| `014aabc0` | TradeCommand_CancelTradeRequest::AppendTypeName | confirmed | Virtual slot 20 (one stack argument). Loads ECX from the argument and appends the 34-byte literal Cancel Trade Request Trade Command with 01241650. |
| `014aa5c0` | TradeCommand_CancelTradeRequest::Clone | likely | Virtual slot 17 (fastcall, ECX=this, no stack arguments). Allocates 0x14 bytes with operator new; if non-null, constructs a CancelTradeRequest on it with 014aa… |
| `014aa500` | TradeCommand_CancelTradeRequest::GetGlobal_01cbe6ac | speculative | Virtual slot 12 (no arguments). Returns the dword at global DAT_01cbe6ac. |
| `014aa510` | TradeCommand_CancelTradeRequest::SetGlobal_01cbe6ac | speculative | Virtual slot 13 (one stack argument). Stores the argument into global DAT_01cbe6ac. |
| `014aa840` | — | unknown | Virtual slot 15 (fastcall, ECX=this, no stack arguments). Queues a 0x14-byte event of type 0x23 carrying this+0x08 and this+0x0c (012fa2e0, 012fa360(1), 012d35… |

**TradeCommand_ChangeTradeSession** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014aa000` | TradeCommand_ChangeTradeSession::TradeCommand_ChangeTradeSe… | confirmed | Fastcall constructor (ECX=this). Calls base constructor 01442520, stores TradeCommand_ChangeTradeSession::vftable, runs the same stack-temporary base construct… |
| `014aa0a0` | TradeCommand_ChangeTradeSession::GetClassName | confirmed | Virtual slot 11 (no arguments). Returns the address of the literal TradeCommand_ChangeTradeSession. |
| `014aa240` | TradeCommand_ChangeTradeSession::CopyFieldsInto | confirmed | Thiscall (ECX=source this, one stack argument = destination, ret 4). Calls 01442740(this, dest) to copy AccountID and TradeSessionID, runs a dynamic_cast of de… |
| `014aa280` | TradeCommand_ChangeTradeSession::Read | confirmed | Virtual slot 9 (thiscall, one stack argument, ret 4). vtbl[4](buf,&buf) header read; then 01442780 base read; then 013c24d0 reads one integer into this+0x0c; t… |
| `014aa2d0` | TradeCommand_ChangeTradeSession::Write | confirmed | Virtual slot 10 (thiscall, one stack argument, ret 4). vtbl[2](buf); no-op trace; 014427e0 base Write (AccountID, TradeSessionID); trace text OldTradeSessionID… |
| `014aa3b0` | TradeCommand_ChangeTradeSession::ScalarDeletingDtor | confirmed | Vtable slot 0 (ECX=this, flag byte on stack). Stores TradeCommand_ChangeTradeSession::vftable, calls base destructor body 01442610, then calls _free(this) when… |
| `014aa410` | TradeCommand_ChangeTradeSession::Dump | confirmed | Virtual slot 19 (ECX=this, one stack argument, ret 4; bytes push the argument and call 01442940 with ECX unchanged). Calls 01442940 (TradeCommand base dump) on… |
| `014aa430` | TradeCommand_ChangeTradeSession::AppendTypeName | confirmed | Virtual slot 20 (one stack argument, ret 4, same 19-byte shape as the other AppendTypeName functions). Loads ECX from the argument and appends the 34-byte lite… |
| `014aa1d0` | TradeCommand_ChangeTradeSession::Clone | likely | Virtual slot 17 (fastcall, ECX=this, no stack arguments). Allocates 0x10 bytes with operator new; if non-null, constructs a ChangeTradeSession on it with 014aa… |
| `014aa0b0` | TradeCommand_ChangeTradeSession::GetGlobal_01cbe6a8 | speculative | Virtual slot 12 (no arguments). Returns the dword at global DAT_01cbe6a8 (mov eax, [01cbe6a8]; ret). |
| `014aa0c0` | TradeCommand_ChangeTradeSession::SetGlobal_01cbe6a8 | speculative | Virtual slot 13 (one stack argument, ret 4 by the single-argument shape). Stores the argument into global DAT_01cbe6a8. |
| `014aa130` | TradeCommand_ChangeTradeSession::QueueEvent0x35 | speculative | Virtual slot 15 (fastcall, ECX=this, no stack arguments, returns 1). Takes a queue pointer from 012d46a0, allocates 0x14 bytes, constructs the block with 012f9… |
| `014aa0e0` | — | unknown | Fastcall (ECX=this). Stores TradeCommand_ChangeTradeSession::vftable into *this, then calls the base destructor body 01442610 on this; no free. |

**TradeCommand_ClearAcceptState** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a9c00` | TradeCommand_ClearAcceptState::TradeCommand_ClearAcceptState | confirmed | Fastcall constructor (ECX=this). Calls base constructor 01442520 on this, stores TradeCommand_ClearAcceptState::vftable, then constructs and destroys a stack t… |
| `014a9c90` | TradeCommand_ClearAcceptState::GetClassName | confirmed | Virtual slot 11 (no arguments). Returns the address of the literal TradeCommand_ClearAcceptState (mov eax, imm; ret). |
| `014a9e20` | TradeCommand_ClearAcceptState::CopyBaseFieldsInto | confirmed | Virtual slot 18 (stdcall, ECX=source this, one stack argument = destination, ret 4). Calls 01442740(this, dest), which copies the source AccountID (+4) and Tra… |
| `014a9e50` | TradeCommand_ClearAcceptState::Read | confirmed | Virtual slot 9 (ECX=this, one stack argument, ret 4). Same shape as ClearTrade Read: vtbl[4](buf,&buf) header read, then 01442780 base read, then vtbl[5](buf);… |
| `014a9e90` | TradeCommand_ClearAcceptState::Write | confirmed | Virtual slot 10 (ECX=this, one stack argument, ret 4). Calls vtbl[2](buf), the no-op trace 013c26e0 with the Starting text, base 014427e0 (TradeCommand Write),… |
| `014a9f60` | TradeCommand_ClearAcceptState::ScalarDeletingDtor | confirmed | Vtable slot 0 (ECX=this, flag byte on stack). Stores TradeCommand_ClearAcceptState::vftable, calls base destructor body 01442610, then calls _free(this) when b… |
| `014a9fc0` | TradeCommand_ClearAcceptState::Dump | confirmed | Virtual slot 19 (ECX=this, one stack argument, ret 4). Calls 01442940 (TradeCommand base dump) on the argument, then appends the 30-byte literal TradeCommand_C… |
| `014a9fe0` | TradeCommand_ClearAcceptState::AppendTypeName | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument and calls 01241650 (std::string append) with the 32-byte literal Clear Accept State Tr… |
| `014a9db0` | TradeCommand_ClearAcceptState::Clone | likely | Virtual slot 17 (fastcall, ECX=this, no stack arguments). Allocates 0x0c bytes with operator new; if non-null, constructs a ClearAcceptState on it with 014a9c0… |
| `014a9ca0` | TradeCommand_ClearAcceptState::GetGlobal_01cbe6a4 | speculative | Virtual slot 12 (no arguments). Returns the dword at global DAT_01cbe6a4 (mov eax, [01cbe6a4]; ret). |
| `014a9cb0` | TradeCommand_ClearAcceptState::SetGlobal_01cbe6a4 | speculative | Virtual slot 13 (one stack argument, ret 4). Stores the argument into global DAT_01cbe6a4 (mov eax,[esp+4]; mov [01cbe6a4],eax; ret 4). |
| `014a9d20` | TradeCommand_ClearAcceptState::QueueEvent0x2b | speculative | Virtual slot 15 (fastcall, ECX=this, no stack arguments, returns 1). Takes a queue pointer from 012d46a0(this), allocates 0x14 bytes, constructs the block with… |
| `014a9cd0` | — | unknown | Fastcall (ECX=this). Stores TradeCommand_ClearAcceptState::vftable into *this, then calls the base destructor body 01442610 on this; no free. |

**TradeCommand_ClearTrade** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a97f0` | TradeCommand_ClearTrade::TradeCommand_ClearTrade | confirmed | Calls base constructor 01442520 under the cookie prologue, stores TradeCommand_ClearTrade::vftable at this, calls 01442520 again (no argument shown), runs the … |
| `014a9880` | TradeCommand_ClearTrade::GetClassName | confirmed | Virtual slot 11. Returns the address of the string TradeCommand_ClearTrade (018fc2e4). |
| `014a9a50` | TradeCommand_ClearTrade::Read | confirmed | Virtual slot 9 (ECX=this, one stack argument, ret 4). Calls vtbl[4](buf, &buf) (header read); if that returns true, calls 01442780 (TradeCommand base read); if… |
| `014a9a90` | TradeCommand_ClearTrade::Write | confirmed | Virtual slot 10 (ECX=this, one stack argument, ret 4). Calls vtbl[2](buf), then the trace calls 013c26e0 (no-op), calls 014427e0 (TradeCommand base Write), ano… |
| `014a9b60` | TradeCommand_ClearTrade::ScalarDeletingDtor | confirmed | Vtable slot 0 (ECX=this, flag byte on stack). Stores TradeCommand_ClearTrade::vftable, calls base destructor body 01442610, then calls _free(this) when bit 0 o… |
| `014a9bc0` | TradeCommand_ClearTrade::Dump | confirmed | Virtual slot 19 (ECX=this, one stack argument, ret 4). Calls 01442940 (TradeCommand base dump, writes the AccountID and tradeSessionID text) on the argument, t… |
| `014a9be0` | TradeCommand_ClearTrade::AppendTypeName | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument and calls 01241650 (std::string append) with the 25-byte literal Clear Trade Trade Com… |
| `014a99b0` | TradeCommand_ClearTrade::Clone | likely | Virtual slot 17. Allocates 0xc bytes (00d833a6), constructs a ClearTrade through 014a97f0 when the allocation is non-null, then calls vtbl[0x48/4 = slot 18] on… |
| `014a9a20` | TradeCommand_ClearTrade::CopyTo | likely | Virtual slot 18. Calls 01442740 on its destination argument, then dynamic-casts it to TradeCommand_ClearTrade (00d8d382 with NetworkCommand and TradeCommand_Cl… |
| `014a9890` | TradeCommand_ClearTrade::GetSharedValue | speculative | Virtual slot 12. Takes no arguments and returns the 32-bit global DAT_01cbe6a0. |
| `014a98a0` | TradeCommand_ClearTrade::SetSharedValue | speculative | Virtual slot 13. Stores its one stack argument into DAT_01cbe6a0 and returns (ret 4). |
| `014a98c0` | TradeCommand_ClearTrade::ResetVtable | speculative | Stores TradeCommand_ClearTrade::vftable at its argument, sets a local to it, and runs the cookie epilogue 01442610. It touches no member fields. |
| `014a9910` | — | unknown | Virtual slot 15. Runs the EH prologue 012d46a0, allocates a 0x14-byte event (00d833a6, then 012f9eb0 when the allocation succeeds), tags it 0x28 (012f8c70), pu… |

**TradeCommand_EndPostedTrades** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a9400` | TradeCommand_EndPostedTrades::TradeCommand_EndPostedTrades | confirmed | Calls base constructor 01442520 under the cookie prologue, stores TradeCommand_EndPostedTrades::vftable at this and returns this. No member is initialised. |
| `014a9480` | TradeCommand_EndPostedTrades::GetClassName | confirmed | Virtual slot 11. Returns the address of the string TradeCommand_EndPostedTrades (018fc1d4). |
| `014a9670` | TradeCommand_EndPostedTrades::Serialize | confirmed | Virtual slot 10. Calls vtbl[2] on this, calls 013c26e0 with the Starting and Ending texts (a no-op stub, so the texts are not emitted), writes base fields thro… |
| `014a97a0` | TradeCommand_EndPostedTrades::TraceFields | confirmed | Virtual slot 19. Appends End Posted Trades Trade Command followed by a newline (0x20 bytes) to the output string, then calls 01442940 on it. |
| `014a97d0` | TradeCommand_EndPostedTrades::TraceName | confirmed | Virtual slot 20. Appends End Posted Trades Trade Command (0x1f bytes) to the output string and returns. |
| `014a9590` | TradeCommand_EndPostedTrades::Clone | likely | Virtual slot 17. Allocates 0xc bytes (00d833a6), constructs an EndPostedTrades through 014a9400 when the allocation is non-null, then calls vtbl[0x48/4 = slot … |
| `014a9600` | TradeCommand_EndPostedTrades::CopyTo | likely | Virtual slot 18. Calls 01442740 on its destination argument, then dynamic-casts it to TradeCommand_EndPostedTrades (00d8d382 with NetworkCommand and TradeComma… |
| `014a9630` | TradeCommand_EndPostedTrades::Deserialize | likely | Virtual slot 9. Calls vtbl[4] (013aee40) with the stream and the second argument by reference and tests its result; reads base data through 01442780; then retu… |
| `014a96c0` | TradeCommand_EndPostedTrades::Create | likely | Allocates 0xc bytes with 00d833a6; when the allocation succeeds it constructs the object through 014a9400 and returns it, otherwise returns 0. |
| `014a9490` | TradeCommand_EndPostedTrades::GetSharedValue | speculative | Virtual slot 12. Takes no arguments and returns the 32-bit global DAT_01cbe69c. |
| `014a94a0` | TradeCommand_EndPostedTrades::SetSharedValue | speculative | Virtual slot 13. Stores its one stack argument into DAT_01cbe69c and returns (ret 4). |
| `014a94c0` | TradeCommand_EndPostedTrades::ResetVtable | speculative | Stores TradeCommand_EndPostedTrades::vftable at its argument, sets a local to it, and runs the cookie epilogue 01442610. It touches no member fields. |
| `014a9510` | — | unknown | Virtual slot 15. Runs the EH prologue 012d46a0, allocates a 0x14-byte event (00d833a6, then 012f9eb0 when the allocation succeeds), tags it 0x8d (012f8c70), qu… |
| `014a9740` | — | unknown | Scalar deleting destructor (vtable slot 0): stores the class vftable at this, runs the cookie epilogue 01442610, then calls _free(this) when bit 0 of its flag … |

**TradeCommand_EndTradeSession** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a9090` | TradeCommand_EndTradeSession::GetClassName | confirmed | Virtual slot 11. Returns the address of the string TradeCommand_EndTradeSession (018fc0c4). |
| `014a9290` | TradeCommand_EndTradeSession::Serialize | confirmed | Virtual slot 10. Calls vtbl[2] on this, calls 013c26e0 with the Starting and Ending texts (a no-op stub, so the texts are not emitted), writes base fields thro… |
| `014a93c0` | TradeCommand_EndTradeSession::TraceFields | confirmed | Virtual slot 19. Calls 01442940 on the output string, then appends End Trade Session Trade Command followed by a newline (0x20 bytes). |
| `014a93e0` | TradeCommand_EndTradeSession::TraceName | confirmed | Virtual slot 20. Appends End Trade Session Trade Command (0x1f bytes) to the output string and returns. |
| `014a91b0` | TradeCommand_EndTradeSession::Clone | likely | Virtual slot 17. Allocates 0xc bytes (00d833a6), constructs an EndTradeSession through 014a9000 when the allocation is non-null, then calls vtbl[0x48/4 = slot … |
| `014a9220` | TradeCommand_EndTradeSession::CopyTo | likely | Virtual slot 18. Calls 01442740 on its destination argument, then dynamic-casts it to TradeCommand_EndTradeSession (00d8d382 with NetworkCommand and TradeComma… |
| `014a9250` | TradeCommand_EndTradeSession::Deserialize | likely | Virtual slot 9. Calls vtbl[4] (013aee40) with the stream and the second argument by reference and tests its result; reads base data through 01442780; then retu… |
| `014a92e0` | TradeCommand_EndTradeSession::Create | likely | Allocates 0xc bytes with 00d833a6; when the allocation succeeds it constructs the object through 014a9000 and returns it, otherwise returns 0. |
| `014a90a0` | TradeCommand_EndTradeSession::GetSharedValue | speculative | Virtual slot 12. Takes no arguments and returns the 32-bit global DAT_01cbe698. |
| `014a90b0` | TradeCommand_EndTradeSession::SetSharedValue | speculative | Virtual slot 13. Stores its one stack argument into DAT_01cbe698 and returns (ret 4). |
| `014a90d0` | TradeCommand_EndTradeSession::ResetVtable | speculative | Stores TradeCommand_EndTradeSession::vftable at its argument, sets a local to it, and runs the cookie epilogue 01442610. It touches no member fields. |
| `014a9000` | — | unknown | Calls base constructor 01442520 under the cookie prologue, stores TradeCommand_EndTradeSession::vftable at this, calls 01442520 again (no argument shown), runs… |
| `014a9120` | — | unknown | Virtual slot 15. Runs the EH prologue 012d46a0, allocates a 0x14-byte event (00d833a6, then 012f9eb0 when the allocation succeeds), tags it 0x27 (012f8c70), pu… |
| `014a9360` | — | unknown | Scalar deleting destructor (vtable slot 0): stores the class vftable at this, runs the cookie epilogue 01442610, then calls _free(this) when bit 0 of its flag … |

**TradeCommand_FinalAcceptTrade** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a8b20` | TradeCommand_FinalAcceptTrade::TradeCommand_FinalAcceptTrade | confirmed | Calls base constructor 01442520 under the cookie prologue, stores TradeCommand_FinalAcceptTrade::vftable at this, calls 01442520 again (no argument shown), run… |
| `014a8bc0` | TradeCommand_FinalAcceptTrade::GetClassName | confirmed | Virtual slot 11. Returns the address of the string TradeCommand_FinalAcceptTrade (018fbf8c). |
| `014a8e10` | TradeCommand_FinalAcceptTrade::Serialize | confirmed | Virtual slot 10. Calls vtbl[2] on this, calls 013c26e0 with the Starting, Trade Button and Ending texts (a no-op stub, so the texts are not emitted), writes ba… |
| `014a8f50` | TradeCommand_FinalAcceptTrade::TraceFields | confirmed | Virtual slot 19. Appends Final Accept Trade Trade Command followed by a newline (0x21 bytes) to the output string, calls 01442940 on it, then appends Trade But… |
| `014a8fe0` | TradeCommand_FinalAcceptTrade::TraceName | confirmed | Virtual slot 20. Appends Final Accept Trade Trade Command (0x20 bytes) to the output string and returns. |
| `014a8c70` | TradeCommand_FinalAcceptTrade::Handle | likely | Virtual slot 15. Allocates a 0x14-byte event (00d833a6, then 012f9eb0 when the allocation succeeds), tags it 0x29 (012f8c70), pushes this+0x8 and this+0x4 (012… |
| `014a8d10` | TradeCommand_FinalAcceptTrade::Clone | likely | Virtual slot 17. Allocates 0x10 bytes (00d833a6), constructs a FinalAcceptTrade through 014a8b20 when the allocation is non-null, then calls vtbl[0x48/4 = slot… |
| `014a8d80` | TradeCommand_FinalAcceptTrade::CopyTo | likely | Virtual slot 18. Calls 01442740 on its destination argument, dynamic-casts the destination to TradeCommand_FinalAcceptTrade (00d8d382 with NetworkCommand and T… |
| `014a8e70` | TradeCommand_FinalAcceptTrade::Create | likely | Allocates 0x10 bytes with 00d833a6; when the allocation succeeds it constructs the object through 014a8b20 and returns it, otherwise returns 0. |
| `014a8bd0` | TradeCommand_FinalAcceptTrade::GetSharedValue | speculative | Virtual slot 12. Takes no arguments and returns the 32-bit global DAT_01cbe694. |
| `014a8be0` | TradeCommand_FinalAcceptTrade::SetSharedValue | speculative | Virtual slot 13. Stores its one stack argument into DAT_01cbe694 and returns (ret 4). |
| `014a8c00` | TradeCommand_FinalAcceptTrade::ResetVtable | speculative | Stores TradeCommand_FinalAcceptTrade::vftable at its argument, sets a local to it, and runs the cookie epilogue 01442610. It touches no member fields. |
| `014a8db0` | — | unknown | Virtual slot 9. Calls vtbl[4] (013aee40) with the stream and a local by reference, ignoring its result; reads base data through 01442780; reads one integer thr… |
| `014a8ef0` | — | unknown | Scalar deleting destructor (vtable slot 0): stores the class vftable at this, runs the cookie epilogue 01442610, then calls _free(this) when bit 0 of its flag … |

**TradeCommand_IssueTradeRequest** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a8490` | TradeCommand_IssueTradeRequest::GetClassName | confirmed | Virtual slot 11. Returns the address of the string TradeCommand_IssueTradeRequest (018fbe04). |
| `014a86b0` | TradeCommand_IssueTradeRequest::Serialize | confirmed | Virtual slot 10. Calls vtbl[2] on this, calls 013c26e0 with the Starting and Ending texts (a no-op stub, so the texts are not emitted), writes base fields thro… |
| `014a8810` | TradeCommand_IssueTradeRequest::Handle | confirmed | Virtual slot 15. Queues an event tagged 0x21 carrying this+0x8, this+0x4 and this+0xc (012f9eb0 event, 012fa2e0 pushes, 012d3550 queue). When 012de760 finds bo… |
| `014a8a40` | TradeCommand_IssueTradeRequest::TraceFields | confirmed | Virtual slot 19. Calls 01442940 on the output string, appends Issue Trade Request Trade Command followed by a newline (0x22 bytes), then appends Issuing to: %d… |
| `014a8b00` | TradeCommand_IssueTradeRequest::TraceName | confirmed | Virtual slot 20. Appends Issue Trade Request Trade Command (0x21 bytes) to the output string and returns. |
| `014a8590` | TradeCommand_IssueTradeRequest::Clone | likely | Virtual slot 17. Allocates 0x14 bytes (00d833a6), constructs an IssueTradeRequest through 014a83f0 when the allocation is non-null, then calls vtbl[0x48/4 = sl… |
| `014a8600` | TradeCommand_IssueTradeRequest::CopyTo | likely | Virtual slot 18. Calls 01442740 on its destination argument, dynamic-casts the destination to TradeCommand_IssueTradeRequest (00d8d382 with NetworkCommand and … |
| `014a8640` | TradeCommand_IssueTradeRequest::Deserialize | likely | Virtual slot 9. Calls vtbl[4] (013aee40) with the stream and the second argument by reference, reads base data through 01442780, reads an integer into this+0xc… |
| `014a8730` | TradeCommand_IssueTradeRequest::Create | likely | Allocates 0x14 bytes with 00d833a6; when the allocation succeeds it constructs the object through 014a83f0 and returns it, otherwise returns 0. |
| `014a84a0` | TradeCommand_IssueTradeRequest::GetSharedValue | speculative | Virtual slot 12. Takes no arguments and returns the 32-bit global DAT_01cbe690. |
| `014a84b0` | TradeCommand_IssueTradeRequest::SetSharedValue | speculative | Virtual slot 13. Stores its one stack argument into DAT_01cbe690 and returns (ret 4). |
| `014a84d0` | TradeCommand_IssueTradeRequest::ResetVtable | speculative | Stores TradeCommand_IssueTradeRequest::vftable at its argument, sets a local to it, and runs the cookie epilogue 01442610. It touches no member fields. |
| `014a8560` | TradeCommand_IssueTradeRequest::InitOutTriple | speculative | Virtual slot 21 (the class's own override; the other classes use the shared 014426a0 here). Writes 1000000000 to its first out-argument and 0 to its second and… |
| `014a83f0` | — | unknown | Calls base constructor 01442520 under the cookie prologue, stores TradeCommand_IssueTradeRequest::vftable at this, calls 01442520 again (no argument shown), ru… |
| `014a87b0` | — | unknown | Scalar deleting destructor (vtable slot 0): stores the class vftable at this, runs the cookie epilogue 01442610, then calls _free(this) when bit 0 of its flag … |

**TradeCommand_LockTrade** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a7e70` | TradeCommand_LockTrade::TradeCommand_LockTrade | confirmed | Calls base constructor 01442520 under the cookie prologue, stores TradeCommand_LockTrade::vftable at this, calls 01442520 again (no argument shown), runs the c… |
| `014a7f10` | TradeCommand_LockTrade::GetClassName | confirmed | Virtual slot 11. Returns the address of the string TradeCommand_LockTrade (018fbc84). |
| `014a80c0` | TradeCommand_LockTrade::Serialize | confirmed | Virtual slot 10. Calls vtbl[2] on this, calls 013c26e0 with the Starting and Ending texts and the mLock label (a no-op stub, so the texts are not emitted), wri… |
| `014a83a0` | TradeCommand_LockTrade::TraceFields | confirmed | Virtual slot 19. Appends Lock Trade Session Trade Command followed by a newline (0x21 bytes) to the output string, then calls 01442940 on the output string. |
| `014a83d0` | TradeCommand_LockTrade::TraceName | confirmed | Virtual slot 20. Appends Lock Trade Session Trade Command (0x20 bytes) to the output string and returns. |
| `014a7fc0` | TradeCommand_LockTrade::Clone | likely | Virtual slot 17. Allocates 0x10 bytes (00d833a6), constructs a LockTrade through 014a7e70 when the allocation is non-null, then calls vtbl[0x48/4 = slot 18] on… |
| `014a8030` | TradeCommand_LockTrade::CopyTo | likely | Virtual slot 18. Calls 01442740 on its destination argument, then calls 00d8d382 to dynamic-cast it to TradeCommand_LockTrade (NetworkCommand and TradeCommand_… |
| `014a8120` | TradeCommand_LockTrade::Create | likely | Allocates 0x10 bytes with 00d833a6; when the allocation succeeds it constructs the object through 014a7e70 and returns it, otherwise returns 0. |
| `014a7f20` | TradeCommand_LockTrade::GetSharedValue | speculative | Virtual slot 12. Takes no arguments and returns the 32-bit global DAT_01cbe68c. |
| `014a7f30` | TradeCommand_LockTrade::SetSharedValue | speculative | Virtual slot 13. Stores its one stack argument into DAT_01cbe68c and returns (ret 4). |
| `014a7f50` | TradeCommand_LockTrade::ResetVtable | speculative | Stores TradeCommand_LockTrade::vftable at its argument, sets a local to it, and runs the cookie epilogue 01442610. It touches no member fields. |
| `014a8060` | — | unknown | Virtual slot 9. Calls vtbl[4] (013aee40) with the stream and a local by reference, ignoring its result; reads base data through 01442780; reads one integer thr… |
| `014a81a0` | — | unknown | Scalar deleting destructor (vtable slot 0): stores the class vftable at this, runs the cookie epilogue 01442610, then calls _free(this) when bit 0 of its flag … |
| `014a8200` | — | unknown | Virtual slot 15. Runs the EH prologue 012d46a0; when its result is zero it reports through 012f5a60 with the text clientApp, the source name TradeCommandLockTr… |

**TradeCommand_PostedTradeError** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a7980` | TradeCommand_PostedTradeError::TradeCommand_PostedTradeError | confirmed | Calls base constructor 01442520 under the cookie prologue, stores TradeCommand_PostedTradeError::vftable at this, zeroes this+0xc and the byte at this+0x10, an… |
| `014a7a00` | TradeCommand_PostedTradeError::GetClassName | confirmed | Virtual slot 11. Returns the address of the string TradeCommand_PostedTradeError (018fbba4). |
| `014a7c70` | TradeCommand_PostedTradeError::Serialize | confirmed | Virtual slot 10. Calls vtbl[2] on this, writes base fields through 014427e0, writes the integer at this+0xc through 013c23f0, writes the byte at this+0x10 as a… |
| `014a7d90` | TradeCommand_PostedTradeError::TraceFields | confirmed | Virtual slot 19. Appends Posted Trade Error Trade Command followed by a newline (0x21 bytes), calls 01442940 on the output string, then appends Error: %d forma… |
| `014a7e50` | TradeCommand_PostedTradeError::TraceName | confirmed | Virtual slot 20. Appends Posted Trade Error Trade Command (0x20 bytes) to the output string and returns. |
| `014a7ab0` | TradeCommand_PostedTradeError::Handle | likely | Virtual slot 15. Allocates a 0x14-byte event (00d833a6, then 012f9eb0 when the allocation succeeds), tags it 0x90 (012f8c70), pushes the byte at this+0x10 (012… |
| `014a7b50` | TradeCommand_PostedTradeError::Clone | likely | Virtual slot 17. Allocates 0x14 bytes (00d833a6), constructs a PostedTradeError through 014a7980 when the allocation is non-null, then calls vtbl[0x48/4 = slot… |
| `014a7bc0` | TradeCommand_PostedTradeError::CopyTo | likely | Virtual slot 18. Calls 01442740 on its destination argument, dynamic-casts the destination to TradeCommand_PostedTradeError (00d8d382 with NetworkCommand and T… |
| `014a7bf0` | TradeCommand_PostedTradeError::Deserialize | likely | Virtual slot 9. Calls vtbl[4] (013aee40) with the stream and a local by reference, reads base data through 01442780, reads an integer into this+0xc through 013… |
| `014a7cb0` | TradeCommand_PostedTradeError::Create | likely | Allocates 0x14 bytes with 00d833a6; when the allocation succeeds it constructs the object through 014a7980 and returns it, otherwise returns 0. |
| `014a7a10` | TradeCommand_PostedTradeError::GetSharedValue | speculative | Virtual slot 12. Takes no arguments and returns the 32-bit global DAT_01cbe688. |
| `014a7a20` | TradeCommand_PostedTradeError::SetSharedValue | speculative | Virtual slot 13. Stores its one stack argument into DAT_01cbe688 and returns (ret 4). |
| `014a7a40` | TradeCommand_PostedTradeError::ResetVtable | speculative | Stores TradeCommand_PostedTradeError::vftable at its argument, sets a local to it, and runs the cookie epilogue 01442610. It touches no member fields. |
| `014a7d30` | — | unknown | Scalar deleting destructor (vtable slot 0): stores the class vftable at this, runs the cookie epilogue 01442610, then calls _free(this) when bit 0 of its flag … |

**TradeCommand_RemoveCardFromTrade** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a6fb0` | TradeCommand_RemoveCardFromTrade::GetClassName | confirmed | Virtual slot 11. Returns the address of the string TradeCommand_RemoveCardFromTrade (018fb90c). |
| `014a7280` | TradeCommand_RemoveCardFromTrade::Serialize | confirmed | Virtual slot 10. Calls vtbl[2] on this, calls 013c26e0 with six trace texts (a no-op stub, so the texts are not emitted), writes base fields through 014427e0, … |
| `014a73b0` | TradeCommand_RemoveCardFromTrade::~TradeCommand_RemoveCardF… | confirmed | Scalar deleting destructor (vtable slot 0): calls the member cleanup 014a6ff0, then calls _free(this) when bit 0 of its flag argument is set; returns this. |
| `014a73d0` | TradeCommand_RemoveCardFromTrade::Handle | confirmed | Virtual slot 15. Queues an event tagged 0x25 carrying this+0x8, this+0x4, the value passed through 012fa3e0 for this+0xc, and this+0x1c. Then, when the lookup … |
| `014a7830` | TradeCommand_RemoveCardFromTrade::TraceFields | confirmed | Virtual slot 19. Calls 01442940 on the output string, appends Remove Card From Trade Trade Command followed by a newline (0x25 bytes), then appends four format… |
| `014a7960` | TradeCommand_RemoveCardFromTrade::TraceName | confirmed | Virtual slot 20. Appends Remove Card From Trade Trade Command (0x24 bytes) to the output string and returns. |
| `014a7130` | TradeCommand_RemoveCardFromTrade::Clone | likely | Virtual slot 17. Allocates 0x24 bytes (00d833a6), constructs a RemoveCardFromTrade through 014a6f00 when the allocation is non-null, then calls vtbl[0x48/4 = s… |
| `014a71a0` | TradeCommand_RemoveCardFromTrade::CopyTo | likely | Virtual slot 18. Calls 01442740 on its destination argument, dynamic-casts the destination to TradeCommand_RemoveCardFromTrade (00d8d382 with NetworkCommand an… |
| `014a7330` | TradeCommand_RemoveCardFromTrade::Create | likely | Allocates 0x24 bytes with 00d833a6; when the allocation succeeds it constructs the object through 014a6f00 and returns it, otherwise returns 0. |
| `014a6fc0` | TradeCommand_RemoveCardFromTrade::GetSharedValue | speculative | Virtual slot 12. Takes no arguments and returns the 32-bit global DAT_01cbe684. |
| `014a6fd0` | TradeCommand_RemoveCardFromTrade::SetSharedValue | speculative | Virtual slot 13. Stores its one stack argument into DAT_01cbe684 and returns (ret 4). |
| `014a6f00` | — | unknown | Calls base constructor 01442520 under the cookie prologue, stores TradeCommand_RemoveCardFromTrade::vftable at this, calls 013d3300, calls 01442520 a second ti… |
| `014a6ff0` | — | unknown | Stores TradeCommand_RemoveCardFromTrade::vftable at this, calls 013d34d0 (member cleanup, ECX-based) and runs the cookie epilogue 01442610. No free is made her… |
| `014a71f0` | — | unknown | Virtual slot 9. Calls vtbl[4] (013aee40) with the stream and the second argument by reference, reads base data through 01442780, reads integers into this+0x10 … |

**TradeCommand_RequestPostedTrades** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d2c00` | TradeCommand_RequestPostedTrades::TradeCommand_RequestPoste… | confirmed | Constructor: calls 01442520 and stores TradeCommand_RequestPostedTrades::vftable; returns this. |
| `013d2c80` | TradeCommand_RequestPostedTrades::CommandName | confirmed | Virtual slot 11: returns the address of the literal TradeCommand_RequestPostedTrades. |
| `013d2cc0` | TradeCommand_RequestPostedTrades::~TradeCommand_RequestPost… | confirmed | Destructor body: stores the vtable, saves this in a local for the unwind frame and calls 01442610; does not free memory. |
| `013d2d20` | TradeCommand_RequestPostedTrades::Clone | confirmed | Virtual slot 17: allocates 0x0c bytes with 00d833a6, constructs a RequestPostedTrades (013d2c00) in them, then calls slot 18 of this with the new object; retur… |
| `013d2ed0` | TradeCommand_RequestPostedTrades::ScalarDeletingDtor | confirmed | Scalar deleting destructor with the body inlined: stores the vtable, calls 01442610, then calls _free on this when bit 0 of the second argument is set; returns… |
| `013d2f30` | TradeCommand_RequestPostedTrades::DebugDump | confirmed | Virtual slot 19: writes Request Posted Trades Trade Command through 01241650 and calls 01442940 on the argument. |
| `013d2f60` | TradeCommand_RequestPostedTrades::EmitDebugName | confirmed | Virtual slot 20: passes the literal Request Posted Trades Trade Command (length 0x23) to 01241650. |
| `013d2d90` | TradeCommand_RequestPostedTrades::CopyInto | likely | Virtual slot 18: calls 01442740 on the argument and then the dynamic cast 00d8d382 to RequestPostedTrades, discarding the result; copies no fields. |
| `013d2dc0` | TradeCommand_RequestPostedTrades::Deserialize | likely | Virtual slot 9: reads a header through slot 4, calls 01442780 on the argument, then calls slot 5 (+0x14) with the argument; returns 0 on any failed step. |
| `013d2e00` | TradeCommand_RequestPostedTrades::Serialize | likely | Virtual slot 10: calls slot 2 with its argument, writes trace text through 013c26e0 (a no-op), calls 014427e0, writes the closing trace text through 013c26e0, … |
| `013d2e50` | TradeCommand_RequestPostedTrades::Create | likely | Allocates 0x0c bytes with 00d833a6; when non-null constructs a RequestPostedTrades with 013d2c00 and returns it, otherwise returns 0. |
| `013d2c90` | TradeCommand_RequestPostedTrades::GetGlobalDword_01cbde14 | speculative | Virtual slot 12: returns the process-global dword DAT_01cbde14. |
| `013d2ca0` | TradeCommand_RequestPostedTrades::SetGlobalDword_01cbde14 | speculative | Virtual slot 13: stores its argument into the global dword DAT_01cbde14. |

**TradeCommand_SetListenStatus** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a6b30` | TradeCommand_SetListenStatus::TradeCommand_SetListenStatus | confirmed | Calls base constructor 01442520 under the cookie prologue, stores TradeCommand_SetListenStatus::vftable at this, zeroes the byte at this+0xc and returns this. |
| `014a6bb0` | TradeCommand_SetListenStatus::GetClassName | confirmed | Virtual slot 11. Returns the address of the string TradeCommand_SetListenStatus (018fb804). |
| `014a6ce0` | TradeCommand_SetListenStatus::CopyTo | confirmed | Virtual slot 18. Dynamic-casts its destination argument to TradeCommand_SetListenStatus (00d8d382 with NetworkCommand and TradeCommand_SetListenStatus type des… |
| `014a6d90` | TradeCommand_SetListenStatus::Serialize | confirmed | Virtual slot 10. Calls vtbl[2] on this, writes base fields through 014427e0, writes the byte at this+0xc as a 32-bit integer through 013c23f0, then calls vtbl[… |
| `014a6eb0` | TradeCommand_SetListenStatus::TraceFields | confirmed | Virtual slot 19. Appends Set Listen Status Trade Command followed by a newline (0x20 bytes) to the output string, then calls 01442940 on the output string. |
| `014a6ee0` | TradeCommand_SetListenStatus::TraceName | confirmed | Virtual slot 20. Appends Set Listen Status Trade Command (0x1f bytes) to the output string and returns. |
| `014a6c70` | TradeCommand_SetListenStatus::Clone | likely | Virtual slot 17. Allocates 0x10 bytes (00d833a6), constructs a SetListenStatus through 014a6b30 when the allocation is non-null, then calls vtbl[0x48/4 = slot … |
| `014a6d30` | TradeCommand_SetListenStatus::Deserialize | likely | Virtual slot 9. Calls vtbl[4] (013aee40) with the stream and the second argument by reference, ignoring its result; reads base data through 01442780; reads one… |
| `014a6dd0` | TradeCommand_SetListenStatus::Create | likely | Allocates 0x10 bytes with 00d833a6; when the allocation succeeds it constructs the object through 014a6b30 and returns it, otherwise returns 0. |
| `014a6bc0` | TradeCommand_SetListenStatus::GetSharedValue | speculative | Virtual slot 12. Takes no arguments and returns the 32-bit global DAT_01cbe680. |
| `014a6bd0` | TradeCommand_SetListenStatus::SetSharedValue | speculative | Virtual slot 13. Stores its one stack argument into DAT_01cbe680 and returns (ret 4). |
| `014a6bf0` | TradeCommand_SetListenStatus::ResetVtable | speculative | Stores TradeCommand_SetListenStatus::vftable at its argument, sets a local to it, and runs the cookie epilogue 01442610. It touches no member fields. |
| `014a6e50` | — | unknown | Scalar deleting destructor (vtable slot 0): stores the class vftable at this, runs the cookie epilogue 01442610, then calls _free(this) when bit 0 of its flag … |

**TradeCommand_SetTradeAllowed** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a65b0` | TradeCommand_SetTradeAllowed::Serialize | confirmed | Virtual slot 10. Calls vtbl[2] on this, calls 013c26e0 with the Starting and Ending texts (a no-op stub, so the texts are not emitted), writes base fields thro… |
| `014a6690` | TradeCommand_SetTradeAllowed::TraceFields | confirmed | Virtual slot 19. Appends Set Trade Allowed Trade Command followed by a newline (0x20 bytes) to the output string, then calls 01442940 on the output string. |
| `014a66c0` | TradeCommand_SetTradeAllowed::TraceName | confirmed | Virtual slot 20. Appends Set Trade Allowed Trade Command (0x1f bytes) to the output string and returns. |
| `014a6840` | TradeCommand_SetTradeAllowed::GetClassName | confirmed | Virtual slot 11. Returns the address of the string TradeCommand_SetTradeAllowed (018fb784). |
| `014a6940` | TradeCommand_SetTradeAllowed::TradeCommand_SetTradeAllowed | confirmed | Calls base constructor 01442520 under the cookie prologue, stores TradeCommand_SetTradeAllowed::vftable at this, calls 0129d6c0 (flag-tree initialisation), cal… |
| `014a69c0` | TradeCommand_SetTradeAllowed::~TradeCommand_SetTradeAllowed | confirmed | Scalar deleting destructor (vtable slot 0): calls the member cleanup 014a67c0, then calls _free(this) when bit 0 of its flag argument is set; returns this. |
| `014a6250` | TradeCommand_SetTradeAllowed::Handle | likely | Virtual slot 15. Walks the list of entries whose head is at this+0x10 (first node from the head sentinel). For each node it allocates a 0x14-byte event (00d833… |
| `014a67c0` | TradeCommand_SetTradeAllowed::DestructBody | likely | Stores TradeCommand_SetTradeAllowed::vftable at this, calls 0129d560 (flag-tree cleanup) and runs the cookie epilogue 01442610. No free is made here. |
| `014a68e0` | TradeCommand_SetTradeAllowed::Deserialize | likely | Virtual slot 9. Calls vtbl[4] (013aee40) with the stream and the second argument by reference, reads base data through 01442780, reads the flag tree into this+… |
| `014a69e0` | TradeCommand_SetTradeAllowed::Clone | likely | Virtual slot 17. Allocates 0x18 bytes (00d833a6), constructs a SetTradeAllowed through 014a6940 when the allocation is non-null, then calls vtbl[0x48/4 = slot … |
| `014a6a50` | TradeCommand_SetTradeAllowed::CopyTo | likely | Virtual slot 18. Calls 01442740 on its destination argument, dynamic-casts the destination to TradeCommand_SetTradeAllowed (00d8d382 with NetworkCommand and Tr… |
| `014a6ab0` | TradeCommand_SetTradeAllowed::Create | likely | Allocates 0x18 bytes with 00d833a6; when the allocation succeeds it constructs the object through 014a6940 and returns it, otherwise returns 0. |
| `014a6850` | TradeCommand_SetTradeAllowed::GetSharedValue | speculative | Virtual slot 12. Takes no arguments and returns the 32-bit global DAT_01cbe67c. |
| `014a6860` | TradeCommand_SetTradeAllowed::SetSharedValue | speculative | Virtual slot 13. Stores its one stack argument into DAT_01cbe67c and returns (ret 4). |

**TradeCommand_SetupFinalTradeAccept** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a59f0` | TradeCommand_SetupFinalTradeAccept::GetClassName | confirmed | Virtual slot 11. Returns the address of the string TradeCommand_SetupFinalTradeAccept (018fb624). |
| `014a5a30` | TradeCommand_SetupFinalTradeAccept::~TradeCommand_SetupFina… | confirmed | Scalar deleting destructor (vtable slot 0): calls the member cleanup 014a5960, then calls _free(this) when bit 0 of its flag argument is set; returns this. |
| `014a5a50` | TradeCommand_SetupFinalTradeAccept::TraceFields | confirmed | Virtual slot 19. Calls 01442940 on the output string, then appends TradeCommand_SetupFinalTradeAccept followed by a newline (0x23 bytes). |
| `014a5a70` | TradeCommand_SetupFinalTradeAccept::TraceName | confirmed | Virtual slot 20. Appends Setup Final Trade Accept Trade Command (0x26 bytes) to the std::string argument and returns (ret 4). |
| `014a5a90` | TradeCommand_SetupFinalTradeAccept::TradeCommand_SetupFinal… | confirmed | Calls base constructor 01442520 under the cookie prologue, stores TradeCommand_SetupFinalTradeAccept::vftable at this, zeroes this+0x10, +0x14, +0x18, +0x20, +… |
| `014a58b0` | TradeCommand_SetupFinalTradeAccept::CopyTo | likely | Virtual slot 18. Calls 01442740 on its destination argument, then calls 00d8d382 to dynamic-cast it to TradeCommand_SetupFinalTradeAccept (NetworkCommand and T… |
| `014a58e0` | TradeCommand_SetupFinalTradeAccept::Serialize | likely | Virtual slot 10. Calls vtbl[2] on this, calls 013c26e0 with four trace texts (a no-op stub, so the texts are not emitted), writes base fields through 014427e0,… |
| `014a5960` | TradeCommand_SetupFinalTradeAccept::DestructBody | likely | Stores TradeCommand_SetupFinalTradeAccept::vftable at this, calls 012c2de0 with ECX set to this+0x1c, then with ECX set to this+0xc, and finishes through the e… |
| `014a5b40` | TradeCommand_SetupFinalTradeAccept::Clone | likely | Virtual slot 17. Allocates 0x2c bytes (00d833a6), constructs a SetupFinalTradeAccept through 014a5a90 when the allocation is non-null, then calls vtbl[0x48/4 =… |
| `014a5bb0` | TradeCommand_SetupFinalTradeAccept::Create | likely | Allocates 0x2c bytes with 00d833a6; when the allocation succeeds it constructs the object through 014a5a90 and returns it, otherwise returns 0. |
| `014a6150` | TradeCommand_SetupFinalTradeAccept::Deserialize | likely | Virtual slot 9. Calls vtbl[4] (013aee40) with the stream and the second argument by reference, reads base data through 01442780, then, when a gate on EDI (not … |
| `014a5a00` | TradeCommand_SetupFinalTradeAccept::GetSharedValue | speculative | Virtual slot 12. Takes no arguments and returns the 32-bit global DAT_01cbe678. |
| `014a5a10` | TradeCommand_SetupFinalTradeAccept::SetSharedValue | speculative | Virtual slot 13. Stores its one stack argument into DAT_01cbe678 and returns (ret 4). |
| `014a5d10` | — | unknown | Virtual slot 15. Allocates a 0x14-byte event (00d833a6, then 012f9eb0), tags it 0x2a (012f8c70) and pushes this+0x8 into it (012fa2e0). It then walks the found… |

**TradeCommand_StartPostedTrades** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a5430` | TradeCommand_StartPostedTrades::TradeCommand_StartPostedTra… | confirmed | Calls base constructor 01442520 under the cookie prologue, stores TradeCommand_StartPostedTrades::vftable at this, zeroes this+0xc and returns this. |
| `014a54b0` | TradeCommand_StartPostedTrades::GetClassName | confirmed | Virtual slot 11. Returns the address of the string TradeCommand_StartPostedTrades (018fb46c). |
| `014a56d0` | TradeCommand_StartPostedTrades::Serialize | confirmed | Virtual slot 10. Calls vtbl[2] on this, calls 013c26e0 with two trace texts (a no-op stub that returns 1, so the texts are not emitted), writes base fields thr… |
| `014a57a0` | TradeCommand_StartPostedTrades::~TradeCommand_StartPostedTr… | confirmed | Scalar deleting destructor (vtable slot 0): stores the class vftable at this, runs the cookie epilogue 01442610, then calls _free(this) when bit 0 of its flag … |
| `014a5800` | TradeCommand_StartPostedTrades::TraceFields | confirmed | Virtual slot 19. Appends Start Posted Trades Trade Command followed by a newline (0x22 bytes), calls 01442940 on the output string, formats Posted Trade Count:… |
| `014a5890` | TradeCommand_StartPostedTrades::TraceName | confirmed | Virtual slot 20. Appends Start Posted Trades Trade Command (0x21 bytes) to the std::string argument and returns (ret 4). |
| `014a5550` | TradeCommand_StartPostedTrades::Handle | likely | Virtual slot 15. Allocates 0x14 bytes, builds an event object (012f9eb0) when the allocation succeeds, tags it 0x8e (012f8c70), pushes this+0xc into it (012fa2… |
| `014a55e0` | TradeCommand_StartPostedTrades::Clone | likely | Virtual slot 17. Allocates 0x10 bytes (00d833a6), constructs a StartPostedTrades through 014a5430 when the allocation is non-null, then calls vtbl[0x48/4 = slo… |
| `014a5650` | TradeCommand_StartPostedTrades::CopyTo | likely | Virtual slot 18. Calls 01442740 on its destination argument, dynamic-casts the destination to TradeCommand_StartPostedTrades (00d8d382 with NetworkCommand and … |
| `014a5680` | TradeCommand_StartPostedTrades::Deserialize | likely | Virtual slot 9. Calls vtbl[4] (013aee40) with the stream and the second argument by reference, then reads base data through 01442780, reads one integer into th… |
| `014a5720` | TradeCommand_StartPostedTrades::Create | likely | Allocates 0x10 bytes with 00d833a6; when the allocation succeeds it constructs the object through 014a5430 and returns it, otherwise returns 0. |
| `014a54c0` | TradeCommand_StartPostedTrades::GetSharedValue | speculative | Virtual slot 12. Takes no arguments and returns the 32-bit global DAT_01cbe674. |
| `014a54d0` | TradeCommand_StartPostedTrades::SetSharedValue | speculative | Virtual slot 13. Stores its one stack argument into DAT_01cbe674 and returns (ret 4). |
| `014a54f0` | TradeCommand_StartPostedTrades::ResetVtable | speculative | Stores TradeCommand_StartPostedTrades::vftable at its argument, sets a local to it, and runs the cookie epilogue 01442610. It touches no member fields. |

**TradeCommand_StartTradeSync** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a4d70` | TradeCommand_StartTradeSync::TradeCommand_StartTradeSync | confirmed | Calls base constructor 01442520 under the cookie prologue, stores TradeCommand_StartTradeSync::vftable at this, calls 01442520 again (no argument shown in the … |
| `014a4e10` | TradeCommand_StartTradeSync::GetClassName | confirmed | Virtual slot 11. Returns the address of the string TradeCommand_StartTradeSync (018fb2cc). |
| `014a4fb0` | TradeCommand_StartTradeSync::Serialize | confirmed | Virtual slot 10. Calls vtbl[2] on this, calls 013c26e0 with three trace texts (a no-op stub that returns 1, so the texts are not emitted), writes base fields t… |
| `014a5090` | TradeCommand_StartTradeSync::~TradeCommand_StartTradeSync | confirmed | Scalar deleting destructor (vtable slot 0): stores the class vftable at this, runs the cookie epilogue 01442610, then calls _free(this) when bit 0 of its flag … |
| `014a50f0` | TradeCommand_StartTradeSync::Handle | confirmed | Virtual slot 15. Queues an event tagged 0x23 carrying this+0x8 and this+0xc and pushes it (012d3550). When this+0xc and this+0x4 both resolve (012de760), it qu… |
| `014a5380` | TradeCommand_StartTradeSync::TraceFields | confirmed | Virtual slot 19. Calls 01442940 on this with the output string, appends Start Trade Sync Trade Command followed by a newline (0x1f bytes), formats Issued From:… |
| `014a5410` | TradeCommand_StartTradeSync::TraceName | confirmed | Virtual slot 20. Appends Start Trade Sync Trade Command (0x1e bytes) to the std::string argument and returns (ret 4). |
| `014a4ec0` | TradeCommand_StartTradeSync::Clone | likely | Virtual slot 17. Allocates 0x10 bytes (00d833a6), constructs a StartTradeSync through 014a4d70 when the allocation is non-null, then calls vtbl[0x48/4 = slot 1… |
| `014a4f30` | TradeCommand_StartTradeSync::CopyTo | likely | Virtual slot 18. Calls 01442740 on its destination argument, dynamic-casts the destination to TradeCommand_StartTradeSync (00d8d382 with NetworkCommand and Tra… |
| `014a4f60` | TradeCommand_StartTradeSync::Deserialize | likely | Virtual slot 9. Calls vtbl[4] (013aee40) with the stream and the second argument by reference, then reads base data through 01442780, reads one integer into th… |
| `014a5010` | TradeCommand_StartTradeSync::Create | likely | Allocates 0x10 bytes with 00d833a6; when the allocation succeeds it constructs the object through 014a4d70 and returns it, otherwise returns 0. |
| `014a4e20` | TradeCommand_StartTradeSync::GetSharedValue | speculative | Virtual slot 12. Takes no arguments and returns the 32-bit global DAT_01cbe670. |
| `014a4e30` | TradeCommand_StartTradeSync::SetSharedValue | speculative | Virtual slot 13. Stores its one stack argument into DAT_01cbe670 and returns (ret 4). |
| `014a4e50` | TradeCommand_StartTradeSync::ResetVtable | speculative | Stores TradeCommand_StartTradeSync::vftable at its argument, sets a local to it, and runs the cookie epilogue 01442610. It touches no member fields. |

**TradeCommand_TransferPostedTrade** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a45a0` | TradeCommand_TransferPostedTrade::Serialize | confirmed | Virtual slot 10. Calls vtbl[2] on this with the stream, writes base fields through 014427e0, then writes a 32-bit flag with 013c23f0: 1 when this+0xc is null, … |
| `014a4760` | TradeCommand_TransferPostedTrade::GetClassName | confirmed | Virtual slot 11. Returns the address of the string TradeCommand_TransferPostedTrade (018fb1fc). |
| `014a47a0` | TradeCommand_TransferPostedTrade::~TradeCommand_TransferPos… | confirmed | Scalar deleting destructor (vtable slot 0): calls the member cleanup 014a46d0, then calls _free(this) when bit 0 of its flag argument is set; returns this. |
| `014a47c0` | TradeCommand_TransferPostedTrade::TraceFields | confirmed | Virtual slot 19. Appends the text Transfer Posted Trade Trade Command followed by a newline (0x24 bytes) to the std::string passed on the stack, then calls 014… |
| `014a47f0` | TradeCommand_TransferPostedTrade::TraceName | confirmed | Virtual slot 20. Appends the text Transfer Posted Trade Trade Command (0x23 bytes) to the std::string passed on the stack and returns (ret 4). |
| `014a4970` | TradeCommand_TransferPostedTrade::TradeCommand_TransferPost… | confirmed | Runs base constructor 01442520 under the cookie epilogue, stores TradeCommand_TransferPostedTrade::vftable at this, zeroes this+0xc, +0x14, +0x18 and +0x1c, an… |
| `014a4420` | TradeCommand_TransferPostedTrade::Handle | likely | Virtual slot 15. If this+0xc is non-null, builds one event object (00d833a6 then 012f9eb0 when the allocation succeeds), tags it with 0x8f (012f8c70, 013d0f10,… |
| `014a46d0` | TradeCommand_TransferPostedTrade::DestructBody | likely | Stores TradeCommand_TransferPostedTrade::vftable at this, frees the buffer at this+0x14 when non-null, zeroes this+0x14, +0x18 and +0x1c, then makes the call 0… |
| `014a49d0` | TradeCommand_TransferPostedTrade::Clone | likely | Virtual slot 17. Allocates 0x20 bytes (00d833a6), constructs a TransferPostedTrade in it through 014a4970 when the allocation is non-null, then calls vtbl[0x48… |
| `014a4ac0` | TradeCommand_TransferPostedTrade::Create | likely | Allocates 0x20 bytes with 00d833a6 and, when the allocation succeeds, constructs the object through 014a4970 and returns it; returns 0 otherwise. |
| `014a4b50` | TradeCommand_TransferPostedTrade::CopyTo | likely | Virtual slot 18. Calls 01442740 on the destination argument, dynamic-casts it to TradeCommand_TransferPostedTrade (00d8d382 with the NetworkCommand and TradeCo… |
| `014a4d00` | TradeCommand_TransferPostedTrade::Deserialize | likely | Virtual slot 9. Calls vtbl[4] (013aee40, which checks a class id) with the stream and the second argument by reference, then reads base data through 01442780 a… |
| `014a4770` | TradeCommand_TransferPostedTrade::GetSharedValue | speculative | Virtual slot 12. Takes no arguments and returns the 32-bit global DAT_01cbe66c. |
| `014a4780` | TradeCommand_TransferPostedTrade::SetSharedValue | speculative | Virtual slot 13. Stores its one stack argument into DAT_01cbe66c and returns (ret 4). |

**TradeConfirmPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6b240` | TradeConfirmPacket::TradeConfirmPacket 🌐 | confirmed | Constructor used by the trade dispatcher for the final confirm: calls the family constructor FUN_00c6a7b0(0x12), installs TradeConfirmPacket::vftable and store… |
| `00c6bad0` | TradeConfirmPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6c5f0` | TradeConfirmPacket::ReadFields 🌐 | likely | Field reader of the confirm packet: after the shared header read it reads one 32-bit value into the destination's +0xc with the standard short-read handling. |
| `00c6c9d0` | — 🌐 | unknown | Deserialize entry point for the confirm packet: builds the reader frame over (data,size), calls the single-u32 reader 00c6c5f0 and returns 1 when no overflow o… |

**TradeEndSessionPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6b750` | TradeEndSessionPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6a980` | TradeEndSessionPacket::TradeEndSessionPacket 🌐 | likely | Constructor used by the trade dispatcher for a received end-session message: calls the family constructor FUN_00c6a7b0(5), installs TradeEndSessionPacket::vfta… |
| `00c6c530` | TradeEndSessionPacket::ReadFields 🌐 | likely | Field reader of the end-session packet: calls the shared header reader 00c6beb0 with the destination packet in ecx, then reads one 32-bit value into the destin… |
| `00c6c730` | — 🌐 | unknown | Deserialize entry point for the end-session packet: when both the destination packet and the buffer are non-null it lays a reader frame over (param_2, param_2+… |

**TradeInvitePacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6c320` | TradeInvitePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: calls the string-carrying destructor body 00c6bd10 (which releases the member at +0xc and restores th… |
| `00c6c290` | TradeInvitePacket::TradeInvitePacket 🌐 | likely | Constructor used by the trade dispatcher for an incoming invitation: calls the family constructor FUN_00c6a7b0(2), installs TradeInvitePacket::vftable, constru… |
| `00c6cab0` | TradeInvitePacket::ReadFields 🌐 | speculative | Field reader of the invite packet: reads the family/sub-opcode header with 00c6beb0, then reads a length-prefixed string into the member at +0xc through FUN_00… |
| `00c6ccd0` | — 🌐 | unknown | Deserialize entry point for the invite packet: builds the reader frame over (data,size), calls the invite field reader 00c6cab0 and returns 1 when no overflow … |

**TradeInviteReplyPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6a8a0` | TradeInviteReplyPacket::TradeInviteReplyPacket 🌐 | confirmed | Constructor of the client's answer to a trade invitation: calls the family constructor FUN_00c6a7b0(3), installs TradeInviteReplyPacket::vftable, stores the se… |
| `00c6b6d0` | TradeInviteReplyPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6cee0` | TradeInviteReplyPacket::Write 🌐 | likely | Serialises the invite reply into an outgoing buffer: writes the family and sub-opcode header with 00c6cc30, then copies the 32-bit session id at +0xc and the o… |

**TradeItemRemovedPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6b3e0` | TradeItemRemovedPacket::TradeItemRemovedPacket 🌐 | confirmed | Constructor used by the trade dispatcher for an item removed from an offer: calls the family constructor FUN_00c6a7b0(0x13), installs TradeItemRemovedPacket::v… |
| `00c6bb90` | TradeItemRemovedPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6c180` | TradeItemRemovedPacket::ReadFields 🌐 | confirmed | Field reader of the item-removed packet: after the shared header read it reads three consecutive 32-bit values into this+0xc, this+0x10 and this+0x14, each wit… |
| `00c6c960` | — 🌐 | unknown | Deserialize entry point for the item-removed packet: builds the reader frame over (data,size), calls the three-u32 reader 00c6c180 and returns 1 when no overfl… |

**TradeLockPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6aa50` | TradeLockPacket::TradeLockPacket 🌐 | confirmed | Constructor of the client's lock request: calls the family constructor FUN_00c6a7b0(8), installs TradeLockPacket::vftable, stores the session id at +0xc, the o… |
| `00c6b7b0` | TradeLockPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6cf80` | TradeLockPacket::Write 🌐 | speculative | Serialises the lock request: writes the header with 00c6cc30, then copies the 32-bit session id at +0xc, the 32-bit revision at +0x10 and the one-byte locked f… |

**TradeMessagePacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6b4d0` | TradeMessagePacket::TradeMessagePacket 🌐 | confirmed | Constructor used by the trade dispatcher for a trade status message: calls the family constructor FUN_00c6a7b0(0xf), installs TradeMessagePacket::vftable and s… |
| `00c6bbf0` | TradeMessagePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6c630` | TradeMessagePacket::ReadFields 🌐 | likely | Field reader of the trade status message: after the shared header read it reads exactly one byte (sign-extended) into the destination's +0xc, with the same zer… |
| `00c6ca40` | — 🌐 | unknown | Deserialize entry point for the trade status message: builds the reader frame over (data,size), calls the one-byte reader 00c6c630 and returns 1 when no overfl… |

**TradeOfferItemPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6ac90` | TradeOfferItemPacket::TradeOfferItemPacket 🌐 | confirmed | Constructor of the client's offer change: calls the family constructor FUN_00c6a7b0(9), installs TradeOfferItemPacket::vftable, stores the session id at +0xc, … |
| `00c6b8d0` | TradeOfferItemPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6d160` | TradeOfferItemPacket::Write 🌐 | likely | Serialises the offer change: writes the header with 00c6cc30, then copies three 32-bit values - the session id at +0xc, the item value at +0x10 and the resulti… |

**TradeRejectedPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6c3f0` | TradeRejectedPacket::TradeRejectedPacket 🌐 | confirmed | Constructor used by the trade dispatcher for a rejected trade: calls the family constructor FUN_00c6a7b0(0x10), installs TradeRejectedPacket::vftable under an … |
| `00c6c450` | TradeRejectedPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: calls the string-carrying destructor body 00c6bdd0 (member at +0xc) and calls _free on this when bit … |
| `00c6cdb0` | — 🌐 | unknown | Deserialize entry point for the rejected packet: when the packet and buffer are non-null it reads the family/sub-opcode header with 00c6beb0 and then the remai… |

**TradeStartSessionPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6c340` | TradeStartSessionPacket::TradeStartSessionPacket 🌐 | confirmed | Constructor used by the trade dispatcher when a session actually starts: calls the family constructor FUN_00c6a7b0(4), installs TradeStartSessionPacket::vftabl… |
| `00c6c3d0` | TradeStartSessionPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: calls the destructor body 00c6bd70 (string member at +0x18) and calls _free on this when bit 0 of the… |
| `00c6cb40` | TradeStartSessionPacket::ReadFields 🌐 | likely | Field reader of the start-session packet: reads the header with 00c6beb0, then two 32-bit identity values into +0x10 and +0x14 as one bounds-checked pair, then… |
| `00c6cd40` | TradeStartSessionPacket::Read 🌐 | likely | Deserialize entry point for the start-session packet: builds the reader frame over (data,size), calls the start-session field reader 00c6cb40 and returns 1 whe… |

**TradeUpdateAcceptPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6ba70` | TradeUpdateAcceptPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6b160` | TradeUpdateAcceptPacket::TradeUpdateAcceptPacket 🌐 | likely | Constructor used by the trade dispatcher for an accept-state update: calls the family constructor FUN_00c6a7b0(0xe), installs TradeUpdateAcceptPacket::vftable … |
| `00c6c590` | TradeUpdateAcceptPacket::ReadFields 🌐 | likely | Field reader of the accept-state update: after the shared header read it reads one 32-bit value into the destination's +0xc, zeroing it and setting the overflo… |
| `00c6c880` | — 🌐 | unknown | Deserialize entry point for the accept-state update: builds the reader frame over (data,size), calls the single-u32 reader 00c6c590 and returns 1 when no overf… |

**TradeUpdateAddItemPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6af30` | TradeUpdateAddItemPacket::TradeUpdateAddItemPacket 🌐 | confirmed | Constructor used by the trade dispatcher for an item-added update: calls the family constructor FUN_00c6a7b0(0xc), installs TradeUpdateAddItemPacket::vftable, … |
| `00c6b9f0` | TradeUpdateAddItemPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: calls the destructor body 00c6afa0 (which destroys the item member at +0x18 and restores BasePacket::… |
| `00c6ce30` | TradeUpdateAddItemPacket::ReadFields 🌐 | confirmed | Field reader of the add-item update: reads the header with 00c6beb0, then a 32-bit session id into +0xc, a 32-bit item value into +0x10, a byte boolean into +0… |
| `00c6d350` | — 🌐 | unknown | Deserialize entry point for the add-item update: builds the reader frame over (data,size), calls the add-item field reader 00c6ce30 (which reads u32, u32, bool… |

**TradeUpdateCoinCountPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6ae20` | TradeUpdateCoinCountPacket::TradeUpdateCoinCountPacket 🌐 | confirmed | Constructor used by the trade dispatcher for a coin-count update: calls the family constructor FUN_00c6a7b0(0xb), installs TradeUpdateCoinCountPacket::vftable,… |
| `00c6b990` | TradeUpdateCoinCountPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6bf50` | TradeUpdateCoinCountPacket::ReadFields 🌐 | likely | Field reader of the coin-count update packet: calls the shared header reader 00c6beb0 and then reads a 32-bit session id into this+0xc, a 32-bit count into thi… |
| `00c6c7a0` | — 🌐 | unknown | Deserialize entry point for the coin-count update: lays the same reader frame over (data,size), calls the field reader 00c6bf50 and returns 1 when no overflow … |

**TradeUpdateItemCountPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6b040` | TradeUpdateItemCountPacket::TradeUpdateItemCountPacket 🌐 | confirmed | Constructor used by the trade dispatcher for an item-count update: calls the family constructor FUN_00c6a7b0(0xd), installs TradeUpdateItemCountPacket::vftable… |
| `00c6ba10` | TradeUpdateItemCountPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6c000` | — 🌐 | unknown | Field reader of the item-count update packet: after the shared header read it reads a 32-bit session id into +0xc, a 32-bit count into +0x10, a byte boolean in… |
| `00c6c810` | — 🌐 | unknown | Deserialize entry point for the item-count update: builds the reader frame over (data,size), calls the five-field reader 00c6c000 and returns 1 when no overflo… |

**TradeUpdateLockPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6b300` | TradeUpdateLockPacket::TradeUpdateLockPacket 🌐 | confirmed | Constructor used by the trade dispatcher for a lock-state update: calls the family constructor FUN_00c6a7b0(0x11), installs TradeUpdateLockPacket::vftable, sto… |
| `00c6bb30` | TradeUpdateLockPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6c110` | TradeUpdateLockPacket::ReadFields 🌐 | confirmed | Field reader of the lock-update packet: calls the shared header reader 00c6beb0, then reads a 32-bit session id into this+0xc and a one-byte boolean lock flag … |
| `00c6c8f0` | TradeUpdateLockPacket::Read 🌐 | confirmed | Deserialize entry point for the lock-state update: builds the reader frame over (data,size), calls the u32-plus-bool reader 00c6c110 and returns 1 when no over… |

**(no class)** (40)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6bd10` | PacketStringDtor_00c6bd10 | confirmed | Destructor body: destroys the SoeUtil string member at this+0xc through FUN_0070a790 under an SEH frame, then writes BasePacket::vftable at +0. |
| `00c6bd70` | PacketStringDtor_00c6bd70 | confirmed | Destructor body: destroys the SoeUtil string member at this+0x18 through FUN_0070a790 under an SEH frame, then writes BasePacket::vftable at +0. |
| `00c6bdd0` | PacketStringDtor_00c6bdd0 | confirmed | Destructor body: destroys the SoeUtil string member at this+0xc through FUN_0070a790 under an SEH frame, then writes BasePacket::vftable at +0. |
| `00c6b5d0` | CheckIdThenCall_00c6b5d0 | likely | Returns true when this+4 is not 0xffffffff, and in that case calls FUN_00cf6f80(param_2) on the object at this+0xa0 before returning; when the id is absent it … |
| `00c6b5f0` | CallPartnerObjectOrZero_00c6b5f0 | likely | Twenty-byte body that returns 0 when this+4 equals 0xffffffff; otherwise it loads the object at this+0xa0 and tail-jumps to FUN_00cf6240 on it (the disassembly… |
| `00c6b610` | CallPartnerObject_00c6b610 | likely | Eleven-byte jump thunk: loads ecx from this+0xa0 and jumps to FUN_00cf6260, so it forwards the call to the partner helper object without any guard of its own. |
| `00c6be30` | SessionTrackerDtor_00c6be30 | likely | Destructor body of the trade session tracker object: frees the heap object held at +0xa0 (its destructor FUN_00cf65b0 then _free) and clears the field, then de… |
| `00c6beb0` | ReadFamilyAndSubOpcode 🌐 | likely | First field reader of every trade packet: with this being the packet and the single stack argument the reader cursor, it bounds-checks two bytes against reader… |
| `00c6cc30` | WriteFamilyAndSubOpcode 🌐 | likely | Writes the trade packet header into an outgoing byte buffer: it takes the two low bytes of the dword at this+4 (the family value) and copies them into the writ… |
| `00c6d420` | SendClientGatewayPacket_00c6d420 🌐 | likely | Client-to-server send path for one trade request: it opens an allocator scope with FUN_008d4a40(0x10000000), serialises the packet body into a stack writer wit… |
| `00c6db40` | SendTradeInviteRefusalAndClear 🌐 | likely | When this+4 is a live session id and this+8 is zero, it constructs a TradeInviteReplyPacket with that id and accepted=0, submits it through the gateway path 00… |
| `00c6dbe0` | SendTradeInviteReply 🌐 | likely | Sends the answer to an open invitation: when this+4 holds a live session id it zeroes the pending flag at this+8, constructs a TradeInviteReplyPacket with (id,… |
| `00c6dc70` | SendTradeLockRequest 🌐 | likely | When this+4 holds a live session id it constructs a TradeLockPacket from (id, this+8, param_2) through 00c6aa50 and submits it through the gateway path 00c6d53… |
| `00c6ddf0` | SendTradeOfferItemRequest 🌐 | likely | When this+4 holds a live session id and the item pointer argument is non-null, it resolves the item through FUN_00cf6c90(param_2,param_3), constructs a TradeOf… |
| `00c6de90` | SendTradeCoinCountRequest 🌐 | likely | When this+4 holds a live session id it constructs a TradeCoinCountPacket from (id, param_2) through 00c6ad60, submits it through the gateway path 00c6d970 and … |
| `014a4b20` | RegisterFactory0x193 | likely | Pushes the creator 014a4ac0 and the id 0x193, calls 013ce130 (no arguments read; returns a registry pointer in EAX), then calls 013ce230 with ECX set to that p… |
| `014a5070` | RegisterFactory0x110 | likely | Pushes the creator 014a5010 and the id 0x110, calls 013ce130 (no arguments read; returns a registry pointer in EAX), then calls 013ce230 with ECX set to that p… |
| `014a5780` | RegisterFactory0x192 | likely | Pushes the creator 014a5720 and the id 0x192, calls 013ce130 (no arguments read; returns a registry pointer in EAX), then calls 013ce230 with ECX set to that p… |
| `014a5c10` | RegisterFactory0x9c | likely | Pushes the creator 014a5bb0 and the id 0x9c, calls 013ce130 (no arguments read; returns a registry pointer in EAX), then calls 013ce230 with ECX set to that po… |
| `014a6b10` | RegisterFactory0x15c | likely | Pushes the creator 014a6ab0 and the id 0x15c, calls 013ce130 (no arguments read; returns a registry pointer in EAX), then calls 013ce230 with ECX set to that p… |
| `014a6e30` | RegisterFactory0x1af | likely | Pushes the creator 014a6dd0 and the id 0x1af, calls 013ce130 (no arguments read; returns a registry pointer in EAX), then calls 013ce230 with ECX set to that p… |
| `014a7390` | RegisterFactory0x97 | likely | Pushes the creator 014a7330 and the id 0x97, calls 013ce130 (no arguments read; returns a registry pointer in EAX), then calls 013ce230 with ECX set to that po… |
| `014a7d10` | RegisterFactory0x195 | likely | Pushes the creator 014a7cb0 and the id 0x195, calls 013ce130 (no arguments read; returns a registry pointer in EAX), then calls 013ce230 with ECX set to that p… |
| `014a8180` | RegisterFactory0x111 | likely | Pushes the creator 014a8120 and the id 0x111, calls 013ce130 (no arguments read; returns a registry pointer in EAX), then calls 013ce230 with ECX set to that p… |
| `014a8790` | RegisterFactory0x93 | likely | Pushes the creator 014a8730 and the id 0x93, calls 013ce130 (no arguments read; returns a registry pointer in EAX), then calls 013ce230 with ECX set to that po… |
| `014a8ed0` | RegisterFactory0x9b | likely | Pushes the creator 014a8e70 and the id 0x9b, calls 013ce130 (no arguments read; returns a registry pointer in EAX), then calls 013ce230 with ECX set to that po… |
| `014a9340` | RegisterFactory0x99 | likely | Pushes the creator 014a92e0 and the id 0x99, calls 013ce130 (no arguments read; returns a registry pointer in EAX), then calls 013ce230 with ECX set to that po… |
| `014a9720` | RegisterFactory0x190 | likely | Pushes the creator 014a96c0 and the id 400 (0x190), calls 013ce130 (no arguments read; returns a registry pointer in EAX), then calls 013ce230 with ECX set to … |
| `00c6dcf0` | SendTradeAcceptRequest 🌐 | speculative | When this+4 holds a live session id it constructs a TradeAcceptPacket from (id, this+8) through 00c6ab10, submits it through the gateway path 00c6d640 and dest… |
| `00c6b5b0` | — | unknown | Scalar deleting destructor of the small helper object that the trade tracker allocates at +0xa0: it calls that object's destructor FUN_00cf65b0 and, when bit 0… |
| `00c6c470` | — | unknown | Constructor of the client-side trade session tracker: stores the owner pointer at +0, sets the session id at +4 to 0xffffffff, zeroes +8, copies the global def… |
| `00c6d530` | — 🌐 | unknown | Same client-gateway send path as 00c6d420 but serialising through FUN_00c6cf80 (the lock request body): allocator scope, body writer, tunneled world packet con… |
| `00c6d640` | — 🌐 | unknown | Same client-gateway send path serialising through FUN_00c6d060 (the accept request body): allocator scope, body writer, tunneled world packet constructor FUN_0… |
| `00c6d750` | — 🌐 | unknown | Same client-gateway send path serialising through FUN_00c6d100 (the cancel request body): allocator scope, body writer, tunneled world packet constructor FUN_0… |
| `00c6d860` | — 🌐 | unknown | Client-gateway send path for the offer-item request, differing from the first three in its packet constructors: it opens the allocator scope, serialises the bo… |
| `00c6d970` | — 🌐 | unknown | Client-gateway send path for the coin-count request: allocator scope, body writer FUN_00c6d240, client-tunneled packet constructor FUN_0089adc0, FUN_00925640 w… |
| `00c6dd70` | — 🌐 | unknown | When this+4 holds a live session id it constructs a TradeCancelPacket from that id through 00c6abd0, submits it through the gateway path 00c6d750, destroys the… |
| `013d0650` | — | unknown | Thiscall setter: stores its second argument at this+0x0c. |
| `013d0af0` | — | unknown | Thiscall setter: stores its second argument at this+0x0c. |
| `013d29b0` | — | unknown | Thiscall copy: copies the dwords at +0x08, +0x0c, +0x48, +0x64 and +0xa0 from the argument, assigns the strings at +0x10, +0x2c, +0x68, +0x84 and +0xa4 with ba… |

### `ui.trade`

**TradeInventoryDataSource** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf5ce0` | TradeInventoryDataSource::Dtor | confirmed | Restores the TradeInventoryDataSource vtables at the object and base offsets, clears shared state DAT_01bf2070, then destroys its embedded member through 00cb1… |
| `00cf5d60` | TradeInventoryDataSource::GetFieldName | confirmed | Returns Available Count for field id 0x1b and No Trade Message for 0x1c; delegates other field ids to 00cb09d0. |
| `00cf6170` | TradeInventoryDataSource::AdjustThisAndJump_00cf61c0 | confirmed | The 11-byte body subtracts 0x254 from ECX and jumps to 00cf61c0; this is a this-adjusting jump thunk to the scalar deleting destructor. |
| `00cf61c0` | TradeInventoryDataSource::ScalarDeletingDtor | confirmed | Calls the TradeInventoryDataSource destructor, frees this when the low bit of the flag is set, and returns the object pointer otherwise. |
| `00cf6ed0` | TradeInventoryDataSource::TradeInventoryDataSource | confirmed | Initializes the datasource base with BaseClient.Trade.Inventory, installs four TradeInventoryDataSource vtable pointers, and stores its constructor arguments. |
| `00cf76e0` | TradeInventoryDataSource::GetCell | confirmed | Handles trade inventory columns, returning counts and item values or formatting ItemNoTrade, ItemEquipped, and ItemEquippedOnPet warnings. |
| `00cf7860` | TradeInventoryDataSource::Initialize | confirmed | Clears trade state, creates partner and local item datasources plus an inventory datasource, then registers them under BaseClient.Trade.PartnerItems, MyItems, … |
| `00cf61e0` | TradeInventoryDataSource::UpdateOtherCoin | likely | When the update flag is false, sets trade state bits to value 2 and writes the supplied coin count at the trade object+0x50 when changed, calls update helper 0… |
| `00cf6b10` | — | unknown | Gets the active player identifier, checks it against the datasource filter, then returns whether the supplied item range has a positive count. |

**TradeItemDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf5ca0` | TradeItemDataSource::LogUpdatePlayerOrOther | confirmed | Checks the byte at this+0x2e8 and logs either TradeHandler:OnUpdatePlayer or TradeHandler:OnUpdateOther. |
| `00cf5cd0` | TradeItemDataSource::ClearFlag_2d0 | confirmed | Clears the byte at this+0x2d0. |
| `00cf6180` | TradeItemDataSource::GetField_2cc | confirmed | Returns the 32-bit field at this+0x2cc. |
| `00cf7200` | TradeItemDataSource::GetCell | confirmed | Resolves a displayed trade item and dispatches by column index to write its ID, count, item metadata, flags, or related values into the UI cell. |
| `00cf70d0` | TradeItemDataSource::TradeItemDataSource | speculative | Initializes its datasource base, installs four TradeItemDataSource vtable pointers, stores a flag, and initializes the embedded item list. |
| `00cf7170` | TradeItemDataSource::DestructorAdjustor | speculative | Subtracts 0x254 from the this pointer, then jumps to 00cf71e0; it is an adjustor thunk, not the destructor body. |
| `00cf7180` | TradeItemDataSource::Cleanup | speculative | Cleans the embedded item list through FUN_00cf6e60, then destroys the datasource base state. |
| `00cf71e0` | TradeItemDataSource::ScalarDeletingDtor | speculative | Calls the class cleanup and frees the object when the delete flag's low bit is set. |

**UI::TradeLobbyScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f7b50` | UI::TradeLobbyScreen::TradeLobbyScreen | confirmed | Constructor (fastcall, ECX = this): calls UI::TradeLobbyScreenBase::Ctor 013fe810, then stores UI::TradeLobbyScreen::vftable at *this. |
| `012f7bb0` | UI::TradeLobbyScreen::DtorBody | confirmed | Non-deleting destructor body: stores UI::TradeLobbyScreen::vftable at *this, then calls UI::TradeLobbyScreenBase::Dtor 013fe870. |
| `012f7c00` | UI::TradeLobbyScreen::~TradeLobbyScreen | confirmed | Scalar deleting destructor (vtable slot [0]): stores UI::TradeLobbyScreen::vftable, calls the destructor body 013fe870, then calls _free on this when bit 0 of … |

**UI::TradeLobbyScreenBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013fe810` | UI::TradeLobbyScreenBase::Ctor | confirmed | Fastcall constructor: calls base constructor 012f8420, then stores UI::TradeLobbyScreenBase::vftable at this. |
| `013fe870` | UI::TradeLobbyScreenBase::Dtor | confirmed | Fastcall destructor body: stores UI::TradeLobbyScreenBase::vftable, then calls base destructor body 012f8480. |
| `013fe8c0` | UI::TradeLobbyScreenBase::ScalarDeletingDtor | confirmed | Virtual slot 0 of UI::TradeLobbyScreenBase::vftable: stores the vtable, calls base destructor body 012f8480, frees the object when the flag argument has bit 0 … |

**UI::TradeScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f7c60` | UI::TradeScreen::TradeScreen | confirmed | Constructor (fastcall, ECX = this): calls UI::TradeScreenBase::Ctor 013fe920, then stores UI::TradeScreen::vftable at *this. |
| `012f7cc0` | UI::TradeScreen::DtorBody | confirmed | Non-deleting destructor body: stores UI::TradeScreen::vftable at *this, then calls UI::TradeScreenBase::Dtor 013fe980. |
| `012f7d10` | UI::TradeScreen::~TradeScreen | confirmed | Scalar deleting destructor (vtable slot [0]): stores UI::TradeScreen::vftable, calls the destructor body 013fe980, then calls _free on this when bit 0 of the f… |

**UI::TradeScreenBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013fe920` | UI::TradeScreenBase::Ctor | confirmed | Fastcall constructor: calls base constructor 012f8420, then stores UI::TradeScreenBase::vftable at this. |
| `013fe980` | UI::TradeScreenBase::Dtor | confirmed | Fastcall destructor body: stores UI::TradeScreenBase::vftable, then calls base destructor body 012f8480. |
| `013fe9d0` | UI::TradeScreenBase::ScalarDeletingDtor | confirmed | Virtual slot 0 of UI::TradeScreenBase::vftable: stores the vtable, calls base destructor body 012f8480, frees the object when the flag argument has bit 0 set, … |

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c05ce0` | RegisterTradeModule | confirmed | Thin registration wrapper (29 bytes: load the 'Trade' module-name pointer, push nup=0, the entry table pointer 0x01829c48 and the Lua state, call FUN_00e39e40,… |
| `00c06020` | Trade_GetPartnerGuid | likely | Formats the trade partner's two guid dwords (*(iVar1+0x10), *(iVar1+0x14)) into a local IString with the SoeUtil format template DAT_0175b2f8 via FUN_00402fd0,… |
| `00cf7050` | AllocateTradeDatasourceRow | speculative | Calls a virtual allocator, constructs an object from two arguments when allocation succeeds, then inserts it in a linked collection. |
| `00c05e20` | — | unknown | Lua 'OfferCoins' handler: checks the single argument is numeric (FUN_00e3a5f0(L,-1)), reads it (FUN_00e3a7b0), resolves the local player/trade object through F… |
| `00c05e90` | — | unknown | Lua 'SelectInventoryCategory' handler: checks the single argument is numeric (FUN_00e3a5f0(L,-1)), reads it as an integer (FUN_00e3a770 + FUN_00d83c90), resolv… |
| `00c05f00` | — | unknown | Builds a stack SoeUtil::IString, resolves the trade object via FUN_008a6650, asks FUN_00c6b5d0 whether a partner name exists; when true it pushes that string (… |
| `00cf6f80` | — | unknown | Reads flags at object offsets 0x4c and 0x50, then assigns either the selected string pointer or null through FUN_00402ce0. |

## Source shards

[[S0287 00a0c3a0-00a13140|S0287]] (1), [[S0383 00c05780-00c0db00|S0383]] (5), [[S0399 00c68550-00c6de90|S0399]] (91), [[S0400 00c6df10-00c73b60|S0400]] (1), [[S0424 00cf1430-00cf68d0|S0424]] (14), [[S0425 00cf69c0-00cfc480|S0425]] (15), [[S0679|S0679]] (13), [[S0689|S0689]] (6), [[S0730|S0730]] (55), [[S0738|S0738]] (6), [[S0751|S0751]] (14), [[S0770|S0770]] (203), [[S0771|S0771]] (96)
