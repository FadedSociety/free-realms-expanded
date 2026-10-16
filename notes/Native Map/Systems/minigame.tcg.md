# System: minigame.tcg

36 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad1590` | `ClientSimpleCardGameManager::Ctor` | ClientSimpleCardGameManager | Constructs the client-side Simple Card Game manager used for TCG sessions. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad1a30` | `StartManagerStateIfIdle` | ClientSimpleCardGameManager | Starts the manager's initial timed state once. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad1a80` | `HandleTcgLaunchTicket` | ClientSimpleCardGameManager | Handles the TCG launch-ticket callback and starts campaign-load presentation when count is positive. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad1ef0` | `ClientSimpleCardGameManager::Shutdown` | ClientSimpleCardGameManager | Shuts down the local TCG client manager and its UI resources. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad22c0` | `ClientSimpleCardGameManager::Dtor` | ClientSimpleCardGameManager | Cleans the Simple Card Game manager object. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad2360` | `ClientSimpleCardGameManager::ScalarDeletingDtor` | ClientSimpleCardGameManager | Scalar deleting destructor in the manager vtable. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad2380` | `AdvanceTcgStateMachineAndRecords` | ClientSimpleCardGameManager | Coordinates TCG initialization, navigation, timeout, and per-frame record cleanup. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad2d30` | `ClientSimpleCardGameManager::InitCardClient` | ClientSimpleCardGameManager | Initializes the local TCG client UI and resources. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad3150` | `StartTcgClientInitialization` | ClientSimpleCardGameManager | Enters the TCG client-initialization state. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `01369c70` | `CTP::Game::DispatchTextCommand` | CTP::Game | Routes the card game's text command lines to the game's handler virtuals. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136ca90` | `CTP::Game::SubmitCardCommand → CTP::Game::SubmitIntroduceCardCommand` | CTP::Game | Creates a command for a card and submits it to the game's command path. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136d1f0` | `CTP::Game::AdvanceTurn` | CTP::Game | Turn advance for the card game: creates the start-of-game turn on the first call and re-enters the turn afterwards. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136d810` | `HandleCaughtGameError` |  | Error recovery for the game loop: a recoverable per-player error is logged and play continues, while a fatal error is logged and sets this+0x1d8. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |

## 🟢 likely (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad1220` | `PollStateUntil4OrZero` |  | Waits for the manager state to reach 4 or reset to 0. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad14f0` | `SetTcgStateAndTimestamp` |  | Records a TCG state transition and its start time. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad1530` | `ElapsedSinceTcgStateTimestamp` |  | Measures elapsed time from the saved TCG state timestamp. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `0136d4f0` | `SubmitListEntriesOrReject ⚠downgrade` |  | Submits all queued list entries of the game when the server accepts commands, and rejects them with CW_ERROR otherwise. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136d640` | `DispatchCommandObject ⚠downgrade` |  | Gate for submitting a game command object: rejects it when the server-ignore flag is set, otherwise routes it to the submission paths. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136d6a0` | `SubmitListEntriesIfActive ⚠downgrade` |  | Variant of SubmitListEntriesOrReject gated on the active flag at this+0x388. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136dbd0` | `CTP::Game::Setup ⚠downgrade` | CTP::Game | Game setup phase for the card game: records the setup time, creates the per-player state and marks the game as set up. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |

## 🟡 speculative (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad1630` | `SetCardsEnabledFlag ? ⚠downgrade` | ClientSimpleCardGameManager | Shared implementation used by ClientTradingCardGameManager as well as the base manager. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `0136ad50` | `Wrapper_0136ad50 ?` |  | Thiscall-style forwarder with one stack argument (ret 4). Inside an SEH frame it sets ECX to the stack argument param_2 (mov ecx,esi), pushes this+0x108 and calls 013691d0, then returns param_2. Ghidra shows param_1 as … | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136b7a0` | `CTP::Game::InsertKeyedRecordIfAbsent ?` | CTP::Game | Finds key param_2 in the map at this+0x200 (01331340). If the key is absent and param_4 is 1, it builds a record from param_3, param_5, param_6 and a string (01332310, 013326d0, 01333810, 01334c70), inserts a node throu… | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136c3e0` | `Wrapper_0136c3e0 ?` |  | Thiscall-style forwarder (ret 4): ECX is advanced by 0x140 to the map at this+0x140, the key pointer from the stack is pushed, and it calls 0136bd70 (TreeFindOrInsertIntKey) on that map. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136c550` | `SubmitToListsExceptKey ?` |  | Walks two 4-byte-element vectors (this+0x28..0x2c and this+0x48..0x4c). For each entry whose value from 01344e20 differs from param_2 it reads the object's virtual slot 0x44 through param_3 and submits via 0136c400. Fin… | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136c680` | `SubmitToListsExceptTwoKeys ?` |  | Same as 0136c550 with two excluded values: walks the vectors at this+0x28 and this+0x48 and submits each entry whose 01344e20 value differs from both param_2 and param_3 via 0136c400, then deletes param_4 through slot 0… | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136c9f0` | `PeekFirstElementForKey ?` |  | Fastcall with the key on the stack (ret 4). Same set and vector lookup as PopFirstElementForKey (0136c900) but does not remove: returns the first 4-byte element of the mapped vector, or 0 when the key or vector is empty. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136ce20` | `ReadCountedKeyValueEntries ?` |  | Fastcall: calls 013c2690 for a count. For each entry it reads a record into a local buffer through 01401a70 on *param_1; on a failed read it cleans up through 0135ad00 and returns 0. Otherwise it takes the first two dwo… | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |

## ⚪ unknown (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0136adc0` | `— ⚠reject` |  | Adds a command tagged with a card to the per-key command table of the card game engine. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136aed0` | `— ⚠reject` | CTP::Game | Moves the commands of one duration key to another key; the log text names it changeDuration. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136b200` | `— ⚠reject` | CTP::Game | Removes the entries of one command-list key whose compared value matches; the game meaning of that value is not established. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136ba00` | `— ⚠reject` | CTP::Game | Moves one player's duration-bearing commands from one key to another. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136c400` | `— ⚠reject` |  | Takes ownership of command param_3. It calls the command's virtual slot 0x4c to receive a string-like result buffer that is freed on exit. When the byte at this+0x388 is set: a dynamic_cast test (00d8d382 against Networ… | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136c7c0` | `— ⚠reject` |  | Walks the ordered container whose sentinel is at this+0x144. For each node whose key (node+0xc) is not param_2, it looks up the mapped vector via 0136bd70, deletes every non-null pointer in that vector through its slot … | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136c900` | `— ⚠reject` |  | Fastcall with the key on the stack (ret 4). Finds the key in the set at this+0x140 (01354a70). When found and the mapped vector (0136bd70) is non-empty, it returns the first 4-byte element, shifts the rest down with _me… | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |
| `0136cb60` | `— ⚠reject` | CTP::Game | Thiscall: if the call through 01350660 on param_2 and 01344f00() gives a non-null object, it reads a vector of ints from that object through virtual slot 0x4c, and for each element increments a counter keyed by that int… | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |

