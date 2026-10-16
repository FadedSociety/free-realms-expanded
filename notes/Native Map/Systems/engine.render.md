# System: engine.render

96 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00463942` | `RegisterGDrawDeviceD3D9Classes` |  | Startup registration that makes the Direct3D 9 draw device and display implementers discoverable by name; this is the anchor naming the classes used throughout shard S0020. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `008a6460` | `BaseClient::InitGraphicsDLL` | BaseClient | Loads and initialises GraphicsDriver.dll from the client's Libraries config exactly once per process; the flag DAT_01be47e8 makes repeat calls no-ops. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008ddbc0` | `SetRenderFloraEnabled` |  | Toggles the RenderFlora terrain option, the flora rendering switch of the client terrain system. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008ef350` | `VideoCaptureSetOutputFile` |  | Applies the video-capture output file to the capture configuration and marks the setting as stored (flag at this+0x8fc). | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `008ef510` | `VideoCaptureSetResolution` |  | Sets the video-capture resolution from a 1/2/other index and marks the setting as stored. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `008ef740` | `VideoCaptureSetQuality` |  | Sets the video-capture quality value and marks the setting as stored. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `009173d0` | `EnsureRenderQualityKeysSet` |  | Makes sure the three rendering-quality config keys exist before the renderer applies them. | [[Native Map/Shards/S0241 009162f0-0091aa10|S0241]] |
| `0091dc20` | `UpdateCameraFromConfigOrPlayer` |  | Camera placement for the client's debug/spectator mode versus normal player follow; called once per frame context from the client loop 0092b170. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091dde0` | `LoadGraphicsQualitySettings` |  | Applies the user's graphics-quality tier at startup or after a settings change. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091e5a0` | `ApplyShadowQualitySettings` |  | Applies the shadow-quality configuration and materializes missing defaults into the settings file. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091e670` | `ApplyRenderDistance` |  | Applies the view-distance setting; callers 00922c10 and 00aecbf0 are settings/graphics-apply paths. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |

## 🟢 likely (46)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `004622fe` | `LockVertexBufferRegion` | GDrawDeviceD3D9 | Ring-buffer lock for the batch writer at 0047735b. The cursor wrap controls discard versus no-overwrite flags; param_3 is the minimum count and param_4 is the requested count. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004623b5` | `LockIndexBufferRegion` | GDrawDeviceD3D9 | Index-storage lock used by the batch writer at 0047735b next to the vertex lock. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004624ac` | `ApplyInitialRenderState` | GDrawDeviceD3D9 | Re-establishes the fixed-function state the batch renderer relies on after device creation or reset; the state set matches FVF 0x144 transformed 2-D vertices with two texture stages. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004627cc` | `InitStageStateDefaults` |  | Stage defaults of the state cache whose values FUN_0045adaa pushes to the device with SetTextureStageState/SetSamplerState; re-applied by the descriptor routines 00466ef3/00466f8b/00466fb9/00467075. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00462910` | `DtorVtable1764A18` |  | Cleanup of the 0x1764a18 record class constructed by 00467154 and destroyed through the deleting destructor 00467190. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `0046295d` | `ScalarDeletingDtor1764A18` |  | Standard MSVC deleting-destructor emission for the 0x1764a18 draw-record class. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00462979` | `RecordCtor1764A48` |  | Grid-row record created by 0046333c; carries the row coordinate pair that the batch expander later writes into the vertex stream. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004629d3` | `DtorVtable1764A48` |  | Cleanup of the grid-row records constructed by 00462979; deleting destructor emitted separately at 004671ac. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00462c36` | `RecordCtor1764670` |  | Record built by 00467368 while preparing a draw batch from a source descriptor object. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00462c72` | `ScalarDeletingDtor1764670` |  | Deleting-destructor emission for the 0x1764670 draw record. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00462d00` | `RecordCtor1764A68` |  | Texture/size record created per texture request by 00466a43 with the width and height pair in +0x1c/+0x20. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00462d6a` | `DtorVtable1764A68` |  | Cleanup of the texture/size records created by 00462d00; deleting destructor emitted at 00467330. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00462de3` | `RecordCtor1764A88` |  | Small per-object record inserted into the draw device list at +0x54; created from the UI/window code at 004784cd and 00478fa4. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00462e52` | `DtorVtable1764A88` |  | Cleanup of the records created by 00462de3; deleting destructor at 0046734c. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00462e95` | `RecordCtor1764688` |  | Front-inserted bookkeeping record of the draw device list rooted at owner+0x54. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00462ed2` | `ScalarDeletingDtor1764688` |  | Deleting-destructor emission for the 0x1764688 list record. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `0046333c` | `EnsureGridRowRecords` |  | Materialises the draw records that represent each grid row before the batch writer appends their vertices. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `0046340f` | `ProbeDeviceCaps` |  | Capability probe used when the display object has no live device yet, so it asks the IDirect3D9 object directly; the filled block feeds the display configuration code. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004634d7` | `GetDisplayExtent` |  | Extent query used by the display/draw device to size its presentation rectangle; the DAT_01bc45f4 hook selects between the system metric and a custom windowed source. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `0046358f` | `GetDeviceDirect3D` |  | Accessor that lets the display layer reach Direct3D for device queries when the draw device exists. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004636e0` | `PresentDirtyRegion` |  | Frame presentation of the display object with dirty-rectangle support, preserving the DirectDraw-era display semantics the GFC display keeps. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00463dc9` | `HitTestTypedRecords` |  | Tests matching type-5 entries in two record collections with 00463d4f; each collection prepares its record differently. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00463ef4` | `CollectRegistryPointers` |  | Builds a temporary array of non-null registry resource pointers, increments their +8 fields, and invokes two processing helpers for each; the trigger meaning is unknown. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00465260` | `ScalarDeletingDtorRecordA` |  | Deletion of the record type consumed by the callers at 0046d3ef and 0046fef5. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00465285` | `ScalarDeletingDtorRecordB` |  | Deletion of the record type consumed by the callers at 0046d647 and 0046f261. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004652aa` | `AppendEncodedDrawRecord60` |  | Record emission called by the per-type draw-record builders at 004689c4 and 00468b2f. The encoder writes the compact type tag through its second argument and returns encoded byte length. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00465395` | `EncodeDrawRecord14` |  | Record emission variant for the 20-byte expanded record layout. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004653d8` | `EncodeDrawRecords14Batch` |  | Batch record emission for the 20-byte expanded record layout. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00465480` | `EncodeDrawRecord14B` |  | Record emission variant used by the callers at 004697cb and 0046992d. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `0046556b` | `EncodeDrawRecord28` |  | Record emission variant used by the callers at 0046b665 and 0046b89c. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004655ae` | `EncodeDrawRecords28Batch` |  | Batch record emission for the 28-byte expanded record layout. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00465656` | `EncodeDrawRecord36` |  | Record emission variant used by the callers at 0046bc5c and 0046be93. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00465699` | `EncodeDrawRecords36Batch` |  | Batch record emission for the 36-byte expanded record layout. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466172` | `DrawSurfaceCtorB` |  | Constructor variant used when both the surface and a clipping rectangle are supplied. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466213` | `BindSurface` |  | Binds the supplied surface through 0045f753; the purpose of slot +0x128 is not established here. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466279` | `DrawSurfaceDetach` |  | Teardown path of the draw-surface object before the underlying surface is destroyed or replaced. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466ce6` | `DrawDevicePopNode32` |  | Free-list pop of the 0x20-byte device record pool, called from many GDS record constructors at 0046b665..0046c0ce. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466d6e` | `DrawDeviceCtor` | GDrawDeviceD3D9 | Object construction of the draw backend registered as GDrawDeviceD3D9 at 00463942; every later method of this shard (buffers, state cache, atlas texture, record pools) hangs off this layout. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466e97` | `ScalarDeletingDtorDrawDevice` | GDrawDeviceD3D9 | Deleting destructor paired with the draw-device constructor 00466d6e and destructor body 00462140. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466f8b` | `ConfigureStageStatesDefault` |  | Default (no texture / simple fill) state cache programming used when a draw has no texture mode descriptor. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00467190` | `ScalarDeletingDtor1764A18B` |  | Deleting-destructor emission for the 0x1764a18 record class constructed at 00467154. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004671ac` | `ScalarDeletingDtor1764A48B` |  | Deleting-destructor emission for the 0x1764a48 grid-row record class constructed at 00462979. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00467330` | `ScalarDeletingDtor1764A68B` |  | Deleting-destructor emission for the 0x1764a68 texture/size record constructed at 00462d00. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `0046734c` | `ScalarDeletingDtor1764A88B` |  | Deleting-destructor emission for the 0x1764a88 record class constructed at 00462de3. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `008ef220` | `CopyConstructRecord40` |  | Copy-constructs the 0x40-byte settings record used by the VideoCapture setters. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `008ef2e0` | `CopyAssignRecord40` |  | Writes a VideoCapture settings record back into the stored settings object. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |

## 🟡 speculative (14)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00462dad` | `AcquireTextureLock ? ⚠rename` |  | Texture acquisition/lock step for one texture-pool record of the draw device. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004633b7` | `ScalarDeletingDtorSurface ? ⚠rename` |  | Deleting destructor of the GFC draw-surface class whose vtable is 0x1764d40 (constructed at 00465e11, 00466172 and 004661c4). | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00465ea4` | `DrawImageRectSlot1BC ? ⚠downgrade` |  | Emitted wrapper of the GFC draw interface image-draw entry point used by the 2-D renderer. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00465f05` | `DrawImageRectSlot1B4 ? ⚠downgrade` |  | Second emitted wrapper of the GFC draw interface fill/image entry used by the 2-D renderer. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00465f60` | `DrawImageViaInterface ? ⚠downgrade` |  | Wrapper that resolves a texture/binding object before the draw-interface dispatch. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00465fa6` | `DrawImageRectSlot1CC ? ⚠downgrade` |  | Third emitted wrapper of the GFC draw interface image entry point. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466019` | `DrawImageRectSlot1C4 ? ⚠downgrade` |  | Combined wrapper selecting between the simple fill entry (+0x1b4) and the image entry (+0x1c4). | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `0046607d` | `DrawImageViaInterface2 ? ⚠downgrade` |  | Second texture-resolving wrapper of the GFC draw interface image entry. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466108` | `SetDrawCallback ? ⚠rename` |  | Callback registration used by the surface object to notify its owner (for example on state changes). | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004661c4` | `DrawSurfaceCtorC ? ⚠downgrade` |  | Constructor variant whose initial state comes from a GDS-style object instead of a raw surface. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466245` | `BindSurfaceWithRect ?` |  | The helper 0045f753 initializes a surface/frame object from an interface and optional rectangle; the purpose of slot +0x128 is not established here. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466cac` | `DrawDevicePopNode4K ? ⚠rename` |  | Free-list pop of the 64KiB-class device buffer pool (0xfffc bytes). | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `008ef4f0` | `CopyRecordStringAt0x20 ?` |  | Thiscall with one stack argument (an IString object pointer). Assigns the string data pointer at +0x20 of the record at this+0x8f8 into that IString through 00402ce0 (receiver = the argument, stack argument = the record… | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `0091a170` | `ApplyRenderConfigViaBuffer1024 ?` |  | Applies a render configuration string to the renderer object. Its caller ApplyShadowQualitySettings is an accepted label. | [[Native Map/Shards/S0241 009162f0-0091aa10|S0241]] |

