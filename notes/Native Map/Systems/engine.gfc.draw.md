# System: engine.gfc.draw

137 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00476180` | `DrawStateCtor` |  | Slot-state object created by 00476691/0047673e and destroyed by 00477877. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476505` | `InterfaceSubCtor` |  | Second constructor emission of the sub-object family whose default constructor is 004765cc. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `0047658c` | `InterfaceSubDtor` |  | Base cleanup of the sub-object family; reached from its deleting destructor 004765b0 and the factory 00476c26. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004765cc` | `WindowCursorHelperCtor` |  | Cursor/window helper embedded in the D3D9 device and page objects; created by 00476a23, 00476b89 and 00476c26. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476b89` | `D3D9ModuleCtor` |  | Holds the d3d9.dll module and the IDirect3D9 factory for the render device family; the factory 00476d95 allocates 0x34 bytes and calls it. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00477877` | `DrawStateDtor` |  | Counterpart of the draw-state constructor 00476180; reached from its deleting destructor 00477D2C. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00824dc0` | `CreateDependentDisplay` |  | Creates the dependent D3D9 draw display object for the client and applies the video mode. | [[Native Map/Shards/S0197 00821940-00827640|S0197]] |

## 🟢 likely (93)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00459393` | `SetStaticTag01af5e4c` |  | Initialiser emission shared by several drawing classes (callers 0045f90f, 004662e0, 00467368, 00474818). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045942b` | `GlobalListInsertHead` |  | Registration of draw objects in the global list headed by DAT_01bc45c4 (callers 0045f753, 004662e0). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `00459448` | `GlobalListRemove` |  | Deregistration counterpart of 0045942b, called from teardown paths (callers 00466279, 004662e0). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a02c` | `PackedRecordArrayDtor` |  | Bulk destruction of queued variant records (counterpart of the record constructor 0045a349). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a0d4` | `RectStoreA` |  | Rect accessor/writer used by the texture encode routines (callers 00465480, 004654c3). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a107` | `RectStoreB` |  | Second rect accessor emission (callers 004654c3, 0046992d, 00469cab). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a147` | `RectFormatConvert` |  | Legacy rect-format conversion used when serialising UI quads (caller 00465699). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a2ad` | `RecordQueueFull` |  | Capacity probe of the 0x98-byte command record before it is pushed (0045a3c2). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a349` | `CommandRecordCtor` |  | Construction/copy of the record that 0045a3c2 snapshots into the shared block. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a3c2` | `CommandRecordFlush` |  | Publishes a queued command record to the shared block queue used by the draw device. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a478` | `QueuePushZero` |  | Pushes a terminator/zero command into the block queue. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a666` | `EnsureBlockReady` |  | Block/queue preparation used by the twelve texture-format writers at 004689c4..00469cab. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a72e` | `InitNode24Buffers` |  | Node initialiser of the draw device's +0xC chain; the accepted DrawDevicePopNode20 (00466c39) calls this on a freshly allocated 0x24-byte node. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a78d` | `FreeNode24Buffers` |  | Node buffer release used by the accepted DrawDeviceListsDestroy (00466b28) and DrawDevicePopNode20 (00466c39). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a7ad` | `BufferAppendByte` |  | Byte writer used by the ten texture-format encode routines at 004652aa..00465699. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a804` | `Node24ScalarDtor` |  | MSVC scalar deleting destructor of the draw-device node initialised by 0045a72e. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a932` | `ReleaseDeviceHandles` |  | Interface cleanup of the draw device; run by the accepted destructor body 00462140 (engine.gfc.core) and by the frame paths 004621ca/00466eb3 (engine.gfc.draw). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a964` | `ZeroDeviceFields90` |  | Field reset run by the accepted DrawDeviceCtor (00466d6e). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a976` | `FitPoolSlot` |  | Slot fitting used by the accepted texture allocator RecordAllocTexture (004627ff). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045aa50` | `StateListCleanupA` |  | First member cleanup run by the accepted draw-device destructor body 00462140 (DtorVtables17649E8). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045aa68` | `StateListCleanupB` |  | Second member cleanup run by the accepted draw-device destructor body 00462140 and by the state initialiser 0045aa8d. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045aa8d` | `InitStateRecordList` |  | Initialises the draw device's record list; the accepted DrawDeviceBeginFrame (00466eb3) calls this as the state initialiser. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045ad06` | `MapFormatId` |  | Format id translation used by the accepted ConfigureStageStatesC (00467075). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045adaa` | `ApplyDeviceStateTables` |  | State-table application used by the accepted ApplyInitialRenderState (004624ac) and the ConfigureStageStates paths. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045ae5f` | `ApplyStateRecord` |  | Second form of the draw-state application (caller 00476d2d). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045af01` | `InitStageRecord` |  | State-cache configuration used by the accepted ConfigureStageStatesA/B/C (00466ef3, 00466fb9, 00467075). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045af64` | `SetStageMode` |  | Stage-mode setter; the accepted ConfigureStageStatesDefault (00466f8b) calls this with 0 to clear the textured stage mode. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045afa4` | `SetBoundTextureRect` |  | Recomputes the bound record's quad; the record constructors RecordCtor1764A68 (00462d00) and RecordCtor1764A88 (00462de3) call it. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045afcc` | `IsTextureCurrent` |  | Tests whether this texture object is the active one (callers 0045b071, 0045b139, 004627ff, 00462a02). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b04d` | `BaseTeardown1764A18` |  | Base teardown run by the record destructor bodies (accepted DtorVtable1764A18 00462910 calls this first). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b132` | `ResetVtableToPurecallBase` |  | Destructor body fragment of the record class whose scalar deleting destructor is 0046295d, which runs this and then frees the object. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b21a` | `BaseTeardown1764A48` |  | Base teardown run by the record destructor bodies (accepted DtorVtable1764A48 004629d3 calls this first). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b23e` | `CopyOrKeyPixels` |  | Row copy / colour-key pixel upload used by the draw device (caller 00462a02). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b2cf` | `DtorBody1764670` |  | Destructor body run by the accepted scalar deleting destructor 00462c72 of the 0x1764670 record. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b2fa` | `BaseTeardown1764A68` |  | Base teardown run by the accepted destructor of the 0x1764a68 record (00462d6a). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b316` | `BaseTeardown1764A88` |  | Base teardown run by the accepted destructor of the 0x1764a88 record (00462e52). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b32c` | `EnsureField18` |  | Lazy creation of the texture object at +0x18 with a capability flag (caller 00462de3). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b395` | `DtorBody1764688` |  | Destructor body run by the accepted scalar deleting destructor 00462ed2 of the 0x1764688 record. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b3d3` | `ReleaseField14` |  | Field cleanup emitted for a texture-owning object (no callers recorded in the export). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b564` | `ValidateChildren` |  | Validation pass over a draw object's children (caller 0046de45). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b5c4` | `ReleaseChildren` |  | Releases a draw object's children (caller 00463ef4). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0046b77b` | `AddDrawRecords28Clip` |  | Queues one clipped batch of 28-byte draw records (the flavour encoded by FUN_0046556b/FUN_004655ae) and marks the affected rectangle dirty on the target object. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046bd72` | `AddDrawRecords36Clip` |  | Virtual method: queues one clipped batch of 36-byte draw records. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046be93` | `AddDrawRecord36ClipCached` |  | Virtual method (vtable slot 0x017648fc): 36-byte record append reusing the object cached clip node. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046c86d` | `DrawSurfaceScalarDeletingDtor` |  | MSVC scalar deleting destructor of the draw-surface class; it occupies the deleting-destructor slot at 0x01764d40, the same vtable the DrawSurface constructors install. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046c8cc` | `AddDrawRecords28ClipEntry` |  | Vtable slot 0x017648d4: five-argument entry of the 28-byte batch append. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046c8ec` | `AddDrawRecord28ClipCachedEntry` |  | Vtable slot 0x017648d8: entry of the cached-clip 28-byte append. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046c90c` | `AddDrawRecord28ClipParamEntry` |  | Vtable slot 0x017648dc: entry of the parameter-clip 28-byte append. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046c92c` | `AddDrawRecord36ClipEntry` |  | Vtable slot 0x017648f4: entry of the 36-byte single-record append. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046c94c` | `AddDrawRecords36ClipEntry` |  | Vtable slot 0x017648f8: entry of the 36-byte batch append. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046c96c` | `AddDrawRecord36ClipCachedEntry` |  | Vtable slot 0x017648fc: entry of the cached-clip 36-byte append. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046c98c` | `AddDrawRecord36ClipParamEntry` |  | Vtable slot 0x01764900: entry of the parameter-clip 36-byte append. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046fa3f` | `DrainQueuedObjectLists` |  | Drains the two object lists during DrawSurface teardown/suspend; the caller context identifies the draw-surface path. | [[Native Map/Shards/S0023 0046f261-00472f3a|S0023]] |
| `00472fd2` | `DeviceMemberArrayCopy` |  | Member-array duplication used by the copy constructor of the 0x34-byte D3D9 module object (caller 00474792), which copy-constructs its +0x10 member through this helper. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `0047305c` | `MemberObjectCopyCtor` |  | Used twice by the render-surface copy constructor 00475ac1 (DrawSurfaceCopyCtor) to duplicate the two 12-byte members at +0x28 and +0x34. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00473112` | `DeviceMemberArrayCopy2` |  | Duplicates the +0x4C member of the render surface in its copy constructor 00475ac1. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004731b8` | `DeviceMemberArrayCopy3` |  | Duplicates the +0xA0 member inside the 0xCC-byte draw-device copy constructor 0047441a. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004740cf` | `DrawQueueNodeInit` |  | Creates a per-object draw node and links it into the owner's node list; the sole caller is the per-slot resolution 004750e3 (DrawSurfaceGetOrCreateState). | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004743fe` | `ScalarDeletingDtorA` |  | Deleting destructor for one GDraw class whose destructor body is 00472a34 (no callers recorded in the export). | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `0047441a` | `DrawDeviceCopyCtor` |  | State duplication for the 0xCC-byte draw device; the accepted DrawDeviceFactory 00479d94 constructs it through this path (selectors 1/3). | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `0047489a` | `MemberObjectCopyCtor2` |  | Duplicates a page/state member; called by 004760a4 for the source member at +4. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00474b7c` | `MemberObjectCopyCtor3` |  | Duplicates the +0x34 member in the page/state copy constructor 00476934. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00474c9e` | `MemberObjectCopyCtor4` |  | Duplicates the +0x50 member in the page/state copy constructor 00476934. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00474dd7` | `MemberObjectCopyCtor5` |  | Duplicates the +0x10 member inside the 0x1C-byte sub-object constructor 00476505. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004752ce` | `AdvanceSegmentCursor` |  | Segment builder used by the accepted draw-record execute/replay paths (004781b0 DrawQueueExecuteRecord, 00479c56 DrawQueueReplayTable) through the thunk 00476081. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00475352` | `DrawQueueWriteChunk` |  | Blob/string-chunk writer of the 0xC4-byte draw queue (the accepted DrawQueueReset 00477d48 reclaims exactly this chunk vector); 004778b3 stores each flushed item's payload location through it. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00475902` | `CreateDrawDeviceForFormat` |  | Per-format creation path of the 0xCC-byte draw-device class; no callers are recorded, so the name rests on the constructor it calls and the accepted DrawDevice family. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00475aa2` | `TwoVtableObjectCtor34` |  | Second 0x34-byte object family; its factory is 00476606. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00475ac1` | `DrawSurfaceCopyCtor` | GDrawSurface | Full duplication of a render surface; the accepted DrawSurfaceFactory 0047afe1 selects it for selectors 1/3. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476081` | `AdjustorThunk_004752ce` |  | The record-writing callers 004781b0 and 00479c56 reach the segment append helper 004752ce through this +0x90 receiver adjustment. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004760a4` | `MemberObjectCopyCtor6` |  | Copy-construction of one member, used by the page-object copy constructor 00476934 - the code constructs, it does not assign. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004760bf` | `ObjectHeaderInit` |  | Header/constructor step of a container-holding GDraw object; its caller 00477c01 is part of the device family. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `0047624a` | `DrawQueueCtor` |  | The DrawQueue family label is supported by the matching vtable pair used by destructor 00479c05 and callers 00476387/0047676c; this constructor itself has no RTTI class string. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476348` | `PageObjectScalarDeletingDtor` |  | Deleting destructor of the page object whose body 0047543e is also invoked by the shared-object release 004769ef; no callers are recorded in the export. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004765b0` | `InterfaceSubScalarDeletingDtor` |  | Deleting destructor used by the 0x1C-byte sub-object factory 00476c26 for selector 5. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476606` | `TwoVtableObjectFactory` |  | Second member of the class-factory family (compare the device factory 00476d95). | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476691` | `CreateDrawState` |  | Creates one per-pass draw-state record for the render object. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `0047673e` | `CreateDrawStateChecked` |  | Call-site convenience wrapper for the draw-state creation path. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476877` | `QueryAllElements` |  | Element-wise query helper of the draw-device family (no callers recorded in this export). | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476934` | `PageObjectCopyCtor` |  | Copy constructor for the 0xA4-byte page object; its caller 00477b31 is the accepted D3D9FrameBufferCopyCtor, so the page objects belong to the frame-buffer (display-state) family. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476982` | `CreatePageObject` |  | Creates the state object used by the state switch 00476d2d, which guards the call on its +4 field. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004769ef` | `PageObjectRelease` |  | Teardown step used by the shared-pointer release 00476cc1. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476a23` | `GetOrLoadCursor` |  | Lazy cursor loading for the window/display layer; used when the client needs a cursor that is not in the standard set. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476c26` | `InterfaceSubFactory` |  | Factory used by the D3D9 device/page code to create or copy the cursor-helper sub-object. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476cc1` | `SharedPtrRelease` |  | Release primitive behind the shared-pointer setters 00476e20/00476e55/00476e70 of the D3D9 device family. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476d95` | `GDisplayD3D9ImplFactory` | GDisplayD3D9Impl | The type identity follows the direct constructor, copy-constructor and destructor callees; the dispatcher itself has no RTTI string. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476e20` | `SharedPtrAssign` |  | Shared-pointer assignment with AddRef; the callers are the accepted DrawQueueAppendRecord 00477e05 and DrawQueueCompile 0047990d. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476e55` | `SharedPtrSet` |  | Plain setter used by the create-and-store helper 00476e88. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476e70` | `SharedPtrClear` |  | Used by the draw-context teardown 00476fe6 and the per-mode resource reset 004771b2. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476e88` | `SharedPtrCreate` |  | Creates the +0x1D4 mode resource of the render surface when it is first needed (guarded by 004772fc). | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476eaf` | `RecordPayloadClear` |  | Payload release/rebind helper used when record arrays are torn down (00476f03) and grown (00476f76). | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004771b2` | `DispatchModeSlotOperations` |  | Selects mode-specific Fx helper and pointer-slot operations. Its render-surface association is inferred from the shared offsets/helper family; no callers are recorded. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004772fc` | `DrawSurfaceEnsureModeResource` |  | Guarded lazy creation of the +0x1D4 mode resource, called by the accepted surface emitters 0047b08e/0047b10f/0047b2ab/0047b332/0047b3ce/0047b44c. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |

## 🟡 speculative (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0045ace1` | `GetRow1af6890 ?` |  | Row lookup over a static 24-entry table used by the stage-record writer 0045af01; the row values are small argument values (1/2) and nothing here or in the accepted ConfigureStageStatesA/B/C labels establishes a hardwar… | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045afdd` | `ReleaseTextureRef ?` |  | Reference release for a draw-device texture record; the texture reading is carried by neighbouring accepted records (RecordAllocTexture 004627ff, AcquireTextureLock 00462dad), not by this function's own code. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0046c7d8` | `DrawSurfaceOperatorNew ? ⚠rename` |  | Class-specific new/delete and placement dispatch for the 0x84-byte draw-surface object whose constructors are FUN_00465e11 (DrawSurfaceCtorA) and FUN_004661c4 (DrawSurfaceCtorC), both of which install vtables 0x1764d40/… | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046c8ac` | `AddQuadClipWithClipObjectEntry ?` |  | Five-argument forwarding entry to FUN_0046b665; the callee operation supports the wrapper role, while no owning class is established here. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `00474792` | `D3D9ModuleCopyCtor ? ⚠downgrade` |  | Duplication path of the D3D9 module factory 00476d95 (selectors 1/3). | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00474818` | `IterateLiveReferencedElements ?` |  | Guarded iteration that calls the referenced object's helper only when both virtual checks pass; a grid-rendering interpretation depends on callee 00474214. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004760fd` | `CacheLookupOrCreate ?` |  | Reference-counted cache used by the accepted line-record emitter 0047a92b (DrawSurfaceEmitLineRecord). The cache is not the client cursor system: that one is the LoadCursorA path 004759df/00476a23 with the 16-bit-key ma… | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |

## ⚪ unknown (30)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00429bb4` | `— ⚠reject` | GCtrl | GCtrl-family virtual slot 100 (35 owners): tiled draw of the control over a rectangle, repeating the control size. | [[Native Map/Shards/S0806 00406512-00621d93|S0806]] |
| `0045af7c` | `— ⚠reject` |  | Part of the draw-state cache configuration: the accepted caller InitStageStateDefaults (004627cc, engine.gfc.draw) writes the default 0x28-byte stage records of the same cache. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b071` | `— ⚠reject` |  | Mask-word to pixel expansion used by the draw-device record classes; the 'texture' reading of the target comes from neighbouring accepted records, not from this function's own code. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b139` | `— ⚠reject` |  | Second mask-word expansion of the draw-device record classes; the 'texture' reading of the target comes from neighbouring accepted records, not from this function's own code. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0046b89c` | `— ⚠reject` |  | Virtual method of the two-vtable class installed by the D3D9 module constructors (vtable slot 0x017648d8); queues one clipped 28-byte draw record reusing the object cached clip node. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046bad7` | `— ⚠reject` |  | Virtual method (vtable slot 0x017648dc): queues one clipped 28-byte draw record using the caller-supplied clip region. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046bc5c` | `— ⚠reject` |  | Virtual method (vtable slot 0x017648f4): queues one clipped 36-byte draw record. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0046c0ce` | `— ⚠reject` |  | Virtual method (vtable slot 0x01764900): 36-byte record append using the caller-supplied clip region. | [[Native Map/Shards/S0022 0046b77b-0046efeb|S0022]] |
| `0047413a` | `— ⚠reject` |  | Inline member initialiser run by the command-buffer constructor 0047624a. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00474178` | `— ⚠reject` |  | Instance construction from the shared prototype table; called through the shared-pointer create helper 00476e88. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00474214` | `— ⚠reject` |  | Immediate-mode drawing of one layer/cell grid, called by the per-frame grid pass 00474818. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00474642` | `— ⚠reject` |  | Construction of one 0xCC-byte draw device; caller 00475902 builds one per format index. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004750e3` | `— ⚠reject` |  | Resolves the per-slot state used by the accepted surface emitters 0047b08e/0047b2ab/0047b3ce before a fill/blit record is emitted. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `0047543e` | `— ⚠reject` |  | Called by the deleting-destructor and shared-release paths at 00476348 and 004769ef. The association with their page object is caller-based; this body has no RTTI class name. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `0047559e` | `— ⚠reject` |  | Direct caller 00476d2d uses the returned record during a state switch; meanings of the individual field identifiers remain unknown. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004757b2` | `— ⚠reject` |  | Entry created by the cache lookup 004760fd when a key is missing; no cursor role is evidenced here. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00475815` | `— ⚠reject` |  | Fills the receiver's container from an external source for the five descriptor groups listed in DAT_01af69b4 (per-group counts through the source's virtual slot +0x18, entries through +0x1C, appended through 00474fb4); … | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `0047589b` | `— ⚠reject` |  | Record creation path of the draw-surface/device family; no callers are recorded, so the name rests on the accepted callee role. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004759df` | `— ⚠reject` |  | Startup population of the window/display cursor set; called from the sub-object constructor 004765cc. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00475da4` | `— ⚠reject` | GDrawSurface | The render-surface identity is supported by its factory 0047afe1 and matching destructor vtable pair at 00479eb9; embedded Fx types are named by vtables written in this body. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476364` | `— ⚠reject` |  | Page/state object backing the device-state switching path 00476d2d. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476387` | `— ⚠reject` |  | Per-target surface creation: one render surface exists per target texture the draw device draws into (its constructor is called by the accepted DrawSurfaceFactory 0047afe1). | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `0047676c` | `— ⚠reject` |  | Start-of-drawing setup that makes one render surface own a draw queue and a page object; the append primitives used are the accepted 00474e3b/00474947 emissions. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476d2d` | `— ⚠reject` |  | Switches the current state object of the draw queue's rendering pass (record+8 of each 32-byte record, applied from the batch execute pass 0047735b). | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476f03` | `— ⚠reject` |  | Destructor of the record-array holder; reached from the slot-state destructor 00477877 and the page/state teardown 0047990d. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476f76` | `— ⚠reject` |  | Record-array maintenance called by the batch pass (0047735b) and the stream flush (00477d48). | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `00476fe6` | `— ⚠reject` |  | Per-frame/end-of-use reset of a render surface; reached from the accepted DrawSurfaceDtor 00479eb9, the frame-buffer teardown 004777fc and the device shutdown 00477947. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `0047735b` | `— ⚠reject` |  | Final execute/drain pass of the draw queue: called at the end of the accepted DrawQueueDrainAndCommit 00477CD1 and from the per-item flush 004778B3 once the queued item count passes 0xFA. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004777fc` | `— ⚠reject` |  | Called by frame-buffer destructor 00477c64; it tears down the current first referenced entry while the container count remains nonzero. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |
| `004778b3` | `— ⚠reject` |  | Per-item consumption step of the draw queue, called by the accepted DrawQueueDrainAndCommit 00477CD1, DrawQueueReset 00477D48, DrawQueueAppendRecord 00477E05 and DrawQueueCompile 0047990D paths. | [[Native Map/Shards/S0024 00472fd2-004778b3|S0024]] |

