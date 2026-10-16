# System: debug.freeform_actor

47 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (26)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009c95b0` | `FreeformActorManipulationEventHandler::Dtor` | FreeformActorManipulationEventHandler | Cleans the event-handler base during unwinding. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c95c0` | `FreeformActorManipulationEventHandler::ScalarDeletingDtor` | FreeformActorManipulationEventHandler | Scalar deleting destructor for the event-handler base. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9750` | `FreeformActorManipulationModeBase::GetField0C` | FreeformActorManipulationModeBase | Shared getter also present in PlaceActor and ManipulateActor mode vtables. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9770` | `FreeformActorManipulationModeBase::Dtor` | FreeformActorManipulationModeBase | Destroys the base portion of the concrete manipulation modes. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c97f0` | `FreeformActorManipulationModeManipulateActor::StoreVectorAndNotifyChild` | FreeformActorManipulationModeManipulate… | Stores a four-component value and notifies the child handler. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9830` | `FreeformActorManipulationModeManipulateActor::ForwardChildSlot4` | FreeformActorManipulationModeManipulate… | Forwards this mode operation to the active child. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9850` | `FreeformActorManipulationModeManipulateActor::ForwardChildSlot1` | FreeformActorManipulationModeManipulate… | Forwards this mode operation to its active child. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c98f0` | `FreeformActorManipulationModeManipulateActorInternalBase::HandleActiveInputTran…` | FreeformActorManipulationModeManipulate… | Tracks entry and exit input states for the internal mode. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9a90` | `FreeformActorManipulationModeScale::PassScaleVectorToChild` | FreeformActorManipulationModeScale | Passes Scale mode's vector to its child operation. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9c10` | `FreeformActorManipulationModeBase::ScalarDeletingDtor` | FreeformActorManipulationModeBase | Scalar deleting destructor for the mode base. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9cc0` | `FreeformActorManipulationModePlaceActor::HandlePlacementEvent` | FreeformActorManipulationModePlaceActor | Handles PlaceActor placement and transform event cases. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9fe0` | `FreeformActorManipulationModePlaceActor::ScalarDeletingDtor` | FreeformActorManipulationModePlaceActor | Scalar deleting destructor for PlaceActor mode. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ca000` | `FreeformActorManipulationModeManipulateActorInternalBase::ScalarDeletingDtor` | FreeformActorManipulationModeManipulate… | Scalar deleting destructor for the InternalBase mode. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ca450` | `FreeformActorManipulationModeManipulateActor::Dtor` | FreeformActorManipulationModeManipulate… | Destroys ManipulateActor mode handler, strings and base subobjects. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ca590` | `AdjustorThunk_009ca5a0` | FreeformActorManipulationModeManipulate… | Adjusts the secondary-base this pointer before forwarding to the shared deleting destructor. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ca5a0` | `FreeformActorManipulationModeManipulateActor::ScalarDeletingDtor` | FreeformActorManipulationModeManipulate… | Scalar deleting destructor for the ManipulateActor mode. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ca7d0` | `FreeformActorManipulationModePlaceActor::Ctor` | FreeformActorManipulationModePlaceActor | Constructs the PlaceActor manipulation mode. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ca8b0` | `FreeformActorManipulationModeManipulateActorInternalBase::Ctor` | FreeformActorManipulationModeManipulate… | Constructs shared state for Translation, Rotation and Scale modes. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ca9a0` | `FreeformActorManipulationModeTranslation::Ctor` | FreeformActorManipulationModeTranslation | Constructs the Translation specialization. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009caa70` | `FreeformActorManipulationModeTranslation::ScalarDeletingDtor` | FreeformActorManipulationModeTranslation | Scalar deleting destructor for Translation mode. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009caad0` | `FreeformActorManipulationModeRotation::Ctor` | FreeformActorManipulationModeRotation | Constructs the Rotation specialization. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009caba0` | `FreeformActorManipulationModeRotation::ScalarDeletingDtor` | FreeformActorManipulationModeRotation | Scalar deleting destructor for Rotation mode. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cac00` | `FreeformActorManipulationModeScale::Ctor` | FreeformActorManipulationModeScale | Constructs the Scale specialization. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cacc0` | `FreeformActorManipulationModeScale::StoreScaleVector` | FreeformActorManipulationModeScale | Stores the Scale mode's four-component vector. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cad30` | `FreeformActorManipulationModeScale::ScalarDeletingDtor` | FreeformActorManipulationModeScale | Scalar deleting destructor for Scale mode. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cb0b0` | `FreeformActorManipulationModeManipulateActor::Ctor` | FreeformActorManipulationModeManipulate… | Constructs the actor manipulation mode and its input/action state. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |

## 🟢 likely (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009c9790` | `FreeformActorManipulationModePlaceActor::SetPlacementVector ⚠downgrade` | FreeformActorManipulationModePlaceActor | Updates the vector supplied to the PlaceActor transform helper. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9870` | `FreeformActorManipulationModeManipulateActor::ForwardAdjustedInput ⚠downgrade` | FreeformActorManipulationModeManipulate… | Routes input through the mode and its secondary handler subobjects. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c98b0` | `FreeformActorManipulationModeManipulateActor::ForwardAdjustedPointerInput ⚠down…` | FreeformActorManipulationModeManipulate… | Routes pointer input through the mode and secondary handler subobjects. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9c30` | `FreeformActorManipulationModePlaceActor::Dtor ⚠downgrade` | FreeformActorManipulationModePlaceActor | Tears down PlaceActor-specific state before the base object. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9d10` | `FreeformActorManipulationModeManipulateActorInternalBase::Dtor ⚠downgrade` | FreeformActorManipulationModeManipulate… | Cleans the internal mode's action and base subobjects. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9da0` | `FreeformActorManipulationModeTranslation::DrawAxisHandles ⚠downgrade` | FreeformActorManipulationModeTranslation | Draws the Translation mode's axis handles. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ca350` | `CreateFreeformTargetEventHandlers` |  | Builds both target event handlers for the freeform manipulation context. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ca5c0` | `BuildTransformHandleArguments` |  | Shared setup helper called by PlaceActor and InternalBase constructors. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009caa20` | `DestroyInternalModeTemporary` |  | Runs InternalBase cleanup for a temporary during exception unwinding. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cab50` | `DestroyInternalModeTemporary` |  | Runs InternalBase cleanup for a temporary during exception unwinding. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cace0` | `DestroyInternalModeTemporary` |  | Runs InternalBase cleanup for a temporary during exception unwinding. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cad90` | `ReplaceModeWithPlaceActor` |  | Replaces the active freeform mode with PlaceActor. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cb230` | `ReplaceActorManipulationMode` |  | Replaces the active manipulation-mode object with a fresh actor mode. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009c9780` | `FreeformActorManipulationModeBase::ForwardNestedEvent ? ⚠rename` | FreeformActorManipulationModeBase | Shared in PlaceActor, InternalBase, ManipulateActor, Translation, Rotation and Scale vtables. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9980` | `FreeformActorManipulationModeManipulateActorInternalBase::GetActiveOrPrimaryFie…` | FreeformActorManipulationModeManipulate… | Shared implementation also present in Translation, Rotation and Scale vtables. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |

## ⚪ unknown (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009c9990` | `— ⚠reject` | FreeformActorManipulationModeRotation | Applies the selected axis delta to the Rotation mode vector while active. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9e70` | `— ⚠reject` | FreeformActorManipulationModeTranslation | Forwards Translation mode vectors to its nested operation. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9e90` | `— ⚠reject` | FreeformActorManipulationModeRotation | Forwards Rotation mode vectors to its nested operation. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ca020` | `— ⚠reject` | FreeformActorManipulationModeTranslation | Applies axis input to Translation mode state. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ca100` | `— ⚠reject` | FreeformActorManipulationModeScale | Applies axis input to Scale mode's four-float state. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cae90` | `— ⚠reject` |  | Changes the active freeform transform mode. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |

