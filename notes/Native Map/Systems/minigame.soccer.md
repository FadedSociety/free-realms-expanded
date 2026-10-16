# System: minigame.soccer

128 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (50)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00afb7a0` | `ControllerSoccer::ControllerSoccer` | ControllerSoccer | Constructs the ControllerSoccer state implementation and sets its initial soccer camera state. | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00b73d40` | `GetBallForTeamIndex` |  | Ball lookup by team used by the family-76 dispatch (the vault note quotes the call at 0x00B82871 as 'get the ball'). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73e00` | `GetStateScriptName` |  | Returns the animation script name for the current soccer state; the referenced table entry carries the string scr_loc_idle. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75660` | `WorldToFieldNormalized` |  | Reads an entity's world position in field-normalized coordinates; the vault note cites FUN_00b75660 as the transform reader used by the target-directed vector getter FUN_00cd9df0, and its inverse is FUN_00b756d0. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b759e0` | `SoccerProcessor::SetTeamObject` | SoccerProcessor | Assigns the per-team controller object on the soccer processor (counterpart of the release path FUN_00b73fb0). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75c30` | `ListSoccerPlayerStatus::AllocNode` | SoeUtil::List<SoccerPlayerStatus,-1> | Node allocator for the tracked-player-status list of the soccer field object; the class binding is read straight from the binary vtable (slot 2 of SoeUtil::List<SoccerPlayerStatus,-1>). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75c70` | `ListSoccerPlayerStatus::FreeNode` | SoeUtil::List<SoccerPlayerStatus,-1> | Node deallocator paired with FUN_00b75c30 for the tracked-player-status list. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75d30` | `ListSoccerPickup::AllocNode` | SoeUtil::List<SoccerPickup,-1> | Node allocator for the soccer pick-up list; class binding read from the binary vtable (slot 2 of SoeUtil::List<SoccerPickup,-1>). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75d70` | `ListSoccerPickup::FreeNode` | SoeUtil::List<SoccerPickup,-1> | Node deallocator paired with FUN_00b75d30. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b7e2d0` | `SetSoccerProcessorActive` |  | Starts or tears down the client-side soccer processor state. | [[Native Map/Shards/S0355 00b7c520-00b807c0|S0355]] |
| `00b7e910` | `UpdateSoccerProcessorView` |  | Runs the soccer processor's per-frame visual and scoreboard updates. | [[Native Map/Shards/S0355 00b7c520-00b807c0|S0355]] |
| `00b80080` | `SoccerProcessor::Ctor` | SoccerProcessor | Initializes the native soccer processor and its client-side resources. | [[Native Map/Shards/S0355 00b7c520-00b807c0|S0355]] |
| `00b807c0` | `SoccerProcessor::Dtor` | SoccerProcessor | Tears down the native soccer processor and its owned client resources. | [[Native Map/Shards/S0355 00b7c520-00b807c0|S0355]] |
| `00b83330` | `SoccerProcessor::ScalarDeletingDtor` | SoccerProcessor | Deleting destructor of SoccerProcessor: slot [0] of SoccerProcessor::vftable. | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00cd7fd0` | `SoccerPlayer::IsFloatFieldNonpositive` | SoccerPlayer | Implements SoccerPlayer virtual slot 8 as a nonpositive-field predicate. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd7ff0` | `SoccerPlayer::GetBallWhenAvailable` | SoccerPlayer | Implements SoccerPlayer virtual slot 4 with a guarded ball lookup. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8010` | `SoccerPlayer::ForwardFloatThroughMappedMethod` | SoccerPlayer | Implements SoccerPlayer virtual slot 36 as a float-forwarding helper. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8020` | `SoccerPlayer::ResetVectorFromField100` | SoccerPlayer | Implements SoccerPlayer virtual slot 11 by rebuilding the four-float vector at +0x30. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8050` | `SoccerPlayer::CallTrackedEntityHelperAndReturnArgument` | SoccerPlayer | Implements SoccerPlayer virtual slot 13 with a helper call and pass-through return. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8080` | `SoccerPlayer::CopyFourFloatDirection` | SoccerPlayer | Implements SoccerPlayer virtual slot 12 as a four-value direction copy. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd80b0` | `SoccerPlayer::GetGlobalLookupField` | SoccerPlayer | Implements SoccerPlayer virtual slot 9 as a lookup-backed field getter. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8330` | `SoccerPlayer::HasNearbyActiveRecord → SoccerPlayer::HasNearbyActiveBall` | SoccerPlayer | Implements SoccerPlayer virtual slot 18 as an active-record proximity check. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd89b0` | `SoccerPlayer::UpdateMappedResource` | SoccerPlayer | Implements SoccerPlayer virtual slot 22 for mapped resource updates. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd89e0` | `SoccerPlayer::UpdateMappedRecordState` | SoccerPlayer | Implements SoccerPlayer virtual slot 1 for a mapped-record state update. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8b00` | `SoccerPlayer::WriteMappedTransform` | SoccerPlayer | Implements SoccerPlayer virtual slot 23 for mapped transform propagation. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8be0` | `SoccerPlayer::GetControllerField1E8` | SoccerPlayer | Implements SoccerPlayer virtual slot 5 as a guarded controller-field getter. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8c00` | `SoccerPlayer::GetControllerField148` | SoccerPlayer | Implements SoccerPlayer virtual slot 6 as a guarded controller-field getter. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8c20` | `SoccerPlayer::OnStateChanged → SoccerPlayer::SetStateWithDuration` | SoccerPlayer | Implements SoccerPlayer virtual slot 21 as a state transition handler. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8f50` | `SoccerPlayer::SetHorizontalDirection` | SoccerPlayer | Implements SoccerPlayer virtual slot 30 for horizontal direction updates. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd9010` | `SoccerPlayer::GetControllerVector` | SoccerPlayer | Implements SoccerPlayer virtual slot 10 as a four-value controller getter. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd9130` | `SoccerPlayer::GetMappedValue6E4` | SoccerPlayer | Implements SoccerPlayer virtual slot 16 as a mapped-value getter. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd9180` | `SoccerPlayer::GetMappedValue6E8` | SoccerPlayer | Implements SoccerPlayer virtual slot 17 as a mapped-value getter. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd91d0` | `SoccerPlayer::GetNestedControllerObject` | SoccerPlayer | Implements SoccerPlayer virtual slot 19 as a guarded nested-object getter. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd9200` | `SoccerPlayer::BuildOrientationMatrix` | SoccerPlayer | Implements SoccerPlayer virtual slot 15 for orientation matrix construction. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd9570` | `SoccerPlayer::SetActionResource` | SoccerPlayer | Implements SoccerPlayer virtual slot 24 as an action-resource state update. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd9af0` | `SoccerPlayer::OnStateChanged` | SoccerPlayer | Implements SoccerPlayer virtual slot 28 as the state-change handler. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd9df0` | `SoccerPlayer::GetTargetDirectedVector` | SoccerPlayer | Implements SoccerPlayer virtual slot 14 as a target-directed vector getter. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cda080` | `SoccerPlayer::InitializeAnimationAssets` | SoccerPlayer | Initializes SoccerPlayer soccer animation and effect resource references. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cdb1a0` | `SoccerPlayer::~SoccerPlayer` | SoccerPlayer | Destroys SoccerPlayer state and releases its referenced client resources. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cdbea0` | `SoccerPlayer::ScalarDeletingDestructor` | SoccerPlayer | MSVC scalar deleting destructor for SoccerPlayer. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdcda0` | `SoccerBall::CleanupAttachedHandles` | SoccerBall | SoccerBall virtual cleanup path for attached objects. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdcdb0` | `SoccerBall::SetAttachedStateBits` | SoccerBall | Changes the selected soccer-ball component state and notifies the component. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdcfd0` | `SoccerBall::EnsureHandle164` | SoccerBall | Lazily creates the soccer-ball attachment handle at +0x164. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdd9e0` | `SoccerBall::DestructorBody` | SoccerBall | Destroys SoccerBall-owned references and embedded subobjects. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cddd80` | `SoccerBallBase::GetFieldB0` | SoccerBallBase | Shared SoccerBallBase/SoccerBall virtual getter for field +0xb0. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cddde0` | `SoccerBall::FillFourDwordOutput → SoccerBall::GetPositionVec4` | SoccerBall | Fills a four-dword output from the current soccer-ball component data. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdde90` | `SoccerBall::FillThreeDwordOutput → SoccerBall::GetPositionVec3` | SoccerBall | Fills a three-dword output from the current soccer-ball component data. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cddf40` | `SoccerBall::ScalarDeletingDestructor` | SoccerBall | MSVC scalar deleting destructor for SoccerBall. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cddf60` | `LoadSoccerBallEffectDefinitions` |  | Loads the two named soccer-ball effect definitions and their associated handles. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cde1c0` | `SoccerBall::Constructor` | SoccerBall | Constructs a SoccerBall object and its embedded asset/requester state. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |

## 🟢 likely (48)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b73b20` | `ReleaseSoccerSoundSlots` |  | Bulk release of the five definition/handle pairs set up by FUN_00b77d70 and FUN_00b75460 (the MG_Soccer cue sounds); called from FUN_00b807c0 (a soccer processor entry outside this shard). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73c50` | `AllSoundDefsSlot3True` |  | Aggregate readiness/validity check over the same five definition slots that FUN_00b73b20 releases (the MG_Soccer cue-sound set per FUN_00b77d70); no caller is recorded in the export. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73cb0` | `CallSoundHandlesWithFloat` |  | Broadcast of a scalar value to the five handle objects created by FUN_00b75460 for the MG_Soccer cue sounds; called from FUN_00b83710, the client registration path. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73d70` | `SetFlag5e6` |  | Second flag set adjacent to FUN_00b73d60 in the same object; no caller is recorded in the export. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73dd0` | `GetControllerStateOrThree` |  | Reads the current state value of the attached controller/session object, defaulting to 3 when no controller is attached; called from FUN_00cd7130. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73df0` | `GetStateScriptId` |  | Returns the id half of the current state's row in the state table whose string half (FUN_00b73e00) holds animation script names like scr_loc_idle. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73fb0` | `ReleaseTeamObjectPair` |  | Teardown of a team's script object (companion of FUN_00b73e70); called from FUN_00b74020, FUN_00b7e2d0 and FUN_00b807c0. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74020` | `ReleaseBothTeamObjects` |  | Releases both teams' script objects at once, e.g. when the match or activity ends. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74290` | `Thunk00cd6a30WithThis → SoccerPickupDestructorEHThunk` |  | Adapter that lets the soccer dispatch call the -1 COMDAT helper FUN_00cd6a30 through a stable address. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b744b0` | `ForwardThreeDerefs → SoccerPickupCtorAdapter` |  | Call-site adapter for the soccer helper FUN_00cd6a70 taking stack-pointer arguments; called from FUN_00b75d90. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75750` | `FieldNormalizedToWorldCopy` |  | Second emission of the normalized-to-world mapping for the target-directed getter FUN_00cd9df0 (its only recorded caller). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b757c0` | `GetField30c` |  | Reads the count field used by the list walkers FUN_00b757d0 and FUN_00b75ac0. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b757d0` | `ListNthFrom304` |  | Indexed access into the tracked-entity list used by the soccer field code; called from FUN_00cd9df0. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75ac0` | `CallSlot6cOnMatchingEntities ⚠downgrade` |  | Broadcast notification to tracked entities matching an id (used by the soccer client when an entity's state changes). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75c90` | `InitEightNodePool` |  | Initialises the eight-node pool used by the caller-instantiated SoeUtil::List<SoccerPlayerStatus,8> implementation; the caller installs that RTTI vtable before calling this helper. The links form a chain, not a ring. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b76050` | `FindTrackedObjectByPair` |  | Entity lookup by the guid/id pair used by the family-76 dispatch (callers FUN_00b771e0, FUN_00b7bce0, FUN_00b80ac0). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b76090` | `ReleaseHandlesViaManager` |  | Teardown of the soccer input/overlay handles held by the field object and by each tracked player; called from FUN_00b7e2d0. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b76180` | `FindTrackedNodeByPair` |  | Node lookup by the guid/id pair for callers that need the list node; called from the family-76 dispatch FUN_00b80ac0. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b761b0` | `AdvanceStateAnimation` |  | Steps the soccer state machine forward and pushes the new state's animation script to every tracked player; one of the two indexed by the input-action mapper FUN_00afb4a0. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b76210` | `ApplyPreviousAnimationState` |  | Decrements the tracked animation-state index and applies the selected prior table entry to each tracked entity; its caller is the input-action mapper 00afb4a0, whose UI action is labelled Previous animation. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b762f0` | `FindPickupByField4` |  | Pick-up lookup by id for the family-76 dispatch (caller FUN_00b80ac0); stride 0x20 matches the pick-up list nodes. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b76b50` | `FreeBinaryTree` |  | Tree/free-list teardown used by FUN_00b77a70, FUN_00b77eb0, the soccer cleanup paths and recursively by itself. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b76f60` | `InitTwoStringArrays` |  | Builds the object behind the soccer field logic (the same object that owns the +0x304 player list and +0x214 client); called from FUN_00b80080. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b77030` | `DtorTwoStringArrays` |  | Teardown half of FUN_00b76f60, used when the soccer field object is destroyed (callers include FUN_00b807c0 and the EH funclets). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b771e0` | `SetLocalPlayerFlagC0 ⚠downgrade` |  | Per-player flag set driven by the local player's own guid (a client-side hint on the tracked entity); called from FUN_00b7de60. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b77a70` | `TreeClearAndFree` |  | Teardown of a soccer tree/free-list member; no caller is recorded. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b77eb0` | `TreeClearAndFree8 → SoeUtil::List<SoccerPlayerStatus,8>::TreeCleanup` |  | Teardown of the second soccer tree/list member (the free-list built by FUN_00b75c90); candidates are the EH funclets at 015e09b8/015e0a18. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00cd6af0` | `ComposeAndApplyObjectTransform` |  | Applies a composed transform in the SoccerProcessor family-76 handling path. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd70b0` | `SetIndexedSoccerActionFlag` |  | Marks one indexed soccer action flag. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd7900` | `SoccerStateName` |  | Provides human-readable names for soccer processor states. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd7a70` | `SoccerPlayer::UpdateTimedResources` | SoccerPlayer | Advances timed SoccerPlayer resource state during its update method. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd7c30` | `SoccerPlayer::ReleaseResourceReferencePairs` | SoccerPlayer | Releases the reference pairs owned by the SoccerPlayer cleanup path. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd80c0` | `SetSoccerVectorFields` |  | Updates three vector-related fields from the caller-provided values. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd80f0` | `MapSoccerActionStateCode` |  | Converts soccer action-related state codes according to the object's mode predicate. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8470` | `ProcessResourceEntryOptionallyTwice` |  | Processes a resource entry from SoccerPlayer update/state paths. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd84c0` | `RunSixResourceEntries` |  | Processes a fixed group of soccer resource entries. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8640` | `ReleaseSoccerPlayerResources` | SoccerPlayer | Releases resource handles during SoccerPlayer cleanup. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8700` | `SoccerPlayer::UpdateResourceSlots` | SoccerPlayer | Updates SoccerPlayer resource slot state during the player update loop. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8e60` | `SoccerPlayer::RunConditionalUpdateHelpers ⚠downgrade` | SoccerPlayer | Implements SoccerPlayer virtual slot 35 by running its guarded update helpers. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd92d0` | `GetOrCreateMappedObject` |  | Resolves or creates an object associated with a selected soccer field. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd9370` | `ProcessSelectedResourceEntry` |  | Processes a selected resource entry from SoccerPlayer state paths. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd93c0` | `GetOrCreateKeyedObject` |  | Resolves or creates a keyed object used by soccer player resource paths. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd96a0` | `BlendAndScaleVectorAt50` |  | Processes one vector state field in the SoccerProcessor family-76 dispatch path. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd97d0` | `BlendAndScaleVectorAt30` |  | Processes a second vector state field in the SoccerProcessor family-76 dispatch path. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd9950` | `TransformAndLookupMappedObject` |  | Transforms input values before resolving the mapped object. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd9ad0` | `InitializeSoccerMappedState` |  | Initializes a keyed soccer state entry from the field at +0x37c. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cda6a0` | `CopyStructuredUpdateRecord` |  | Copies a structured record used by the SoccerPlayer update path. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cde480` | `InitializeObjectAndEffectState → SoccerBall::Initialize` |  | Called by SoccerProcessor::DispatchFamily76Packet while initializing soccer-ball state; the exact dispatch subtype is unresolved. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |

## 🟡 speculative (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b73d60` | `SetFlag5e5 ? ⚠downgrade` |  | Flag set used by the soccer object's state code; no caller is recorded in the export. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73e10` | `GetOwnerField10 ? ⚠rename` |  | Reads the object stored in the first field of the ball/free-ball record; called from the FreeBall send path FUN_00cdc710. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73e70` | `StartTeamHandle ? ⚠rename` |  | Starts the per-team script/sound object (goal or match announcement) for team 0/1; called from the family-76 dispatch and FUN_00b76320. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75a60` | `ClearTeamOverlayHandles ? ⚠rename` |  | Clears the finished per-team overlay objects (companions of the +0x618/+0x630 slot records). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75d90` | `ListAppendNode ? ⚠rename` |  | Generic append used by the soccer field lists (players/pickups) with the allocator pair FUN_00b75c30/FUN_00b75d30; no caller is recorded because it is reached through a vtable. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75e30` | `SoccerNodeScalarDtor ? ⚠rename` |  | Deleting destructor of the soccer list-node class whose destructor body is FUN_00cd6a30 (same base the wrapper FUN_00b74290 calls). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b76150` | `FindTrackedByField34 ? ⚠rename` |  | Entity lookup by the id stored at node+0x34; called from FUN_00cd73b0. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b76a90` | `ListRemoveAndFreeNode ? ⚠rename` |  | Removal path paired with the append helper FUN_00b75d90 for the soccer lists (tracked players and pickups); called from the dispatcher, the cleanup paths and FUN_00b77a00. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b77a00` | `ListRemoveAll ? ⚠rename` |  | Clears a soccer list (tracked players or pickups) in one call; no caller is recorded. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00cd6a70` | `SoccerPickup::SoccerPickup ?` | SoccerPickup | Constructs a SoccerPickup instance from the three supplied values and initializes its owned reference fields. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd6aa0` | `SoccerPickup::DeletingDestructor ?` | SoccerPickup | Implements the scalar deleting destructor for SoccerPickup. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cdb370` | `SoccerPlayer::UpdateDebugDisplay ? ⚠rename` | SoccerPlayer | Per-frame soccer-player update with a conditional soccer debug display. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdd100` | `SoccerBall::UpdateFiveEffectSlots ? ⚠rename` | SoccerBall | Updates the soccer-ball effect entries for five component types. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |

## ⚪ unknown (17)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00afbc60` | `— ⚠reject` | ControllerSoccer | Thiscall with one int arg (time step), vtable slot 51 (+0xcc). Returns unless the object at this+0x74 -> +0x120 -> +0x21c is nonzero. Reads 01071fc0 and sets flag +0x1c4 when it changes. Computes a clamped interpolation… | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00b75570` | `— ⚠reject` |  | Adapter that dispatches to the +0x20c sub-object's method FUN_00cd70b0 through a stable address (used by the soccer client code at 00cd89e0/00cd9af0). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b756d0` | `— ⚠reject` |  | Turns a field-normalized coordinate back into a world point, used by the family-76 dispatch when applying remote vectors to field positions. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b76320` | `— ⚠reject` |  | Per-team start with an explicit transform (used by the goal/announcement path); no caller is recorded in the export. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b763b0` | `— ⚠reject` |  | Feeds a caller-supplied value into the running per-team object (companion of FUN_00b73e70); no caller is recorded in the export. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b77480` | `— ⚠reject` |  | Computes the soft target direction used by the soccer assist/HUD code; callers FUN_00b7e910 and FUN_00cd8050. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b83710` | `— ⚠reject` |  | Per-frame tick of the soccer match state machine. Its callers are the PlayClient update and the world-ready loop. | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b839e0` | `— ⚠reject` |  | Advances the soccer state from 8 to 9 and sends the game state record for that step through the tunnel builder. | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b83ae0` | `— ⚠reject` |  | Builds the sub-10 AcquirePickUp packet record and tunnels it when the soccer object at this+0x214+0x280 exists. | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00cd6a30` | `— ⚠reject` | SoccerPickup | Releases a SoccerPickup object's owned and linked resources. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd79c0` | `— ⚠reject` |  | Computes a soccer-related ratio for its caller. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8ec0` | `— ⚠reject` |  | Supplies a mapped record to the SoccerPlayer callback wrapper. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd8f30` | `— ⚠reject` | SoccerPlayer | Implements SoccerPlayer virtual slot 29 as a mapped-record callback. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd9a00` | `— ⚠reject` | SoccerPlayer | Implements SoccerPlayer virtual slot 27 as a local event callback. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cda220` | `— ⚠reject` | SoccerPlayer | Constructs and initializes a SoccerPlayer instance. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cdd200` | `— ⚠reject` | SoccerBall | Applies either the supplied matrix or its interpolated form to the current soccer-ball component. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdd300` | `— ⚠reject` | SoccerBall | Per-frame soccer-ball update and attached-effect maintenance. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |

