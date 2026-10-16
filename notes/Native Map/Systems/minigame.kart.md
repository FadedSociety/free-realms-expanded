# System: minigame.kart

64 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (21)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00afc510` | `ControllerVehicleRace::ControllerVehicleRace` | ControllerVehicleRace | Constructs the kart-racing vehicle controller on top of ControllerVehicle and runs its race setup calls. | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00afc5e0` | `ControllerVehicleRace::~ControllerVehicleRace` | ControllerVehicleRace | Tears down the kart-racing controller and delegates base teardown to ControllerVehicle. | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00afc660` | `ControllerVehicleRace::ScalarDeletingDtor` | ControllerVehicleRace | Destroys a ControllerVehicleRace and frees its storage on the delete path. | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00ba1a40` | `VehicleRaceProcessor::UpdateBoostBarAndBackwardsHud` | VehicleRaceProcessor | Per-frame HUD update for the vehicle-race backwards-driving indicator and the boost bar. | [[Native Map/Shards/S0362 00ba0350-00ba4650|S0362]] |
| `00ba23d0` | `VehicleRaceProcessor::PushMiniMapRacerLocations` | VehicleRaceProcessor | Per-frame minimap feed: sends each racer's guid, place and position to the Lua minimap. | [[Native Map/Shards/S0362 00ba0350-00ba4650|S0362]] |
| `00ba2b90` | `VehicleRaceProcessor::VehicleRaceProcessor` | VehicleRaceProcessor | Constructs the VehicleRaceProcessor game-processor object that owns all race lists and the racer state. | [[Native Map/Shards/S0362 00ba0350-00ba4650|S0362]] |
| `00ba2f80` | `VehicleRaceProcessor::~VehicleRaceProcessor` | VehicleRaceProcessor | Teardown of the race processor: releases every list node and the race-event listeners before the base destructor runs. | [[Native Map/Shards/S0362 00ba0350-00ba4650|S0362]] |
| `00ba39f0` | `VehicleRaceProcessor::ScalarDeletingDtor` | VehicleRaceProcessor | Virtual slot 0 of VehicleRaceProcessor (read from the binary): deleting a processor runs its teardown and frees it. | [[Native Map/Shards/S0362 00ba0350-00ba4650|S0362]] |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | VehicleRaceProcessor | Processes inbound vehicle-race updates on the client. | [[Native Map/Shards/S0364 00ba74d0-00bac070|S0364]] |
| `00ce0820` | `ProxiedVehicleRaceMine::~ProxiedVehicleRaceMine` | ProxiedVehicleRaceMine | Tears down a kart race mine proxy and releases the objects it owns. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce0970` | `ProxiedVehicleRaceMine::ProxiedVehicleRaceMine` | ProxiedVehicleRaceMine | Constructs a kart race mine proxy from a node reference. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce09a0` | `ProxiedVehicleRaceMine::ScalarDeletingDtor` | ProxiedVehicleRaceMine | Scalar deleting destructor of the kart race mine proxy. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce0dd0` | `ProxiedVehicleRaceExplosion::~ProxiedVehicleRaceExplosion` | ProxiedVehicleRaceExplosion | Destroys a kart race explosion proxy, releasing the object at +0x8 and the resource at +0x40. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce0ee0` | `ProxiedVehicleRaceExplosion::ScalarDeletingDtor` | ProxiedVehicleRaceExplosion | Scalar deleting destructor of the kart race explosion proxy. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce0f00` | `ProxiedVehicleRaceExplosion::ProxiedVehicleRaceExplosion` | ProxiedVehicleRaceExplosion | Constructs a kart race explosion proxy from an owner pointer and a node reference. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce1310` | `ProxiedVehicleProximityArrow::~ProxiedVehicleProximityArrow` | ProxiedVehicleProximityArrow | Destroys a proximity-arrow proxy. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce1340` | `ProxiedVehicleProximityArrow::ScalarDeletingDtor` | ProxiedVehicleProximityArrow | Scalar deleting destructor of the proximity-arrow proxy. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce28a0` | `VehicleRaceResultsDataSource::ColumnNameByIndex` | VehicleRaceResultsDataSource | Returns the column header text for a race results row. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce3060` | `VehicleRaceResultsDataSource::~VehicleRaceResultsDataSource` | VehicleRaceResultsDataSource | Destroys a vehicle race results data source and its row array. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce3140` | `VehicleRaceResultsDataSource::VehicleRaceResultsDataSource` | VehicleRaceResultsDataSource | Constructs the race results data source used by the vehicle race processor. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce31d0` | `VehicleRaceResultsDataSource::ScalarDeletingDtor` | VehicleRaceResultsDataSource | Scalar deleting destructor of the vehicle race results data source. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |

## 🟢 likely (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00705c30` | `CompareGuidPair ⚠downgrade` |  | Compares the two GUID words used to recognize the local kart racer. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007071f0` | `SetCallbackState1 ⚠downgrade` |  | Stores callback state one for the kart countdown callback path. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00b9aee0` | `HasLiveLocalRacer` |  | Predicate used by the race results path (caller 00ba6530) to ask whether the local player's racer has a live vehicle object before reading race results. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9d210` | `StoreRacerIndexedPair` |  | Indexed store used by the 33-family dispatcher (caller 00ba74d0) in its per-racer loop that fills four pairs (index 0..3) of a racer object at +0x15f0 before writing the racer's lap fields at +0x15ec and +0x1610. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9e0c0` | `VehicleRaceProcessor::FindRacerByGuid ⚠downgrade` | VehicleRaceProcessor | Guid-to-racer lookup used by the 33-family dispatch paths (every packet carrying an owner guid selects its racer through this) and by the proxied-object code (00ce0140/00ce0340/00ce09c0). | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9e100` | `FindRacerInfoByGuid` |  | Second guid lookup in the processor, over the RacerInfo list (node size 0x28, links +0x20/+0x24); caller 00ce3330 uses it from the race-session code. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9e130` | `FindProxiedVehicleRacePickupById` |  | Id-to-object resolution used by the 33-family dispatcher before touching a proxied pickup (the pickup-node key lives at +4). | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9e160` | `FindProxiedVehicleRaceMissileById` |  | Id-to-object resolution for the missile messages of the 33 family, called from the dispatcher 00ba74d0 (e.g. the 33/25/33/26 destroy/update paths). | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9e190` | `FindProxiedVehicleRaceMineById` |  | Id-to-object resolution for the mine messages of the 33 family, called from the dispatcher 00ba74d0 (e.g. 33/30/33/31). | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9e1c0` | `FindProxiedVehicleRaceExplosionById` |  | Id-to-object resolution for the explosion messages of the 33 family (e.g. 33/27/33/28), called from the dispatcher 00ba74d0; the explosion node stores the key inside a sub-object at +8. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9e1f0` | `ForEachRacerCallLocalStubs` |  | Bookkeeping pass over racers (per-racer hook points) that this build compiled to no-ops; possibly race stat flush points. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9e860` | `SortRacerListWithComparator` |  | Orders the racer list each update: caller 00ba2700 is the VehicleRaceProcessor per-frame update, and the finish-protocol note records the racer-list order being produced by the sort predicate 00b9abe0. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9f1b0` | `RacerInfoConstructor` |  | Constructor invoked for each new racer-info node by the list add 00b9f6c0 (the RacerInfo node's guid field at +0x10 and links at +0x20/+0x24 are filled by the caller afterwards). | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00ce0340` | `VehicleMissileInFlightSetup` |  | Creates the in-flight missile effect for a kart racer, keyed by the Vehicle_Missile_InFlight name. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce31f0` | `VehicleRaceResultsDataSource::GetCell` | VehicleRaceResultsDataSource | Formats one cell of the race results table (column order as in ColumnNameByIndex: Position, Guid, Name, BestLapTime, TotalTime, Finished). | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |

## 🟡 speculative (14)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00afc640` | `ControllerVehicleRace::ReadChain_74_11c_2cc ?` | ControllerVehicleRace | Thiscall with no args, vtable slot 53 (+0xd4). Returns dword this->[+0x74]->[+0x11c]->[+0x2cc], or 0 when any link is null (tests at each step, then xor eax). Plain ret. | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00b9abc0` | `SetByte198 ?` |  | Writes the byte 1 to this+0x198 and returns. Single 8-byte store (`mov byte ptr [ecx+0x198], 1; ret`), no branches or calls. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9abd0` | `SetByte199 ?` |  | Writes the byte 1 to this+0x199 and returns. Single 8-byte store (`mov byte ptr [ecx+0x199], 1; ret`), no branches or calls. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9e490` | `AllRacersHaveLiveObjects ? ⚠rename` |  | Predicate used by the results path (caller 00ba6530) to check that every racer still has a live vehicle object. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9e500` | `RacerFlagCountEquals1C4 ? ⚠rename` |  | Results-screen predicate (caller 00ba6530): exactly the expected number of racers (+0x1c4) carry the +0xf00 flag. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9e530` | `RacerFlagCountEquals1C8 ? ⚠rename` |  | Complementary results-screen predicate (caller 00ba6530) to 00b9e500: exactly the expected number of racers (+0x1c8) have the +0xf00 flag clear. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00ce08a0` | `CopyVec4FromLookupThenDispatch ?` |  | If this+0xb8 is nonzero: calls vtbl+0x10 of the object at this+0x1c with this+0x14; if that returns an object, copies its four dwords at +0x190..+0x19c into a local vector and passes its address to vtbl+0x10 of the obje… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce0940` | `TailForwardLookup_1c_14_10 ?` |  | Calls vtbl+0x10 of the object at [this+0x1c] with the dword at [this+0x14]; if the returned pointer is nonnull it tail-jumps to that object's vtbl+0x1c, forwarding the stack argument; otherwise returns 0 (ret 4). | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce09c0` | `ProxyMatrixComposeDispatch ?` |  | When byte this+0xb0 is nonzero and the racer lookup FUN_00b9e0c0 succeeds with racer+0x10b0 nonzero, copies the 16-float block returned by FUN_00fe3b20 and composes it with the floats at this+0x30..+0x6c into a local 4x… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce0e70` | `TickFloatReleaseOnNegative ?` |  | With one float argument: if [this+8] is nonnull it calls FUN_01088290 with the float, and when the float at +0x38 of that object is negative it calls vtbl[0] of that object with 1 and nulls [this+8]; when byte [this+0x1… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce1270` | `SetWordBit1ThenVtbl3c ?` |  | Thiscall with one dword argument (ret 4): when the object at [this+0x14] has bit 1 of its 16-bit word at +0x10 different from the low bit of the argument, it sets that bit to the argument, calls FUN_00735a30 when the dw… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce1380` | `ProxyActiveUpdate ?` |  | Returns immediately unless [this+0x20] is nonzero with its +0x10b0 dword nonzero, [this+0x14] is nonzero, and bit 1 of the word at [this+0x14]+0x10 is set; otherwise it builds a 16-float block from FUN_00fe3b20 and a se… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce1b30` | `CreateObjectSetFlags01 ?` |  | Thiscall with a string argument, a value argument (stored at this+0x20) and a pointer argument (param_6, four dwords read from +0x30..+0x3c): copies the string argument with FUN_006f9e60, builds a local parameter block … | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce29b0` | `VehicleRaceResultsDataSource::GetValueByCount ?` | VehicleRaceResultsDataSource | Virtual at VehicleRaceResultsDataSource::vftable +0x30 slot [2] (no stack arguments): reads count = dword [[this+0x224]+0x194]; returns 0 when count is below 10, the dword [[this+0x224]+0x2a0] when count is exactly 10, … | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |

## ⚪ unknown (14)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00afc680` | `— ⚠reject` | ControllerVehicleRace | Thiscall, one float arg (time step), vtable slot 50 (+0xc8). Walks a linked list headed at this+0xc8 (node: +0x00 element ptr, +0x04 next). Elements with type 0 and flag at +0x0d dispatch action codes 0x13-0x16 to 009cf… | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00b9a660` | `— ⚠reject` | VehicleRaceProcessor | Teardown half of the VehicleRaceProcessor race-event listener table: 00b9f360 fills exactly these 24 pairs with Vehicle_* event ids and sound-emitter objects, so this routine unregisters every race sound emitter and dro… | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9e240` | `— ⚠reject` |  | Race-completion predicate used by the results path (caller 00ba6530): every racer must pass the per-racer check and the local player object must satisfy its own test. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9e2b0` | `— ⚠reject` |  | Ready/completion gate for the race used by the results path (caller 00ba6530): all 24 Vehicle_* event conditions must hold and every racer must pass 00c54750. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9e4c0` | `— ⚠reject` |  | Applies a per-racer state byte and zeroes a float on the live vehicle, from the race results path (caller 00ba6530). | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9ef60` | `— ⚠reject` |  | Called by the input/broadcast builder 00ba48a0; the destination offset 0x1b40 is inside a ProxiedVehicleRace node (0x1bc0 bytes), so each proxied racer keeps an embedded copy of the client-data packet (constructed by 00… | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00ba1c70` | `— ⚠reject` | VehicleRaceProcessor | Per-frame lap HUD and lap-timer update for the vehicle race. | [[Native Map/Shards/S0362 00ba0350-00ba4650|S0362]] |
| `00ba1ea0` | `— ⚠reject` | VehicleRaceProcessor | Per-frame race-position HUD update (the top-place list and the local player's place). | [[Native Map/Shards/S0362 00ba0350-00ba4650|S0362]] |
| `00ba2700` | `— ⚠reject` | VehicleRaceProcessor | Fills the race results screen: sorts the racer list and raises RaceResults:Populate once the race has finished. | [[Native Map/Shards/S0362 00ba0350-00ba4650|S0362]] |
| `00ce0b80` | `— ⚠reject` |  | Arms a kart race mine: stores its placement block and creates its Vehicle_Mine_Beep object. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce0f90` | `— ⚠reject` |  | Thiscall with nine stack arguments (ret 0x24, returns 1 in al): copies four dwords from the argument at param_6 to this+0x20..+0x2c, allocates 0x50 bytes with FUN_00d833a6 and on success builds an object with FUN_010882… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce11f0` | `— ⚠reject` | ProxiedVehicleRaceExplosion | Creates a particle definition asset for the explosion proxy. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce12c0` | `— ⚠reject` | ProxiedVehicleProximityArrow | Constructs a proximity-arrow proxy from a node reference. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce3330` | `— ⚠reject` | VehicleRaceResultsDataSource | Refreshes the race results rows from the racer list at the end of a kart race (finish protocol). | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/1 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/3 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/4 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/5 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/8 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/10 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/11 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/12 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/13 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/14 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/15 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/16 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/18 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/20 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/21 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/23 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/24 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/25 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/26 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/27 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/28 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/29 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/30 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/31 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/33 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket` | recv |  | 33/34 | Hypothesis: the original service sent these C33 subtype responses. This depends on the native dispatch and client/capture notes; it does not establish server rules. |
