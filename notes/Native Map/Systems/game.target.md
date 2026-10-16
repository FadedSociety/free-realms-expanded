# System: game.target

27 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008b3390` | `MatchOneAndZero` | TargetProcessorEventHandler | Shared TargetProcessorEventHandler virtual predicate for the fixed argument pair (1,0). | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008b3470` | `TargetProcessorPlayerEventHandler::TargetProcessorPlayerEventHandler` | TargetProcessorPlayerEventHandler | Constructor of the player variant of the target-processor event handler: the class that handles player targets in the client's targeting pipeline (the tile variant is 008b3590). | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008b3590` | `TargetProcessorTileEventHandler::TargetProcessorTileEventHandler` | TargetProcessorTileEventHandler | Constructor of the tile variant of the target-processor event handler; callers include the labelled PetTileTargetHandler::PetTileTargetHandler (00b54780), so pet tile targeting builds this handler. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `00acae80` | `DispatchCollisionEvent` |  | Collision/handler dispatch of the target processor: it turns a physics hit into a call on the registered handler object (the family that House*/Pet* target handlers belong to) and manages the temporary reference to the … | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00acb770` | `TargetProcessor::TargetProcessor` | TargetProcessor | Constructor of the target processor component that the client's message router reaches at this+0x1a8 and that drives the actor-movement/target search (update 00acbc80). | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00acb8c0` | `TargetProcessor::Destructor` | TargetProcessor | Cleanup body of the target processor, called by its scalar deleting destructor 00acbc60. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00acbc60` | `TargetProcessor::ScalarDeletingDtor` | TargetProcessor | Deleting destructor of TargetProcessor, slot 0 of its vtable. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |

## 🟢 likely (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008b3380` | `TargetProcessorEventHandler::DestructorCleanup_008b3380` | TargetProcessorEventHandler | Resets the object to the TargetProcessorEventHandler vtable and returns; this distinct cleanup entry is called from compiler EH unwind funclets. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008b3400` | `TargetProcessorEventHandler::DestructorCleanup_008b3400` | TargetProcessorEventHandler | Resets the object to the TargetProcessorEventHandler vtable and returns; this distinct cleanup entry is called from compiler EH unwind funclets. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008b34e0` | `TargetProcessorEventHandler::DestructorCleanup_008b34e0` | TargetProcessorEventHandler | Resets the object to the TargetProcessorEventHandler vtable and returns; this distinct cleanup entry is called from compiler EH unwind funclets. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008b3600` | `TargetProcessorEventHandler::DestructorCleanup_008b3600` | TargetProcessorEventHandler | Resets the object to the TargetProcessorEventHandler vtable and returns; this distinct cleanup entry is called from compiler EH unwind funclets. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008bb4d0` | `MoveAndSelectTargetControllerHandler::LoadDefaultPair` | MoveAndSelectTargetControllerHandler | Loads the pair of default values into fields +8 and +0x0c; their meanings are not established. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008c61e0` | `TargetHolder_ApplyLocalTarget` |  | Applies a locally constructed Target to the holder at this; accepted Target::ApplyVectorAndRadius (00c01190) and Target::ApplyGuidTarget (00c01520) call it, so the Target role rests on those callers and on the Target vt… | [[Native Map/Shards/S0225 008c52a0-008ca980|S0225]] |
| `008c6c10` | `TargetPair_Apply` |  | Sets a pending (param_2, param_3) target pair on the holder and resolves it to an object; the accepted BaseClient::Slot65_GuidPairHandler (008d3020) calls it, so the pair-target role rests on that caller. | [[Native Map/Shards/S0225 008c52a0-008ca980|S0225]] |
| `0093bcc0` | `TileEventHandler::HandleTargetEvent ⚠downgrade` | TileEventHandler | Handle a tile target event and update target coordinates before forwarding input. | [[Native Map/Shards/S0248 009392c0-0093d740|S0248]] |
| `00acae30` | `CopyThreeInputBitsToState` |  | Input-message component of TargetProcessor, reached through the client component router. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00acb170` | `NormalizeQuaternionAndProbe` |  | Orientation probe helper used by the target search 00acb920: normalise a candidate direction quaternion and hand it to the collision/selection query before a target is accepted. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00acb490` | `NotifyHandlers` | TargetProcessor | Event fan-out of the target processor to its registered handlers (the House*/Pet* target-handler family), called by the per-frame update 00acbc80 and by the input message handler 00acc160 whenever a movement/target stat… | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a312c0` | `ArrayGetOrEmplaceSlot8 ?` |  | Returns this[4] + index*8. When index is at or beyond the count at this+8, first calls 00a2f990 to emplace the slot at that index. | [[Native Map/Shards/S0293 00a2fef0-00a35000|S0293]] |

## ⚪ unknown (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008b33b0` | `— ⚠reject` | TargetProcessorEventHandler | Deleting destructor of TargetProcessorEventHandler: the class vtable at 01802274 holds this address at slot 0, and the body itself only restores the two base vtable pointers and calls operator delete when bit 0 of the f… | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008b3530` | `— ⚠reject` | TargetProcessorPlayerEventHandler | Deleting destructor of TargetProcessorPlayerEventHandler: the class vtable at 01802284 holds this address at slot 0, and the body itself only restores the two base vtable pointers and calls operator delete when bit 0 of… | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008b3650` | `— ⚠reject` | TargetProcessorTileEventHandler | Deleting destructor of TargetProcessorTileEventHandler: the class vtable at 01802298 holds this address at slot 0, and the body itself only restores the two base vtable pointers and calls operator delete when bit 0 of t… | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008c80e0` | `— ⚠reject` |  | Appends a Target object (8-byte element, vtable plus one dword) to a Target array and initializes it from the argument through FUN_0101c990; the reopened TargetArrayRealloc (008ac460) is the growth path. The only caller… | [[Native Map/Shards/S0225 008c52a0-008ca980|S0225]] |
| `008c9f90` | `— ⚠reject` |  | Appends a range of Target objects to a Target array; the reopened CopyCtor_ArrayTarget (008d40a0) and AssignSpanFromArray (008d4ce0) callers rest the copy role on neighbour evidence. | [[Native Map/Shards/S0225 008c52a0-008ca980|S0225]] |
| `00acb920` | `— ⚠reject` | TargetProcessor | TargetProcessor update helper that filters collision-query records and returns an eligible candidate; the exact gameplay meaning of the actor type values remains unknown. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00acbc80` | `— ⚠reject` | TargetProcessor | Per-frame TargetProcessor probe update: advances its query, filters a hit into a target record, and publishes state changes to registered handlers. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00acc160` | `— ⚠reject` | TargetProcessor | Input-side entry of the target processor: the client's message router 008a7c20 (RouteMessage13ToComponents) feeds it the movement input messages and it converts them into target-processor events. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |

