# System: world.targeting

15 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009c9ab0` | `FreeformActorManipulationTerrainClickHandler::DispatchTargetClick` | FreeformActorManipulationTerrainClickHa… | Forwards terrain click parameters to the active target handler. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9ae0` | `FreeformActorManipulationTerrainClickHandler::AcceptInputKind` | FreeformActorManipulationTerrainClickHa… | Filters input kinds accepted by the terrain click handler. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9b20` | `FreeformActorManipulationActorClickHandler::DispatchTargetClick` | FreeformActorManipulationActorClickHand… | Forwards actor click parameters to the active target handler. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9b50` | `FreeformActorManipulationActorClickHandler::AcceptInputKind` | FreeformActorManipulationActorClickHand… | Filters input kinds accepted by the actor click handler. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9ed0` | `FreeformActorManipulationActorClickHandler::Ctor` | FreeformActorManipulationActorClickHand… | Creates a handler that forwards actor clicks to its target processor. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009c9f80` | `FreeformActorManipulationActorClickHandler::ScalarDeletingDtor` | FreeformActorManipulationActorClickHand… | Scalar deleting destructor for the actor click handler's target-processor base. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ca240` | `FreeformActorManipulationTerrainClickHandler::Ctor` | FreeformActorManipulationTerrainClickHa… | Creates the handler that forwards terrain clicks to its target processor. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ca2f0` | `FreeformActorManipulationTerrainClickHandler::ScalarDeletingDtor` | FreeformActorManipulationTerrainClickHa… | Scalar deleting destructor for the terrain click handler's base. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c017f0` | `Target::ApplyGuidAndVector` | Target | Applies an operation between GUID-backed and vector-backed targets. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009c9f30` | `TargetProcessorEventHandler::Dtor ? ⚠rename` | TargetProcessorEventHandler | Destroys the target-processor event-handler base subobject. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ca2a0` | `TargetProcessorEventHandler::Ctor ? ⚠rename` | TargetProcessorEventHandler | Initializes TargetProcessorEventHandler before its event callbacks run. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c01190` | `— ⚠reject` | Target | Applies a vector-based target operation using a temporary Target instance. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c01320` | `— ⚠reject` | Target | Applies an operation between two coordinate vectors. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c01520` | `— ⚠reject` | Target | Applies a target operation using a GUID and optional parameters. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c019d0` | `— ⚠reject` | Target | Applies an operation between vector-backed and GUID-backed target objects. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |

