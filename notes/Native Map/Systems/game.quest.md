# System: game.quest

53 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (16)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a98a0` | `HashListMap_SignatureGuid_Construct → HashListMap_SignatureCharacterGuid_Constr…` | SoeUtil::HashListMap | Initialises the map keyed by the SoeUtil signature-character guid strong type (used where quests are tracked per character signature). | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `0091bb40` | `QuestRuntimeNodeCtor` |  | Part of runtime quest creation: the vault note records that both full-self (00940c60) and QuestAdd (00c7bd10) insert through 009225d0 -> 0091bb40 -> 00baf490 -> 00baf380, where 00baf380 loops raw objective nodes. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `009225d0` | `QuestRuntimeNodeCreateAndAppend` |  | Runtime quest node creation that appends to the quest list; 00940c60 and 00c7bd10 insert through it. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00922810` | `QuestObjectiveConstructWithKey_F0` |  | Raw quest-objective step that stores the tuple key of a temporary quest objective at +0xf0; the vault note cites 00922810 for that store. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `0092b050` | `ClearTemporaryQuestObjectiveRecords` |  | Processes temporary raw quest-objective records using tuple keys distinct from definition IDs. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `0093c840` | `DecodeQuestRecord` |  | Decode the native quest-record shape cited by the opening-quest analysis. | [[Native Map/Shards/S0248 009392c0-0093d740|S0248]] |
| `00940c60` | `ReadQuestCollectionAndTail` |  | Reads quest data into live state; cited notes document insertion and objective-key normalization. Runtime class name is unknown. | [[Native Map/Shards/S0249 0093daf0-00942c60|S0249]] |
| `00a99220` | `QuestStartHandler::DismissEndScreen` | QuestStartHandler | Dismisses the quest start end screen by posting its named event with a gating code computed from the player state. | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00bab950` | `CopyObjectiveWithRuntimeKey` |  | Copies an objective record and assigns its runtime key from the supplied definition ID. | [[Native Map/Shards/S0364 00ba74d0-00bac070|S0364]] |
| `00baba60` | `InsertObjectiveByRuntimeKey` |  | Inserts a runtime objective keyed by its definition ID. | [[Native Map/Shards/S0364 00ba74d0-00bac070|S0364]] |
| `00c79fc0` | `SoeUtil::HashListMap_int_ClientQuestData_256_neg1::Ctor` | SoeUtil::HashListMap<int,ClientQuestDat… | Quest map container with 256 buckets, keyed by quest id with ClientQuestData values (element type from the vtable name). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c7a910` | `ApplyObjectiveActivated` |  | Applies an objective-activation update to the local quest record. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7a990` | `ApplyObjectiveUpdate` |  | Applies a live objective-counter update to the local journal state. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7aa10` | `ApplyObjectiveComplete` |  | Applies objective completion to local journal state. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7bd10` | `ApplyQuestAddPacket` |  | Adds a decoded quest to local quest state. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7c130` | `RemoveQuestRecord` |  | Removes a completed quest record from local state. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |

## 🟢 likely (29)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a9ba0` | `HashListMap_ObjectiveData_Construct ⚠downgrade` | SoeUtil::HashListMap | Initialises the 16-bucket objective map used by the quest/objective runtime. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `008ad140` | `HashListMap256_UnlinkNode_190` |  | Removal step for the client quest map whose nodes hash on their +0x190 byte. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `0091bbb0` | `QuestRuntimeNodeCtor_2` |  | Second runtime-quest helper in the same insertion family (caller 00922650, sibling of the 009225d0 chain); the parallel structure indicates a second node/collection type used by the same loaders. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091bc30` | `Ctor_Call009142f0_Store0x214` |  | Member constructor used by the same quest-object family as 0091bb40/0091bbb0 (caller 009226f0); the stored dword is a key/id for the new node but its exact meaning is not established. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091c8b0` | `Ctor_EmbedRewardBundleAt0x18` |  | Record constructor in the quest-data family: the raw quest reader 00922810, QuestObjectiveAddedPacket::Construct, 009b9f40/009baf90 and 00bab790 all build objects through it, so it is the decoded objective/quest record … | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091c940` | `Ctor_EmbedRewardBundle30_HashListObjectiveData` |  | Quest record constructor (objectives + rewards): called by the full-self loader 00940c60 and by QuestAddPacket::Construct (00c7bf10), which matches the quest record that QuestAdd delivers. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091c9f0` | `Ctor_EmbedRewardBundle_HashListObjectiveData_PairIds` |  | Second quest-record constructor (called from the full-self path 00940e60 and from 00bab710/00bab8e0), carrying two id pairs plus the objective hash list and reward bundle. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0092ac70` | `ClientQuestData::UnlinkAndDestroy` | ClientQuestData | Removes one quest record from its owning client quest map. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00940e60` | `ReadCountedQuestRecordsAndTail` |  | Reads a shorter quest-record variant, inferred from shared record constructors and field offsets. | [[Native Map/Shards/S0249 0093daf0-00942c60|S0249]] |
| `00a43240` | `GetQuestStringAt15C` |  | String getter over a live quest record used by the journal data source getters (caller 00c98170); the quest-record binding is neighbour-derived from the accepted callee labels (FindQuestRecord) and the empty-string fall… | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a43270` | `GetQuestStringAt16C` |  | Second string getter of the same journal getter pair; both are consumed by 00c98170. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00babc00` | `CopyRecordAndObjectives` |  | Copies a record with an owned collection of runtime-keyed objectives and embedded data. | [[Native Map/Shards/S0364 00ba74d0-00bac070|S0364]] |
| `00baf380` | `ObjectiveData_CopyCtor` |  | Constructs a runtime ObjectiveData entry from a source definition during quest creation. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `00baf490` | `RuntimeNode_ObjectiveCollectionCtor` |  | Builds the objective collection of a runtime quest node. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `00c24400` | `RequestProgressiveQuestStart` |  | Requests starting the specified progressive quest. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c244d0` | `RequestProgressiveQuestClaimSlot` |  | Requests claiming a progressive-quest slot. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c245a0` | `RequestProgressiveQuestClaimPrize` |  | Requests claiming a progressive-quest prize. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c7a610` | `FindQuestRecord` |  | Looks up an active runtime quest record. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7a670` | `AllocateQuestRecord` |  | Allocates storage used by the runtime quest map. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7a6b0` | `FreeQuestRecord` |  | Releases a runtime quest-record allocation. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7a890` | `SelectHashRecordPointer` |  | Updates local selected-quest state used by the journal. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7aa90` | `ApplyObjectiveFailed` |  | Applies an objective-failure update to local quest state. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7ab00` | `ReapplyStoredSelection` |  | Reapplies the current quest selection to the journal helper. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7ab20` | `RepairSelectionAfterRecordRemoval` |  | Repairs journal selection and UI state after a quest record is erased. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7c190` | `RemoveQuestRecord` |  | A duplicate compiled entry for local quest removal; exact distinction from adjacent copies is not visible in pseudocode. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7c1f0` | `RemoveQuestRecord` |  | A duplicate compiled entry for local quest removal; exact distinction from adjacent copies is not visible in pseudocode. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7c660` | `ApplyAddedObjective` |  | Copies a newly decoded objective into its runtime quest record. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7c8a0` | `ReleaseFourReferencesAndStore` |  | Cleans up quest-manager references and owned quest storage. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7ca80` | `ConstructQuestDataStore` |  | Initializes the quest-data storage and journal datasource members of a client data object. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |

## 🟡 speculative (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a97d70` | `SendQuestReplyPacket ?` |  | Sends the client's quest reply for the pending quest id it holds at +0x1c and tears down the pending-reply state; the class comes from constructor 00a86400, whose body stores QuestReplyPacket::vftable. No callers appear… | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00a97e10` | `SendQuestReplyPacketAndReset ?` |  | Second quest-reply send path that additionally runs the UI step 008c5d90, tears down the pending reply and restores default field pair; the class comes from constructor 00a86400 (QuestReplyPacket::vftable). No callers i… | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00a97ef0` | `SendQuestEndReplyPacket ?` |  | Sends the client's quest-end (turn-in) reply for the pending quest id and tears down the pending state; the class comes from constructor 00a864c0, whose body stores QuestEndReplyPacket::vftable. No callers in the databa… | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00a97f90` | `SendQuestStartBreadcrumbPacket ?` |  | Sends the quest-start breadcrumb notice for a quest id passed in; the class comes from constructor 00a86580, whose body stores QuestStartBreadcrumbPacket::vftable. Called from 00c18aa0 (outside the shard). | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00cf7b70` | `FindRuntimeQuestById ?` |  | Resolves a runtime quest record from its identifier. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00922650` | `— ⚠reject` |  | Runtime quest node creation of the second copy, called from the counted quest reader 00940e60. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00c7c540` | `— ⚠reject` |  | Updates local selected-quest state and requests matching server state. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7c5d0` | `— ⚠reject` |  | Sets the quest-helper auto-switch flag locally and requests its remote update. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |

