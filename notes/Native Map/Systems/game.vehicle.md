# System: game.vehicle

31 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (14)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ba5eb0` | `SetVehicleRaceActive` |  | Starts or tears down the local vehicle-race controller; the cited register branch is the nonzero flag, while flag zero is teardown. | [[Native Map/Shards/S0363 00ba48a0-00ba7460|S0363]] |
| `00ba6530` | `VehicleRaceProcessor::UpdateRaceState` | VehicleRaceProcessor | Per-tick vehicle-race state and results-screen controller, including the lap-limit results trigger and score-screen transition. | [[Native Map/Shards/S0363 00ba48a0-00ba7460|S0363]] |
| `00cc1a00` | `ControllerVehicle::UpdateMovement` | ControllerVehicle | Shared vehicle movement update used by ControllerVehicle and its race and demolition-derby subclasses. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cdf8e0` | `ProxiedVehicleRace::DestructorBody` | ProxiedVehicleRace | Destructor body for the proxied vehicle-race object. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdf9b0` | `ProxiedVehicleRace::Constructor` | ProxiedVehicleRace | Constructs a proxied vehicle-race record with its embedded client-data packet object. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdfa70` | `ProxiedVehicleRace::ScalarDeletingDestructor` | ProxiedVehicleRace | MSVC scalar deleting destructor for ProxiedVehicleRace. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdfbb0` | `ProxiedVehicleRacePickup::DestructorBody` | ProxiedVehicleRacePickup | Destroys a proxied vehicle-race pickup and releases its owned objects. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdfc50` | `ProxiedVehicleRacePickup::Constructor` | ProxiedVehicleRacePickup | Constructs a proxied vehicle-race pickup list node. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdfc80` | `ProxiedVehicleRacePickup::ScalarDeletingDestructor` | ProxiedVehicleRacePickup | MSVC scalar deleting destructor for ProxiedVehicleRacePickup. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdff80` | `ProxiedVehicleRaceMissile::DestructorBody` | ProxiedVehicleRaceMissile | Destroys a proxied vehicle-race missile and releases its owned references. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00ce00e0` | `ProxiedVehicleRaceMissile::Constructor` | ProxiedVehicleRaceMissile | Constructs a proxied vehicle-race missile list node. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00ce0120` | `ProxiedVehicleRaceMissile::ScalarDeletingDestructor` | ProxiedVehicleRaceMissile | MSVC scalar deleting destructor for ProxiedVehicleRaceMissile. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cf4230` | `BindVehicleJumpEvents → BindVehicleJumpSoundEvents` |  | Binds four named vehicle jump/bounce event resources to runtime handles. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf5040` | `BindTerrainHitEvents` |  | Binds terrain-impact event resources to runtime object handles. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |

## 🟢 likely (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ba64b0` | `RequestVehicleRaceNpcReset` |  | Requests the vehicle-race NPC reset through the gateway. | [[Native Map/Shards/S0363 00ba48a0-00ba7460|S0363]] |
| `00cdfca0` | `BuildTransformSceneObject → ProxiedVehicleRacePickup::BuildEffectAndSceneObject` |  | Vehicle-race packet handling calls this routine to create or update a transformed scene object. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdff40` | `Wrapper_00cdff40` |  | Forwards the object stored at this+0x20 to the shared target. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdff50` | `Wrapper_00cdff50` |  | Forwards the object stored at this+0x20 to the shared target. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdff60` | `Wrapper_00cdff60` |  | Forwards the object stored at this+0x20 to the shared target. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdff70` | `Wrapper_00cdff70` |  | Forwards the object stored at this+0x20 to the shared target. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00ce0140` | `ComposeAndApplyObjectTransform → ApplyRacerRelativeTransformToBoundObjects` |  | Vehicle_Missile_InFlight processing calls this transform helper after resolving the owning racer. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cf3f80` | `AllFourObjectsReady` |  | Checks whether all four grouped vehicle-related objects report ready; this role is inferred from its caller. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf4450` | `AllFourObjectsPresentAndReady` |  | Checks readiness of all four grouped vehicle resources; this is inferred from the adjacent vehicle event initialization path. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf4d20` | `AllSixObjectsReady` |  | Checks whether the six objects in the adjacent terrain-event group report ready; feature assignment is based on caller context. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf53a0` | `AllElevenObjectsReady` |  | Checks readiness of a large group of vehicle event resources; assigned by its sole caller context. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf5450` | `StopTwoObjects` |  | Marks and stops two related runtime objects; feature attribution follows the caller's vehicle state context. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ce0050` | `UpdateTwoBoundObjectsFromTransform ? ⚠rename` |  | Updates the two bound vehicle-race objects from the current transform. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cf4210` | `StopObjectIfNotReady ? ⚠downgrade` |  | Issues a vehicle-group state transition when the selected object is not marked complete; based on its sole caller. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ba48a0` | `— ⚠reject` |  | Per-frame vehicle-race update and outbound input broadcaster; the native body builds client position claims and coordinates locally observed race events. | [[Native Map/Shards/S0363 00ba48a0-00ba7460|S0363]] |
| `00ba6300` | `— ⚠reject` | VehicleRaceProcessor | Advances a pending vehicle-race countdown and emits its countdown-status message when the timer expires. | [[Native Map/Shards/S0363 00ba48a0-00ba7460|S0363]] |
| `00cf5ae0` | `— ⚠reject` |  | Binds vehicle surface, skid and drafting event resources to runtime state. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00ba5eb0` | `SetVehicleRaceActive` | send | PacketTunneledClientPacket |  | Hypothesis: the original server treated the start-branch registration message as a request to establish race participation and then supplied state or participant configuration. This depends on the 33/9 client send and the separate incoming race-state/configuration dispatch; neither the original server handler nor its policy is available. |
| `00ba6530` | `VehicleRaceProcessor::UpdateRaceState` | send | PacketTunneledClientPacket |  |  |
| `00ba64b0` | `RequestVehicleRaceNpcReset` | send | PacketTunneledClientPacket |  | Hypothesis: the original server interpreted the reset request as a vehicle-race NPC reset operation. The requested NPC set, timing, authorization, and resulting state are not established by this client sender. |
| `00ba48a0` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: the original server likely received these client position and hit claims and correlated them with race ownership and server-side route or object state. This depends on the 33/7 and 33/17 handlers and the race participant/configuration messages; packet emission does not establish original validation or reward rules. |
| `00ba48a0` | `— ⚠reject` | send |  | 33/7 | Hypothesis: the original server likely received these client position and hit claims and correlated them with race ownership and server-side route or object state. This depends on the 33/7 and 33/17 handlers and the race participant/configuration messages; packet emission does not establish original validation or reward rules. |
| `00ba48a0` | `— ⚠reject` | send |  | 33/17 | Hypothesis: the original server likely received these client position and hit claims and correlated them with race ownership and server-side route or object state. This depends on the 33/7 and 33/17 handlers and the race participant/configuration messages; packet emission does not establish original validation or reward rules. |
| `00ba48a0` | `— ⚠reject` | send |  | 33/8 | Hypothesis: the original server likely received these client position and hit claims and correlated them with race ownership and server-side route or object state. This depends on the 33/7 and 33/17 handlers and the race participant/configuration messages; packet emission does not establish original validation or reward rules. |
| `00ba6300` | `— ⚠reject` | send | PacketTunneledClientPacket |  |  |
