# System: game.derby

45 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009ee280` | `VehicleDemolitionDerbyProcessor_LoadSoundEvents` | VehicleDemolitionDerbyProcessor | Per-bout setup of the derby's event bank from the HUD/camera method 009f1b80; the name list is the authoritative mapping between the handle offsets used elsewhere (009eacc0, 009eb140, 009eb300, 009ed3d0) and their gamep… | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009f01f0` | `VehicleDemolitionDerbyProcessor::RefreshHudValues` | VehicleDemolitionDerbyProcessor | Refreshes the demolition-derby HUD widgets (score, health/boost bars, race order) from the local vehicle state. | [[Native Map/Shards/S0282 009f01f0-009f3940|S0282]] |
| `009f0f30` | `VehicleDemolitionDerbyProcessor::VehicleDemolitionDerbyProcessor` | VehicleDemolitionDerbyProcessor | Constructor of the demolition-derby processor, which owns the vehicle, pickup, DerbyInfo and results lists. | [[Native Map/Shards/S0282 009f01f0-009f3940|S0282]] |
| `009f11a0` | `VehicleDemolitionDerbyProcessor::~VehicleDemolitionDerbyProcessor` | VehicleDemolitionDerbyProcessor | Destructor body of the demolition-derby processor (its scalar deleting wrapper is 008a5e40). | [[Native Map/Shards/S0282 009f01f0-009f3940|S0282]] |
| `009f1b80` | `VehicleDemolitionDerby_SetActive` |  | Turns the demolition-derby mode on or off for this client: HUD, camera, sound events, lists, and the register-player record. | [[Native Map/Shards/S0282 009f01f0-009f3940|S0282]] |
| `009f2d90` | `VehicleDemolitionDerby_UpdateStateMachine` |  | Per-frame state machine of the demolition derby, called from the client update loops. | [[Native Map/Shards/S0282 009f01f0-009f3940|S0282]] |
| `00cc1070` | `ControllerVehicle::Destroy` | ControllerVehicle | Destroys the shared controller-vehicle base used by vehicle minigame variants. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc10d0` | `ControllerVehicle::CallBaseThenVirtualA4` | ControllerVehicle | Performs a base vehicle operation followed by a class-specific virtual callback. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc14b0` | `ControllerVehicle::ControllerVehicle` | ControllerVehicle | Initializes shared controller state for vehicle minigame variants. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc16d0` | `ControllerVehicle::ScalarDeletingDtor` | ControllerVehicle | Implements scalar deleting destruction for ControllerVehicle. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |

## 🟢 likely (22)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009eaad0` | `Copy64BytesAndSetFlag` |  | Mechanical block-copy helper called from the derby processor path; object type is not established by this function itself. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009eb140` | `VehicleDemolitionDerbyProcessor_ApplyFloatTo17Handles` | VehicleDemolitionDerbyProcessor | Applies one float (an int argument cast to float) to nearly all of the processor's event handles (every pair except the first two) from the countdown/bout-start method 009f2d90; FUN_0078fcc0's own callers sit in many UI… | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009eb300` | `VehicleDemolitionDerbyProcessor_StopFourHandles` | VehicleDemolitionDerbyProcessor | Stops four long-running derby handles during the HUD teardown 009f1b80; the four offsets are exactly the 'Active' slots plus UI_LowHealthNotify in the name table 009ee280 writes (0x100=UI_LowHealthNotify, 0x130=Shield_A… | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009eb380` | `VehicleDemolitionDerbyProcessor_CheckLocalVehicleThreshold` | VehicleDemolitionDerbyProcessor | Threshold query on the local derby vehicle, called from the countdown/start method 009f2d90; 009ed640 clears the +0x18c flag and the +0x190 value on every vehicle beforehand, so this is a per-round re-arm check. The uni… | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009eb5b0` | `ProxiedVehicleDemolitionDerby_DtorWrapper` | ProxiedVehicleDemolitionDerby | Wrapper around the ProxiedVehicleDemolitionDerby destructor body; static-destruction registration is not established by this function. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009eb600` | `ProxiedVehicleDemolitionDerbyPickup_DtorWrapper` | ProxiedVehicleDemolitionDerbyPickup | Wrapper around the ProxiedVehicleDemolitionDerbyPickup destructor body; static-destruction registration is not established by this function. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009eb960` | `ProxiedVehicleDemolitionDerbyPickup_ConstructWrapper` | ProxiedVehicleDemolitionDerbyPickup | Constructs a ProxiedVehicleDemolitionDerbyPickup in freshly allocated list storage; its only caller 009eccd0 (the pickup list append) allocates the node first. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ed1e0` | `DerbyVehicleList_FindByKeyPair` |  | Participant-key lookup used by the derby dispatcher (cases 5, 0x0b and 0x0a) and by the send tick 009f22c0 to resolve a vehicle from the 64-bit key the wire carries. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ed220` | `DerbyInfoList_FindByKeyPair` |  | Name/info lookup for a participant key pair; used by the results-name resolver FUN_00c66780 and by the dispatcher. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ed260` | `DerbyPickupList_FindById` |  | Pickup lookup by id; dispatcher case 0x0d uses it with the id read from the DestroyProxiedVehicleDemolitionDerbyPickup record and then removes the node with 009ed780. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ed350` | `DerbyResultsList_FindByKeyPair` |  | Lookup in the DerbyResultsStruct list (the same 0x20-byte records 009ef180 fills: key pair at +0/+4, actor at +8); called by the dispatcher when a result row must be found for a participant key. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ed3d0` | `DerbyStartGate_AllElementsAndVehiclesReady` |  | Countdown/start gate used by 009f2d90 before the race timer starts: it holds the round until every handle is usable and every vehicle reports ready. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ed610` | `DerbyVehicles_AllHaveStateObject` |  | Structural readiness predicate consulted by the countdown/start method 009f2d90; it asserts every vehicle has its state object allocated. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ed640` | `DerbyVehicles_SetRoundFlagAndClearField` |  | Per-round re-arm on all vehicles, called from the countdown/start method 009f2d90; the pair it writes is the one 009eb380 later queries through FUN_00fe3e10. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ed680` | `DerbyVehicles_CountFlaggedEqualsD0` |  | Cross-check of the player-count invariant: +0xd0 is the processor's player-count field the dispatcher fills from subtype 3, so this asserts that many vehicles carry the +0xf00 flag. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ed6b0` | `DerbyVehicles_CountUnflaggedEqualsD4` |  | The complement of 009ed680 using the npc-count field +0xd4 filled from subtype 4 (NpcCount), called from the same countdown/start method 009f2d90. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009eef70` | `DerbyInfo_DtorWrapper` | DerbyInfo | Wrapper around the DerbyInfo destructor body; its registration as a static destructor is not established by this function. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ef2a0` | `DerbyInfo_ctor` | DerbyInfo | Allocates-ready constructor for a DerbyInfo node; the wrapper 009effd0 calls it, and its destructor counterpart 009ee630 frees exactly that +0x88 string. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ef840` | `DerbyInfo_scalarDeletingDtor` | DerbyInfo | Delete path of DerbyInfo nodes; no callers in the database because it is reached through a vtable. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009effd0` | `DerbyInfo_ctorWrapper` | DerbyInfo | Out-of-line constructor entry for DerbyInfo used by the processor-side caller 009f05f0. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009f0830` | `RecordCopyAssign_ArraysAt8cAnd9c` |  | Copies the sub-0x13 (SetDerbyConfig) record; dispatcher 009f39b0 case 0x13 calls it after the 009f0700 read. | [[Native Map/Shards/S0282 009f01f0-009f3940|S0282]] |
| `009f22c0` | `VehicleDemolitionDerby_TickVehicles` |  | Per-frame derby vehicle update: integrates vehicle state and sends pickup-hit, vehicle-hit and state-update packets for the local player. | [[Native Map/Shards/S0282 009f01f0-009f3940|S0282]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009ed380` | `ForEachDerbyVehicle_CallTwoStubs ? ⚠downgrade` |  | Per-vehicle sweep around the derby round boundary (it sits between the leader helpers and the readiness gates); the two callees are unresolved virtual-like stubs, so only the iteration is claimed. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |

## ⚪ unknown (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009eacc0` | `— ⚠reject` | VehicleDemolitionDerbyProcessor | Releases the processor-held event definition and live-instance pairs populated by 009ee280; called from processor cleanup paths 009f11a0 and 009f1b80. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009eb8e0` | `— ⚠reject` | ProxiedVehicleDemolitionDerby | Constructs a ProxiedVehicleDemolitionDerby in freshly allocated list storage; its only caller 009ecc20 allocates the list node and then calls this before linking it. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ecd70` | `— ⚠reject` | ProxiedVehicleDemolitionDerby | Delete path for a derby actor; reached from the list teardown 009ed6e0 when a node is removed and freed. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ecdd0` | `— ⚠reject` | ProxiedVehicleDemolitionDerbyPickup | Delete path for a derby pickup; reached from the pickup-list teardown 009ed780 when a node is removed and freed. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ed290` | `— ⚠reject` |  | Leader determination over the derby vehicles (the +0x124 value is the same score field the results list copies into its score column), so the client can show who is ahead. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ed2e0` | `— ⚠reject` |  | Marks exactly one vehicle as the current leader; unlike 009ed290 it keeps the first maximum when scores tie. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ed5a0` | `— ⚠reject` |  | Second readiness gate next to 009ed3d0, also used by the countdown/start method 009f2d90; the object at *(this+4)+0xc8 is the client interface whose slot 9 plus a +0x232 flag represent the local player state. | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009edf00` | `— ⚠reject` |  | Stores the newest subtype-5 record inside the object at +0x1b40; the caller 009f22c0 is the derby tick, but this body alone does not prove whether param_1 is the processor or a ProxiedVehicleDemolitionDerby actor (both … | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ee630` | `— ⚠reject` | DerbyInfo | Node destructor of the DerbyInfo list: the list allocator 009ecb00 is RTTI-bound to SoeUtil::List<DerbyInfo,-1> with 0xb0-byte nodes whose links sit at +0xa8/+0xac exactly where the unlink helper 009f0100 manipulates th… | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009ef180` | `— ⚠reject` | VehicleDemolitionDerbyProcessor | Fills the results screen data (the Lua table BaseClient.VehicleDemolitionDerbyProcessor.DerbyResults reads place, guid, name, score, damage - place comes from the sort order, score from +0xc, damage from the +0x10 diffe… | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |
| `009f20f0` | `— ⚠reject` | VehicleDemolitionDerbyProcessor | Per-frame countdown for the demolition derby: updates the Countdown HUD and sends the countdown-status record. | [[Native Map/Shards/S0282 009f01f0-009f3940|S0282]] |
| `00cc1810` | `— ⚠reject` | ControllerVehicle | Applies a transform to the controller vehicle's stored pose fields. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `009f1b80` | `VehicleDemolitionDerby_SetActive` | send | PacketTunneledClientPacket |  |  |
| `009f2d90` | `VehicleDemolitionDerby_UpdateStateMachine` | send | PacketTunneledClientPacket |  |  |
| `009f22c0` | `VehicleDemolitionDerby_TickVehicles` | send | PacketTunneledClientPacket |  |  |
| `009f20f0` | `— ⚠reject` | send | PacketTunneledClientPacket |  |  |
