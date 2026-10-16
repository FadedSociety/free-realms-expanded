# System: game.activity

157 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (26)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009c19a0` | `AcceptActivityLaunchInvite` |  | Accepts by invoking the direct InviteResponse sender; the local request node is not changed by this function. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c1a00` | `RemoveActivityLaunchRequest` |  | Removes a pending activity-launch request from manager indexes and list. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c1c60` | `DeclineActivityLaunchInvite` |  | Declines a pending activity-launch invitation. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c2230` | `ClientActivityLaunchProcessorStubCtor` | ClientActivityLaunchManager::ClientActi… | Initializes a named activity-launch processor stub. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c22c0` | `GetField08` |  | Shared getter at offset +8 across several class vtables. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c22d0` | `ClientActivityLaunchProcessorStubScalarDeletingDtor` | ClientActivityLaunchManager::ClientActi… | Deletes a processor-stub object when requested by the scalar-delete flag. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c2920` | `RegisterActivityLaunchProcessor` |  | Registers a name-keyed activity-launch processor and replaces an existing same-key entry. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c2cc0` | `ActivityLaunchManagerDtor` |  | Tears down the activity-launch manager state; the manager role is anchored by its request list and shutdown caller. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c2fc0` | `ActivityLaunchManagerInit` |  | Initializes client-side activity-launch request management. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c3270` | `HandleActivityLaunchRequest` |  | Processes a received activity-launch request; the merge path replaces/removes an earlier request rather than merging its fields. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c4c30` | `FormatTimerDefinitions` |  | Produces a diagnostic listing of timer definitions. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c4d70` | `FormatActiveTimerNames` |  | Produces a diagnostic listing of active timers. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c52b0` | `ClientTimerEventCtor` | ClientTimerEvent | Creates a client timer event derived from ExecuteLuaEvent. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c5330` | `ClientTimerEventScalarDeletingDtor` | ClientTimerEvent | Scalar deleting destructor for ClientTimerEvent. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c5990` | `LoadTimerDefinitions` |  | Loads local timer definitions from the client data file. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c5cd0` | `LoadSavedClientTimers` |  | Restores client timer records from a local persistence file. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c6670` | `SaveClientTimers` |  | Persists saved and active timer state to a local file. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `00be6700` | `ClientActivityManager::ClearLists` | ClientActivityManager | Empties the two linked lists held by the activity manager (+0x16868 and +0x16a7c). | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be6760` | `Tutorial_SetOrClearEntry` |  | Sets or clears a per-character tutorial entry keyed by the tutorial id. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be68c0` | `ActivityUserData_LoadTutorialIds` |  | Loads the stored tutorial ids for the current character from ActivityUserData.ini into the activity manager table. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be6d30` | `ClientActivityManager::LoadActivityFiles` | ClientActivityManager | Loads all activity definition data files for the client activity manager from its data directory. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be70b0` | `ClientActivityManager::ClientActivityManager` | ClientActivityManager | Constructs the client activity manager, its activity definition tables, and the activity finder data source. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00c4d730` | `ClientActivityLaunchRequest::ScalarDeletingDtor` | ClientActivityLaunchRequest | Polymorphic deletion entry for the client-side launch request object. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4e1b0` | `SendReadyNotification` | ClientActivityLaunchRequest | Client command that marks the local player ready for the launch; the vault review note records this as the packet reached from the Ui.ReadyForActivity binding via 00c1b1c0. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4e490` | `HandleActivityLaunchMessage` | ClientActivityLaunchRequest | Single entry point that routes every activity-launch message to its handler; it is the receive-side counterpart of the family constructors. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `01082860` | `ClientActivityDefinitionDtorBody` | ClientActivityDefinition | Destructor body of ClientActivityDefinition (activity definition record). | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

## 🟢 likely (68)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009c1060` | `ClientActivityLaunchRequestCtor` |  | The callee constructs ClientActivityLaunchRequest; caller 009c1540 then links the object into a list. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c15e0` | `ClientActivityLaunchRequestScalarDeletingDtor` |  | Scalar deleting destructor for ClientActivityLaunchRequest. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c17c0` | `FindActivityLaunchRequestById` |  | Searches the manager list head and successor links for a matching +0xa4 value; the activity-request interpretation comes from receive-handler caller 009c3590. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c1860` | `UnlinkAndDestroyActivityLaunchRequest` |  | Unlinks and destroys a ClientActivityLaunchRequest node, then releases its typed-list storage. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c1c20` | `RemoveNodesWithByteB4Set` |  | Walks the manager list, saves each successor, and calls 009c1a00 for nodes with nonzero byte +0xb4; the caller and removal callee provide the activity-request context, while the flag meaning is unknown. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c21b0` | `InitializeActivityLaunchPacketMember` |  | The packet-member role follows direct constructor callers including 00c4b3e0; this function itself has no class vtable. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c2350` | `ConstructActivityLaunchProcessorStub` |  | Direct callee 009c2230 constructs the named processor-stub type; the role of the additional scalar at +0x48 remains unknown. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c4100` | `FindTimerDefinitionByKey` |  | Looks up a timer definition; exact owning type is inferred from timer manager callers. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c4170` | `FindSavedTimerByKey` |  | Searches saved timer records; exact owning type is inferred from caller 009c6000. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c41f0` | `FindActiveTimerByString` |  | Searches active timer entries by string key; label follows callers 009c5590 and 009c5720. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c4230` | `ReadInt64FromDelimitedField` |  | Parses a saved-timer text field. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c42a0` | `ReadInt64FromDelimitedFieldAlt` |  | Parses a saved-timer numeric field; semantic field identity is not visible here. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c4630` | `SixIStringsInit → TimerDefinitionStringFieldsInit` |  | Initializes a timer-related aggregate string set. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c4820` | `InitializeSixStringTimerNode → InitializeTimerDefinitionNode` |  | Initializes an inserted timer-definition node; interpretation follows its use by 009c4e60. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c48e0` | `InitializeTimerStringNode → InitializeActiveTimerDataNode` |  | Initializes a node inserted by a timer container; interpretation follows caller 009c5050. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c49a0` | `TimerDataStringsScalarDeletingDtor` |  | Scalar deleting destructor wrapper for a timer-related string aggregate. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c4e60` | `InsertHashListNode` |  | Appends and buckets a newly allocated node; timer-definition loading uses this routine through caller 009c5990. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c4f00` | `InsertHashListNode` |  | Appends and buckets a new node; saved-timer loading uses this routine through caller 009c5cd0. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c5050` | `InsertHashListNode` |  | Appends and buckets a new node; timer-event creation uses this routine through caller 009c5350. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c5550` | `CreateTimerEventAtElapsedTime` |  | Schedules a timer event relative to the current clock. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c5590` | `StartClientTimer` |  | Starts a configured client timer by definition name. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c5720` | `StopClientTimer` |  | Stops a named timer and records its state for persistence when the timer flags request it. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c5f60` | `RestoreSavedTimersForClient` |  | Restores eligible saved timers for this client session. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c6000` | `StartAutoTimers` |  | Starts configured automatic client timers. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c63c0` | `ClientTimersInitialize` |  | Initializes timer configuration and persisted state during PlayClient startup. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c63e0` | `ClientTimersStart` |  | Advances the timer manager into its running state. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c65e0` | `ClientTimersManagerInit` |  | Initializes the client's timer manager. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c6ac0` | `ClientTimersManagerDtor` |  | Persists and tears down the client timer manager. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `00bdfc00` | `LocalClientActivityDefinition_DtorHelper_2` |  | Second destruction helper for a local ClientActivityDefinition; no caller is recorded in the map database. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00be58f0` | `LoadActivityCategories_txt` |  | Loads the activity category definitions from ActivityCategories.txt (caret-delimited lines, '#' comments) into the activity tables; the field meanings are not verified beyond the parser calls. | [[Native Map/Shards/S0375 00be0120-00be58f0|S0375]] |
| `00be71e0` | `ClientActivityManager_ScalarDtor` | ClientActivityManager | Scalar deleting destructor for ClientActivityManager. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00c4a390` | `ActivityLaunchRequest_FirstMember` |  | Read accessor used by the member-iteration code (caller 00c4f3d0) that walks the list through the +0x50 next links. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4a5e0` | `ActivityMember_NextNode` |  | Iteration step used by the same loop as 00c4a390 (caller 00c4f3d0). | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4b700` | `ClientActivityLaunchRequest_BuildDisplayName` | ClientActivityLaunchRequest | Produces the per-request prefix used by the ClientActivityLaunchRequests.log lines. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4b7a0` | `ActivityLaunchRequest_DumpMemberList` | ClientActivityLaunchRequest | Debug dump of the assembled group, called from the logging helper 00c4f190. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4bb70` | `ActivityMemberList_UnlinkAndFree` |  | Single-node removal used by the list destructor 00c4d370, the drain helpers 00c4cc70/00c4ccb0 and the RemoveMember logic 00c4c1a0. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4bc00` | `ActivityMemberList_InsertTailFromDescriptor` |  | Insert path used by the AddMember logic 00c4c070 when a member is not yet present. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4c070` | `ActivityLaunchRequest_AddMember` | ClientActivityLaunchRequest | Insert/update rule of the launch-member table, driven by the AddMember message handler 00c4d4a0. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4c1a0` | `ActivityLaunchRequest_RemoveMember` | ClientActivityLaunchRequest | Removal rule of the launch-member table (self-removal cancels the local request), used by the RemoveMember handler 00c4c6e0 and the kick path 00c4c6e0. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4c300` | `HandleInviteMemberReplyMessage` | ClientActivityLaunchRequest | Receive-side application of the server's answer to an invitation (inviteSent flag and result code) plus UI notification. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4c440` | `HandleInviteMemberResultMessage` | ClientActivityLaunchRequest | Receive-side application of the invited player's accept/decline outcome plus UI notification. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4c7f0` | `HandleKickEventMessage` | ClientActivityLaunchRequest | Applies a server-side kick, distinguishing 'I was kicked' from 'another member was kicked' for the UI. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4ca90` | `HandleOnMatchmakingLaunchMessage` | ClientActivityLaunchRequest | Applies the parsed launch notification to the request delegate; the original server's matchmaking state transition is not established by this handler. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4cb60` | `HandleOnMatchmakingCancelMessage` | ClientActivityLaunchRequest | Applies the server's matchmaking-cancel message so the request leaves the searching state. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4ce40` | `HandleUpdateMemberStateMessage` | ClientActivityLaunchRequest | Applies the server's per-member state flags (the seven bits logged in the other member handlers) to the local member record. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4d370` | `ClientActivityLaunchRequest::~ClientActivityLaunchRequest` | ClientActivityLaunchRequest | Non-deleting destructor of the launch request, called from five unwind funclets and from the scalar deleting destructor 00c4d730. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4d4a0` | `HandleAddMemberMessage` | ClientActivityLaunchRequest | Receive path that merges a server-side member record into the local launch request. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4d840` | `ApplyInviteDetails` | ClientActivityLaunchRequest | Turns an invite-details message into the local request state (owner guid, activity ids, member list). | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4d980` | `HandleInviteDetailsMessage` | ClientActivityLaunchRequest | Validation rule that cancels an invite the local player is not listed in. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4d9f0` | `InviteResponse_SerializeAndSend` |  | Transport step behind the 'Sent InviteResponse' sender 00c4dfe0. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4dac0` | `OwnerLaunchRequest_SerializeAndSend` |  | Transport step behind the 'Sent OwnerLaunchRequest.' sender 00c4e090. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4db90` | `OwnerMatchmakingRequest_SerializeAndSend` |  | Transport step behind the 'Sent OwnerMatchmakingRequest.' sender 00c4e120. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4dc60` | `ReadyNotification_SerializeAndSend` |  | Transport step behind the 'Sent ReadyNotification.' sender 00c4e1b0 (reached from the Ui.ReadyForActivity binding). | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4dd30` | `QuitNotification_SerializeAndSend` |  | Transport step behind the 'Sent QuitNotification.' sender 00c4e240. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4dfe0` | `SendInviteResponse` | ClientActivityLaunchRequest | Client command that accepts or declines an activity invitation. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4e090` | `SendOwnerLaunchRequest` | ClientActivityLaunchRequest | Client command that asks the server to start the activity for the group. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4e120` | `SendOwnerMatchmakingRequest` | ClientActivityLaunchRequest | Client command that asks the server to place the group into matchmaking. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4e240` | `SendQuitNotification` | ClientActivityLaunchRequest | Client command that tells the server the local player leaves the group. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4e2d0` | `SendPlayerInvite` | ClientActivityLaunchRequest | Client command that invites another character into the group. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4e3b0` | `SendPlayerRemove` | ClientActivityLaunchRequest | Client command that removes another character from the group. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `01082450` | `HashListMapActivityCtor16` | SoeUtil::HashListMap<int,ClientActivity… | Constructor of the 16-bucket SoeUtil::HashListMap keyed by int for ClientActivityDefinition::FeaturedActivityEntry (class name from RTTI). | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01082570` | `HashListActivitySlot1AllocNode` | SoeUtil::HashList<ClientActivityDefinit… | Node-allocate slot of the 16-bucket FeaturedActivityEntry hash list (0x30-byte nodes), container infrastructure. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `010825b0` | `HashListActivitySlot2FreeNode` | SoeUtil::HashList<ClientActivityDefinit… | Node-free slot of the 16-bucket FeaturedActivityEntry hash list (0x30-byte nodes, alignment 8). | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01082780` | `HashListMapActivityDtorBody` | SoeUtil::HashListMap<int,ClientActivity… | Destructor body of the 16-bucket FeaturedActivityEntry hash list; container cleanup. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `010827b0` | `HashListMapActivityScalarDeletingDtor` | SoeUtil::HashListMap<int,ClientActivity… | Scalar deleting destructor (vtable slot 0) of the 16-bucket FeaturedActivityEntry hash list. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `010827f0` | `HashListMapActivityDtorBodyEh` | SoeUtil::HashListMap<int,ClientActivity… | Destructor body of the 16-bucket FeaturedActivityEntry hash list, called from the ClientActivityDefinition destructor. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01082920` | `HashListActivityCtor16` | SoeUtil::HashList<ClientActivityDefinit… | Constructor of the 16-bucket HashList for ClientActivityDefinition::FeaturedActivityEntry nodes. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `010829a0` | `HashListActivityScalarDeletingDtor` | SoeUtil::HashList<ClientActivityDefinit… | Scalar deleting destructor (vtable slot 0) of the 16-bucket HashList for FeaturedActivityEntry nodes. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

## 🟡 speculative (38)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009c0ee0` | `DestroyClientActivityLaunchRequest ?` |  | The request-destructor role follows direct callee 00c4d370; the wrapper itself only adds exception handling. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009c46f0` | `TimerEventFieldsInit ? ⚠rename` |  | Initializes fields for a timer-related event object. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `009c4880` | `InitializeTimerEventNode ? ⚠rename` |  | Initializes a node inserted by the saved-timer container; interpretation follows 009c4f00. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `00bdffe0` | `InitRecordDefaultsAndStoreDword ? ⚠downgrade` |  | Builds one entry of an activity-related record table; no caller is recorded in the map database. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00ce7570` | `ClientRequirementCheckCountry::LoadValue ?` | ClientRequirementCheckCountry | Calls conversion helper on field +0x30 and stores its return at +0x60. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce75d0` | `CompareRequirementInteger ?` |  | Subtracts one from selector +0x1c and compares integers using equal, not-equal, less, less-or-equal, greater, or greater-or-equal. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce7660` | `CompareRequirementInteger ?` |  | Subtracts one from selector +0x1c and compares integers using equal, not-equal, less, less-or-equal, greater, or greater-or-equal. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce76f0` | `CompareRequirementInteger ?` |  | Subtracts one from selector +0x1c and compares integers using equal, not-equal, less, less-or-equal, greater, or greater-or-equal. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce7780` | `CompareRequirementFloat ?` |  | Subtracts one from selector +0x1c and compares the float against an integer using six equality/order operators. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce7860` | `ClientRequirementCheckCountry::Evaluate ?` | ClientRequirementCheckCountry | Reads country value from client context and compares it with loaded requirement value using the configured integer operator. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce7890` | `ClientRequirementCheckLanguage::Evaluate ?` | ClientRequirementCheckLanguage | Reads language value from client context and compares it with loaded requirement using configured integer operator. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce78c0` | `ClientRequirementCheckVipRank::Evaluate ?` | ClientRequirementCheckVipRank | Reads VIP rank from client context and compares it with stored float requirement using the configured operator. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce78f0` | `ClientRequirementCheckCurrentProfile::Evaluate ?` | ClientRequirementCheckCurrentProfile | Reads current profile selector and compares it against the stored value with configured integer operator. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce7920` | `ClientRequirementCheckRenamePending::Evaluate ?` | ClientRequirementCheckRenamePending | Checks profile API value 3 and applies selector 1 as equal-to-3 or selector 2 as not-equal-to-3. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce7980` | `ClientRequirementCheckLastPlayedProfileOfTypeRank::Evaluate ?` | ClientRequirementCheckLastPlayedProfile… | Finds profile entry keyed by stored type, resolves it through client map, and compares its rank with configured value. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce7a90` | `ClientRequirementCheckLanguage::LoadValue ?` | ClientRequirementCheckLanguage | Copies string at object +0x34, passes it to a conversion helper, and stores its result at +0x60. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce9170` | `LoadActivityCategoriesThunk ?` |  | Loads this+0x428 into ECX and tail-jumps to 00cfce70. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce9180` | `LoadActivityDefinitionsThunk ? ⚠rename` |  | Loads this+0x42c into ECX and tail-jumps to 00cfd070. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce9190` | `ActivityLoaderThunk_1 ? ⚠rename` |  | Loads this+0x428 into ECX and tail-jumps to 00cfbe20. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce91a0` | `ActivityLoaderThunk_2 ? ⚠rename` |  | Loads this+0x428 into ECX and tail-jumps to 00cfbe20. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce9ad0` | `ActivityMapThunk ?` |  | Adds 0x214 to ECX and tail-jumps to 00ce98d0. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce9ae0` | `ScheduledActivityMapThunk ?` |  | Adds 0x214 to ECX and tail-jumps to 00ce99e0. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cea3a0` | `ActivityDataSourceMap::FindOrCreate ?` | ActivityDataSourceMap | Looks up a datasource by hash and key; returns it if matched, returns null when creation is disabled, otherwise allocates a datasource, inserts it, and acquires a reference. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cea4c0` | `ScheduledActivityDataSourceMap::FindOrCreate ?` | ActivityDataSourceMap | Looks up scheduled datasource by hash and key; returns it if found, otherwise optionally allocates, inserts, and references a new scheduled datasource. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cea5f0` | `ActivityLists::RefreshFeatured ?` |  | Finds the Featured list, resets it, appends each activity whose featured byte at +0x31 is set, and optionally refreshes the list. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cea650` | `ActivityLists::RefreshPreferred ?` |  | Finds the Preferred list, resets it, tests each activity's visibility, adds visible ids, and optionally refreshes the list. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cea6c0` | `ActivityLists::RefreshMemberOnly ?` |  | Finds the MemberOnly list, resets it, appends activities whose byte at +0x3c is set, and optionally refreshes the list. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cea720` | `ActivityLists::Rebuild ?` |  | Resets All and related activity lists, adds every activity id and qualifying Featured/Preferred/MemberOnly/custom category ids, and records ranked activities. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cea940` | `ScheduledActivity::Start ? ⚠rename` |  | Finds or creates the scheduled activity source, configures it from the supplied value, invokes its virtual method at slot 14, and returns success. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cea980` | `ScheduledActivity::Update ?` |  | Looks up an existing scheduled activity source without creating it, then forwards the supplied value to its update helper. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cea9b0` | `ScheduledActivity::Remove ?` |  | Looks up an existing scheduled activity source without creating it and forwards to its removal helper when found. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cea9d0` | `ActivityList::AddId ? → ActivityList::EnsureIdPresent` |  | Finds or creates an activity list and appends the supplied activity id when the list is available. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceaa20` | `Activity::Reset ?` |  | Looks up an activity without creating it and invokes its reset helper when found. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cfb680` | `FindScheduledActivityById ?` |  | Looks up scheduled activity data by identifier; called by an activity datasource cell getter. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfb6c0` | `CompareScheduledActivityOrder ?` |  | Orders scheduled activities for a datasource or scheduler. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfba20` | `RemoveActivityRecordById ?` |  | Removes an activity record from a client collection. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfba70` | `RemoveScheduledActivityById ?` |  | Removes a scheduled activity record. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfc480` | `FindClientActivityById ?` |  | Looks up or creates an activity record in the client activity collection. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

## ⚪ unknown (25)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009c6040` | `— ⚠reject` |  | Processes completion of a client timer. | [[Native Map/Shards/S0274 009c2cc0-009c8d80|S0274]] |
| `00bdf120` | `— ⚠reject` |  | Clears the preferred and member-only activity lists; the two callees carry the resource names Activities.Preferred and Activities.MemberOnly. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdf490` | `— ⚠reject` |  | Destruction helper for a local ClientActivityDefinition, no caller is recorded in the map database; the same wrapper pattern appears four times in this range. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdf4e0` | `— ⚠reject` |  | Destruction helper for a local ClientScheduledActivityDefinition, no caller is recorded in the map database. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdfc50` | `— ⚠reject` |  | Second destruction helper for a local ClientScheduledActivityDefinition; no caller is recorded in the map database. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdfdd0` | `— ⚠reject` |  | Construction helper for a local ClientActivityDefinition; 00be0fc0 is the only caller. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdfe20` | `— ⚠reject` |  | Construction helper for a local ClientScheduledActivityDefinition; 00be1060 is the only caller. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00be5b30` | `— ⚠reject` |  | Loads the activity portal section definitions from the client data file ActivityPortalSections.txt. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be5d70` | `— ⚠reject` |  | Calls 00769710 and reads a 32-bit value at (result+4)+0x74; calls 00702670 to produce a 64-bit value; appends a node through 00be4ec0 (on *(param_1+4)); calls 01083010; then stores the low byte of 01082750(64-bit sum of… | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be6bb0` | `— ⚠reject` | ClientActivityManager | Destructor body of the client activity manager: flushes the activity user data file and releases its lists, maps and definition containers, then leaves the BaseDefinitionManager base. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00c4a550` | `— ⚠reject` | ClientActivityLaunchRequest | Lookup used to map an incoming guid to the local ClientActivityLaunchMember, e.g. to log 'we are not part of the invite' in 00c4d980. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4bf90` | `— ⚠reject` |  | Insert path used by the UpdateMemberState logic 00c4ccb0 when the addressed member is not yet present. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4c240` | `— ⚠reject` | ClientActivityLaunchRequest | Receive-side handler that forwards a parsed PlayerHasBeenAccepted object to the request delegate. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4c580` | `— ⚠reject` | ClientActivityLaunchRequest | Marks the local request as launched and records whether the local character is in the participant list. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4c6e0` | `— ⚠reject` | ClientActivityLaunchRequest | Applies a server-side member removal to the local launch request and informs the UI. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4c9c0` | `— ⚠reject` | ClientActivityLaunchRequest | Applies the server's 'matchmaking started' message to the local request and notifies the UI delegate. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4cc70` | `— ⚠reject` |  | List teardown used by the request destructor path and the UpdateMemberState parser 00c4ccb0. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4d2a0` | `— ⚠reject` | ClientActivityLaunchRequest | Creates the per-request state object that tracks the invited group, its members and its matchmaking phase. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4de00` | `— ⚠reject` |  | Transport step behind the 'Sent PlayerInvite. guid=%llu, name=%s' sender 00c4e2d0. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c4def0` | `— ⚠reject` |  | Transport step behind the 'Sent PlayerRemove. guid=%llu, name=%s' sender 00c4e3b0. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00ce79f0` | `— ⚠reject` | ClientRequirementCheckNudge | Scans context list for matching type/id fields, then returns match or its negation according to selector +0x1c. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cea090` | `—` |  | Initializes activity hash maps, clears two pointers, allocates/constructs shared activity objects of 0x47c and 0x368 bytes, then assigns them to fields. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cea190` | `—` |  | Releases two shared objects when the global gate is enabled, clears both fields, then destroys scheduled and activity maps. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceaa00` | `— ⚠reject` |  | Looks up an activity without creating it and forwards the provided value to the activity update helper when found. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceaa40` | `— ⚠reject` |  | Clears scheduled activity list, walks schedule entries, applies each entry value, then invokes list refresh. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00c4e490` | `HandleActivityLaunchMessage` | recv |  | 1/2/3/4/5/7/8/9/0xd… | Hypothesis: client usage suggests an original server-authoritative group flow: this dispatcher routes received family subtypes into local request-state handlers, while separate client functions send invite/ready/launch/matchmaking requests. This depends on dispatcher cases and handlers in this function plus the send paths 00c4dfe0 and 00c4e090..00c4e3b0. The original server code is unavailable, a… |
