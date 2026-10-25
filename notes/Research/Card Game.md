---
title: Card Game (TCG and Treasure War)
updated: 2026-10-17
status: research and server design
---

# Card Game (TCG and Treasure War)

Task T031. The phrase "Battle Cards" does not occur in the shipped
`Resources` tables or the TCG analysis artifacts. The client names two card
games:

| Game | Client identity | Runtime | Separate service |
|---|---|---|---|
| Free Realms Trading Card Game (TCG) | 124 `MiniGameData` TYPE 16 rows; TYPE 3 row 11 is also labelled "FR Trading Card Game" | `tcg/FreeRealmsTCG.dll`, a Qt 4 module composited into the main window | Yes. DLL defaults to `lvsccg-01-02.station.sony.com:17782` (gateway) and `lvsccg-01-01.station.sony.com:17783` |
| Treasure War | one TYPE 23 row, id 727 | the card client embedded in `FreeRealms.exe`, with Scaleform screens and `CTP::Game` / `CWGame` | Yes. In-exe defaults to the paired hosts on 21782/21783 |

The two games share serializer and engine ancestry. They do not share a lobby,
a collection, or a server process. The main game's family 39 is only the
launch envelope; card actions do not ride on family 39 after the card client
starts.

This note treats the TYPE 16 TCG as the first build target. It is the game used
by the Card Duelist job and the 124 NPC duel rows. Treasure War is a separate
track and is deferred until the TCG launch path is understood end to end.

## Evidence boundary

Source precedence is client bytes and client Lua, then reviewed video, then
the wiki. No OSFR, Legacy, emulator, or server code is used as a protocol
source. Named functions retain their native-map confidence. DLL functions
without accepted labels remain `FUN_<address>`; their roles are stated only
where the decompiled imports, call chain, or class string supports them.

The Build Guide's top-level DLL section was written before the later DLL
analysis. [[Research/TCG Client Protocol]] W61/W61b supersedes its "everything else is
unknown" boundary for the numeric factory map, framing, login command IDs,
native localization stream, and outer collection/archetype layers. The Build
Guide remains the best source for the launch bridge and Treasure War UI. Its
network-command 546/575 rows describe the separate in-executable Treasure War
client; they are not evidence that those IDs are registered by `FreeRealmsTCG.dll`.

Build Guide anchors used here:

- [[Build Guides/Gameplay/Trading Card Game|Trading Card Game]] overview lines
  18-40; launch lines 69-76; data lines
  111-153; network lines 154-182; server requirements lines 183-213.
- [[Build Guides/Gameplay/Minigame Framework|Minigame Framework]] type registry
  and launch/routing section around lines 129-185.
- [[Build Guides/Gameplay/Matchmaking and Lobby|Matchmaking and Lobby]]
  card-client lobby at lines 125-149.
- [[Build Guides/Content/Jobs/Card Duelist|Card Duelist]] job overview at lines
  17-26 and opponent rows at 202-240.

The preserved working client is now
`../lab/working/OSFR.Client/OSFR Client`. The older
`../lab/runs/client-baseline-20261008T015138Z/client` audit path now contains only
logs, so this note cites the working copy and verifies it by hash. The DLL hash
still matches the W61/W61b target:

    ../lab/working/OSFR.Client/OSFR Client/tcg/FreeRealmsTCG.dll
    SHA-256 22f0e8f9dfc9cb398898e227fb9744b6da1989e8dcfff7b5fae85e4bd162cd8b
    8,990,720 bytes

`tcg/frtcg.ini` was not opened. No player-chosen name, account name, or chat
text is included in this note or its test design.

Paths beginning `../lab/` are relative to this private worktree root, not to
this note's directory. The lab is the sibling analysis directory named by
[[Project Map]].

## Client and UI flow

### Entry points

1. The main HUD card button opens the utility card panel on Windows. On Apple
   it calls `TradingCardGame.StartTreasureWar()` directly.
2. The utility panel has two script callbacks:
   - `gotoTCG` calls `TradingCardGame.StartTutorial()`.
   - `gotoCardBattles` calls `TradingCardGame.StartTreasureWar()`.
3. The welcome-screen `TCGLobby` / `gotoTCG` links also call
   `StartTutorial()`.
4. Equipping a card-store item arms `tcgItemEquipped` or `twItemEquipped`;
   the store's completion handler launches the matching game.

The relevant client Lua is
`../lab/decompiled/client-ui/ScriptsBase.lua`:

- HUD branch and utility panel: lines 12853-12862.
- utility callbacks: lines 13710-13714.
- welcome-screen branch: lines 14537 and 14546.
- store completion: lines 19036 and 19041.
- `TradingCardGameHandler`: lines 54329-54405.
- `OnLoadFail`, portal return, and `ui_ccg_ServerError`: lines 54402-54405.

`StartTutorial()` and `StartTreasureWar()` are activity-join requests. The
sender is `00be7d80`; it looks up the activity id and sends through
`PacketTunneledClientWorldPacket` (gateway opcode 6) for entry type 1 or
`PacketTunneledClientPacket` (gateway opcode 5) for entry type 2. The inner
activity-join body is 167/1/2 (`activity_portal.rs`: header, u32 activity id,
u32 client clock offset). The two hashed keys are Lua globals, resolved by T504:
`0x6d54a188` is `Tcg.ActivityId` and `0xbfd77cb0` is `Scg.ActivityId`
(registered by PlayClient init `0092e3bc`); `StartTutorial` (`00c04970`) and
`StartTreasureWar` (`00c049f0`) scan the settings list for them and fall back
to 1049.0 and 1078.0. FreeRealms-Legacy's activity ids 1049 (tutorial, MiniGameData
537) and 1078 (Treasure War, MiniGameData 727) are those defaults, so they are the
original ids. T504 ships the 16 category-11 rows and Treasure War from
`data/content/card_game.json` and answers Play with a refusal until a card
service exists.

### Main-game launch handshake

The client creates a mini-game state with type 16 (DLL TCG) or type 23
(Treasure War). Family 39 uses this header on every packet:

    i16 opcode = 39
    i8  sub-opcode
    i32 state/instance id
    i32 second header id (reader slot +0x14)
    i32 third header id (reader slot +0x10)

Observed card-specific/generic flow:

1. The client requests game start with generic `39/5`
   `[instance, -1, -1]` (the writer substitutes the current instance).
2. The server creates and describes the state with generic `39/16`; this must
   arrive before most state-dependent messages.
3. The server announces the start with generic `39/17`. Encounter-flagged
   states use `41/2` instead.
4. For Treasure War, `ClientSimpleCardGameManager` loads
   `SimpleCardAssets.txt`, reaches its state 5, and sends `39/0x25`.
5. The server returns `39/0x28` with two `i32` halves of a 64-bit launch
   ticket. TYPE 16 stores those halves for the DLL launch; the exact gating
   between ticket receipt and DLL load remains unresolved. TYPE 23 passes a
   third header id to its launch handler; a positive value selects campaign
   mode and modes 2-4 select tournament, collection, or matchmaking. Reading
   the third value as a campaign mission id is likely, not proven.
6. `39/0x29` returns the Lua handler to the portal and displays the server
   error. The TYPE 16 manager owns the asset download and DLL load, but the
   original ordering and timeout are not fully decoded.
7. Closing the card client sends `39/6`. Generic `39/18`, `39/47`, and `39/19`
   still drive the shared results sequence; `39/7` is the automatic results
   acknowledgement and `26/42` is the player's Close.

`RequestTCGChallengePacket` is `39/0x26`, with one id in the header's state
slot. The constructor and sender exist, but no static caller of the sender was
found. The server should validate the packet if it arrives, but must not make
the entry flow depend on it until its trigger is recovered.

### What the player sees after launch

The TYPE 16 path leaves the main HUD and opens `TradingCardGameHandler` in the
mini-game z-layer. Its TCG screens cover the lobby, campaign, collection,
decks, matchmaking, tournaments, delivery, redemption, and trade. The main
window hides HUD, dock, and chat. The DLL owns the card UI surfaces; the main
exe composites them and forwards mouse, wheel, keyboard, focus, avatar, and
portrait traffic.

The TYPE 23 path uses the in-exe card client and `TCCW_*.swf` screens. Its
Home/Campaign/Collection/Tournaments/Profile/Game/Delivery/Redemption/Rewards/
Imbue Holocrons/Treasure Trove/Event Calendar/Rules/Friends flows are separate
from the TYPE 16 lobby.

The Card Duelist content is server/world content around the TCG: quest offers,
NPC gates, row selection, prize-card sentences, and Card Duelist progression.
It is not part of the card service wire protocol. See
[[Build Guides/Content/Jobs/Card Duelist|Card Duelist]].

## Opcode and command table

### Main game, family 39

This is the launch and shared-results layer only.

| ID | Decimal | Name / effect | Direction | Payload and status |
|---|---:|---|---|---|
| `39/5` | 39/5 | `RequestStartMiniGame` | C>S | header only (15 bytes); the three header integers are `[instance, -1, -1]` (sender `009be7d0`, writer `009bdc60`); confirmed |
| `39/6` | 39/6 | leave card/mini-game | C>S | header; sent when the card client closes |
| `39/7` | 39/7 | results acknowledgement | C>S | header; generic framework |
| `39/16` | 39/16 | mini-game info/create state | S>C | state definition, goals, rewards and preload fields; field meanings partly inferred |
| `39/17` | 39/17 | ordinary game started | S>C | header; encounter states use 41/2 |
| `39/18` | 39/18 | game over | S>C | won byte, stars, elapsed time, total score |
| `39/19` | 39/19 | remove state / show results | S>C | triggers the shared result dialog when the state flag allows it |
| `39/21` | 39/21 | stars earned | S>C | star increment |
| `39/37` (`0x25`) | 39/37 | card client ready | C>S | header only; confirmed on the Treasure War manager path |
| `39/38` (`0x26`) | 39/38 | TCG challenge request | C>S | one header id; sender exists, trigger unknown |
| `39/40` (`0x28`) | 39/40 | card-game start / launch ticket | S>C | header plus two `i32`; TYPE 23 also reads the third header id |
| `39/41` (`0x29`) | 39/41 | card load failure | S>C | header; Lua `OnLoadFail` |
| `39/47` | 39/47 | end scores | S>C | counted score rows |

Header direction and field ownership are from the client readers and writers,
not a captured server transcript. The client's own writer for the generic
results acknowledgement is known; the original server's ordering relative to
the visible results page is reconstructed until native acceptance.

### Separate TCG card service

These are class/command ids on the DLL's direct TCP link. They are not family
39 sub-opcodes. The curated map in
`../lab/content/tcg/analysis/message-table.json` contains 60 EXE/DLL
cross-referenced rows. All 60 numeric ids occur in the DLL's 499-row factory
registration table. 22 class names match exactly, 22 are shared
`GameCommand` base registrations, and 16 names remain unresolved.

| ID | Class / operation | Direction | Decoded or useful field detail |
|---:|---|---|---|
| 414 (`0x19e`) | `GatewayCommand_GetConnectionServer` | C>S | unrouted frame; receives SessionID, Username, Host, Port, Success, Error Type; result event `0x95`/`0x96` |
| 411 (`0x19b`) | `LoginCommand_SendSessionID` | C>S | v7; SessionID, Platform, challenge key, parent-session flag, login client, locale, character name/id, ticket, int map, flag |
| 410 (`0x19a`) | `LoginCommand_SendPassword` | C>S | v7; manual credential branch |
| 78 (`0x4e`) | `LoginCommand_GetChallenge` | C>S | manual login route |
| 79 (`0x4f`) | `LoginCommand_SendChallenge` | S>C | v2; inbound challenge; client computes MD5/SHA-1 answer |
| 80 (`0x50`) | `LoginCommand_RespondToChallenge` | C>S | challenge answer |
| 81 (`0x51`) | `LoginCommand_Results` | S>C | v6; success, explanation, account id, server id, session string, restrictions |
| 103 (`0x67`) | `TradingCard::Lobby` | S>C | v3 lobby snapshot: five child lists, roles, groups, lobby type, deck, tournament fields |
| 92 (`0x5c`) | `LobbyCommand_Join` | C>S | v5; group id, role, string; body has further version-gated fields |
| 93 (`0x5d`) | `LobbyCommand_Leave` | C>S | v3; body not decoded |
| 99 (`0x63`) | `MatchCommand_SetReady` | C>S | v1 bool |
| 100 (`0x64`) | `MatchCommand_SetBid` | C>S | v1 int |
| 101 (`0x65`) | `MatchCommand_SelectDeck` | C>S | v2 string, bool, nested deck object |
| 102 (`0x66`) | `MatchCommand_SelectStartPosition` | C>S | int and int list |
| 116 (`0x74`) | `MatchCommand_LaunchGame` | C>S | base fields; client then constructs its game object |
| 124 (`0x7c`) | `MatchCommand_SetMatchState` | S>C likely | one int state |
| 294 (`0x126`) | `MatchCommand_SetBulkMatchState` | S>C likely | group-id and match-state containers |
| 295 (`0x127`) | `MatchCommand_SetTimer` | S>C likely | one int duration |
| 486 (`0x1e6`) | `MatchCommand_InitiateChallenge` | C>S | v2; three ints |
| 67 (`0x43`) | `GameCommand_SetupGame` | S>C likely | v10; game/player ordering, versions, match id, duration, flags, game number, league, structure |
| 58 (`0x3a`) | `GameCommand_SelectDeckForPlayer` | S>C likely | per-player deck selection |
| 117 (`0x75`) | `GameCommand_ReadyForStartOfGame` | C>S | start gate; the client suppresses other game commands until started |
| 59 (`0x3b`) | `GameCommand_ActionPlayed` | C>S | rule/action event |
| 62 (`0x3e`) | `GameCommand_ButtonPressed` | C>S | UI/game action |
| 160 (`0xa0`) | `GameCommand_ConcedeGame` | C>S | concession |
| 165 (`0xa5`) | `GameCommand_FinishMultiaction` | C>S | not proven to mean end turn |
| 471 (`0x1d7`) | `GameCommand_EndGame` | S>C likely | registered; exact result fields unknown |
| 193 (`0xc1`) | `CommandObject_DrawCard` | nested | command object, not a top-level socket command |
| 206 (`0xce`) | `CommandObject_PlayCard` | nested | command object; reader/writer present |
| 83-87 | `SynchronizationCommand_Start` through `_Update` | mixed, unproven | collection synchronization |
| 141-143 | `SynchronizationCommand_StartDelivery`, `_ProcessFulfillment`, `_RemoveInstances` | mixed, unproven | delivery and fulfilment |
| 271, 301, 302 | `SynchronizationCommand_SetDesired`, `_AddNonTradable`, `_RemoveNonTradable` | mixed, unproven | collection synchronization |
| 361-363 | `DeckCommand_AddOnlineDeck`, `_DeleteOnlineDeck`, `_PopulateOnlineDeckData` | mixed, unproven | online decks |
| 147-157, 270, 272, 273 | `TradeCommand_*` | mixed, unproven | issue/accept/add/remove/lock/final-accept/sync; 25 classes exist |
| 342 (`0x156`) | `LobbyCommand_AddBooster` | unknown | booster catalogue/state, not proof of purchase |
| 449 (`0x1c1`) | `ArchCommand_StoreArchetype` | S>C likely | archetype delivery/storage path |
| 440 (`0x1b8`) | `ECommCommand_OrderHistory` | S>C likely | commerce history |
| 441 (`0x1b9`) | `ECommCommand_RedeemItemGrant` | S>C likely | v3 item grant bridge |
| 97, 446 | `LobbyCommand_SendText`, `_SendUChatText` | C>S likely | chat only; no text is recorded here |

The manual challenge exchange is **78 C>S, 79 S>C, 80 C>S**. Command 79's
DLL execute slot is `FUN_1017a080`; that function computes the client answer,
calls the `LoginCommand_RespondToChallenge` (80) constructor `FUN_101790a0`,
and queues the result through `FUN_10068af0` (`w61b-decompiled-excerpts.c`,
function 1017a080). A server must send 79 after receiving 78; waiting for a
client-sent 79 stalls manual login.

Two network IDs in the Build Guide are not part of this DLL table or the
499-row registration map:

| ID | Name / effect | Direction | Evidence boundary |
|---:|---|---|---|
| 546 (`0x222`) | `NetworkCommand_UpdateDataVersion` | S>C (in-exe) | v2 described by the Treasure War Build Guide; no DLL factory registration, reader, writer, or handshake ordering recovered |
| 575 (`0x23f`) | `NetworkCommand_RequestData` | C>S (in-exe) | Build Guide records payload `0x9f3`; no DLL factory registration, reader, writer, or handshake ordering recovered |

These are EXE-only candidates until a DLL-specific registration and
field-level reader/writer are recovered. Do not add them to the DLL card
service's accepted-command map merely because the two clients share command
family names.

Unresolved operations are explicit in `message-table.json`: `attack`,
`end_turn`, pack purchase/open, and ratings read/write. No attack-named
factory registration exists among the 499 IDs. `GameTurn` is a registered
component type, not evidence that `FinishMultiaction` is end turn.

### Framing and routing

TCG DLL game connection:

    outbound: [htonl(body_length + 12)][3 x htonl routing word][body]
    inbound:  [htonl(N)][class id][body]

`GatewayCommand_GetConnectionServer` is sent before the routing triple is in
use:

    [htonl(body_length)][body]

Common routing words:

| Scope | Routing triple | Third/second word |
|---|---|---|
| Lobby, Account, Login, Network, Match, Synchronization, Arch, UChat | `1000000000, 0, 0` | none |
| GameCommand | `1000000001, 0, game_id` | game id |
| TradeCommand | `1000000002, 0, trade_session_id` | trade session |
| GuildCommand | `1000000003, 0, 0` | none |
| ECommCommand | `1000000004, 0, 0` | none |
| EventCommand | `1000000005, account_id, 0` | account id |
| UserDataCommand | `1000000007, account_id, 0` | account id |

Every inheritance level writes its own class header. A leaf command can carry
the same class id and version more than once before its own fields. Integers
are either compact serialized values or network-order 32-bit values, selected
by a global mode flag whose setter is still unresolved.

## Data sources

### Game and row tables

| Artifact | Recovered content | SHA-256 |
|---|---|---|
| `Resources/MiniGameData.txt` | 783 rows, 30,734 bytes. TYPE 16: 124 rows; TYPE 23: 1 row (727); TYPE 3: 4 rows (7, 11, 107, 108). TYPE 16 difficulty counts: `-1:1, 1:41, 2:25, 3:39, 4:8, 5:10`. 119 rows resolve a name and description. 116 contain "Be the first to score 12 points"; 8 do not. | `8ce3cc61a2ad324912465e8d1785d246ccd680a01712e256fddf702ce856c252` |
| `Resources/MiniGameTypeData.txt` | TYPE 3, 16, 23 all declare `IS_FLASH=1`; no TCG row is members-only; all three are active | `2d200849d113c0f4b071b869774843b2fe9d07095235e7a6e907be83156e27d0` |
| `Resources/MiniGameGroupLinks.txt` | 610 links; TYPE 3 rows link to group 3, TYPE 16 to group 16, TYPE 23 to group 23 | `bff472fb91e88ebf6077f541012701cc774dcfb1d7b89261b2ad92d95ffcb623` |
| `Resources/TradingCardAssets.txt` | 45 `.lst` references and 151 `.dds` names for the TCG UI and art | `04892129b36d994a7309ff17192c7218459938365ee52ca6d554a43443736726` |
| `Resources/SimpleCardAssets.txt` | 62 `.lst` references and 244 `.dds` names for the in-exe card UI, effects, and campaign images | `763a0f98e917426564d7b2ff02adb99f5c2d6eee0dc6f3d03b0a87abdde0c18f` |
| `reports/locale-id-to-text.json` | 85,932 main-client locale strings used to join the TYPE 16 `NAME_ID` and `DESCRIPTION_ID` columns | `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844` |

The TYPE 16 names include 44 round rows, 68 rows containing `Deck`, three
`Battle ...` rows, three rows containing `Challenge`, and five unnamed rows.
Those categories overlap where, for example, a named deck is also a challenge
deck. The four deck adjectives each have 12 rows:

    trick=12, creature=12, resource=12, tournament=12

These rows are activity configuration and prize text. They are not deck lists.
No shipped table maps each row to the 40 cards in its opponent deck.

`MiniGameData` line anchors in the preserved working copy:

- row 387 (first TCG deck): line 371
- lobby 499: line 419
- tutorial 537: line 457
- Treasure War 727: line 637
- Kai challenge 1036: line 703
- unnamed 9997: line 782

The TYPE 3 rows are not a clean alias for TYPE 16. Row 11 resolves to
"FR Trading Card Game". Rows 7 and 108 resolve to unrelated text, and row 107
has no name. Do not place them in a TYPE 16 loader merely because they mention
the card game.

### TCG DLL and stores

| Artifact | Recovered content | Count / hash |
|---|---|---|
| `tcg/FreeRealmsTCG.dll` | PE32 x86, Qt 4; Ghidra analyzed 59,393 function entries; direct factory map has 499 unique numeric IDs, 382 named classes, 494 named creators. 117 class names and five creators remain unresolved. | `22f0e8f9dfc9cb398898e227fb9744b6da1989e8dcfff7b5fae85e4bd162cd8b` |
| `tcg/data/archetypes/index.dat` | Four-scalar index rows; row fields include id, compressed length, decoded length, storage offset | 1,391 rows; `2549e3d8942593bacd3e3f3656a47f4246b5b8d5f376719a5759559cae8410a3` |
| `tcg/data/archetypes/storage.dat` | zlib storage records; all 1,391 ranges decompress at the declared decoded length | `877f514ddde96779eea98b408a25b942bbcd5d6fad7190ce8657d53e87764ee9` |
| `tcg/data/archetypes/actionIndex.dat` / `actionStorage.dat` | action store and four-int index | 32,387 / 319,434 bytes |
| `tcg/data/archetypes/ruleIndex.dat` / `ruleStorage.dat` | rule store | 113,593 / 844,325 bytes |
| `tcg/data/archetypes/msgIndexen.dat` / `msgStorageen.dat` | English message store | 17,976 / 102,000 bytes |
| `tcg/data/collections/default.cln` | header `#`, version 3.3, 7-byte opaque label, four zero bytes, compact integer stream at offset 15 | 356,892 bytes; 195,063 integers; all 201 candidate IDs appear at least once; `7cc6b4c8e70a1a31863f2921379d1a79aca8bfe58836ed5434daa9d67bb61595` |
| `tcg/data/tutorial/index` / `objects` | tutorial rule sets and object database | 24,304 / 177,665 bytes |
| `tcg/data/campaign.dat` | 668-byte shipped campaign description | `cf9ccd16427831dbd2518e36a3e581e0b85ec360c770f8a6734dce3dd3772648` |
| `tcg/locale/en_us_data.dat` | 5,092 rows: 5,013 `ucdt`, 79 `ugdt` | `b631c83e05a6a1647d71ff7eb7d34df13db7e10f9455db10e88f99c0484db522` |
| `tcg/AssetsTcg_manifest.txt` | 58 manifest lines in the preserved copy, including `.z` archetype, collection, locale, DLL, and RCC assets | `ec5f38852efbef2d2e34c47e678a8717adc599024cead2e779f27cdec391ec5c` |
| `tcg/AssetsTcgW_000.pack` | 57 entries, 9 file types | 113,356,959 bytes; `60cd84e43ee76654f95d248b9919c7f76a4aa01740ea9fea6fc801f526ed3b4f` |
| `tcg/cards.rcc` | Qt RCC v1: 414 entries, including 403 PNG resources and 11 fonts | `4383c893334073960629516e0024ba1546e6a7fd2d154cdcaa31c59fd3706de0` |

The archetype decoder validates only the index and zlib/storage layers. The
inner object fields are still unresolved. `cards.json` contains 201
art-linked archetype candidates, not 201 confirmed playable cards. It records
one unmatched art id, `100011272`, and leaves name, set, rarity, type, cost,
stats, and rules-text values `null`. An art/index match is insufficient: a
sample product record has booster labels while matching the art path.

The generated text keys are:

    Tcg_title_%d, Tcg_subtitle_%d, Tcg_game_text_%d,
    Tcg_lore_%d, Tcg_traits_%d

The mapping from those generated keys to the 5,092-row TCG locale is not
recovered. Do not infer card names from numeric proximity.

The `.cln` integer stream is a token sequence only. A token count is not an
owned quantity. Record boundaries, the four-byte prefix, and owner mapping are
unknown, so a server must not import this sample as a live collection.

### Existing code to reuse

[[Research/Pieces/010 Trading card game]] and
`prototype/osfr-tools/src/restoration/piece_010.rs` already provide:

- the row classifier and four deck archetypes;
- the recovered 12-point target;
- strict increasing event ids, refusal without mutation;
- a reconnect snapshot;
- a named reward-card hook;
- 18 behavioral tests.

It is not registered in `lib.rs`, has no wire codec, and accepts a validated
point stream rather than card play. `piece_043` already has a
`TcgMatch` outcome source for Card Duelist progression. Both are useful
starting points, not a playable card service.

## Server design

### Process boundary

Keep the current game gateway separate from the card service:

- The main gateway owns login, the world session, activity admission, the
  64-bit launch ticket, and the TYPE 16/23 mini-game state.
- The card service owns its TCP listener, session login, lobby, matching,
  card state, collection/deck synchronization, and match results.
- A ticket is minted by the main server and consumed once by the card service.
  It is bound to the authenticated account/character and expires after a
  bounded launch window. The card service never trusts a client-supplied
  account id, deck value, card value, or reward.
- The TYPE 16 and TYPE 23 services must not share a lobby table. They can share
  framing and serializer helpers only.

Proposed module ownership, subject to lead review because the listener and
ticket validation are security-sensitive:

| Proposed module | Responsibility |
|---|---|
| `card_catalog.rs` | immutable row, locale, archetype, card-candidate and deck definitions |
| `card_wire.rs` | big-endian frames, routing words, class header/version codec, bounded field readers |
| `card_session.rs` | ticket consumption, login result, data-version and lobby snapshots |
| `card_match.rs` | lobby/match state, command ordering, snapshots, deterministic refusal |
| `card_progress.rs` | collection instances, decks, campaign progress, delivery/fulfilment |
| `card_rewards.rs` | one-time prize/reward bridge into the existing atomic reward path |
| `minigame.rs` + `gameplay.rs` | TYPE 16/23 state admission and launch envelope |

No new dependency is justified. The existing serializer, zlib, and content
loaders cover the observed format work. A new binary or manifest target is a
Cargo/lead decision because the autopilot rules reserve `Cargo.toml`.

### Authority and persistence

Server authority is reconstructed policy `(R)`. The client contains a full
rule engine and can evaluate card text, but no capture proves whether the
original server, the client, or both decided a match. The safe policy is:

- Clients submit intents; the server validates turn, ownership, cost, target,
  phase, and command order.
- The server owns the match seed, turn/phase transitions, scores, winner, and
  reward eligibility.
- A rejected intent, stale command, replay, or invalid snapshot changes no
  score, collection, deck, or reward.
- Persist collection instances and decks by stable ids. Do not use display
  names as keys.
- Persist result and reward in one transaction. Reload and reconnect must
  reject a duplicate settlement.

Keep an explicit compatibility fallback for field order and the integer mode.
Never silently reinterpret an unknown class id or version.

## Build plan and gates

### Slice 0: freeze the evidence and the content contract

Deliver:

- this note;
- a checked-in catalogue fixture generated from the preserved client tables;
- a provenance manifest listing input hashes and the output schema version;
- row counts and locale joins as test data, not hard-coded UI strings.

Gate:

- exactly 783 `MiniGameData` rows, 124 TYPE 16, one TYPE 23, four TYPE 3;
- exactly 119 localized TYPE 16 names/descriptions and 116 twelve-point
  descriptions;
- exact deck archetype counts `12/12/12/12`;
- 1,391 archetype store rows decode at their declared lengths;
- changing a count by one makes the gate fail;
- no player or chat text appears in the fixture.

### Slice 1: data-loader and catalogue API

Deliver `card_catalog.rs` and a deterministic `CardGameRow` model. Preserve
`None` for unresolved name, description, reward, locator, and difficulty.
Reject duplicate ids, malformed rows, out-of-range difficulty, and unknown
archetype values.

Gate:

- positive tests use only synthetic rows constructed in test code;
- the real audit reproduces the recovered counts and is byte-stable across two
  runs;
- a canary that changes one row's type or target must fail;
- no gateway or gameplay route is added yet.

### Slice 2: launch state machine

Deliver the existing mini-game integration for TYPE 16 and TYPE 23:

1. accept the activity join;
2. create a state from one row;
3. send generic 39/16 and 39/17;
4. mint a ticket and send 39/0x28 only when the expected ready signal arrives;
5. return 39/0x29 on a failed launch;
6. consume 39/6 and restore the world state.

Gate:

- synthetic matrix covers join, info, start, ready, ticket, close, and failure;
- ticket halves are one 64-bit value and a zero/absent ticket cannot launch;
- duplicate ready, stale ticket, wrong state/type, and close twice are
  refusals that leave the state unchanged;
- every new `application_handler` selector gets an `app_fuzz` seed;
- native gate: the Windows client reaches the loading panel and accepts the
  launch or failure path. Do not claim a playable card UI at this slice.

Unproven candidate step (T504, not built): the client's 39 dispatcher
(`009bfea0`) case 0x43 fires `ActivityEvents:OnCreateGameResult` only when the
byte at +0x18 of the packet (ctor `009aa230`) is 0. No Lua listener was found,
and where it sits relative to BeginLoad is (R). The 117/118 OneTimeSession
exchange is not the TCG ticket: its only senders are `0093bec0` (session type
41, vtable slot `01820298` beside the PacketInGamePurchase strings) and
`0093bf50` (type 68, "RequestChinaSnsSesstionTicket"). The ticket stays
39/0x28 (`009a9de0`, two i32 halves). The sequence hint comes from TCG-Files
(no licence, nothing copied), and nothing there shows FreeRealmsTCG.dll
actually loaded.

### Slice 3: TCG card-service bootstrap

Deliver the separate listener and the DLL-compatible framing:

1. `GatewayCommand_GetConnectionServer` (414) with the unrouted form;
2. session login 411 and results 81;
3. manual challenge 78-80 only as a separate path;
4. lobby snapshot 103.

Do not schedule the 546/575 data-version path here yet. Both IDs currently
come from the in-executable Treasure War client in the Build Guide and are
absent from the DLL's 60-row curated map and 499-row registration table.
First recover a DLL-specific registration plus field-level readers/writers and
handshake ordering, then add them as a separate reviewed slice.

Gate:

- golden frames round-trip class header, version, routing triple, and bounded
  body;
- malformed length, unknown class id, truncation, duplicate session, expired
  ticket, replay, and wrong account are refusals with no state change;
- a real DLL handshake transcript is required before marking the login path
  recovered; a synthetic fixture alone is not native acceptance;
- every accepted DLL command ID resolves to a direct registration row; an
  in-exe command name alone is not sufficient;
- security review is required before exposing the listener.

### Slice 4: catalogue delivery and account collection

Deliver archetype/action/rule/message delivery and collection synchronization:

- read 1,391 archetype rows and retain unresolved inner fields;
- join only verified localization keys;
- represent card instances, foils, non-tradable flags, and decks;
- keep collection sync idempotent and transactionally atomic.

Gate:

- all 1,391 storage rows pass a golden hash test;
- 201 art-linked candidates are labelled `candidate`, not playable;
- unknown inner fields are preserved, never invented;
- `.cln` token counts are not accepted as quantities;
- delivery, add, remove, and fulfilment are once-only under retry;
- refusal tests assert byte-for-byte unchanged account state.

Do not implement deck ownership or trade until record boundaries and online
deck field layouts are decoded.

### Slice 5: lobby, challenge, and deck selection

Deliver command handling for 92/93, 99-102, 116, 124/294/295, 486, and 103.
Start with a two-client private harness using generated fixtures. Keep
unresolved matchmaking, rating, tournament, and league operations explicit.

Gate:

- every body has a field-level reader and a bounded writer test, or is marked
  unsupported;
- a client cannot join a full or unauthorized lobby;
- duplicate ready/launch and stale match state are refused;
- deck choice is checked against the account collection;
- a captured original-DLL lobby transcript is required for field order beyond
  the current reader evidence.

### Slice 6: match shell and authority

Deliver the `SetupGame` -> `ReadyForStartOfGame` -> action -> result shell.
Use `piece_010` only as a scored-stream reference. Do not call that engine a
TCG rules implementation.

Gate:

- command ordering is deterministic and replayable;
- client intent cannot set scores, owner, winner, or reward;
- out-of-sync, illegal command, timeout, concede, and reconnect are covered;
- an unproven attack/end-turn id stays unsupported rather than guessed;
- one exact match trace, or a field-level decode of the DLL rule path, is
  required before claiming a playable TCG match.

If neither trace nor rule/store decoding is available, stop at the lobby. This
is a deliberate stop condition, not a license to invent scoring.

### Slice 7: first NPC duel and Card Duelist reward

Pick one TYPE 16 row with a resolved 12-point target and a named prize card.
Use a private, server-seeded NPC deck `(R)` restricted to fixtures. Run the
real UI through launch, match, results, Close, and reload.

Gate:

- the exact selected row is visible in the lobby;
- one match reaches 12 points with no more than one terminal result;
- the prize and Card Duelist XP are granted once through the shared atomic
  reward path;
- a duplicate turn-in, reconnect, stale result, and replay grant nothing;
- the original client shows the result and returns to the world with the
  pre-match job/position restored;
- native evidence includes packet/file traces and a reload, not only a test
  count.

### Slice 8: collections, packs, trades, tournaments

Add only after the match and reward gate is native-green:

- 83-87, 141-143, 271, 301, 302;
- 361-363 for online decks;
- 342, 449, 440, 441 for catalogue/commerce;
- 147-157, 270, 272, 273 for trade;
- event/tournament commands only after their field layouts are decoded.

Gate:

- pack purchase and open are server-priced and idempotent;
- card, virtual-reward, and item grants use the existing atomic reward path;
- trade has an explicit two-sided final-accept boundary and serializes with
  collection/deck saves;
- every refusal preserves both parties' in-memory and persisted state;
- account and card values are never accepted from client-provided totals.

### Slice 9: Treasure War, separate track

Reuse only the frame/class-codec ideas. Use TYPE 23 and its 21782/21783 link.
The in-exe engine already has a visible setup/turn/snapshot path, but its
card data source, server authority, and card-record schema are still open.

Gate:

- no shared TCG lobby, collection, or deck table;
- the TYPE 23 branch is exercised independently through its own campaign/home
  screens;
- the first acceptance is a single-player campaign row, not the TCG Duel Row;
- any borrowed rule or reward behavior is labelled `(R)`.

## Open gaps

1. Whether the original TCG server or the DLL client resolved rules and
   damage. No match trace exists.
2. Attack and end-turn command IDs. They are absent or unproven.
3. Full field layouts for lobby, match, collection, deck, trade, pack, and
   rating commands.
4. Inner archetype fields, generated key-to-locale mapping, and the distinction
   between a product record and a playable card.
5. `.cln` record boundaries, quantities, and the four-byte prefix.
6. Which actor triggers the `39/0x26` challenge request.
7. ~~The two hashed activity ids and the inner activity-join body behind gateway
   opcodes 5/6.~~ Resolved by T504: `Tcg.ActivityId` 1049 and `Scg.ActivityId` 1078
   (defaults at `00c04970` / `00c049f0`), body 167/1/2.
8. The sessionID dialog trigger and the exact gateway-to-card login transcript.
9. Card Duelist quest prerequisites, coin/XP amounts, and prize-card grants.
10. TYPE 3 rows 7, 107, and 108, which do not resolve cleanly to TCG content.
11. DLL registration, field layouts, and handshake ordering for
    `NetworkCommand_RequestData` (575) and `NetworkCommand_UpdateDataVersion`
    (546). The current evidence is from the separate in-exe Treasure War
    client, not `FreeRealmsTCG.dll`.
12. No native TCG run exists. No end-to-end card-game claim is supported by
    the current evidence.

## Source map

- `../lab/working/OSFR.Client/OSFR Client/Resources/MiniGameData.txt`
- `../lab/working/OSFR.Client/OSFR Client/Resources/MiniGameTypeData.txt`
- `../lab/working/OSFR.Client/OSFR Client/Resources/MiniGameGroupLinks.txt`
- `../lab/working/OSFR.Client/OSFR Client/Resources/TradingCardAssets.txt`
- `../lab/working/OSFR.Client/OSFR Client/Resources/SimpleCardAssets.txt`
- `../lab/working/OSFR.Client/OSFR Client/tcg/FreeRealmsTCG.dll`
- `../lab/working/OSFR.Client/OSFR Client/tcg/data/`
- `../lab/working/OSFR.Client/OSFR Client/tcg/locale/en_us_data.dat`
- `../lab/content/tcg/analysis/dll-registration.tsv`
- `../lab/content/tcg/analysis/message-table.json`
- `../lab/content/tcg/analysis/README.md`
- `../lab/content/tcg/cards.json`
- `../lab/content/tcg/collections.json`
- `../lab/content/tcg/inventory.json`
- `../lab/decompiled/client-ui/ScriptsBase.lua`
- `../lab/reports/locale-id-to-text.json`
- `../lab/native-map/guides-scratch/TCG/narrative.md`
- [[Research/TCG Client Protocol]]
- [[Research/TCG Card Data]]
- [[Research/Pieces/010 Trading card game]]
- [[Build Guides/Gameplay/Trading Card Game]]
- [[Build Guides/Gameplay/Minigame Framework]]
- [[Build Guides/Gameplay/Matchmaking and Lobby]]
- [[Build Guides/Gameplay/Trade]]
- [[Build Guides/Content/Jobs/Card Duelist]]