## ⚪ unknown (25)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `004621ca` | `— ⚠reject` | GDrawDeviceD3D9 | Geometry-buffer initialisation of the Direct3D 9 draw device class registered as GDrawDeviceD3D9 at 00463942; the 0x1c-byte vertex stride with FVF 0x144 and the 16-bit index buffer are the storage the batch writer at 00… | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00462463` | `— ⚠reject` | GDrawDeviceD3D9 | Per-record submit step of the 2-D batch renderer: a textured triangle list limited to one texture per call. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004627ff` | `— ⚠reject` |  | Texture allocation for one pool record of the draw device; format 0x15 is D3DFMT_A8R8G8B8 and the size clamp uses the capability values cached by 00466d6e. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00462a02` | `— ⚠reject` |  | Ghidra retained only a partial body. Raw bytes test the dword at *(arg+0x44)+0x24; the zero branch reaches 00462c21, where AL is set to 1. On the nonzero path, the code makes indirect calls through vtable slots +0x48, +… | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00462c8e` | `— ⚠reject` |  | Adopts the draw device 512x512 atlas texture (created 0x200 by 0x200 in 00466d6e) for a record; the 0x400 byte accounting matches a 32-by-32-texel sub-tile. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00462f15` | `— ⚠reject` |  | Sprite-sheet/atlas grid table consumed by the 2-D batch renderer; the page entries supply the (u,v) bases and reciprocals while the cell entries hold the per-cell rectangles. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `0046321e` | `— ⚠reject` |  | Flips the atlas grid mapping between the two layout orientations used by the batch renderer. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00463d4f` | `— ⚠reject` |  | Sub-step of the display hit-test: records whose owner interface matches the tested object mark a hit through the shared rectangle helper. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00463fc3` | `— ⚠reject` |  | Managed replacement of a per-index resource (index 5) on the draw interface while keeping the stored interface at +0x3c in step. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004652ed` | `— ⚠reject` |  | Batch variant used when a whole run of records shares the same draw offsets. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004654c3` | `— ⚠reject` |  | Batch record emission for the second 20-byte record family. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00465e11` | `— ⚠reject` |  | Constructor of the GFC render-target/display-surface object; the paired bound state makes it the base of the objects whose deleting destructor is 004633b7. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004662e0` | `— ⚠reject` |  | State hand-over between two surface objects (for example when a device is lost and re-created and the live surface must move to the new object). | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466a43` | `— ⚠reject` |  | Texture object factory of the draw device used by the GDS texture callers; the reciprocals stored at +0x2c/+0x30 are the coordinate scale factors of the 2-D renderer. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466b28` | `— ⚠reject` |  | Teardown helper of the draw device pools (called by 00466bc1/00466c39/00466cac/00466ce6 on allocation failure and by the device teardown). | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466bc1` | `— ⚠reject` |  | Free-list pop of the 0x1010-byte device buffer pool (the 0x1000 payload matches the 4096-vertex capacity of the batch buffers). | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466c39` | `— ⚠reject` |  | Free-list pop of the 0x24-byte device record pool. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466eb3` | `— ⚠reject` | GDrawDeviceD3D9 | Per-frame acquisition of the draw device; the caller chain at 00467824/0046cac7/00479fce re-runs it after device resets. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466ef3` | `— ⚠reject` |  | Per-draw texture/blend mode programming of the state cache that FUN_0045adaa later pushes to the device; the descriptor flag word at +0xc selects the mode (bit 0x300 = texture count, bit 0x1000 = two-texture path). | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00466fb9` | `— ⚠reject` |  | Textured descriptor variant of 00466ef3 for the draw mode that also programs the second texture stage. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00467075` | `— ⚠reject` |  | Third texture/filter mode programming of the state cache; the 0x13 and 0x16 values are the axis-selector codes of the descriptor. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `00467154` | `— ⚠reject` |  | Record type created by the caller at 004757b2 and destroyed through 00467190/00462910. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `004671c8` | `— ⚠reject` |  | Bridges the layered draw object to the record submitter 00462a02; the record id at +0x28 selects which of the stored 0x3c-byte records is drawn. | [[Native Map/Shards/S0020 004621ca-0046734c|S0020]] |
| `009178a0` | `— ⚠reject` |  | Stores the shadow quality setting in the Rendering config section and applies it. The Lua binding SetShadowQuality (caller 00c16560) uses it. | [[Native Map/Shards/S0241 009162f0-0091aa10|S0241]] |
| `00b4bf30` | `— ⚠reject` |  | Takes a pointer to two 8-float boxes (param_3, min and max corners) and a pointer to a 16-float matrix (param_4) and a float; computes a centre and radius from the box corners with sqrt and rsqrtps, writes an 8-float re… | [[Native Map/Shards/S0345 00b4b3e0-00b50190|S0345]] |

