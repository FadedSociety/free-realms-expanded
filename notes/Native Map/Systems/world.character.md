# System: world.character

95 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (38)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00963840` | `QueuePositionAndDirectionUpdate` |  | The movement reader and research notes identify this as the MovementType 2 target-position enqueue path; queue consumption is handled elsewhere. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964b00` | `ProxiedCharacterAttachmentGroup::Destructor` | ProxiedCharacterAttachmentGroup | Cleanup body for ProxiedCharacterAttachmentGroup. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964b70` | `ProxiedCharacterAttachmentGroup::GetField174` | ProxiedCharacterAttachmentGroup | Shared implementation also occupies GuiModelAttachmentGroup vtable slot 23. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964b80` | `ProxiedCharacterAttachmentGroup::ScalarDeletingDestructor` | ProxiedCharacterAttachmentGroup | Scalar-deleting destructor for ProxiedCharacterAttachmentGroup. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964ba0` | `PlayerSwitchProfileAttachmentGroup::Constructor` | PlayerSwitchProfileAttachmentGroup | Constructs the profile-switch attachment-group object over the shared attachment-group base. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964c50` | `PlayerSwitchProfileAttachmentGroup::GetAdjustedField174` | PlayerSwitchProfileAttachmentGroup | Virtual slot 24 exposes the field with the profile-group base adjustment removed. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964c70` | `PlayerSwitchProfileAttachmentGroup::UpdateBaseAndNotify` | PlayerSwitchProfileAttachmentGroup | Virtual slot 1 performs a base update followed by a conditionally gated callback. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964ca0` | `PlayerSwitchProfileAttachmentGroup::NotifyWhenReady` | PlayerSwitchProfileAttachmentGroup | Virtual slot 18 is a condition-gated callback. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964cc0` | `PlayerSwitchProfileAttachmentGroup::FilterField174` | PlayerSwitchProfileAttachmentGroup | Virtual slot 23 filters the stored field value. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964d50` | `PlayerSwitchProfileAttachmentGroup::Destructor` | PlayerSwitchProfileAttachmentGroup | Destructor body for the profile-switch attachment group. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964da0` | `PlayerSwitchProfileAttachmentGroup::ScalarDeletingDestructor` | PlayerSwitchProfileAttachmentGroup | Scalar-deleting destructor for PlayerSwitchProfileAttachmentGroup. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `0096fa70` | `ProxiedCharacter::Ctor` | ProxiedCharacter | Constructs a ProxiedCharacter actor and initializes its owned state and helper collections. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `00970770` | `ProxiedCharacter::GetField200 → ProxiedCharacter::SubobjectAtC8` | ProxiedCharacter | Provides a pointer to an interior ProxiedCharacter field through virtual slot 10. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `00970780` | `ProxiedCharacter::SetField128` | ProxiedCharacter | Sets a ProxiedCharacter field through virtual slot 25. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `00970790` | `ProxiedCharacter::Field200AdjustorThunk` | ProxiedCharacter | Adapts a secondary ProxiedCharacter base pointer for the slot-3 getter. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `009707a0` | `ProxiedCharacter::ScalarDeletingDtorAdjustorThunk50` | ProxiedCharacter | Adjusts the deleting-destructor this pointer for a secondary ProxiedCharacter subobject. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `009707b0` | `ProxiedCharacter::ScalarDeletingDtorAdjustorThunk54` | ProxiedCharacter | Adjusts the deleting-destructor this pointer for another secondary ProxiedCharacter subobject. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `009707c0` | `ProxiedCharacter::IsFlagBit7ClearAtEC` | ProxiedCharacter | Reports whether bit 7 of the ProxiedCharacter byte at offset 0xec is clear. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `009707d0` | `ProxiedCharacter::GetFlagBit0At144` | ProxiedCharacter | Exposes bit 0 of a ProxiedCharacter flag byte through virtual slot 27. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `009707e0` | `ProxiedCharacter::AdjustorThunkTo00961550` | ProxiedCharacter | Adapts a secondary ProxiedCharacter base pointer before forwarding the virtual call. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `009707f0` | `ProxiedCharacter::SetFlagBit0At144` | ProxiedCharacter | Updates the same ProxiedCharacter flag bit exposed by virtual slot 27. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `00970810` | `ProxiedCharacter::CreateStringFromField56C` | ProxiedCharacter | Constructs a string result from a ProxiedCharacter field. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `00970880` | `ProxiedCharacter::CreateStringFromField58C` | ProxiedCharacter | Constructs a string result from another ProxiedCharacter field. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `009710b0` | `RemoveRider` |  | Removes a rider relationship. The teardown call site is ProxiedCharacter destruction, but this function itself has no class vtable evidence. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `00972570` | `ProxiedCharacter::Dtor` | ProxiedCharacter | Runs ProxiedCharacter teardown and clears an outstanding rider relationship. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `00972d30` | `Mount` |  | Mounts a rider onto a seat-bearing actor. The exact operation is named by its own log string; this function does not establish a class vtable. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `00b035c0` | `ControllerPreviewPet::ControllerPreviewPet` | ControllerPreviewPet | Constructs the controller that previews a pet or mount model, seeding the vector at +0x164..+0x16c that FUN_00b03550 protects. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b036b0` | `ControllerPreviewPet::ScalarDeletingDtor` | ControllerPreviewPet | Deletes ControllerPreviewPet instances; ControllerRoot::vftable[7] (FUN_009510b0) carries the identical deleting-destructor body, which is how the slot was identified. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00bf0f70` | `ProxiedCharacterAttachmentGroupManager::IsEmpty` | ProxiedCharacterAttachmentGroupManager | Tests whether this attachment group manager's field at offset 0x118 is empty. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf1320` | `ProxiedCharacterAttachmentGroupManager::UpdateVisibility` | ProxiedCharacterAttachmentGroupManager | Propagates manager and group state to attachment visibility values. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf1420` | `ProxiedCharacterAttachmentGroupManager::Ctor` | ProxiedCharacterAttachmentGroupManager | Constructs an empty proxied-character attachment group manager. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf14c0` | `ProxiedCharacterAttachmentGroupManager::Dtor` | ProxiedCharacterAttachmentGroupManager | Destroys the attachment group manager and its owned list. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf1530` | `ProxiedCharacterAttachmentGroupManager::ProcessGroup` | ProxiedCharacterAttachmentGroupManager | Processes character attachment group state, including dual-wield slot validation and activation. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `010444a0` | `CharacterBase::EntryTagName` | CharacterBase | Virtual that yields the string key "entryTag"; the consumer of the key is not in this function. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `010444f0` | `CharacterBase::EntryTagName2` | CharacterBase | Second vtable slot returning the same "entryTag" key; shares the implementation of 010444a0. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01044500` | `CharacterBase::AdjustorThunk_016c` | CharacterBase | This-adjusting thunk for CharacterBase virtual slot [10]; forwards to the base implementation at 0101c8a0. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `010449e0` | `CharacterBase::DtorBody` | CharacterBase | Destructor body for CharacterBase; called by the scalar deleting destructor 01044b90. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01044b90` | `CharacterBase::ScalarDeletingDtor` | CharacterBase | Scalar deleting destructor of CharacterBase (vtable slot 0). | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |

## 🟢 likely (31)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0089f3f0` | `TestBit0x80_6f8` |  | State predicate read by 00973200 (ProxiedCharacter::Process, whose strings cover terrain-object actor lookup after zone load), so the bit belongs to a character/actor state word; the bit's meaning is not recovered. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `0089f4c0` | `TestBit0x8000_6f8` |  | Second state predicate over the same character state dword as 0089f3f0, read by 00973200 (ProxiedCharacter::Process); the bit's meaning is not recovered. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `0089f4d0` | `TestBit0x01_6fa` |  | Third state predicate over the character state flags next to the +0x6f8 dword, read by 00973200 (ProxiedCharacter::Process); the bit's meaning is not recovered. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `009580f0` | `ProxiedCharacter::LogAttachmentGroupError` | ProxiedCharacter | ProxiedCharacter virtual slot 5 logs the supplied attachment-group error value to the named log file. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00961630` | `IsProfileContextCurrent` |  | Called by ProxiedCharacter::OnProfileSwapComplete; that caller context suggests a profile-state predicate, but the exact contract is unresolved. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00962f30` | `SetCharacterStateFlags` |  | Synchronizes two flag bits and a stored value, then invokes a callback when the related object exists. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00962fb0` | `ClearOwnedCharacterBuffers` |  | Releases and clears three owned pointers; the first cleanup also invokes the destination-list helper. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `009630e0` | `ProcessQueuedTriples` |  | Consumes queued three-dword records through a helper and resets the queue count. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00963790` | `SetCharacterFlagAndNotifyModel` |  | Propagates a changed state bit to the related model when that model is available. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00963ef0` | `ReleaseHandleAndSetModelFlag` |  | Combines optional-handle release with a model-flag update. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00963fa0` | `SetFlagAndRefreshRelatedObject` |  | Synchronizes a state bit and invokes a helper on the resolved related object. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964ce0` | `PlayerSwitchProfileAttachmentGroup::CanApplyProfileChange ⚠downgrade` | PlayerSwitchProfileAttachmentGroup | Virtual slot 14 is a profile-change eligibility predicate over context and node state. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `0096bd90` | `ProxiedCharacter::SwitchToTemporaryAppearance` | ProxiedCharacter | Switches a ProxiedCharacter to a temporary appearance and logs an error when no actor remains after the previous temporary appearance is terminated. | [[Native Map/Shards/S0258 00969a10-0096de80|S0258]] |
| `0096f800` | `ProcessCharacterAttachmentData` |  | Processes temporary attachment data. An attachment refresh/update interpretation is likely from the typed list and helper flow, but its exact semantics are not named directly. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `00971250` | `UpdateSeatAttachment ⚠downgrade` |  | Updates a seat attachment. The error-log string identifies ProxiedCharacter context, but no class vtable is installed in this function. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `00b006d0` | `CapsuleSweepMove3Iters` |  | Movement step of the regular character controller: the horizontal displacement is projected onto the up axis, a step height is added so the character can climb low ledges, and the residual slide factor of the collision … | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b01040` | `MoveInSubStepsAndRecord` |  | Sub-stepped movement used by FUN_00b01250: it splits one frame displacement into collision-resolved positions and collects them in the caller's four-slot Vector4 array. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03550` | `ControllerPreviewPet::PreserveVec3AndCallBase ⚠downgrade` | ControllerPreviewPet | Keeps the pet preview's own orientation/position vector stable across the shared controller update; the base slot-53 body FUN_0094d3b0 is shared by the camera, garage, inspect and pan/zoom controllers. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00bf11e0` | `ProxiedCharacterAttachmentGroupManager::RemoveGroup ⚠downgrade` | ProxiedCharacterAttachmentGroupManager | Removes a character attachment group from the manager while applying associated cleanup. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf1290` | `ProxiedCharacterAttachmentGroupManager::RemoveGroupsForId ⚠downgrade` | ProxiedCharacterAttachmentGroupManager | Removes associated attachment groups for a character or group identifier. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `01044480` | `CharacterBase::FieldAddr_05ec` | CharacterBase | Address accessor for CharacterBase field +0x5ec; the game meaning of the field is not established by this function. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01044490` | `CharacterBase::SetFloat_05ec` | CharacterBase | Float setter for CharacterBase field +0x5ec; game meaning of the field is not established by this function. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `010444b0` | `CharacterBase::FieldAddr_05f0` | CharacterBase | Address accessor for CharacterBase field +0x5f0; the game meaning of the field is not established by this function. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `010444c0` | `CharacterBase::SetFloatFromPtr_05f0` | CharacterBase | Float setter for CharacterBase field +0x5f0; game meaning of the field is not established by this function. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `010444d0` | `CharacterBase::FieldAddr_05f4` | CharacterBase | Address accessor for CharacterBase field +0x5f4; the game meaning of the field is not established by this function. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `010444e0` | `CharacterBase::SetFloat_05f4` | CharacterBase | Float setter for CharacterBase field +0x5f4; game meaning of the field is not established by this function. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01044510` | `CharacterBase::FieldAddr_016c` | CharacterBase | Address accessor for CharacterBase field +0x16c; the game meaning of the field is not established by this function. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01044520` | `CharacterBase::SetDwordPair_0010_0014` | CharacterBase | Stores a two-dword value at CharacterBase +0x10/+0x14. Function 01044560 formats the same pair as a 64-bit owner id (%llu) in its Pets.log message, so the pair is probably an owner id; that link is cross-function and no… | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01044540` | `CharacterBase::GetDword_0114` | CharacterBase | Getter for CharacterBase field +0x114; the game meaning of the field is not established by this function. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01044710` | `CharacterBase::FormatCounterPlus1Into34` | CharacterBase | Writes the decimal form of a counter plus one into a string field of an object found through the virtual lookup in slot 51; the object type and counter source are not established here. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01044760` | `CharacterBase::FormatCounterMinus1Into34` | CharacterBase | Writes the decimal form of a counter minus one into a string field of an object found through the virtual lookup in slot 51; the object type and counter source are not established here. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |

## 🟡 speculative (17)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009580d0` | `ProxiedCharacter::DestructorAdjustor ? ⚠rename` | ProxiedCharacter | Adds 0x154 to ECX and jumps to 00bf1ee0; decoded bytes show this is an adjustor thunk, not the destructor body. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00958490` | `ProxiedCharacter::Virtual29 ?` | ProxiedCharacter | Calls vtable slot 0x70 with param_2 and destination this+0x610. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00958630` | `ProxiedCharacter::MarkFlagAt100 ?` | ProxiedCharacter | Stores byte one at offset 0x100 of the object referenced by this+8. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00959ab0` | `ProxiedCharacter::GetPair24 ?` | ProxiedCharacter | Copies two dwords from this+0x24 and this+0x28 to the output pair. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `0095a120` | `ProxiedCharacter::HasActiveEffect ? ⚠rename` | ProxiedCharacter | Walks an effect list and returns false if any active node's field +0x20 satisfies 0072a1c0; returns true when none do. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `0095a170` | `ProxiedCharacter::SetEffectsEnabled ?` | ProxiedCharacter | Walks active effect nodes and calls 0072ae80 on each node's field +0x20 with the inverse of param_2 as its second argument. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `0095a3c0` | `ProxiedCharacter::SetContext504 ? ⚠rename` | ProxiedCharacter | Calls the object at this+0x734 virtual slot 4 with this+0x504 as its argument. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00960060` | `MoveDestinationNodesToArray ?` |  | Walks nodes from this+0x74c, appends each node's three selected words to a 0xc-byte array, removes each node through FUN_0095e740, and advances using its saved next link. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `00961550` | `ProxiedCharacter::DispatchCurrentAction ? ⚠rename` | ProxiedCharacter | Ordinary ProxiedCharacter virtual at vtable slot 0; its action domain is not established by this body. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00963130` | `ClearCharacterStateAndReleaseObjects ? ⚠downgrade` |  | The body performs a broad character-state reset; specific effects of the indirect callbacks remain unknown. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00963d80` | `SetByteFlag0x140Bit0 ?` |  | Sets bit 0 at this+0x140 when param_2 is nonzero; otherwise clears that bit. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00963da0` | `SetAttachmentStateFlags ?` |  | Sets bit 6 at byte +0x145 when param_2 is nonzero; otherwise clears that bit and sets bit 0 at +0x140. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00963f20` | `SetCharacterBitAndPropagateByte ?` |  | When bit 6 at byte +0x147 differs from param_2, updates that bit, obtains a related object through the virtual method, and writes param_2 to its byte at +0x226 when present. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `0096aaf0` | `ProxiedCharacter::SetRecordValue_v30 ?` | ProxiedCharacter | Virtual slot 30 of ProxiedCharacter::vftable. For index 0..17 it finds or creates the record as in 0096aa00, releases record+0x1d0 through 00969910 when set, and stores param_2 at record+0x1cc. | [[Native Map/Shards/S0258 00969a10-0096de80|S0258]] |
| `0096c300` | `SwitchTemporaryAppearanceFromRecord ?` |  | Fastcall(ECX=this). When this[0x3ca] is positive and the vtable+0x24 object has bit 1 set in byte +0x10, it constructs a PlayerUpdatePacketUpdateTemporaryAppearance object (008b4370), copies two dwords from the object r… | [[Native Map/Shards/S0258 00969a10-0096de80|S0258]] |
| `00bf7b00` | `ProxiedCharacterInitFields ?` |  | Initializes a ProxiedCharacter-sized object: zeroes most fields, stores the constants DAT_018277ec, DAT_01b676bc, DAT_017ed82c, _DAT_0175b420, DAT_01809b68, _DAT_0181c234 and DAT_01827804 into fields +0x2c, +0x38, +0x4c… | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |
| `01044550` | `CharacterBase::ReadDword_StackArg_0090 ?` | CharacterBase | One stack arg, ret 4. Bytes 8b 44 24 04 8b 80 90 00 00 00 c2 04 00 = mov eax,[esp+4]; mov eax,[eax+0x90]; ret 4. Returns the dword at arg+0x90 and does not read ECX. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |

## ⚪ unknown (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0095a250` | `— ⚠reject` | ProxiedCharacter | Queries a name/string helper, resolves an object using this+0xb4, invokes a virtual callback with a resolved field, then conditionally calls a second interface callback. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `0095e890` | `— ⚠reject` |  | Enqueues one destination record for later character movement processing. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `00960480` | `— ⚠reject` | ProxiedCharacter | Updates character state and clears pending list data. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `00960520` | `— ⚠reject` | ProxiedCharacter | Applies position and direction state to a proxied character, with extra local-owner handling. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `00b00ac0` | `— ⚠reject` | ControllerRegular | Builds the regular on-foot character controller and caches its movement tuning (walk/run speeds, step heights, air control) in the file-global slots that FUN_00b006d0 and FUN_00b01250 read every frame. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b01250` | `— ⚠reject` | ControllerRegular | Movement heart of the regular on-foot controller: turns the input state into a collision-resolved per-frame position and basis for the local player. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b02880` | `— ⚠reject` | ControllerRegular | Native input entry point of the on-foot character controller: hotkeys, action-bar clicks, tab targeting and the movement axes all arrive here. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00bf0f80` | `— ⚠reject` |  | Removes attachment groups whose group identifiers match entries in the global list. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `01044650` | `— ⚠reject` | CharacterBase | Thiscall with seven stack params (Ghidra). Calls 00703540, then formats param_3 with the format string at 01770978 (bytes 25 64 00 = "%d") through 00402fd0 into a stack string buffer. Then calls the virtual at this vtab… | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |

