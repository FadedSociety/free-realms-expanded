# System: minigame

36 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a5850` | `IsMiniGameFlashType` |  | Predicate over the current minigame type (queried through FUN_009ad0b0/FUN_009ad290): true for types 3, 6-9, 14-16, 22-23 and 1009, which the action dispatcher 008a7eb0 routes through the flash-minigame key path (mode 4… | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a6b50` | `StartPendingMiniGameIfState2` |  | State-2 gate for the pending-minigame start path. The address-citing note links the target to the opcode-39/sub-5 ready acknowledgement. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `009ad480` | `ApplyMiniGameGameOverResult` |  | Consumes already-decoded result fields and updates client state; the cited note says score-screen presentation occurs elsewhere. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009ad830` | `SetEndGameDialogClosed` |  | Logs the close event, clears state byte +0x2c0, and sets byte +0x2c1; the event string carries the feature name. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009b0c70` | `ClientMiniGameManager::UpdateStateFlags` | ClientMiniGameManager | Updates manager flags for two state slots. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009b0cc0` | `SetMiniGameReadyAndDispatchEvent` |  | Sets local readiness state and notifies the minigame UI event handler; this function does not send a network packet. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `00c42440` | `ObjectiveStatus_FormatText` |  | Produce the localized status text shown for one minigame goal state record (the record FindRecordByKey_Stride0x110 returns). | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |

## 🟢 likely (25)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009acf60` | `PortalRows_GetCount` |  | Portal row count for the known portal sink caller. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009acf70` | `PortalRow_GetNext` |  | Caller 00c478b0 uses this return as the next portal-row pointer; the sink value comes separately from row+0x20 in that loop. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009ad0b0` | `MiniGameStateLookup` |  | Resolves state for minigame handlers and data sources. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009ad100` | `MiniGameStateLookupRepeat` |  | Resolves state for goal and score data sources. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009ad150` | `GetCurrentStatePointer` |  | Exposes current state to minigame datasource code. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009ad160` | `GetCurrentStateScoreBlock` |  | Provides the score block used by score-table accessors. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009ad3c0` | `HasPositiveCountAt20` |  | Used as a minigame-active guard by gameplay callers. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009ad420` | `SetCurrentStateFlag2b8` |  | Called by minigame quit handling; flag meaning unresolved. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009ad8d0` | `GetCurrentStateField0c` |  | Opaque state accessor used by a command dispatcher. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009ad8f0` | `GetCurrentStateField4` |  | Opaque state accessor. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009ad900` | `GetCurrentStateField0` |  | Opaque state accessor. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009aef40` | `CountGroupEntriesNamedChallenge` |  | Counts challenge entries in a minigame group. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009af660` | `InitMiniGameDataRecord` |  | Likely minigame data row constructor based on its unlocked-data parser callers. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009af7b0` | `InitMiniGameRecordVariant` |  | Likely minigame group row initializer based on its table parser caller. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009af860` | `InitMiniGameRecordVariant2` |  | Likely row initializer from minigame table parsing callers. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009af900` | `InitMiniGameGroupElementRecord` |  | Likely group-element record constructor based on array growth caller. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009afd00` | `MiniGameDataRecord_Dtor` |  | Cleanup for a multi-string minigame data record. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009afe40` | `MiniGameGroupElementInfo_StringDtor` |  | Cleanup for a string-containing minigame record. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009afeb0` | `MiniGameRecordTwoStringDtor` |  | Cleanup for a two-string minigame record. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009b0b20` | `MiniGameDataRecord_Dtor` |  | Cleanup for minigame data records. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009b0db0` | `DispatchMiniGameExitEvent` |  | When this+0x18 is nonzero, resolves HandlerMiniGameStart, appends :exit, and dispatches the local UI event; class identity is unresolved. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009b1440` | `MiniGameGroupElementInfo_CopyCtor` |  | Group-element record copy constructor based on its append callers. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009b1560` | `MiniGameScoreInfo_CopyCtor` |  | Score-info array element copy constructor based on its callers. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009b16e0` | `MiniGameStateListInsertTail` |  | Adds a created state to manager's active-state collection. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `00c42400` | `FindRecordByKey_Stride0x110` |  | Find one runtime minigame goal state by goal id in the active minigame object; the opcode-45/3 and 45/4 handlers 009b6700/009b6950 set the found record's status dword (+0x30) and re-format its text. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c42740` | `— ⚠reject` |  | Mirror server-side goal counters into one client goal record and refresh its text without touching the id or status. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c42870` | `— ⚠reject` |  | Materialise one runtime goal state for the client minigame from its loaded data, with a human-readable name guaranteed. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c42a30` | `— ⚠reject` |  | Zero-initialise one minigame instance (goal list, strings, score arrays) before its id is stored at +0x918 by 009b8190. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c42c60` | `— ⚠reject` |  | Regenerate the client's goal records for a minigame from its loaded goal data, used by the populate/start paths 009bb2d0 and 009beb70. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |

