---
type: build-guide
section: World
updated: 2026-10-12
---

# Zones and Terrain

Part of [[Build Guides]] › World. Related: [[Actors and NPCs]], [[Assets and Pack Files]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

367 accepted functions: 177 confirmed, 83 likely, 70 speculative, 37 unknown. Network-linked (`wire`): 2.

Subsystems: `world.terrain` (136), `engine.area` (134), `world.pointofinterest` (36), `ui.campaign` (25), `engine.campaign` (21), `world.area` (15)

### `engine.area`

**ClientAreaAdvJournalProperty** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bea660` | ClientAreaAdvJournalProperty::Ctor | confirmed | SEH-framed constructor (ret 8, two stack arguments). Stores the ClientAreaProperty vtable, zeroes +0x8..+0xc, +0x10..+0x14, +0x20, +0x28 and +0x30..+0x38, stor… |
| `00bea710` | ClientAreaAdvJournalProperty::DtorBody | confirmed | SEH-framed teardown. Stores the ClientAreaAdvJournalProperty vtable, calls 00ceb8d0 with one argument 0 (ECX=this), then stores the ClientAreaProperty vtable a… |
| `00bead20` | ClientAreaAdvJournalProperty::TypeName | confirmed | Virtual slot [10] of ClientAreaAdvJournalProperty. Returns the address of the literal 'AdvJournal' (mov eax, 0x1826ff8; ret). |
| `00bead30` | ClientAreaAdvJournalProperty::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]). Calls the AdvJournal DtorBody 00bea710 on this, then calls _free(this) when bit 0 of the flag argument is set (re… |
| `00bea770` | ClientAreaAdvJournalProperty::TypeNameHash | likely | Virtual slot [2] of ClientAreaAdvJournalProperty. Instruction-identical (branch targets aside) to ClientAreaTimerProperty::TypeNameHash (00bea080): hashes the … |

**ClientAreaAmbientSoundProperty** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00beb240` | ClientAreaAmbientSoundProperty::Ctor | confirmed | SEH-framed constructor (ret 0xc, three stack arguments). Stores the ClientAreaProperty vtable, zeroes the base fields as in the AdvJournal constructor, sets +0… |
| `00beb310` | ClientAreaAmbientSoundProperty::DtorBody | confirmed | SEH-framed teardown. Stores the ClientAreaAmbientSoundProperty vtable; if the owner pointer at this+0x18 is non-null and the owner byte +0x1aa is nonzero, call… |
| `00beb930` | ClientAreaAmbientSoundProperty::TypeName | confirmed | Virtual slot [10] of ClientAreaAmbientSoundProperty. Returns the literal 'Sound(Background)' when bit 0 of the dword at +0x3c is set, otherwise the literal 'Mu… |
| `00beb950` | ClientAreaAmbientSoundProperty::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]). Calls the ambient DtorBody 00beb310 on this, then calls _free(this) when bit 0 of the flag argument is set (ret 4… |
| `00bed720` | ClientAreaAmbientSoundProperty::GetTrackId | confirmed | Loads and returns the 32-bit value stored at object offset 0x38. |
| `00beb380` | ClientAreaAmbientSoundProperty::TypeNameHash | likely | Virtual slot [2] of ClientAreaAmbientSoundProperty. Instruction-identical (branch targets aside) to ClientAreaTimerProperty::TypeNameHash (00bea080): hashes th… |
| `00beb970` | ClientAreaAmbientSoundProperty::Forward_00cebc60 | likely | Virtual slot [4] (ret 4). Forwards its one stack argument to 00cebc60 with ECX=this and returns. |

**ClientAreaCombatMusicProperty** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bec9a0` | ClientAreaCombatMusicProperty::Ctor | confirmed | SEH-framed constructor (ret 8, two stack arguments). Stores the ClientAreaProperty vtable, zeroes the base fields, stores argument 1 at +0x18 and argument 2 at… |
| `00beca30` | ClientAreaCombatMusicProperty::DtorBody | confirmed | SEH-framed teardown. Stores the ClientAreaCombatMusicProperty vtable; if the owner pointer at this+0x18 is non-null and the owner byte +0x1aa is nonzero, calls… |
| `00bed080` | ClientAreaCombatMusicProperty::ScalarDeletingDtor | confirmed | Calls the class destructor, conditionally frees the object when flag bit 0 is set, and returns the object pointer otherwise. |
| `00bed0a0` | ClientAreaCombatMusicProperty::GetTrackId | confirmed | Loads and returns the 32-bit value stored at object offset 0x38. |
| `00bed0b0` | ClientAreaCombatMusicProperty::SetTrackId | confirmed | Stores the supplied 32-bit value at object offset 0x38 and returns. |
| `00becaa0` | ClientAreaCombatMusicProperty::Hash | likely | Reads the property string through vtable slot +0x28, applies the 0x401 byte hash recurrence, and calls FUN_00705bc0 for long strings. |
| `00bed070` | ClientAreaCombatMusicProperty::TypeName | likely | Returns a pointer to the literal property type name CombatMusic. |

**ClientAreaCompositeEffectProperty** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00beb9a0` | ClientAreaCompositeEffectProperty::Ctor | confirmed | SEH-framed constructor (ret 8, two stack arguments). Stores the ClientAreaProperty vtable and zeroes the base fields, stores argument 1 at +0x18 and argument 2… |
| `00bebae0` | ClientAreaCompositeEffectProperty::DtorBody | confirmed | SEH-framed teardown. Stores the ClientAreaCompositeEffectProperty vtable, calls 00cecaf0 with one argument 0 (ECX=this), then stores the ClientAreaProperty vta… |
| `00bec100` | ClientAreaCompositeEffectProperty::TypeName | confirmed | Virtual slot [10] of ClientAreaCompositeEffectProperty. Returns the address of the literal 'CompositeEffect' (mov eax, 0x1826fa8; ret). |
| `00bec110` | ClientAreaCompositeEffectProperty::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]). Calls the composite DtorBody 00bebae0 on this, then calls _free(this) when bit 0 of the flag argument is set (ret… |
| `00bebab0` | ClientAreaCompositeEffectProperty::GetPtrAt50 | likely | Virtual slot [12] of ClientAreaCompositeEffectProperty (ECX=this). Returns this+0x50 as an address (lea eax, [ecx+0x50]; ret) without reading memory. |
| `00bebac0` | ClientAreaCompositeEffectProperty::GetPtrAt60 | likely | Virtual slot [13] of ClientAreaCompositeEffectProperty (ECX=this). Returns this+0x60 as an address (lea eax, [ecx+0x60]; ret) without reading memory. |
| `00bebad0` | ClientAreaCompositeEffectProperty::GetFloat44 | likely | Virtual slot [14] of ClientAreaCompositeEffectProperty (ECX=this). Loads the float at this+0x44 onto the x87 stack and returns (fld dword ptr [ecx+0x44]; ret). |
| `00bebb40` | ClientAreaCompositeEffectProperty::TypeNameHash | likely | Virtual slot [2] of ClientAreaCompositeEffectProperty. Instruction-identical (branch targets aside) to ClientAreaTimerProperty::TypeNameHash (00bea080): hashes… |
| `00bec130` | ClientAreaCompositeEffectProperty::GetField_3c | speculative | Virtual slot [3] of ClientAreaCompositeEffectProperty (ECX=this). Returns the dword at this+0x3c (mov eax, [ecx+0x3c]; ret). |
| `00bec140` | ClientAreaCompositeEffectProperty::Wrapper_00cebf50 | speculative | Virtual slot [4] (ret 4). Forwards its one stack argument to 00cebf50 with ECX=this and returns. |

**ClientAreaData** (19)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0ac90` | ClientAreaData::Dtor | confirmed | Restores ClientAreaData and IString base vtables, destroys child nodes, and decrements the shared string buffer reference count. |
| `00b0ad90` | ClientAreaData::GetField_1b4 | confirmed | Returns the 32-bit value stored at object offset 0x1b4. |
| `00b0ae20` | ClientAreaData::ScalarDeletingDtor | confirmed | Calls ClientAreaData cleanup and frees the object when the deleting flag's low bit is set; otherwise returns the pointer. |
| `00be8e10` | ClientAreaData::ForwardSlot24 | confirmed | Loads the object's vtable at [ecx] and tail-jumps to its slot +0x60 (vtable slot 24) with the same this; no arguments, no local state (bytes: mov eax,[ecx]; mo… |
| `00beae80` | ClientAreaData::WriteXmlProperties | confirmed | Virtual slot [21] of ClientAreaData (ret 4). Writes the attribute parentAreaId when the dword at +0x1ac is not -1, writes AreaEnabled = 0 when the byte at +0x1… |
| `00bed0c0` | ClientAreaData::Enable | confirmed | Increments the active reference count, sets the enabled byte on the first reference, then notifies linked listeners or resolves and invokes the selected area t… |
| `00bed130` | ClientAreaData::Disable | confirmed | Decrements the active reference count, clears the enabled byte when it drops below one, and notifies linked listeners or the selected area target. |
| `00bed1a0` | ClientAreaData::GetCenter | confirmed | Initializes an output vector, copies a center returned by virtual slot +0x38, or computes the midpoint of the bounds returned by slot +0x3c. |
| `00bed250` | ClientAreaData::SetCenter | confirmed | Writes the supplied center into the direct shape center when available, otherwise translates both corners of the eight-float bounds by the center delta. |
| `00bed590` | ClientAreaData::LoadXmlProperties | confirmed | Reads parentAreaId and AreaEnabled attributes, then iterates Property elements and dispatches each to the selected property object's XML loader. |
| `00bedeb0` | ClientAreaData::Ctor | confirmed | Initializes the base area object, clears property links and state fields, stores parameters, sets defaults and obtains an additional default value. |
| `00bee200` | ClientAreaData::CreateProperty | confirmed | Parses the property type name, allocates and constructs a matching property object, then links it into the area list and a hash bucket. |
| `00be8e20` | ClientAreaData::FlagSlot17Fallback | likely | If the byte at this+0x1a9 equals 1, calls vtable slot 17 (+0x44); when that returns zero it tail-jumps to 01076090 with this; otherwise returns without action … |
| `00beae40` | ClientAreaData::CountNodesByTypeNameICase | likely | Thiscall with one stack argument s (ret 4). Walks the list headed at this+0x190 (next link at node+0x14). For each node calls vtable slot +0x28 on it (type-nam… |
| `00beaf80` | ClientAreaData::CallPropertySlot7ThenClearFlag | likely | Virtual slot [7] of ClientAreaData (plain ret, ECX=this). For each node in the property list at this+0x190 calls node vtable slot +0x1c (slot 7) with ECX=node,… |
| `00bed690` | ClientAreaData::GetMinimumPropertyValue | likely | Returns the minimum virtual value at slot +0x2c among linked properties, or calls the selected area's virtual slot +0x6c. |
| `00bead50` | ClientAreaData::PredicateKind1AndLinkCheck | speculative | Virtual slot [3] of ClientAreaData. ECX is not used. Takes one stack argument p (ret 4) and returns 1 when p is non-null, p+0x28 == 1, p+0x40 is non-null and 0… |
| `00bead80` | ClientAreaData::SetStateAndStamp | speculative | Thiscall with (char state, int param3), ret 8. Stores the current 64-bit tick from WindowsTickTime (007027f0) at this+0x1c8/+0x1cc. If the byte at +0x1a9 diffe… |
| `00beafb0` | ClientAreaData::CommitPendingStateAfter6000Ms | speculative | Fastcall (ECX=object), no stack arguments. If the byte at +0x1a9 differs from the byte at +0x1a8 and the 64-bit time since the stamp at +0x1c8 is at least 6000… |

**ClientAreaDataLoader** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d1970` | ClientAreaDataLoader::ClientAreaDataLoader | confirmed | Constructor of the client area-data loader: installs the base interface vtables ClientAreaDataManagerInterface (primary slot) and AssetManagement::AssetDeliver… |
| `008d1a10` | ClientAreaDataLoader::ScalarDeletingDtorThunk | confirmed | Eight-byte adjustor thunk decoded from the raw bytes (83 e9 04 / e9 a8 00 00 00): subtracts 4 from this (converting the secondary interface subobject pointer a… |
| `008d1a20` | ClientAreaDataLoader::~ClientAreaDataLoader | confirmed | Destructor body of the area-data loader: rewrites both vtable slots to ClientAreaDataLoader::vftable, releases the embedded SoeUtil::IString<char> at +0x14 (pa… |
| `008d1ac0` | ClientAreaDataLoader::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (slot [0] of the primary vtable): runs the destructor body FUN_008d1a20 and frees the object through the CRT _free thunk when b… |
| `00b0c010` | ClientAreaDataLoader::OnData | confirmed | Copies a received byte buffer into a null-terminated array, conditionally calls the area shape loader, then releases temporary storage. |

**ClientAreaDataManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0ac10` | ClientAreaDataManager::Reset | confirmed | Clears manager flags and conditionally walks its area list, unregistering and deleting eligible entries; full-clear mode delegates to list removal. |
| `00b0c140` | ClientAreaDataManager::CreateShape | confirmed | Matches a shape string against sphere, box, dome, or floor, allocates the concrete type, and inserts it into the manager. |
| `00b0c320` | ClientAreaDataManager::Dtor | confirmed | Restores manager vtables, resets a mode byte, clears area entries, destroys the name hash map, and invokes subordinate cleanup routines. |

**ClientAreaDataManagerInterface** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a5430` | ClientAreaDataManagerInterface::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: writes ClientAreaDataManagerInterface::vftable and calls _free(this) when param_2 bit 0 is set; no direct callers in the export. |
| `008a5420` | ClientAreaDataManagerInterface::~ClientAreaDataManagerInter… | likely | Destructor body: writes ClientAreaDataManagerInterface::vftable into the object vtable slot and returns; its callers are EH funclets 01584a10 and 01584a50. |

**ClientAreaDataShaped<ClientAreaDome>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0a650` | ClientAreaDome::ReadShape | confirmed | Reads three center coordinates and radius into offsets 0x1e0, 0x1e4, 0x1e8, and 0x1f0, setting the fourth coordinate from a default. |
| `00b0a730` | ClientAreaDome::WriteShape | confirmed | Writes shape, center coordinates, and radius from the dome object's geometry fields. |
| `00b0b500` | ClientAreaDome::Ctor | confirmed | Calls the base area constructor and installs the ClientAreaDataShaped<ClientAreaDome> vtable. |
| `00b0b570` | ClientAreaDome::ContainsPoint | confirmed | Computes squared distance between four point components and stored center, then compares against squared radius. |
| `00b0bae0` | ClientAreaDome::ScalarDeletingDtor | confirmed | Calls ClientAreaData cleanup and conditionally frees the object when the low flag bit is set. |
| `00b0c3d0` | ClientAreaDome::CallGeometryHelper | confirmed | Adds 0x1e0 to this, passes that adjusted pointer to FUN_00b0bbc0 together with the second argument, then returns. |

**ClientAreaDataShaped<SoeUtil::Math::Box_t<float>_>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0a410` | ClientAreaBox::ReadShape | confirmed | Reads six coordinate keys into offsets 0x1e0 through 0x1f8 and initializes the intervening values from a shared default. |
| `00b0a560` | ClientAreaBox::WriteShape | confirmed | Writes shape and six box-bound coordinate fields using values stored at offsets 0x1e0 through 0x1f8. |
| `00b0b110` | ClientAreaBox::Ctor | confirmed | Calls the base area constructor and installs the box-shaped ClientAreaDataShaped vtable. |
| `00b0b1c0` | ClientAreaBox::ContainsPoint | confirmed | Compares four point components against stored lower and upper bounds and returns true only when all four are within range. |
| `00b0ba80` | ClientAreaBox::ScalarDeletingDtor | confirmed | Calls ClientAreaData cleanup and conditionally frees the object when the low flag bit is set. |
| `00b0c2d0` | ClientAreaBox::FindIntersectingBounds | confirmed | Clears a result link, calls the spatial query with the box's stored bounds, and returns a matched node pointer adjusted by 0x34 or null. |

**ClientAreaDataShaped<SoeUtil::Math::Sphere_t<float>_>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0a370` | ClientAreaSphere::WriteShape | confirmed | Writes shape=sphere and stores center coordinates and radius from offsets 0x1e0, 0x1e4, 0x1e8, and 0x1f0. |
| `00b0aff0` | ClientAreaSphere::Ctor | confirmed | Calls the base area constructor and installs the spherical ClientAreaDataShaped vtable. |
| `00b0b0b0` | ClientAreaSphere::ContainsPoint | confirmed | Subtracts four point components from the stored center and returns whether squared distance is less than the stored radius squared. |
| `00b0ba20` | ClientAreaSphere::ScalarDeletingDtor | confirmed | Calls ClientAreaData cleanup and conditionally frees the object when the low flag bit is set. |
| `00b0c3b0` | ClientAreaSphere::CallGeometryHelper | confirmed | Adds 0x1e0 to this, passes that adjusted pointer as an argument to FUN_008e4330, then returns. |

**ClientAreaFloor** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0ae40` | ClientAreaFloor::Ctor | confirmed | Calls the base area constructor, zeros three fields, installs the ClientAreaFloor vtable, and sets a state byte at offset 0x1ab. |
| `00b0af00` | ClientAreaFloor::CopyBounds | confirmed | Copies four 32-bit values from offsets 0x1e0, 0x1e4, 0x1e8, and 0x1ec into the caller output array. |
| `00b0af30` | ClientAreaFloor::ShapeName | confirmed | Returns a pointer to the literal string floor. |
| `00b0af40` | ClientAreaFloor::Identity | confirmed | Returns its first argument unchanged. |
| `00b0b9c0` | ClientAreaFloor::ScalarDeletingDtor | confirmed | Calls ClientAreaData cleanup and frees the object when the low flag bit is set; otherwise returns the pointer. |
| `00beb090` | ClientAreaFloor::SetVec4FlagIfMoved | speculative | Virtual slot [19] of ClientAreaFloor (ret 4, one pointer argument to a 4-float vector). When the argument differs from the vector at +0x1e0..+0x1ec, computes t… |

**ClientAreaMinimapProperty** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bed730` | ClientAreaMinimapProperty::Ctor | confirmed | Initializes ClientAreaProperty fields, stores constructor values and a float constant, installs the minimap vtable, and initializes two shared strings empty. |
| `00beddd0` | ClientAreaMinimapProperty::Dtor | confirmed | Releases two reference-counted strings, invokes FUN_00ced950, and restores the base ClientAreaProperty vtable. |
| `00bede90` | ClientAreaMinimapProperty::ScalarDeletingDtor | confirmed | Calls the minimap property destructor, conditionally frees the object when flag bit 0 is set, and returns the object pointer otherwise. |
| `00beddc0` | ClientAreaMinimapProperty::TypeName | likely | Returns a pointer to the literal type identifier MiniMap. |
| `00bed810` | — | unknown | Reads the property string through vtable slot +0x28, hashes up to 50 nonzero bytes with the 0x401 recurrence, and calls FUN_00705bc0 for long strings. |

**ClientAreaPhysicsProperty** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00be9840` | ClientAreaPhysicsProperty::Ctor | confirmed | Stores ClientAreaProperty vtable and initialises the base fields as in ClientAreaRoomTypeProperty::Ctor (arguments at +0x18 and +0x1c, constant _DAT_0175b420 a… |
| `00be9930` | ClientAreaPhysicsProperty::DtorBody | confirmed | Stores the ClientAreaPhysicsProperty vtable at *this, calls 00ceb240(0) with an unwind frame, then stores the ClientAreaProperty vtable at *this and returns. |
| `00be9990` | ClientAreaPhysicsProperty::NameHash | confirmed | Same body as 00be9240 (byte-identical except the rel32 displacement of the 00705bc0 call): calls vtable slot +0x28 on this, computes the one-at-a-time hash of … |
| `00be9f40` | ClientAreaPhysicsProperty::GetTypeName | confirmed | Returns a pointer to the literal string 'Physics' (mov eax, offset; ret). |
| `00be9f50` | ClientAreaPhysicsProperty::ScalarDtor | likely | Calls 00be9930 (physics destructor body) on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |

**ClientAreaProperty** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00be90d0` | ClientAreaProperty::GetFloat24 | confirmed | Returns the float at this+0x24 (mov eax,[ecx+0x24]; ret); no other effect. |
| `00be90e0` | ClientAreaProperty::DtorBody | confirmed | Stores the ClientAreaProperty vtable at *this and returns; no members are released. |
| `00beb220` | ClientAreaProperty::CopyFieldsFrom | confirmed | Thiscall with one stack argument src (ret 4). Copies the dword at src+0x18 into this+0x18 and the dword at src+0x1c into this+0x1c. |
| `00be9110` | ClientAreaProperty::ScalarDtor | likely | Stores the ClientAreaProperty vtable at *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |

**ClientAreaRandomEffectProperty** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bec160` | ClientAreaRandomEffectProperty::Ctor | confirmed | SEH-framed constructor (ret 8, two stack arguments). Stores the ClientAreaProperty vtable, zeroes the base fields, stores argument 1 at +0x18 and argument 2 at… |
| `00bec310` | ClientAreaRandomEffectProperty::DtorBody | confirmed | SEH-framed teardown. Stores the ClientAreaRandomEffectProperty vtable, calls stub 00ced200 with one argument 0 (ECX=this), stores the AmbientRandomEffect vtabl… |
| `00bec930` | ClientAreaRandomEffectProperty::TypeName | confirmed | Virtual slot [10] of ClientAreaRandomEffectProperty. Returns the address of the literal 'RandomEffect' (mov eax, 0x1826fc4; ret). |
| `00bec950` | ClientAreaRandomEffectProperty::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]). Calls the random-effect DtorBody 00bec310 on this, then calls _free(this) when bit 0 of the flag argument is set … |
| `00bee090` | ClientAreaRandomEffectProperty::CopyFrom | confirmed | Copies the incoming property's string through the virtual string getter, compares it with the local key, and copies scalar fields when keys match. |
| `00bec380` | ClientAreaRandomEffectProperty::TypeNameHash | likely | Virtual slot [2] of ClientAreaRandomEffectProperty. Instruction-identical (branch targets aside) to ClientAreaTimerProperty::TypeNameHash (00bea080): hashes th… |
| `00bec980` | ClientAreaRandomEffectProperty::Forward_00ced500 | likely | Virtual slot [4] (ret 4). Forwards its one stack argument to 00ced500 with ECX=this and returns. |
| `00bec970` | ClientAreaRandomEffectProperty::GetField_50 | speculative | Virtual slot [3] of ClientAreaRandomEffectProperty (ECX=this). Returns the dword at this+0x50 (mov eax, [ecx+0x50]; ret). |

**ClientAreaRoomTypeProperty** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00be9130` | ClientAreaRoomTypeProperty::Ctor | confirmed | Stores ClientAreaProperty vtable, zeroes fields at +8, +0xc, +0x10, +0x14, +0x20 and the bytes at +0x28; stores its two arguments at +0x18 and +0x1c, the const… |
| `00be91e0` | ClientAreaRoomTypeProperty::DtorBody | confirmed | Stores the ClientAreaRoomTypeProperty vtable at *this, calls 00ceaea0(0) (room-type removal hook per its accepted label) with an unwind frame, then stores the … |
| `00be97f0` | ClientAreaRoomTypeProperty::GetDword38 | confirmed | Returns the dword at this+0x38 (mov eax,[ecx+0x38]; ret). |
| `00be9800` | ClientAreaRoomTypeProperty::SetDword38 | confirmed | Stores the dword argument at this+0x38; returns void (ret 4 shape per the pseudocode). |
| `00be9810` | ClientAreaRoomTypeProperty::GetTypeName | confirmed | Returns a pointer to the literal string 'RoomType' (mov eax, offset; ret). |
| `00be9820` | ClientAreaRoomTypeProperty::ScalarDtor | likely | Calls 00be91e0 (room-type destructor body) on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00be9240` | — | unknown | Calls vtable slot +0x28 (slot 10, the type-name getter) on this and computes a one-at-a-time hash of the returned string (h = (c + h) * 0x401; h ^= h >> 6 per … |

**ClientAreaTimerProperty** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00be9f70` | ClientAreaTimerProperty::Ctor | confirmed | Stores ClientAreaProperty vtable and initialises the base fields (arguments at +0x18 and +0x1c, constant _DAT_0175b420 at +0x24, zeroes elsewhere), then instal… |
| `00bea020` | ClientAreaTimerProperty::DtorBody | confirmed | Stores ClientAreaTimerProperty vtable at *this, calls 00ceb3c0 (a stub whose body is only return) with an unwind frame, then stores the ClientAreaProperty vtab… |
| `00bea630` | ClientAreaTimerProperty::TypeName | confirmed | Virtual slot [10] of ClientAreaTimerProperty. Returns the address of the literal 'Timer' (mov eax, 0x1826ff0; ret). |
| `00bea640` | ClientAreaTimerProperty::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]). Calls the timer DtorBody 00bea020 on this, then calls _free(this) when bit 0 of the flag argument is set (ret 4),… |
| `00bea080` | ClientAreaTimerProperty::TypeNameHash | likely | Virtual slot [2] of ClientAreaTimerProperty (ECX=this, ret). Calls vtable slot +0x28 (vtbl[10]) to get a C string, then hashes its bytes until NUL with h = (c … |

**SoeUtil::HashMap** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008aab00` | HashMap_NameEntry_ClientAreaDataManager_Construct | confirmed | Constructor of the client area-data name map: installs the SoeUtil::HashMap<int,ClientAreaDataManager::NameEntry,128,-1> vtable, zeroes +4/+8 and memsets the 1… |

**(no class)** (18)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0a7d0` | LoadAreaDefinitionShapes | likely | Iterates AreaDefinition elements, reads each shape discriminator, creates a matching area object, then invokes three virtual loaders on it. |
| `00b0bfa0` | OpenAreasXml | speculative | Sets a path from the argument, appends Areas.xml unless the path is GlobalClientAreas.xml, then invokes a virtual datasource method and returns its result. |
| `00be8e60` | AreaSlot18Out0 | speculative | Calls the object's vtable slot +0x48 (slot 18) with a pointer to a 16-byte local (stdcall, callee pops the argument) and returns the float at the buffer's offs… |
| `00be8e80` | AreaSlot18Out1 | speculative | Calls vtable slot +0x48 (slot 18) with a pointer to a 16-byte local (stdcall, callee pops) and returns the float at the buffer's offset 4 (fld dword [esp+4]). |
| `00be8ea0` | AreaSlot18Out2 | speculative | Calls vtable slot +0x48 (slot 18) with a pointer to a 16-byte local (stdcall, callee pops) and returns the float at the buffer's offset 8 (fld dword [esp+8]). |
| `00beb020` | ElapsedMsSinceStamp1D8 | speculative | Fastcall (ECX=object), no stack arguments. If the 64-bit stamp at +0x1d8/+0x1dc equals the global sentinel pair (DAT_01beae88/DAT_01beae8c) returns 0. Otherwis… |
| `00bed370` | ClientAreaData::GetExtent | speculative | Returns a uniform extent vector from a shape radius at offset 0x10 or subtracts the minimum bounds coordinates from the maximum bounds coordinates. |
| `00bed400` | ClientAreaData::SetExtent | speculative | Sets a direct shape radius or adjusts both corners of a bounds box around its midpoint to match the supplied extent vector. |
| `00b0afa0` | — | unknown | Creates a stack exception frame, calls FUN_00b0ac90 on a guarded stack-local object, and restores the prior exception chain. |
| `00b0b5d0` | — | unknown | Creates an exception frame, calls FUN_00b0ac90 with a guarded stack value, then restores the previous exception chain. |
| `00b0b620` | — | unknown | Creates an exception frame and invokes FUN_00b0ac90 on a guarded local value before restoring the exception chain. |
| `00b0b670` | — | unknown | Creates an exception frame and invokes FUN_00b0ac90 on a stack-local guard before restoring the prior exception chain. |
| `00b0b6f0` | — | unknown | Clears a result link, calls FUN_00b0b210, and returns the matched node plus 0x34 when the result is nonempty. |
| `00b0bb90` | — | unknown | Clears an output link, runs FUN_00b0b210, and returns a matched node adjusted by 0x34 or null. |
| `00b0bc70` | — | unknown | Obtains XML data, filters candidate elements by virtual predicates and a mode byte, creates AreaDefinition records, then writes a parse-success flag. |
| `00be8ec0` | — | unknown | Thiscall with one stack float: calls vtable slot +0x48 with a 16-byte stack buffer (result unused), then stores the float argument into a stack slot and calls … |
| `00be8f00` | — | unknown | Calls vtable slot +0x48 with a stack buffer, writes the float argument into the buffer's first dword, then calls vtable slot +0x4c with a different stack addre… |
| `00be8f40` | — | unknown | Calls vtable slot +0x48 with a stack buffer, stores the argument dword at a stack slot (uStack_1c) and calls vtable slot +0x4c with the pointer auStack_24; Ghi… |

### `engine.campaign`

**CWCampaignMap::CriticalSection** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012516f0` | CWCampaignMap::CriticalSection::ScalarDeletingDtor | confirmed | Slot [0] of CWCampaignMap::CriticalSection::vftable, 28 bytes. Stores the vtable into *this and, when bit 0 of the byte argument is set, calls _free(this) thro… |
| `012516a0` | CWCampaignMap::CriticalSection::Dtor | likely | 7 bytes: c7 01 imm32 c3. Stores CWCampaignMap::CriticalSection::vftable into the first dword of *this and returns; no free. |

**Campaign** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01484e60` | Campaign::~Campaign | confirmed | Destructor body without free: stores the Campaign vtable, then for each non-null element of the pointer list at +0x64..+0x68 calls the element's vtable slot 0 … |
| `01485030` | Campaign::TypeName | confirmed | Virtual (vtable slot 11) returning the literal 'Campaign'. |
| `01485040` | Campaign::GetClassStatic | confirmed | Virtual (vtable slot 12) returning the per-class static dword DAT_01cbe5c4. |
| `01485050` | Campaign::SetClassStatic | confirmed | Virtual (vtable slot 13) storing its argument in the per-class static dword DAT_01cbe5c4. |
| `014850a0` | Campaign::~Campaign | confirmed | Scalar deleting destructor (vtable slot 0): calls the destructor body 01484e60, then frees the block with _free when bit 0 of the argument is set; returns the … |
| `014856b0` | Campaign::Campaign | confirmed | Default constructor: stores the PersistentBase, PersistentComponent and Campaign vtables, sets the Enabled byte (+8) to 1 and the Premium Only byte (+9) to 0, … |
| `01485850` | Campaign::Factory | likely | Allocates 0x78 bytes with operator new (00d833a6), calls the Campaign constructor 014856b0 on the block when the allocation succeeds, and returns the object or… |

**CampaignScenario** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01483c60` | CampaignScenario::TypeName | confirmed | Virtual (vtable slot 11) returning the literal 'CampaignScenario'. |
| `01483c70` | CampaignScenario::GetClassStatic | confirmed | Virtual (vtable slot 12) returning the per-class static dword DAT_01cbe5c0. |
| `01483c80` | CampaignScenario::SetClassStatic | confirmed | Virtual (vtable slot 13) storing its argument in the per-class static dword DAT_01cbe5c0. |
| `01483ca0` | CampaignScenario::~CampaignScenario | confirmed | Scalar deleting destructor (vtable slot 0): calls the destructor body 01483af0, then frees the block with _free when bit 0 of the argument is set; returns the … |
| `01483e10` | CampaignScenario::CampaignScenario | confirmed | Default constructor: stores the PersistentBase, PersistentComponent and CampaignScenario vtables in turn, zeroes the int at +4, sets the six std::string member… |
| `01483af0` | CampaignScenario::~CampaignScenario | likely | Destructor body without free: stores the CampaignScenario vtable, then for each of the six std::string members (at +0xc, +0x28, +0x44, +0x60, +0x7c, +0x98, des… |
| `01483fd0` | CampaignScenario::Factory | likely | Allocates 0xc0 bytes with operator new (00d833a6), calls the CampaignScenario constructor 01483e10 on the block when the allocation succeeds, and returns the o… |

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01483cc0` | CopyStringAt0c | speculative | Returns a copy of the std::string at this+0xc by resetting the output string to the empty SSO state (capacity 15, size 0) and assigning the whole source string… |
| `01483d30` | CopyStringAt44 | speculative | Returns a copy of the std::string at this+0x44 (same body as 01483cc0 with the source offset 0x44). |
| `01483da0` | CopyStringAt60 | speculative | Returns a copy of the std::string at this+0x60 (same body as 01483cc0 with the source offset 0x60). |
| `014850c0` | CopyStringAt0cAlt | speculative | Returns a copy of the std::string at this+0xc (same body as 01483cc0 CopyStringAt0c, which is the CampaignScenario copy). |
| `01485130` | CopyStringAt44Alt | speculative | Returns a copy of the std::string at this+0x44 (same body as 01483d30 CopyStringAt44). |

### `ui.campaign`

**CWCampaignMap::CriticalSection** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012583b0` | CWCampaignMap::CriticalSection::StaticInit | likely | Guarded one-time init (flag DAT_01cbcfb0): stores CWCampaignMap::CriticalSection::vftable into static DAT_01cbcfa8, zeroes DAT_01cbcfac and registers an atexit… |

**Engine::CollectionServiceBase** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013ae840` | Engine::CollectionServiceBase::ItemDeliveryNotify | confirmed | Virtual slot 6: if the flag byte at this+0xdc is set and this+0xb0 is zero, it reads the collection state (012c5080, 012c4060), builds the strings Delivery and… |

**Engine::FRTWCampaignScreenLoaded** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01250df0` | Engine::FRTWCampaignScreenLoaded::TypeName | confirmed | Slot [1] of Engine::FRTWCampaignScreenLoaded::vftable. Returns the constant C string 'CampaignScreenLoaded'. |
| `01250e10` | Engine::FRTWCampaignScreenLoaded::DebugName | confirmed | Slot [4] of its vftable. Returns the constant C string 'CampaignScreenLoaded : 0 : .\engine\campaign\frtwcampaignhandlers.cc:24'. |
| `01250e20` | Engine::FRTWCampaignScreenLoaded::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01250d80` | Engine::FRTWCampaignScreenLoaded::ctor | likely | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWCampaignScreenLoaded::vftable into the object; when the int argument is nonze… |
| `012510d0` | — | unknown | Slot [2] of Engine::FRTWCampaignScreenLoaded::vftable, 1426 bytes. Creates an action named 'InitializeCampaigns' (FUN_012ec220) and then, in a loop of 40 itera… |

**Engine::FRTWCampaignScreenUpdate** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01250cb0` | Engine::FRTWCampaignScreenUpdate::DebugName | confirmed | Slot [4] of its vftable. Returns the constant C string 'CampaignScreenUpdate : 0 : .\engine\campaign\frtwcampaignhandlers.cc:52'. |
| `01250cc0` | Engine::FRTWCampaignScreenUpdate::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01250c20` | Engine::FRTWCampaignScreenUpdate::ctor | likely | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWCampaignScreenUpdate::vftable into the object; when the int argument is nonze… |
| `01250c90` | Engine::FRTWCampaignScreenUpdate::TypeName | likely | Slot [1] of Engine::FRTWCampaignScreenUpdate::vftable. Returns the constant C string 'CampaignScreenUpdate'. |
| `01250f80` | Engine::FRTWCampaignScreenUpdate::RefreshIfActive | speculative | Slot [2] of Engine::FRTWCampaignScreenUpdate::vftable, 31 bytes. Calls FUN_012e0ba0, then FUN_0124d010; if that returns nonzero it calls FUN_0131faa0; returns … |

**UI::CampaignScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f81b0` | UI::CampaignScreen::CampaignScreen | confirmed | Constructor (fastcall, ECX = this): calls UI::FRTWCampaignScreen::FRTWCampaignScreen 013ff620 (which runs base init 01511db0 and stores its own vtable), then s… |
| `012f8210` | UI::CampaignScreen::DtorBody | confirmed | Non-deleting destructor body: stores UI::CampaignScreen::vftable at *this, then calls 013ff680 (UI::FRTWCampaignScreen::DtorBody). |
| `012f8260` | UI::CampaignScreen::~CampaignScreen | confirmed | Scalar deleting destructor (vtable slot [0]): stores UI::CampaignScreen::vftable, calls the destructor body 013ff680, then calls _free on this when bit 0 of th… |

**UI::CampaignScreenBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01511e10` | UI::CampaignScreenBase::~CampaignScreenBase | confirmed | Stores UI::CampaignScreenBase::vftable into *this, then calls FUN_012f8480 (base destructor) and returns. |
| `01511e60` | UI::CampaignScreenBase::ScalarDeletingDtor | confirmed | Virtual slot 0 of UI::CampaignScreenBase. Stores the vtable, calls FUN_012f8480, then calls _free on this if flag bit 0 of the second argument is set. Returns … |
| `01511db0` | UI::CampaignScreenBase::CampaignScreenBase | likely | Calls FUN_012f8420 on this, then stores UI::CampaignScreenBase::vftable into *this and returns this. |

**UI::FRTWCampaignScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013ff620` | UI::FRTWCampaignScreen::FRTWCampaignScreen | confirmed | Constructor: calls 01511db0 on this (base initialisation), then stores UI::FRTWCampaignScreen::vftable (018e3e04) into *this and returns this. |
| `013ff680` | UI::FRTWCampaignScreen::DtorBody | confirmed | Non-deleting destructor body: stores UI::FRTWCampaignScreen::vftable into *param_1, calls 01511e10 on the object, returns. Does not free. |
| `013ff6f0` | UI::FRTWCampaignScreen::~FRTWCampaignScreen | confirmed | Scalar deleting destructor: stores UI::FRTWCampaignScreen::vftable into *this, calls 01511e10 on this, frees this through _free when bit 0 of the stack byte is… |

**UI::StartCampaignMode** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124dfe0` | UI::StartCampaignMode::OpenMenuScreen | likely | Virtual method in slot [2] of UI::StartCampaignMode::vftable. If FUN_00f9c130() returns nonzero it builds a std::string 'UI::MenuScreen' and passes it with (1,… |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0131faa0` | CampaignScreen_PushState | likely | Calls FUN_012d08e0 (returns an object), then vtable slot +4 with id 0x1054 to fetch a ValueData (asserts mTypeID == kIntValueMapTypeID when not 0xe). Loops ove… |
| `00f9c7d0` | — | unknown | Thiscall with an action name and a value: when the owner-thread lookup 00f9c130 is non-null and the name begins with 'goCampaign', allocates 0x40 bytes, constr… |
| `00f9c8f0` | — | unknown | Thiscall-style with an action name and a value: when the owner-thread lookup 00f9c130 is non-null and the name begins with 'campaign', allocates 0x40 bytes, co… |

### `world.area`

**ClientAreaAdvJournalProperty** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ceb810` | ClientAreaAdvJournalProperty::LoadRegionOverride | speculative | Reads RegionOverride from the property source and stores the supplied integer at object offset +0x38. |
| `00ceb840` | ClientAreaAdvJournalProperty::Log | speculative | Logs inherited property fields and the RegionOverride value at index 14. |
| `00ceb890` | ClientAreaAdvJournalProperty::Apply | speculative | For an active area object whose type and pointer fields are valid, checks a helper, then passes RegionOverride to a state tracking helper. |
| `00ceb8d0` | ClientAreaAdvJournalProperty::Remove | speculative | For an active area object whose type and pointer fields are valid, checks a helper, then removes RegionOverride through a state tracking helper. |

**ClientAreaPhysicsProperty** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ceaf20` | ClientAreaPhysicsProperty::Log | speculative | Logs inherited property fields plus JumpHeightScalar, GravityScalar, BounceScalar, BounceVelocityCap, and AutoJump. |
| `00ceb120` | ClientAreaPhysicsProperty::Load | speculative | Loads four named physics values with atof, reads AutoJump as a boolean, conditionally updates a live object, and clamps a nested value to 250. |
| `00ceb240` | ClientAreaPhysicsProperty::Reset | speculative | For an active object with type 1, resets selected physics fields at +0x490/+0x498 to the default float and zeros +0x49c/+0x4a0 when property values are non-def… |
| `00ceb2d0` | ClientAreaPhysicsProperty::Apply | speculative | For an active object of type 1, copies physics properties to it; if AutoJump is enabled, sets bit 0x10 at object+0x142 and calls a state helper. |

**ClientAreaRoomTypeProperty** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ceae00` | ClientAreaRoomTypeProperty::WriteRoomTypeId | speculative | Writes the property name RoomTypeId and the field at object offset +0x38 through the supplied serializer helper. |
| `00ceae20` | ClientAreaRoomTypeProperty::Log | speculative | Obtains a formatted base description, writes the inherited field at index 7, and logs RoomTypeId from index 14. |
| `00ceae70` | ClientAreaRoomTypeProperty::OnApply | speculative | If the client setting at global offset +0x24c is nonzero, passes fields at +0x1c and +0x38 to the apply helper. |
| `00ceaea0` | ClientAreaRoomTypeProperty::OnRemove | speculative | If the client setting at global offset +0x24c is nonzero, passes fields at +0x1c and +0x38 to the remove helper. |

**ClientAreaTimerProperty** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ceb330` | ClientAreaTimerProperty::Load | speculative | Reads StartTimer from the property source and stores its nonzero value as a boolean at object offset +0x38. |
| `00ceb370` | ClientAreaTimerProperty::Log | speculative | Logs the inherited property fields and StartTimer boolean. |
| `00ceb740` | ClientAreaTimerProperty::SendTimerPacket | speculative | If StartTimer is true constructs PacketClientAreaTimer(1,-1); otherwise constructs it with (2,-1); sends only when gateway field +0x280 is nonzero. |

### `world.pointofinterest`

**ClientPointOfInterestManager** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b980d0` | ClientPointOfInterestManager_Destruct | confirmed | Destructor of the point-of-interest manager: stores ClientPointOfInterestManager::vftable, releases the data source pointer at +0x36e4 (virtual slot 0x28 with … |
| `00b981f0` | ClientPointOfInterestManager_DeletingDestruct | confirmed | Scalar deleting destructor: calls the manager destructor 00b980d0 and then _free(this) when bit 0 of param_2 is set. |
| `00b98e10` | ClientPointOfInterestManager_Construct | confirmed | Constructor of the point-of-interest manager: runs the PointOfInterestDefinitionManager base constructor FUN_010313c0, stores ClientPointOfInterestManager::vft… |
| `00b991d0` | ClientPointOfInterestManager_SendTeleportRequest 🌐 | confirmed | Validates a point-of-interest id against the local table (FUN_0102fb50) and, when the id is known, constructs the PacketZoneTeleportRequest with 00b97b80 and s… |
| `00b97ea0` | ClientPointOfInterestManager_UnlinkRowFromBucket | likely | Removes a row from the manager hash table: computes the bucket from the row key at param_2+0x5c modulo 1000, walks the chain at this+0x14 (next pointer at row+… |
| `00b98160` | ClientPointOfInterestManager_RemoveRow | likely | Removes one point-of-interest row: unlinks it from its hash bucket with 00b97ea0, unlinks it from the doubly linked row list (prev at +0x54, next at +0x58, hea… |
| `00b98220` | ClientPointOfInterestManager_RemoveAllRows | likely | Loops while the row list head at this+8 is non-null, removing each row through 00b98160 (its argument is read back from this+8 on every iteration); returns wit… |
| `00b98b70` | ClientPointOfInterestManager_OnDefinitionReply 🌐 | likely | Receive handler for the point-of-interest definition reply: constructs a PacketPointOfInterestDefinitionReply temporary (00b97ad0), builds a reader over (buffe… |
| `00b99150` | ClientPointOfInterestManager_SendDefinitionRequest | likely | Constructs a PacketPointOfInterestDefinitionRequest temporary with 00b97a20 and, when the session pointer at this+0x36e0 -> +0x280 is non-null, sends it throug… |

**ClientPointOfInterestManager::InternalDs** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b97c40` | ClientPointOfInterestManager_InternalDs_GetColumnName | confirmed | Column-name table of the point-of-interest data source: a switch over the column index returns the literal names Id, Name, Zone ID, X, Y, Z, Heading, IconId, N… |
| `00b98ce0` | ClientPointOfInterestManager_InternalDs_Destruct | confirmed | Destructor body of the point-of-interest data source: stores the InternalDs vtable into the three interface slots at +0, +0x30 and +0x34 and calls the base Gui… |
| `00b98d70` | ClientPointOfInterestManager_InternalDs_Construct | confirmed | Constructor of the point-of-interest data source: runs the base IDataAccessTable/GuiUserDataTable constructor FUN_00922890 with param_2, stores the InternalDs … |
| `00b98df0` | ClientPointOfInterestManager_InternalDs_DeletingDestruct | confirmed | Scalar deleting destructor: calls the data source destructor 00b98ce0 and then _free(this) when bit 0 of param_2 is set. |
| `00b97cf0` | InternalDs_Thunk_slot2_GetRowCount | likely | Adjustor thunk of 8 bytes: adds 4 to this and tail-jumps to FUN_010c8f30, the generic data source implementation this interface slot forwards to. |
| `00b97d00` | ClientPointOfInterestManager_InternalDs_RefreshAllRows | likely | Calls the virtual at slot 3 of the helper object at this+0x34, then iterates the whole point-of-interest table from FUN_0102fb90 through the next-row accessor … |
| `00b987b0` | — | unknown | Column value accessor: rejects column indices at or above the row count or missing rows, looks the point-of-interest row up through FUN_0102fb50 by the id at r… |

**PointOfInterestDefinitionManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010311d0` | PointOfInterestDefinitionManager::DtorBody | confirmed | Fastcall destructor body (ECX = manager): stores the PointOfInterestDefinitionManager vtable at +0, adds 4 to ECX and calls the HashListMap destructor body 010… |
| `010313c0` | PointOfInterestDefinitionManager::Construct | confirmed | Fastcall ctor (ECX = manager): installs the PointOfInterestDefinitionManager vtable at +0, calls the HashList constructor 01031300 on this+4, and returns this. |
| `01031420` | PointOfInterestDefinitionManager::ScalarDeletingDtor | confirmed | Scalar deleting destructor (ECX = this, flag argument): calls the manager destructor body 010311d0, calls _free(this) when bit 0 of the flag is set, returns th… |

**SoeUtil::HashList<PointOfInterestDefinition,1000,100>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01031300` | HashList_PointOfInterestDefinition_Construct | confirmed | Fastcall ctor (ECX = this): calls the HashListMap constructor 01031100 on this, then installs the SoeUtil::HashList<PointOfInterestDefinition,1000,100> vtable … |
| `01031360` | HashList_PointOfInterestDefinition_ScalarDeletingDtor | confirmed | Scalar deleting destructor (ECX = this, flag argument): calls the HashListMap destructor body 01030630 on this, calls _free(this) when bit 0 of the flag is set… |

**SoeUtil::HashListMap<int,PointOfInterestDefinition,1000,100>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010310c0` | HashListMap_PointOfInterestDefinition_AllocNodeThunk | confirmed | Virtual slot [1] thunk: adds 0xfb4 to ECX (the embedded node pool) and tail-jumps to FreeNodePoolPop (01030fe0). |
| `010310d0` | HashListMap_PointOfInterestDefinition_FreeNodeThunk | confirmed | Virtual slot [2] thunk: adds 0xfb4 to ECX and tail-jumps to FreeNodePush (01031050). |
| `010310e0` | HashListMap_PointOfInterestDefinition_ScalarDeletingDtor | confirmed | Scalar deleting destructor (ECX = this, flag argument): calls the HashListMap destructor body 01030630 on this, calls _free(this) when bit 0 of the flag is set… |
| `01031100` | HashListMap_PointOfInterestDefinition_Construct | confirmed | Fastcall ctor (ECX = this). Installs the HashListMap<int,PointOfInterestDefinition,1000,-1> vtable, zeroes +0x10, +0x8, +0xc and memsets the 1000-entry bucket … |
| `01031180` | HashListMap_PointOfInterestDefinition_UnwindDtor | likely | EH unwind cleanup funclet: pushes ECX (the partly built map) into its frame, calls the HashListMap destructor body 01030630 on it, restores the EH chain and re… |

**(no class)** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ce560` | SelectPointOfInterestAndRequestTeleport | likely | Copies a supplied Vector4 into state, compares current and stored paths, scans type-7 records, selects a closer eligible record, requests teleport by its id, u… |
| `009cead0` | UpdatePointPathState | likely | Updates state flags, captures a position, clears existing linked entries, prunes nodes against a computed path, removes near-duplicate Vector4 nodes, and notif… |
| `009ced20` | UpdatePointPathTransitions | likely | Checks elapsed-time fields and state flags, resets the path state on timeout, requests route updates under state-specific intervals, and clears a flag when the… |
| `009ceee0` | RestartPointPathFromArgument | likely | Resets path-related state, sets a flag at this+0x8c3, and invokes FUN_009ce560 with the supplied argument. |
| `009cef10` | RestorePointPathFromStoredVector | likely | When the byte at this+0x8c2 is set, resets path state, sets this+0x8c3, and calls FUN_009ce560 with the Vector4 at this+0x10. |
| `00b98070` | PoiDefinitionRow_DeletingDestruct | likely | Scalar deleting destructor of the point-of-interest definition row: calls FUN_01030030 (the row destructor) and then _free(this) when bit 0 of param_2 is set. |
| `00b98290` | PoiDefinitionRow_ReadFields | likely | Reads one point-of-interest definition row from a wire reader into param_1: 13 dwords into row indices 0-7, 10-12 and 14-15 (indices 8, 9 and 13 skipped), ever… |
| `00b98990` | PoiDefinitionReply_ReadRowStream | likely | Reads the framed point-of-interest record stream from a reader: first consumes one flag byte (returning when it cannot be read), then loop-constructs a row (FU… |
| `00b97e10` | — | unknown | SEH-framed destructor emission for a point-of-interest definition row: calls FUN_01030030 (the row destructor that carries the IString members) under a try/exc… |
| `00b98510` | — | unknown | Walks the row list at this+0xc (next at row+0x15 dword index) and, for every row, formats a debug line into a local IString with the field labels NameId, Locat… |

### `world.terrain`

**ApplicationObjectTerrainDataManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010bf100` | ApplicationObjectTerrainDataManager_DtorBody | confirmed | Fastcall destructor body with this in ECX. Stores ApplicationObjectTerrainDataManager::vftable into [this], releases the IString at +0x54 (its buffer at +0x58 … |
| `010bf230` | ApplicationObjectTerrainDataManager_Ctor | confirmed | Fastcall constructor body with this in ECX. Stores ApplicationObjectTerrainDataManager::vftable into [this], runs the HashList ctor body 010bf190 on this, then… |
| `010bf2b0` | ApplicationObjectTerrainDataManager_ScalarDeletingDtor | confirmed | Thiscall with one flag byte (ret 4). Calls 010bf100 (manager dtor body) on this, frees this through the _free thunk when bit 0 of the flag is set, and returns … |

**GameRuntimeTerrain::Chunk** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d4dc20` | ChunkDtor | confirmed | Cancels and releases job references, frees owned buffers, clears the Tile array, releases reference-counted resources, destroys render-batch arrays, and resets… |
| `00d4dea0` | AdjustorThunk_00d4e210 | confirmed | Subtracts 8 from ECX, then jumps to 00d4e210; this is an adjustor thunk, not the deleting-destructor body. |
| `00d4e0a0` | ChunkCtor | confirmed | Installs base and Chunk vtables, stores the supplied handler pointer, clears chunk state fields, initializes Tile and render-batch arrays, and stores the handl… |
| `00d4e210` | ChunkScalarDeletingDtor | confirmed | Runs Chunk cleanup, conditionally frees the Chunk object when flag bit 0 is set, and otherwise returns the object pointer. |
| `00d3a2a0` | ChunkCtorWithKey | likely | Thiscall with two stack pointers (key pointer, handler pointer): calls ChunkCtor 00d4e0a0 with the dword at the handler pointer, then stores the dword at the k… |
| `00d3b570` | GameRuntimeTerrain::Chunk::ScalarDeletingDtor | likely | Thiscall scalar deleting destructor (ret 4, SEH-framed): calls the Chunk destructor body 00d4dc20 with this in ECX, then calls _free(this) when bit 0 of the fl… |
| `00d39ec0` | ChunkDtorEHCleanup | speculative | Unwind cleanup funclet with an SEH frame: calls the Chunk teardown 00d4dc20 with ECX (the object) and then restores the previous exception frame and returns. |

**GameRuntimeTerrain::ChunkCache** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d3eb30` | GameRuntimeTerrain::ChunkCache::Slot5GzneName | confirmed | Virtual slot [5] of ChunkCache (thiscall, no stack args; returns bool): formats the IString '%s.gzne' with the dword at this+0xc4ddc through 00402fd0 (IStringF… |
| `00d3ab00` | — | unknown | Virtual slot [3] of ChunkCache: calls slot [6] of this with (param_2+4, param_2+8); when that returns false it calls slot [4] of this with the value 3, the thi… |

**GameRuntimeTerrain::FloraDefinition** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d3cab0` | GameRuntimeTerrain::FloraDefinition::FloraDefinition | confirmed | Fastcall constructor body: stores the FloraDefinition vftable at +0; embeds two IString members, one with vftable at +4 (buffer +8 set to the shared empty stri… |
| `00d3cf60` | GameRuntimeTerrain::FloraDefinition::~FloraDefinition | confirmed | Fastcall destructor body: stores the FloraDefinition vftable at +0; when the object at +0x2c is non-null calls its vtable slot 0 with flag 1 (deleting destruct… |
| `00d3e210` | GameRuntimeTerrain::FloraDefinition::ScalarDeletingDtor | confirmed | Virtual slot [0] of FloraDefinition (thiscall, flag stack arg): calls the destructor body 00d3cf60 on this, then _free(this) when bit 0 of the flag is set; ret… |
| `00d3e0a0` | GameRuntimeTerrain::FloraDefinition::DtorThunk | likely | SEH-framed wrapper (plain ret): saves the exception frame and calls the FloraDefinition destructor body 00d3cf60 with ECX=this (preserved from entry), then res… |
| `00d3e950` | GameRuntimeTerrain::FloraDefinition::ScalarDeletingDtorB | likely | Thiscall scalar deleting destructor copy (flag stack arg): calls the FloraDefinition destructor body 00d3cf60 on this, then _free(this) when bit 0 of the flag … |

**GameRuntimeTerrain::FloraDefinition::Texture** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d39600` | FloraDefinitionTextureCtor | confirmed | Thiscall with one stack argument (owner object): stores the AssetRequestInterface::AssetRequester vtable, copies the dword at owner+0x34 into +4, then installs… |
| `00d3aa10` | GameRuntimeTerrain::FloraDefinition::Texture::ScalarDeletin… | confirmed | Virtual slot [0] of the Texture class: calls the dtor body 00d39670 on this, then _free(this) when bit 0 of the flag argument is set; returns this. |
| `00d39670` | — | unknown | Destructor body (no free): stores the FloraDefinition::Texture vtable, releases the ref-counted object at +8 (calling owner slot +0x3c when its count is 1, the… |
| `00d39710` | — | unknown | Vtable slot [1] of FloraDefinition::Texture with one pointer argument: stores it at this+8, increments its count at +4 with a locked add and returns 1. |
| `00d39740` | — | unknown | Vtable slot [3] of FloraDefinition::Texture with one argument: allocates 0x34 bytes through 00d833a6; when non-null it calls TextureAsset ctor 00d355b0 on the … |

**GameRuntimeTerrain::FloraManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d39800` | FloraManager::VSlot3ForwardIfField628 | speculative | Vtable slot [3] of GameRuntimeTerrain::FloraManager with one argument: when the dword at this+0x628 is non-zero calls 00758cb0 with (argument+8, 1); returns wi… |

**GameRuntimeTerrain::InvisibleWall** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d3a720` | GameRuntimeTerrain::InvisibleWall::~InvisibleWall | confirmed | Destructor body: stores the InvisibleWall vftable at +0, and when +0x14 is set calls 00769710 then 00fd8160 with the +0x14 object; releases the refcounted obje… |
| `00d3b5d0` | GameRuntimeTerrain::InvisibleWall::InvisibleWall | confirmed | Fastcall constructor body: stores the InvisibleWall vftable at +0 and zeroes the dwords at +4, +8, +0xc, +0x10 and +0x14. |
| `00d3bea0` | GameRuntimeTerrain::InvisibleWall::ScalarDeletingDtor | confirmed | Virtual slot [0] of InvisibleWall (thiscall, flag stack arg): calls the destructor body 00d3a720 on this, then _free(this) when bit 0 of the flag is set; retur… |
| `00d3b760` | InvisibleWallDtorThunk | likely | SEH-framed wrapper (plain ret, no stack args): pushes the scope frame, stores ECX (this) at [esp+4], calls the InvisibleWall destructor body 00d3a720 with ECX=… |
| `00d3ca50` | GameRuntimeTerrain::InvisibleWall::ScalarDeletingDtorSeh | likely | Thiscall scalar deleting destructor with an SEH frame (ret 4): calls the InvisibleWall destructor body 00d3a720 with this in ECX, then calls _free(this) when b… |

**GameRuntimeTerrain::RuntimeObject** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d3e0f0` | GameRuntimeTerrain::RuntimeObject::~RuntimeObject | confirmed | Fastcall SEH-framed destructor body: stores the RuntimeObject vftable at +0; when the refcounted object at +0x188 is set it decrements that object's count at +… |
| `00d3e1f0` | GameRuntimeTerrain::RuntimeObject::ScalarDeletingDtor | confirmed | Virtual slot [0] of RuntimeObject (thiscall, flag stack arg): calls the destructor body 00d3e0f0 on this, then _free(this) when bit 0 of the flag is set; retur… |
| `00d3e9d0` | GameRuntimeTerrain::RuntimeObject::RuntimeObject | confirmed | Fastcall constructor body: stores the ActorHandler vftable then the RuntimeObject vftable at +0; zeroes +4 and +8; runs the embedded StringFixed<64> constructo… |
| `00d3e540` | GameRuntimeTerrain::RuntimeObject::DtorThunk | likely | SEH-framed wrapper (plain ret): saves ECX (this) on entry, calls the RuntimeObject destructor body 00d3e0f0 with that receiver, then restores the frame and ret… |
| `00d3e1e0` | GameRuntimeTerrain::RuntimeObject::ClearField194 | speculative | Virtual slot [13] of RuntimeObject (13 bytes): stores zero to the dword at +0x194 and returns. |

**GameRuntimeTerrain::RuntimeTerrainSystem** (30)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d368d0` | RuntimeTerrainSystem::DtorBody | confirmed | Destructor body (no free of this): stores the RuntimeTerrainSystem vtable at +0, +4 and +0xc; releases the objects at +0x1c and +0x18 through their slot 0 with… |
| `00d36a20` | RuntimeTerrainSystem::AdjustedDeletingDtorSec4 | confirmed | Adjustor thunk for the secondary vtable at +4 (slot [0]): subtracts 4 from ECX and jumps to the scalar deleting destructor 00d36c90. |
| `00d36a30` | RuntimeTerrainSystem::AdjustedDeletingDtorSec0c | confirmed | Adjustor thunk for the secondary vtable at +0xc (slot [0]): subtracts 0xc from ECX and jumps to the scalar deleting destructor 00d36c90. |
| `00d36a40` | RuntimeTerrainSystem::Ctor | confirmed | Thiscall constructor: installs LoadableZoneInterface, AssetDeliveryHandlerInterface and TerrainEntityManagerInterface vtables at +0, +4 and +0xc and then the R… |
| `00d36c90` | RuntimeTerrainSystem::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the destructor body 00d368d0 and frees the block through _free when bit 0 of the argument is set; returns this. |
| `00d36f20` | RuntimeTerrainSystem::CheckTilesReadyInRadius | likely | Vtable slot [19] of RuntimeTerrainSystem with args (position float pointer, radius float). When the pointer at this+0x18 is set, its byte at +0xc4de8 is set an… |
| `00d362a0` | RuntimeTerrainSystem::ReleaseChildren | speculative | Vtable slot [1] of RuntimeTerrainSystem: if the object at +0x1c is set, calls its slot 0 with flag 1 and clears the pointer; the same for +0x18; calls 00d37740… |
| `00d362f0` | RuntimeTerrainSystem::GetByte10 | speculative | Vtable slot [2] of RuntimeTerrainSystem: returns the byte at this+0x10 (4 bytes). |
| `00d36300` | RuntimeTerrainSystem::SetByte10 | speculative | Vtable slot [3] of RuntimeTerrainSystem: stores the byte argument at this+0x10 and returns with ret 4. |
| `00d36310` | RuntimeTerrainSystem::IsField18Null | speculative | Vtable slot [4] of RuntimeTerrainSystem: returns 1 when the dword at this+0x18 is zero, else 0 (cmp, sete). |
| `00d36320` | RuntimeTerrainSystem::GetField1cField14OrZero | speculative | Vtable slot [12] of RuntimeTerrainSystem: if the pointer at this+0x1c is non-null returns the dword at its +0x14, otherwise returns 0. |
| `00d36330` | RuntimeTerrainSystem::SetField30AndForward | speculative | Vtable slot [13] of RuntimeTerrainSystem: stores the dword argument at this+0x30; when the pointer at this+0x1c is non-null tail-jumps to 00d38830 with that ob… |
| `00d36350` | RuntimeTerrainSystem::ForwardIfField1c_00d38110 | speculative | Vtable slot [20] of RuntimeTerrainSystem: when the pointer at this+0x1c is non-null, tail-jumps to 00d38110 with it as ECX; ret 4 otherwise. |
| `00d36360` | RuntimeTerrainSystem::ForwardIfField1c_00d38140 | speculative | Vtable slot [21] of RuntimeTerrainSystem: when the pointer at this+0x1c is non-null, tail-jumps to 00d38140 with it as ECX; ret 4 otherwise. |
| `00d36370` | RuntimeTerrainSystem::SetByteField1c30 | speculative | Vtable slot [15] of RuntimeTerrainSystem: when the pointer at this+0x1c is non-null stores the byte argument at its +0x30; ret 4. |
| `00d36390` | RuntimeTerrainSystem::ForwardToField1c_00d38ef0 | speculative | Vtable slot [22] of RuntimeTerrainSystem: loads ECX from this+0x1c without a null check and tail-jumps to 00d38ef0. |
| `00d363a0` | RuntimeTerrainSystem::QueryFromVec2Field1c | speculative | Vtable slot [16] of RuntimeTerrainSystem: when the pointer at this+0x1c is non-null, pushes the float pair at argument+0 and argument+8 and calls 00d38260 on i… |
| `00d363e0` | RuntimeTerrainSystem::QueryIfField18_00d3bfb0 | speculative | Vtable slot [17] of RuntimeTerrainSystem: when the dword at this+0x18 is non-zero tail-jumps to 00d3bfb0 with that object as ECX; otherwise returns 0 (ret 4). |
| `00d36400` | RuntimeTerrainSystem::ForwardIfField18_00d3c020 | speculative | Vtable slot [18] of RuntimeTerrainSystem: when the dword at this+0x18 is non-zero tail-jumps to 00d3c020 with that object as ECX; ret otherwise. |
| `00d36410` | RuntimeTerrainSystem::SetDword34 | speculative | Vtable slot [11] of RuntimeTerrainSystem: stores the dword argument at this+0x34 (ret 4). |
| `00d364a0` | RuntimeTerrainSystem::SecondaryCheckThenNotify | speculative | Secondary-vtable slot [2] at +0xc of RuntimeTerrainSystem (0183f598): calls 00d3ab40 with the argument; when that returns nonzero calls 00d433f0 and, when that… |
| `00d36580` | RuntimeTerrainSystem::GetPtrField868OrDefault | speculative | Vtable slot [5] of RuntimeTerrainSystem: when the pointer at this+0x1c is non-null returns the dword at its +0x868, otherwise returns the address constant 0x17… |
| `00d36630` | RuntimeTerrainSystem::UpdateTimed | speculative | Vtable slot [9] of RuntimeTerrainSystem with one argument (a time value). Seeds this+0x98 with the argument when zero; when the argument minus this+0x98 exceed… |
| `00d367a0` | RuntimeTerrainSystem::SecondaryLookupAndRegister | speculative | Secondary-vtable slot [1] at +0xc (ECX = secondary). Calls slot 0x44 of the primary object with the argument; when it returns null, calls 00d3ab40 with the arg… |
| `00d369d0` | RuntimeTerrainSystem::SetByte38 | speculative | Vtable slot [6] of RuntimeTerrainSystem: stores the byte argument at this+0x38 (ret 4). |
| `00d369e0` | RuntimeTerrainSystem::GetByte38 | speculative | Vtable slot [7] of RuntimeTerrainSystem: returns the byte at this+0x38 (4 bytes). |
| `00d365a0` | — | unknown | Vtable slot [8] of RuntimeTerrainSystem with a pointer to four dwords (param_2): copies them to this+0x20..+0x2c. When the bubble pointer at this+0x1c is set i… |
| `00d366d0` | — | unknown | Vtable slot [10] of RuntimeTerrainSystem with one byte argument. Looks up key 0x5c3e6a59 through 007333e0; when found it converts the argument through 00737d30… |
| `00d36840` | — | unknown | Secondary-vtable slot [3] at +4 of RuntimeTerrainSystem with one argument (a file-name object). Compares its name with 'TerrainCalendarDefinitions.txt' through… |
| `00d36cb0` | — | unknown | Vtable slot [0] of RuntimeTerrainSystem, but not a destructor: calls slot 1 of this (ReleaseChildren), looks up key 0x5c3e6a59 through 007333e0 and on a miss w… |

**GameRuntimeTerrain::Tile** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d4a350` | TileCtor | confirmed | Installs the Tile vtable, clears tile fields and flags, initializes its arrays and lists, and sets default state values. |
| `00d4a6f0` | TileDtor | confirmed | Clears tile state, releases owned resources and arrays, removes linked RawGroup and RawArea entries, and resets the Tile vtable. |
| `00d4ba30` | TileScalarDeletingDtor | confirmed | Runs Tile cleanup, conditionally frees the Tile object when flag bit 0 is set, and otherwise returns the same pointer. |
| `00d45ab0` | DeleteOwnedListObjects | likely | Fastcall (ECX = this), no arguments. If the byte at this+0x498 is nonzero, walks the intrusive list from this+0x43c through node+0x60; for each node, calls the… |
| `00d45af0` | ReleaseFlaggedListElements | likely | Fastcall, ECX = the Tile (this). Walks the list at +0x404 (element pointer = node[0]). For each element it calls element->00d43300(); if that returns 0, or if … |
| `00d46f20` | TileVirtualSlot3 | speculative | Thiscall with one stack argument (ret 4) and the Tile as ECX. Tests byte +0x49a with pointer +0x4a0 (pass A) and byte +0x49b with pointer +0x4ac (pass B). With… |

**GameRuntimeTerrain::TileBubble** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d38830` | TileBubble::SetRadius | confirmed | Thiscall with one stack argument (radius): when the radius exceeds 24 it logs to TileBubbleRadius.log and clamps it to 24; when it differs from this+0x14 it st… |
| `00d39220` | TileBubble::Ctor | confirmed | Thiscall with (grid object, radius, centre pointer): installs the TileBubble vtable, stores the grid object at +0x10 and zero at +0x14 (radius field set later)… |
| `00d39310` | TileBubbleScalarDtor | confirmed | Scalar deleting destructor (vtable slot [0] of TileBubble): calls the destructor body 00d38e60 and frees the block through _free when bit 0 of the argument is … |
| `00d38e60` | — | unknown | Destructor body (no free of this): stores the TileBubble vtable at +0, releases the tile object at +0x4c through its slot 0 with flag 1 when set and clears it,… |
| `00d38ef0` | — | unknown | Thiscall on a TileBubble (reached from RuntimeTerrainSystem slot 22 and UpdateTimed with ECX = the bubble at +0x1c): clears the record list at +0x58 when the f… |
| `00d39330` | — | unknown | Thiscall with no arguments: when the flag at +0x31 is zero it formats the requested and received counts (+0x44 and +0x48) into the message buffer at +0x864 thr… |

**SoeUtil::HashList<ApplicationObjectTerrainData,15,-1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010bf190` | HashList_ApplicationObjectTerrainData_Ctor | confirmed | Fastcall constructor body with this in ECX. Stores the HashListMap vtable, zeroes +0x4, +0x8, +0xc, +0x10, clears the 0x3c-byte bucket array at +0x14 with _mem… |
| `010bf210` | HashList_ApplicationObjectTerrainData_ScalarDeletingDtor | confirmed | Thiscall with one flag byte (ret 4). Calls 010bf090 (the map dtor body) on this, frees this through the _free thunk when bit 0 of the flag is set, and returns … |
| `010bdb60` | HashListAllocNode_7c | likely | Virtual slot [1] with no arguments. Same manager-or-CRT pattern as 010bda80, with the fallback CrtAlignedAlloc(0x7c, 4). The 0x7c size matches the node layout … |
| `010bdba0` | HashListFreeNode | likely | Virtual slot [2] taking one pointer. Calls 00401c00 (FreeMemory) with the pointer and alignment 4. |

**SoeUtil::HashListMap<int,ApplicationObjectTerrainData,15,-1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010bd640` | HashListMap_ApplicationObjectTerrainData_Ctor | confirmed | Fastcall constructor body with this in ECX. Stores SoeUtil::HashListMap<int,ApplicationObjectTerrainData,15,-1>::vftable into [this], zeroes +0x4, +0x8, +0xc a… |
| `010bf020` | HashListMap_ApplicationObjectTerrainData_DtorBody | confirmed | Fastcall destructor body with this in ECX. Stores the HashListMap vtable into [this], then removes the head entry at this+8 through 010bef20 while one is prese… |
| `010bf050` | HashListMap_ApplicationObjectTerrainData_ScalarDeletingDtor | confirmed | Thiscall with one flag byte (ret 4). Stores the map vtable into [this], removes each entry through 010bef20 while the head at +8 is present, frees this through… |
| `010bf090` | HashListMap_ApplicationObjectTerrainData_DtorBodyFramed | confirmed | Fastcall destructor body with this in ECX, with an exception-handling frame. Stores the map vtable into [this] and removes each entry at this+8 through 010bef2… |

**SoeUtil::List<GameRuntimeTerrain::RawLight,0>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d49010` | RawLightListInit | confirmed | Fastcall (ECX = this), no arguments. Installs the RawLight list vtable with template argument -1 and then, overwriting it, the one with template argument 0 at … |

**SoeUtil::List<GameRuntimeTerrain::RuntimeObject*,0>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d48550` | RuntimeObjectListInit | confirmed | Fastcall (ECX = this), no arguments. Installs the RuntimeObject-pointer list vtable with template argument -1 and then, overwriting it, the one with template a… |

**TerrainEntityManagerInterface** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d361b0` | TerrainEntityManagerInterface::DtorBody | confirmed | Stores the TerrainEntityManagerInterface vtable at this+0 and returns; no release or free. |
| `00d361c0` | TerrainEntityManagerInterface::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores the TerrainEntityManagerInterface vtable, and when bit 0 of the argument is set calls _free on this; retur… |

**TerrainObjectDeployment** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d37880` | TerrainObjectDeployment::DtorBody | confirmed | Destructor body: stores the TerrainObjectDeployment vtable at +0; clears the list at the head +0x10 of the embedded map at +8 through 00d37550; when the pointe… |
| `00d37cf0` | TerrainObjectDeployment::Ctor | confirmed | Thiscall with one stack argument (param_2, stored at +0x4 and later at +0x101c): installs the AssetDeliveryHandlerInterface vtable at +0 and then the TerrainOb… |
| `00d37d80` | TerrainObjectDeployment::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0] of TerrainObjectDeployment): calls the destructor body 00d37880 and frees the block through _free when bit 0 of the… |
| `00d37da0` | — | unknown | Vtable slot [3] of TerrainObjectDeployment (stack args: text object param_1 with data +4, length +8, capacity +0xc; name object param_2 with string at +4). App… |

**(no class)** (45)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d46110` | FloraPositionInstanceArraysInit | confirmed | Fastcall (ECX = owner). Installs the Array<GameRuntimeTerrain::FloraPosition,0,1> vtable at +8 and zeroes the words at +0xc, +0x10 and +0x14. Installs the Arra… |
| `00d46180` | FloraPositionInstanceArraysDtorBody | confirmed | Fastcall (ECX = owner), no arguments. At +0x20 stores 0, installs the FloraInstance array vtable at +0x18, calls FreeMemory(data at +0x1c, 0x10) and nulls +0x1… |
| `00d477f0` | FloraRecordCopyCtor | confirmed | Thiscall (ECX = destination record, one stack argument = source record), ret 4. Copies source dwords 0 and 1, installs the FloraPosition array vtable at +8 and… |
| `00d48480` | TileIsReadyOrReport | confirmed | Thiscall with a flag byte and an output string pointer (ret 8). Returns 0 with a formatted message in the output when the tile's flags at +0x498 to +0x49b are … |
| `00d4ba50` | TileCreateAll | confirmed | Checks tile texture handles and fallback availability, logs a failure and returns 0 when required data is absent, otherwise calls creation helpers and returns … |
| `010bec70` | ObjectTerrainDataReadXml | confirmed | Thiscall with the map in ECX and one stack argument (a file name). Loads the file text with 00fbf4a0 into an IString and parses it with 00d549d0; returns 0 on … |
| `00d37a60` | LoadTerrainObjectDeploymentGroups | likely | Thiscall with one stack argument (name): assigns the name through 00402ce0; when the pointer at this+0x1020 is null it allocates 0x1238 bytes (TLS/global alloc… |
| `00d38b60` | TileBubbleRegisterTile | likely | Thiscall with (x, z, out count pointer): queries 00d3ff50 with flag 1 for the tile at (x, z) and gets its record in a local. When the record is absent it retur… |
| `00d3aac0` | CaseFoldedNameFind64 | likely | Thiscall: hashes the C string at param_2+4 with HashCStringCaseFolded (00705b00), takes the low 6 bits as a bucket index into the 64 head pointers at this+0xc6… |
| `00d3c050` | IdTable1024FindMatching | likely | Thiscall with one stack arg (key): indexes the 1024-bucket chains at this+0xc7e38 with key & 0x3ff and walks the chain (next link at node+0x1bc) for a node who… |
| `00d3c320` | ChunkListRemoveDestroy | likely | Thiscall (this in ECX, node stack arg; SEH-framed): calls 00d3a040 on the node, unlinks the node from the list through node+0x4e94 (prev) and node+0x4e98 (next… |
| `00d3e010` | List_InvisibleWall_RemoveDestroy | likely | Thiscall (list in ECX, node stack arg): unlinks the node from the list (prev link at node+0x1c, next link at node+0x18; head at +4 and tail at +8 updated), dec… |
| `00d3f6b0` | Pool4ea4ReleaseBlock | likely | Thiscall (pool in ECX, block node as stack arg): walks the free list at pool+8 and unlinks every entry that lies inside the block (between the block's first en… |
| `00d3f710` | Pool4ea4Pop | likely | Fastcall (pool in ECX): when the free-list head at pool+8 is null it grows the pool through 00d3f590 (Pool4ea4GrowBlock); then pops the head entry, decrements … |
| `00d3f780` | Pool4ea4Push | likely | Thiscall (pool in ECX, entry stack arg): links the entry at the head of the free list (entry+0 = old head, head at pool+8), increments the count at pool+0xc; w… |
| `00d46420` | BuildListNodeObjectsOnce | likely | Fastcall (ECX = owner, param_1), no arguments. Runs only when the byte at +0x498 is zero: walks the node list from +0x43c through node+0x60. For each node it a… |
| `00d47340` | FloraPositionInstanceArraysScalarDeletingDtor | likely | Thiscall with one byte argument, ret 4. Calls 00d46180 (the two-array owner destructor body) with ECX = this, then calls _free(this) when bit 0 of the argument… |
| `00d47720` | ArrayDestroySuffix0x28 | likely | Thiscall with one stack argument (count k), ret 4. Sets newSize = size(+8) - k. For each element from newSize up to the old size, with stride 0x28 from data (+… |
| `00d479c0` | ArrayReallocMove0x28 | likely | Thiscall with two stack arguments (count, flag), ret 8. Calls the array's vtable slot 1 with (count, &cap, flag) for new storage. If the new pointer differs fr… |
| `00d47aa0` | TileActivateListElements | likely | Thiscall with one stack argument (limit), ret 4. Runs only while the byte at +0x494 is zero. Constructs a local StringFixed<128> (00402950); when +0x14 is non-… |
| `00d487f0` | ArrayElementResetOrGrow0x28 | likely | Thiscall with one stack argument (index), ret 4. If index is below size (+8) it calls 00d46180 with ECX = the existing 0x28-byte element (its arrays are freed … |
| `00d48ae0` | ArrayElementResetInit0x28 | likely | Thiscall with one stack argument (index), ret 4. Calls 00d487f0(index) with the same ECX (ArrayElementResetOrGrow0x28). If that returns null it returns 0; othe… |
| `00d48b50` | ArrayAppendCopy0x28 | likely | Thiscall with two stack arguments (source records, count), ret 8. Computes newSize = size(+8) + count; if capacity (+0xc) is below newSize it calls 00d479c0(ne… |
| `00d48e70` | ArrayAssignCopy0x28 | likely | Thiscall with one stack argument (source array), ret 4. Destroys all current records of this through 00d47720(size). Then, when the source count (+8) is positi… |
| `00d49cb0` | ArrayDestroySuffix0xB8 | likely | Thiscall with one stack argument (count k), no return value. Sets newSize = size(+8) - k. For each record from newSize to the old size, with stride 0xb8 from d… |
| `00d49d30` | RecordCopyCtor0xB8 | likely | Thiscall (ECX = destination record) with one stack argument (source record). Copies the dword at source+0 to destination+0, then calls 00d48f80 with ECX = dest… |
| `010bde30` | ObjectTerrainDataWriteXml | likely | Fastcall with the object in ECX whose ObjectTerrainData list head is at +0xc (next link +0x6c). Calls 00d51ba0 first, then for each entry writes DataName (+0x8… |
| `00d3aa30` | LazyCreateAt2cAndNotify | speculative | Fastcall with this in ECX: when +0x2c is zero and +0x34 is non-null, allocates 0x10 bytes (operator new 00d833a6); on success stores 00d39600(this) into +0x2c … |
| `00d3bec0` | GridCellLookupField28 | speculative | Thiscall (this in ECX; x and y stack args): snaps x and y down to the cell origin with the cell size at this+0xc4df4, calls 00d38790 on the snapped pair; when … |
| `00d3bf40` | IdTable64Remove | speculative | Thiscall with one stack arg (obj): when obj is non-null and key obj+0x26c is non-zero, walks the 64-bucket chains at this+0xc6fcc (bucket = key & 0x3f) for the… |
| `00d3bfb0` | IdTable64FindThenQuery | speculative | Thiscall with one stack arg (key): indexes the 64-bucket chains at this+0xc6fcc with key & 0x3f, finds the node with +0x10 equal to the key and then the node w… |
| `00d3c020` | ListNodeValueApplyFn | speculative | Fastcall (this in ECX): walks the list whose head is at this+0xc70d4 (next link at node+8); for each node whose value at +4 is non-null, calls 00d433f0 with th… |
| `00d3da00` | TriangleListEmitAllNodes | speculative | Fastcall (this in ECX): calls 00fc6820 with budget -1 (no limit); when the global byte DAT_01bf2a68 is non-zero it walks the list whose head is at this+0xc7e10… |
| `00d3da50` | AgedNodeReleaseThrottle | speculative | Fastcall (this in ECX): increments the counter at this+0xc7e1c; when the counter exceeds the global threshold DAT_01bf2abc and the object at this+0x18 is non-n… |
| `00d3dd60` | FloraDefinitionInitStore40 | speculative | Thiscall (this in ECX, pointer to a dword as stack arg): calls the FloraDefinition constructor 00d3cab0 on this, stores the dword at *arg into this+0x40, and r… |
| `00d3e650` | ListAppendTailAndBucketInsert256 | speculative | Thiscall (container in ECX, node stack arg): clears the node's list-prev-or-next word at +0x4e98, sets its +0x4e94 to the old tail (container+8) and links it a… |
| `00d3e760` | IdTable1024InsertAppend | speculative | Thiscall (container in ECX, node stack arg): stores the old tail at node+0x1b4 with node+0x1b0 zeroed and links the node after the old tail (or as head at cont… |
| `00d47c40` | SubrecordSampleToInstances0x30 | speculative | Fastcall (ECX = owner), no stack arguments. Sets the byte at +0x497 to 1. When the byte at +0x496 is zero and the index at +0x484 is non-negative, it reads chu… |
| `00d39840` | — | unknown | Cdecl-style factory with three arguments (grid object, radius, centre pointer): allocates 0xa80 bytes through 00d833a6; when the allocation is non-null it cons… |
| `00d3c0c0` | — | unknown | Thiscall with one stack arg (obj): when obj is non-null, walks the 512-bucket chain at this+0xc70e0 (bucket = (obj+0x180) & 0x1ff, chain link node+0x14) for th… |
| `00d3c510` | — | unknown | Fastcall (this in ECX): allocates a node through the container's vtable slot [2] (no arguments; the /GS cookie in the Ghidra output is not an argument); on suc… |
| `00d3da40` | — | unknown | Thunk (no stack args): adds 0xc to ECX and tail-jumps to 00d3c320 (ChunkListRemoveDestroy), so the target sees this+0xc as its object pointer. |
| `00d3f590` | — | unknown | Thiscall (pool in ECX, requested count as stack arg): computes the block size as the requested count clamped to between 2 and 13 and to the value returned by T… |
| `00d46640` | — | unknown | Thiscall with three stack arguments (ret 0xc). Walks the triangles of a chunk: three 16-bit indices per triangle, each indexing 0x20-byte vertex records (posit… |
| `00d48c00` | — | unknown | Fastcall wrapper with an EH frame: ECX = array, passed unchanged to 00d488e0 (InlineArrayDtor); plain ret. |

## Source shards

[[S0220 008a3ce0-008a8500|S0220]] (2), [[S0221 008a86b0-008b1d80|S0221]] (1), [[S0227 008d0920-008d5c60|S0227]] (4), [[S0276 009cdf80-009d3fe0|S0276]] (5), [[S0334 00b0a370-00b10840|S0334]] (38), [[S0360 00b94bf0-00b9a300|S0360]] (21), [[S0376 00be5b30-00bea020|S0376]] (25), [[S0377|S0377]] (42), [[S0378 00becaa0-00bf1530|S0378]] (22), [[S0422 00ce4e30-00ceb920|S0422]] (15), [[S0439|S0439]] (52), [[S0440|S0440]] (40), [[S0442|S0442]] (23), [[S0443 00d49d90-00d4f150|S0443]] (8), [[S0542|S0542]] (2), [[S0567|S0567]] (10), [[S0592|S0592]] (13), [[S0658 0124dfe0-01253ee0|S0658]] (13), [[S0660|S0660]] (1), [[S0689|S0689]] (3), [[S0697|S0697]] (1), [[S0723|S0723]] (1), [[S0739|S0739]] (3), [[S0764|S0764]] (19), [[S0790|S0790]] (3)
