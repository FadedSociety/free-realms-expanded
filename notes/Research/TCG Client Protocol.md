# FreeRealmsTCG client protocol

**Passes:** 2026-10-13, Tasks W61 and W61b. **Scope:** separate `FreeRealmsTCG.dll`; analysis only. The in-executable Treasure War/Scaleform client is a different target. The [Trading Card Game Build Guide](../Build%20Guides/Gameplay/Trading%20Card%20Game.md) identifies the DLL load boundary at `FreeRealms.exe` `010d1ab0` and launch setup at `00ad47f0`; its prior protocol section was explicitly unknown. The W40 [TCG Card Data note](TCG%20Card%20Data.md) inventories the separate `tcg/` assets and open data questions. The W61b addendum below supersedes W61's unknown status for the DLL numeric factory map and updates the login, match, and decoder findings.

Evidence labels in this note:

- **DLL:** decompiled from the shipped `FreeRealmsTCG.dll` using Ghidra 12.1.4 full auto-analysis.
- **EXE map:** command IDs and handler addresses from `lab/native-map/run/map.sqlite`, queried read-only with `python -I scripts/native_map/nmq.py command ...`. These are addresses in `FreeRealms.exe`, not DLL function addresses.
- **(R):** reconstructed service policy or proposal, not a recovered client rule.

The input copy is `lab/runs/client-baseline-20261008T015138Z/client/tcg/FreeRealmsTCG.dll`, SHA-256 `22F0E8F9DFC9CB398898E227FB9744B6DA1989E8DCFFF7B5FAE85E4BD162CD8B`. Full headless analysis completed and saved in 224 seconds. Exported evidence is under [`lab/content/tcg/analysis/`](../../../lab/content/tcg/analysis/README.md). `tcg/frtcg.ini` was not opened; no player names or chat text were recorded.

## Transport and endpoints

**DLL imports.** This is a 32-bit Windows DLL. `imports.tsv` records 30 `WS2_32.dll` imports: `accept`, `inet_addr`, `bind`, `listen`, `getpeername`, `gethostname`, `getsockname`, `ntohs`, `GetAddrInfoW`, `inet_ntoa`, `FreeAddrInfoW`, `socket`, `setsockopt`, `connect`, `WSASetLastError`, `gethostbyname`, `getsockopt`, `WSACleanup`, `select`, `__WSAFDIsSet`, `shutdown`, `closesocket`, `ioctlsocket`, `WSAStartup`, `send`, `recv`, `WSAGetLastError`, `htons`, `htonl`, and `ntohl`. Ghidra resolves 28 imported ordinals to names; `GetAddrInfoW` and `FreeAddrInfoW` are named imports. No `QtNetwork4.dll`, `QTcpSocket`, `QNetworkAccessManager`, `QSslSocket`, WinHTTP, WinINet, or SSL/TLS library import was found. QtCore does import `QUrl`.

The game command link is a custom TCP `NetworkService` (`FUN_10191dc0`, with the source-path string `NetworkService.cpp`) and `TcpLibrary::TcpConnectionPlatformDriver`, not Qt Network. A separate custom `TinyHttp::HttpConnection`/`TcpLibrary` path and plain HTTP asset URLs are present. The DLL contains these asset bases:

- Live: `http://fr.patch.station.sony.com/patch/freerealmstcg/live/assets/`
- Stage: `http://sdt-devpatch.station.sony.com:7000/patch/freerealmstcg/staging_assets/`

**Game server selection.** `FUN_10218df0` parses `--realm=` and selects realm defaults. For production / `frtcg_live` (and the shared beta case), the gateway is `lvsccg-01-02.station.sony.com:17782` and the default server is `lvsccg-01-01.station.sony.com:17783`. `frtcg_livetest` uses `sdtccg-dev2.station.sony.com` on ports 17382/17383; `frtcg_stage` uses the same host on 17182/17183. `frtcg_china` uses `10.132.64.6` on 17782/17783. These values are recovered defaults, not proof that those historical servers are reachable.

`FUN_10191f60` optionally reads `host.svr` and parses case-insensitive `host`, `port`, `ghost`, and `gport` overrides; this establishes the config format without reading any such file. Do not confuse `--host=fr` with the TCP server host: `FUN_10218df0` interprets it as the game/client host label. The launcher supplies `--lang`, `--country`, `--username`, `--realm`, `--sessionID`, `--character`, `--challenge`, `--ticket`, `--host=fr`, `--is-founder`, and `--post-login=tutorial` (Build Guide launch-boundary evidence). No actual player argument value is included here.

**Connect and login.** `FUN_10193430` creates a nonblocking TCP socket, sets `TCP_NODELAY`, and connects. `FUN_101935d0` builds a `GatewayCommand_GetConnectionServer`; the DLL constructor and serializer are `FUN_10121770` and `FUN_10121d20`, with the class string at `1065bfb4` and xref `10121810`. This is direct evidence of the gateway request path. The gateway response parser and its complete handoff sequence were not recovered in this pass.

The DLL contains `LoginCommand_SendSessionID` and `LoginCommand_SendPassword` class strings (`10667024` / `1017b8f0`, `10666efc` / `1017ad80`). W61b's direct DLL factory table confirms session ID 411 and challenge IDs 78–81. The manual challenge/password path is also present in DLL code. Exact launch-session command trigger/order, field meanings, and the complete response-to-lobby sequence remain unresolved; details and the split UI paths are recorded in the W61b addendum below.

**Framing.** Direct DLL evidence is in `FUN_101921e0` (send) and `FUN_10192500` (receive):

- Outbound: a big-endian length equal to serialized command bytes plus 12, then three big-endian routing words (12 bytes), then the command's serialized body.
- Inbound: a big-endian length `N`, followed by `N` bytes. The receiver waits for the whole frame, reads the leading body class ID, creates that component via the `ComponentFactory`, and invokes its reader/handler.
- The observed game frame functions write/read the serialized command body directly; they show no frame compression or encryption step. No SSL/TLS import exists. Compression or transforms inside specific command payloads were not exhaustively ruled out.

`FUN_100940b0` lazily constructs the `ComponentFactory`; `FUN_100938d0` initializes its generic factory/map storage. W61b recovered the direct DLL numeric registrations; see the addendum and `dll-registration.tsv`. The curated `message-table.json` still keeps EXE handler addresses separate from DLL class/slot addresses and marks unresolved DLL class names rather than treating the EXE handler as a DLL entry point. Exact DLL string xrefs remain in `observed-message-strings.tsv`.

## Card data and rules

The DLL directly opens local data stores. `FUN_10022680` and `FUN_100446e0` construct readers rooted at `data\archetypes` with `.dat` data. The recovered named tables/readers are:

- `actionIndex`: `FUN_10021f70`
- `index`: `FUN_10044510`
- `ruleIndex`: `FUN_101c54e0`
- `msgIndex`: `FUN_10182410`, whose key helper `FUN_10181650` returns `msgIndex`

Each indexed row passes four scalar reads to the relevant tree/map helper. For `actionIndex` and `ruleIndex`, `FUN_10021c60` compares the first incoming scalar as the lookup key. The scalar helper `FUN_100977a0` supports compact variable-length integers when its global mode flag is clear and big-endian 32-bit integers when set. The mode setup was not established at these file-reader call sites; field meanings beyond the key are still unknown. W40's independent store-index audit found 1,391 four-integer index rows with compressed/decoded lengths and storage offsets, but it did not deserialize the inner archetype objects. The exact card field layout (title, set, rarity, type, cost, statistics, and rules text) is not recovered from this DLL pass.

`FUN_10070b50` opens `data\collections` through the local `.cln` reader. W40 safely decoded one collection sample: marker `#`, version 3.3, seven-byte opaque label, four zero bytes, then compact integers from offset 15. All 195,063 integers decode, but record boundaries and ownership quantities remain unknown. That token count is not a card quantity table. No additional collection file or personal config was opened for this pass.

The data is therefore **both local and network-fed**: local `data\archetypes` `.dat` tables and `data\collections` readers are direct DLL facts; the EXE map's `ArchCommand_StoreArchetype` ID 449 is corroborated by the DLL class string at `1067a7ac` (xref `10213510`) and indicates a server-delivered/stored archetype path. The division between a complete local catalog and server misses is unresolved.

The DLL has `actionIndex` and `ruleIndex` tables, and its class strings include game/action command families. This proves local rule/action data and client-side command handling exist; it does **not** prove whether a match is resolved authoritatively on the client or server. Treat authoritative server-side validation and deterministic resolution as **(R)** until a match trace or handler proves it.

## Persistence and service boundary

The EXE command map, name-correlated to DLL classes, exposes collection synchronization (`SynchronizationCommand_StartInstances`, `SendInstances`, `Complete`, `Update`, plus delivery/fulfillment and add/remove-instance operations) and online-deck operations (`DeckCommand_AddOnlineDeck`, `DeleteOnlineDeck`, `PopulateOnlineDeckData`). This is evidence of protocol operations, not proof of the backend's durable schema.

- Persist player-owned collection instances and accepted online decks: **(R)** service policy, guided by those sync/deck operations. Keep the client's `.cln` catalog/collection reader separate from account-owned server state until record semantics are decoded.
- Persist trade outcomes only after the final acceptance flow: **(R)**. The map has request, accept, add/remove-card, lock, final-accept, and clear-state messages, but no server transaction implementation was recovered.
- Ratings: no rating-specific read/write operation or storage schema was recovered. The UI/build guide's leaderboard references do not establish the protocol.
- Store/packs: `LobbyCommand_AddBooster` ID 342, `ArchCommand_StoreArchetype` ID 449, `ECommCommand_OrderHistory` ID 440, `ECommCommand_RedeemItemGrant` ID 441, and synchronization fulfillment operations are relevant EXE-map entries. The pack purchase/open flow and its account/item reward bridge are not decoded. Integrating rewards through Free Realms item grants is **(R)** pending direct evidence.
- Chat has `LobbyCommand_SendText` ID 97 and `LobbyCommand_SendUChatText` ID 446. This pass recovered class/type references only; no chat content was recorded.

The scoped message table includes login/auth, collection sync, online decks, matchmaking/challenge, game actions, trade, booster/catalog, commerce/reward, and chat entries. It also records missing explicit `attack`, `end_turn`, pack-purchase/open, and rating operations rather than assigning speculative IDs.

## Rust service outline

Implement in this order; items marked **(R)** are reconstructed policy until the DLL's numeric factory map, payload serializers, and server behavior are verified:

1. **Gateway/session bootstrap:** accept only the launch-provided session material, select a realm endpoint, and implement the directly registered `GatewayCommand_GetConnectionServer` ID 414. Keep the session-dialog send trigger/order separate from the directly observed manual challenge/password paths until its callback is recovered. Exact gateway and login payload fields still need mapping.
2. **Static catalog and fetch/cache:** parse the local archetype tables and their index/storage records; add an `ArchCommand_StoreArchetype`-compatible missing-definition path. Preserve unresolved inner fields instead of inventing card stats.
3. **Collection sync and decks (R persistence):** implement start/instance/update/complete synchronization and online-deck add/delete/populate. Require an explicit ownership and deck schema before importing `.cln` tokens as quantities.
4. **Challenges and matchmaking:** implement challenge, ready, bid, deck selection, match-state and launch messages from verified field serializers.
5. **Matches (R authority policy):** validate and resolve actions server-side, return ordered game-state/action results, and preserve unknown attack/end-turn operations as unknown until mapped.
6. **Trade, boosters, rewards, and chat:** add after final-accept semantics, commerce payloads, and the Free Realms reward boundary are recovered.

## Open questions

- Refine the recovered DLL factory table: 117 rows have unresolved class names and five have unresolved creator callbacks; verify the session-login callback and any generic registrations before claiming semantic coverage.
- Recover a launch-to-gateway-to-session-login transcript, gateway response decoding, exact routing-word meaning, timeouts/reconnect behavior, and server host override precedence.
- Determine whether any command payload uses compression or encryption and identify the scalar mode for each archetype/index table.
- Decode the inner archetype records, localization key/value join, `.cln` record boundaries and ownership quantities, and deck layout/limits.
- Establish which side resolves rules, and map card play, attack, phase/end-turn, concede, and result/reward semantics.
- Recover matchmaking/rating persistence, store/booster purchase/open, item-grant fulfillment, and trade atomicity.

All confirmed addresses, commands, hashes, and decompilation outputs are in [`lab/content/tcg/analysis/README.md`](../../../lab/content/tcg/analysis/README.md), `targeted-functions.c`, `relevant-functions.c`, `imports.tsv`, and `message-table.json`.

## W61b addendum: DLL registration, login, match actions, and data

### Evidence and the direct factory map

W61b reused the saved Ghidra 12.1.4 project for the same DLL hash above, with `-process -noanalysis`, `-max-cpu 2`, and the lab's 4G heap. The runner is `lab/content/tcg/analysis/run-w61b.ps1`; exact decompilation and exported rows are in `w61b-registration-initializers.c`, `dll-registration.tsv`, `w61b-decompiled-excerpts.c`, `w61b-symbol-xrefs.tsv`, and `w61b-launch-ui-strings.tsv`.

- `FUN_100940b0` lazily constructs the DLL `ComponentFactory`, whose backing map is initialized through `FUN_100938d0`. Static initializer functions call `FUN_100941c0(id, creator)`; `FUN_100941c0` obtains an entry through `FUN_10094130` and stores the creator at that entry. `dll-registration.tsv` contains **499 unique numeric IDs**. The companion initializer dump exposes each direct call; `merge_dll_registration.py` checks the table ID against that decompiled call for all 499 rows before it modifies `message-table.json`.
- Receive path: `FUN_10192500` waits for the full length-prefixed frame, reads the body class ID, looks up/creates the command through `FUN_100940b0`, calls its reader slot `+0x24`, then its dispatch slot `+0x38`. The common `NetworkCommand` dispatch at `FUN_1018a8b0` invokes the object's execute slot `+0x3c`. Nested `CommandObject_*` dispatch uses `FUN_101aa930` at `+0x38` and processes `PlayElement`/`Card` components.
- The 60 selected rows carried forward as EXE-map cross-references all have the same numeric IDs registered in the DLL. The class-name comparison is 22 exact, 22 shared `GameCommand` base registrations, and 16 unresolved names. These results show numeric-ID agreement within this curated set; they do not prove equal payload meaning or server behavior. `exe_handler` values remain FreeRealms.exe addresses, separate from DLL creator and vtable addresses.
- DLL registrations include `CommandObject_GetTarget` ID `0xd6` and `GameTurn` ID `0xe8`. They have no row in this curated EXE cross-reference subset; this is not evidence that those IDs are absent from the full EXE map. `GameTurn` is a registered component type, not proof of an end-turn request.
- The full registration table has five rows without a recovered creator and 117 without a recovered class name; the class/slot blanks are retained as unknown. See `dll-registration.tsv`. The map is directly recovered as numeric registrations, but some type names and fields remain unresolved.

### Connect, login, and UI states

- `FUN_101935d0` constructs and sends `GatewayCommand_GetConnectionServer` (DLL registration ID `0x19e` / decimal 414) on the gateway connect path. Its constructor/serializer are `FUN_10121770` / `FUN_10121d20`; registration initializer/creator are `10121df0` / `10121d90`; the class reader and execute methods are `10121c60` / `10121980`. The gateway response handler updates endpoint state, reconnects, and emits events `0x95`/`0x96`. Therefore the first directly identified application command on the gateway path is the gateway request, before the later login flow. The precise gateway reply schema and second-connection transcript remain unknown.
- The DLL parses launch arguments `--sessionID`, `--username`, `--challenge`, `--character`, `--characterID`, and `--ticket` at `FUN_1037a970`, `1037aa80`, `1037ab90`, `1037aca0`, `1037adb0`, and `1037aec0`; only argument names are recorded, never values. `FUN_1037b200` stores session ID and username into login-service state. `FUN_1037bba0` stores challenge, character, character ID, and ticket, then selects a login UI branch. If session ID is nonempty, `FUN_1037b5f0` creates a `sessionLoginDialog`. ID `0x19b`/411 is directly registered as `LoginCommand_SendSessionID` with constructor `1017bac0` and serializer `1017b700`, which serializes a list, multiple strings, a boolean, and further strings. The exact trigger and order for sending that command from the launcher session-dialog route are not established.
- The manual credential UI path is clearer. In `FUN_10379c20`, when its `+0x70` mode field is zero, it constructs a `LoginCommand_GetChallenge` ID `0x4e`/78 and queues it through `FUN_10068af0`; that helper calls the frame sender `FUN_101921e0` when the command connection is active. DLL registration and methods are `10175e40` (constructor), `101760c0` (reader), `10176110` (writer), and `10175f70` (execute).
- The matching registered `LoginCommand_SendChallenge` ID `0x4f`/79 has execute function `FUN_1017a080`. It computes a challenge response locally from command and login-service fields, constructs `LoginCommand_RespondToChallenge` ID `0x50`/80, populates its fields, and queues that command through `FUN_10068af0`. Thus the challenge calculation is client-side; it says nothing about who resolves TCG match rules. The following registered `LoginCommand_Results` ID `0x51`/81 handler `FUN_10179530` interprets result fields and emits application events. Exact meanings of all response fields and event codes are not recovered.
- Login screen string dispatch recognizes `WALoginScreen` in `FUN_105167d0`; lobby dispatch recognizes `WALobbyScreen` in `FUN_10516320` and `ZPLobbyScreen` in `FUN_105224d0`. The source shows these screen handlers and login events but does not prove a complete, ordered response-to-lobby state machine. In the separate credential-mode branch of `FUN_10379c20`, the `LoginCommand_SendPassword` ID `0x19a`/410 object is populated and queued. Whether that branch is used after gateway setup depends on UI state; it is not the same evidence as the launcher sessionID branch.
- Exact credential payload labels, launcher ticket semantics, response waits/timeouts, retries, gateway host handoff fields, and the session-dialog send callback remain unresolved. Do not fill these in from the sibling FreeRealms.exe map alone.

### Match action registrations and authority boundary

- ID `0xce`/206 registers `CommandObject_PlayCard`; ID `0xc1`/193 registers `CommandObject_DrawCard`; ID `0xd6`/214 registers `CommandObject_GetTarget`. The command-object reader/writer/dispatch/execute slots recorded in `dll-registration.tsv` include `10136390`, `101a9200`, `101aa930`, and `100793c0` for these types. `FUN_101aa930` locally walks play elements and card objects, demonstrating client-side command/action processing.
- EXE-map IDs `0x3b`/59 (`GameCommand_ActionPlayed`), `0xa0`/160 (`GameCommand_ConcedeGame`), `0xa5`/165 (`GameCommand_FinishMultiaction`), and `0x1d7`/471 (`GameCommand_EndGame`) each map directly to a DLL registration with the generic class `GameCommand`; their reader/writer/execute path includes `10109300`, `10109260`, and `10109130`. `1018a8b0` is the wrapper dispatch. The generic class name is a base-type match, not an independently decoded per-command schema.
- No `Attack`-named registration is present among the 499 DLL IDs. `GameCommand_FinishMultiaction` is not proven to mean end turn, and the registered `GameTurn` type does not resolve that question. Keep attack and end-turn operations unassigned.
- Local command-object processing and local `actionIndex`/`ruleIndex` data establish some client-side evaluation. They do not show whether the client chooses legal actions/outcomes or merely previews/applies server results. Server-side legality checks, turn order, damage, target validation, deterministic resolution, and authoritative result/reward policy are **(R)** until confirmed by a match trace or deeper handler proof.

### Card rows and `.cln` decoder sketch

The existing W40 decoder was rerun to validate the bounded outer structure; W61b did not claim an inner object schema.

- Archetype index: the client DLL reader `FUN_10044510` consumes four scalars per row and `FUN_10021c60` uses the first as the lookup key. The sibling FreeRealms.exe native map provides precise row names at `ArchetypeDB_GetArchetype` `012b8ff0`: `card_id`, compressed length, decoded length, and storage offset. These names are confirmed in the W40 native-map evidence, not inferred from the DLL field positions. `FUN_100977a0` can decode compact integers or fixed big-endian 32-bit values, selected by a mode flag whose initialization at the reader call sites is unresolved.
- Decoder sketch: decode four index scalars; use ID as key; take exactly the compressed length at storage offset; zlib-decompress and check exact decoded length; pass the decompressed buffer to the class/object reader. The bounded rerun validated all **1,391** index/storage rows. Its generated `w61b-cards-validation.json` equals `cards.json` exactly and retains 201 art-linked candidates, one unmatched art ID, and unresolved semantic fields. The compressed index/data layer is validated; the inner archetype object fields are not.
- `.cln` sample: `FUN_10070b50` identifies the DLL collection-reader path. The bounded sample decoder reads `#`, version 3.3, the seven-byte opaque label, four zero bytes, then compact integers to EOF. It returns **195,063 integers**, with all **201/201** candidate card IDs occurring at least once. The generated `w61b-collections-validation.json` matches `collections.json` except for its source-path string. Do not interpret token frequency as ownership quantity.
- `.cln` decoder sketch: verify marker/version; retain the opaque seven-byte field; record the four zero bytes; decode compact integers through EOF and validate token conservation. Row boundaries, owner mapping, quantities, flags, and later fields are not established, so grouping is not implemented as a recovered fact. The runtime `Card` network component (ID `0xa8`, reader `1019df70`, writer `10066730`) is separate evidence and has not been shown to be the root class of local archetype records.

### Revised Rust service outline

This outline replaces the W61 sketch where W61b supplies direct DLL evidence. Anything marked **(R)** remains proposed server behavior rather than a client-recovered rule.

1. Implement the observed length-prefixed TCP framing and direct gateway command ID 414 with the DLL field order once its gateway payload is fully decoded. Treat the host handoff and retry policy as **(R)** until the reply fields and connection trace are established.
2. Keep launcher session login and manual password/challenge login as distinct paths. Implement IDs 78 through 81 and ID 410 only from the DLL readers/writers; keep session ID 411's trigger/order open until the dialog callback and full command field names are recovered. Authentication, ticket validation, account mapping, expiry, and rejection policy are **(R)**.
3. Decode archetype index rows and zlib storage records; add inner field names only when the object reader/schema is field-mapped. Retain unresolved values instead of inventing stats or rules text.
4. Parse `.cln` outer header and compact integer stream, but do not import token frequencies as owned quantities. Account collection ownership, quantities, reconciliation, and persistence are **(R)** pending boundaries and sync semantics.
5. Register/deserialize known match command IDs and preserve nested action order. Enforce server-side legality, target rules, card costs, damage, turn transitions, and authoritative results as **(R)**; leave attack and end turn unmapped until proven.
6. Add matchmaking, decks, trade, store/rewards, ratings, and chat from verified field layouts. Deck ownership and limits, trade atomicity, reward/item grants, rating updates, and persistence are **(R)** where the client evidence does not define them.

### Remaining gaps

- 117 direct factory rows have no recovered class name and five have no recovered creator; refine these from their callback/vtable data before claiming every registered type is semantically identified.
- The launcher session-dialog callback that sends ID 411 and the full gateway-to-auth transcript are untraced. Payload field names, session/ticket semantics, result event meanings, waits/timeouts, retries, and host-handoff schema are incomplete.
- Attack/end-turn IDs and match authority are unresolved. A local action handler is not proof of local authority.
- The local archetype object schema and `.cln` record grouping/ownership quantities remain unresolved. The decoder reruns validate only the index/compression and outer integer-stream layers.
- No live protocol capture or server implementation was inspected in W61b; do not claim server behavior from client constructors, EXE map names, or local rule data.
