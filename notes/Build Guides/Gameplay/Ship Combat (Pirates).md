---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Ship Combat (Pirates)

Part of [[Build Guides]] › Gameplay. Related: [[Minigame Framework]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

309 accepted functions: 66 confirmed, 73 likely, 96 speculative, 74 unknown. Network-linked (`wire`): 7.

Subsystems: `minigame.shipcombat` (291), `net.pirates` (18)

### `minigame.shipcombat`

**ControllerShip** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af7280` | ControllerShip::Ctor | confirmed | Thiscall constructor taking three stack arguments forwarded to base constructor 0094f6c0 (Controller). Stores ControllerShip::vftable at +0, initialises float … |
| `00af7430` | ControllerShip::Dtor | confirmed | Fastcall destructor body (ECX = this): stores ControllerShip::vftable at +0, sets the SEH state to -1, calls base Controller destructor 0094fab0 and returns. I… |
| `00af7620` | ControllerShip::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor body: stores ControllerShip::vftable at +0, calls base destructor 0094fab0, then frees this with _free when bit 0 of the se… |
| `00af7a10` | ControllerShip::Update | confirmed | Thiscall virtual at ControllerShip vtable slot 51. Scales the time argument by the float at 0x017ebf0c. Builds a 2-vector from the input fields +0x48/+0x4c (wh… |
| `00af8120` | ControllerShip::ApplyInputActions | confirmed | Thiscall virtual at ControllerShip vtable slot 50. Reads a flag from the object returned by 00a52cc0(this+0x80, this+0x84) (byte +0x120). Walks the entry list … |
| `00af7480` | ControllerShip::VcallA4AfterBaseCall | speculative | Fastcall virtual at ControllerShip vtable slot 9. Calls 0094e210 with no arguments, then virtual call +0xa4 on this with the global constant at 0x01814c18. |
| `00af74b0` | ControllerShip::GetF0 | speculative | Getter at ControllerShip vtable slot 24: returns the float at this+0xf0 as float10. |
| `00af74c0` | ControllerShip::SetF0Gated | speculative | Virtual at ControllerShip vtable slot 23: stores the dword argument at this+0xf0 only when the byte at this+0x144 is zero. |
| `00af74e0` | ControllerShip::GetF100 | speculative | Getter at ControllerShip vtable slot 27: returns the float at this+0x100 as float10. |
| `00af74f0` | ControllerShip::SetF100AndNegTanF0fc | speculative | Virtual at ControllerShip vtable slot 26: when the byte at this+0x144 is zero, stores the float argument at this+0x100 and stores -tan(argument) (x87 fptan) at… |
| `00af7680` | ControllerShip::SetVec4FromAxisAngle | speculative | Virtual at ControllerShip vtable slot 28. When the byte at this+0x144 is zero, calls FUN_00770e70 with a local constant vector (0, 1.0, 0, 0) and the float arg… |
| `00af7730` | ControllerShip::ComputeNegXformVec4 | speculative | Virtual at ControllerShip vtable slot 33. Builds a normalised 3-vector from the cross-product terms of this+0x110..+0x11c, calls FUN_00770e70 with the value at… |

**ShipCombatCannonConfig** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a62030` | ShipCombatCannonConfigDtor | likely | Resets the embedded cannon-config list vtable, removes all its nodes, then releases the reference-counted string member at +0x2c. |

**ShipCombatPacketCreateProxiedStructure** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a620d0` | — | unknown | Destroys the packet's member state, calls the packet base destructor, then resets the object to BaseShipCombatPacket and BasePacket vtables. |

**ShipCombatProcessor** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a4fb70` | ShipCombatProcessor_VSlot1_ReleaseAndDispatch | confirmed | Virtual method at slot 1 of ShipCombatProcessor: when bit 0 of the word at +0x10 is set it clears that bit (and word 0xfffe), releases the member at +4 through… |
| `00a6bcb0` | ShipCombatProcessor::~ShipCombatProcessor | confirmed | Destructor: writes the ShipCombatProcessor vtable and calls 00a6b5f0, destroys debug lists, releases the interface at +0x440 when present and the global releas… |
| `00a6cbb0` | ShipCombatProcessor::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the processor destructor 00a6bcb0 on this; when bit 0 of the flag argument is set it frees this through _free (thunk 00d82463… |
| `00a6b0c0` | — | unknown | Advances the state machine through earlier unprocessed states before the requested state, logging each transition and setting state bits in +0x304. Entry actio… |
| `00a6b5f0` | — | unknown | Drains the fort, chest, projectile and pickup lists at +0x74, +0x84, +0x54 and +0x64; releases objects at +0x3c4 and +0x308 via their vtable delete calls; rele… |
| `00a6b8a0` | — | unknown | Constructor of the Pirates ship-combat processor: stores ActorHandler::vftable then ShipCombatProcessor::vftable, keeps the owner in param_1[1], zeroes the sta… |
| `00a6bff0` | — | unknown | Per-frame update of the Pirates processor, gated on the initialised flag at +0x10: runs the script-side update FUN_00ca7480 and the readiness reporter 00a69480… |

**ShipCombatProjectile** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01087ff0` | ShipCombatProjectile::DtorBody | confirmed | Fastcall dtor body (7 bytes), ECX = this. Stores ShipCombatProjectile::vftable at +0x0 and returns. |
| `01088050` | ShipCombatProjectile::ScalarDeletingDtor | confirmed | Thiscall with one byte flag arg, ret 4. Stores ShipCombatProjectile::vftable at +0x0 (the body has no member cleanup call), then if bit 0 of the flag is set ca… |
| `01088070` | ShipCombatProjectile::StepMotion | confirmed | Thiscall with one float arg (time step), no return value. Subtracts (+0x1c * dt) from +0x24, then adds dt times the values at +0x20, +0x24, +0x28 and +0x2c to … |
| `010881c0` | ShipCombatProjectile::CtorFromBuffer | confirmed | Thiscall with two stack args (buffer start, buffer length), ret 8. Stores ShipCombatProjectile::vftable at +0x0, stores DAT_01cb7f94 at +0x10, zeroes +0x14 and… |
| `01088000` | ShipCombatProjectile::Ctor | likely | Thiscall with three stack args (ret 0xc). Stores ShipCombatProjectile::vftable at +0x0, stores param_2 at +0x10, param_4 at +0x14 and param_3 at +0x18, zeroes … |

**ShipCombatProxiedChest** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cacfa0` | ShipCombatProxiedChest::Ctor | confirmed | Initializes proxied structure fields, stores chest parameters, sets subtype 2, installs chest vtable, and clears remaining state. |
| `00cad590` | ShipCombatProxiedChest::ScalarDeletingDtor | confirmed | Calls chest destructor and frees the object when the delete flag's low bit is set. |
| `00cad5b0` | ShipCombatProxiedChest::Update | confirmed | Compares chest state flags, creates or clears effect state on transitions, and dispatches state-specific effect helpers. |
| `00cad030` | ShipCombatProxiedChest::Dtor | likely | Installs the chest vtable, runs virtual resource cleanup through its owner, and releases remaining chest state. |
| `00a53e40` | — | unknown | MSVC scalar deleting destructor shape: under an SEH frame with the stack cookie it calls FUN_00cad030 (the ShipCombatProxiedChest destructor body, whose code r… |

**ShipCombatProxiedFort** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cadb20` | ShipCombatProxiedFort::Dtor | confirmed | Installs the fort vtable, frees an owned cannon-barrage list if present, and calls virtual cleanup on its stored base object. |
| `00cadc60` | ShipCombatProxiedFort::Ctor | confirmed | Initializes proxied structure base fields, stores fort arguments, and installs the fort vtable. |
| `00cadd20` | ShipCombatProxiedFort::ScalarDeletingDtor | confirmed | Calls fort destructor and frees the object when the delete flag's low bit is set. |
| `00a50450` | ShipCombatProxiedFort::DtorAdapter | likely | EH-framed adapter calls the ShipCombatProxiedFort destructor body at 00cadb20 with the incoming object in ECX, then returns. The Ghidra DAT_01b839d8 XOR expres… |
| `00a53de0` | — | unknown | MSVC scalar deleting destructor shape: under an SEH frame with the stack cookie it calls FUN_00cadb20 (the ShipCombatProxiedFort destructor body, whose code re… |

**ShipCombatProxiedProjectile** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cabd60` | ShipCombatProxiedProjectile::Dtor | confirmed | Resets projectile state, releases its referenced object according to a flag, removes a tracked effect when present, and runs base cleanup. |
| `00cabf50` | ShipCombatProxiedProjectile::ScalarDeletingDtor | confirmed | Calls projectile destructor and frees the object when the delete flag's low bit is set. |
| `00cac100` | ShipCombatProxiedProjectile::Update | confirmed | Updates transform from the referenced object, draws the cannonball shadow, and forwards a time or frame value to a helper. |
| `00a50d00` | ShipCombatProxiedProjectile::CtorAdapter | likely | EH-framed constructor adapter to 00cabeb0. The decoded call sequence places the incoming object in ECX and forwards the three dereferenced stack arguments; the… |
| `00cabeb0` | ShipCombatProxiedProjectile::Ctor | likely | Calls a base initializer, installs the projectile vtable, stores source and owner references, and zeroes remaining state fields. |
| `00a53d20` | — | unknown | MSVC scalar deleting destructor shape: under an SEH frame with the stack cookie it calls FUN_00cabd60 (the ShipCombatProxiedProjectile destructor body, whose c… |

**ShipCombatProxiedShip** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c99ea0` | ShipCombatProxiedShip::DispatchSlot1 | confirmed | Loads the receiver's vtable, reads slot one, and tail-jumps to that virtual method. |
| `00ca1440` | ShipCombatProxiedShip::Dtor | confirmed | Restores the ShipCombatProxiedShip vtables, releases held references and owned arrays, clears the hold UI records, releases strings and shared state, then unre… |
| `00ca1d70` | ShipCombatProxiedShip::Ctor | confirmed | Initializes the primary and asset-requester subobject vtables, binds the ship's owner context, zeros runtime fields and initializes the embedded string, hold a… |
| `00ca2100` | ShipCombatProxiedShip::DeletingDtor | confirmed | Runs 00ca1440, frees the object when the deleting flag's low bit is set, and otherwise returns the object pointer. |
| `00ca2120` | ShipCombatProxiedShip::InitializeState | confirmed | Copies the source state, copies 16 dwords into the embedded block, registers multiple references, initializes a 0xb0-byte object, and attaches matching linked … |
| `00c9dbc0` | CreateCollisionDataAsset | likely | Virtual at ShipCombatProxiedShip::vftable +0x190 slot 3. Returns 0 when argument 2 is non-zero. Otherwise allocates 0x2c bytes with operator new (00d833a6) and… |
| `00ca2470` | ShipCombatProxiedShip::Update | likely | Converts the update delta, advances timers and state helpers, updates transform and orientation state, selects movement animations from velocity thresholds, an… |
| `00ca4510` | ShipCombatProxiedShip::ScalarDeletingDtor | likely | Calls the ShipCombatProxiedShip destructor body at 00ca1440, frees the allocation when requested, and otherwise returns the object pointer. |
| `00c9a170` | ShipCombatProxiedShip::GetField90 | speculative | Calls a helper to retrieve a global object, then returns its 32-bit field at offset +0x90. |
| `00c9be60` | ProxySlotReleaseAndUnlink | speculative | For an index argument below 0xb, takes the value at this+0x270+index*4 and calls FUN_007e7f00 on it with flag 1 (twice when the table at 0x16c+8 is set). Then,… |
| `00c9f170` | ConditionalSetRefIfNameMatches | speculative | Virtual slot 1 of the secondary ShipCombatProxiedShip vtable. When argument 3 is zero, compares a name string with the one at this-0x114 through 0071c050; rele… |
| `00ca17c0` | — | unknown | Calls the full object destructor at 00ca2100 and returns its result. |

**ShipCombatProxiedStructure** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cacba0` | ShipCombatProxiedStructure::CtorBase | speculative | Stores the ShipCombatProxiedStructure vtable pointer in the first object field. |
| `00cacc70` | — | unknown | Stores the structure vtable and frees the object when the scalar deleting flag's low bit is set. |

**ShipCombatResultsDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00caf230` | ShipCombatResultsDataSource::GetColumnName | confirmed | Returns the name for each column index, including Position, Guid, Name, Color, Gold, Kills, Score, and trophy fields. |
| `00caf380` | ShipCombatResultsDataSource::GetCount | confirmed | Returns the 32-bit value stored at object offset 0x230. |
| `00cafa50` | ShipCombatResultsDataSource::Dtor | confirmed | Installs results datasource vtables, destroys embedded row array, frees its storage, and calls inherited datasource cleanup. |
| `00cafb30` | ShipCombatResultsDataSource::Ctor | confirmed | Registers named results datasource, installs its vtables, stores passed owner, and initializes specialized results array. |
| `00cafbc0` | ShipCombatResultsDataSource::ScalarDeletingDtor | confirmed | Calls results datasource destructor and frees object when scalar deleting flag's low bit is set. |
| `00cafbe0` | ShipCombatResultsDataSource::GetCellValue | confirmed | Checks row index, selects a field by column ID, and writes the requested result value into the supplied output structure. |

**ShipCombatShip** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01087d00` | ShipCombatShip::ScalarDeletingDtor | confirmed | Thiscall with one byte flag arg, ret 4. Calls 01087bb0 (ShipCombatShip dtor body) on this, then if bit 0 of the flag is set calls the free thunk 00d82463 on th… |
| `01087bb0` | — | unknown | Fastcall dtor body (SEH-framed), ECX = this. Stores ShipCombatShip::vftable at +0x0, if the pointer at +0x14 is non-null calls its vtable slot 0 with flag 1 an… |
| `01087c60` | — | unknown | Thiscall with three stack args plus a byte flag (ret 0xc). Stores ShipCombatShip::vftable at +0x0, stores param_2 at +0x8 and param_3 at +0xc, sets +0x10 to 0x… |

**ShipCombatShipProcessor** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ca4ca0` | ShipCombatShipProcessor::Ctor | confirmed | Installs processor and embedded list vtables, zeros state and pointer fields, stores one constructor argument, then binds six pirate battle audio events. |
| `00ca4d40` | ShipCombatShipProcessor::Dtor | confirmed | Restores the processor vtable, releases six shared references, restores the embedded list vtable, then drains and destroys the proxied ship nodes. |
| `00a4f780` | ShipCombatShipProcessor::ScalarDeletingDtor | likely | MSVC scalar deleting destructor: calls the class destructor body at 00ca4d40 (whose own code references ShipCombatShipProcessor::vftable and SoeUtil::List<Ship… |
| `00ca3270` | — | unknown | When both pointer arguments and the nested +0x14 object exist, calls 00fefa30 with three values and clears receiver field +0x2d0. |

**ShipCombatZonePickupLocationConfig** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01066d60` | ShipCombatZonePickupLocationConfig::Ctor | likely | Constructor of the pickup-location element: stores the default float block at +0..+0x24, installs the PickupInstance array vtable at +0x28, and zeroes the arra… |

**ShipCombatZonePickupLocationConfig::PickupInstance** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01066550` | PickupInstanceCtor | likely | Constructs a 0x18-byte PickupInstance slot: stores DAT_017eb904 in its first two dwords, installs the IString vtable at +8 with the empty-string buffer at +0xc… |

**SoeUtil::Array<ShipCombatProxiedShip::ShipHoldUIData,0,1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c9c3f0` | SoeUtilArrayShipHoldUIData_Dtor | confirmed | Stores the Array<ShipHoldUIData> vftable at this, zeroes the count at +8, frees the element buffer at +4 through FUN_00401c00 with alignment 4, and clears +4. … |
| `00c9c770` | SoeUtilArrayShipHoldUIData_ScalarDeletingDtor | confirmed | Virtual slot 0 of the Array<ShipHoldUIData> vftable. Inlines the destructor body of 00c9c3f0 (vftable store, count zeroed, buffer freed with FUN_00401c00 align… |

**SoeUtil::Array<ShipCombatResultsDataSource::dsShipCombatResults,0,1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00caf810` | ResultsArray::ScalarDeletingDtor | confirmed | Destroys array elements, releases storage, and frees the array object when delete flag's low bit is set. |

**SoeUtil::IStringFixed<char,2>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c9c5f0` | SoeUtilIStringFixed2_Ctor | confirmed | Stores the IString<char> vftable, points the buffer at the static empty string (0x1bef5f4), zeroes the size (+8) and capacity (+0xc), then stores the IStringFi… |
| `00c9c650` | SoeUtilIStringFixed2_GetBufferForSize | confirmed | Virtual slot 1. When the requested size (argument 2) is below 7, sets the out flag to 0, the out capacity to 6 and returns this+0x10 (the inline buffer). Other… |
| `00c9c680` | SoeUtilIStringFixed2_FreeHeapBuffer | confirmed | Virtual slot 2. Frees argument 2 with FUN_00401c00 (alignment 0) unless it equals this+0x10, the inline buffer, in which case it does nothing. |
| `00c9dc90` | SoeUtilIStringFixed2_DtorBody | confirmed | Stores the IStringFixed<char,2> vftable, and when the capacity at +0xc is above zero releases the heap buffer: decrements the refcount stored 4 bytes before th… |
| `00c9dd80` | SoeUtilIStringFixed2_ScalarDeletingDtor | confirmed | Virtual slot 0 of the IStringFixed<char,2> vftable. Calls the destructor body 00c9dc90, then when bit 0 of argument 2 is set calls _free on this, and returns t… |

**SoeUtil::StringFixed<2>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c9e260` | SoeUtilStringFixed2_Ctor | confirmed | Calls the IStringFixed<char,2> constructor 00c9c5f0 on this, then stores the StringFixed<2> vftable at this and returns this. |
| `00c9e2c0` | SoeUtilStringFixed2_ScalarDeletingDtor | confirmed | Virtual slot 0 of the StringFixed<2> vftable. Calls the IStringFixed<char,2> destructor body 00c9dc90, then when bit 0 of argument 2 is set calls _free on this… |

**(no class)** (210)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c9aa40` | PirateColorFromName | confirmed | Compares a string against eight pir_* color names and returns the associated ARGB color, defaulting to zero for unrecognized names. |
| `00c9f280` | ShipDebugHudDraw | confirmed | When the flag at owner+0x264 is set, formats ship telemetry strings: Speed (vehicle speed against max), Head (heading from atan2 of the chassis record times DA… |
| `00ca0460` | ShipCannonHandlesAndReticleBind | confirmed | Resolves the keyed object handles at this+0x1c4, +0x1c8, +0x1cc, +0x1d0 and +0x1d4 through 00c9eaf0 and 00c9ebe0 (copying each result's +0x268 value). When the… |
| `00cabf70` | DrawCannonballShadow | confirmed | Builds a shadow draw payload from projectile coordinates and draws the cannonball shadow texture when render resources exist. |
| `00caf4d0` | InitResultsEntryString | confirmed | Installs SoeUtil string vtable, points data at the shared empty string, and clears length and capacity fields. |
| `00caf7e0` | DestroyShipCombatResultsArray | confirmed | Installs specialized results array vtable, destroys its elements, frees storage, and clears storage pointer. |
| `01065ed0` | JetStreamConfigLoad | confirmed | Reads one jet-stream entry from named float keys: start X/Z read from keys StartEdgeX/StartEdgeZ; computes the end point as start + JetStreamLength*(sin, cos)(… |
| `01066cf0` | PickupSpawnChanceLoad | confirmed | Reads the Pickup key (string via 0108a7f0 into 006f9e60) and the SpawnChance and RespawnChance floats into the first two floats of the element it is given (ECX… |
| `01066fd0` | PickupLocationConfigLoad | confirmed | Reads one PickupLocation entry: SpawnX/SpawnY/SpawnZ, Scale, Radius, MinSpawnTime, MaxSpawnTime, MinRespawnTime and MaxRespawnTime into float fields; then load… |
| `01067400` | CannonedEntryLoad | confirmed | Reads one fort entry from keys StartX/StartY/StartZ, Angle, Health, RespawnTime, DockIndex (stored as index minus 1), ModelId, DeadModelId, ExplodeEffectId, Co… |
| `01067ac0` | ShipCombatZoneConfigCtor | confirmed | Default-initialises the ShipCombat zone configuration object: an IString at +0, numeric defaults (including 0x682 and 0xc-based values), the dock list vtable a… |
| `01067f20` | ShipCombatZoneConfigDtor | confirmed | Tears down the zone configuration members in reverse order: rank-modifier, tag-location, fort, jet-stream, whirlpool, and pickup arrays (each with its vtable r… |
| `010680f0` | ShipZoneConfigLoad | confirmed | Loads the ShipCombat zone configuration: reads the scalar keys (DefaultShipConfig, MaxPlayers, MaxScore, EnableAI, timers, parking/repulsor/shove-out tuning, e… |
| `01069df0` | ProjectileSourceConfigLoad | confirmed | Reads one cannon/projectile node: ReloadTime, RapidReloadTime, ProjectileSpeed, LobAngle, AutoFire keys (LobRange, Radius, MinAngle, MaxAngle, LeadTime, Error)… |
| `0106a560` | ShipPhysicsConfigLoad | confirmed | Reads the ship physics and handling block: SpeedFactor, CenterOfMassZ, Length, Width, Height, Spinniness, MaxSpin, speed limits, turbo and jet-stream speeds, d… |
| `0106b940` | LoadShipCombatShipProperties | confirmed | Reads a ship-combat property table into this object: HullActor, ForwardMastActor, RearMastActor, MidMastActor, MidMastTwoActor and CollisionDataAsset/PhysicsCo… |
| `00a4f7f0` | ForwardFieldsToManager_330_33c | likely | Reads the manager interface at *(*(this+4)+0x8a0) and forwards this object's fields: +0x330 to interface virtual slot 0x34, +0x334 to slot 0x50 and +0x338 to s… |
| `00a4fa10` | SelectField_14_Else_8 | likely | Returns 0 unless the object at +0x14 exists and its field +8 is non-zero, in which case that field is returned; when the +0x14 object is absent/empty it falls … |
| `00a50840` | HashBucket_RemoveByKey360 | likely | Unlinks the specified node from the bucket chain selected by its key at node+0x360. The bucket table pointer is at map+0x14; the index is (bucket-count at map+… |
| `00a51c20` | ListStepFromHead184 | likely | Starts at *(this+0x184). While the requested step count is positive, follows the +0x200 link and decrements the count; returns 0 if a followed link is null. A … |
| `00a52cf0` | CycleIndex_38c | likely | Advances a selection index at +0x38c by the argument, wraps it to (count-1) when it goes negative and to 0 when it reaches the count reported by *(*(this+8)+0x… |
| `00a55160` | FindNodeByField9C_LinkE0 | likely | Walks nodes starting at object offset +0x74, compares each node's +0x9c field with the requested value, and returns the matching node or zero. |
| `00a55190` | FindLinkedNodeByField20_Link90 | likely | Walks nodes from object offset +0x84 using the +0x90 next link, compares each node's +0x20 field, and returns the matching node or zero. |
| `00a55ce0` | ShipCombatProxiedFortList_Append | likely | Allocates a node, initializes it through FUN_00a542c0, links it at the tail through +0xe0/+0xe4 links, and increments the owning list count. |
| `00a55d90` | ShipCombatProxiedChestList_Insert | likely | Allocates a node, initializes it through FUN_00a54330, links it after the tail using +0x90/+0x94 fields, and increments the list count. |
| `00a5da90` | ClearArrays_3cc_3dc | likely | Gets the manager at 00769710 and its pointer at +0x16c. Only when that pointer is non-null does it walk the counted arrays at +0x3cc and +0x3dc, releasing each… |
| `00a5e6a0` | AppendNodeLink70 | likely | Calls the list virtual slot +8 with no stack arguments; when it returns nonzero, calls 00a5ce50 with the three supplied values. It links the resulting node aft… |
| `00a5ebc0` | ReadCountedNodeList | likely | Removes existing destination nodes, reads a signed count, and for each positive item while the reader error byte is clear calls 00a59060 to allocate and link a… |
| `00a5ed80` | ReadUpdateGameState | likely | Constructs a temporary UpdateGameState packet, reads its two-short header and one value, rejects reader failure or unsigned (value-1) greater than 0x0d, stores… |
| `00a5ee90` | ReadUpdateShipCannons | likely | Constructs a temporary UpdateShipCannons packet, parses its header and payload, checks two global identifiers, resolves a player/ship owner, and copies eight d… |
| `00a5f030` | ReadSetPlayerCount | likely | Constructs a temporary SetPlayerCount packet, reads its header and payload, then stores the two decoded dwords at param_1+0x1c and +0x20 when the bounded read … |
| `00a5f120` | ReadDestroyProxiedShip | likely | Constructs and parses a DestroyProxiedShip packet; after a valid header/body and changed global identifiers, resolves a ship and invokes the remove/update help… |
| `00a5f260` | ProcessMatchedRecord | likely | Reads a bounded record with two identifiers, looks up the pair, applies kind/state-dependent checks, sets the matched entry marker byte when its identifier equ… |
| `00a5f470` | UpdateMatchedEntryTransform | likely | Parses an identifier and scalar, finds a linked entry with that identifier, checks the associated object's state, computes a transformed 4x4 matrix, then calls… |
| `00a5f860` | CreateEntryIfIdAbsent | likely | Parses a bounded record; continues only if the reader succeeds, the id differs from DAT_01be6e80, the decoded float is positive, and another decoded field is n… |
| `00a5fd20` | ReadShipCombatCountedValues | likely | Reads a bounded payload and its count, rejects reader failure or nonpositive count, clears the target field at +0x410, passes the decoded values to 00a56790, a… |
| `00a5ff90` | ReadUpdateProxiedStructure | likely | Constructs and parses an UpdateProxiedStructure packet. After a complete successful read, it searches only when decoded id differs from DAT_01be6e18; on the fi… |
| `00a602a0` | BuildTransformPairArrays | likely | Clears prior arrays. If source fields +0x13c and +0x140 are nonzero, it ensures both arrays cover the count at +0x238, resolves both transform objects, and onl… |
| `00a604c0` | BuildShipFlagMetadata | likely | Initializes the string field to ss_pir_home_flag.adr, rebuilds a list of ship entries, obtains each entry's string and numeric metadata through virtual methods… |
| `00a60940` | ClearPairArray_3ec | likely | Gets the manager at 00769710 and its pointer at +0x16c. Only when non-null and count +0x3f0 is positive does it walk 8-byte entries at +0x3ec, releasing each n… |
| `00a615b0` | ReadCountThenPayload | likely | Checks the input span, reads one 32-bit count (zero on an empty span), then calls the body reader to populate the remaining supplied fields and returns whether… |
| `00a618a0` | ReadCountedRecordNodes | likely | Removes the existing nodes, reads a signed count, and for each positive item while the reader error byte is clear reads one dword, calls 00a5d4a0 to create a p… |
| `00a61940` | ReadCountedRecordNodes | likely | Removes the existing nodes, reads a signed count, and for each positive item while the reader error byte is clear reads one dword, calls 00a5d5c0 to create a p… |
| `00a61a00` | ReadCountedNodeList | likely | Removes nodes from the destination list, reads its signed count, then while the reader error byte is clear constructs a node with 00a5d830 and passes it with t… |
| `00a61a70` | ReadProxiedStructureFields | likely | Reads seven consecutive dwords into destination offsets +0x0c through +0x24 (marking the reader error flag and zeroing a field on short reads), consumes a sign… |
| `00a61c30` | ReadCountedNodeList | likely | Clears a destination list, reads its count, then calls 00a5bca0 and 00a5b660 for each item while the stream error flag remains clear. |
| `00a61d40` | ReadCountedFloatPayloadRecords | likely | Removes existing nodes and reads a signed count. While the count remains and the reader error byte is clear, it allocates and links a node through 00a59220, th… |
| `00a61dd0` | ShipCombatConfigMembersDtor | likely | Releases several SoeUtil string members, destroys an array of ShipCombatShipDamageStateConfig, clears a ShipCombatCannonConfig list, and releases the final str… |
| `00a62140` | HandleCameraConfig | likely | Constructs a temporary ShipCombatPacketCameraConfig, parses the bounded payload, stores its decoded dword at param_1+0x280, checks the current object and fallb… |
| `00a68950` | ShipCombatProcessor_HandleCreateProxiedStructure | likely | Builds a temporary CreateProxiedStructure packet and parses the buffer. On complete parse with a non-null identifier, first scans the fort list at +0x74 for a … |
| `00af7f70` | TraceCameraState | likely | Thiscall debug routine: builds trace text 'Last camera position: (x, y, z) Pitch: p Distance: d' from the object's values (virtual +0x6c and +0x60 results and … |
| `00c51850` | AllSevenResourceListsReady | likely | Walks seven linked lists at this+0x14, 0xc8, 0x17c, 0x230, 0x2e4, 0x398, and 0x44c; returns 0 if any entry's vtable slot 3 is false, otherwise 1. |
| `00c9f0c0` | SetPirateColorFromNameAndNotify | likely | Assigns a string through 006f9e60, stores the ARGB value returned by 00c9aa40 (PirateColorFromName) at this+0x4cc, and when argument 2's length at +8 is positi… |
| `00ca0f50` | UpdateHoldUiReferences | likely | Divides the hold count from +0xb8 by the configured slot size, then releases two stored object references per active entry when the global state is available. |
| `00ca4a00` | AddProxiedShipIfMissing | likely | Searches for a node matching two identifiers, constructs and links a proxied ship if absent, initializes its flags and invokes setup; removes it and returns nu… |
| `00ca4ac0` | RemoveProxiedShip | likely | Clears the current-object field when it points at the requested node, then unlinks and destroys that node through 00ca47f0. |
| `00ca4ae0` | ClearProxiedShipList | likely | Repeatedly removes the current first node at +4 through 00ca47f0 until the list is empty. |
| `00cac340` | InitializeProxiedProjectile | likely | Resolves an actor reference, optionally creates a projectile effect, stores supplied time, loads the cannonball shadow texture, and updates the transform. |
| `01066600` | ProxiedStructureConfigCtor | likely | Constructs a proxied-structure configuration object of at least 0x16c bytes: writes IString/list defaults and numeric defaults at +0..+0x168, builds sub-object… |
| `01066750` | EntryPointConfigLoad | likely | Reads spawn/bank/chest/flag/enter entry-point keys into a float block: ShipSpawnX/Y/Z and ShipSpawnHeading, BankX/Y/Z and BankRadius, ChestX/Y/Z, FlagX/Y/Z, Fl… |
| `01066df0` | ArrayEmplaceBackStride0x18 | likely | Appends one 0x18-byte slot to the array at ECX (count+1, growing through 00a5bab0 when full) and constructs it through 01066550; returns the element, or 0 when… |
| `01067370` | FortEntryCtor | likely | Default-initialises a 0x4c fort entry: zeroes the leading numeric fields, installs an IString at +0x28 pointing at the empty string, and installs the List<Ship… |
| `01067800` | PickupLocationEmptyStaticGet | likely | Returns the address of a process-wide default pickup-location object (DAT_01cb73c0); on first use it constructs it through 01066d60 and registers an atexit tea… |
| `0106a3a0` | GearShiftConfigLoad | likely | Reads one gear entry: ShiftUpSpeed, ShiftDownSpeed, BaseAcceleration, BaseDeceleration, TurboAcceleration, BrakeDeceleration, SidewaysDeceleration, then the an… |
| `0106ae50` | ShipPhysicsConfigCtor | likely | Default-initialises the ship physics config block: float and integer defaults in its leading dwords, a StringFixed64 member (00402a10), the gear list construct… |
| `0106b520` | Elem0x28Ctor | likely | Constructs a 0x28-byte element: zeroes the first dword, runs the StringFixed10 constructor 00c68f90 on the element (sub-object location not resolved), and zero… |
| `0106b680` | ShipCombatShipConfigCtor | likely | Default-initialises the ship configuration block: the 'Default' name strings and <EMPTY> strings in IString members, numeric defaults (1000, 0x14, 100, 50, 0x4… |
| `01087670` | TickDownPositiveFloatTimers_164to18c | likely | Thiscall with one float arg (the delta). For each of the float fields at this+0x164, +0x168, +0x16c, +0x170, +0x174, +0x178, +0x17c, +0x180, and then three flo… |
| `01087790` | ReplaceOwnedObjectAt14_0x310 | likely | Thiscall with three stack args (ret 0xc). Deletes the object at this+0x14 through its vtable slot 0 with flag 1 if present, reads this's vtable slot 3 (+0xc) v… |
| `010879c0` | SetBit8AtFlags1c | likely | Thiscall with one byte arg (0 or non-zero). If bit 3 (0x8) of this+0x1c differs from the argument, sets or clears that bit, and when the owned pointer at this+… |
| `00a529b0` | AllDotsSameSign | speculative | Five-point sign test: computes the four dot products (P1-P2).(P3-P2), (P1-P3).(P4-P3), (P1-P4).(P5-P4) and (P1-P5).(P2-P5) using the shared two-float subtracti… |
| `00a52cc0` | CallMember8WithPair | speculative | When the member at +8 of the object is non-null, tail-forwards the two value arguments to it through FUN_00ca46a0 (bytes 83 79 08 00 74 1d show the +8 null che… |
| `00a5fe30` | ReadDestroyProxiedStructure | speculative | Constructs and parses a DestroyProxiedStructure packet, skips the sentinel id, searches the list at param_1+0x74 by node id at +0x20 using link +0xe0, and call… |
| `00af75a0` | SetConfigBlock140AndApply | speculative | Thiscall setter: stores param_2 at this+0x140; when it is non-zero, calls 00af7520 and clears the dword at this+0x150. |
| `00af75e0` | SetByte14D | speculative | Thiscall setter: stores the byte argument at this+0x14d. |
| `00af7850` | ComputeOffsetVec4 | speculative | Thiscall helper: returns 0 when this+0x150 is zero and 00a52cc0(this+0x80, this+0x84) is zero, and returns 0 when 00c9ac30(0) is zero. Otherwise writes four fl… |
| `00c9bee0` | ChassisOriginOffsetAlongRow0 | speculative | When this+0x14 is non-null and the record from FUN_00fedc30 has +0x34 inside [DAT_01811154, DAT_017ec67c], copies its 16 floats, offsets the origin (local_20..… |
| `00c9c070` | CompositeEffectKeyOrPassthrough | speculative | When the flag at this+0x19c->+0x134->+0x308 is set and argument 4 is non-zero, looks up the key with FUN_00c53d90; if found, calls FUN_00738c30 on three global… |
| `00c9c1a0` | AttachFourKeyedActorsThenVisit | speculative | For the four 32-bit keys at this+0x1c8..this+0x1d4, resolves each non-zero key through the vtable+0x10 lookup on this+0x1a4 and calls FUN_0077c0e0 (AttachChild… |
| `00c9c7b0` | StrideArray0x30AppendCopy | speculative | Appends argument 3 records of 0x30 bytes to the array at this (count +8, capacity +0xc, buffer +4): grows the buffer through FUN_00a51090 when count+n exceeds … |
| `00c9c970` | ChassisEffectHandlesRefresh | speculative | Calls FUN_007e7f00 on the global at 0x16c when set (twice when its +8 is set). When this+0x1c4 resolves through the vtable+0x10 lookup on this+0x1a4 and this+0… |
| `00c9cb30` | CompositeEffectFromResolvedHandle | speculative | When this+0x1c4 resolves through the vtable+0x10 lookup on this+0x1a4 to an object, calls FUN_00c9bfe0 with the object's +0x268 value, the key read from the wo… |
| `00c9ce80` | MountedPairTransformUpdate | speculative | Returns when this+0x14 is null. Copies the chassis record from FUN_00fedc30 and builds two offset transforms (origin shifted by +/-this+0xf4 and this+0xf8 alon… |
| `00c9d4b0` | CountdownEffectRefire | speculative | Decrements the float timer at this+0x1b4 by argument 2 while it is between 0 and DAT_017eb904. When the timer is negative and this+0x1c4 resolves through the v… |
| `00c9dc40` | StrideArray0x30AssignFrom | speculative | Sets the count at this+8 to 0, then appends the records of the source array (argument 2: buffer at +4, count at +8) through FUN_00c9c7b0; with an empty source … |
| `00c9de10` | StrideArray0x1cIndexGrow | speculative | Returns the address of element argument 2 in the array at this (buffer +4, count +8, stride 0x1c). When the index is at or beyond the count, reserves capacity … |
| `00c9dee0` | ProxyStateFlagsDispatch | speculative | Compares the high 16 bits of the state word at this+0x1c with the cached word at this+0x1fc. On a change it handles bit 16 and bit 18 (calls 00c9be60 with 0 or… |
| `00c9e070` | DestroyStackIStringFixed2 | speculative | Calls the IStringFixed<char,2> destructor body 00c9dc90 on a stack temporary (EH cleanup) and returns. |
| `00c9e320` | StrideArray0x1cAppendZeroed | speculative | Appends one element of stride 0x1c to the array at this: grows the buffer through 00c9c830 when the count plus one exceeds capacity, takes the element at buffe… |
| `00c9e3f0` | CopyRecord0x30WithIString | speculative | Copy-constructs a 0x30-byte record at this from the record at argument 2: copies 12 dwords, then sets the IString<char> vftable at +0x30 with an empty-string b… |
| `00c9e4a0` | StrideArray0x1cEmplaceZero | speculative | Calls 00c9de10 with argument 1 (the index-grow accessor for the 0x1c-stride array), and when the result is non-null zeroes its first four dwords. Returns the e… |
| `00c9e510` | ProxyIndexedSlotRefresh | speculative | Stores argument 2 as the index at this+0x1e4, runs the 0x28-stride table at this+0x148 through the assign helper 006f9e60 and 00c9c480, releases the global tab… |
| `00c9ecf0` | FormatFractionStringAndPost | speculative | Copies the three dwords at argument 2 +0x2f4, +0x2f8, +0x2fc and argument 3 into a local record, calls 007776b0 and 00779ea0 (SetFloatClampedMin_2e8 likely lab… |
| `00c9ee90` | ProxyStateEventDispatch | speculative | Calls 01087ab0 to get an index, and when it differs from this+0x1e4 calls 00c9e510. If this+0x20 is at most this+0xac, for slots 1 to 4 (keys at this+0x1c4+4*k… |
| `00ca0410` | CopyRecord0x30WithIStringAt | speculative | Thiscall: copy-constructs the record at ECX from argument 2 by calling 00c9e3f0 inside an EH frame, and returns the record pointer (ECX). |
| `00ca06f0` | StrideArray0x1cGetOrAppend | speculative | Returns the element address for the index in argument 2 of the 0x1c-stride array at this (buffer +4, count +8). When the index is at or beyond the count it fir… |
| `00ca10b0` | InitializeHoldUiSlots | speculative | Computes the number of hold slots, initializes each 0x1c-byte record with configured values, zeroes two fields, and calculates a slot position before setting +… |
| `00ca13a0` | CopyOrientationState | speculative | Copies the object at +8, two counted buffers at +4 and +0x14, clears destination +0x18, then walks a linked list at +0x24 using +0x30 links. |
| `00ca17d0` | CopyProxiedShipState | speculative | Copies 44 dwords from the source, then delegates copying of the subobjects at +0xb0 and +0x104 to helpers. |
| `00ca1970` | UpdateProxySceneObjects | speculative | Obtains an object through a virtual call, updates its position and orientation, configures two scene objects through virtual methods, and creates or updates a … |
| `00ca1d50` | ApplyOrientationCopyAndSetFlag | speculative | Copies orientation state from the second argument through 00ca13a0 and sets byte +0x590 on the receiver to one. |
| `00ca2340` | ApplyProxyTransform | speculative | Copies a 16-dword transform, computes a yaw angle, updates several referenced objects and forwards the transform into another object's virtual methods. |
| `00ca3010` | SegmentCrossingPoint2D | speculative | Normalizes a 2D segment direction, tests signed side values for a crossing, computes the intersection distance, writes the intersection coordinates, and return… |
| `00ca3210` | CopyFourDwordVector | speculative | Copies four consecutive dwords from the input into receiver offsets +0xe0 through +0xec. |
| `00ca32b0` | ReleaseSixReferences | speculative | For six pointer/reference pairs at offsets 0x20 through 0x4c, calls the associated object's virtual release when present, decrements its reference count, and c… |
| `00ca33c0` | AreSixReferencesReady | speculative | Checks six referenced objects in order using virtual slot +0xc and returns one only if each object exists and reports ready. |
| `00ca3450` | UpdateSixReferencedObjects | speculative | Calls the helper at 0078fcc0 once with the delta for each non-null reference at offsets 0x24, 0x2c, 0x34, 0x3c, 0x44, and 0x4c. |
| `00ca3750` | ResizeEightDwordArray | speculative | Requests new storage through a virtual allocator, copies existing 8-dword elements into it, releases the previous buffer, and stores the new pointer and size m… |
| `00ca4070` | ReadCurrentWorldPosition | speculative | Reads two words at offsets +0x20 and +0x28 from the object returned by 00fedc30 and writes them to the output pair. |
| `00ca46a0` | FindLinkedEntryByTwoIds | speculative | Starts at the node pointer at +0x10, follows +0x5d0 links, and returns the first node whose +8 and +0xc fields match both arguments. |
| `00ca4700` | FindLinkedEntryByField10 | speculative | For nonnegative input, walks the node chain from +0x10 via +0x5d0 and returns the first node whose +0x10 field matches, else null. |
| `00ca4730` | CheckNearbyWorldPoint | speculative | Copies two input words into a temporary structure and calls 00ca41a0 with the supplied point; the callee performs the distance comparison. |
| `00ca4770` | AllLinkedEntriesReady | speculative | Traverses the nodes beginning at +0x10, asks each one whether it is ready through 00c9af30, and returns false on the first failure. |
| `00ca47c0` | SetByteAndVisitList | speculative | Stores the low byte of its second argument at receiver offset +0x1c, then invokes a fixed callback over nodes in the list at +0xc. |
| `00ca4960` | InitializeOrientationElement | speculative | Ensures an array element at the requested index, initializes its four vector words, clears three fields, sets one field to -1, and returns its address. |
| `00ca4dc0` | BuildShipOrientationCandidates | speculative | Builds an array of direction and distance records from active linked entities and another linked entity list, transforming positions and orientations into the … |
| `00ca5210` | ClassifyShipCombatContact | speculative | Builds an orientation candidate array, compares candidates against current forward and lateral vectors, then returns a classification value and releases the te… |
| `00ca6134` | ApplyUpdatePlayerStateFields | speculative | Uses the supplied identifiers and values to find a linked entity, adjust counters and flags, notify dependent objects, update one nested state object, then des… |
| `00ca7480` | ShipCombatFrameUpdate_00ca7480 🌐 | speculative | Per-frame update of a ship-combat processor: walks the ship list at this+0x10 (linked through +0x5d0); for ships with a non-zero +0x14 it reads position data (… |
| `00cac170` | LookupShipCombatActor | speculative | Looks up an actor from an identifier, initializes a temporary string from actor data, queries a registered object, and returns a result field through an output… |
| `00cac450` | GetReferencedObjectPlus400 | speculative | Calls a virtual method on the referenced object and returns its result plus 400. |
| `00cac490` | InitializeStructureState | speculative | Stores two supplied references and initializes remaining structure state fields to zero, a sentinel, or a current default value. |
| `00cac760` | ReleaseEffectReferences | speculative | Checks the shared effect manager and releases nonzero references at offsets 0x2c and 0x30, with an extra release when a manager flag is set. |
| `00cac7d0` | CreateTimedEffect | speculative | Releases prior effect references, builds a transform from actor fields, creates an effect object, and stores the new time value. |
| `00cac8b0` | ReleaseAndNotifyReferencedObject | speculative | Releases stored effect references and invokes virtual slot 8 on the referenced object. |
| `00cacbc0` | StructureTransformOffset | speculative | Returns the input pointer advanced by 0x30 bytes. |
| `00cacc00` | StructureTransformOffset2 | speculative | Returns the input pointer advanced by 0x30 bytes. |
| `00caccf0` | BuildStructureEffectParameters | speculative | Fills a local effect parameter structure and passes it to the effect setup helper. |
| `00cacd80` | BuildStructureEffectParametersVariantA | speculative | Fills a local effect parameter structure and passes it to the effect setup helper. |
| `00cace00` | BuildStructureEffectParametersVariantB | speculative | Fills a local effect parameter structure and passes it to the effect setup helper. |
| `00cace90` | BuildStructureEffectParametersVariantC | speculative | Fills a local effect parameter structure and passes it to the effect setup helper. |
| `00cacf10` | EnableStructureChildEffects | speculative | When the input collection has elements and a referenced object exists, calls the collection helper with arguments 1 and 0. |
| `00cad0b0` | UpdateChestEffectsByState | speculative | For state values 1 and 4, checks the actor-height predicate and calls the matching effect parameter helper when it passes. |
| `00cad100` | UpdateShipCombatStructureEffects | speculative | Updates structure effect state from actor references and timed conditions, calling effect creation, traversal, and release helpers as branches change. |
| `00cad420` | TraverseAndReleaseActorEffects | speculative | Walks actor effect references, checks flags, and releases effect objects when tracked state fails cleanup conditions. |
| `00cad4f0` | BuildStructureEffectTransform | speculative | Gets a referenced actor value, builds a transform from global vectors and local position fields, and passes it to an effect helper. |
| `00cad6b0` | RefreshStructureEffectState | speculative | Selects one actor reference, refreshes structure effects, and removes prior references when selected state changes. |
| `00cadbf0` | CopyShipCombatRecord | speculative | Copies ten leading words, invokes a copy helper for the embedded string, copies following fields, and copies a trailing nested value. |
| `00cadd40` | CreateFortCannonBarrage | speculative | Initializes embedded cannon list, resolves actor and barrage parameters, creates cannon entries, and links them into fort state. |
| `00cae510` | FindCannonByKey | speculative | Hashes a string key, selects one of 12 buckets, walks the bucket chain, and compares each entry string before returning a match or null. |
| `00caf150` | InitializeCannonMapForActor | speculative | Constructs cannon map, stores actor parameters, initializes map buckets, and records supplied actor data. |
| `00caf1f0` | InsertCannonByStringKey | speculative | Hashes the string at offset 0x5c of the supplied record and forwards key and record to map insertion. |
| `00caf390` | GrowShipCombatResultsArray | speculative | Chooses a new array capacity from requested size and current capacity, then returns the capacity through an output pointer. |
| `00caf430` | FreeShipCombatResultsArray | speculative | Passes supplied storage pointer and size 8 to the allocation helper. |
| `00caf450` | MoveOrDestroyResultsEntry | speculative | Either dispatches the entry destructor through its vtable or swaps array storage and count fields with the supplied descriptor. |
| `00caf5c0` | DestroyResultsEntryStringsFromIndex | speculative | Destroys entries from computed starting index through current array count, then stores the reduced count. |
| `00caf600` | CopyShipCombatResultsEntry | speculative | Copies entry header fields, initializes embedded IString, calls a shared copy helper for that string, and copies remaining row fields. |
| `00caf7c0` | ClearResultsDataSourceRows | speculative | Forwards count at object offset 0x260 to the results-array clear helper. |
| `00caf9c0` | InitializeResultsEntryAtIndex | speculative | Obtains or extends indexed row, initializes embedded string fields, and returns the row value. |
| `00cb0080` | RefreshShipCombatResults | speculative | Walks participants, creates or reuses result rows, copies identifiers and names, updates score and trophy fields, and marks changed columns. |
| `01069c80` | IStringDefaultsInit | speculative | Writes numeric defaults into the leading dwords of a struct (through +0x64), installs the IString vtable at +0x58 with the buffer pointer at +0x5c, and assigns… |
| `0106a300` | IStringStructZeroInit | speculative | Zeroes the first twelve dwords of a struct of at least 0x40 bytes, installs the IString vtable at +0x30 with the empty-string buffer at +0x34, and zeroes the t… |
| `0106b580` | MastEntryConfigLoad | speculative | Reads a mast-style entry: MinHealth into +0 (integer), MastTextureAlias into the string at +8 (assigned through 006f9e60), and EffectId into +0x24. |
| `01087fc0` | ShipRecordAssignAndNotify | speculative | Thiscall with three stack args (ret 0xc). Calls 01087d20 (ship record copy-assign) with the first stack argument, then calls the vtable slot 1 (+0x4) of the ob… |
| `00a4f680` | — | unknown | Releases two handle pairs of the object: for the pair at +0x3b0/+0x3b4, if +0x3b4 holds an interface pointer it calls its virtual function at slot 0 with argum… |
| `00a4f880` | — | unknown | Readiness gate of the pirate-battle loading screen: queries several subsystems (the interface at *(this+4)+0xc8 through virtual slot 0x24, FUN_0076c590, the by… |
| `00a4f9b0` | — | unknown | Looks up the interface at *(*(this+4)+0xec): if its virtual slot 0 returns 9 it is returned, otherwise FUN_0094c6f0() is queried the same way and returned when… |
| `00a4fa70` | — | unknown | Resolves the kind-9 object exactly as 00a4f9b0 (+0xec interface, then FUN_0094c6f0 fallback) and, when one is found, forwards the argument to FUN_00af75e0; ret… |
| `00a503b0` | — | unknown | Zero-argument trampoline: under an SEH frame with a stack cookie it simply calls FUN_00cabd60 (whose own code references ShipCombatProxiedProjectile::vftable) … |
| `00a50400` | — | unknown | Zero-argument trampoline: under an SEH frame with a stack cookie it simply calls FUN_00cac8b0 and returns. |
| `00a504a0` | — | unknown | Zero-argument trampoline: under an SEH frame with a stack cookie it simply calls FUN_00cad030 (whose own code references ShipCombatProxiedChest::vftable and Sh… |
| `00a528d0` | — | unknown | Piecewise-linear ratio evaluator: for thresholds param_2*c1..param_2*c4 (globals 017ea718, 01814c14, 01802764, 01809b68) it accumulates (param_1 - threshold)*s… |
| `00a52b20` | — | unknown | Applies this object's local configuration to the shared manager interface *(*(this+4)+0x8a0): calls interface virtual slots 0x18 and 0xc with 0, forwards field… |
| `00a52c20` | — | unknown | Pops a node from the object's free list (FUN_00a500a0 returns the unlinked node), queries a validator virtual (slot 0x10) on the manager at *(*(this+4)+0xe8) w… |
| `00a5faa0` | — | unknown | Constructs and parses a DestroyProxiedPickup packet, scans nodes from param_1+0x64 through link +0x3c for the decoded id, then follows the matched-node branche… |
| `00a60a90` | — | unknown | Clears the previous record array, verifies the transform object, sets capacity to the source count, converts each source pair plus the object's +0x170 value th… |
| `00a67a70` | — | unknown | Inbound handler for the ship-combat render-debug-data message (reached as case 0x23 of the family dispatcher 00a6d270): constructs a temporary ShipCombatPacket… |
| `00a69480` | — 🌐 | unknown | Uses the player-state object at processor+0x14. The raw code tests the third argument in ECX and skips the following OR when it is nonzero; when zero, OR EAX,E… |
| `00a6c3b0` | — | unknown | Constructs a CreateProxiedShip packet and parses the supplied buffer. After full consumption, a non-null two-word identifier and a nonnegative dword at packet+… |
| `00a6c630` | — | unknown | Stores param_2 in the byte flag at this+0x10 and returns early when it is unchanged. Clear path: resets the Pirates timer and hides the Pirates HUD panels thro… |
| `00af7520` | — | unknown | Thiscall helper on a ControllerShip-layout object: calls virtual +0x5c with dword +0x14 and virtual +0x68 with dword +0x20 of the block at this+0x140, then cop… |
| `00c9bb40` | — | unknown | Gated on this+0x1c4 resolving through the vtable+0x10 lookup on *(this+0x1a4). Builds a 16-dword record per entry from the object returned by vtable+0xc on thi… |
| `00c9bfe0` | — | unknown | When the byte flag at this+0x19c->+0x134->+0x308 is set and argument 4 is non-zero, calls FUN_00c53d90 with that key; if an entry is found, calls ProcessCompos… |
| `00c9c120` | — | unknown | Copies the 16-dword identity matrix at DAT_01bc2660 into a local, sets the extra local float to this+0x118 times argument 3, and when argument 2 is non-null an… |
| `00c9c260` | — | unknown | Calls FUN_00791050 on each node of the list at this+0x5b8 (next at +0x30). When the flag at this+0x1a0 -> +0x264 is set: projects a point (floats from argument… |
| `00c9c480` | — | unknown | Loops four times over the 32-bit keys at this+0x1c8..0x1d4. For each key it resolves the object through the vtable+0x10 lookup on this+0x1a4; if found, calls t… |
| `00c9c6a0` | — | unknown | Loops seven times over the 32-bit keys at this+0x1c4..0x1dc. For each key it resolves the object through the vtable+0x10 lookup on *(this+0x1a4); if found, cal… |
| `00c9c830` | — | unknown | Virtual slot 1 of the array's vftable allocates a new buffer for argument 3 (capacity) and when the pointer differs from the current buffer copies the count of… |
| `00c9cb90` | — | unknown | Walks the linked list at owner+0x214 (link +0x200, selected by this+0x10) to pick a node; computes a yaw angle with fpatan from the node's (+0x20, +0x28) minus… |
| `00c9d560` | — | unknown | Switches on argument 2 (0, 2, 3, 4) to select a config id at the world config +0x1d0, +0x1d4, +0x1d8 or +0x1dc (other values return). When that id is non-zero … |
| `00c9d8e0` | — | unknown | Chooses a state id from the owner record (+0x20, +0x28, flag +0x1c) and the current state at this+0x1e8, returning one of 0x1bbd, 0x1bbe, 0x1bbf, 0x1bc0 or the… |
| `00c9e0c0` | — | unknown | Calls 007699f0 once, then for each of 14 event names looks up the id with 0070aab0 and stores it at this+0x530 + 8*k, then calls 00c9afe0 with the id and the s… |
| `00c9e680` | — | unknown | Compares the low 16 bits of the state word at this+0x1c with the cached value at this+0x1f8 and returns on no change. On a change, for each bit it runs an acti… |
| `00c9eaf0` | — | unknown | Builds a 0x200-byte request on the stack: stores the dword at this+0x194 as its first element, copies the string at this+0x28 through 006f9e60, stores the dwor… |
| `00c9ebe0` | — | unknown | When argument 2 has a positive length at +8 and this+0x1c4 resolves through the vtable+0x10 lookup on this+0x1a4, builds the same kind of 0x200-byte request as… |
| `00ca07c0` | — | unknown | Reads the Lua integer field 'ShowUI' through 00976630 into a value seeded from this+0xb8 divided by the config at +0x1e8. When that value is zero and the local… |
| `00ca35b0` | — | unknown | Creates an exception-protected stack temporary and invokes the ShipCombatProxiedShip destructor body on it before leaving scope. |
| `00ca3690` | — | unknown | Creates an exception-protected local ShipCombatProxiedShip using 00ca1d70 and returns the supplied value. |
| `00ca40b0` | — | unknown | Increments the source reference count, allocates a 0x58-byte object, initializes it using five source fields and a global context, stores it, then balances a t… |
| `00ca41a0` | — | unknown | When receiver and nested state are available and a flag is clear, subtracts the supplied 2D point from a transformed world point and returns whether its length… |
| `00ca46d0` | — | unknown | Starts from the node at +0x10 and follows up to the requested number of +0x5d0 links, stopping on a null node; it returns no node. |
| `00ca47a0` | — | unknown | Walks nodes from +0x10 through +0x5d0; returns one at the end, or returns the first node address masked with 0xffffff00 when its +0x14 field is zero. |
| `00ca47f0` | — | unknown | Repairs predecessor and successor links, decrements the list count, invokes the ShipCombatProxiedShip cleanup helper on a stack-scoped object, then releases th… |
| `00cabe50` | — | unknown | Looks up the referenced object, copies its transform through virtual slot 0x1c, and updates a time value when the stored deadline passes. |
| `00cac470` | — | unknown | Calls virtual slot 0x10 on the referenced object and discards the returned value. |
| `00cac4f0` | — | unknown | Fetches an actor, decreases a stored float by elapsed time, and creates or refreshes an effect while actor and timer guards pass. |
| `00cac8d0` | — | unknown | Resolves an effect definition by name, builds parameters and transform data, calls the object factory, and initializes returned effect fields. |
| `00cacc90` | — | unknown | Queries an actor through a virtual method and compares returned floating-point values against a global threshold before returning a boolean. |
| `00cad750` | — | unknown | Resolves effect definitions, initializes effect data and actor references, then stores created effect handles in the caller's structure. |
| `00cae5a0` | — | unknown | Reads stored positions and times, searches actor effects, and updates a cannon record from matching effect state. |
| `00cae6f0` | — | unknown | Stores a referenced actor pointer, reads three global transform vectors, and passes a local transform structure to the transform helper. |
| `00cae7d0` | — | unknown | Updates cannon timing and positions, searches attached actor effects, and creates, changes, or removes cannon effect entries as state changes. |
| `00caebb0` | — | unknown | Copies cannon record data into a new entry, reads current ship-combat context, and updates position and related state. |
| `00caec40` | — | unknown | Builds a cannon record from actor and transform inputs, queries effect state, and inserts or updates the record in the cannon map. |
| `00caf6f0` | — | unknown | Requests storage from the array allocator, destroys existing elements, copies retained entries, and frees old storage when capacity changes. |
| `00caf870` | — | unknown | Returns indexed row when present; otherwise grows capacity, initializes intervening rows, updates count, and returns requested row address. |
| `00caf940` | — | unknown | Increases row count by one, grows capacity if needed, initializes the new row, and returns its initialized value. |
| `00cafb00` | — | unknown | Initializes row when requested index is beyond current count, then returns address computed from base plus index times 0x50. |
| `0106b090` | — | unknown | Default-initialises the camera configuration block: float defaults at +0x0..+0x1c, an IString at +0x60 pointing at a default buffer, integer defaults 0xc, 0xc … |
| `01087d20` | — | unknown | Thiscall with one stack arg (source record), returns this (ret 4), 614 bytes. Copy-assigns eight SoeUtil string members at source +0x0..+0x70 (each through 006… |

### `net.pirates`

**ShipCombatPacketCreateProxiedStructure** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a653c0` | ShipCombatPacketCreateProxiedStructure::ShipCombatPacketCre… 🌐 | confirmed | Constructor of the CreateProxiedStructure packet: base-initialises through 00a4ec00(0x1f) (which stores the BasePacket and BaseShipCombatPacket vtables, the ba… |
| `00a65440` | ShipCombatPacketCreateProxiedStructure_ScalarDtor | confirmed | Scalar deleting destructor of the CreateProxiedStructure packet: runs the packet destructor body 00a620d0 and, when bit 0 of param_2 is set, releases the objec… |

**ShipCombatPacketRenderDebugData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a65460` | ShipCombatPacketRenderDebugData::ShipCombatPacketRenderDebu… 🌐 | confirmed | Constructor of the RenderDebugData packet: base-initialises through 00a4ec00(0x23) (base opcode 0x5e plus sub-opcode 0x23), installs ShipCombatPacketRenderDebu… |
| `00a654f0` | ShipCombatPacketRenderDebugData_Dtor | confirmed | Destructor body of the RenderDebugData packet: resets the member lists to their list vtables and drains them (text list at +0x2c through 00a5d0f0, line list at… |
| `00a670f0` | ShipCombatPacketRenderDebugData_ScalarDtor | confirmed | Scalar deleting destructor of the RenderDebugData packet: runs the packet destructor 00a654f0 and, when bit 0 of param_2 is set, releases the object through th… |

**(no class)** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a64c20` | ReadThreeSubObjects | likely | Reads three sub-objects into this from the stream/handle in param_2 by forwarding to the three readers 009d65e0, 00a5cd30 and 00a5cdc0 with the destination add… |
| `00a65140` | ShipCombatPacketUpdatePlayerGameStates_Build | likely | Serialises a one-dword game-state report packet: writes the two 16-bit header values at this+4/this+8 into the output stream object param_2 through 00a61ca0, t… |
| `00a651a0` | ShipCombat_CommandPayloadWrite | likely | Serialises the ship-combat command payload into the output stream object param_2: calls the command header writer 0090ac70(stream) first, then appends the dwor… |
| `00a65980` | ShipCombat_ProcessAIDebugData | likely | Handler for the AIDebugData message: constructs a temporary ShipCombatPacketAIDebugData (00a527e0), reads the two-short header with 00a5a0a0, and when the payl… |
| `00a665e0` | ShipCombatPacketUpdatePlayer_ReadBody 🌐 | likely | Reads the UpdatePlayer packet body from the reader param_2 into the packet: reads a 32-bit flag word and then, per set bit, more data - bit 0 reads up to three… |
| `00a669d0` | CreateProxiedStructure_ReadFields | likely | Reads the CreateProxiedStructure payload into the packet: reads the two-short header with 00a5a0a0, the dword at +0xc, then decodes two sub-objects through 00a… |
| `00a67350` | ReadRenderDebugPayload | likely | Reads the RenderDebugData payload into param_1: builds a local reader over (param_2,param_3), reads the two-short header with 00a5a0a0, then decodes the sub-ob… |
| `00a66c00` | ReadShipState | speculative | Reads a ship state block into param_1: 16 consecutive floats (a 4x4 transform at +0x00..+0x3c) with bounds checks (missing data zero-fills and sets the reader … |
| `00a62280` | — | unknown | Decodes a ship-combat announcer message and turns it into UI notifications: it constructs a temporary ShipCombatPacketAnnouncerMessage (callee 00a526f0), reads… |
| `00a64ba0` | — | unknown | Lenient payload check: builds a local stream reader over (param_2,param_3), reads the two-short ship-combat header with 00a5a0a0 and the following dword with 0… |
| `00a655a0` | — | unknown | Handler for the RefreshUI message: constructs a temporary ShipCombatPacketRefreshUI (00a4eeb0), reads the two-short header with 00a5a0a0, and when the payload … |
| `00a65780` | — 🌐 | unknown | Constructs a temporary PlayerAwards packet, reads the two-short header and a dword, then processes 17 entries at 0x18-byte strides. For each non-empty 8-byte i… |
| `00a67110` | — 🌐 | unknown | Constructs and parses a temporary UpdatePlayer packet. When body flag bit 3 is set, it resolves the target object; if processor flag +0x391 is clear, it may ca… |

## Source shards

[[S0298 00a4b480-00a53380|S0298]] (21), [[S0299 00a533a0-00a59140|S0299]] (7), [[S0301 00a5da90-00a62140|S0301]] (29), [[S0302 00a62280-00a67470|S0302]] (18), [[S0303 00a674c0-00a6c3b0|S0303]] (9), [[S0304 00a6c630-00a70790|S0304]] (2), [[S0330|S0330]] (17), [[S0394|S0394]] (1), [[S0407 00c955b0-00c9baf0|S0407]] (3), [[S0408|S0408]] (50), [[S0409 00ca0f50-00ca6310|S0409]] (43), [[S0410 00ca6401-00cab4e0|S0410]] (1), [[S0411 00cab620-00cb0bb0|S0411]] (70), [[S0576|S0576]] (24), [[S0577|S0577]] (1), [[S0581|S0581]] (13)
