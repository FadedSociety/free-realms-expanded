# System: game.garage

11 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009d2530` | `VehiclePartProcessor::Ctor` | VehiclePartProcessor | Initializes a client vehicle-part processor. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009d2eb0` | `VehiclePartProcessor::Dtor` | VehiclePartProcessor | Releases resources owned by the vehicle-part processor. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `00b86d40` | `GarageManager::GarageManager` | GarageManager | Creates the manager that drives the vehicle garage screen (see Start 00b877d0 and the packet request wrappers 00b87970/00b87a30/00b87b50). | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b86ff0` | `GarageManager::ScalarDeletingDtor` | GarageManager | Deleting destructor for the garage manager created by 00b86d40. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b877d0` | `GarageManager::Start` | GarageManager | Enter the vehicle garage screen and tell the server/scripts about it. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009d2640` | `ReleaseAndClearCachedResource` |  | Releases the cached resource held by the vehicle processor during shutdown. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009d2670` | `StopAndClearInterfaceAtC` |  | Stops and clears an owned interface during garage shutdown. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `00b86da0` | `GarageManager::Stop` | GarageManager | Close the vehicle garage screen and restore the normal HUD/input state; 00b877d0 is the matching Start path. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b87a30` | `GarageManager::SelectVehicle` | GarageManager | Apply a vehicle selection locally and tell the server, then refresh the configuration. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b87b50` | `GarageManager::RemovePartFromSlot ? ⚠downgrade` | GarageManager | Ask the server to detach the part in the given vehicle slot while the garage is open. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b87970` | `— ⚠reject` | GarageManager | Push the player's current vehicle equipment array to the server while the garage is open. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b877d0` | `GarageManager::Start` | send | PacketTunneledClientPacket |  | Hypothesis: entering the garage begins a server-side session that owns the vehicle state (parts, colors, active vehicle); the explicit initial config request (opcode 1) supports that the server held the authoritative vehicle configuration. |
| `00b87a30` | `GarageManager::SelectVehicle` | send | PacketTunneledClientPacket |  | Hypothesis: after a selection the client re-requests the configuration (opcode 1) as an acknowledgement/refresh; the server's role in confirming the selection is not preserved. |
| `00b87b50` | `GarageManager::RemovePartFromSlot ? ⚠downgrade` | send | PacketTunneledClientPacket |  | Hypothesis: the server performs the actual part removal and returns an updated config; no server code is preserved to confirm the reply path. |
| `00b87970` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: this is the client's answer to a server request for the current loadout (or a refresh after an edit); the round trip cannot be proven from the client alone. |
