# System: engine.area

92 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (64)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a5430` | `ClientAreaDataManagerInterface::ScalarDeletingDestructor` | ClientAreaDataManagerInterface | Compiler-generated delete wrapper for the client area data manager interface. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008aab00` | `HashMap_NameEntry_ClientAreaDataManager_Construct` | SoeUtil::HashMap | Name index of the ClientAreaDataManager (area/zone definition lookup). | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `008d1970` | `ClientAreaDataLoader::ClientAreaDataLoader` | ClientAreaDataLoader | Builds the client object that loads area (zone) data through the asset-delivery handler interface it implements; its only caller 0090bd50 owns it. | [[Native Map/Shards/S0227 008d0920-008d5c60|S0227]] |
| `008d1a10` | `ClientAreaDataLoader::ScalarDeletingDtorThunk` | ClientAreaDataLoader | Deleting-destructor entry of the loader's secondary vtable 01802a94 (slot [0]), which points 4 bytes into the object because ClientAreaDataLoader implements two interfaces. | [[Native Map/Shards/S0227 008d0920-008d5c60|S0227]] |
| `008d1a20` | `ClientAreaDataLoader::~ClientAreaDataLoader` | ClientAreaDataLoader | Non-deleting destructor of the loader constructed by 008d1970; run by the slot-0 scalar deleting destructor 008d1ac0 and from two EH unwind funclets. | [[Native Map/Shards/S0227 008d0920-008d5c60|S0227]] |
| `008d1ac0` | `ClientAreaDataLoader::ScalarDeletingDtor` | ClientAreaDataLoader | Heap-delete path for ClientAreaDataLoader instances. | [[Native Map/Shards/S0227 008d0920-008d5c60|S0227]] |
| `00b0a370` | `ClientAreaSphere::WriteShape` | ClientAreaDataShaped<SoeUtil::Math::Sph… | Serializes a spherical client area definition. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0a410` | `ClientAreaBox::ReadShape` | ClientAreaDataShaped<SoeUtil::Math::Box… | Loads box bounds from a client area definition. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0a560` | `ClientAreaBox::WriteShape` | ClientAreaDataShaped<SoeUtil::Math::Box… | Serializes a box-shaped client area definition. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0a650` | `ClientAreaDome::ReadShape` | ClientAreaDataShaped<ClientAreaDome> | Loads dome geometry from a client area definition. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0a730` | `ClientAreaDome::WriteShape` | ClientAreaDataShaped<ClientAreaDome> | Serializes dome geometry in a client area definition. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0ac10` | `ClientAreaDataManager::Reset` | ClientAreaDataManager | Resets or clears the manager's client area entries. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0ac90` | `ClientAreaData::Dtor` | ClientAreaData | Cleans up a base client area object and owned resources. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0ad90` | `ClientAreaData::GetField_1b4` | ClientAreaData | Exposes a base client-area field through a shared virtual method. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0ae20` | `ClientAreaData::ScalarDeletingDtor` | ClientAreaData | Implements scalar deleting destruction for ClientAreaData. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0ae40` | `ClientAreaFloor::Ctor` | ClientAreaFloor | Initializes a floor-shaped client area instance. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0af00` | `ClientAreaFloor::CopyBounds` | ClientAreaFloor | Returns the floor area's stored four-component bounds. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0af30` | `ClientAreaFloor::ShapeName` | ClientAreaFloor | Provides the floor shape discriminator. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0af40` | `ClientAreaFloor::Identity` | ClientAreaFloor | Provides an identity operation in the ClientAreaFloor virtual table. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0aff0` | `ClientAreaSphere::Ctor` | ClientAreaDataShaped<SoeUtil::Math::Sph… | Initializes a spherical client area object. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0b0b0` | `ClientAreaSphere::ContainsPoint` | ClientAreaDataShaped<SoeUtil::Math::Sph… | Tests whether a point lies inside a spherical client area. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0b110` | `ClientAreaBox::Ctor` | ClientAreaDataShaped<SoeUtil::Math::Box… | Initializes a box-shaped client area object. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0b1c0` | `ClientAreaBox::ContainsPoint` | ClientAreaDataShaped<SoeUtil::Math::Box… | Tests whether a point lies inside a box-shaped client area. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0b500` | `ClientAreaDome::Ctor` | ClientAreaDataShaped<ClientAreaDome> | Initializes a dome-shaped area instance. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0b570` | `ClientAreaDome::ContainsPoint` | ClientAreaDataShaped<ClientAreaDome> | Tests point inclusion using the dome area's stored center and radius. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0b9c0` | `ClientAreaFloor::ScalarDeletingDtor` | ClientAreaFloor | Implements scalar deleting destruction for ClientAreaFloor. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0ba20` | `ClientAreaSphere::ScalarDeletingDtor` | ClientAreaDataShaped<SoeUtil::Math::Sph… | Implements scalar deleting destruction for spherical area objects. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0ba80` | `ClientAreaBox::ScalarDeletingDtor` | ClientAreaDataShaped<SoeUtil::Math::Box… | Implements scalar deleting destruction for box-shaped area objects. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0bae0` | `ClientAreaDome::ScalarDeletingDtor` | ClientAreaDataShaped<ClientAreaDome> | Implements scalar deleting destruction for dome-shaped area objects. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0c010` | `ClientAreaDataLoader::OnData` | ClientAreaDataLoader | Consumes area-definition data through the loader interface. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0c140` | `ClientAreaDataManager::CreateShape` | ClientAreaDataManager | Creates and registers concrete client-area geometry objects. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0c2d0` | `ClientAreaBox::FindIntersectingBounds` | ClientAreaDataShaped<SoeUtil::Math::Box… | Finds an indexed spatial entry intersecting this box area. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0c320` | `ClientAreaDataManager::Dtor` | ClientAreaDataManager | Destroys area-manager-owned entries and name lookup state. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0c3b0` | `ClientAreaSphere::CallGeometryHelper` | ClientAreaDataShaped<SoeUtil::Math::Sph… | Invokes a helper on the spherical geometry member. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0c3d0` | `ClientAreaDome::CallGeometryHelper` | ClientAreaDataShaped<ClientAreaDome> | Invokes a helper using the dome geometry member. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00be8e10` | `ClientAreaData::ForwardSlot24` | ClientAreaData | Forwarding virtual: delegates to the area subclass implementation at vtable slot 24. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be90d0` | `ClientAreaProperty::GetFloat24` | ClientAreaProperty | Area property getter returning the float field at +0x24; shared by the property subclasses listed in vtable slot 11. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be90e0` | `ClientAreaProperty::DtorBody` | ClientAreaProperty | Base destructor body of the area property class; called from 21 unwind funclets. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be9130` | `ClientAreaRoomTypeProperty::Ctor` | ClientAreaRoomTypeProperty | Constructs a room-type area property from its two constructor arguments. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be91e0` | `ClientAreaRoomTypeProperty::DtorBody` | ClientAreaRoomTypeProperty | Destructor body for the room-type property; runs the removal hook before the base-class vtable is restored. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be97f0` | `ClientAreaRoomTypeProperty::GetDword38` | ClientAreaRoomTypeProperty | Room-type property getter for the stored dword at +0x38. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be9800` | `ClientAreaRoomTypeProperty::SetDword38` | ClientAreaRoomTypeProperty | Room-type property setter paired with ClientAreaRoomTypeProperty::GetDword38 (slot 3). | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be9810` | `ClientAreaRoomTypeProperty::GetTypeName` | ClientAreaRoomTypeProperty | Returns the property type name 'RoomType' for the room-type property. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be9840` | `ClientAreaPhysicsProperty::Ctor` | ClientAreaPhysicsProperty | Constructs the physics area property; clamps a value on its owner object to 100 when the owner reports more. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be9930` | `ClientAreaPhysicsProperty::DtorBody` | ClientAreaPhysicsProperty | Destructor body for the physics area property; resets physics state through 00ceb240 before the base vtable is restored. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be9990` | `ClientAreaPhysicsProperty::NameHash` | ClientAreaPhysicsProperty | Hashes the physics property name for keyed lookup, as ClientAreaRoomTypeProperty::NameHash does for the room-type property. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be9f40` | `ClientAreaPhysicsProperty::GetTypeName` | ClientAreaPhysicsProperty | Returns the property type name 'Physics' for the physics area property. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be9f70` | `ClientAreaTimerProperty::Ctor` | ClientAreaTimerProperty | Constructs the timer area property from its two constructor arguments. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00bea020` | `ClientAreaTimerProperty::DtorBody` | ClientAreaTimerProperty | Destructor body for the timer area property; the reset call it makes is a no-op stub. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00bed080` | `ClientAreaCombatMusicProperty::ScalarDeletingDtor` | ClientAreaCombatMusicProperty | Implements scalar deleting destruction for the combat music property. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bed0a0` | `ClientAreaCombatMusicProperty::GetTrackId` | ClientAreaCombatMusicProperty | Gets this property object's track identifier field. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bed0b0` | `ClientAreaCombatMusicProperty::SetTrackId` | ClientAreaCombatMusicProperty | Sets this property object's track identifier field. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bed0c0` | `ClientAreaData::Enable` | ClientAreaData | Enables an area data object and propagates activation to attached listeners. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bed130` | `ClientAreaData::Disable` | ClientAreaData | Disables an area data object after its final active reference is released. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bed1a0` | `ClientAreaData::GetCenter` | ClientAreaData | Returns the area center from its shape or bounding extents. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bed250` | `ClientAreaData::SetCenter` | ClientAreaData | Moves the area shape to a supplied center point. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bed590` | `ClientAreaData::LoadXmlProperties` | ClientAreaData | Loads common client area attributes and type-specific property elements from XML. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bed720` | `ClientAreaAmbientSoundProperty::GetTrackId` | ClientAreaAmbientSoundProperty | Gets the ambient sound property track identifier. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bed730` | `ClientAreaMinimapProperty::Ctor` | ClientAreaMinimapProperty | Constructs a minimap client area property with default strings and flags. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00beddd0` | `ClientAreaMinimapProperty::Dtor` | ClientAreaMinimapProperty | Destroys the minimap property and its owned string references. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bede90` | `ClientAreaMinimapProperty::ScalarDeletingDtor` | ClientAreaMinimapProperty | Implements scalar deleting destruction for the minimap property. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bedeb0` | `ClientAreaData::Ctor` | ClientAreaData | Constructs common client area data state used by shaped and floor areas. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bee090` | `ClientAreaRandomEffectProperty::CopyFrom` | ClientAreaRandomEffectProperty | Copies random effect values from a matching property instance. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bee200` | `ClientAreaData::CreateProperty` | ClientAreaData | Creates typed area properties from XML type tags. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |

## 🟢 likely (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a5420` | `ClientAreaDataManagerInterface::~ClientAreaDataManagerInterface` | ClientAreaDataManagerInterface | Teardown step for the client area data manager interface; the class name comes from the in-body vtable restore, the destructor role from the two EH callers. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `00b0a7d0` | `LoadAreaDefinitionShapes ⚠downgrade` |  | Dispatches XML area definitions into shape-specific client area objects. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00be8e20` | `ClientAreaData::FlagSlot17Fallback` | ClientAreaData | Fallback path for an area flagged at +0x1a9 when its slot-17 query reports no result; the fallback target is not identified. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be9110` | `ClientAreaProperty::ScalarDtor` | ClientAreaProperty | Scalar deleting destructor for ClientAreaProperty. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be9820` | `ClientAreaRoomTypeProperty::ScalarDtor` | ClientAreaRoomTypeProperty | Scalar deleting destructor for the room-type area property. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be9f50` | `ClientAreaPhysicsProperty::ScalarDtor` | ClientAreaPhysicsProperty | Scalar deleting destructor for the physics area property. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00becaa0` | `ClientAreaCombatMusicProperty::Hash ⚠downgrade` | ClientAreaCombatMusicProperty | Computes the hash used by client area combat music property keys. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bed070` | `ClientAreaCombatMusicProperty::TypeName ⚠downgrade` | ClientAreaCombatMusicProperty | Supplies the serialized client-area property type identifier. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bed690` | `ClientAreaData::GetMinimumPropertyValue ⚠downgrade` | ClientAreaData | Computes a minimum property value for the area data object. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00beddc0` | `ClientAreaMinimapProperty::TypeName ⚠downgrade` | ClientAreaMinimapProperty | Supplies the serialized client-area property type identifier. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |

## 🟡 speculative (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b0bfa0` | `OpenAreasXml ?` |  | Opens an area-definition XML datasource. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00be8e60` | `AreaSlot18Out0 ?` |  | Calls the object's vtable slot +0x48 (slot 18) with a pointer to a 16-byte local (stdcall, callee pops the argument) and returns the float at the buffer's offset 0 (fld dword [esp]). | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be8e80` | `AreaSlot18Out1 ?` |  | Calls vtable slot +0x48 (slot 18) with a pointer to a 16-byte local (stdcall, callee pops) and returns the float at the buffer's offset 4 (fld dword [esp+4]). | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be8ea0` | `AreaSlot18Out2 ?` |  | Calls vtable slot +0x48 (slot 18) with a pointer to a 16-byte local (stdcall, callee pops) and returns the float at the buffer's offset 8 (fld dword [esp+8]). | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00bed370` | `ClientAreaData::GetExtent ?` |  | Reports the shape extent values for area data. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bed400` | `ClientAreaData::SetExtent ?` |  | Changes the extent of the area shape. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |

## ⚪ unknown (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b0afa0` | `—` |  | Creates a stack exception frame, calls FUN_00b0ac90 on a guarded stack-local object, and restores the prior exception chain. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0b5d0` | `—` |  | Creates an exception frame, calls FUN_00b0ac90 with a guarded stack value, then restores the previous exception chain. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0b620` | `—` |  | Creates an exception frame and invokes FUN_00b0ac90 on a guarded local value before restoring the exception chain. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0b670` | `—` |  | Creates an exception frame and invokes FUN_00b0ac90 on a stack-local guard before restoring the prior exception chain. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0b6f0` | `—` |  | Clears a result link, calls FUN_00b0b210, and returns the matched node plus 0x34 when the result is nonempty. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0bb90` | `—` |  | Clears an output link, runs FUN_00b0b210, and returns a matched node adjusted by 0x34 or null. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0bc70` | `— ⚠reject` |  | Loads selected local or global client area definitions from XML content. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00be8ec0` | `—` |  | Thiscall with one stack float: calls vtable slot +0x48 with a 16-byte stack buffer (result unused), then stores the float argument into a stack slot and calls vtable slot +0x4c with a pointer to that slot; ret 4. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be8f00` | `— ⚠reject` |  | Calls vtable slot +0x48 with a stack buffer, writes the float argument into the buffer's first dword, then calls vtable slot +0x4c with a different stack address (auStack_24); ret 4 from Ghidra's view. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be8f40` | `—` |  | Calls vtable slot +0x48 with a stack buffer, stores the argument dword at a stack slot (uStack_1c) and calls vtable slot +0x4c with the pointer auStack_24; Ghidra's frame naming differs from the bytes. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be9240` | `— ⚠reject` | ClientAreaRoomTypeProperty | Hashes the room-type property name for keyed lookup; the hash shape is the one-at-a-time string hash. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00bed810` | `— ⚠reject` | ClientAreaMinimapProperty | Computes the hash used for minimap property keys. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |

