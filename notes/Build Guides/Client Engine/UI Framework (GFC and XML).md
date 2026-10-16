---
type: build-guide
section: Client Engine
updated: 2026-10-12
---

# UI Framework (GFC and XML)

Part of [[Build Guides]] › Client Engine. Related: [[Scaleform and Flash UI]], [[Text and Localization]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

5390 accepted functions: 2050 confirmed, 1697 likely, 961 speculative, 682 unknown. Network-linked (`wire`): 21.

Subsystems: `engine.gfc.core` (1458), `engine.gfc.control` (1362), `ui.datasource` (682), `ui.gui` (257), `engine.gfc.draw` (144), `engine.gfc.string` (130), `engine.gfc.serialization` (115), `engine.gfc.font` (114), `ui.core` (101), `engine.gfc.app` (97), `ui.factory` (76), `engine.gfc.fx` (66), `engine.gfc.win32` (66), `ui.tree` (63), `ui` (61), `engine.gfc.text` (59), `engine.datasource` (56), `engine.gfc.bitmap` (48), `engine.gfc.layout` (37), `ui.controls` (33), `ui.data` (32), `ui.bugreport` (30), `engine.gfc.config` (27), `engine.gfc.parse` (21), `ui.options` (19), `ui.dstree` (18), `engine.gfc.controls` (15), `ui.notification` (14), `ui.jobcustomization` (14), `ui.imagelist` (13), `ui.headshot` (13), `ui.minimap` (12), `ui.login` (12), `ui.job` (11), `ui.namechange` (11), `ui.pirates` (8), `ui.video` (8), `ui.merchant` (7), `ui.whatsnew` (7), `ui.dialog` (6), `ui.social` (6), `ui.socialshare` (6), `ui.tintgroup` (6), `ui.guild` (5), `ui.statdefinitions` (5), `ui.achievement` (5), `ui.equipscreen` (4), `ui.colors` (4), `ui.loading_screen` (4), `ui.marketplace` (4), `ui.processor` (3), `ui.view` (2), `ui.photographer` (2), `ui.help` (2), `ui.display_defaults` (2), `ui.commandhistory` (1), `ui.claim` (1), `ui.promotions` (1), `ui.version` (1), `ui.openurl` (1), `ui.profile` (1), `ui.hud` (1)

_Large or library guide: only confirmed and likely functions are listed (3747 of 5390)._

### `engine.datasource`

**NameValueDataSource** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a1ad0` | NameValueDataSource::ScalarDeletingDestructor | likely | Deleting destructor wrapper: calls FUN_0098dcf0 (which stores NameValueDataSource::vftable) and then calls _free(this) when param_2 bit 0 is set. |

**NudgeDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a41490` | NudgeDataSource::GetFieldName | confirmed | Virtual slot 1 of NudgeDataSource: maps a field id 0..0x14 to a literal field name and returns it, covering Id, ProgramId, GroupId, ContentId, StartDate, EndDa… |
| `00a41780` | NudgeDataSource::GetField1C | confirmed | Virtual slot 2 of NudgeDataSource returns the dword at *(*(*(param_1+0x224)+0x3e0)+0x1c). Its vtable is installed at object+0x30, so param_1+0x224 addresses ob… |
| `00a419e0` | NudgeDataSource::GetFieldValue | confirmed | Virtual slot 4 of NudgeDataSource: walks the nudge record list at *(*(this+0x224)+0x3e0)+0x14 (next at +0xd8) to row param_2, then writes field id param_3 into… |
| `00a41ee0` | NudgeDataSource::~NudgeDataSource | confirmed | Destructor body of the nudge data source: installs NudgeDataSource::vftable at +0/+0x30/+0x34 and calls the table base destructor FUN_0091bf60 with this. |
| `00a41f60` | NudgeDataSource::NudgeDataSource | confirmed | Constructor of the nudge data source: runs the table base constructor FUN_00922890 with the registry name that the caller supplied through PTR_s_BaseClient.Nud… |
| `00a41fe0` | NudgeDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0) of NudgeDataSource: calls the destructor body 00a41ee0 and frees the 0x258-byte object with _free when bit 0 of the … |

**T4DataSource** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e8d2d0` | T4DataSource::DtorBody | confirmed | Sets the vptr at this+0 to T4DataSource::vftable and returns. The body has no member cleanup and no free (bytes c7 01 cc 12 86 01 c3). |
| `00e8d2e0` | T4DataSource::DtorDeleteByFlag | confirmed | Sets the vptr at this+0 to T4DataSource::vftable; when bit 0 of the byte argument is set it calls 00e60fc0 on this (free), and returns this. |

**T4ExtractDataSource** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e8e2c0` | T4ExtractDataSource::DtorBody | confirmed | Sets the vptr to T4ExtractDataSource::vftable; calls 00e61030 on param_1[2] when nonnull; calls vtable slot [0] with flag 1 on the object at param_1[3] when no… |
| `00e8e360` | T4ExtractDataSource::Open | confirmed | Stores param_3 at param_1[4] and zeroes *param_4. Builds '<name>_data.dir' with sprintf into a 0x100-byte T4CharArray buffer (name from 00e53310 and 00e861e0 a… |

**T4ExtractDataSourceLoader::IndexParser** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e8d340` | IndexParser::ScalarDeletingDtor | confirmed | Sets the vptr at this+0 to T4ExtractDataSourceLoader::IndexParser::vftable; when bit 0 of the byte argument is set it calls 00e60fc0 on this (free), and return… |
| `00e8db30` | IndexParser::ScalarDeletingDtor | confirmed | Byte-identical duplicate of 00e8d340: sets the vptr at this+0 to T4ExtractDataSourceLoader::IndexParser::vftable; when bit 0 of the flag argument is set it cal… |
| `00e8df10` | IndexParser::ScalarDeletingDtor | confirmed | Byte-identical duplicate of 00e8d340: sets the vptr at this+0 to T4ExtractDataSourceLoader::IndexParser::vftable; when bit 0 of the flag argument is set it cal… |

**T4ExtractDataSourceLoader::IndexParser::NoOffsetParser** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e8db50` | NoOffsetParser::ParseLine | confirmed | Parses one index line without an offset column: strtoul key into param_2[0]; a tab, then strtoul size stored as size << 8 with the type byte kept low; a tab an… |

**T4ExtractDataSourceLoader::IndexParser::WithOffsetParser** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e8da40` | WithOffsetParser::ParseLine | confirmed | Parses one tab-separated index line: strtoul key into param_1[0]; a tab, then strtoul offset into param_1[1]; a tab, then strtoul size into param_1[2] (stored … |

**WhatsNewDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a3f940` | WhatsNewDataSource::GetFieldValue | confirmed | Virtual slot 4 of WhatsNewDataSource reached through the +0x30 subobject vtable: resolves a row index by a virtual call on the object at this+4, indexes the an… |
| `00a406c0` | WhatsNewDataSource::~WhatsNewDataSource | confirmed | Destructor body of the announcement data source: installs WhatsNewDataSource::vftable at +0 and at the two secondary subobjects +0x30 and +0x34, then calls the… |
| `00a407c0` | WhatsNewDataSource::WhatsNewDataSource | confirmed | Constructor: runs the table base constructor FUN_00922890 with the registry-name string, installs WhatsNewDataSource::vftable at +0/+0x30/+0x34, stores the own… |
| `00a40850` | WhatsNewDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0) of WhatsNewDataSource: calls the destructor body 00a406c0 and frees the 0x260-byte object with _free when bit 0 of t… |

**(no class)** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a41870` | NudgePollShowWindow | confirmed | Periodic nudge check: computes the elapsed time as (current time from FUN_00702670) minus the stored base at +8, and when that value exceeds the stored maximum… |
| `00e8df30` | IndexParserCreate | confirmed | With selector 1 it allocates 8 bytes through 00e611f0, sets the vptr to NoOffsetParser::vftable and writes 3 to +4. Otherwise it allocates 4 bytes through 00e6… |
| `00e8e030` | ExtractLoaderInit | confirmed | Zeroes the dword at this+0, stores param_2 at this+0xc and zeroes this+0x10. Allocates a 0x100-byte line buffer through 00e60d50 (tag T4ExtractDataSourceLoader… |
| `009fa000` | PopulateSelectedDataSourceValues | likely | Calls virtual slot +0x0c on the data-source receiver at this+0x34 with no stack argument. Reads the selected key from (*(this+0x254)+0xb4), follows the matchin… |
| `00a41910` | FormatIso8601Date | likely | Formats the six dwords at param_1 as an ISO-8601 UTC timestamp into the IString param_2 using the format "%04d-%02d-%02dT%02d:%02d:%02dZ" through SoeUtil::IStr… |
| `00a42470` | NotifyNudgesMatchingKey 🌐 | likely | Walks the nudge record list at *(*(*(FUN_00769710()+4))+0x3e0)+0x14 (next at +0xd8) and for every record whose TrackingType1 (+0xb8) equals 1 and whose Trackin… |
| `00a42540` | NotifyNudgesTrackingType2 🌐 | likely | Sweeps the nudge record list at *(*(*(FUN_00769710()+4))+0x3e0)+0x14 (next at +0xd8) and sends a nudge update through 00a42330 with record+0x24, record+0x28 an… |
| `00cc39a0` | ClearFlag2B0AndTailDispatch | likely | Stores zero at this+0x2b0, loads the function pointer at this vtable+0x10, and tail-jumps through it. |
| `00cc3a10` | RefreshAndSetFlag258 | likely | Calls the virtual function at the object stored at this+0x34, using that subobject as receiver, then stores one at this+0x258. |
| `00cc3c70` | RebuildFilteredLinkedList | likely | Calls the nested data-source reset, removes all current map nodes, walks the source chain at this+600, copies qualifying records into newly allocated nodes, in… |
| `00cc55d0` | ClearFlag25CAndTailDispatch | likely | Stores zero at this+0x25c, loads the function pointer at this vtable+0x10, and tail-jumps through it. |
| `00e8de00` | DirItemsLoadCount | likely | Pushes the 'Count' settings key with 00e68d80, reads the value into a T4 string through the object's vtable slot [1] (offset +4) and seeks it with 00e8c9e0, po… |
| `00e8e0b0` | ExtractLoaderRelease | likely | Frees the line buffer at param_1[1] through 00e60dc0 (tag line 0x180) when it is nonnull, then if the object at *param_1 is nonnull calls its vtable slot [0] w… |
| `00e8e130` | ExtractHeaderLoop | likely | Reads a line through the object at *this (vtable slot +0xc into the buffer this[1] of size this[2]); while the read succeeds and the line starts with '#', a li… |
| `00e8e190` | ParseIndexLines | likely | Reads lines through the object at *this (vtable slot +0xc). For each line that starts with a digit it takes an entry from the array at param_2+8 (12-byte strid… |
| `00e8e6d0` | ExtractLoadScoped | likely | Runs 00e8e030 (loader init) on param_1, then 00e8e360 with (param_2[4], *param_2, param_3), then 00e8e0b0 (loader release) and returns the result of 00e8e360. |

### `engine.gfc.app`

**GApplication** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00405d3d` | GApplication::InitializeControlManager | confirmed | Part of GApplication::Initialize: when the byte at +0x1A0 is clear it initializes the control manager at +0x1A8 through FUN_005180a5; on failure it reports the… |
| `00405ddc` | GApplication::ShutdownControlManager | likely | When the flag byte at +0x1A0 is set, calls FUN_00518119 on the control manager at +0x1A8 and removes it through virtual slot +0xAC. |
| `0043baad` | GApplication::CleanupControlManagerState | likely | When field +0x174 is negative, tears down the control manager's entry list through 0043a965, calls 00405ddc on this, and clears byte +0x1a0; otherwise it retur… |

**GCtrlTheme_Infinity** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00411f68` | GCtrlTheme_Infinity::Load | confirmed | Interns the driver name GFC_Drivers_CtrlTheme_Infinity, looks the object up in the global registry and, when absent, loads the module L"GCtrlTheme_Infinity.dll… |

**GuiApp** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0043eb16` | GuiApp::InitializeControlsA | confirmed | Guarded control-manager initialisation (runs only while this+0x174 < 0): calls the variant setup FUN_0043D9CD(param_2..param_5); on failure reports the wide st… |
| `0043ebc0` | GuiApp::InitializeControlsB | confirmed | Guarded control-manager initialisation (this+0x174 < 0): calls the variant setup FUN_0043DC9A(param_2,param_3,param_4); on failure reports 'GFC Error: GApplica… |
| `0043ec67` | GuiApp::InitializeControlsC | confirmed | Guarded control-manager initialisation (this+0x174 < 0): calls the variant setup FUN_0043DE42(param_2,param_3,param_4) (this handled by the callee); on failure… |
| `0043fcfb` | GuiApp::InitializeControlsFromStoredSize | confirmed | Runs only while this+0x174 is negative. Reads signed size words at +0x398/+0x39C, using globals DAT_01BC6974/DAT_01BC73B0 when zero, then calls 0043F98D. On fa… |
| `004c4c01` | GuiApp::ForwardChildSlot124 | confirmed | Virtual method in GuiApp::vftable slot 69 (+0x114): loads the object at this+0x88 and calls its vtable slot +0x124 with (this, param_2), then returns. |
| `004c4c19` | GuiApp::ForwardChildSlot128 | confirmed | Virtual method in GuiApp::vftable slot 77 (+0x134): loads the object at this+0x88 and calls its vtable slot +0x128 with (this, param_2), then returns. |
| `004c4c31` | GuiApp::GetFlag120Bit1 | confirmed | Virtual method in the GuiApp::vftable +0x28 table, slot 6: returns bit 1 of the dword at this+0x120 as a byte (mov, shr 1, and al 1, ret; 11 bytes). |
| `004c4c3c` | GuiApp::SecondaryCastOrNull | confirmed | Virtual method in the GuiApp::vftable +0x28 table, slot 7: returns this-4 when this is not the 0x28 secondary base address and 0 otherwise (lea, add -4, neg/sb… |
| `004c4c49` | GuiApp::ThunkTail18 | confirmed | Virtual method in the GuiApp::vftable +0x24 table, slot 6: adds 4 to this, loads the object pointer stored there and tail-jumps to its vtable slot +0x18 (no ar… |
| `004c4c51` | GuiApp::ThunkTail18 | confirmed | Virtual method in the GuiApp::vftable +0x24 table, slot 7: the same body as 004c4c49 (adds 4 to this, loads the pointer there and tail-jumps to its vtable slot… |
| `008330e0` | GuiApp::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor shape (ECX = this, one stack flag argument): calls the dtor body 0082f1c0 on this, tests bit 0 of the flag, and when it is … |
| `00834250` | GuiApp::CopyCtor | confirmed | Thiscall with one stack argument (source GuiApp). Calls 0043ea76(source) for the base initialisation, stores the GuiApp vtables at +0, +4, +0x24 and +0x28, cop… |
| `00436eb1` | GuiAppShutdownUI | likely | When the dword at +0x174 is negative, calls FUN_00431f07 with ECX=this+0x1a8, then GApplication::ShutdownControlManager (00405ddc) with ECX=this, then clears t… |

**(no class)** (46)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00405ef4` | TrySetState3A6 | likely | When bit 4 of the flag dword at +0x294 is clear, calls FUN_004e8d36(param_2), zeroes the 16-bit field at +0x3A4 and stores param_3 at byte +0x3A6; returns whet… |
| `00434a67` | DispatchApplicationEvent | likely | Dispatches on the event-type short at param_2+0x1c against five global type ids: activation/deactivation (state bits 0x20/0x40 at +0xec, parameter block throug… |
| `0043e7db` | CtorVtable0175F520 | likely | Calls base initializer 0054AB7D(param_2), writes four vtable-family pointers at object offsets 0, 4, 0x24 and 0x28, constructs the control manager at this+0x1A… |
| `0043ea76` | CtorDefaultVtable0175F520 | likely | Calls base initializer 0054AB7D(0), writes the same vtable-family pointers as 0043E7DB, constructs the control manager at this+0x1A8 through 0043A470, clears t… |
| `0043ead5` | DtorVtable0175F520 | likely | Reinstalls the vtable-family pointers at offsets 0, 4, 0x24 and 0x28; calls 00436EB1 with ECX=this, releases the string member at this+0x3A0 through 004DC61B, … |
| `0043f98d` | CreateControlManagerFromDisplays | likely | Builds a local control-manager descriptor (flags 0x10002, or 0x30002 when the device interface reports bit 8; constant 0x610888 at +4), resolves its arguments … |
| `0043fc7a` | CreateInstance0175F520 | likely | Reduced mode-selector factory for the 0x3A8-byte GApplication class: mode 0 runs the constructor in place (FUN_0043E7DB); mode 2 allocates through FUN_00405CA7… |
| `0043fcdf` | ScalarDeletingDtor0175F520 | likely | Calls destructor body 0043EAD5; when param_2 bit 0 is set, calls the shared free helper through thunk 00405CC4; otherwise returns param_1. The function uses RE… |
| `004485c9` | BaseCtorVtableP17635C8 | likely | Constructs a small polymorphic base object: runs the base constructor chain 004472bd with param_2, then stores the vtable pointer 017635c8 at offset 0 and retu… |
| `00449f5b` | WindowApplyWin32Style | likely | Applies the requested style mask: blocks when flag +0x3c is set, combines requested and cached flags, updates HWND style and extended style with rollback on fa… |
| `0044a326` | BaseCopyCtorVtableP1763790 | likely | Calls base constructor 004485c9 with param_2, stores PTR_FUN_01763790 at this+0, and returns this. |
| `0044a9fe` | SmallObjectCtorVtableP1763A40 | likely | Constructs a small polymorphic object: runs the base constructor 004485c9 with param_2, then overwrites offset 0 with vtable pointer 01763a40 and returns this. |
| `0044aa16` | TreeResetOrFreeFirstRecord | likely | Checks the tree begin/end iterators; on an empty tree clears the tree and cached id/value fields, otherwise frees the pointer in the first node value. |
| `0044b071` | StlVecFindIteratorA | likely | Builds the container's end and begin iterators with the checked constructor 004440e9, runs the 4-byte linear find 0044a8d7 over them against the key at param_3… |
| `0044b1dc` | StlVectorEraseByKey | likely | Erase-by-key member for a 4-byte element vector at +0x34: locates the key through the find-iterator builder 0044b071 (checked iterator construction 004440e9 pl… |
| `0044b255` | DrainRegisteredObjectList | likely | When byte +0xe4 is set, repeatedly obtains element 0 and calls its virtual slot 0x40 until the list is empty; then resets related tree/cache state and clears +… |
| `0044b296` | NextUniqueTreeKey | likely | Selects the counter at +0x100 for param_2 == 0 or +0x104 otherwise. Each pass forms a candidate from the old counter (ORing bit 31 for the alternate space), st… |
| `0044c949` | MultiVtableCtorCloneBC | likely | Constructs the multi-vtable class: runs the base constructor 005383ea on param_2, installs the five vtable pointers 01763ba0/01763b9c/01763b7c/01763b54/01763b3… |
| `0044ca67` | EraseTreeEntryAndFree | likely | Looks up the record key at param_2+8, compares the iterator with the tree end, then erases the node and frees param_2 when present; otherwise returns the compa… |
| `0044caed` | VectorCopyConstructA | likely | Copy-construction helper for a vector member: sizes the source through 00443a3a, allocates the new buffer with 0044ba33, builds the source end and begin iterat… |
| `0044cb77` | StlVectorResizeC | likely | Resize member for a 16-byte element vector: when the requested size is larger it builds the end iterator (checked constructor 004178d9) and appends the differe… |
| `0044cc10` | VectorCopyConstructB | likely | Second vector copy-construction helper: sizes from 00443ae2, allocates with 0044ba77, builds the source iterators with the factories 00446917/0044692b and copi… |
| `0044cca8` | StlVectorInsertAtA | likely | Insert-one member: computes the insertion offset (0 when the container is empty, otherwise the distance from the begin iterator built by 00446bde measured with… |
| `0044ceaf` | StlVectorInsertAtB | likely | Insert-one member: offset via the begin iterator built by 00446a9e and the distance helper 0044424c (0 when empty), one-element insert core 0044be77, iterator … |
| `0044cf12` | StlVectorInsertAtC | likely | Insert-one member: offset via the begin iterator built by 004440e9 and the distance helper 004441e5, one-element insert core 0044c02e, iterator rebuilt with 00… |
| `0044cf75` | StlVectorResizeA | likely | Resize member: when the requested size is larger it builds the end iterator (00446a69) and appends the difference through the insert core 0044c1e5; when smalle… |
| `0044d000` | StlVectorInsertAtD | likely | Insert-one member: offset via the begin iterator built by 00446a69 and the distance helper 00444217, one-element insert core 0044c1e5, iterator rebuilt with 00… |
| `0044d063` | StlVectorInsertAtE | likely | Insert-one member: offset via the begin iterator built by 00446c20 and the distance helper 00444333, one-element insert core 0044c39c, iterator rebuilt with 00… |
| `0044d0c6` | StlVectorResizeB | likely | Resize member for the counted-assign vector: when larger it builds the end iterator (00446c62) and appends the difference through 0044c5b0; when smaller it bui… |
| `0044d166` | StlVectorInsertAtF | likely | Insert-one member: offset via the begin iterator built by 00446ca4 and the distance helper 004443a6, one-element insert core 0044c78d, iterator rebuilt with 00… |
| `0044d1de` | MultiVtableCtorP1763BA0 | likely | Constructs the multi-vtable class from nothing: runs the base constructor 0053829d, zeroes the field +0xbc, installs the five vtable pointers 01763ba0/01763b9c… |
| `0044d21d` | ScalarDeletingDtorA | likely | Calls destructor body 00424ef8; when param_2 bit 0 is set, releases this through the deletion thunk 004422ad; otherwise returns this. |
| `0044d2ef` | WindowChildAt | likely | Returns the child element at an index: guards on flag +0x3c clear and the list pointer +0x28 non-null, fetches the child list through virtual slot 0x7c into a … |
| `0044d34d` | WindowUpdateChildren | likely | Updates the window's children: when flag +0x3c is clear it fetches the child container through virtual slot 0x7c; with param_2 == 1 it builds the iterator bloc… |
| `0044d449` | WindowDisplayStateCommand | likely | Window display-state operation: computes the target position from GetWindowRect and the client-area virtual 0xd8; switch on param_2 handles 0 (virtual 0x40), 1… |
| `0044dc54` | ReadClipboardText | likely | Reads text from the Win32 clipboard: GetClipboardData(13) (CF_UNICODETEXT) or, failing that, GetClipboardData(1) (CF_TEXT); allocates a 0x34-byte GStringObject… |
| `0044dcb3` | PasteClipboardText | likely | Clipboard paste helper: constructs the helper object 004466ce, calls the clipboard reader 0044dc54, then runs the teardown helper 0044350d and returns the stri… |
| `0044de50` | StlVectorPushBackC | likely | Push_back member: when size (00443ae2) < capacity (00443fab) it constructs one element at the end through the fill-and-advance step 0044b3bb and stores the new… |
| `0044df6b` | StlVectorPushBack4F | likely | Pushes one 4-byte element: when size is below capacity it constructs at the current end through 0044b446 and stores the returned end at +8; when full it constr… |
| `0044dfd0` | CachedObjectRelease | likely | Runs the cached-record reset 0044db39 and then drops the object cached at +0x14 through virtual slot 0x10(0) when it is present. |
| `0044e0db` | SetCachedObjectWithTimeStamp | likely | Stores an object pointer at +4 and, when it is non-null, stamps the pair at +8/+0xc with the current time by calling the GetLocalTime-based helper 004f9372 on … |
| `0044e180` | InvokeVirtual44Then20 | likely | Calls virtual slot 0x44 on this; when the returned pointer is non-null it calls virtual slot 0x20 on that object and returns the result, otherwise returns 0. |
| `0044e19f` | InvokeVirtual44Then54 | likely | Calls virtual slot 0x44 on this, then invokes virtual slot 0x54 on the returned object with (param_2, this+0x90). |
| `005471f1` | DtorVtable017759D0 | likely | Fastcall destructor body: reinstalls the four vtable-family pointers (*param_1 = &PTR_FUN_017759d0, param_1[1] = &PTR_LAB_017759c8, param_1[9] = &PTR_LAB_01775… |
| `005472aa` | CreateApplicationModuleInstance | likely | Thiscall (ECX=this). Returns 1 immediately when the flag word at this+0x174 is already negative. Otherwise it allocates a 0x34-byte object with FUN_004d9855 an… |
| `008d21d0` | UpdateLoadingScreenAndAbortCheck | likely | Per-frame application update: when the app is not in the WFWR path it calls FUN_00825640(1,1) and reports 'GUI is aborting: WFWR' through the app's virtual err… |

### `engine.gfc.bitmap`

**(no class)** (20)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00482804` | RegisterGTiledBitmapFontClass | confirmed | Looks up or creates the GFC class registration named "GTiledBitmapFont" with base size 0x18 via FUN_0053da53, extracts the registration payload through FUN_004… |
| `00486164` | CtorVtables1769BA0 | likely | Calls the GObjectBase constructor FUN_004d99d1, installs &PTR_FUN_01769BA0 / &PTR_LAB_01769B98 and zeroes eight member dwords from +0x38 to +0x58. |
| `004861d6` | CtorVtables1769BA0_FromSize | likely | Same member initialization as 00486164 (GObjectBase constructor plus the 01769BA0 vtable block and eight zeroed dwords) followed by FUN_0047CA5E(param_2,param_… |
| `0048621e` | CtorVtables1769BA0_FromPair | likely | Same member initialization as 00486164, then calls FUN_0047CA5E(param_2, *param_3, param_3[1]) so the image is created from a format plus a two-dword size pair. |
| `00486328` | DtorVtables1769BA0 | likely | Destructor of the bitmap/frame-sequence class: restores the vtable block 01769BA0/01769B98, releases the owned image through FUN_0047CC98 when +0x44 (field 0x1… |
| `00486c7e` | RangedValuesAllEqual | likely | Returns 1 when every element of the dword array in [param_2, param_2+param_3) equals param_4, 0 as soon as one differs; the elements are reached through the ch… |
| `0048741e` | ConvertAndStorePixel | likely | Reads the format code and metadata through param_1, converts the 32-bit colour param_5 and stores a packed value at caller-supplied address param_2. The fused … |
| `00487b20` | ConvertAndStorePixelWithFastPaths | likely | Uses a fused first branch for 0x211555, 0x214444 and 0x618888, then a general layout-pack branch, a sub-byte luminance/converter path, and a format-specific st… |
| `004882dd` | BitmapGetPixel | likely | Bounds-checks x = param_2 against descriptor[1] and y = param_3 against descriptor[2] (descriptor = *(this+0x44); 0 on null descriptor/data or flag 0x40000 wit… |
| `004884a4` | BitmapSetPixelRaw | likely | Same descriptor/bounds checks as 004882dd, then stores the caller-supplied pre-formatted value param_4 at the computed pixel address: 3-byte store for 0x410888… |
| `00488686` | BitmapSetPixel | likely | Full converted set-pixel path: resolves descriptor at this+0x44, rejects null data (descriptor[9]==0), out-of-range x/y and custom formats without a converter,… |
| `00488a1f` | BitmapSetPixelWithAlphaPlane | likely | Set-pixel variant with separate alpha-plane support: when *descriptor has flag 0x800000 it first stores the colour's alpha byte (param_4>>24) into the alpha pl… |
| `00488ff6` | BitmapSetPixelAlpha | likely | Writes only the alpha component of a pixel: bounds-checks x/y against descriptor[1]/[2]; when flag 0x800000 is set it stores param_4 at descriptor[7]*x + descr… |
| `0048b14e` | PcxHeaderRead | likely | Reads a 128-byte PCX header through the stream wrapper opened by FUN_00485ccc: four single bytes into +0..+3 (manufacturer/version/encoding/bits-per-pixel), si… |
| `0048b287` | PcxHeaderWrite | likely | Writes the same 128-byte PCX header back through the stream wrapper from FUN_00485d66: single bytes +0..+3, 16-bit fields +4..+0xe, 0x30-byte palette block at … |
| `0048bb24` | PpmHeaderWrite | likely | For magic 0x3350 or 0x3650, opens a stream and writes the two-byte magic and first newline without checking either result. It writes width and height through 0… |
| `0048c220` | PackPixelToDepth | likely | Packs the colour param_3 into the buffer param_1 for the requested bit depth param_2: 8 -> one byte obtained from the converter object param_4's virtual slot +… |
| `0048c2de` | TgaHeaderRead | likely | Reads the TGA header fields in sequence. Earlier field-read results are ignored; only the final one-byte read at +0x13 is checked. If it returns 1, seeks forwa… |
| `0048c3d3` | TgaHeaderWrite | likely | Writes the TGA header fields in sequence; earlier write results are ignored. Only the final one-byte write at +0x13 is checked. If it returns 1, seeks by the I… |
| `0048c4f5` | TgaWriteColorMapEntries | likely | When the stream object is present, reads entry count from header+6 and entry depth from header+8. For nonzero counts, fetches each source entry through virtual… |

### `engine.gfc.config`

**Preferences::PreferencesManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012e43c0` | Preferences::PreferencesManager::DtorBody | confirmed | __fastcall (ECX = this). Stores the PreferencesManager vftable at this, then calls the base dtor body 012e41e0 on this. Returns void. |
| `012e4480` | Preferences::PreferencesManager::ScalarDeletingDtor | confirmed | Virtual slot [0] of PreferencesManager: thiscall with one flag byte. Stores the PreferencesManager vftable, calls the base dtor body 012e41e0 on this, then cal… |
| `012e4360` | Preferences::PreferencesManager::Ctor | likely | __fastcall (ECX = this), no stack arguments. Calls the PreferencesManagerBase constructor body 012e42d0 on this, then stores the Preferences::PreferencesManage… |

**Preferences::PreferencesManagerBase** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012e41e0` | Preferences::PreferencesManagerBase::DtorBody | confirmed | __fastcall (ECX = this), no stack arguments. Stores the PreferencesManagerBase vftable at this, tears down the map at this+0x20 through 012e4170, frees the fil… |
| `012e4340` | Preferences::PreferencesManagerBase::ScalarDeletingDtor | confirmed | Virtual slot [0] of PreferencesManagerBase: thiscall with one flag byte. Calls the dtor body 012e41e0 on this, then calls _free(this) only when bit 0 of the fl… |
| `012e2fb0` | Preferences::PreferencesManagerBase::LoadKeyValueFile | likely | __thiscall, one stack argument (file name string). Stores a copy of the name in this+4 (assign), opens the file with _fopen (mode r) and returns if it fails. O… |
| `012e3250` | Preferences::PreferencesManagerBase::SaveKeyValueFile | likely | __thiscall, one stack argument (file name string), ret 4. Opens the file with _fopen mode w; on success copies the name into this+4 and walks the ValueData map… |
| `012e42d0` | Preferences::PreferencesManagerBase::Ctor | likely | __fastcall (ECX = this), no stack arguments. Stores the PreferencesManagerBase vftable at this, initialises the filename string at this+4 empty (capacity 0xf, … |

**(no class)** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00408920` | LoadTooltipTheme | confirmed | Guarded by FUN_004c6e64, reads the theme attributes ToolTipFlags (with the enum tag GFC_CT_TOOLTIP_DISABLED through slot +0x44), ShowDelay, HideDelay, ToolTipW… |
| `00408aac` | SaveTooltipTheme | confirmed | Guarded by FUN_004c6e70, writes the same nine tooltip theme attributes (ToolTipFlags via slot +0x48 with the GFC_CT_TOOLTIP_DISABLED tag, the delay/size fields… |
| `004094c7` | SaveLayoutTheme | confirmed | Writes the byte at +0x16 under the name HStretch and the byte at +0x17 under VStretch through the config object's slot +0x20 (only when each byte is non-zero),… |
| `004c355f` | ReadInputBindings | confirmed | Thiscall-style reader (object pointer in param_1): returns 0 unless 004c6e64 accepts the object; then looks up the 'Bindings' array via the object's vtable slo… |
| `004c3757` | WriteInputBindings | confirmed | Writer counterpart of 004c355f (object pointer in param_1, returns 0 unless 004c6e70 accepts it): opens the 'Bindings' array through vtable slot +0x74 under th… |
| `00401d80` | ConfigReadInt64 | likely | Reads a text value from the source object into a 2048-byte stack buffer through FUN_006fba40(source, buffer, 0x800, param_3, 1), parses that text as a signed 6… |
| `00403280` | ConfigReadInt64 | likely | Config wrapper: formats the key text with 00403150 into a 1024-byte stack buffer, reads the value text through thunk_FUN_006febe0(handle, key, formatted, out, … |
| `006f9920` | KeyValueFileLoad | likely | Opens the text file named at this+0xc with fopen mode rt and reads it line by line with a 0x2000-byte line buffer. A line that starts with '[' creates a 0x54-b… |
| `006f9bb0` | KeyValueSet | likely | Thiscall with three stack strings: param_2 is the section name, param_3 the key name and param_4 the value text. It finds the section by name (006fb490 over th… |
| `006f9db0` | FreeChildChain | likely | Walks the child list headed at this+0x48 through the next pointer at +0x4c and, for each node, frees the pointer at +0x40 and then the node itself. Returns wit… |
| `006f9fd0` | NodeChainFreeAndMaybeSelf | likely | Thiscall with one stack argument (ret 4): calls FreeChildChain on this, then frees this when the argument's bit 0 is set, and returns this. |
| `006fa1b0` | KeyValueSetFromInt | likely | Thiscall with one stack argument (the int) and the section and key strings: formats the int with %d into a 32-byte stack buffer through _sprintf, then calls Ke… |
| `006fa210` | KeyValueSetFromFloat | likely | Formats a float argument with %f into a 32-byte stack buffer through _sprintf and passes it to KeyValueSet with the section and key strings. |
| `006fa270` | KeyValueSetFromDouble | likely | Formats an 8-byte double argument with %f into a 32-byte stack buffer through _sprintf and passes it to KeyValueSet with the section and key strings. |
| `006fa2d0` | KeyValueSetFromBool | likely | Formats a bool-like byte argument as 1 when it equals 0x01 and 0 otherwise, with %d into a 32-byte stack buffer through _sprintf, then passes it to KeyValueSet… |
| `006fa8a0` | SectionKeyNamesToStrings | likely | Thiscall (this = key-value list, param_2 = section name, param_3 = IString<char> array). It finds the section index with 006f9720, then for each key of that se… |
| `012e4410` | PreferencesManagerEnsureGlobal | likely | No arguments (plain ret). If the global DAT_01cbd764 is null: allocates 0x2c bytes with operator new (00d833a6); when the allocation is non-null it constructs … |

### `engine.gfc.control`

**Button** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c28470` | RegisterButtonModule | confirmed | Lua module initialiser: calls the register helper 00e39e40(L,'Button',table@0x0182f3d8) whose entries are SetFocus->00c28270, Show->00c28320, Hide->00c28350, G… |
| `00c280f0` | Button::SetText | likely | Lua method body: requires exactly 2 arguments, guards argument 1 with 00c28090, reads the string argument with 00e39c30(L,2,0), converts it through 0083ce80 in… |
| `00c281b0` | Button::SetProperty | likely | Lua method body: requires exactly 3 arguments, guards argument 1 with 00c28090, reads both string arguments (00e39c30 at indices 2 and 3), resolves the propert… |
| `00c28270` | Button::SetFocus | likely | Lua method body: guards argument 1 with 00c28090, zeroes a 7-dword structure plus word (seeded from DAT_01bc3b40) and passes it to the button's virtual method … |
| `00c28610` | Button::GetSize | likely | Lua method body: guards argument 1 with 00c28090, then pushes the 32-bit fields at control+0x70 and control+0x74 as Lua numbers through 00e3aa30 and returns 2 … |
| `00c28650` | Button::GetProperty | likely | Lua method body: requires exactly 2 arguments, guards argument 1 with 00c28090, resolves the property name string (00e39c30 + 00820460) and, when the id is not… |
| `00c28760` | Button::GetText | likely | Lua method body: guards argument 1 with 00c28090, builds a SoeUtil::IString<char> temporary (empty data 01bc7e20) and reads the button text through 006066d6 wi… |

**EditControl** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c27c50` | RegisterEditControlModule | confirmed | Lua module initialiser: calls the register helper 00e39e40(L,'EditControl',table@0x0182f2b8) whose entries include clear->00c279c0, getCharCount->00c27980, set… |
| `00c27780` | EditControl::setProperty | likely | Lua method body: requires exactly 3 arguments, guards argument 1 with 00c27650, reads both string arguments (00e39c30 at index 2 and 3), resolves the property … |
| `00c277f0` | EditControl::getInfo | likely | Lua method body: guards argument 1 with 00c27650, then calls 00857db0 with three output slots (local_4, local_8, local_c) and pushes the three returned values … |
| `00c278d0` | EditControl::setFocus | likely | Lua method body: guards argument 1 with 00c27650, zeroes a 7-dword structure plus a word (local_2c, local_10 seeded from DAT_01bc3b40) and passes it to the con… |
| `00c27ad0` | EditControl::setCaret | likely | Lua method body: guards argument 1 with 00c27650 and, when exactly 2 arguments are present, reads the integer at index 2 through 00e3a7b0(L,2,0) and calls 0064… |
| `00c27df0` | EditControl::getSize | likely | Lua method body: guards argument 1 with 00c27650, then pushes the 32-bit fields at control+0x70 and control+0x74 as Lua numbers through 00e3aa30 and returns 2 … |
| `00c27e30` | EditControl::setHitTestNone | likely | Lua method body: when exactly 1 argument is present it guards it with 00c27650 and calls 0043833b with the control's style dword at +0x58 rewritten as (flags &… |
| `00c27e80` | EditControl::setPlacementLTWH | likely | Lua method body: guards argument 1 with 00c27650 and, when exactly 5 arguments are present, reads four integers through 00e3a7b0 (indices 2..5), packs them wit… |
| `00c27f10` | EditControl::getText | likely | Lua method body: guards argument 1 with 00c27650, builds a SoeUtil::IString<char> temporary (empty data 01bc7e20) and asks the control for its text through 006… |

**GButtonCtrl** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `006063c1` | WriteThisAndToken01b1cad4 | likely | Virtual method (GButtonCtrl vtable slot 1): if the third argument is non-null it stores the receiver (ECX) through it, and it always stores the global data wor… |
| `0060671a` | PostStateChange7 | likely | Calls the receiver's virtual slot +0x228 with two zero arguments and then FUN_00411506(7) (accepted label PostStateChange: builds the state-change record with … |
| `0060f8a9` | GButtonCtrl::StyleWordFromF0 | likely | Thiscall without stack arguments. Reads this+0xf0. When bit 0x1000 is clear, or the 0xc000 field is neither 0xc000 nor 0x8000, returns 0x400031 OR 0x20000 (whe… |

**GCheckBoxCtrl** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00606832` | WriteThisAndToken01b1d294 | confirmed | Virtual method (GCheckBoxCtrl vtable slot 1): if the third argument is non-null it stores the receiver (ECX) through it, and it always stores the global data w… |
| `00630f60` | GCheckBoxCtrl::PaintFaceAndCaption | likely | Vtable slot [105] of GCheckBoxCtrl (GuiCheckBoxCtrl has the same slot). Calls FUN_0040f51e and returns 2 when it does. Otherwise it emits a kind-2 GFillFx reco… |

**GComboBoxCtrl** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00606d17` | WriteThisAndToken01b1d40c | confirmed | Virtual method (GComboBoxCtrl vtable slot 1): if the third argument is non-null it stores the receiver (ECX) through it, and it always stores the global data w… |
| `00606e9c` | GComboBoxCtrl::SubProbeElseOff230 | likely | Thiscall with three stack arguments (ret 0xc; A2 is read). When A2 is nonzero and the embedded object at this+0x24 accepts it (its vtable[0xd](A2) returns nonz… |
| `00606ec8` | GComboBoxCtrl::ClearFlagEcRemoveId | likely | Thiscall with one 16-bit stack argument (ret 4). When it equals the short DAT_01bd5ea0, clears bit 0x10000000 in the dword at this+0xec and calls 00424fe0(DAT_… |
| `00610454` | GComboBoxCtrl::GetField118 | likely | Thiscall without stack arguments: returns the dword at this+0x118. |
| `00610557` | GComboBoxCtrl::GetField11c | likely | Thiscall without stack arguments: returns the dword at this+0x11c. |
| `016dc228` | RegisterGComboBoxCtrlNode | likely | Thiscall with ECX = node &DAT_01bd6c8c. Pushes 0, the pointer slot &PTR_s_GFC_CF_COMBOBOX_NOBORDER (01b1d428) and the string 'GComboBoxCtrl', then calls 004c62… |

**GCtrl** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007055a0` | SetZLayer | confirmed | Stores the supplied z-layer value in the control object field at offset 0x8c. |
| `00406512` | GCtrl::TailJumpOff1e4 | likely | Thiscall thunk (bytes 8b 01 ff a0 e4 01 00 00): loads the vtable from ECX and jumps to this->vtable+0x1e4 (slot 121) with the caller's stack unchanged. No body… |
| `0040651a` | GCtrl::TailJumpOff1e8 | likely | Thiscall thunk (bytes 8b 01 ff a0 e8 01 00 00): loads the vtable from ECX and jumps to this->vtable+0x1e8 (slot 122) with the caller's stack unchanged. |
| `00406522` | GCtrl::TailJumpOff1f0 | likely | Thiscall thunk (bytes 8b 01 ff a0 f0 01 00 00): loads the vtable from ECX and jumps to this->vtable+0x1f0 (slot 124) with the caller's stack unchanged. |
| `00406c45` | GCtrl::Match3TagStoreFlag | likely | Thiscall with eight stack arguments (ret 0x20). Compares the 16-bit first argument with the shorts DAT_01bc7308, DAT_01bc713c and DAT_01bc7488. On a match it s… |
| `00407318` | GCtrl::GuardedDispatchOff210 | likely | Thiscall with four stack arguments (ret 0x10). If A1 is non-null and A1->vtable[0x20](A2) returns nonzero, returns this->vtable[0x210](A1, A3, A4) (its EAX). O… |
| `004118d6` | GCtrl::GuardedChildPairCall | likely | Thiscall with two stack arguments (ret 8). Returns 0 when A1 is 0. Otherwise p = this->vtable[0x204](1); if p is non-null: p->vtable[0xc](), r = p->vtable[0x78… |
| `00411922` | GCtrl::GuardedChildTripleCall | likely | Thiscall with three stack arguments (ret 0xc). Same shape as 004118d6: returns 0 when A1 is 0; p = this->vtable[0x204](1); when p is non-null calls p->vtable[0… |
| `0041e428` | GCtrl::ZeroArg1Quad | likely | Thiscall-convention virtual with three stack arguments (ret 0xc). Writes zero to the four dwords at its first stack argument ([esp+4]). ECX and the other two s… |

**GEditCtrl** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00607f8c` | GEditCtrl::ToggleFlag100AndNotify | likely | Thiscall with one 16-bit stack argument (ret 4). When it equals the short DAT_01bd5fd4, toggles bit 0x1 of the dword at this+0x100 and calls this->vtable[0xa9]… |
| `0060823b` | GEditCtrl::TestArg1Zero | likely | Virtual with five stack arguments (ret 0x14). Returns AL=1 when the first stack argument is zero, else 0. ECX is not used. |
| `0061c532` | GEditCtrl::ApplyStateByteArg | likely | Thiscall with one byte stack argument (ret 4). Calls 0041950b(A1). When A1 is zero: calls 00424fe0(DAT_01bd5fd4), clears bit 0x1 of this+0x100, calls this->vta… |

**GFormCtrl** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00609fee` | GFormCtrl::FlagsE8Or220000 | likely | Thiscall without stack arguments: returns the dword this+0xe8 OR 0x220000. |

**GLabelCtrl** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `006153ea` | GLabelCtrl::StyleWordFromEc | likely | Thiscall without stack arguments. Reads this+0xec. When bit 0x10 is set and the 0xc0 field is 0xc0 or 0x80, returns 0x220000 OR a word built from 00609c78 (ret… |

**GRadioButtonCtrl** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `006068ca` | WriteThisAndToken01b1d354 | confirmed | Virtual method (GRadioButtonCtrl vtable slot 1): if the third argument is non-null it stores the receiver (ECX) through it, and it always stores the global dat… |
| `00631533` | GRadioButtonCtrl::PaintFaceAndCaption | likely | Vtable slot [105] of GRadioButtonCtrl (GuiRadioButtonCtrl has the same slot). The pseudocode is identical to 00630f60 and the raw bytes differ only in rel32 di… |

**GTabCtrl** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `006a2771` | GTabCtrl::ReadTabProperties | confirmed | Virtual slot [62] of GTabCtrl (also in GuiTabCtrl). Calls the base reader 00436f31 on the source object param_2 and returns 0 if it fails. On success zeroes th… |
| `006a292a` | GTabCtrl::CopyStateFrom | likely | Virtual slot [64] of GTabCtrl (also GuiTabCtrl). Accepts the source param_2 through 00437372 (returns 0 when rejected). On success copies the source fields +0x… |

**GTableCtrl** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `006a2a20` | GTableCtrl::LifecycleDispatch | likely | Non-virtual mode-switched create/destroy helper taking three arguments: mode (param_1), object (param_2) and an extra argument (param_3). Mode 0: in-place cons… |
| `006a2b97` | GTableCtrl::EnsureChildControls | likely | Thiscall(this, param_2 flag word, param_3 flag word, param_4 int pointer) returning a char that is 1 when any child changed. Returns 0 when this+0x90 bit 0x10 … |

**GWindowCtrl** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0060c2c8` | GWindowCtrl::TopBitsToCode | likely | Thiscall without stack arguments. Takes the top two bits of this+0xf0: 0 gives 0x4440033, 0x40000000 gives 0x8440033, 0x80000000 gives 0xc440033, and any other… |
| `0060c50d` | GWindowCtrl::Notify0EPost7 | likely | Thiscall with two unused stack arguments (ret 8). Calls 004396f7 with one pushed argument 0xe (ECX=this), then 00411506(7). |
| `0060c63e` | GWindowCtrl::QueryPopupMenuProperty | likely | Thiscall without stack arguments. Builds the string "PopupMenu" (004e4e6d with length -1) in a temporary, calls the sub-object at this+0x24 through vtable[0x13… |
| `0060c739` | GWindowCtrl::ArgZeroCallSlot162 | likely | Thiscall with one byte stack argument (ret 4). When the argument is zero, calls this->vtable[0xa2]() (slot 162) with no arguments. |
| `0061f417` | GWindowCtrl::ForwardConst1 | likely | Thiscall with one stack argument that is ignored (ret 4). Pushes the constant 1 and calls 0061ef45 (ECX=this from entry). |
| `00629b7a` | GWindowCtrl::UpdateFrameChildren | likely | Vtable slot [120] of GWindowCtrl (GuiWindowCtrl has the same slot). Returns 0 at once when flag bit 0x10 at +0x90 is set or the owner pointer at +0x44 is null.… |

**GuiButtonCtrl** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0062fc71` | GuiButtonCtrl::ComputeLayoutRects | likely | Vtable slot [162] shared by GuiButtonCtrl and the check/radio classes. Writes three rectangles of four ints through the caller's pointers param_2, param_3 and … |
| `006309fc` | GuiButtonCtrl::PaintFaceAndCaption | likely | Vtable slot [105] of GuiButtonCtrl (also reachable through vftable[4][101]). Calls FUN_0040f51e and returns 2 when it does. Otherwise it picks two colors from … |

**GuiImageCtrl** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00609138` | GuiImageCtrl::FlagE8SelectCode | likely | Thiscall returning a byte: 0x44 when this+0xe8 has bit 0x10 set, otherwise 0x33 (test, neg, sbb, and 0x11, add 0x33). |
| `0061cfd4` | GuiImageCtrl::NotifyFlagGatedCallouts | likely | Thiscall with one stack argument (ret 4). Calls 0041950b(A1). When A1 is zero and this+0xe8 has bit 0x10000000, calls 0061507b(0) with ECX=this. When A1 is non… |
| `0062745d` | GuiImageCtrl::PaintBorderAndBlit | likely | Vtable slot [105] of GuiImageCtrl. Calls FUN_0040f51e and returns 2 when it returns 2. Otherwise it reads a width and height from this+0x94..0xa0 and the flag … |

**ImageControl** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c26bb0` | ImageControl::getSize | likely | Lua method body: calls the shared ImageControl userdata checker 00c267a0 with stack index 1; when that returns the wrapped control pointer it pushes the two 32… |
| `00c26bf0` | ImageControl::getPosition | likely | Lua method body: calls the shared ImageControl userdata checker 00c267a0 with stack index 1; when it returns the wrapped control pointer it pushes the two 32-b… |

**LabelControl** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c27320` | RegisterLabelControlModule | confirmed | Lua module initialiser: calls the register helper 00e39e40(L,'LabelControl',table@0x0182f198) whose entries are getText->00c274c0, setText->00c271b0 and find->… |
| `00c271b0` | LabelControl::setText | likely | Lua method body: requires exactly 2 arguments (lua_gettop-style 00e3a390), guards argument 1 with 00c27150, reads the string argument with 00e39c30(L,2,0), wra… |
| `00c274c0` | LabelControl::getText | likely | Lua method body: requires exactly 1 argument, guards argument 1 with 00c27150, builds a SoeUtil::IString<char> temporary (vftable 0175b24c, empty-string data 0… |

**TableControl** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c26e10` | RegisterTableControlModule | confirmed | Lua module initialiser: calls the register helper 00e39e40(L,'TableControl',table@0x0182f0f0) whose entries are getSelectedRow->00c26d10, getData->00c26fb0, fi… |
| `00c26d10` | TableControl::getSelectedRow | likely | Lua method body: guards the argument with 00c26cb0(L,1); when the guard returns a control it calls 00663fa0 to copy the control's selection pair (fields +0x138… |
| `00c26fb0` | TableControl::getData | likely | Lua method body: guards with 00c26cb0(L,1); when lua_gettop-style 00e3a390(L) reports exactly 3 arguments it reads two integers with 00e3a7b0(L,-2) and 00e3a7b… |

**WebBrowser** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c288d0` | RegisterWebBrowserModule | confirmed | Lua module initialiser: loads the module-name pointer 0x01b6c4dc ('WebBrowser') and the method table 0x0182f4e8 (BeginIMEDialog->00c28860, ConfirmCompositionSt… |

**Window** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c28e60` | RegisterWindowModule | confirmed | Lua module initialiser: calls the register helper 00e39e40(L,'Window',table@0x0182f580) whose entries include GetPosition->00c29050, SetPosition->00c28a50, Get… |
| `00c289e0` | Window::SetProperty | likely | Lua method body: requires exactly 3 arguments, guards argument 1 with 00c28980, reads both string arguments (00e39c30 at indices 2 and 3) and, after resolving … |
| `00c28c90` | Window::PostFocus | likely | Lua method body: requires exactly 1 argument, guards it with 00c28980, zeroes a 7-dword structure plus word (seeded from DAT_01bc3b40) and passes it to the win… |
| `00c28d50` | Window::SetCaption | likely | Lua method body: requires exactly 2 arguments, guards argument 1 with 00c28980, reads the string at index 2 with 00e39c30 and, when non-empty, calls the proper… |
| `00c29000` | Window::IsVisible | likely | Lua method body: guards argument 1 with 00c28980 and, when the guard returns a window, pushes a Lua boolean through 00e3ac00 with the value ((window flags at +… |
| `00c29050` | Window::GetPosition | likely | Lua method body: guards argument 1 with 00c28980, then pushes the 32-bit fields at control+0x68 and control+0x6c as Lua numbers through 00e3aa30 and returns 2 … |
| `00c290f0` | Window::GetSize | likely | Lua method body: guards argument 1 with 00c28980, then pushes the 32-bit fields at window+0x70 and window+0x74 as Lua numbers through 00e3aa30 and returns 2 Lu… |
| `00c29130` | Window::SetFocus | likely | Lua method body: when exactly 1 argument is present it guards it with 00c28980, otherwise it resolves the global scene by name through 00c1a4d0('GameScene'); i… |
| `00c29280` | Window::ShowModal | likely | Lua method body: guards argument 1 with 00c28980, then on the window calls the property interface at +0x1a0 virtual slot 0x14 with property 0xc and the default… |
| `00c29380` | Window::GetProperty | likely | Lua method body: requires exactly 2 arguments, guards argument 1 with 00c28980, resolves the property name (00e39c30 + 00820460) and, when the id is not negati… |

**(no class)** (1115)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00546301` | ReadPropertyMapping | confirmed | Reads one property-mapping record from the data node param_1 into the value written by FUN_004f6424. It first reads Flags through vtable +0x44 with the default… |
| `0054699f` | ReadPropertyMappingTable | confirmed | Loads a control's mapping table from the data node param_2: clears the two mapping containers (FUN_004212d3/FUN_004212ad), zeroes the read-only byte at this+0x… |
| `00546bb2` | ReadPropertyMappingsOnly | confirmed | Reduced version of the mapping-table reader: clears the mapping container (FUN_004212d3), zeros this+0x18, then for every child of PropertyMappings reads Prope… |
| `006074b2` | WriteComboBoxItemFlags | confirmed | Item writer: when FUN_004c6e70 passes it asks the object at this+0x14 (slot +0x30) for the item list and validates param_2 through slot +0x28; then it writes t… |
| `00c27150` | CheckLabelControl | confirmed | Shared argument guard for the LabelControl Lua methods: requires a userdata at the given stack index (checktype helper 00e39bb0 with tag 7), tests the 'LabelCo… |
| `00c27650` | CheckEditControl | confirmed | Shared argument guard for the EditControl Lua methods: requires a userdata at the given stack index (00e39bb0 with tag 7), tests the 'EditControl' metatable na… |
| `00c28090` | CheckButton | confirmed | Shared argument guard for the Button Lua methods: requires a userdata at the given stack index (00e39bb0 with tag 7), tests the 'Button' metatable name through… |
| `00c28980` | CheckWindow | confirmed | Shared argument guard for the Window Lua methods: requires a userdata at the given stack index (00e39bb0 with tag 7), tests the 'Window' metatable name through… |
| `016be0f3` | GfcControlProp_ColorLinkRollover | confirmed | Static initializer: list head from 004c886d; node 01bc4060 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be125` | GfcControlProp_ColorLinkInactive | confirmed | Static initializer: list head from 004c886d; node 01bc3f84 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be157` | GfcControlProp_ColorLinkInactiveSelect | confirmed | Static initializer: list head from 004c886d; node 01bc31d8 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be189` | GfcControlProp_ColorLinkInactiveRollover | confirmed | Static initializer: list head from 004c886d; node 01bc409c gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be1bb` | GfcControlProp_ColorLinkDisable | confirmed | Static initializer: list head from 004c886d; node 01bc3754 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be1ed` | GfcControlProp_ColorLinkDisableSelect | confirmed | Static initializer: list head from 004c886d; node 01bc42b4 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be21f` | GfcControlProp_ColorLinkDisableRollover | confirmed | Static initializer: list head from 004c886d; node 01bc3b04 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be251` | GfcControlProp_ColorImageDefault | confirmed | Static initializer: list head from 004c886d; node 01bc4264 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be283` | GfcControlProp_ColorImageSelect | confirmed | Static initializer: list head from 004c886d; node 01bc2e90 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be2b5` | GfcControlProp_ColorImageInactive | confirmed | Static initializer: list head from 004c886d; node 01bc2d98 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be2e7` | GfcControlProp_ColorImageInactiveSelect | confirmed | Static initializer: list head from 004c886d; node 01bc39b4 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be319` | GfcControlProp_ColorImageDisable | confirmed | Static initializer: list head from 004c886d; node 01bc2fb8 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be34b` | GfcControlProp_ColorImageDisableSelect | confirmed | Static initializer: list head from 004c886d; node 01bc43ec gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be37d` | GfcControlProp_ColorTopLayerFactor | confirmed | Static initializer: list head from 004c886d; node 01bc2954 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be3af` | GfcControlProp_ColorTopLayerColorKey | confirmed | Static initializer: list head from 004c886d; node 01bc3400 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be3e1` | GfcControlProp_ColorTopLayerColorKeyMask | confirmed | Static initializer: list head from 004c886d; node 01bc3624 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be413` | GfcControlProp_ColorBottomLayerFactor | confirmed | Static initializer: list head from 004c886d; node 01bc2b44 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be445` | GfcControlProp_ColorBottomLayerColorKey | confirmed | Static initializer: list head from 004c886d; node 01bc34e4 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be477` | GfcControlProp_ColorBottomLayerColorKeyMask | confirmed | Static initializer: list head from 004c886d; node 01bc2a88 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be4a9` | GfcControlProp_ColorHitTestLayerFactor | confirmed | Static initializer: list head from 004c886d; node 01bc433c gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be4db` | GfcControlProp_ColorHitTestLayerColorKey | confirmed | Static initializer: list head from 004c886d; node 01bc3214 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be50d` | GfcControlProp_ColorHitTestLayerColorKeyMask | confirmed | Static initializer: list head from 004c886d; node 01bc38e4 gets +4 type &DAT_0176026c, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be53f` | GfcControlProp_Combine | confirmed | Static initializer: list head from 004c886d; node 01bc3b70 gets +4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name, +0x10 old head; node becomes the list h… |
| `016be56e` | GfcControlProp_CombineBackground | confirmed | Static initializer: list head from 004c886d; node 01bc2f24 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be5a0` | GfcControlProp_CombineSurface | confirmed | Static initializer: list head from 004c886d; node 01bc2744 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be5d2` | GfcControlProp_CombineDialogInner | confirmed | Static initializer: list head from 004c886d; node 01bc2c78 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be604` | GfcControlProp_CombineInfo | confirmed | Static initializer: list head from 004c886d; node 01bc2a4c gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be636` | GfcControlProp_CombineToolTip | confirmed | Static initializer: list head from 004c886d; node 01bc4228 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be668` | GfcControlProp_CombineInnerDefault | confirmed | Static initializer: list head from 004c886d; node 01bc3584 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be69a` | GfcControlProp_CombineInnerSelect | confirmed | Static initializer: list head from 004c886d; node 01bc37e0 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be6cc` | GfcControlProp_CombineInnerInactive | confirmed | Static initializer: list head from 004c886d; node 01bc369c gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be6fe` | GfcControlProp_CombineInnerInactiveSelect | confirmed | Static initializer: list head from 004c886d; node 01bc40d8 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be730` | GfcControlProp_CombineInnerDisable | confirmed | Static initializer: list head from 004c886d; node 01bc3030 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be762` | GfcControlProp_CombineInnerDisableSelect | confirmed | Static initializer: list head from 004c886d; node 01bc3e0c gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be794` | GfcControlProp_CombineBorderDefault | confirmed | Static initializer: list head from 004c886d; node 01bc3e3c gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be7c6` | GfcControlProp_CombineBorderSelect | confirmed | Static initializer: list head from 004c886d; node 01bc3b2c gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be7f8` | GfcControlProp_CombineBorderInactive | confirmed | Static initializer: list head from 004c886d; node 01bc37f4 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be82a` | GfcControlProp_CombineBorderInactiveSelect | confirmed | Static initializer: list head from 004c886d; node 01bc27f8 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be85c` | GfcControlProp_CombineBorderDisable | confirmed | Static initializer: list head from 004c886d; node 01bc38bc gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be88e` | GfcControlProp_CombineBorderDisableSelect | confirmed | Static initializer: list head from 004c886d; node 01bc31a8 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be8c0` | GfcControlProp_CombineTextDefaultDest | confirmed | Static initializer: list head from 004c886d; node 01bc3b8c gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be8f2` | GfcControlProp_CombineTextDefaultFactor | confirmed | Static initializer: list head from 004c886d; node 01bc3180 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be924` | GfcControlProp_CombineTextSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bc35d4 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be956` | GfcControlProp_CombineTextSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bc3934 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be988` | GfcControlProp_CombineTextInactiveDest | confirmed | Static initializer: list head from 004c886d; node 01bc26e0 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be9ba` | GfcControlProp_CombineTextInactiveFactor | confirmed | Static initializer: list head from 004c886d; node 01bc3768 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016be9ec` | GfcControlProp_CombineTextInactiveSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bc343c gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bea1e` | GfcControlProp_CombineTextInactiveSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bc2c28 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bea50` | GfcControlProp_CombineTextDisableDest | confirmed | Static initializer: list head from 004c886d; node 01bc2f10 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bea82` | GfcControlProp_CombineTextDisableFactor | confirmed | Static initializer: list head from 004c886d; node 01bc3a3c gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016beab4` | GfcControlProp_CombineTextDisableSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bc2918 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016beae6` | GfcControlProp_CombineTextDisableSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bc27d0 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016beb18` | GfcControlProp_CombineTextShadowDefaultDest | confirmed | Static initializer: list head from 004c886d; node 01bc297c gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016beb4a` | GfcControlProp_CombineTextShadowDefaultFactor | confirmed | Static initializer: list head from 004c886d; node 01bc3970 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016beb7c` | GfcControlProp_CombineTextShadowSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bc30d8 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bebae` | GfcControlProp_CombineTextShadowSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bc2f74 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bebe0` | GfcControlProp_CombineTextShadowInactiveDest | confirmed | Static initializer: list head from 004c886d; node 01bc3c00 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bec12` | GfcControlProp_CombineTextShadowInactiveFactor | confirmed | Static initializer: list head from 004c886d; node 01bc3c94 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bec44` | GfcControlProp_CombineTextShadowInactiveSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bc3414 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bec76` | GfcControlProp_CombineTextShadowInactiveSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bc3724 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016beca8` | GfcControlProp_CombineTextShadowDisableDest | confirmed | Static initializer: list head from 004c886d; node 01bc32c4 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016becda` | GfcControlProp_CombineTextShadowDisableFactor | confirmed | Static initializer: list head from 004c886d; node 01bc26cc gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bed0c` | GfcControlProp_CombineTextShadowDisableSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bc2b6c gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bed3e` | GfcControlProp_CombineTextShadowDisableSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bc4278 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bed70` | GfcControlProp_CombineImageDefaultDest | confirmed | Static initializer: list head from 004c886d; node 01bc2864 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016beda2` | GfcControlProp_CombineImageDefaultFactor | confirmed | Static initializer: list head from 004c886d; node 01bc3688 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bedd4` | GfcControlProp_CombineImageSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bc3e64 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bee06` | GfcControlProp_CombineImageSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bc2fcc gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bee38` | GfcControlProp_CombineImageInactiveDest | confirmed | Static initializer: list head from 004c886d; node 01bc3660 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bee6a` | GfcControlProp_CombineImageInactiveFactor | confirmed | Static initializer: list head from 004c886d; node 01bc3520 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bee9c` | GfcControlProp_CombineImageInactiveSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bc3f38 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016beece` | GfcControlProp_CombineImageInactiveSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bc276c gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bef00` | GfcControlProp_CombineImageDisableDest | confirmed | Static initializer: list head from 004c886d; node 01bc404c gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bef32` | GfcControlProp_CombineImageDisableFactor | confirmed | Static initializer: list head from 004c886d; node 01bc34bc gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bef64` | GfcControlProp_CombineImageDisableSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bc4300 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bef96` | GfcControlProp_CombineImageDisableSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bc3ca8 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016befc8` | GfcControlProp_CombineTopLayerDest | confirmed | Static initializer: list head from 004c886d; node 01bc2b00 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016beffa` | GfcControlProp_CombineTopLayerFactor | confirmed | Static initializer: list head from 004c886d; node 01bc4314 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf02c` | GfcControlProp_CombineBottomLayerDest | confirmed | Static initializer: list head from 004c886d; node 01bc36e0 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf05e` | GfcControlProp_CombineBottomLayerFactor | confirmed | Static initializer: list head from 004c886d; node 01bc3f98 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf090` | GfcControlProp_CombineHitTestLayerDest | confirmed | Static initializer: list head from 004c886d; node 01bc29e0 gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf0c2` | GfcControlProp_CombineHitTestLayerFactor | confirmed | Static initializer: list head from 004c886d; node 01bc3ddc gets +4 type &DAT_01760b20, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf0f4` | GfcControlProp_Font | confirmed | Static initializer: list head from 004c886d; node 01bc38a8 gets +4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name, +0x10 old head; node becomes the list h… |
| `016bf123` | GfcControlProp_FontNormal | confirmed | Static initializer: list head from 004c886d; node 01bc29a4 gets +4 type &DAT_017614e0, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf155` | GfcControlProp_FontBold | confirmed | Static initializer: list head from 004c886d; node 01bc316c gets +4 type &DAT_017614e0, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf187` | GfcControlProp_FontItalic | confirmed | Static initializer: list head from 004c886d; node 01bc4164 gets +4 type &DAT_017614e0, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf1b9` | GfcControlProp_FontBoldItalic | confirmed | Static initializer: list head from 004c886d; node 01bc34d0 gets +4 type &DAT_017614e0, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf1eb` | GfcControlProp_FontHeader | confirmed | Static initializer: list head from 004c886d; node 01bc3790 gets +4 type &DAT_017614e0, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf21d` | GfcControlProp_FontHeaderBold | confirmed | Static initializer: list head from 004c886d; node 01bc2f60 gets +4 type &DAT_017614e0, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf24f` | GfcControlProp_FontFooter | confirmed | Static initializer: list head from 004c886d; node 01bc38d0 gets +4 type &DAT_017614e0, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf281` | GfcControlProp_FontFooterBold | confirmed | Static initializer: list head from 004c886d; node 01bc381c gets +4 type &DAT_017614e0, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf2b3` | GfcControlProp_FontFixed | confirmed | Static initializer: list head from 004c886d; node 01bc28dc gets +4 type &DAT_017614e0, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf2e5` | GfcControlProp_FontFixedBold | confirmed | Static initializer: list head from 004c886d; node 01bc3a04 gets +4 type &DAT_017614e0, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf317` | GfcControlProp_FlagBlitFxTopLayer | confirmed | Static initializer: list head from 004c886d; node 01bc3d8c gets +4 type &DAT_01760104 (bytes read u32), +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; … |
| `016bf349` | GfcControlProp_FlagBlitFxBottomLayer | confirmed | Static initializer: list head from 004c886d; node 01bc3088 gets +4 type &DAT_01760104 (bytes read u32), +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; … |
| `016bf37b` | GfcControlProp_FlagBlitFxHitTestLayer | confirmed | Static initializer: list head from 004c886d; node 01bc4388 gets +4 type &DAT_01760104 (bytes read u32), +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; … |
| `016bf3ad` | GfcControlProp_Icon | confirmed | Static initializer: list head from 004c886d; node 01bc3200 gets +4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name, +0x10 old head; node becomes the list h… |
| `016bf3dc` | GfcControlProp_IconFileUnknownSmall | confirmed | Static initializer: list head from 004c886d; node 01bc3ab4 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf40e` | GfcControlProp_IconFileUnknownMedium | confirmed | Static initializer: list head from 004c886d; node 01bc2ee0 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf440` | GfcControlProp_IconFileTextSmall | confirmed | Static initializer: list head from 004c886d; node 01bc3c14 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf472` | GfcControlProp_IconFileTextMedium | confirmed | Static initializer: list head from 004c886d; node 01bc3e78 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf4a4` | GfcControlProp_IconFileImageSmall | confirmed | Static initializer: list head from 004c886d; node 01bc3598 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf4d6` | GfcControlProp_IconFileImageMedium | confirmed | Static initializer: list head from 004c886d; node 01bc3d3c gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf508` | GfcControlProp_IconFolderOpenSmall | confirmed | Static initializer: list head from 004c886d; node 01bc4250 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf53a` | GfcControlProp_IconFolderOpenMedium | confirmed | Static initializer: list head from 004c886d; node 01bc32d8 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf56c` | GfcControlProp_IconFolderCloseSmall | confirmed | Static initializer: list head from 004c886d; node 01bc30b0 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf59e` | GfcControlProp_IconFolderCloseMedium | confirmed | Static initializer: list head from 004c886d; node 01bc2968 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf5d0` | GfcControlProp_IconFolderFilledOpenSmall | confirmed | Static initializer: list head from 004c886d; node 01bc2eb8 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf602` | GfcControlProp_IconFolderFilledOpenMedium | confirmed | Static initializer: list head from 004c886d; node 01bc3638 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf634` | GfcControlProp_IconFolderFilledCloseSmall | confirmed | Static initializer: list head from 004c886d; node 01bc2ea4 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf666` | GfcControlProp_IconFolderFilledCloseMedium | confirmed | Static initializer: list head from 004c886d; node 01bc3b54 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf698` | GfcControlProp_IconDriveFixedSmall | confirmed | Static initializer: list head from 004c886d; node 01bc2e68 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf6ca` | GfcControlProp_IconDriveFixedMedium | confirmed | Static initializer: list head from 004c886d; node 01bc2d70 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf6fc` | GfcControlProp_IconDriveRemovableSmall | confirmed | Static initializer: list head from 004c886d; node 01bc3f24 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf72e` | GfcControlProp_IconDriveRemovableMedium | confirmed | Static initializer: list head from 004c886d; node 01bc3710 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf760` | GfcControlProp_IconDriveOpticalSmall | confirmed | Static initializer: list head from 004c886d; node 01bc34a8 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf792` | GfcControlProp_IconDriveOpticalMedium | confirmed | Static initializer: list head from 004c886d; node 01bc311c gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf7c4` | GfcControlProp_IconDriveNetworkSmall | confirmed | Static initializer: list head from 004c886d; node 01bc2bd0 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf7f6` | GfcControlProp_IconDriveNetworkMedium | confirmed | Static initializer: list head from 004c886d; node 01bc2cb4 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf828` | GfcControlProp_IconComputerSmall | confirmed | Static initializer: list head from 004c886d; node 01bc2a24 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf85a` | GfcControlProp_IconComputerMedium | confirmed | Static initializer: list head from 004c886d; node 01bc3008 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf88c` | GfcControlProp_IconNewSmall | confirmed | Static initializer: list head from 004c886d; node 01bc4214 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf8be` | GfcControlProp_IconNewMedium | confirmed | Static initializer: list head from 004c886d; node 01bc26f4 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf8f0` | GfcControlProp_IconOpenSmall | confirmed | Static initializer: list head from 004c886d; node 01bc3610 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf922` | GfcControlProp_IconOpenMedium | confirmed | Static initializer: list head from 004c886d; node 01bc3380 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf954` | GfcControlProp_IconSaveSmall | confirmed | Static initializer: list head from 004c886d; node 01bc3c50 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf986` | GfcControlProp_IconSaveMedium | confirmed | Static initializer: list head from 004c886d; node 01bc3998 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf9b8` | GfcControlProp_IconSaveAllSmall | confirmed | Static initializer: list head from 004c886d; node 01bc2fa4 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bf9ea` | GfcControlProp_IconSaveAllMedium | confirmed | Static initializer: list head from 004c886d; node 01bc4024 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfa1c` | GfcControlProp_IconUndoSmall | confirmed | Static initializer: list head from 004c886d; node 01bc39dc gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfa4e` | GfcControlProp_IconUndoMedium | confirmed | Static initializer: list head from 004c886d; node 01bc2dc0 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfa80` | GfcControlProp_IconRedoSmall | confirmed | Static initializer: list head from 004c886d; node 01bc3e94 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfab2` | GfcControlProp_IconRedoMedium | confirmed | Static initializer: list head from 004c886d; node 01bc439c gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfae4` | GfcControlProp_IconCutSmall | confirmed | Static initializer: list head from 004c886d; node 01bc3dc8 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfb16` | GfcControlProp_IconCutMedium | confirmed | Static initializer: list head from 004c886d; node 01bc2e7c gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfb48` | GfcControlProp_IconCopySmall | confirmed | Static initializer: list head from 004c886d; node 01bc3f10 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfb7a` | GfcControlProp_IconCopyMedium | confirmed | Static initializer: list head from 004c886d; node 01bc3a70 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfbac` | GfcControlProp_IconPasteSmall | confirmed | Static initializer: list head from 004c886d; node 01bc3060 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfbde` | GfcControlProp_IconPasteMedium | confirmed | Static initializer: list head from 004c886d; node 01bc3f60 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfc10` | GfcControlProp_IconDeleteSmall | confirmed | Static initializer: list head from 004c886d; node 01bc2758 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfc42` | GfcControlProp_IconDeleteMedium | confirmed | Static initializer: list head from 004c886d; node 01bc2be4 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfc74` | GfcControlProp_IconDialogError | confirmed | Static initializer: list head from 004c886d; node 01bc2850 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfca6` | GfcControlProp_IconDialogWarning | confirmed | Static initializer: list head from 004c886d; node 01bc3d14 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfcd8` | GfcControlProp_IconDialogQuestion | confirmed | Static initializer: list head from 004c886d; node 01bc37cc gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfd0a` | GfcControlProp_IconDialogInformation | confirmed | Static initializer: list head from 004c886d; node 01bc2cf0 gets +4 type &DAT_01761670, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfd3c` | GfcControlProp_Cursor | confirmed | Static initializer: list head from 004c886d; node 01bc4128 gets +4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name, +0x10 old head; node becomes the list h… |
| `016bfd6b` | GfcControlProp_CursorDefault | confirmed | Static initializer: list head from 004c886d; node 01bc2ff4 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfd9d` | GfcControlProp_CursorCustom | confirmed | Static initializer: list head from 004c886d; node 01bc386c gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfdcf` | GfcControlProp_CursorNone | confirmed | Static initializer: list head from 004c886d; node 01bc3c28 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfe01` | GfcControlProp_CursorArrow | confirmed | Static initializer: list head from 004c886d; node 01bc3808 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfe33` | GfcControlProp_CursorReverseArrow | confirmed | Static initializer: list head from 004c886d; node 01bc27e4 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfe65` | GfcControlProp_CursorCross | confirmed | Static initializer: list head from 004c886d; node 01bc40ec gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfe97` | GfcControlProp_CursorNo | confirmed | Static initializer: list head from 004c886d; node 01bc3f4c gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfec9` | GfcControlProp_CursorText | confirmed | Static initializer: list head from 004c886d; node 01bc3674 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfefb` | GfcControlProp_CursorWait | confirmed | Static initializer: list head from 004c886d; node 01bc355c gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bff2d` | GfcControlProp_CursorHelp | confirmed | Static initializer: list head from 004c886d; node 01bc2aec gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bff5f` | GfcControlProp_CursorLink | confirmed | Static initializer: list head from 004c886d; node 01bc3d28 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bff91` | GfcControlProp_CursorDragCopy | confirmed | Static initializer: list head from 004c886d; node 01bc3a50 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bffc3` | GfcControlProp_CursorDragMove | confirmed | Static initializer: list head from 004c886d; node 01bc3880 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016bfff5` | GfcControlProp_CursorDragLink | confirmed | Static initializer: list head from 004c886d; node 01bc3cec gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0027` | GfcControlProp_CursorDragNone | confirmed | Static initializer: list head from 004c886d; node 01bc41e4 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0059` | GfcControlProp_CursorUp | confirmed | Static initializer: list head from 004c886d; node 01bc3ba0 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c008b` | GfcControlProp_CursorDown | confirmed | Static initializer: list head from 004c886d; node 01bc30c4 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c00bd` | GfcControlProp_CursorRight | confirmed | Static initializer: list head from 004c886d; node 01bc3300 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c00ef` | GfcControlProp_CursorLeft | confirmed | Static initializer: list head from 004c886d; node 01bc2780 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0121` | GfcControlProp_CursorVSplit | confirmed | Static initializer: list head from 004c886d; node 01bc2bbc gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0153` | GfcControlProp_CursorHSplit | confirmed | Static initializer: list head from 004c886d; node 01bc37b8 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0185` | GfcControlProp_CursorSizeAll | confirmed | Static initializer: list head from 004c886d; node 01bc2b1c gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c01b7` | GfcControlProp_CursorSizeUp | confirmed | Static initializer: list head from 004c886d; node 01bc43c4 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c01e9` | GfcControlProp_CursorSizeDown | confirmed | Static initializer: list head from 004c886d; node 01bc350c gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c021b` | GfcControlProp_CursorSizeRight | confirmed | Static initializer: list head from 004c886d; node 01bc28b4 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c024d` | GfcControlProp_CursorSizeLeft | confirmed | Static initializer: list head from 004c886d; node 01bc309c gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c027f` | GfcControlProp_CursorSizeUpDown | confirmed | Static initializer: list head from 004c886d; node 01bc4010 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c02b1` | GfcControlProp_CursorSizeLeftRight | confirmed | Static initializer: list head from 004c886d; node 01bc4400 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c02e3` | GfcControlProp_CursorSizeUpLeft | confirmed | Static initializer: list head from 004c886d; node 01bc35ac gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0315` | GfcControlProp_CursorSizeUpRight | confirmed | Static initializer: list head from 004c886d; node 01bc39c8 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0347` | GfcControlProp_CursorSizeDownLeft | confirmed | Static initializer: list head from 004c886d; node 01bc3fc0 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0379` | GfcControlProp_CursorSizeDownRight | confirmed | Static initializer: list head from 004c886d; node 01bc2d5c gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c03ab` | GfcControlProp_CursorMoveNormal | confirmed | Static initializer: list head from 004c886d; node 01bc2828 gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c03dd` | GfcControlProp_CursorMoveVertical | confirmed | Static initializer: list head from 004c886d; node 01bc292c gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c040f` | GfcControlProp_CursorMoveHorizontal | confirmed | Static initializer: list head from 004c886d; node 01bc3eec gets +4 type &DAT_01761d14, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0441` | GfcControlProp_Text | confirmed | Static initializer: list head from 004c886d; node 01bc2d34 gets +4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name, +0x10 old head; node becomes the list h… |
| `016c0470` | GfcControlProp_TextCropDots | confirmed | Static initializer: list head from 004c886d; node 01bc3044 gets +4 type &DAT_0175d3a4 (p<c GString>), +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; no… |
| `016c04a2` | GfcControlProp_Binding | confirmed | Static initializer: list head from 004c886d; node 01bc3260 gets +4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name, +0x10 old head; node becomes the list h… |
| `016c04d1` | GfcControlProp_BindingFocusNext | confirmed | Static initializer: list head from 004c886d; node 01bc40b0 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0503` | GfcControlProp_BindingFocusPrevious | confirmed | Static initializer: list head from 004c886d; node 01bc3ed8 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0535` | GfcControlProp_BindingFocusLeft | confirmed | Static initializer: list head from 004c886d; node 01bc423c gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0567` | GfcControlProp_BindingFocusRight | confirmed | Static initializer: list head from 004c886d; node 01bc3328 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0599` | GfcControlProp_BindingFocusUp | confirmed | Static initializer: list head from 004c886d; node 01bc3c64 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c05cb` | GfcControlProp_BindingFocusDown | confirmed | Static initializer: list head from 004c886d; node 01bc4100 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c05fd` | GfcControlProp_BindingMoveBegin | confirmed | Static initializer: list head from 004c886d; node 01bc3314 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c062f` | GfcControlProp_BindingMoveEnd | confirmed | Static initializer: list head from 004c886d; node 01bc2f88 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0661` | GfcControlProp_BindingMoveCancel | confirmed | Static initializer: list head from 004c886d; node 01bc34f8 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0693` | GfcControlProp_BindingMoveLeft | confirmed | Static initializer: list head from 004c886d; node 01bc283c gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c06c5` | GfcControlProp_BindingMoveRight | confirmed | Static initializer: list head from 004c886d; node 01bc4328 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c06f7` | GfcControlProp_BindingMoveUp | confirmed | Static initializer: list head from 004c886d; node 01bc2730 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0729` | GfcControlProp_BindingMoveDown | confirmed | Static initializer: list head from 004c886d; node 01bc2c50 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c075b` | GfcControlProp_BindingMoveUpLeft | confirmed | Static initializer: list head from 004c886d; node 01bc38f8 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c078d` | GfcControlProp_BindingMoveUpRight | confirmed | Static initializer: list head from 004c886d; node 01bc3428 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c07bf` | GfcControlProp_BindingMoveDownLeft | confirmed | Static initializer: list head from 004c886d; node 01bc41a0 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c07f1` | GfcControlProp_BindingMoveDownRight | confirmed | Static initializer: list head from 004c886d; node 01bc3db4 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0823` | GfcControlProp_BindingMoveConstrainOn | confirmed | Static initializer: list head from 004c886d; node 01bc2794 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0855` | GfcControlProp_BindingMoveConstrainOff | confirmed | Static initializer: list head from 004c886d; node 01bc31bc gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0887` | GfcControlProp_BindingMoveUnitOn | confirmed | Static initializer: list head from 004c886d; node 01bc35e8 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c08b9` | GfcControlProp_BindingMoveUnitOff | confirmed | Static initializer: list head from 004c886d; node 01bc428c gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c08eb` | GfcControlProp_BindingMoveSnapToOn | confirmed | Static initializer: list head from 004c886d; node 01bc29cc gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c091d` | GfcControlProp_BindingMoveSnapToOff | confirmed | Static initializer: list head from 004c886d; node 01bc2ef4 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c094f` | GfcControlProp_BindingSizeBegin | confirmed | Static initializer: list head from 004c886d; node 01bc3830 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0981` | GfcControlProp_BindingSizeEnd | confirmed | Static initializer: list head from 004c886d; node 01bc3d00 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c09b3` | GfcControlProp_BindingSizeCancel | confirmed | Static initializer: list head from 004c886d; node 01bc3144 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c09e5` | GfcControlProp_BindingSizeLeft | confirmed | Static initializer: list head from 004c886d; node 01bc3858 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0a17` | GfcControlProp_BindingSizeRight | confirmed | Static initializer: list head from 004c886d; node 01bc288c gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0a49` | GfcControlProp_BindingSizeUp | confirmed | Static initializer: list head from 004c886d; node 01bc41f8 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0a7b` | GfcControlProp_BindingSizeDown | confirmed | Static initializer: list head from 004c886d; node 01bc3228 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0aad` | GfcControlProp_BindingSizeUpLeft | confirmed | Static initializer: list head from 004c886d; node 01bc3548 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0adf` | GfcControlProp_BindingSizeUpRight | confirmed | Static initializer: list head from 004c886d; node 01bc329c gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0b11` | GfcControlProp_BindingSizeDownLeft | confirmed | Static initializer: list head from 004c886d; node 01bc41b4 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0b43` | GfcControlProp_BindingSizeDownRight | confirmed | Static initializer: list head from 004c886d; node 01bc3274 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0b75` | GfcControlProp_BindingSizeComboOn | confirmed | Static initializer: list head from 004c886d; node 01bc39f0 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016c0ba7` | GfcControlProp_BindingSizeComboOff | confirmed | Static initializer: list head from 004c886d; node 01bc3534 gets +4 type &DAT_01762160, +8 owner GFC_Control_Theme, +0xc name, +0x10 old head; node becomes the … |
| `016d85d4` | GfcControlProp_ButtonColorTextShadowFocusPress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8606` | GfcControlProp_ButtonColorTextShadowFocusDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8638` | GfcControlProp_ButtonColorTextShadowFocusRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d866a` | GfcControlProp_ButtonColorTextShadowToggle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d869c` | GfcControlProp_ButtonColorTextShadowTogglePress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d86ce` | GfcControlProp_ButtonColorTextShadowToggleDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8700` | GfcControlProp_ButtonColorTextShadowToggleRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8732` | GfcControlProp_ButtonColorTextShadowToggleFocus | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8764` | GfcControlProp_ButtonColorTextShadowToggleFocusPress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8796` | GfcControlProp_ButtonColorTextShadowToggleFocusDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d87c8` | GfcControlProp_ButtonColorTextShadowToggleFocusRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d87fa` | GfcControlProp_ButtonColorTextShadowInactive | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d882c` | GfcControlProp_ButtonColorTextShadowInactivePress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d885e` | GfcControlProp_ButtonColorTextShadowInactiveDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8890` | GfcControlProp_ButtonColorTextShadowInactiveRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d88c2` | GfcControlProp_ButtonColorTextShadowInactiveToggle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d88f4` | GfcControlProp_ButtonColorTextShadowInactiveTogglePress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8926` | GfcControlProp_ButtonColorTextShadowInactiveToggleDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8958` | GfcControlProp_ButtonColorTextShadowInactiveToggleRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d898a` | GfcControlProp_ButtonColorTextShadowWinoff | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d89bc` | GfcControlProp_ButtonColorTextShadowWinoffPress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d89ee` | GfcControlProp_ButtonColorTextShadowWinoffDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8a20` | GfcControlProp_ButtonColorTextShadowWinoffRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8a52` | GfcControlProp_ButtonColorTextShadowWinoffToggle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8a84` | GfcControlProp_ButtonColorTextShadowWinoffTogglePress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8ab6` | GfcControlProp_ButtonColorTextShadowWinoffToggleDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8ae8` | GfcControlProp_ButtonColorTextShadowWinoffToggleRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016d8b1a` | GfcControlProp_ButtonCombine | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016d8b49` | GfcControlProp_ButtonCombineInnerDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8b7b` | GfcControlProp_ButtonCombineInnerFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8bad` | GfcControlProp_ButtonCombineHighlightDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8bdf` | GfcControlProp_ButtonCombineHighlightFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8c11` | GfcControlProp_ButtonCombineTextDefaultDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8c43` | GfcControlProp_ButtonCombineTextDefaultFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8c75` | GfcControlProp_ButtonCombineTextDisableDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8ca7` | GfcControlProp_ButtonCombineTextDisableFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8cd9` | GfcControlProp_ButtonCombineTextShadowDefaultDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8d0b` | GfcControlProp_ButtonCombineTextShadowDefaultFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8d3d` | GfcControlProp_ButtonCombineTextShadowDisableDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8d6f` | GfcControlProp_ButtonCombineTextShadowDisableFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8da1` | GfcControlProp_ButtonCombineImageDefaultDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8dd3` | GfcControlProp_ButtonCombineImageDefaultFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8e05` | GfcControlProp_ButtonCombineImageDisableDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8e37` | GfcControlProp_ButtonCombineImageDisableFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016d8e69` | GfcControlProp_ButtonCursor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016d8e98` | GfcControlProp_ButtonCursorPress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d8eca` | GfcControlProp_ButtonCursorTracking | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d8efc` | GfcControlProp_ButtonCursorDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d8f2e` | GfcControlProp_ButtonCursorToggle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d8f60` | GfcControlProp_ButtonCursorTogglePress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d8f92` | GfcControlProp_ButtonCursorToggleTracking | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d8fc4` | GfcControlProp_ButtonImage | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016d8ff3` | GfcControlProp_ButtonImageDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d9025` | GfcControlProp_ButtonImagePress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d9057` | GfcControlProp_ButtonImageDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d9089` | GfcControlProp_ButtonImageRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d90bb` | GfcControlProp_ButtonImageFocus | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d90ed` | GfcControlProp_ButtonImageFocusPress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d911f` | GfcControlProp_ButtonImageFocusDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d9151` | GfcControlProp_ButtonImageFocusRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d9183` | GfcControlProp_ButtonImageToggle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d91b5` | GfcControlProp_ButtonImageTogglePress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d91e7` | GfcControlProp_ButtonImageToggleDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d9219` | GfcControlProp_ButtonImageToggleRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d924b` | GfcControlProp_ButtonImageToggleFocus | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d927d` | GfcControlProp_ButtonImageToggleFocusPress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d92af` | GfcControlProp_ButtonImageToggleFocusDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d92e1` | GfcControlProp_ButtonImageToggleFocusRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d9313` | GfcControlProp_ButtonImageInactive | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d9345` | GfcControlProp_ButtonImageInactivePress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d9377` | GfcControlProp_ButtonImageInactiveDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d93a9` | GfcControlProp_ButtonImageInactiveRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d93db` | GfcControlProp_ButtonImageInactiveToggle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d940d` | GfcControlProp_ButtonImageInactiveTogglePress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d943f` | GfcControlProp_ButtonImageInactiveToggleDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d9471` | GfcControlProp_ButtonImageInactiveToggleRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d94a3` | GfcControlProp_ButtonImageWinoff | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d94d5` | GfcControlProp_ButtonImageWinoffPress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d9507` | GfcControlProp_ButtonImageWinoffDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d9539` | GfcControlProp_ButtonImageWinoffRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d956b` | GfcControlProp_ButtonImageWinoffToggle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d959d` | GfcControlProp_ButtonImageWinoffTogglePress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d95cf` | GfcControlProp_ButtonImageWinoffToggleDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d9601` | GfcControlProp_ButtonImageWinoffToggleRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBitmap>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016d9633` | GfcControlProp_ButtonBinding | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016d9662` | GfcControlProp_ButtonBindingButtonPress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016d9694` | GfcControlProp_ButtonBindingButtonRelease | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016d96c6` | GfcControlProp_ButtonBindingButtonExecute | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016d96f8` | GfcControlProp_ButtonBindingButtonCancel | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016d972a` | GfcControlProp_ButtonBindingButtonCommit | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016d975c` | GfcControlProp_ButtonBindingButtonEnd | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016d978e` | GfcControlProp_ButtonTimer | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016d97bd` | GfcControlProp_ButtonTimerHoldTimerFirst | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d97ec` | GfcControlProp_ButtonTimerHoldTimerRepeat | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d981b` | GfcControlProp_ButtonTimerAnimate | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d984a` | GfcControlProp_ButtonMessage | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016d9879` | GfcControlProp_ButtonMessageOnExecute | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d98a8` | GfcControlProp_ButtonMessageOnUIExecute | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d98d7` | GfcControlProp_ButtonMessageOnHoldTimer | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9906` | GfcControlProp_ButtonMessageOnKeyPress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9935` | GfcControlProp_ButtonMessageOnMousePress | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9964` | GfcControlProp_ButtonMessageOnKeyRelease | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9993` | GfcControlProp_ButtonMessageOnMouseRelease | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d99c2` | GfcControlProp_ButtonMessageOnToggle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d99f1` | GfcControlProp_ButtonMessageOnUIToggle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9a20` | GfcControlProp_ButtonMessageOnStateChange | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9a4f` | GfcControlProp_ButtonMessageOnUIStateChange | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9a7e` | GfcControlProp_ButtonMessageOnTextChange | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9aad` | GfcControlProp_ButtonMessageOnCancelTracking | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9adc` | GfcControlProp_ButtonMessageOnUICancelTracking | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9b0b` | GfcControlProp_ButtonMessageOnCommitTracking | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9b3a` | GfcControlProp_ButtonMessageOnUICommitTracking | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9b69` | GfcControlProp_ButtonMessageOnEndTracking | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9b98` | GfcControlProp_ButtonMessageOnUIEndTracking | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9bc7` | GfcControlProp_ButtonMessageDoExec | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9bf6` | GfcControlProp_ButtonMessageDoHit | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9c25` | GfcControlProp_ButtonMessageDoSetToggle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9c54` | GfcControlProp_ButtonMessageDoSetState | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9c83` | GfcControlProp_ButtonMessageDoCancelTracking | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9cb2` | GfcControlProp_ButtonMessageDoCommitTracking | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9ce1` | GfcControlProp_ButtonMessageDoEndTracking | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9d10` | GfcControlProp_ButtonMessageDoSetText | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9d3f` | GfcControlProp_ButtonMessageDoUpdateText | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175f744 (rdata 'v'), node+8 = 0 (no owner), node+0x10 = old head; nod… |
| `016d9d7e` | GfcControlRegister_GButtonCtrl | confirmed | Thiscall registration: ECX = node 01bd60a4, pushes flag 1, the style-table pointer 01b1cb08 (descriptor 01b1cacc+0x3c) and the name 'GButtonCtrl', then calls 0… |
| `016d9d95` | GfcControlProp_StandardCheckBox | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; … |
| `016d9dc7` | GfcControlProp_StandardCheckBoxStyle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyInterface>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; … |
| `016d9e19` | GfcControlProp_StandardRadioButton | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; … |
| `016d9e4b` | GfcControlProp_StandardRadioButtonStyle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyInterface>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; … |
| `016d9e7d` | GfcControlProp_ComboBox | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016d9eac` | GfcControlProp_StandardComboBoxStyle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyInterface>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; … |
| `016d9ede` | GfcControlProp_EditComboBoxStyle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyInterface>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; … |
| `016d9f10` | GfcControlProp_FlatComboBoxStyle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyInterface>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; … |
| `016d9f42` | GfcControlProp_DefaultComboBoxStyle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyInterface>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; … |
| `016d9f74` | GfcControlProp_ComboBoxFlag | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016d9fa3` | GfcControlProp_ComboBoxFlagNoBorder | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016d9fd5` | GfcControlProp_ComboBoxFlagNoInnerBorder | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da007` | GfcControlProp_ComboBoxFlagNoInnerFill | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da039` | GfcControlProp_ComboBoxFlagSelectOnShow | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da06b` | GfcControlProp_ComboBoxFlagPopupVStyle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da09d` | GfcControlProp_ComboBoxFlagComboBoxMaskInvalidateControl | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da0cf` | GfcControlProp_ComboBoxRenderer | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016da0fe` | GfcControlProp_ComboBoxRendererTextItem | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GComboBoxTextItemRenderer>', node+8 = owner GFC_Control_Theme, node+0x10 = old… |
| `016da130` | GfcControlProp_ComboBoxSize | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016da191` | GfcControlProp_ComboBoxSizePopupMinimum | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GSize', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes hea… |
| `016da2bd` | GfcControlProp_ComboBoxSizeItemSpace | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da2ef` | GfcControlProp_ComboBoxSizeTextMinimum | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da353` | GfcControlProp_ComboBoxSizeImage | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GSize', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes hea… |
| `016da385` | GfcControlProp_ComboBoxSizeImageColumn | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da41b` | GfcControlProp_ComboBoxSizeDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GSize', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes hea… |
| `016da44d` | GfcControlProp_ComboBoxSizeCenterPopupMaxScreenPercent | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da47f` | GfcControlProp_ComboBoxSizeCenterPopupMaxItemCount | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da4b1` | GfcControlProp_ComboBoxSizeCenterPopupMinScrollScreenPercent | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da4e3` | GfcControlProp_ComboBoxSizeCenterPopupMinScrollItemCount | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da515` | GfcControlProp_ComboBoxSizeUpDownPopupMaxScreenPercent | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da547` | GfcControlProp_ComboBoxSizeUpDownPopupMaxItemCount | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da579` | GfcControlProp_ComboBoxSizeUpDownPopupMinScrollScreenPercent | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da5ab` | GfcControlProp_ComboBoxSizeUpDownPopupMinScrollItemCount | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da5dd` | GfcControlProp_ComboBoxDelay | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016da60c` | GfcControlProp_ComboBoxDelayIgnoreSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016da63e` | GfcControlProp_ComboBoxChild | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016da66d` | GfcControlProp_ComboBoxChildOuterBorder | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBorderCtrl>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node be… |
| `016da69f` | GfcControlProp_ComboBoxChildInnerBorder | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBorderCtrl>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node be… |
| `016da6d1` | GfcControlProp_ComboBoxChildItemEdit | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GEditCtrl>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node beco… |
| `016da703` | GfcControlProp_ComboBoxChildPopupButton | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GButtonCtrl>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node be… |
| `016da735` | GfcControlProp_ComboBoxChildPopupBoxTable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GTableCtrl>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016da767` | GfcControlProp_ComboBoxStyleId | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016da796` | GfcControlProp_ComboBoxStyleIdOuterBorder | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GPropertyId', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016da7c8` | GfcControlProp_ComboBoxStyleIdInnerBorder | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GPropertyId', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016da7fa` | GfcControlProp_ComboBoxStyleIdItemEdit | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GPropertyId', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016da82c` | GfcControlProp_ComboBoxStyleIdPopupButton | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GPropertyId', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016da85e` | GfcControlProp_ComboBoxStyleIdPopupBoxTable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GPropertyId', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016da890` | GfcControlProp_ComboBoxColor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016da8bf` | GfcControlProp_ComboBoxColorViewItemInnerDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016da8f1` | GfcControlProp_ComboBoxColorViewItemInnerSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016da923` | GfcControlProp_ComboBoxColorViewItemInnerRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016da955` | GfcControlProp_ComboBoxColorViewItemInnerDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016da987` | GfcControlProp_ComboBoxColorViewItemImageDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016da9b9` | GfcControlProp_ComboBoxColorViewItemImageSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016da9eb` | GfcControlProp_ComboBoxColorViewItemImageRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016daa1d` | GfcControlProp_ComboBoxColorViewItemImageDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016daa4f` | GfcControlProp_ComboBoxColorViewItemTextDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016daa81` | GfcControlProp_ComboBoxColorViewItemTextSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016daab3` | GfcControlProp_ComboBoxColorViewItemTextRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016daae5` | GfcControlProp_ComboBoxColorViewItemTextDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dab17` | GfcControlProp_ComboBoxColorViewItemTextShadowDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dab49` | GfcControlProp_ComboBoxColorViewItemTextShadowSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dab7b` | GfcControlProp_ComboBoxColorViewItemTextShadowRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dabad` | GfcControlProp_ComboBoxColorViewItemTextShadowDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dabdf` | GfcControlProp_ComboBoxColorPopupItemInnerDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dac11` | GfcControlProp_ComboBoxColorPopupItemInnerSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dac43` | GfcControlProp_ComboBoxColorPopupItemInnerDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dac75` | GfcControlProp_ComboBoxColorPopupItemInnerDisableSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016daca7` | GfcControlProp_ComboBoxColorPopupItemTextDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dacd9` | GfcControlProp_ComboBoxColorPopupItemTextSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dad0b` | GfcControlProp_ComboBoxColorPopupItemTextDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dad3d` | GfcControlProp_ComboBoxColorPopupItemTextDisableSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dad6f` | GfcControlProp_ComboBoxColorPopupItemTextShadowDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dada1` | GfcControlProp_ComboBoxColorPopupItemTextShadowSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dadd3` | GfcControlProp_ComboBoxColorPopupItemTextShadowDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dae05` | GfcControlProp_ComboBoxColorPopupItemTextShadowDisableSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dae37` | GfcControlProp_ComboBoxColorPopupItemImageDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dae69` | GfcControlProp_ComboBoxColorPopupItemImageSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016dae9b` | GfcControlProp_ComboBoxColorPopupItemImageDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016daecd` | GfcControlProp_ComboBoxColorPopupItemImageDisableSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016daeff` | GfcControlProp_ComboBoxCombine | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016daf2e` | GfcControlProp_ComboBoxCombineViewItemInnerDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016daf60` | GfcControlProp_ComboBoxCombineViewItemInnerSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016daf92` | GfcControlProp_ComboBoxCombineViewItemInnerRollover | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016dafc4` | GfcControlProp_ComboBoxCombineViewItemInnerDisable | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016daff6` | GfcControlProp_ComboBoxCombineViewItemImageDefaultDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db028` | GfcControlProp_ComboBoxCombineViewItemImageDefaultFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db05a` | GfcControlProp_ComboBoxCombineViewItemImageSelectDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db08c` | GfcControlProp_ComboBoxCombineViewItemImageSelectFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db0be` | GfcControlProp_ComboBoxCombineViewItemImageRolloverDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db0f0` | GfcControlProp_ComboBoxCombineViewItemImageRolloverFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db122` | GfcControlProp_ComboBoxCombineViewItemImageDisableDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db154` | GfcControlProp_ComboBoxCombineViewItemImageDisableFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db186` | GfcControlProp_ComboBoxCombineViewItemTextDefaultDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db1b8` | GfcControlProp_ComboBoxCombineViewItemTextDefaultFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db1ea` | GfcControlProp_ComboBoxCombineViewItemTextSelectDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db21c` | GfcControlProp_ComboBoxCombineViewItemTextSelectFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db24e` | GfcControlProp_ComboBoxCombineViewItemTextRolloverDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db280` | GfcControlProp_ComboBoxCombineViewItemTextRolloverFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db2b2` | GfcControlProp_ComboBoxCombineViewItemTextDisableDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db2e4` | GfcControlProp_ComboBoxCombineViewItemTextDisableFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db316` | GfcControlProp_ComboBoxCombineViewItemTextShadowDefaultDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db348` | GfcControlProp_ComboBoxCombineViewItemTextShadowDefaultFact… | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db37a` | GfcControlProp_ComboBoxCombineViewItemTextShadowSelectDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db3ac` | GfcControlProp_ComboBoxCombineViewItemTextShadowSelectFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db3de` | GfcControlProp_ComboBoxCombineViewItemTextShadowRolloverDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db410` | GfcControlProp_ComboBoxCombineViewItemTextShadowRolloverFac… | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db442` | GfcControlProp_ComboBoxCombineViewItemTextShadowDisableDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db474` | GfcControlProp_ComboBoxCombineViewItemTextShadowDisableFact… | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db4a6` | GfcControlProp_ComboBoxCombinePopupItemInnerDefault | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db4d8` | GfcControlProp_ComboBoxCombinePopupItemInnerSelect | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db50a` | GfcControlProp_ComboBoxCombinePopupItemInnerDisable | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db53c` | GfcControlProp_ComboBoxCombinePopupItemInnerDisableSelect | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db56e` | GfcControlProp_ComboBoxCombinePopupItemTextDefaultDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db5a0` | GfcControlProp_ComboBoxCombinePopupItemTextDefaultFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db5d2` | GfcControlProp_ComboBoxCombinePopupItemTextSelectDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db604` | GfcControlProp_ComboBoxCombinePopupItemTextSelectFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db636` | GfcControlProp_ComboBoxCombinePopupItemTextDisableDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db668` | GfcControlProp_ComboBoxCombinePopupItemTextDisableFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db69a` | GfcControlProp_ComboBoxCombinePopupItemTextDisableSelectDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db6cc` | GfcControlProp_ComboBoxCombinePopupItemTextDisableSelectFac… | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db6fe` | GfcControlProp_ComboBoxCombinePopupItemTextShadowDefaultDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db730` | GfcControlProp_ComboBoxCombinePopupItemTextShadowDefaultFac… | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db762` | GfcControlProp_ComboBoxCombinePopupItemTextShadowSelectDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db794` | GfcControlProp_ComboBoxCombinePopupItemTextShadowSelectFact… | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db7c6` | GfcControlProp_ComboBoxCombinePopupItemTextShadowDisableDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db7f8` | GfcControlProp_ComboBoxCombinePopupItemTextShadowDisableFac… | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db82a` | GfcControlProp_ComboBoxCombinePopupItemTextShadowDisableSel… | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db85c` | GfcControlProp_ComboBoxCombinePopupItemTextShadowDisableSel… | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db88e` | GfcControlProp_ComboBoxCombinePopupItemImageDefaultDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db8c0` | GfcControlProp_ComboBoxCombinePopupItemImageDefaultFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db8f2` | GfcControlProp_ComboBoxCombinePopupItemImageSelectDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db924` | GfcControlProp_ComboBoxCombinePopupItemImageSelectFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db956` | GfcControlProp_ComboBoxCombinePopupItemImageDisableDest | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db988` | GfcControlProp_ComboBoxCombinePopupItemImageDisableFactor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db9ba` | GfcControlProp_ComboBoxCombinePopupItemImageDisableSelectDe… | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016db9ec` | GfcControlProp_ComboBoxCombinePopupItemImageDisableSelectFa… | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColorCombine' at node+4 and owner 'GFC_Control_Theme'… |
| `016dba1e` | GfcControlProp_ComboBoxFont | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the name string at node+0xc and type 'p<c GPropertyNamespace>' at node+4, then… |
| `016dba4d` | GfcControlProp_ComboBoxFontDefault | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GFont>' at node+4 and owner 'GFC_Control_Theme' at n… |
| `016dba7f` | GfcControlProp_ComboBoxFontSelect | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GFont>' at node+4 and owner 'GFC_Control_Theme' at n… |
| `016dbab1` | GfcControlProp_ComboBoxBinding | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the name string at node+0xc and type 'p<c GPropertyNamespace>' at node+4, then… |
| `016dbae0` | GfcControlProp_ComboBoxBindingCommitItem | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GInputBinding>' at node+4 and owner 'GFC_Control_The… |
| `016dbb12` | GfcControlProp_ComboBoxBindingShowPopup | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GInputBinding>' at node+4 and owner 'GFC_Control_The… |
| `016dbb44` | GfcControlProp_ComboBoxBindingCursorShowPopup | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GInputBinding>' at node+4 and owner 'GFC_Control_The… |
| `016dbb76` | GfcControlProp_ComboBoxBindingExecute | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GInputBinding>' at node+4 and owner 'GFC_Control_The… |
| `016dbba8` | GfcControlProp_ComboBoxBindingCancel | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GInputBinding>' at node+4 and owner 'GFC_Control_The… |
| `016dbbda` | GfcControlProp_ComboBoxMessage | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the name string at node+0xc and type 'p<c GPropertyNamespace>' at node+4, then… |
| `016dbc09` | GfcControlProp_ComboBoxMessageOnShowPopup | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbc38` | GfcControlProp_ComboBoxMessageOnUIShowPopup | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbc67` | GfcControlProp_ComboBoxMessageOnPopupSelect | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbc96` | GfcControlProp_ComboBoxMessageOnUIPopupSelect | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbcc5` | GfcControlProp_ComboBoxMessageOnItemSelect | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbcf4` | GfcControlProp_ComboBoxMessageOnUIItemSelect | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbd23` | GfcControlProp_ComboBoxMessageOnItemChange | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbd52` | GfcControlProp_ComboBoxMessageOnUIItemChange | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbd81` | GfcControlProp_ComboBoxMessageOnCommitItem | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbdb0` | GfcControlProp_ComboBoxMessageOnTriggerPopup | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbddf` | GfcControlProp_ComboBoxMessageOnPopupButtonStateChange | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbe0e` | GfcControlProp_ComboBoxMessageTableOnKeyDown | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbe3d` | GfcControlProp_ComboBoxMessageTableOnKeyUp | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbe6c` | GfcControlProp_ComboBoxMessageTableOnKeyChar | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbe9b` | GfcControlProp_ComboBoxMessageTableOnNavDown | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbeca` | GfcControlProp_ComboBoxMessageDoShowPopup | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbef9` | GfcControlProp_ComboBoxMessageDoHitPopupItem | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbf28` | GfcControlProp_ComboBoxMessageDoSetCurrentItem | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbf57` | GfcControlProp_ComboBoxMessageDoSelectPopupItem | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbf86` | GfcControlProp_ComboBoxMessageDoSelectItem | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbfb5` | GfcControlProp_ComboBoxMessageDoSelectPopupNext | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dbfe4` | GfcControlProp_ComboBoxMessageDoSelectPopupPrev | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dc013` | GfcControlProp_ComboBoxMessageDoSelectItemNext | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dc042` | GfcControlProp_ComboBoxMessageDoSelectItemPrev | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dc071` | GfcControlProp_ComboBoxMessageDoCommitItem | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dc0a0` | GfcControlProp_ComboBoxMessageDoEndTracking | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dc0cf` | GfcControlProp_ComboBoxTimer | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the name string at node+0xc and type 'p<c GPropertyNamespace>' at node+4, then… |
| `016dc0fe` | GfcControlProp_ComboBoxTimerIgnoreSelect | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_0175f744 at node+4 and the name string at node+0xc, the… |
| `016dc12d` | GfcControlProp_ComboBoxFilter | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the name string at node+0xc and type 'p<c GPropertyNamespace>' at node+4, then… |
| `016dc15c` | GfcControlProp_ComboBoxFilterExternalClickDown | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_017abe88 (the .rdata string 'p<v>') at node+4 and the n… |
| `016dc18b` | GfcControlProp_ComboBoxFilterExternalClickUp | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_017abe88 (the .rdata string 'p<v>') at node+4 and the n… |
| `016dc1ba` | GfcControlProp_ComboBoxFilterExternalModalNotify | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_017abe88 (the .rdata string 'p<v>') at node+4 and the n… |
| `016dc1e9` | GfcControlProp_ComboBoxFilterExternalActivate | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, the shared record &DAT_017abe88 (the .rdata string 'p<v>') at node+4 and the n… |
| `016dc31f` | GfcControlProp_EditClipboardTextFormat | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, type 'p<c GRangePropertyMap>' at node+4 and the name string at node+0xc, then … |
| `016dc34e` | GfcControlProp_Edit | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, type 'p<c GPropertyNamespace>' at node+4 and the name string at node+0xc, then… |
| `016dc37d` | GfcControlProp_StandardEditStyle | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GPropertyInterface>' at node+4 and owner 'GFC_Contro… |
| `016dc3af` | GfcControlProp_FlatEditStyle | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GPropertyInterface>' at node+4 and owner 'GFC_Contro… |
| `016dc3e1` | GfcControlProp_SolidEditStyle | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GPropertyInterface>' at node+4 and owner 'GFC_Contro… |
| `016dc413` | GfcControlProp_ClearEditStyle | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GPropertyInterface>' at node+4 and owner 'GFC_Contro… |
| `016dc445` | GfcControlProp_MultilineEditStyle | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GPropertyInterface>' at node+4 and owner 'GFC_Contro… |
| `016dc477` | GfcControlProp_DefaultEditStyle | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GPropertyInterface>' at node+4 and owner 'GFC_Contro… |
| `016dc4a9` | GfcControlProp_EditFlag | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, type 'p<c GPropertyNamespace>' at node+4 and the name string at node+0xc, then… |
| `016dc4d8` | GfcControlProp_EditFlagClamp | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc50a` | GfcControlProp_EditFlagNoInnerFill | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc53c` | GfcControlProp_EditFlagNoBorder | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc56e` | GfcControlProp_EditFlagHistory | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc5a0` | GfcControlProp_EditFlagSelect | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc5d2` | GfcControlProp_EditFlagTabSize | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc604` | GfcControlProp_EditFlagTabKey | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc636` | GfcControlProp_EditFlagVAlign | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc668` | GfcControlProp_EditFlagHAlign | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc69a` | GfcControlProp_EditFlagWrap | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc6cc` | GfcControlProp_EditFlagCropLeft | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc6fe` | GfcControlProp_EditFlagCropRight | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc730` | GfcControlProp_EditFlagCropTop | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc762` | GfcControlProp_EditFlagCropBottom | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc794` | GfcControlProp_EditFlagScrollFrame | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc7c6` | GfcControlProp_EditFlagMultiline | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc7f8` | GfcControlProp_EditFlagEditMaskInvalidateCaret | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc82a` | GfcControlProp_EditFlagEditMaskInvalidateSelection | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc85c` | GfcControlProp_EditFlagEditMaskInvalidateControl | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u64' from the shared .rdata record &DAT_0175fef0 at node… |
| `016dc88e` | GfcControlProp_EditRenderer | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, type 'p<c GPropertyNamespace>' at node+4 and the name string at node+0xc, then… |
| `016dc8bd` | GfcControlProp_EditRendererText | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GEditTextRenderer>' at node+4 and owner 'GFC_Control… |
| `016dc8ef` | GfcControlProp_EditRendererSelect | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GEditSelectRenderer>' at node+4 and owner 'GFC_Contr… |
| `016dc921` | GfcControlProp_EditRendererCaret | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GEditCaretRenderer>' at node+4 and owner 'GFC_Contro… |
| `016dc953` | GfcControlProp_EditRendererDragCaret | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GEditCaretRenderer>' at node+4 and owner 'GFC_Contro… |
| `016dc985` | GfcControlProp_EditChild | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, type 'p<c GPropertyNamespace>' at node+4 and the name string at node+0xc, then… |
| `016dc9b4` | GfcControlProp_EditChildScrollFrame | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GScrollFrameCtrl>' at node+4 and owner 'GFC_Control_… |
| `016dc9e6` | GfcControlProp_EditChildBorder | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GBorderCtrl>' at node+4 and owner 'GFC_Control_Theme… |
| `016dca18` | GfcControlProp_EditChildInnerBorder | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GBorderCtrl>' at node+4 and owner 'GFC_Control_Theme… |
| `016dca4a` | GfcControlProp_EditChildPopupMenu | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GMenuCtrl>' at node+4 and owner 'GFC_Control_Theme' … |
| `016dca7c` | GfcControlProp_EditChildDropMenu | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GMenuCtrl>' at node+4 and owner 'GFC_Control_Theme' … |
| `016dcaae` | GfcControlProp_EditChildDragLayer | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GCtrl>' at node+4 and owner 'GFC_Control_Theme' at n… |
| `016dcae0` | GfcControlProp_EditChildHistory | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GEditActionHistory>' at node+4 and owner 'GFC_Contro… |
| `016dcb12` | GfcControlProp_EditStyleId | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, type 'p<c GPropertyNamespace>' at node+4 and the name string at node+0xc, then… |
| `016dcb41` | GfcControlProp_EditStyleIdScrollFrame | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPropertyId' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dcb73` | GfcControlProp_EditStyleIdBorder | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPropertyId' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dcba5` | GfcControlProp_EditStyleIdInnerBorder | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPropertyId' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dcbd7` | GfcControlProp_EditStyleIdPopupMenu | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPropertyId' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dcc09` | GfcControlProp_EditStyleIdDropMenu | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPropertyId' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dcc3b` | GfcControlProp_EditStyleIdDragLayer | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPropertyId' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dcc6d` | GfcControlProp_EditStyleIdIMCListPanel | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPropertyId' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dcc9f` | GfcControlProp_EditStyleIdIMCListTable | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPropertyId' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dccd1` | GfcControlProp_EditStyleIdIMCListLabel | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPropertyId' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dcd03` | GfcControlProp_EditText | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, type 'p<c GPropertyNamespace>' at node+4 and the name string at node+0xc, then… |
| `016dcd32` | GfcControlProp_EditTextMaskChar | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u16' from the shared .rdata record &DAT_0177059c at node… |
| `016dcd64` | GfcControlProp_EditTextRegExpFormat | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dcd96` | GfcControlProp_EditTextRegExpWord | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dcdc8` | GfcControlProp_EditTextDragDropMenuCopy | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dcdfa` | GfcControlProp_EditTextDragDropMenuMove | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dce2c` | GfcControlProp_EditTextDragDropMenuLink | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dce5e` | GfcControlProp_EditTextDragDropMenuCancel | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dce90` | GfcControlProp_EditTextPopupMenuUndo | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dcec2` | GfcControlProp_EditTextPopupMenuUndoSingle | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dcef4` | GfcControlProp_EditTextPopupMenuRedo | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dcf26` | GfcControlProp_EditTextPopupMenuRedoSingle | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dcf58` | GfcControlProp_EditTextPopupMenuCut | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dcf8a` | GfcControlProp_EditTextPopupMenuCopy | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dcfbc` | GfcControlProp_EditTextPopupMenuPaste | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dcfee` | GfcControlProp_EditTextPopupMenuDelete | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dd020` | GfcControlProp_EditTextPopupMenuSelectAll | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GString>' at node+4 and owner 'GFC_Control_Theme' at… |
| `016dd052` | GfcControlProp_EditFontText | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GFont>' at node+4 and owner 'GFC_Control_Theme' at n… |
| `016dd084` | GfcControlProp_EditSize | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, type 'p<c GPropertyNamespace>' at node+4 and the name string at node+0xc, then… |
| `016dd0b3` | GfcControlProp_EditSizeTextOffset | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPoint' at node+4 and owner 'GFC_Control_Theme' at nod… |
| `016dd0e5` | GfcControlProp_EditSizeMaxDataSize | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u32' from the shared .rdata record &DAT_01760104 at node… |
| `016dd117` | GfcControlProp_EditSizeMaxCharCount | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u32' from the shared .rdata record &DAT_01760104 at node… |
| `016dd149` | GfcControlProp_EditSizeMaxLineCount | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u32' from the shared .rdata record &DAT_01760104 at node… |
| `016dd17b` | GfcControlProp_EditSizeMaxStringWidth | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u32' from the shared .rdata record &DAT_01760104 at node… |
| `016dd1ad` | GfcControlProp_EditSizeTabSize | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u32' from the shared .rdata record &DAT_01760104 at node… |
| `016dd1df` | GfcControlProp_EditSizeTabStops | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'p<c GInt32VectorObject>' at node+4 and owner 'GFC_Contro… |
| `016dd211` | GfcControlProp_EditSizeViewMargin | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GRectMargin' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dd243` | GfcControlProp_EditSizePageMargin | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GRectMargin' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dd275` | GfcControlProp_EditSizePageWidth | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u32' from the shared .rdata record &DAT_01760104 at node… |
| `016dd2a7` | GfcControlProp_EditSizeScrollShiftUp | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u32' from the shared .rdata record &DAT_01760104 at node… |
| `016dd2d9` | GfcControlProp_EditSizeScrollShiftDown | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u32' from the shared .rdata record &DAT_01760104 at node… |
| `016dd30b` | GfcControlProp_EditSizeScrollShiftLeft | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u32' from the shared .rdata record &DAT_01760104 at node… |
| `016dd33d` | GfcControlProp_EditSizeScrollShiftRight | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'u32' from the shared .rdata record &DAT_01760104 at node… |
| `016dd36f` | GfcControlProp_EditSizeCaretInsert | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GSize' at node+4 and owner 'GFC_Control_Theme' at node… |
| `016dd3a1` | GfcControlProp_EditSizeCaretOverwrite | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GSize' at node+4 and owner 'GFC_Control_Theme' at node… |
| `016dd3d3` | GfcControlProp_EditSizeCaretDragCopy | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GSize' at node+4 and owner 'GFC_Control_Theme' at node… |
| `016dd405` | GfcControlProp_EditSizeCaretDragMove | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GSize' at node+4 and owner 'GFC_Control_Theme' at node… |
| `016dd437` | GfcControlProp_EditSizeCaretDragLink | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GSize' at node+4 and owner 'GFC_Control_Theme' at node… |
| `016dd469` | GfcControlProp_EditSizeCaretOffsetInsert | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPoint' at node+4 and owner 'GFC_Control_Theme' at nod… |
| `016dd49b` | GfcControlProp_EditSizeCaretOffsetOverwrite | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPoint' at node+4 and owner 'GFC_Control_Theme' at nod… |
| `016dd4cd` | GfcControlProp_EditSizeCaretOffsetDragCopy | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPoint' at node+4 and owner 'GFC_Control_Theme' at nod… |
| `016dd4ff` | GfcControlProp_EditSizeCaretOffsetDragMove | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPoint' at node+4 and owner 'GFC_Control_Theme' at nod… |
| `016dd531` | GfcControlProp_EditSizeCaretOffsetDragLink | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GPoint' at node+4 and owner 'GFC_Control_Theme' at nod… |
| `016dd563` | GfcControlProp_EditSizeCaretExtentInsert | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GRectMargin' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dd595` | GfcControlProp_EditSizeCaretExtentOverwrite | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GRectMargin' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dd5c7` | GfcControlProp_EditSizeCaretExtentDragCopy | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GRectMargin' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dd5f9` | GfcControlProp_EditSizeCaretExtentDragMove | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GRectMargin' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dd62b` | GfcControlProp_EditSizeCaretExtentDragLink | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GRectMargin' at node+4 and owner 'GFC_Control_Theme' a… |
| `016dd65d` | GfcControlProp_EditSizeDefault | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GSize' at node+4 and owner 'GFC_Control_Theme' at node… |
| `016dd68f` | GfcControlProp_EditColor | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes 0 at node+8, type 'p<c GPropertyNamespace>' at node+4 and the name string at node+0xc, then… |
| `016dd6be` | GfcControlProp_EditColorInnerDefault | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColor' at node+4 and owner 'GFC_Control_Theme' at nod… |
| `016dd6f0` | GfcControlProp_EditColorInnerInactive | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColor' at node+4 and owner 'GFC_Control_Theme' at nod… |
| `016dd722` | GfcControlProp_EditColorInnerDisable | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColor' at node+4 and owner 'GFC_Control_Theme' at nod… |
| `016dd754` | GfcControlProp_EditColorSelectDefault | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColor' at node+4 and owner 'GFC_Control_Theme' at nod… |
| `016dd786` | GfcControlProp_EditColorSelectInactive | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColor' at node+4 and owner 'GFC_Control_Theme' at nod… |
| `016dd7b8` | GfcControlProp_EditColorSelectDisable | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColor' at node+4 and owner 'GFC_Control_Theme' at nod… |
| `016dd7ea` | GfcControlProp_EditColorCaretInsertDefault | confirmed | Static initializer. Gets the GFC list head through 004c886d, writes the name string at node+0xc, type 'c GColor' at node+4 and owner 'GFC_Control_Theme' at nod… |
| `016e50d4` | GfcControlProp_TreeFlagNoLines | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e5106` | GfcControlProp_TreeFlagDrawSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e5138` | GfcControlProp_TreeFlagTreeMaskInvalidateControl | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e516a` | GfcControlNs_TreeRenderer | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e5199` | GfcControlProp_TreeRendererItem | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GTreeItemRenderer>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; n… |
| `016e51cb` | GfcControlProp_TreeRendererIndent | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GTreeIndentRenderer>', node+8 = owner GFC_Control_Theme, node+0x10 = old head;… |
| `016e51fd` | GfcControlNs_TreeDelay | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e522c` | GfcControlProp_TreeDelayRolloverAutoExpand | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e525e` | GfcControlNs_TreeSize | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e528d` | GfcControlProp_TreeSizeIndentWidth | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e52bf` | GfcControlProp_TreeSizeIndentHeight | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e52f1` | GfcControlProp_TreeSizeItemSpace | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_01760104 (rdata 'u32'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e5323` | GfcControlProp_TreeSizeItemContentMargin | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GRectMargin', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016e5355` | GfcControlProp_TreeSizeItemTextMargin | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GRectMargin', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016e5387` | GfcControlNs_TreeColor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e53b6` | GfcControlProp_TreeColorIndentConnectorDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e53e8` | GfcControlProp_TreeColorIndentConnectorSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e541a` | GfcControlProp_TreeColorIndentConnectorOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e544c` | GfcControlProp_TreeColorIndentConnectorMultiselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e547e` | GfcControlProp_TreeColorIndentConnectorInactive | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e54b0` | GfcControlProp_TreeColorIndentConnectorInactiveSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e54e2` | GfcControlProp_TreeColorIndentConnectorInactiveOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5514` | GfcControlProp_TreeColorIndentConnectorInactiveMultiselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5546` | GfcControlProp_TreeColorIndentConnectorDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5578` | GfcControlProp_TreeColorIndentConnectorDisableSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e55aa` | GfcControlProp_TreeColorIndentConnectorDisableOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e55dc` | GfcControlProp_TreeColorIndentConnectorDisableMultiselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e560e` | GfcControlProp_TreeColorIndentExpandImageDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5640` | GfcControlProp_TreeColorIndentExpandImageSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5672` | GfcControlProp_TreeColorIndentExpandImageOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e56a4` | GfcControlProp_TreeColorIndentExpandImageMultiselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e56d6` | GfcControlProp_TreeColorIndentExpandImageInactive | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5708` | GfcControlProp_TreeColorIndentExpandImageInactiveSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e573a` | GfcControlProp_TreeColorIndentExpandImageInactiveOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e576c` | GfcControlProp_TreeColorIndentExpandImageInactiveMultiselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e579e` | GfcControlProp_TreeColorIndentExpandImageDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e57d0` | GfcControlProp_TreeColorIndentExpandImageDisableSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5802` | GfcControlProp_TreeColorIndentExpandImageDisableOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5834` | GfcControlProp_TreeColorIndentExpandImageDisableMultiselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5866` | GfcControlProp_TreeColorItemImageDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5898` | GfcControlProp_TreeColorItemImageSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e58ca` | GfcControlProp_TreeColorItemImageOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e58fc` | GfcControlProp_TreeColorItemImageMultiselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e592e` | GfcControlProp_TreeColorItemImageInactive | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5960` | GfcControlProp_TreeColorItemImageInactiveSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5992` | GfcControlProp_TreeColorItemImageInactiveOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e59c4` | GfcControlProp_TreeColorItemImageInactiveMultiselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e59f6` | GfcControlProp_TreeColorItemImageDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5a28` | GfcControlProp_TreeColorItemImageDisableSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5a5a` | GfcControlProp_TreeColorItemImageDisableOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5a8c` | GfcControlProp_TreeColorItemImageDisableMultiselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e5abe` | GfcControlNs_TreeCombine | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e5aed` | GfcControlProp_TreeCombineIndentConnectorDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5b1f` | GfcControlProp_TreeCombineIndentConnectorSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5b51` | GfcControlProp_TreeCombineIndentConnectorOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5b83` | GfcControlProp_TreeCombineIndentConnectorMultiselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5bb5` | GfcControlProp_TreeCombineIndentConnectorInactive | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5be7` | GfcControlProp_TreeCombineIndentConnectorInactiveSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5c19` | GfcControlProp_TreeCombineIndentConnectorInactiveOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5c4b` | GfcControlProp_TreeCombineIndentConnectorInactiveMultiselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5c7d` | GfcControlProp_TreeCombineIndentConnectorDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5caf` | GfcControlProp_TreeCombineIndentConnectorDisableSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5ce1` | GfcControlProp_TreeCombineIndentConnectorDisableOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5d13` | GfcControlProp_TreeCombineIndentConnectorDisableMultiselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5d45` | GfcControlProp_TreeCombineIndentExpandImageDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5d77` | GfcControlProp_TreeCombineIndentExpandImageSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5da9` | GfcControlProp_TreeCombineIndentExpandImageOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5ddb` | GfcControlProp_TreeCombineIndentExpandImageMultiselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5e0d` | GfcControlProp_TreeCombineIndentExpandImageInactive | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5e3f` | GfcControlProp_TreeCombineIndentExpandImageInactiveSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5e71` | GfcControlProp_TreeCombineIndentExpandImageInactiveOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5ea3` | GfcControlProp_TreeCombineIndentExpandImageInactiveMultisel… | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5ed5` | GfcControlProp_TreeCombineIndentExpandImageDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5f07` | GfcControlProp_TreeCombineIndentExpandImageDisableSelect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5f39` | GfcControlProp_TreeCombineIndentExpandImageDisableOffselect | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5f6b` | GfcControlProp_TreeCombineIndentExpandImageDisableMultisele… | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5f9d` | GfcControlProp_TreeCombineItemImageDefaultDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e5fcf` | GfcControlProp_TreeCombineItemImageDefaultFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e6001` | GfcControlProp_TreeCombineItemImageSelectDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e6033` | GfcControlProp_TreeCombineItemImageSelectFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e6065` | GfcControlProp_TreeCombineItemImageOffselectDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e6097` | GfcControlProp_TreeCombineItemImageOffselectFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e60c9` | GfcControlProp_TreeCombineItemImageMultiselectDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e60fb` | GfcControlProp_TreeCombineItemImageMultiselectFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e612d` | GfcControlProp_TreeCombineItemImageInactiveDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e615f` | GfcControlProp_TreeCombineItemImageInactiveFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e6191` | GfcControlProp_TreeCombineItemImageInactiveSelectDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e61c3` | GfcControlProp_TreeCombineItemImageInactiveSelectFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e61f5` | GfcControlProp_TreeCombineItemImageInactiveOffselectDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e6227` | GfcControlProp_TreeCombineItemImageInactiveOffselectFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e6259` | GfcControlProp_TreeCombineItemImageInactiveMultiselectDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e628b` | GfcControlProp_TreeCombineItemImageInactiveMultiselectFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e62bd` | GfcControlProp_TreeCombineItemImageDisableDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e62ef` | GfcControlProp_TreeCombineItemImageDisableFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e6321` | GfcControlProp_TreeCombineItemImageDisableSelectDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e6353` | GfcControlProp_TreeCombineItemImageDisableSelectFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e6385` | GfcControlProp_TreeCombineItemImageDisableOffselectDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e63b7` | GfcControlProp_TreeCombineItemImageDisableOffselectFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e63e9` | GfcControlProp_TreeCombineItemImageDisableMultiselectDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e641b` | GfcControlProp_TreeCombineItemImageDisableMultiselectFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e644d` | GfcControlNs_TreeBinding | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e647c` | GfcControlProp_TreeBindingExpandItemTrue | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016e64ae` | GfcControlProp_TreeBindingExpandItemFalse | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016e64e0` | GfcControlProp_TreeBindingExpandItemToggle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016e6512` | GfcControlProp_TreeBindingLastIndentExpandItemTrue | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016e6544` | GfcControlProp_TreeBindingLastIndentExpandItemFalse | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016e6576` | GfcControlProp_TreeBindingLastIndentExpandItemToggle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016e65a8` | GfcControlProp_TreeBindingExpandChildrenTrue | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016e65da` | GfcControlProp_TreeBindingExpandChildrenFalse | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016e660c` | GfcControlProp_TreeBindingExpandChildrenToggle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GInputBinding>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016e663e` | GfcControlNs_TreeMessage | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e666d` | GfcControlMsg_TreeMessageOnExpandItem | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = pointer &DAT_0175f744 (rdata 'v'), node+8 = 0, node+0x10 = old head; node becomes head. |
| `016e669c` | GfcControlMsg_TreeMessageOnUIExpandItem | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = pointer &DAT_0175f744 (rdata 'v'), node+8 = 0, node+0x10 = old head; node becomes head. |
| `016e66cb` | GfcControlMsg_TreeMessageDoExpandItem | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = pointer &DAT_0175f744 (rdata 'v'), node+8 = 0, node+0x10 = old head; node becomes head. |
| `016e66fa` | GfcControlMsg_TreeMessageDoExpandToggle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = pointer &DAT_0175f744 (rdata 'v'), node+8 = 0, node+0x10 = old head; node becomes head. |
| `016e6729` | GfcControlMsg_TreeMessageDoExpand | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = pointer &DAT_0175f744 (rdata 'v'), node+8 = 0, node+0x10 = old head; node becomes head. |
| `016e6758` | GfcControlMsg_TreeMessageDoCollapse | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = pointer &DAT_0175f744 (rdata 'v'), node+8 = 0, node+0x10 = old head; node becomes head. |
| `016e6787` | GfcControlNs_TreeTimer | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e67b6` | GfcControlMsg_TreeTimerRolloverAutoExpand | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = pointer &DAT_0175f744 (rdata 'v'), node+8 = 0, node+0x10 = old head; node becomes head. |
| `016e683c` | GfcControlNs_Window | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e686b` | GfcControlProp_StandardWindowStyle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyInterface>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; … |
| `016e689d` | GfcControlProp_ToolWindowStyle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyInterface>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; … |
| `016e68cf` | GfcControlProp_DialogWindowStyle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyInterface>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; … |
| `016e6901` | GfcControlProp_DefaultWindowStyle | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyInterface>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; … |
| `016e6933` | GfcControlNs_WindowFlag | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e6962` | GfcControlProp_WindowFlagNoTitleBar | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175fef0 (rdata 'u64'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e6994` | GfcControlProp_WindowFlagNoResizeGrip | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175fef0 (rdata 'u64'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e69c6` | GfcControlProp_WindowFlagNoPopupMenu | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175fef0 (rdata 'u64'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e69f8` | GfcControlProp_WindowFlagNoBorder | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175fef0 (rdata 'u64'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e6a2a` | GfcControlProp_WindowFlagNoInnerBorder | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175fef0 (rdata 'u64'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e6a5c` | GfcControlProp_WindowFlagNoInnerFill | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175fef0 (rdata 'u64'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e6a8e` | GfcControlProp_WindowFlagInnerMovable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175fef0 (rdata 'u64'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e6ac0` | GfcControlProp_WindowFlagWindowMaskInvalidateControl | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type pointer &DAT_0175fef0 (rdata 'u64'), node+8 = owner GFC_Control_Theme, node+0x10 = … |
| `016e6af2` | GfcControlNs_WindowChild | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e6b21` | GfcControlProp_WindowChildTitleBar | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GTitleBarCtrl>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016e6b53` | GfcControlProp_WindowChildOuterBorder | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBorderCtrl>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node be… |
| `016e6b85` | GfcControlProp_WindowChildInnerBorder | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GBorderCtrl>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node be… |
| `016e6bb7` | GfcControlProp_WindowChildResizeGrip | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GSizeGripCtrl>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node … |
| `016e6be9` | GfcControlProp_WindowChildPopupMenu | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GMenuCtrl>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node beco… |
| `016e6c1b` | GfcControlNs_WindowStyleId | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e6c4a` | GfcControlProp_WindowStyleIdTitleBar | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GPropertyId', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016e6c7c` | GfcControlProp_WindowStyleIdOuterBorder | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GPropertyId', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016e6cae` | GfcControlProp_WindowStyleIdInnerBorder | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GPropertyId', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016e6ce0` | GfcControlProp_WindowStyleIdResizeGrip | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GPropertyId', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016e6d12` | GfcControlProp_WindowStyleIdPopupMenu | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GPropertyId', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016e6d44` | GfcControlNs_WindowText | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e6d73` | GfcControlProp_WindowTextPopupMenuMove | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GString>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e6da5` | GfcControlProp_WindowTextPopupMenuSize | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GString>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e6dd7` | GfcControlProp_WindowTextPopupMenuMinimize | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GString>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e6e09` | GfcControlProp_WindowTextPopupMenuMaximize | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GString>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e6e3b` | GfcControlProp_WindowTextPopupMenuCollapse | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GString>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e6e6d` | GfcControlProp_WindowTextPopupMenuCollapseRestore | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GString>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e6e9f` | GfcControlProp_WindowTextPopupMenuMinimizeRestore | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GString>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e6ed1` | GfcControlProp_WindowTextPopupMenuMaximizeRestore | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GString>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e6f03` | GfcControlProp_WindowTextPopupMenuRestore | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GString>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e6f35` | GfcControlProp_WindowTextPopupMenuClose | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GString>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e6f67` | GfcControlProp_WindowTextPopupMenuHelp | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GString>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e6f99` | GfcControlProp_WindowTextPopupMenuWindows | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GString>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e6fcb` | GfcControlNs_WindowSize | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e6ffa` | GfcControlProp_WindowSizeResizeGrip | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GSize', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes hea… |
| `016e702c` | GfcControlProp_WindowSizeDefaultMinNormal | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GSize', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes hea… |
| `016e705e` | GfcControlProp_WindowSizeDefaultMinCollapsed | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GSize', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes hea… |
| `016e7090` | GfcControlProp_WindowSizeDefaultMinMinimized | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GSize', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes hea… |
| `016e70c2` | GfcControlProp_WindowSizeDefaultMinMaximized | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GSize', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes hea… |
| `016e70f4` | GfcControlProp_WindowSizeExtentMarginNormal | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GRectMargin', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016e7126` | GfcControlProp_WindowSizeExtentMarginCollapsed | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GRectMargin', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016e7158` | GfcControlProp_WindowSizeExtentMarginMinimized | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GRectMargin', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016e718a` | GfcControlProp_WindowSizeExtentMarginMaximized | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GRectMargin', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becom… |
| `016e71bc` | GfcControlNs_WindowCursor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e71eb` | GfcControlProp_WindowCursorMove | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e721d` | GfcControlProp_WindowCursorMoveUp | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e724f` | GfcControlProp_WindowCursorMoveDown | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e7281` | GfcControlProp_WindowCursorMoveUpDown | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e72b3` | GfcControlProp_WindowCursorMoveLeft | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e72e5` | GfcControlProp_WindowCursorMoveRight | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e7317` | GfcControlProp_WindowCursorMoveLeftRight | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e7349` | GfcControlProp_WindowCursorSize | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e737b` | GfcControlProp_WindowCursorSizeUp | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e73ad` | GfcControlProp_WindowCursorSizeDown | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e73df` | GfcControlProp_WindowCursorSizeLeft | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e7411` | GfcControlProp_WindowCursorSizeRight | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e7443` | GfcControlProp_WindowCursorSizeUpLeft | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e7475` | GfcControlProp_WindowCursorSizeUpRight | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e74a7` | GfcControlProp_WindowCursorSizeDownLeft | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e74d9` | GfcControlProp_WindowCursorSizeDownRight | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e750b` | GfcControlProp_WindowCursorHelp | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GCursor>', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node become… |
| `016e753d` | GfcControlNs_WindowColor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e756c` | GfcControlProp_WindowColorInner | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e759e` | GfcControlProp_WindowColorTitleDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e75d0` | GfcControlProp_WindowColorTitleDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e7602` | GfcControlProp_WindowColorTitleWinoff | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e7634` | GfcControlProp_WindowColorTitleWinoffDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e7666` | GfcControlProp_WindowColorTitleShadowDefault | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e7698` | GfcControlProp_WindowColorTitleShadowDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e76ca` | GfcControlProp_WindowColorTitleShadowWinoff | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e76fc` | GfcControlProp_WindowColorTitleShadowWinoffDisable | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColor', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node becomes he… |
| `016e772e` | GfcControlNs_WindowCombine | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e775d` | GfcControlProp_WindowCombineInner | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e778f` | GfcControlProp_WindowCombineTitleDefaultDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e77c1` | GfcControlProp_WindowCombineTitleDefaultFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e77f3` | GfcControlProp_WindowCombineTitleDisableDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e7825` | GfcControlProp_WindowCombineTitleDisableFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e7857` | GfcControlProp_WindowCombineTitleWinoffDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e7889` | GfcControlProp_WindowCombineTitleWinoffFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e78bb` | GfcControlProp_WindowCombineTitleWinoffDisableDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e78ed` | GfcControlProp_WindowCombineTitleWinoffDisableFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e791f` | GfcControlProp_WindowCombineTitleShadowDefaultDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e7951` | GfcControlProp_WindowCombineTitleShadowDefaultFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e7983` | GfcControlProp_WindowCombineTitleShadowDisableDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e79b5` | GfcControlProp_WindowCombineTitleShadowDisableFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e79e7` | GfcControlProp_WindowCombineTitleShadowWinoffDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e7a19` | GfcControlProp_WindowCombineTitleShadowWinoffFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e7a4b` | GfcControlProp_WindowCombineTitleShadowWinoffDisableDest | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e7a7d` | GfcControlProp_WindowCombineTitleShadowWinoffDisableFactor | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'c GColorCombine', node+8 = owner GFC_Control_Theme, node+0x10 = old head; node bec… |
| `016e7aaf` | GfcControlNs_WindowFont | confirmed | Static initializer: list head from 004c886d; node+0xc = name, node+4 = type 'p<c GPropertyNamespace>', node+8 = 0 (no owner), node+0x10 = old head; node become… |
| `016e7ade` | GfcControlProp_WindowFontTitle | confirmed | Static initializer: list head from 004c886d; node 01bd39c8 = {+4 type p<c GFont>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Window_Font_Title, +0x10 ol… |
| `016e7b10` | GfcControlProp_WindowBinding | confirmed | Static initializer: list head from 004c886d; node 01bd554c = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Window_Binding, +0x10 old head… |
| `016e7b3f` | GfcControlProp_WindowBindingMoveBegin | confirmed | Static initializer: list head from 004c886d; node 01bd69dc = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Window_Binding_Move… |
| `016e7b71` | GfcControlProp_WindowBindingMoveCancel | confirmed | Static initializer: list head from 004c886d; node 01bd5694 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Window_Binding_Move… |
| `016e7ba3` | GfcControlProp_WindowBindingMoveCommit | confirmed | Static initializer: list head from 004c886d; node 01bd21c8 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Window_Binding_Move… |
| `016e7bd5` | GfcControlProp_WindowBindingSizeBegin | confirmed | Static initializer: list head from 004c886d; node 01bd5e78 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Window_Binding_Size… |
| `016e7c07` | GfcControlProp_WindowBindingSizeCancel | confirmed | Static initializer: list head from 004c886d; node 01bd5b94 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Window_Binding_Size… |
| `016e7c39` | GfcControlProp_WindowBindingSizeCommit | confirmed | Static initializer: list head from 004c886d; node 01bd62e0 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Window_Binding_Size… |
| `016e7c6b` | GfcControlProp_WindowBindingSizeSelectLeft | confirmed | Static initializer: list head from 004c886d; node 01bd20ec = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Window_Binding_Size… |
| `016e7c9d` | GfcControlProp_WindowBindingSizeSelectRight | confirmed | Static initializer: list head from 004c886d; node 01bd3ab4 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Window_Binding_Size… |
| `016e7ccf` | GfcControlProp_WindowBindingSizeSelectTop | confirmed | Static initializer: list head from 004c886d; node 01bd2d60 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Window_Binding_Size… |
| `016e7d01` | GfcControlProp_WindowBindingSizeSelectBottom | confirmed | Static initializer: list head from 004c886d; node 01bd465c = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Window_Binding_Size… |
| `016e7d33` | GfcControlProp_WindowBindingPopupMenu | confirmed | Static initializer: list head from 004c886d; node 01bd49e4 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Window_Binding_Popu… |
| `016e7d65` | GfcControlProp_WindowBindingPopupMenuBeginTracking | confirmed | Static initializer: list head from 004c886d; node 01bd02a8 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Window_Binding_Popu… |
| `016e7d97` | GfcControlProp_WindowMessage | confirmed | Static initializer: list head from 004c886d; node 01bd3af0 = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Window_Message, +0x10 old head… |
| `016e7dc6` | GfcControlProp_WindowMessageOnMinimize | confirmed | Static initializer: list head from 004c886d; node 01bd5440 = {+4 type &DAT_0175f744 (bytes 76 00, 'v'), +8 owner 0, +0xc name GFC_Control_Window_Message_OnMini… |
| `016e7df5` | GfcControlProp_WindowMessageOnMaximize | confirmed | Static initializer: list head from 004c886d; node 01bd3b40 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_OnMaximize, +0x10 old… |
| `016e7e24` | GfcControlProp_WindowMessageOnRestore | confirmed | Static initializer: list head from 004c886d; node 01bd4050 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_OnRestore, +0x10 old … |
| `016e7e53` | GfcControlProp_WindowMessageOnCollapse | confirmed | Static initializer: list head from 004c886d; node 01bd2e14 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_OnCollapse, +0x10 old… |
| `016e7e82` | GfcControlProp_WindowMessageOnCloseRequest | confirmed | Static initializer: list head from 004c886d; node 01bd10ec = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_OnCloseRequest, +0x10… |
| `016e7eb1` | GfcControlProp_WindowMessageOnPopupMenuItemMessage | confirmed | Static initializer: list head from 004c886d; node 01bd134c = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_OnPopupMenu_ItemMessa… |
| `016e7ee0` | GfcControlProp_WindowMessageDoClose | confirmed | Static initializer: list head from 004c886d; node 01bd54b8 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_DoClose, +0x10 old he… |
| `016e7f0f` | GfcControlProp_WindowMessageDoMinimize | confirmed | Static initializer: list head from 004c886d; node 01bd2ae0 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_DoMinimize, +0x10 old… |
| `016e7f3e` | GfcControlProp_WindowMessageDoMaximize | confirmed | Static initializer: list head from 004c886d; node 01bd55d8 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_DoMaximize, +0x10 old… |
| `016e7f6d` | GfcControlProp_WindowMessageDoRestore | confirmed | Static initializer: list head from 004c886d; node 01bd6504 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_DoRestore, +0x10 old … |
| `016e7f9c` | GfcControlProp_WindowMessageDoCollapse | confirmed | Static initializer: list head from 004c886d; node 01bd27dc = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_DoCollapse, +0x10 old… |
| `016e7fcb` | GfcControlProp_WindowMessageDoHelp | confirmed | Static initializer: list head from 004c886d; node 01bd02ec = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_DoHelp, +0x10 old hea… |
| `016e7ffa` | GfcControlProp_WindowMessageDoMove | confirmed | Static initializer: list head from 004c886d; node 01bd33a4 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_DoMove, +0x10 old hea… |
| `016e8029` | GfcControlProp_WindowMessageDoSize | confirmed | Static initializer: list head from 004c886d; node 01bd0b40 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_DoSize, +0x10 old hea… |
| `016e8058` | GfcControlProp_WindowMessageDoCancelResizing | confirmed | Static initializer: list head from 004c886d; node 01bd5d74 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_DoCancelResizing, +0x… |
| `016e8087` | GfcControlProp_WindowMessageDoCancelMoving | confirmed | Static initializer: list head from 004c886d; node 01bd101c = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_DoCancelMoving, +0x10… |
| `016e80b6` | GfcControlProp_WindowMessageDoShowPopupMenu | confirmed | Static initializer: list head from 004c886d; node 01bd00b8 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Window_Message_DoShowPopupMenu, +0x1… |
| `016e811c` | GfcControlProp_Header | confirmed | Static initializer: list head from 004c886d; node 01bd9158 = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Header, +0x10 old head}; the n… |
| `016e814b` | GfcControlProp_StandardHeaderTableStyle | confirmed | Static initializer: list head from 004c886d; node 01bde504 = {+4 type p<c GPropertyInterface>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_StandardHeader… |
| `016e817d` | GfcControlProp_DefaultHeaderTableStyle | confirmed | Static initializer: list head from 004c886d; node 01bddd84 = {+4 type p<c GPropertyInterface>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_DefaultHeaderT… |
| `016e81af` | GfcControlProp_HeaderFlag | confirmed | Static initializer: list head from 004c886d; node 01bdf16c = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Header_Flag, +0x10 old head}; … |
| `016e81de` | GfcControlProp_HeaderFlagNoInnerFill | confirmed | Static initializer: list head from 004c886d; node 01bda2f8 = {+4 type &DAT_01760104 (bytes 'u32'), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Header_Fla… |
| `016e8210` | GfcControlProp_HeaderFlagHeaderMaskInvalidateControl | confirmed | Static initializer: list head from 004c886d; node 01bdea18 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Header_Flag_Header… |
| `016e8242` | GfcControlProp_HeaderChild | confirmed | Static initializer: list head from 004c886d; node 01bd7440 = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Header_Child, +0x10 old head};… |
| `016e8271` | GfcControlProp_HeaderChildHButton | confirmed | Static initializer: list head from 004c886d; node 01bda090 = {+4 type p<c GHeaderButtonCtrl>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Header_Child_HB… |
| `016e82a3` | GfcControlProp_HeaderChildVButton | confirmed | Static initializer: list head from 004c886d; node 01bdced4 = {+4 type p<c GHeaderButtonCtrl>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Header_Child_VB… |
| `016e82d5` | GfcControlProp_HeaderStyleId | confirmed | Static initializer: list head from 004c886d; node 01bdab60 = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Header_StyleId, +0x10 old head… |
| `016e8304` | GfcControlProp_HeaderStyleIdHButton | confirmed | Static initializer: list head from 004c886d; node 01bdc794 = {+4 type c GPropertyId, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Header_StyleId_HButton, … |
| `016e8336` | GfcControlProp_HeaderStyleIdHSortAscendButton | confirmed | Static initializer: list head from 004c886d; node 01bd7cd8 = {+4 type c GPropertyId, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Header_StyleId_HSortAsce… |
| `016e8368` | GfcControlProp_HeaderStyleIdHSortDescendButton | confirmed | Static initializer: list head from 004c886d; node 01bd8614 = {+4 type c GPropertyId, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Header_StyleId_HSortDesc… |
| `016e839a` | GfcControlProp_HeaderStyleIdVButton | confirmed | Static initializer: list head from 004c886d; node 01bddd70 = {+4 type c GPropertyId, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Header_StyleId_VButton, … |
| `016e83cc` | GfcControlProp_HeaderStyleIdVSortAscendButton | confirmed | Static initializer: list head from 004c886d; node 01bdd270 = {+4 type c GPropertyId, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Header_StyleId_VSortAsce… |
| `016e83fe` | GfcControlProp_HeaderStyleIdVSortDescendButton | confirmed | Static initializer: list head from 004c886d; node 01bdd914 = {+4 type c GPropertyId, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Header_StyleId_VSortDesc… |
| `016e8430` | GfcControlProp_HeaderMessage | confirmed | Static initializer: list head from 004c886d; node 01bda230 = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Header_Message, +0x10 old head… |
| `016e845f` | GfcControlProp_HeaderMessageOnButtonExecute | confirmed | Static initializer: list head from 004c886d; node 01bd8b84 = {+4 type &DAT_0175f744 (bytes 'v'), +8 owner 0, +0xc name GFC_Control_Header_Message_OnButtonExecu… |
| `016e84e5` | GfcControlProp_Menu | confirmed | Static initializer: list head from 004c886d; node 01bdb304 = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Menu, +0x10 old head}; the nod… |
| `016e8514` | GfcControlProp_StandardMenuStyle | confirmed | Static initializer: list head from 004c886d; node 01bd7510 = {+4 type p<c GPropertyInterface>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_StandardMenu_S… |
| `016e8546` | GfcControlProp_DefaultMenuStyle | confirmed | Static initializer: list head from 004c886d; node 01bd83b8 = {+4 type p<c GPropertyInterface>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_DefaultMenu_St… |
| `016e8578` | GfcControlProp_MenuFlag | confirmed | Static initializer: list head from 004c886d; node 01bdc960 = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Menu_Flag, +0x10 old head}; th… |
| `016e85a7` | GfcControlProp_MenuFlagNoBorder | confirmed | Static initializer: list head from 004c886d; node 01bdada4 = {+4 type &DAT_01760104 (bytes 'u32'), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Flag_… |
| `016e85d9` | GfcControlProp_MenuFlagNoInnerFill | confirmed | Static initializer: list head from 004c886d; node 01bd8c38 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Flag_NoInnerF… |
| `016e860b` | GfcControlProp_MenuFlagRollover | confirmed | Static initializer: list head from 004c886d; node 01bd8454 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Flag_Rollover… |
| `016e863d` | GfcControlProp_MenuFlagSelectOnShow | confirmed | Static initializer: list head from 004c886d; node 01bd98a4 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Flag_Select_O… |
| `016e866f` | GfcControlProp_MenuFlagSelectDisabledItems | confirmed | Static initializer: list head from 004c886d; node 01bdc6e0 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Flag_Select_D… |
| `016e86a1` | GfcControlProp_MenuFlagSelectToggleMenu | confirmed | Static initializer: list head from 004c886d; node 01bd8ca4 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Flag_Select_T… |
| `016e86d3` | GfcControlProp_MenuFlagPopupFocusByMouse | confirmed | Static initializer: list head from 004c886d; node 01bd7da0 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Flag_PopupFoc… |
| `016e8705` | GfcControlProp_MenuFlagPopupFocusByKeyboard | confirmed | Static initializer: list head from 004c886d; node 01bda1cc = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Flag_PopupFoc… |
| `016e8737` | GfcControlProp_MenuFlagAutoPopupByKeyboard | confirmed | Static initializer: list head from 004c886d; node 01bda72c = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Flag_AutoPopu… |
| `016e8769` | GfcControlProp_MenuFlagAutoPopupByMouse | confirmed | Static initializer: list head from 004c886d; node 01bda5f0 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Flag_AutoPopu… |
| `016e879b` | GfcControlProp_MenuFlagPopupAutoSelectByKeyboard | confirmed | Static initializer: list head from 004c886d; node 01bdecd0 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Flag_PopupAut… |
| `016e87cd` | GfcControlProp_MenuFlagHidePopupOnLoseFocusByKeyboard | confirmed | Static initializer: list head from 004c886d; node 01bdb6d8 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Flag_HidePopu… |
| `016e87ff` | GfcControlProp_MenuFlagMenuMaskInvalidateControl | confirmed | Static initializer: list head from 004c886d; node 01bd7f18 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Flag_MenuMask… |
| `016e8831` | GfcControlProp_MenuRenderer | confirmed | Static initializer: list head from 004c886d; node 01bd7e70 = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Menu_Renderer, +0x10 old head}… |
| `016e8860` | GfcControlProp_MenuRendererSeparatorItem | confirmed | Static initializer: list head from 004c886d; node 01bd916c = {+4 type p<c GMenuSeparatorItemRenderer>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_R… |
| `016e8892` | GfcControlProp_MenuRendererTextItem | confirmed | Static initializer: list head from 004c886d; node 01bd96e4 = {+4 type p<c GMenuTextItemRenderer>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Render… |
| `016e88c4` | GfcControlProp_MenuSize | confirmed | Static initializer: list head from 004c886d; node 01bdc214 = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Menu_Size, +0x10 old head}; th… |
| `016e88f3` | GfcControlProp_MenuSizeMinimum | confirmed | Static initializer: list head from 004c886d; node 01bd84d4 = {+4 type c GSize, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Size_Minimum, +0x10 old h… |
| `016e8925` | GfcControlProp_MenuSizeMargin | confirmed | Static initializer: list head from 004c886d; node 01bdb8a4 = {+4 type c GRectMargin, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Size_Margin, +0x10 … |
| `016e8957` | GfcControlProp_MenuSizeItemHeight | confirmed | Static initializer: list head from 004c886d; node 01bd9fdc = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Size_ItemHeig… |
| `016e8989` | GfcControlProp_MenuSizeTextSpace | confirmed | Static initializer: list head from 004c886d; node 01bd9338 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Size_TextSpac… |
| `016e89bb` | GfcControlProp_MenuSizeTextMinimum | confirmed | Static initializer: list head from 004c886d; node 01bdbd9c = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Size_TextMini… |
| `016e89ed` | GfcControlProp_MenuSizeTextMargin | confirmed | Static initializer: list head from 004c886d; node 01bdc6a4 = {+4 type c GRectMargin, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Size_TextMargin, +0… |
| `016e8a1f` | GfcControlProp_MenuSizeImage | confirmed | Static initializer: list head from 004c886d; node 01bd8ffc = {+4 type c GSize, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Size_Image, +0x10 old hea… |
| `016e8a51` | GfcControlProp_MenuSizeImageColumn | confirmed | Static initializer: list head from 004c886d; node 01bdb3b8 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Size_ImageCol… |
| `016e8a83` | GfcControlProp_MenuSizeImageSpace | confirmed | Static initializer: list head from 004c886d; node 01bdaa04 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Size_ImageSpa… |
| `016e8ab5` | GfcControlProp_MenuSizeArrowColumn | confirmed | Static initializer: list head from 004c886d; node 01bdc50c = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Size_ArrowCol… |
| `016e8ae7` | GfcControlProp_MenuSizeArrowSpace | confirmed | Static initializer: list head from 004c886d; node 01bdb4d0 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Size_ArrowSpa… |
| `016e8b19` | GfcControlProp_MenuSizeSeparator | confirmed | Static initializer: list head from 004c886d; node 01bdb3cc = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Size_Separato… |
| `016e8b4b` | GfcControlProp_MenuSizeSubMenuMargin | confirmed | Static initializer: list head from 004c886d; node 01bd8650 = {+4 type c GRectMargin, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Size_SubMenuMargin,… |
| `016e8b7d` | GfcControlProp_MenuSizeExtentMargin | confirmed | Static initializer: list head from 004c886d; node 01bdb3f4 = {+4 type c GRectMargin, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Size_ExtentMargin, … |
| `016e8baf` | GfcControlProp_MenuChild | confirmed | Static initializer: list head from 004c886d; node 01bd9488 = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Menu_Child, +0x10 old head}; t… |
| `016e8bde` | GfcControlProp_MenuChildBorder | confirmed | Static initializer: list head from 004c886d; node 01bda838 = {+4 type p<c GBorderCtrl>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Child_Border, +0… |
| `016e8c10` | GfcControlProp_MenuChildTable | confirmed | Static initializer: list head from 004c886d; node 01bdf180 = {+4 type p<c GTableCtrl>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Child_Table, +0x1… |
| `016e8c42` | GfcControlProp_MenuChildTearOffButton | confirmed | Static initializer: list head from 004c886d; node 01bd9e08 = {+4 type p<c GButtonCtrl>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Child_TearOffBut… |
| `016e8c74` | GfcControlProp_MenuStyleId | confirmed | Static initializer: list head from 004c886d; node 01bd7c10 = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Menu_StyleId, +0x10 old head};… |
| `016e8ca3` | GfcControlProp_MenuStyleIdBorder | confirmed | Static initializer: list head from 004c886d; node 01bdd584 = {+4 type c GPropertyId, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_StyleId_Border, +0x… |
| `016e8cd5` | GfcControlProp_MenuStyleIdTable | confirmed | Static initializer: list head from 004c886d; node 01bda4a8 = {+4 type c GPropertyId, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_StyleId_Table, +0x1… |
| `016e8d07` | GfcControlProp_MenuStyleIdTearOffButton | confirmed | Static initializer: list head from 004c886d; node 01bd8f48 = {+4 type c GPropertyId, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_StyleId_TearOffButt… |
| `016e8d39` | GfcControlProp_MenuDelay | confirmed | Static initializer: list head from 004c886d; node 01bd7d78 = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Menu_Delay, +0x10 old head}; t… |
| `016e8d68` | GfcControlProp_MenuDelayDropDown | confirmed | Static initializer: list head from 004c886d; node 01bd9680 = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Delay_DropDow… |
| `016e8d9a` | GfcControlProp_MenuDelayDraggingDropDown | confirmed | Static initializer: list head from 004c886d; node 01bdcc2c = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Delay_Draggin… |
| `016e8dcc` | GfcControlProp_MenuDelayHideMenu | confirmed | Static initializer: list head from 004c886d; node 01bdd43c = {+4 type &DAT_01760104 (u32), +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Delay_HideMen… |
| `016e8dfe` | GfcControlProp_MenuColor | confirmed | Static initializer: list head from 004c886d; node 01bd91bc = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Menu_Color, +0x10 old head}; t… |
| `016e8e2d` | GfcControlProp_MenuColorInnerDefault | confirmed | Static initializer: list head from 004c886d; node 01bdae30 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Inner_Default, +0x… |
| `016e8e5f` | GfcControlProp_MenuColorInnerSelect | confirmed | Static initializer: list head from 004c886d; node 01bdae80 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Inner_Select, +0x1… |
| `016e8e91` | GfcControlProp_MenuColorInnerDisable | confirmed | Static initializer: list head from 004c886d; node 01bd9460 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Inner_Disable, +0x… |
| `016e8ec3` | GfcControlProp_MenuColorInnerDisableSelect | confirmed | Static initializer: list head from 004c886d; node 01bdcaec = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Inner_DisableSelec… |
| `016e8ef5` | GfcControlProp_MenuColorInnerInactiveSelect | confirmed | Static initializer: list head from 004c886d; node 01bdb038 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Inner_InactiveSele… |
| `016e8f27` | GfcControlProp_MenuColorTextDefault | confirmed | Static initializer: list head from 004c886d; node 01bda3b0 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Text_Default, +0x1… |
| `016e8f59` | GfcControlProp_MenuColorTextSelect | confirmed | Static initializer: list head from 004c886d; node 01bdd284 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Text_Select, +0x10… |
| `016e8f8b` | GfcControlProp_MenuColorTextDisable | confirmed | Static initializer: list head from 004c886d; node 01bdcefc = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Text_Disable, +0x1… |
| `016e8fbd` | GfcControlProp_MenuColorTextDisableSelect | confirmed | Static initializer: list head from 004c886d; node 01bde2f4 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Text_DisableSelect… |
| `016e8fef` | GfcControlProp_MenuColorTextInactiveSelect | confirmed | Static initializer: list head from 004c886d; node 01bdef50 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Text_InactiveSelec… |
| `016e9021` | GfcControlProp_MenuColorTextShadowDefault | confirmed | Static initializer: list head from 004c886d; node 01bddeb8 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_TextShadow_Default… |
| `016e9053` | GfcControlProp_MenuColorTextShadowSelect | confirmed | Static initializer: list head from 004c886d; node 01bd73b4 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_TextShadow_Select,… |
| `016e9085` | GfcControlProp_MenuColorTextShadowDisable | confirmed | Static initializer: list head from 004c886d; node 01bdce70 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_TextShadow_Disable… |
| `016e90b7` | GfcControlProp_MenuColorTextShadowDisableSelect | confirmed | Static initializer: list head from 004c886d; node 01bd9644 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_TextShadow_Disable… |
| `016e90e9` | GfcControlProp_MenuColorTextShadowInactiveSelect | confirmed | Static initializer: list head from 004c886d; node 01bdba30 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_TextShadow_Inactiv… |
| `016e911b` | GfcControlProp_MenuColorImageDefault | confirmed | Static initializer: list head from 004c886d; node 01bdca38 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Image_Default, +0x… |
| `016e914d` | GfcControlProp_MenuColorImageSelect | confirmed | Static initializer: list head from 004c886d; node 01bd8228 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Image_Select, +0x1… |
| `016e917f` | GfcControlProp_MenuColorImageDisable | confirmed | Static initializer: list head from 004c886d; node 01bdcec0 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Image_Disable, +0x… |
| `016e91b1` | GfcControlProp_MenuColorImageDisableSelect | confirmed | Static initializer: list head from 004c886d; node 01bdc974 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Image_DisableSelec… |
| `016e91e3` | GfcControlProp_MenuColorImageInactiveSelect | confirmed | Static initializer: list head from 004c886d; node 01bd9010 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Image_InactiveSele… |
| `016e9215` | GfcControlProp_MenuColorToggleImageDefault | confirmed | Static initializer: list head from 004c886d; node 01bd9a44 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_ToggleImage_Defaul… |
| `016e9247` | GfcControlProp_MenuColorToggleImageSelect | confirmed | Static initializer: list head from 004c886d; node 01bda5a0 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_ToggleImage_Select… |
| `016e9279` | GfcControlProp_MenuColorToggleImageDisable | confirmed | Static initializer: list head from 004c886d; node 01bd8e0c = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_ToggleImage_Disabl… |
| `016e92ab` | GfcControlProp_MenuColorToggleImageDisableSelect | confirmed | Static initializer: list head from 004c886d; node 01bdaca0 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_ToggleImage_Disabl… |
| `016e92dd` | GfcControlProp_MenuColorToggleImageInactiveSelect | confirmed | Static initializer: list head from 004c886d; node 01bd7dc8 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_ToggleImage_Inacti… |
| `016e930f` | GfcControlProp_MenuColorSeparator | confirmed | Static initializer: list head from 004c886d; node 01bd88c0 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_Separator, +0x10 o… |
| `016e9341` | GfcControlProp_MenuColorSeparatorShadow | confirmed | Static initializer: list head from 004c886d; node 01bd9234 = {+4 type c GColor, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Color_SeparatorShadow, +… |
| `016e9373` | GfcControlProp_MenuCombine | confirmed | Static initializer: list head from 004c886d; node 01bdc4bc = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Menu_Combine, +0x10 old head};… |
| `016e93a2` | GfcControlProp_MenuCombineInnerDefault | confirmed | Static initializer: list head from 004c886d; node 01bda0a4 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Inner_Def… |
| `016e93d4` | GfcControlProp_MenuCombineInnerSelect | confirmed | Static initializer: list head from 004c886d; node 01bdd0b0 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Inner_Sel… |
| `016e9406` | GfcControlProp_MenuCombineInnerDisable | confirmed | Static initializer: list head from 004c886d; node 01bdde7c = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Inner_Dis… |
| `016e9438` | GfcControlProp_MenuCombineInnerDisableSelect | confirmed | Static initializer: list head from 004c886d; node 01bdc690 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Inner_Dis… |
| `016e946a` | GfcControlProp_MenuCombineInnerInactiveSelect | confirmed | Static initializer: list head from 004c886d; node 01bd7d8c = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Inner_Ina… |
| `016e949c` | GfcControlProp_MenuCombineTextDefaultDest | confirmed | Static initializer: list head from 004c886d; node 01bda174 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Text_Defa… |
| `016e94ce` | GfcControlProp_MenuCombineTextDefaultFactor | confirmed | Static initializer: list head from 004c886d; node 01bddf1c = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Text_Defa… |
| `016e9500` | GfcControlProp_MenuCombineTextSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bde224 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Text_Sele… |
| `016e9532` | GfcControlProp_MenuCombineTextSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bd785c = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Text_Sele… |
| `016e9564` | GfcControlProp_MenuCombineTextDisableDest | confirmed | Static initializer: list head from 004c886d; node 01bd8cb8 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Text_Disa… |
| `016e9596` | GfcControlProp_MenuCombineTextDisableFactor | confirmed | Static initializer: list head from 004c886d; node 01bd8fac = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Text_Disa… |
| `016e95c8` | GfcControlProp_MenuCombineTextDisableSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bd86f0 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Text_Disa… |
| `016e95fa` | GfcControlProp_MenuCombineTextDisableSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bd9b98 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Text_Disa… |
| `016e962c` | GfcControlProp_MenuCombineTextInactiveSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bd9694 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Text_Inac… |
| `016e965e` | GfcControlProp_MenuCombineTextInactiveSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bdbcc0 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Text_Inac… |
| `016e9690` | GfcControlProp_MenuCombineTextShadowDefaultDest | confirmed | Static initializer: list head from 004c886d; node 01bd8ea8 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_TextShado… |
| `016e96c2` | GfcControlProp_MenuCombineTextShadowDefaultFactor | confirmed | Static initializer: list head from 004c886d; node 01bd7ed4 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_TextShado… |
| `016e96f4` | GfcControlProp_MenuCombineTextShadowSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bded20 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_TextShado… |
| `016e9726` | GfcControlProp_MenuCombineTextShadowSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bdcb8c = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_TextShado… |
| `016e9758` | GfcControlProp_MenuCombineTextShadowDisableDest | confirmed | Static initializer: list head from 004c886d; node 01bd8bd4 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_TextShado… |
| `016e978a` | GfcControlProp_MenuCombineTextShadowDisableFactor | confirmed | Static initializer: list head from 004c886d; node 01bd978c = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_TextShado… |
| `016e97bc` | GfcControlProp_MenuCombineTextShadowDisableSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bda054 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_TextShado… |
| `016e97ee` | GfcControlProp_MenuCombineTextShadowDisableSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bdccb8 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_TextShado… |
| `016e9820` | GfcControlProp_MenuCombineTextShadowInactiveSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bd8fe8 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_TextShado… |
| `016e9852` | GfcControlProp_MenuCombineTextShadowInactiveSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bde238 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_TextShado… |
| `016e9884` | GfcControlProp_MenuCombineImageDefaultDest | confirmed | Static initializer: list head from 004c886d; node 01bd8a44 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Image_Def… |
| `016e98b6` | GfcControlProp_MenuCombineImageDefaultFactor | confirmed | Static initializer: list head from 004c886d; node 01bd8e44 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Image_Def… |
| `016e98e8` | GfcControlProp_MenuCombineImageSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bd7748 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Image_Sel… |
| `016e991a` | GfcControlProp_MenuCombineImageSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bdb990 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Image_Sel… |
| `016e994c` | GfcControlProp_MenuCombineImageDisableDest | confirmed | Static initializer: list head from 004c886d; node 01bde24c = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Image_Dis… |
| `016e997e` | GfcControlProp_MenuCombineImageDisableFactor | confirmed | Static initializer: list head from 004c886d; node 01bdca4c = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Image_Dis… |
| `016e99b0` | GfcControlProp_MenuCombineImageDisableSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bd8d94 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Image_Dis… |
| `016e99e2` | GfcControlProp_MenuCombineImageDisableSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bd8824 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Image_Dis… |
| `016e9a14` | GfcControlProp_MenuCombineImageInactiveSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bdbe28 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Image_Ina… |
| `016e9a46` | GfcControlProp_MenuCombineImageInactiveSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bdc3cc = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Image_Ina… |
| `016e9a78` | GfcControlProp_MenuCombineToggleImageDefaultDest | confirmed | Static initializer: list head from 004c886d; node 01bdda14 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_ToggleIma… |
| `016e9aaa` | GfcControlProp_MenuCombineToggleImageDefaultFactor | confirmed | Static initializer: list head from 004c886d; node 01bdd024 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_ToggleIma… |
| `016e9adc` | GfcControlProp_MenuCombineToggleImageSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bd93c4 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_ToggleIma… |
| `016e9b0e` | GfcControlProp_MenuCombineToggleImageSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bdc41c = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_ToggleIma… |
| `016e9b40` | GfcControlProp_MenuCombineToggleImageDisableDest | confirmed | Static initializer: list head from 004c886d; node 01bdbc10 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_ToggleIma… |
| `016e9b72` | GfcControlProp_MenuCombineToggleImageDisableFactor | confirmed | Static initializer: list head from 004c886d; node 01bd9fc8 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_ToggleIma… |
| `016e9ba4` | GfcControlProp_MenuCombineToggleImageDisableSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bd9d04 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_ToggleIma… |
| `016e9bd6` | GfcControlProp_MenuCombineToggleImageDisableSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bd8d6c = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_ToggleIma… |
| `016e9c08` | GfcControlProp_MenuCombineToggleImageInactiveSelectDest | confirmed | Static initializer: list head from 004c886d; node 01bdc988 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_ToggleIma… |
| `016e9c3a` | GfcControlProp_MenuCombineToggleImageInactiveSelectFactor | confirmed | Static initializer: list head from 004c886d; node 01bdc5f0 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_ToggleIma… |
| `016e9c6c` | GfcControlProp_MenuCombineSeparator | confirmed | Static initializer: list head from 004c886d; node 01bdc520 = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Separator… |
| `016e9c9e` | GfcControlProp_MenuCombineSeparatorShadow | confirmed | Static initializer: list head from 004c886d; node 01bd89ec = {+4 type c GColorCombine, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Combine_Separator… |
| `016e9cd0` | GfcControlProp_MenuImageCheckItemDefault | confirmed | Static initializer: list head from 004c886d; node 01bde568 = {+4 type p<c GBitmap>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Image_CheckItem_Defa… |
| `016e9d02` | GfcControlProp_MenuImageCheckItemDisable | confirmed | Static initializer: list head from 004c886d; node 01bdd1dc = {+4 type p<c GBitmap>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Image_CheckItem_Disa… |
| `016e9d34` | GfcControlProp_MenuImageCheckItemToggle | confirmed | Static initializer: list head from 004c886d; node 01bd8b98 = {+4 type p<c GBitmap>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Image_CheckItem_Togg… |
| `016e9d66` | GfcControlProp_MenuImageCheckItemToggleDisable | confirmed | Static initializer: list head from 004c886d; node 01bd82b4 = {+4 type p<c GBitmap>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Image_CheckItem_Togg… |
| `016e9d98` | GfcControlProp_MenuImageRadioItemDefault | confirmed | Static initializer: list head from 004c886d; node 01bdedc0 = {+4 type p<c GBitmap>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Image_RadioItem_Defa… |
| `016e9dca` | GfcControlProp_MenuImageRadioItemDisable | confirmed | Static initializer: list head from 004c886d; node 01bdc110 = {+4 type p<c GBitmap>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Image_RadioItem_Disa… |
| `016e9dfc` | GfcControlProp_MenuImageRadioItemToggle | confirmed | Static initializer: list head from 004c886d; node 01bd807c = {+4 type p<c GBitmap>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Image_RadioItem_Togg… |
| `016e9e2e` | GfcControlProp_MenuImageRadioItemToggleDisable | confirmed | Static initializer: list head from 004c886d; node 01bd9298 = {+4 type p<c GBitmap>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Image_RadioItem_Togg… |
| `016e9e60` | GfcControlProp_MenuFont | confirmed | Static initializer: list head from 004c886d; node 01bde57c = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Menu_Font, +0x10 old head}; th… |
| `016e9e8f` | GfcControlProp_MenuFontDefault | confirmed | Static initializer: list head from 004c886d; node 01bdc058 = {+4 type p<c GFont>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Font_Default, +0x10 ol… |
| `016e9ec1` | GfcControlProp_MenuFontSelect | confirmed | Static initializer: list head from 004c886d; node 01bd9fa0 = {+4 type p<c GFont>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Font_Select, +0x10 old… |
| `016e9ef3` | GfcControlProp_MenuBinding | confirmed | Static initializer: list head from 004c886d; node 01bda550 = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Menu_Binding, +0x10 old head};… |
| `016e9f22` | GfcControlProp_MenuBindingSelectItemLeft | confirmed | Static initializer: list head from 004c886d; node 01bd8ed0 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Binding_Select… |
| `016e9f54` | GfcControlProp_MenuBindingSelectItemRight | confirmed | Static initializer: list head from 004c886d; node 01bdb430 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Binding_Select… |
| `016e9f86` | GfcControlProp_MenuBindingSelectItemUp | confirmed | Static initializer: list head from 004c886d; node 01bdb408 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Binding_Select… |
| `016e9fb8` | GfcControlProp_MenuBindingSelectItemDown | confirmed | Static initializer: list head from 004c886d; node 01bdd928 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Binding_Select… |
| `016e9fea` | GfcControlProp_MenuBindingSelectItemHome | confirmed | Static initializer: list head from 004c886d; node 01bd80f4 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Binding_Select… |
| `016ea01c` | GfcControlProp_MenuBindingSelectItemEnd | confirmed | Static initializer: list head from 004c886d; node 01bdbd38 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Binding_Select… |
| `016ea04e` | GfcControlProp_MenuBindingExecute | confirmed | Static initializer: list head from 004c886d; node 01bd7490 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Binding_Execut… |
| `016ea080` | GfcControlProp_MenuBindingCancel | confirmed | Static initializer: list head from 004c886d; node 01bdef78 = {+4 type p<c GInputBinding>, +8 owner GFC_Control_Theme, +0xc name GFC_Control_Menu_Binding_Cancel… |
| `016ea0b2` | GfcControlProp_MenuMessage | confirmed | Static initializer: list head from 004c886d; node 01bda740 = {+4 type p<c GPropertyNamespace>, +8 owner 0, +0xc name GFC_Control_Menu_Message, +0x10 old head};… |
| `016ea0e1` | GfcControlProp_MenuMessageOnSelect | confirmed | Static initializer: list head from 004c886d; node 01bd7db4 = {+4 type &DAT_0175f744 (bytes 'v'), +8 owner 0, +0xc name GFC_Control_Menu_Message_OnSelect, +0x10… |
| `016ea110` | GfcControlProp_MenuMessageOnUISelect | confirmed | Static initializer: list head from 004c886d; node 01bd8c4c = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_OnUISelect, +0x10 old h… |
| `016ea13f` | GfcControlProp_MenuMessageOnItemMessage | confirmed | Static initializer: list head from 004c886d; node 01bdbf2c = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_OnItemMessage, +0x10 ol… |
| `016ea16e` | GfcControlProp_MenuMessageOnItemToggle | confirmed | Static initializer: list head from 004c886d; node 01bdac50 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_OnItemToggle, +0x10 old… |
| `016ea19d` | GfcControlProp_MenuMessageOnTriggerItem | confirmed | Static initializer: list head from 004c886d; node 01bda668 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_OnTriggerItem, +0x10 ol… |
| `016ea1cc` | GfcControlProp_MenuMessageOnTriggerPopup | confirmed | Static initializer: list head from 004c886d; node 01bd7df0 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_OnTriggerPopup, +0x10 o… |
| `016ea1fb` | GfcControlProp_MenuMessageOnCloseMenu | confirmed | Static initializer: list head from 004c886d; node 01bdad04 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_OnCloseMenu, +0x10 old … |
| `016ea22a` | GfcControlProp_MenuMessageOnUICloseMenu | confirmed | Static initializer: list head from 004c886d; node 01bdcfa4 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_OnUICloseMenu, +0x10 ol… |
| `016ea259` | GfcControlProp_MenuMessageOnCloseStickyMenu | confirmed | Static initializer: list head from 004c886d; node 01bda188 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_OnCloseStickyMenu, +0x1… |
| `016ea288` | GfcControlProp_MenuMessageDoPopupMenu | confirmed | Static initializer: list head from 004c886d; node 01bd7c88 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_DoPopupMenu, +0x10 old … |
| `016ea2b7` | GfcControlProp_MenuMessageDoCloseMenu | confirmed | Static initializer: list head from 004c886d; node 01bddb50 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_DoCloseMenu, +0x10 old … |
| `016ea2e6` | GfcControlProp_MenuMessageDoHit | confirmed | Static initializer: list head from 004c886d; node 01bd7fb4 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_DoHit, +0x10 old head};… |
| `016ea315` | GfcControlProp_MenuMessageDoHitItem | confirmed | Static initializer: list head from 004c886d; node 01bd7c74 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_DoHitItem, +0x10 old he… |
| `016ea344` | GfcControlProp_MenuMessageDoSelectItem | confirmed | Static initializer: list head from 004c886d; node 01bd78b4 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_DoSelectItem, +0x10 old… |
| `016ea373` | GfcControlProp_MenuMessageDoSelectItemUp | confirmed | Static initializer: list head from 004c886d; node 01bd9528 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_DoSelectItemUp, +0x10 o… |
| `016ea3a2` | GfcControlProp_MenuMessageDoSelectItemDown | confirmed | Static initializer: list head from 004c886d; node 01bd75c4 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_DoSelectItemDown, +0x10… |
| `016ea3d1` | GfcControlProp_MenuMessageDoSelectItemLeft | confirmed | Static initializer: list head from 004c886d; node 01bdb444 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_DoSelectItemLeft, +0x10… |
| `016ea400` | GfcControlProp_MenuMessageDoSelectItemRight | confirmed | Static initializer: list head from 004c886d; node 01bdcd80 = {+4 type &DAT_0175f744 (v), +8 owner 0, +0xc name GFC_Control_Menu_Message_DoSelectItemRight, +0x1… |
| `0041e58b` | ArrangeElementGeometry | likely | Reads a metric through virtual slot +0x18 of param_2+0x24 (default 10), then tests keyed cases through FUN_004c0ec3. Some matches return immediately through th… |
| `0041eea4` | GetElementMargin | likely | Fills param_2 with this element's margin rectangle. It loads four ints from +0x60/+0x64/+0x68/+0x6c plus the 16-bit selector at +0x5c, substitutes the constant… |
| `0042050b` | TranslateSubtree | likely | Builds a stack GObjectIterator on the subobject at +0x24 through slot +0x50 with argument 3, then visits its children. For each child whose +0x64 flags have bi… |
| `00546901` | ReadTargetMapping | likely | Reads the TargetId property id from the node (FUN_0054616f); if it is missing the function returns 0 without building anything. Otherwise it reads the optional… |
| `00546fe9` | ReadPropertyMappingRanges | likely | Guarded by the non-null test 004c6e64 on the data node param_1. It calls slot +0x80 of the interface object at this+0x14 (ECX = this+0x14), then resolves the L… |
| `0054845e` | ReadMessageBlock | likely | Thiscall (ECX=this) with one stack argument, ret 4. Guarded by the non-null test on the data node, it interns the wide string L"Message" (004e889c) and passes … |
| `00605fa3` | CallVtbl34ThenChild130 | likely | Calls virtual slot +0x34 of the receiver with argument 1 and, only when that returns non-zero, tail-calls virtual slot +0x130 of the child object at this+0xf4 … |
| `006060b7` | DtorVtables17806A8 | likely | Destructor body: installs the class vtable triple (PTR_FUN_017806a8 at +0, PTR_LAB_017806a4 at +4, PTR_LAB_01780590 at +0x24), destroys two embedded members th… |
| `0060622d` | ComputeObjectSizeWithChildren | likely | Size query: starts from FUN_0040eb1d (base size), then for each child object at this+0x108 and this+0x130 calls child virtual slot +0x1c and, only when it retu… |
| `00606284` | AssignGStringAt0x100 | likely | Setter that forwards its single argument to the GString-style copy-assign FUN_004e52ed applied to the receiver's member at this+0x100 (disassembly: add ecx,0x1… |
| `006062f6` | CallChild108Vtbl38 | likely | 11-byte pass-through thunk (add ecx,0x108; mov eax,[ecx]; jmp [eax+0x38]): forwards the caller's arguments to virtual slot +0x38 of the sub-object at this+0x10… |
| `00606322` | CallChild130Vtbl38 | likely | 11-byte pass-through thunk (add ecx,0x130; mov eax,[ecx]; jmp [eax+0x38]): forwards all caller arguments to virtual slot +0x38 of the sub-object at this+0x130,… |
| `006064f5` | PackFormatFrom0xF0 | likely | Pure flag packer (no calls): from this+0xf0, bits 0x30000 select 0x1000/0x1010/0x1020 (default 0x1000), bits 0x3000000 equal 0x2000000/0x3000000 add 0x100/0x20… |
| `00606694` | CallChild108Vtbl34 | likely | 11-byte pass-through thunk (add ecx,0x108; mov eax,[ecx]; jmp [eax+0x34]): forwards all arguments to virtual slot +0x34 of the sub-object at this+0x108 with th… |
| `0060669f` | CallChild108Vtbl38_2 | likely | 11-byte pass-through thunk (add ecx,0x108; mov eax,[ecx]; jmp [eax+0x38]): forwards all arguments to virtual slot +0x38 of the sub-object at this+0x108 with th… |
| `006066cb` | CallChild108Vtbl44_SetText | likely | 11-byte pass-through thunk (add ecx,0x108; mov eax,[ecx]; jmp [eax+0x44]): forwards all arguments to virtual slot +0x44 of the sub-object at this+0x108 with th… |
| `006066d6` | CallChild108Vtbl48_GetText | likely | 11-byte pass-through thunk (add ecx,0x108; mov eax,[ecx]; jmp [eax+0x48]): forwards all arguments to virtual slot +0x48 of the sub-object at this+0x108 with th… |
| `006066e1` | CallChild108Vtbl50 | likely | 11-byte pass-through thunk (add ecx,0x108; mov eax,[ecx]; jmp [eax+0x50]): forwards all arguments to virtual slot +0x50 of the sub-object at this+0x108 with th… |
| `006067f7` | GetField140 | likely | 7-byte getter (mov eax,[ecx+0x140]; ret) that returns the dword field at this+0x140. |
| `00606818` | GetField144 | likely | 7-byte getter (mov eax,[ecx+0x144]; ret) that returns the dword field at this+0x144. |
| `0060681f` | ReturnClassInfoBlock01b1d158 | likely | 6-byte accessor that returns the constant address of the global data block at 0x01b1d158 - the whole body is the address-of, no arguments or side effects. |
| `00606966` | Forward10ArgTo0067506d | likely | 8-argument shim (ret 0x20) that calls FUN_0067506d with ten arguments: its first four arguments, then 0, then its fifth, then the two dwords of the record poin… |
| `00606c1e` | CallVtbl34OnE8 | likely | Forwards seven stack arguments to virtual slot +0x34 of the object at this+0xe8, prefixed with the constant 0xffffffff as its first argument. |
| `00606c47` | CallVtbl30OnE8 | likely | Forwards seven stack arguments to virtual slot +0x30 of the object at this+0xe8, prefixed with the constant 0xffffffff as its first argument - sibling of 00606… |
| `00606dc2` | GetField130 | likely | 7-byte getter (mov eax,[ecx+0x130]; ret) returning the dword field at this+0x130. |
| `00606e1c` | GetSelectedIndex124 | likely | 7-byte getter (mov eax,[ecx+0x124]; ret) returning the dword field at this+0x124. |
| `00606fdc` | SetInterfaceAt0x24 | likely | Stores param_2 at this+0x24 and then this+0x28 = param_2->vtable[0x288]() when param_2 is non-null, 0 otherwise - i.e. it installs an interface pointer togethe… |
| `00607001` | StoreToken01b1d824 | likely | 13-byte store: mov eax,[esp+4]; mov ecx,[0x1b1d824]; mov [eax],ecx; ret - writes the global data word DAT_01b1d824 to the caller's out pointer. |
| `00607027` | StoreToken01b1d8dc | likely | 13-byte store that writes the global data word DAT_01b1d8dc to the caller's out pointer; no other work. |
| `0060704d` | StoreToken01b1d9bc | likely | 13-byte store that writes the global data word DAT_01b1d9bc to the caller's out pointer; no other work. |
| `00607073` | StoreToken01b1da9c | likely | 13-byte store that writes the global data word DAT_01b1da9c to the caller's out pointer; no other work. |
| `0060711d` | WriteThisAndToken01b1ddbc | likely | 25-byte accessor: if the third argument is non-null it stores the receiver (ECX) through it, and it always stores the global data word DAT_01b1ddbc through the… |
| `00607291` | StoreToken01b1df34 | likely | 13-byte store that writes the global data word DAT_01b1df34 to the caller's out pointer; no other work. |
| `00607329` | DrainVtbl78 | likely | Loop: calls the receiver's virtual slot +0x78 with argument 0 and repeats while the returned byte is non-zero; then returns 1 (the epilogue is inc al on the ze… |
| `00607371` | StoreToken01b1dff4 | likely | 13-byte store that writes the global data word DAT_01b1dff4 to the caller's out pointer; no other work. |
| `0060737e` | WriteThisAndToken01b1dff4 | likely | 25-byte accessor: if the third argument is non-null it stores the receiver (ECX) through it, and always stores the global data word DAT_01b1dff4 through the se… |
| `0060754a` | CopyComboBoxItemData | likely | Copies the item payload from param_2 to this: after the (stub) check FUN_004c6e7c and the container validation through slot +0x2c of the object at this+0x14, i… |
| `006076a3` | StoreToken01b1e0b4 | likely | 13-byte store that writes the global data word DAT_01b1e0b4 to the caller's out pointer; no other work. |
| `006076b0` | WriteThisAndToken01b1e0b4 | likely | 25-byte accessor: if the third argument is non-null it stores the receiver (ECX) through it, and always stores the global data word DAT_01b1e0b4 through the se… |
| `006077b5` | SetStringNotify | likely | Method of an embedded sub-object (owner accessed at negative offsets): compares the string held through *(this+8) with *param_2 using FUN_004ca923 (dword/half-… |
| `0060781f` | SetIntAndFlagNotify | likely | Method of an embedded sub-object (owner at negative offsets): when this+0x10 differs from the int param_2 or the byte at this+0x14 differs from param_3, it sto… |
| `00607920` | StoreToken01b1e374 | likely | 13-byte store (mov eax,[esp+4]; mov ecx,[0x1b1e374]; mov [eax],ecx; ret) that copies the global data word DAT_01b1e374 to the caller's out pointer; it has no o… |
| `006079dd` | StoreToken01b1e42c | likely | 13-byte store (mov eax,[esp+4]; mov ecx,[0x1b1e42c]; mov [eax],ecx; ret) that copies the global data word DAT_01b1e42c to the caller's out pointer; it has no o… |
| `016d9df9` | DescriptorPush_01b1d28c | likely | Pushes the address of static descriptor 01b1d28c onto list DAT_01bd70c0 through 0064dfc0 (ECX = node 01bd0dc8, node+0 = descriptor, node+4 = old head). |
| `016d9e09` | DescriptorPush_01b1d34c | likely | Pushes the address of static descriptor 01b1d34c onto list DAT_01bd70c0 through 0064dfc0 (ECX = node 01bd2218, node+0 = descriptor, node+4 = old head). |
| `016e6825` | RegisterGTreeCtrlNode | likely | Thiscall with ECX = static node DAT_01bd312c. Pushes 0, &PTR_s_GFC_CF_TREE_HIDEROOTINDENT (01b21ca8, a style-table entry) and the string 'GTreeCtrl' (01780450)… |
| `016e80e5` | RegisterGWindowCtrlNode | likely | Static init: ECX = static node 01bd0b08; pushes 1 (flag), style table &PTR_s_GFC_CF_WINDOW_POS_HFIXED_01b21ec8 and the string GWindowCtrl, then calls 004c6281,… |
| `016e84ce` | RegisterGHeaderCtrlNode | likely | Static init: ECX = static node 01bdb74c; pushes 0 (flag), style table &PTR_s_GFC_CF_HEADER_HORIZONTAL_01b22b70 and the string GHeaderCtrl, then calls 004c6281 … |

### `engine.gfc.controls`

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00418ce1` | StoreControlProperties | likely | Control base-class writer: after the shared pre-check FUN_005074BF it stores the whole control state into the data node - 'CtrlFlags' (with the GFC_CF_NOTVISIB… |
| `00418f4d` | SetStyleId | likely | Sets the 16-bit field at +0xB0 when it differs from the argument (a zero argument is replaced by the default DAT_01BC3570), then builds a seven-dword notificat… |
| `00418fc6` | ResolveLinkedObject | likely | Resolves a linked object: returns 0 when the manager link at +0x44 is null; when that manager's +0x11C field equals 1 it prefers this object's +0xBC link and, … |
| `00419123` | WalkChildrenRecursive | likely | When bit 0x8000000 of the flag word at +0x64 is set it composes the client rectangle through FUN_0040F723 into a local 16-byte buffer (the handle argument is p… |
| `0041925b` | FindDirectionalNeighbor | likely | Directional neighbour search over the child controls: builds a stack GObjectIterator (kind 1) and for each child, for each of nine anchor points computed from … |
| `0041950b` | NotifyIfFlagged0E | likely | When the argument byte is non-zero, calls FUN_0041177A with the constant 0xE; otherwise returns immediately. |
| `004195ff` | ClearChildField48 | likely | Walks the direct children through a stack GObjectIterator (slot +0x50 with kind 3), zeroes the dword at +0x48 of every child found by FUN_00407B86 and destroys… |
| `004196d8` | SetTreeObjectRecursive | likely | Attaches an object to the control and propagates it down the tree: saves the old object's slot +0x18 low byte, retains the new object through its slot +0xC (th… |
| `00419790` | SetField40Recursive | likely | Stores param_2 at +0x40, then (unless the +0x38 link is set, in which case the value becomes this+0x24) walks the children with a stack GObjectIterator and rec… |
| `004197f3` | ClearField40Recursive | likely | Clears the +0x40 field of this control and recursively of all descendants: stores 0 at +0x40, walks the children with a stack GObjectIterator and calls itself … |
| `00419846` | ScalePositionToTarget | likely | Maps a source position onto the tree target: when the manager at +0x44 is in state 2 and FUN_0040EDC6/FUN_0040EE03/FUN_00418FC6 all resolve and the resolved ob… |

### `engine.gfc.core`

**CtrlManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0043a470` | CtrlManager::CtrlManager | confirmed | Runs base and record initialization, installs the CtrlManager vtable set, clears its manager fields and lists, sets +0xec to 0x40, releases the referenced fiel… |
| `0043a78f` | CtrlManager::CtrlManagerVariant | likely | Runs the base and record initialization, installs the same vtable set as the primary constructor, initializes the lists and manager fields, releases the refere… |
| `0043d7ea` | CtrlManager::~CtrlManager | likely | Rewrites the CtrlManager vtable fields, tears down the manager entry list and +0xf4 buffer, destroys the 0x30-stride vector and tree containers, releases inter… |
| `0043f971` | CtrlManager::ScalarDeletingDtor | likely | Calls CtrlManager destructor body 0043D7EA; when param_2 bit 0 is set, calls the shared free helper through thunk 00408259; otherwise returns param_1. The func… |

**FlashManager** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d2040` | FlashManager::FlashManager | confirmed | Constructor of the Flash movie manager: installs FlashManagerInterface::vftable (interface base), then overrides it with FlashManager::vftable, installs the So… |
| `008d20a0` | FlashManager::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (slot [0] of the Flash movie manager vtable): runs the out-of-line destructor body FUN_00d56540 and frees the object through th… |

**GBatchBlit** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0048006a` | RegisterGBatchBlit | confirmed | Registers the GFC class "GBatchBlit" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GBitmap** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004860fc` | GBitmap::CopyFrom | likely | Copy/assign routine of the GBitmap class (its caller 004945D3 is the GBitmap class-registry record factory, whose create op allocates 0x30 bytes and whose copy… |
| `0048612f` | GBitmap::~GBitmap | likely | Destructor of the GBitmap class: releases the interface at +0x2C through vtable slot +0x10 (zeroing the field, then a now-dead second guarded release) and free… |

**GBitmapAccessWrapper** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0042afb6` | GBitmapAccessWrapper::GBitmapAccessWrapper | confirmed | Constructor of the GBitmapAccessWrapper class: runs the base initializer 0041493a, installs the GBitmapAccessWrapper vtable at +0, allocates 0x34 bytes through… |
| `0041507f` | GBitmapAccessWrapper::ForwardAccessSlot78 | likely | When the data access object at +8 is present, tail-calls its vtable slot +0x78 with no arguments and returns the result; otherwise returns 0. |
| `0041508e` | GBitmapAccessWrapper::ForwardAccessSlot90 | likely | Returns 0 when param_2 is null or no data access is held at +8; otherwise forwards param_2 to the access object's vtable slot +0x90 and returns its result. |
| `00420e36` | GBitmapAccessWrapper::Ops40 | likely | Opcode-dispatched construction of 40-byte GBitmapAccessWrapper objects with the same shape as 00420db2: ops 2/3 allocate 0x28 bytes with FUN_00d833a6, ops 0..3… |

**GBitmapFont** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0047e32e` | GBitmapFontCtor | likely | Constructor seen from outside: calls FUN_004d99d1 for the base and installs primary vtable 01768d10 at offset 0 and secondary vtable 01768d0c at +4, returning … |
| `00481da2` | GBitmapFont::CopyCtor | likely | Calls base copy initializer FUN_004e68fb(param_2), installs the primary vtable pointer 01768d10 at offset 0 and secondary pointer 01768d0c at +4, and returns p… |

**GBitmapIEBmp** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0047e586` | RegisterGBitmapIEBmp | confirmed | Registers the GFC class "GBitmapIEBmp" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GBitmapIEPcx** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0047e6f5` | RegisterGBitmapIEPcx | confirmed | Registers the GFC class "GBitmapIEPcx" (base size 0x18) with the class registry. |

**GBitmapIEPpm** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0047eb49` | RegisterGBitmapIEPpm | confirmed | Registers the GFC class "GBitmapIEPpm" (base size 0x18) with the class registry. |

**GBitmapIETga** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0047ec59` | RegisterGBitmapIETga | confirmed | Registers the GFC class "GBitmapIETga" (base size 0x18) with the class registry. |

**GColor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0047ecdf` | RegisterGColor | confirmed | Registers the GFC class "GColor" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GColorCombine** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0047ec99` | RegisterGColorCombine | confirmed | Registers the GFC class "GColorCombine" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GCursor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0047fef4` | RegisterGCursor | confirmed | Registers the GFC class "GCursor" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GDataAccess** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004c537a` | GDataAccess::StoreGlobalPair_01cc42b4 | confirmed | Virtual method at GDataAccess::vftable slot 1: stores ECX (this) to *arg2 when arg2 is nonnull, then stores the dword at global 0x01cc42b4 to *arg1 (stdcall, r… |

**GDataAccessImpl** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00428d6c` | GDataAccessImpl::GDataAccessImpl | confirmed | Constructor of the GFC data-access implementation: installs the GDataAccessImpl vtable at +0, zeroes +8, +0xC and +0x10 and stores the argument at +0x14; retur… |
| `0042af23` | GDataAccessImplCopy | confirmed | Copy-style constructor of a data-access object: installs the GDataAccessImpl vtable at +0, copies the source's embedded 32-byte-element vector at source+4 thro… |
| `0042af4a` | GDataAccessImplCreate | confirmed | Inline factory: allocates 0x34 bytes through 00409351, constructs a data-access object in place (GObjectBase base, param_1 at +0x14, the intermediate 0x175d1a0… |
| `004c5549` | GDataAccessImpl::StoreGlobalPair_01cc49bc | confirmed | Virtual method at GDataAccessImpl::vftable slot 1 (stdcall, ret 8): same shape as 004c5523 with the global at 0x01cc49bc. |
| `00428d89` | GDataAccessImpl::ScalarDeletingDtor | likely | MSVC scalar deleting destructor for a data-access object: runs the destructor chain 00424eca (whose own analysis rewrites the GInterface vtable) and frees the … |
| `004c5562` | GDataAccessImpl::CompareSetField14 | likely | Shared virtual implementation (thiscall, object in ECX, stack args value and expected): sets the dword at this+0x14 to the value only when it equals the expect… |
| `004c557b` | GDataAccessImpl::GetField14 | likely | Shared virtual getter (4 bytes: mov eax,[ecx+0x14]; ret): returns the dword at this+0x14. The same body is slot 8 of GDataAccessImpl and of GStringObject +0x18… |

**GDataAccessImplBase** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004c5523` | GDataAccessImplBase::StoreGlobalPair_01cc490c | confirmed | Virtual method at GDataAccessImplBase::vftable slot 1 (stdcall, ret 8): stores ECX to *arg2 when nonnull, then stores the dword at global 0x01cc490c to *arg1. |

**GDataAccessWrapper** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00414952` | GDataAccessWrapper::GetOwner | confirmed | Returns the GObject pointer stored at this+0x10. |
| `00414956` | GDataAccessWrapper::GetDataId | confirmed | Writes the 16-bit GPropertyId stored at this+0xc through the caller's return buffer (return-by-value of a 16-bit id). |
| `00414964` | GDataAccessWrapper::GetDataAccess | confirmed | Returns the GDataAccess pointer stored at this+8. |
| `0041493a` | GDataAccessWrapper::GDataAccessWrapper | likely | Default constructor: writes the shared vtable PTR_FUN_0175dc60, zeroes +8 (the data-access pointer), the 16-bit +0xc (the data id) and +0x10/+0x14. |
| `004149a9` | GDataAccessWrapper::GDataAccessWrapper3 | likely | Constructor taking a 16-bit property id: writes the shared vtable, zeroes +8/+0x10/+0x14 and stores the id at +0xc. |
| `004149c8` | GDataAccessWrapper::~GDataAccessWrapper | likely | Destructor (listed as called from 46 EH unwind funclets): when this+0x14 is 1 or 2 it notifies the data-access interface at +8 through its virtual +0x5c passin… |
| `00414a48` | GDataAccessWrapper::SetOwner | likely | Stores param_2 at +0x10 (the owner field read back by the neighbouring getter 00414952) and resets the value descriptor: +0x14 = 0, +0x18/+0x1C/+0x20 = 0, +0x2… |
| `00414ad7` | GDataAccessWrapper::SetOwnerValueKind2 | likely | Stores param_2 at +0x10 (owner) and param_3 at +0x18, marks the value descriptor with kind 2 (+0x14 = 2), then performs the same id/access fixup as the kind 0 … |
| `00414b4f` | GDataAccessWrapper::SetOwnerValueKind1 | likely | Stores param_2 at +0x10 (owner), param_3/param_4/param_5 at +0x18/+0x1C/+0x20 and param_6 at +0x24, marks the descriptor kind 1 (+0x14 = 1), then does the same… |
| `00414bcb` | GDataAccessWrapper::LoadFromData | likely | Reads a binding out of a data node: asks param_2's vtable slot +0x64 for the node selected by param_3 and reads that node's 16-bit 'Id' attribute into the low … |
| `00414d77` | GDataAccessWrapper::CopyBindingFrom | likely | Copies another wrapper's binding into this one: when the source (param_3) has no 16-bit id and holds a data access at +8, resolves a value through FUN_004fdb5f… |
| `00414dd0` | GDataAccessWrapper::ClearBoundValue | likely | When a data access is held at +8, builds a scoped access context through FUN_004c6dcb/FUN_004cd10a, asks the access's vtable slot 2 for the data node of that c… |
| `00414e1d` | GDataAccessWrapper::SetDataAccess | likely | Attaches param_2 as the wrapper's data access: when a value descriptor is already recorded (+0x14 == 1 or 2) it first pushes that descriptor to the old access'… |
| `00414f07` | GDataAccessWrapper::SetDataAccessIfChanged | likely | Public attach entry: returns 1 immediately when the wrapper has no owner (+0x10 == 0) or when the candidate access param_2 is already held at +8; otherwise cle… |
| `00414f29` | GDataAccessWrapper::SetDataId | likely | Sets the 16-bit property id at +0xC. When the id is non-zero and the wrapper is not already consistent it builds an access object for the current owner through… |
| `0041b3b3` | GDataAccessWrapper::ScalarDeletingDtor | likely | Calls FUN_004149C8 on this, frees this through _free when param_2 bit 0 is set, and returns this otherwise. |

**GDate** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004c43eb` | GDate::SetDate | confirmed | Thiscall (this in ECX) with an int, a MonthType enum and a byte: stores the int at this+8, the enum at this+4, and the byte at this+0. |

**GDateTime** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004eaca1` | GDateTime::ReadJulianTime | confirmed | Looks up the GDateTime attribute group on the object in param_2 (key L"GDateTime" via 004e4e6d, vtable +0x68 on the interface at *param_1). When found it reads… |
| `004ead09` | GDateTime::WriteJulianTime | confirmed | Looks up the GDateTime attribute group on the object in param_2 (key L"GDateTime" via 004e4e6d, vtable +0x6c on the interface at *param_1). When found it write… |

**GDisplay** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0048019a` | RegisterGDisplay | confirmed | Registers the GFC class "GDisplay" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GDrawDevice** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0048011c` | RegisterGDrawDevice | confirmed | Registers the GFC class "GDrawDevice" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GDrawSurface** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004800c3` | RegisterGDrawSurface | confirmed | Registers the GFC class "GDrawSurface" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GDrawText** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0048104e` | RegisterGDrawText | confirmed | Registers the GFC class "GDrawText" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GEvent** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00705570` | GEventDefaultConstructorClosure | confirmed | Invokes the GEvent default-construction routine with zero initialization argument. |

**GFont** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00481ad1` | RegisterGFont | confirmed | Registers the GFC class "GFont" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GFontDesc** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00481a78` | RegisterGFontDesc | confirmed | Registers the GFC class "GFontDesc" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GFontDevice** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00481bda` | RegisterGFontDevice | confirmed | Registers the GFC class "GFontDevice" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GFontDeviceWin32** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00481d4a` | RegisterGFontDeviceWin32 | confirmed | Registers the GFC class "GFontDeviceWin32" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GFontWrapper** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00481e59` | RegisterGFontWrapper | confirmed | Registers the GFC class "GFontWrapper" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GFormCtrl** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0043edc1` | GFormCtrl::ProcessControlMessage | confirmed | Reads the 16-bit message id at param_2+0x1C and compares it with global message-id constants. It routes matched ids to this object vtable slots from +0x130 thr… |

**GFormattedString** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0048232e` | GFormattedString::WriteGlobalAndOptionalArgument | confirmed | Writes DAT_01af9c8c into *param_2 and, when param_3 is non-null, writes param_1 into *param_3. |
| `00482520` | GFormattedString::FetchRecordDword | confirmed | Loads the dword stored at *(this+0x1C) and writes it through the out-parameter, returning a value whose low byte is 1 (the success flag kept in AL). |

**GFormattedStringAccess** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00482308` | GFormattedStringAccess::WriteGlobalAndOptionalArgument | confirmed | Writes DAT_01af9bcc into *param_2 and, when param_3 is non-null, writes param_1 into *param_3. |

**GHBoxLayout** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0043b8ed` | GHBoxLayout::GHBoxLayout | confirmed | Calls layout base constructor 00435c70 with the supplied argument, writes the GHBoxLayout vtable at object offsets +0 and +4, and returns this. |
| `0043b90c` | GHBoxLayout::ForwardTo5017F8 | confirmed | The stack adapter forwards its receiver and two stack arguments to 005017f8, cleans the temporary 12-byte argument area, then returns with an 8-byte callee-pop… |
| `0043b920` | GHBoxLayout::AdjustorThunkToDestructor | confirmed | Adjusts ECX by subtracting 4, then jumps directly to 0043bb36; it does not contain the target destructor body. |
| `0043bb36` | GHBoxLayout::ScalarDeletingDestructor | confirmed | Runs destructor body 0043591d; when flag bit 0 is set, frees the object through 004c6a61; otherwise returns the original object pointer. The x86 body has the s… |
| `0043fbca` | GHBoxLayout::CreateInstance | likely | Mode-selector factory for GHBoxLayout: mode 0 constructs in place (base initialiser FUN_0043B78D(0,0) plus the two GHBoxLayout vtable slots); mode 1 copy-const… |

**GInterface** (20)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00404731` | GInterface::ScalarDeletingDtor | confirmed | Deleting destructor: rewrites the GInterface vtable and frees the object with _free when the flag argument's low bit is set; returns the pointer. |
| `004047ef` | GInterface::ScalarDeletingDtor_2 | confirmed | Second emission of the GInterface deleting destructor: rewrites the GInterface vtable and frees the object with _free when the flag argument's low bit is set. |
| `00404a1b` | GInterface::ScalarDeletingDtor_3 | confirmed | Third emission of the GInterface deleting destructor: rewrites the GInterface vtable and frees the object with _free when the flag argument's low bit is set. |
| `00405337` | GInterface::ScalarDeletingDtor_4 | confirmed | Fourth emission of the GInterface deleting destructor: rewrites the GInterface vtable and frees the object with _free when the flag argument's low bit is set. |
| `0040607b` | GInterface::ScalarDeletingDtor_5 | confirmed | Fifth emission of the GInterface deleting destructor: rewrites the GInterface vtable and frees the object with _free when the flag argument's low bit is set. |
| `00408ddc` | GInterface::~GInterface_3 | confirmed | Releases the embedded region container at +0x30 through FUN_004c9f6e, then rewrites the object's vptr to GInterface::vftable; returns nothing. |
| `00428ed9` | GInterfaceSubobjectDtor | confirmed | Destructor/cleanup chain: calls 004dc61b on the subobject at +0x30, calls the GInterface base teardown 00424eca on the subobject at +0x18 (guarded to 0), rewri… |
| `0042abc3` | GInterfaceReleaseDtor | confirmed | Destructor chain: releases the interface held at +0x30 through its vtable slot +0x10 when present, runs the GInterface base teardown 00424eca, rewrites the GIn… |
| `00480b23` | GInterface::ScalarDeletingDtor | confirmed | Destructor thunk: installs the GInterface vtable at the object start and, when bit 0 of the flag is set, frees the object with the CRT free (_free). |
| `0048f702` | GInterface::~GInterface_6 | confirmed | Base-class cleanup emission: rewrites the object's two vtable pointers (the receiver and the slot after it when the pointer is non-null) to GInterface::vftable… |
| `004c64d4` | GInterface::StoreGlobalPair_01cc57d4 | confirmed | Virtual method at GInterface::vftable slot 1: stores ECX to *arg2 when nonnull, then stores the dword at global 0x01cc57d4 to *arg1 (same shape as 004c537a). |
| `00404716` | GInterface::~GInterface | likely | Resets the object's vptr to the GInterface vtable and returns; referenced exclusively from EH unwind funclets, i.e. it is the base-class destructor/cleanup ste… |
| `004047e8` | GInterface::~GInterface_2 | likely | Second emission of the GInterface base cleanup: resets the object's vptr to the GInterface vtable and returns; referenced from unwind funclets only. |
| `0040e96b` | GInterface::ScalarDeletingDtor | likely | Scalar deleting destructor: rewrites the object's vtable to GInterface::vftable and, when the flag argument's low bit is set, frees the object with the CRT _fr… |
| `00414902` | GInterface::ScalarDeletingDtor_6 | likely | Scalar deleting destructor: runs FUN_00408ddc (the GInterface destructor body, which releases the held container and rewrites the GInterface vtable per its own… |
| `00424eca` | GInterface::~GInterface_3 | likely | Destructor step: runs the cleanup FUN_0042213e (frees and zeroes the small vector field at +4..+0xc) and then resets the object's vptr at +0 to GInterface::vft… |
| `00424edd` | GInterface::~GInterface_4 | likely | Destructor step: deletes the object's CRITICAL_SECTION through 004cb4b0 (DeleteCriticalSection), frees and zeroes the 4-byte-element vector field at +4..+0xc t… |
| `00428f0d` | GInterfaceSubobjectDeletingDtor | likely | MSVC scalar deleting destructor for the object whose destructor chain is 00428ed9: runs that destructor and, when the flag argument's low bit is set, frees the… |
| `00445248` | ScalarDeletingDtorGInterface | likely | Deleting destructor for the GInterface base: writes the shared base vtable PTR_LAB_0175c44c into the second dword of the object when the pointer is non-null, r… |
| `004ea20c` | GInterfaceLifecycle | likely | Mode switch on param_1: 0 and 1 install the vtable PTR_LAB_0176f8c4 in *param_2 when it is nonnull; 2 and 3 allocate 4 bytes through operator new (00d833a6), i… |

**GInterfaceIE** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0054582a` | GInterfaceIE::RegisterInterfaceIE | confirmed | Registers an interface import/export record. It bails out with the OutputDebugStringA text GFC Error: GInterfaceIE::RegisterInterfaceIE VC8 optimization bug hi… |

**GLock** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00705550` | GLockDefaultConstructorClosure | confirmed | Invokes the GLock default-construction routine with a zero argument. |

**GMessageTarget** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00428cbb` | GMessageTarget::GMessageTarget | confirmed | Base constructor of the GFC message-target class: runs the GObjectBase base constructor 004d99d1, stores the GMessageTarget interface vtable into both +0 and +… |
| `00428d3a` | GMessageTarget::GMessageTargetB | confirmed | Second constructor overload of the GFC message-target class: runs the GObjectBase base constructor 004d99d1, installs the GMessageTarget vtable at +0 and +4, z… |
| `00429a8a` | CtorVtables175D4C8B | confirmed | Message-target constructor of the 0x175d4c8 control class: runs the GObjectBase base constructor 004d99d1, installs the GMessageTarget vtable at +0/+4, zeroes … |
| `0042a9bb` | CtorVtables175D9E8B | likely | Message-target constructor of the 0x175d9e8 control class: installs the GMessageTarget vtable at +0/+4, zeroes the link fields, attaches the source through 004… |
| `004c6bc9` | GMessageTarget::StoreGlobalPair_01cc622c | likely | Virtual method at GMessageTarget::vftable slot 1 (stdcall): stores ECX to *arg2 when nonnull, then stores the dword at global 0x01cc622c to *arg1 (same shape a… |

**GMessageTargetList** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0043ff6f` | GMessageTargetList::HaveMessageTargets | confirmed | Tiny const predicate: returns true when the next-pointer at this+4 no longer points at this+4, i.e. when the circular target list rooted at this+4 is non-empty. |

**GMutex** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00705560` | GMutexDefaultConstructorClosure | confirmed | Invokes the GMutex default-construction routine with initialization argument one. |

**GObjectBase** (46)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004c6f6c` | GObjectBase::StoreGlobalPair_01cc6a74 | confirmed | Virtual method at GObjectBase::vftable slot 1 (stdcall): stores ECX to *arg2 when nonnull, then stores the dword at global 0x01cc6a74 to *arg1 (same shape as 0… |
| `004c6f85` | GObjectBase::ForwardChildSlot20 | confirmed | Virtual method in GObjectBase::vftable slot 8 (shared with GObjectIterator, GLabelCtrl, GButtonCtrl, GCheckBoxCtrl and other listed tables): gets a child throu… |
| `004c6fa5` | GObjectBase::ForwardChildFlag1Slot24 | confirmed | Virtual method in GObjectBase::vftable slot 9: gets a child through this vtable slot +0x18 with flag 1; returns 0 when the child is null, otherwise calls the c… |
| `004c6fc5` | GObjectBase::ForwardChildSlot28 | confirmed | Virtual method in GObjectBase::vftable slot 10: gets a child through this vtable slot +0x18 with flag 0; when it is non-null it tail-jumps to the child's slot … |
| `004c6fda` | GObjectBase::ForwardChildFlag1Slot2C | confirmed | Virtual method in GObjectBase::vftable slot 11: child from this vtable slot +0x18 with flag 1; returns 0x30000 when the child is null, otherwise the child's sl… |
| `004c7012` | GObjectBase::ForwardChildFlag0Slot34 | confirmed | Virtual method in GObjectBase::vftable slot 13: child from this vtable slot +0x18 with flag 0; when non-null calls the child's slot +0x34 and returns nothing. |
| `004c7047` | GObjectBase::ForwardChildFlag1Slot3C | confirmed | Virtual method in GObjectBase::vftable slot 15: child from this vtable slot +0x18 with flag 1; returns 0 when null, otherwise the child's slot +0x3c result (dw… |
| `004c705e` | GObjectBase::ForwardChildFlag1Slot44 | confirmed | Virtual method in GObjectBase::vftable slot 17: child from this vtable slot +0x18 with flag 1; returns 0 when null, otherwise the child's slot +0x44 result (dw… |
| `004c7073` | GObjectBase::ForwardChildFlag1Slot48 | confirmed | Virtual method in GObjectBase::vftable slot 18: child from this vtable slot +0x18 with flag 1; returns 0 when null, otherwise the child's slot +0x48 result (by… |
| `004c70ea` | GObjectBase::ForwardChildFlag0Slot58 | confirmed | Virtual method in GObjectBase::vftable slot 22: child from this vtable slot +0x18 with flag 0; returns 0 when null, otherwise the child's slot +0x58 result (by… |
| `004c7103` | GObjectBase::ForwardChildFlag1Slot64 | confirmed | Virtual method in GObjectBase::vftable slot 25: child from this vtable slot +0x18 with flag 1; returns 0 when null, otherwise the child's slot +0x64 result (dw… |
| `004c7118` | GObjectBase::ForwardChildFlag1Slot6C | confirmed | Virtual method in GObjectBase::vftable slot 27: child from this vtable slot +0x18 with flag 1; returns 0 when null, otherwise the child's slot +0x6c result (dw… |
| `004c712f` | GObjectBase::ForwardChildFlag1Slot70Arg | confirmed | Virtual method in GObjectBase::vftable slot 28: child from this vtable slot +0x18 with flag 1; returns 0 when null, otherwise the child's slot +0x70 result (by… |
| `004c7153` | GObjectBase::ForwardChildFlag1Slot74Args2 | confirmed | Virtual method in GObjectBase::vftable slot 29: child from this vtable slot +0x18 with flag 1; returns 0 when null, otherwise the child's slot +0x74 result (by… |
| `004e68fb` | GObjectBase::GObjectBase | confirmed | Thiscall constructor with one argument: stores GObjectBase::vftable at +0, runs 004c9e45 on the two sentinel tables, installs PTR_LAB_0176f9ac, zeroes the dwor… |
| `004c717b` | GObjectBase::ForwardSlot32 | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x18](1); null child returns 0; otherwise pushes the three args and calls child->vtable[0x80] (slot … |
| `004c71a6` | GObjectBase::ForwardSlot31 | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x18](1); null child returns 0; otherwise pushes the three args and calls child->vtable[0x7c] (slot … |
| `004c71ce` | GObjectBase::ForwardSlot30 | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x18](1); null child returns 0; otherwise pushes the three args and calls child->vtable[0x78] (slot … |
| `004c71f6` | GObjectBase::ForwardSlot33 | likely | Thiscall, two stack args (ret 8). child = this->vtable[0x18](1); null child returns 0; otherwise tail-jumps to child->vtable[0x84] (slot 33) so the child retur… |
| `004c7210` | GObjectBase::ForwardSlot37 | likely | Thiscall, two stack args (ret 8). child = this->vtable[0x18](0); null child returns 0; otherwise pushes the two args and calls child->vtable[0x94] (slot 37), r… |
| `004c7237` | GObjectBase::ForwardSlot36 | likely | Thiscall, two stack args (ret 8). child = this->vtable[0x18](0); null child returns 0; otherwise pushes the two args and calls child->vtable[0x90] (slot 36), r… |
| `004c725e` | GObjectBase::ForwardSlot35 | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x18](0); null child returns 0; otherwise pushes the four args and calls child->vtable[0x8c] (slot 3… |
| `004c728d` | GObjectBase::ForwardSlot34 | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x18](0); null child returns 0; otherwise pushes the four args and calls child->vtable[0x88] (slot 3… |
| `004c72bc` | GObjectBase::ForwardSlot39 | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x18](0); null child returns 0; otherwise pushes the four args and calls child->vtable[0x9c] (slot 3… |
| `004c72eb` | GObjectBase::ForwardSlot38 | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x18](0); null child returns 0; otherwise pushes the four args and calls child->vtable[0x98] (slot 3… |
| `004c731a` | GObjectBase::ForwardSlot43 | likely | Thiscall, one stack arg (ret 4). child = this->vtable[0x18](1); null child returns 0; otherwise pushes the arg and calls child->vtable[0xac] (slot 43), returni… |
| `004c733d` | GObjectBase::ForwardSlot42 | likely | Thiscall, one stack arg (ret 4). child = this->vtable[0x18](1); null child returns 0; otherwise pushes the arg and calls child->vtable[0xa8] (slot 42), returni… |
| `004c7360` | GObjectBase::ForwardSlot40 | likely | Thiscall, one stack arg (ret 4). child = this->vtable[0x18](1); null child returns 0; otherwise pushes the arg and calls child->vtable[0xa0] (slot 40), returni… |
| `004c7383` | GObjectBase::ForwardSlot41 | likely | Thiscall, one stack arg (ret 4). child = this->vtable[0x18](1); null child returns 0; otherwise pushes the arg and calls child->vtable[0xa4] (slot 41), returni… |
| `004c73a6` | GObjectBase::ForwardSlot44 | likely | Thiscall, no stack args. child = this->vtable[0x18](1); null child returns 0; otherwise calls child->vtable[0xb0] (slot 44) with no args and returns its AL zer… |
| `004c73c2` | GObjectBase::ForwardSlot46 | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x18](0); null child returns 0; otherwise pushes the four args and calls child->vtable[0xb8] (slot 4… |
| `004c73f1` | GObjectBase::ForwardSlot47 | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x18](1); null child returns 0; otherwise pushes the three args and calls child->vtable[0xbc] (slot … |
| `004c741c` | GObjectBase::ForwardSlot49 | likely | Thiscall, two stack args (ret 8). child = this->vtable[0x18](0); null child returns 0; otherwise pushes the two args and calls child->vtable[0xc4] (slot 49), r… |
| `004c7443` | GObjectBase::ForwardSlot50 | likely | Thiscall, two stack args (ret 8). child = this->vtable[0x18](0); null child returns 0; otherwise pushes the two args and calls child->vtable[0xc8] (slot 50), r… |
| `004c746a` | GObjectBase::ForwardSlot51 | likely | Thiscall, two stack args (ret 8). child = this->vtable[0x18](1); null child returns 0; otherwise pushes the two args and calls child->vtable[0xcc] (slot 51), r… |
| `004c7491` | GObjectBase::ForwardSlot52 | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x18](0); null child returns 0; otherwise pushes the four args and calls child->vtable[0xd0] (slot 5… |
| `004c74c0` | GObjectBase::ForwardSlot53 | likely | Thiscall, two stack args (ret 8). child = this->vtable[0x18](1); null child returns 0; otherwise pushes the two args and calls child->vtable[0xd4] (slot 53), r… |
| `004c74e7` | GObjectBase::ForwardSlot54 | likely | Thiscall, no stack args. child = this->vtable[0x18](1); null child returns 0; otherwise calls child->vtable[0xd8] (slot 54) with no args and returns its AL zer… |
| `004c7503` | GObjectBase::ForwardSlot55 | likely | Thiscall, one stack arg (ret 4). child = this->vtable[0x18](1); null child returns 0; otherwise tail-jumps to child->vtable[0xdc] (slot 55) so the child return… |
| `004c751d` | GObjectBase::ForwardSlot56 | likely | Thiscall, two stack args (ret 8). child = this->vtable[0x18](1); null child returns 0; otherwise pushes the two args and calls child->vtable[0xe0] (slot 56), r… |
| `004c7544` | GObjectBase::ForwardSlot57 | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x18](0); null child returns 0; otherwise pushes the four args and calls child->vtable[0xe4] (slot 5… |
| `004c7573` | GObjectBase::ForwardSlot58 | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x18](0); null child returns 0; otherwise pushes the four args and calls child->vtable[0xe8] (slot 5… |
| `004c75a2` | GObjectBase::ForwardSlot59 | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x18](0); null child returns 0; otherwise pushes the three args and calls child->vtable[0xec] (slot … |
| `004c75cd` | GObjectBase::ForwardSlot60 | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x18](1); null child returns 0; otherwise pushes the three args and calls child->vtable[0xf0] (slot … |
| `004e69fd` | GObjectBase::Slot7 | likely | Virtual slot 7 shared by GObjectBase and its descendants: asks the receiver's vtable slot +0x18 for a child with flag 1; when a child exists it tail-calls the … |
| `004e6a14` | GObjectBase::Slot20 | likely | Virtual slot 20 shared by GObjectBase and its descendants: asks the receiver's vtable slot +0x18 for a child with flag 1; when a child exists it tail-calls the… |

**GObjectIterator** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00404831` | GObjectIterator::GObjectIterator | confirmed | Constructs a GObjectIterator: writes the GObjectIterator vtable at +0, zeroes the fields at +4/+8, and when the source object's field +8 is non-null it invokes… |
| `00404861` | GObjectIterator::~GObjectIterator | confirmed | Destructor body: rewrites the GObjectIterator vtable, releases the held interface at +8 through its vtable slot +8 when present, then rewrites the base GInterf… |
| `004050f2` | GObjectIterator::~GObjectIterator_2 | confirmed | Second emission of the GObjectIterator destructor body: rewrites the iterator vtable, releases the held interface through its vtable slot +8 when present, then… |
| `00404884` | GObjectIterator::Assign | likely | Assignment primitive for the iterator state: when the source's field +8 is null it releases the currently held interface and copies the raw value from source+4… |
| `004048d6` | GObjectIterator::ScalarDeletingDtor | likely | Scalar deleting destructor: runs the GObjectIterator destructor body FUN_00404861, then frees the object with _free when the flag argument's low bit is set. |
| `00404950` | GObjectIteratorStackRunner | likely | Builds a stack GObjectIterator (vtable written, fields zeroed), invokes the container's virtual slot +0x94(iterator, param_2) which performs the iteration, des… |
| `004050d1` | GObjectIterator::CopyCtor | likely | Copy-constructor path: builds the iterator base from the source through 00404831, then copies the two extra fields at +0xC/+0x10 from the source; returns this. |
| `004050f7` | GObjectIterator::CopyCtor2 | likely | Second copy-constructor path: assigns the iterator state through 00404884, then copies the two extra fields at +0xC/+0x10 from the source; returns this. |
| `0040511b` | GObjectIterator::InitFromKey | likely | Initializes an iterator object (vtable at +0, zeroes +4/+8/+0x10) and asks the container param_1 to fill it through virtual slot +0x50(iterator, param_3); retu… |
| `00405141` | GObjectIterator::InitFromKey2 | likely | Initializes an iterator object (vtable at +0, zeroes +4/+8/+0x10) and asks the container param_1 to fill it through virtual slot +0x54(iterator, param_3, param… |
| `0041188e` | GObjectIterator::Next | likely | Advance step: with this+0xc bit 1 set it returns FUN_00407283() directly; otherwise, when the current object FUN_004e7384(1) differs from the cached one at thi… |
| `0041218c` | GObjectIterator::Advance2 | likely | Second advance variant: with this+0xc bit 2 set it returns FUN_004c82fc(); otherwise it compares FUN_004e7384(1) with the cache at this+0x10, advances and retu… |

**GPathAliasObject** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007054e0` | GPathAliasObjectDefaultConstructorClosure | confirmed | Constructs a temporary reference string from the static alias value and releases that temporary before returning. |

**GPoint** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004eb3c2` | GPoint::ReadPair | confirmed | Looks up the GPoint attribute group on the object in param_2 (key L"GPoint" via 004e4e6d, vtable +0x68 on the interface at *param_1). When found it reads the f… |
| `004eb454` | GPoint::WritePair | confirmed | Looks up the GPoint attribute group on the object in param_2 (key L"GPoint" via 004e4e6d, vtable +0x6c on the interface at *param_1). When found it writes the … |

**GPropertyInterface** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00411de6` | GPropertyInterface::Ctor | confirmed | Constructs an object with three vtable sub-objects: calls FUN_004d99d1, writes GPropertyInterface::vftable into the sub-object at +0x14 (then replaces it with … |
| `00411e80` | GPropertyInterface::Dtor | confirmed | Destructor: releases the interface pointer at +0x18 (virtual slot +0x10), writes GInterface::vftable into the sub-object at +0x14 (bytes: neg/sbb/and select pa… |
| `00411f26` | GPropertyInterface::CtorWithSource | confirmed | Construction from an existing source: calls FUN_004e68fb(param_2), writes the same three vtable pointers, AddRefs the interface at param_2+0x18 (virtual +0xc) … |
| `0041a5d4` | GPropertyInterface::ScalarDeletingDtor | likely | Calls FUN_00411E80 on this, frees this through the operator-delete thunk when param_2 bit 0 is set, and returns this otherwise. |
| `0049327c` | GPropertyInterface::Construct | likely | If param_1 is non-null, installs GPropertyInterface::vftable and initializes member vtables, the shared empty-string marker at +0x14 and its companion pointer … |
| `0049369f` | GPropertyInterface::ConstructFromSource | likely | If param_1 is non-null, installs GPropertyInterface::vftable and initializes the base members. Computes source base as param_2+0x10 when param_2 is non-null (o… |

**GRange** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004eb226` | GRange::ReadFirstLast | confirmed | Looks up the GRange attribute group on the object in param_2 (key L"GRange" via 004e4e6d, vtable +0x68 on the interface at *param_1). When found it reads First… |
| `004eb2b8` | GRange::WriteFirstLast | confirmed | Looks up the GRange attribute group on the object in param_2 (key L"GRange" via 004e4e6d, vtable +0x6c on the interface at *param_1). When found it writes Firs… |

**GRangeMargin** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004ebb12` | GRangeMargin::ReadBeginEnd | confirmed | Looks up the GRangeMargin attribute group on the object in param_2 (key L"GRangeMargin" via 004e4e6d, vtable +0x68 on the interface at *param_1). When found it… |
| `004ebba4` | GRangeMargin::WriteBeginEnd | likely | Looks up the GRangeMargin attribute group on the object in param_2 (key L"GRangeMargin" via 004e4e6d, vtable +0x6c on the interface at *param_1). When found it… |

**GRangePropertyMapInterface** (36)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004c97db` | GRangePropertyMapInterface::Slot16ChildCall | likely | Thiscall, five stack args (ret 0x14). child = this->vtable[0x30](1); null child returns 0; otherwise pushes the args and calls child->vtable[0x40] (slot 16), r… |
| `004c980a` | GRangePropertyMapInterface::Slot17ChildCall | likely | Thiscall, five stack args (ret 0x14). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x44] (slot 17), r… |
| `004c9839` | GRangePropertyMapInterface::Slot18ChildCall | likely | Thiscall, six stack args (ret 0x18). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x48] (slot 18), re… |
| `004c986b` | GRangePropertyMapInterface::Slot19ChildCall | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x30](1); null child returns 0; otherwise pushes the args and calls child->vtable[0x4c] (slot 19), r… |
| `004c9897` | GRangePropertyMapInterface::Slot21ChildCall | likely | Thiscall, five stack args (ret 0x14). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x54] (slot 21), r… |
| `004c98c6` | GRangePropertyMapInterface::Slot20ChildCall | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x50] (slot 20), r… |
| `004c98f2` | GRangePropertyMapInterface::Slot23ChildCall | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x5c] (slot 23), r… |
| `004c991e` | GRangePropertyMapInterface::Slot22ChildCall | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x58] (slot 22), r… |
| `004c994a` | GRangePropertyMapInterface::Slot25ChildCall | likely | Thiscall, five stack args (ret 0x14). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x64] (slot 25), r… |
| `004c9979` | GRangePropertyMapInterface::Slot24ChildCall | likely | Thiscall, five stack args (ret 0x14). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x60] (slot 24), r… |
| `004c99a8` | GRangePropertyMapInterface::Slot26ChildCall | likely | Thiscall, five stack args (ret 0x14). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x68] (slot 26), r… |
| `004c99d7` | GRangePropertyMapInterface::Slot27ChildCall | likely | Thiscall, five stack args (ret 0x14). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x6c] (slot 27), r… |
| `004c9a06` | GRangePropertyMapInterface::Slot28ChildCall | likely | Thiscall, five stack args (ret 0x14). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x70] (slot 28), r… |
| `004c9a35` | GRangePropertyMapInterface::Slot29ChildCall | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x74] (slot 29), r… |
| `004c9a5d` | GRangePropertyMapInterface::Slot30ChildCall | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x78] (slot 30), r… |
| `004c9a85` | GRangePropertyMapInterface::Slot32ChildCall | likely | Thiscall, no stack args. child = this->vtable[0x30](0); null child returns 0; otherwise calls child->vtable[0x80] (slot 32) with no args and returns its AL zer… |
| `004c9aa1` | GRangePropertyMapInterface::Slot31ChildCall | likely | Thiscall, two stack args (ret 8). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x7c] (slot 31), retur… |
| `004c9ac5` | GRangePropertyMapInterface::Slot33ChildCall | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x84] (slot 33), r… |
| `004c9af0` | GRangePropertyMapInterface::Slot37ChildTail | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x30](1); null child returns 0; otherwise tail-jumps to child->vtable[0x94] (slot 37) so the child r… |
| `004c9b0a` | GRangePropertyMapInterface::Slot36ChildTail | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x30](1); null child returns 0; otherwise tail-jumps to child->vtable[0x90] (slot 36) so the child r… |
| `004c9b24` | GRangePropertyMapInterface::Slot35ChildTail | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x30](1); null child returns 0; otherwise tail-jumps to child->vtable[0x8c] (slot 35) so the child r… |
| `004c9b3e` | GRangePropertyMapInterface::Slot40ChildTail | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x30](1); null child returns 0; otherwise tail-jumps to child->vtable[0xa0] (slot 40) so the child r… |
| `004c9b58` | GRangePropertyMapInterface::Slot39ChildTail | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x30](1); null child returns 0; otherwise tail-jumps to child->vtable[0x9c] (slot 39) so the child r… |
| `004c9b72` | GRangePropertyMapInterface::Slot38ChildTail | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x30](1); null child returns 0; otherwise tail-jumps to child->vtable[0x98] (slot 38) so the child r… |
| `004c9b8c` | GRangePropertyMapInterface::Slot44ChildTail | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x30](1); null child returns 0; otherwise tail-jumps to child->vtable[0xb0] (slot 44) so the child r… |
| `004c9ba6` | GRangePropertyMapInterface::Slot46ChildTail | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x30](1); null child returns 0; otherwise tail-jumps to child->vtable[0xb8] (slot 46) so the child r… |
| `004c9bc0` | GRangePropertyMapInterface::Slot45ChildTail | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x30](1); null child returns 0; otherwise tail-jumps to child->vtable[0xb4] (slot 45) so the child r… |
| `004c9bda` | GRangePropertyMapInterface::Slot47ChildTail | likely | Thiscall, two stack args (ret 8). child = this->vtable[0x30](1); null child returns 0; otherwise tail-jumps to child->vtable[0xbc] (slot 47) so the child retur… |
| `004c9bf4` | GRangePropertyMapInterface::Slot48ChildTail | likely | Thiscall, two stack args (ret 8). child = this->vtable[0x30](1); null child returns 0; otherwise tail-jumps to child->vtable[0xc0] (slot 48) so the child retur… |
| `004c9c0e` | GRangePropertyMapInterface::Slot49ChildTail | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x30](1); null child returns 0; otherwise tail-jumps to child->vtable[0xc4] (slot 49) so the child r… |
| `004c9c6b` | GRangePropertyMapInterface::Slot34ChildCall | likely | Thiscall, three stack args (ret 0xc). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0x88] (slot 34), r… |
| `004c9c96` | GRangePropertyMapInterface::Slot53ChildCall | likely | Thiscall, four stack args (ret 0x10). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0xd4] (slot 53), r… |
| `004c9cc5` | GRangePropertyMapInterface::Slot52ChildTail | likely | Thiscall, one stack arg (ret 4). child = this->vtable[0x30](0); null child returns 0; otherwise tail-jumps to child->vtable[0xd0] (slot 52) so the child return… |
| `004c9cdd` | GRangePropertyMapInterface::Slot54ChildCall | likely | Thiscall, six stack args (ret 0x18). child = this->vtable[0x30](0); null child returns 0; otherwise pushes the args and calls child->vtable[0xd8] (slot 54), re… |
| `004c9d12` | GRangePropertyMapInterface::Slot56ChildTail | likely | Thiscall, one stack arg (ret 4). child = this->vtable[0x30](0); null child returns 0; otherwise tail-jumps to child->vtable[0xe0] (slot 56) so the child return… |
| `004c9d2a` | GRangePropertyMapInterface::Slot55ChildTail | likely | Thiscall, two stack args (ret 8). child = this->vtable[0x30](0); null child returns 0; otherwise tail-jumps to child->vtable[0xdc] (slot 55) so the child retur… |

**GRect** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004eb714` | GRect::ReadLeftTopRightBottom | confirmed | Looks up the GRect attribute group on the object in param_2 (key L"GRect" via 004e4e6d, vtable +0x68 on the interface at *param_1). When found it reads Left (v… |
| `004eb7f3` | GRect::WriteLeftTopRightBottom | confirmed | Looks up the GRect attribute group on the object in param_2 (key L"GRect" via 004e4e6d, vtable +0x6c on the interface at *param_1). When found it writes Left (… |

**GRectMargin** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004eb8dc` | GRectMargin::ReadMargins | confirmed | Looks up the GRectMargin attribute group on the object in param_2 (key L"GRectMargin" via 004e4e6d, vtable +0x68 on the interface at *param_1). When found it r… |
| `004eb9bb` | GRectMargin::WriteMargins | confirmed | Looks up the GRectMargin attribute group on the object in param_2 (key L"GRectMargin" via 004e4e6d, vtable +0x6c on the interface at *param_1). When found it w… |

**GRegion** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00705590` | GetRectCount | confirmed | Returns the dword stored at offset four of the GRegion object. |

**GScrollableInterface** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00606bda` | GScrollableInterface_ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores GInterface::vftable at +0 and, when bit 0 of the flag byte is set, frees the object with _free; returns the object pointer. |

**GSemaphore** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00705580` | GSemaphoreDefaultConstructorClosure | confirmed | Invokes the GSemaphore default-construction routine with initialization argument one. |

**GShadowFont** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00481f24` | RegisterGShadowFont | confirmed | Registers the GFC class "GShadowFont" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GSize** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004eb55e` | GSize::ReadWidthHeight | confirmed | Looks up the GSize attribute group on the object in param_2 (key L"GSize" via 004e4e6d, vtable +0x68 on the interface at *param_1). When found it reads Width (… |
| `004eb5f0` | GSize::WriteWidthHeight | confirmed | Looks up the GSize attribute group on the object in param_2 (key L"GSize" via 004e4e6d, vtable +0x6c on the interface at *param_1). When found it writes Width … |

**GStringAccess** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0042aeb7` | GStringAccessCreate | confirmed | Inline factory: allocates 0x34 bytes through 0040929c and constructs a GStringAccess-derived object in place (GObjectBase base, GStringAccess and GDataAccessIm… |
| `0042c556` | GStringAccess::QuerySlot74WithTemp | confirmed | Builds a temporary through FUN_00428293(1, param_2), invokes the receiver's vtable slot +0x74 with a 16-byte stack result, destroys the temporary with FUN_0042… |

**GStringAccessWrapper** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0042ae63` | GStringAccessWrapper::GStringAccessWrapper | confirmed | Constructor of the GStringAccessWrapper class: runs the base initializer 0041493a, installs the GBitmap-free GStringAccessWrapper vtable at +0, allocates 0x34 … |
| `00415038` | GStringAccessWrapper::ForwardAccessSlot6C | likely | When the data access object at +8 is present, forwards (param_2, param_3, param_4) to that object's vtable slot +0x6C and returns its result; returns 0 when no… |
| `0041505a` | GStringAccessWrapper::ForwardAccessSlot68 | likely | When the data access object at +8 is present, forwards param_2 to that object's vtable slot +0x68 and returns its result; returns 0 when no access is held. |
| `00420db2` | GStringAccessWrapper::Ops40 | likely | Dispatches construction and destruction operations for 0x28-byte GStringAccessWrapper objects. Ops 0 and 2 call default constructor FUN_0041493a then store the… |

**GStringObject** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00428da5` | GStringObject::GStringObject | confirmed | Constructor of the GStringObject class: runs the base constructor 004d99d1, installs GStringAccess at +0x14 and GDataAccessImpl at +0x18, zeroes +0x20/+0x24/+0… |
| `0042c58a` | GStringObject::QuerySlot74WithTemp | confirmed | Second emission of the same body as QuerySlot74WithTempA: FUN_00428293(1, param_2), then the receiver's vtable slot +0x74 with a 16-byte stack result, then FUN… |
| `0044d239` | GStringObject::GStringObject | confirmed | Constructs a GStringObject around a wide-character source: runs the base constructor 004d99d1, installs the GStringAccess::vftable and GDataAccessImpl::vftable… |
| `0044d294` | GStringObject::GStringObject_2 | confirmed | Ansi constructor of the same GStringObject layout: base constructor 004d99d1, the two interface vtables at +0x14/+0x18, zeroed fields, param_3 at +0x2c, GStrin… |

**GTimeZone** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004eadce` | GTimeZone::ReadFromAttributes | confirmed | Looks up the GTimeZone attribute group on the object in param_2 (key L"GTimeZone" via 004e4e6d, vtable +0x68 on the interface at *param_1). When found it reads… |
| `004eaf1a` | GTimeZone::WriteToAttributes | confirmed | Looks up the GTimeZone attribute group on the object in param_2 (key L"GTimeZone" via 004e4e6d, vtable +0x6c on the interface at *param_1). When found it write… |

**GTypedFolderObject** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00705540` | GTypedFolderObjectDefaultConstructorClosure | confirmed | Invokes the GTypedFolderObject default-construction routine with its zero initialization argument. |

**GVBoxLayout** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0043b8b2` | GVBoxLayout::GVBoxLayout | confirmed | Calls layout base constructor 00435c70 with the supplied argument, writes the GVBoxLayout vtable at object offsets +0 and +4, and returns this. |
| `0043b8d1` | GVBoxLayout::ForwardTo5017F8 | confirmed | The stack adapter forwards its receiver and two stack arguments to 005017f8, cleans the temporary 12-byte argument area, then returns with an 8-byte callee-pop… |
| `0043b8e5` | GVBoxLayout::AdjustorThunkToDestructor | confirmed | Adjusts ECX by subtracting 4, then jumps directly to 0043baf9; it does not contain the target destructor body. |
| `0043baf9` | GVBoxLayout::ScalarDeletingDestructor | confirmed | Runs destructor body 0043591d; when flag bit 0 is set, frees the object through 004c6a61; otherwise returns the original object pointer. The x86 body has the s… |
| `0043e765` | GVBoxLayout::GVBoxLayout | confirmed | Constructor: for a non-null object pointer runs the base initialiser FUN_0043B78D(1,0) and installs the GVBoxLayout vtable at slot [0] and the secondary vtable… |
| `0043fb2c` | GVBoxLayout::CreateInstance | likely | Mode-selector factory for GVBoxLayout: mode 0 constructs in place (base initialiser FUN_0043B78D(1,0) plus the two GVBoxLayout vtable slots); mode 1 copy-const… |

**GuiModelManagerInterface** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a57d0` | GuiModelManagerInterface::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: writes GuiModelManagerInterface::vftable and calls _free(this) when param_2 bit 0 is set; no direct callers in the export. |
| `008a5780` | GuiModelManagerInterface::~GuiModelManagerInterface | likely | Destructor body: writes GuiModelManagerInterface::vftable into the object vtable slot and returns; callers are 0158bed0 and 0164a5d0. |

**SoeUtil::Array<GFxWrap::GuiGfxParam,1,1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d12a0` | GFxWrap::GuiGfxParamArray::NodeFree | confirmed | Frees the supplied pointer with eight-byte alignment unless it equals the array's inline-buffer address rounded up to eight-byte alignment. |
| `009d1c10` | GFxWrap::GuiGfxParamArray::Dtor | confirmed | Restores inline-array then base-array vtables, destroys stored parameters, frees non-inline storage, and clears its pointer. |
| `009d1cb0` | GFxWrap::GuiGfxParamArray::Allocate | confirmed | For requests of at most one element, returns aligned inline storage and reports capacity one; larger requests delegate to FUN_0083eb60. |
| `009d1ff0` | GuiGfxParamArrayCtor | confirmed | Initializes the base array pointer, count, and capacity, then installs the one-inline-element array vtable. |
| `009d2050` | GuiGfxParamArray::ScalarDeletingDtor | confirmed | Runs the inline-array destructor and frees the object when the deleting flag is set. |
| `009d2070` | GuiGfxParamArray::SwapContents | confirmed | Exchanges the two arrays' contents through a temporary: copies the source into temporary storage, clears the source, copies the old destination into the source… |

**SoeUtil::HashListMap** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008aacc0` | HashListMap_GuiSharedProperty_Construct | confirmed | Constructor: installs the SoeUtil::HashListMap<int,GuiPropertyDataSource::PropertySetFilter::SharedProperty,128,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets… |

**(no class)** (651)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00413fc3` | LoadPlacement | confirmed | Loads the GCtrlPlacement group: reads PlacementFlags (default GFC_CPF_LEFT_LEFT) into *param_3, reads ZOrder (default GFC_CPF_ZORDER_BELOWBOTTOM) and ORs the r… |
| `00414098` | StorePlacement | confirmed | Store twin of LoadPlacement: writes PlacementFlags from *param_3 (default GFC_CPF_LEFT_LEFT), ZOrder from *param_3 & 0x700000 (default GFC_CPF_ZORDER_BELOWBOTT… |
| `004293ec` | ApplyTargetMessage | confirmed | Message-application method of the control class: reads the 16-bit message tag at message+0x1C and maps known tags to option bits (0x200/0x400/0x800/0x1000) on … |
| `0042aa83` | CtorVtables175E920 | confirmed | Constructor of a data-access control class (primary table 0x175e920): runs the GObjectBase base constructor, installs the GDataAccessImpl base vtable at +0x18,… |
| `0042ac19` | CtorVtables175EAF8 | confirmed | Constructor of a string-access data object (primary table 0x175eaf8): runs the GObjectBase base constructor, stores param_2 at +0x14, installs the GStringAcces… |
| `0042ad3e` | CtorVtables175ECF8 | confirmed | Constructor of a second data-access object (primary table 0x175ecf8): runs the GObjectBase base constructor, stores param_2 at +0x14, installs the intermediate… |
| `0043b396` | GStringDataAccessLifecycleDispatcher | confirmed | Dispatches modes 0 and 2 to initialize the GStringAccess/GDataAccessImpl object (mode 2 first allocates 0x34 bytes); modes 1 and 3 call 00435394 (mode 3 alloca… |
| `0043df98` | TraverseChildrenRecursively | confirmed | Recursive object-tree walk: builds a stack GObjectIterator (vtable installed from GObjectIterator::vftable), stores the object (param_1) into the global slot [… |
| `00443499` | CtorEmbeddedGPropertyInterface | confirmed | Calls base initializer 004D99D1, writes GPropertyInterface::vftable at +0x14, installs the primary/secondary tables at offsets 0/+4, then overwrites +0x14 with… |
| `004434f7` | DtorResetEmbeddedGInterface | confirmed | When param_1 is nonnull, writes GInterface::vftable at param_1+0x14, then tail-jumps to 00526562; the target performs the remaining base-interface destruction. |
| `004467dc` | CtorEmbeddedGPropertyInterface1763084 | confirmed | Constructor with an embedded property interface: runs the base initialiser FUN_004e68fb(param_2), writes the GPropertyInterface vtable into slot [5] (this+0x14… |
| `0045728f` | GInterfaceDerivedDtor | confirmed | Destructor body of a GInterface-derived class: destroys the two tree/map members through the teardown bodies 00456960 and 00456912, then rewrites the object's … |
| `00462140` | DtorVtables17649E8 | confirmed | Destructor sequence of a GInterface-family object: stores the class tables 017649e8/017649e4, runs the member cleanups FUN_0045aa50 and FUN_0045aa68, releases … |
| `0047e5b9` | ProbeBitmapSignature | confirmed | Reads two bytes through virtual slot 0x18, compares them with the literal "BM", restores the stream state, and returns whether they match. |
| `00485b9a` | DtorWithInterfaceAt18 | confirmed | Releases the referenced interface at +0x18 (vtable slot +0x10), resets the GInterface sub-object at +0x14 to GInterface::vftable, then calls the GObjectBase de… |
| `00485ca2` | DtorWithInterfaceAt18_2 | confirmed | Second emission of the destructor: releases the interface at +0x18, resets the GInterface sub-object at +0x14 to GInterface::vftable and calls the GObjectBase … |
| `0048d12b` | GetColorRGBA | confirmed | Fetches the 'GColor' sub-object from the property bag param_1 through the bag's read slot +0x68 and reads its component properties with the sub-object's slot +… |
| `0048d1fe` | SetColorRGBA | confirmed | Store twin of FUN_0048d12b: fetches the 'GColor' sub-object through the bag's write slot +0x6c and writes the packed bytes param_3+2 (Red), param_3+1 (Green), … |
| `0048d380` | LookupNamedColor | confirmed | Named-colour lookup: calls FUN_0048d2db with the caller's key and the static colour-name table beginning at PTR_s_AliceBlue_017676a0 with 0x225 (549) entries, … |
| `0048ecf1` | PropertyAccessHolderCtorFrom | confirmed | Constructs the property-access holder from a source object: calls FUN_004e68fb with the destination in ECX and param_2 as its argument, installs the GPropertyI… |
| `0048edcd` | PropertyAccessHolderDtor | confirmed | Destructor of the property-access holder: reinstalls the class vtables, releases the interface at +0x18 through its slot +0x10(0), resets the +0x14 sub-object … |
| `0048eef6` | PropertyAccessHolderCtor2 | confirmed | Second emission of the default holder constructor: runs the GObjectBase constructor FUN_004d99d1, installs the GPropertyInterface vtable at +0x14 plus the clas… |
| `0048ef21` | PropertyAccessHolderDtor2 | confirmed | Second emission of the holder destructor: reinstalls the class vtables, releases the interface at +0x18 through slot +0x10(0), resets the +0x14 sub-object vtab… |
| `00606bd3` | ResetVptrGInterface | confirmed | Single store (mov [ecx],GInterface::vftable; ret) that resets the object's vptr to the GInterface base vtable; its only references are two MSVC unwind funclets… |
| `0091bf60` | Dtor_GuiUserDataTableAt34 | confirmed | Destructor of a holder object: restores GuiUserDataTable::vftable at this+0x34, destroys the embedded SoeUtil::HashListMap<int,ColumnNameLookupNode,128,-1> via… |
| `00a80c30` | CastToGuildMemberDataSource | confirmed | Returns 0 for a null input; otherwise RTTI-casts through GuiDataSource to GuildMemberDataSource using 00d8d382. If param_2 is nonzero and the cast fails, calls… |
| `00a80d10` | CastToFriendsListDataSource | confirmed | Returns 0 for a null input; otherwise RTTI-casts through GuiDataSource to FriendsListDataSource using 00d8d382. If param_2 is nonzero and the cast fails, calls… |
| `004046c0` | ListHeadInsert | likely | Zeroes the 16-bit head field at +0, fetches a list head through FUN_004c886d, stores param_2/param_3/param_4 into the record at +0xC/+0x4/+0x8, then links this… |
| `00406c89` | CtorVtables175CA50 | likely | Runs the GObjectBase base constructor FUN_004d99d1 and then stores the two-pointer vtable pair 0x175ca50/0x175ca4c at +0/+4 of the object; returns the object. |
| `00406cd9` | CtorVtables175CA50B | likely | Runs the argument-taking GObjectBase base constructor FUN_004e68fb(param_2) and then stores the vtable pair 0x175ca50/0x175ca4c at +0/+4; returns the object. |
| `00406d9a` | CtorVtables175CB68 | likely | Runs the GObjectBase base constructor FUN_004d99d1 and then stores the vtable pair 0x175cb68/0x175cb64 at +0/+4 of the object; returns the object. |
| `00406dea` | CtorVtables175CB68B | likely | Runs the argument-taking GObjectBase base constructor FUN_004e68fb(param_2) and then stores the vtable pair 0x175cb68/0x175cb64 at +0/+4; returns the object. |
| `00406eab` | CtorVtables175CC80 | likely | Runs the GObjectBase base constructor FUN_004d99d1 and then stores the vtable pair 0x175cc80/0x175cc7c at +0/+4; returns the object. |
| `00406efb` | CtorVtables175CC80B | likely | Runs the argument-taking GObjectBase base constructor FUN_004e68fb(param_2) and then stores the vtable pair 0x175cc80/0x175cc7c at +0/+4; returns the object. |
| `004076dd` | CtorVtables175CDA0 | likely | Runs the base constructor FUN_0052f5b0 and then stores the two-pointer vtable pair 0x175cda0/0x175cd98 at +0/+4; returns the object. |
| `00408613` | CtorVtables175CF88 | likely | Runs the base constructor FUN_00528f2f and stores three data pointers: 0x175cf88 at +0, 0x175cf84 at +4 and 0x175cf08 at +0x14; returns the object. |
| `004090b4` | CtorVtables175D240 | likely | Runs the GObjectBase base constructor FUN_004d99d1, stores param_2 at +0x14, then stores the vtable pair 0x175d240/0x175d23c at +0/+4; returns the object. |
| `00409180` | CtorVtables175D240B | likely | Runs the argument-taking base constructor FUN_004e68fb(param_2), stores the vtable pair 0x175d240/0x175d23c, and copies the field at +0x14 of param_2 into +0x1… |
| `004093c3` | CtorVtables175D358 | likely | Stores the base vtable pointer 0x175c44c at +0, copies param_2's dwords at +4/+8 to +4/+8, then overwrites +0 with the final vtable pointer 0x175d358. |
| `00409c17` | ReleaseHeldPtr | likely | When the pointer at +0 is non-null, calls virtual slot +0x10 on it with 0 (the release call); otherwise does nothing. |
| `00409c4e` | InterfacePtrAssign | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `00409d43` | InterfacePtrAssign_2 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `00409dc1` | InterfacePtrAssign_3 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `00409e26` | ReleaseHeldPtr_2 | likely | When the pointer at +0 is non-null, calls virtual slot +0x10 on it with 0 (the release call); otherwise does nothing. |
| `00409e9b` | InterfacePtrAssign_4 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `00409fc5` | InterfacePtrAssign_5 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `0040a024` | InterfacePtrAssign_6 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `0040a0c6` | InterfacePtrAssign_7 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `0040a187` | InterfacePtrAssign_8 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `0040a22c` | InterfacePtrAssign_9 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `0040a28b` | InterfacePtrAssign_10 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `0040a338` | InterfacePtrAssign_11 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `0040a39a` | InterfacePtrAssign_12 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `0040a602` | ReleaseHeldPtr_3 | likely | When the pointer at +0 is non-null, calls virtual slot +0x10 on it with 0 (the release call); otherwise does nothing. |
| `0040a639` | InterfacePtrAssign_13 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `0040a65b` | CtorVtables175D39C | likely | Runs the base constructor FUN_004c9e10 and stores the single vtable pointer 0x175d39c at +0; returns the object. |
| `0040a66d` | CtorVtables175D358 | likely | Runs the base constructor FUN_004c9e10 and stores the vtable pointer 0x175d358 at +0; returns the object. |
| `0040a6fb` | ReleaseHeldPtr_4 | likely | When the pointer at +0 is non-null, calls virtual slot +0x10 on it with 0 (the release call); otherwise does nothing. |
| `0040a70f` | CtorVtables175D3A0 | likely | Runs the base constructor FUN_004c9e10 and stores the vtable pointer 0x175d3a0 at +0; returns the object. |
| `0040a75d` | InterfacePtrAssign_14 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `0040a802` | InterfacePtrAssign_15 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `0040a97d` | InterfacePtrAssign_16 | likely | Releases the currently held interface pointer at +0 when present (virtual slot +0x10 with 0), then stores param_2 and returns the object. |
| `0040b6e7` | DtorReleaseRefsClearVariant | likely | Destructor body: releases the interface pointers at +0x2c and +4 through vtable slot +0x10 with a zero argument, and clears the six-slot tagged value object at… |
| `0040c1af` | NameKeyedGlobalLookup | likely | Lazily initialised global accessor: a one-time guard (_DAT_01bc4430 low bit) zeroes the cached dword DAT_01bc442c, then while the flag byte DAT_01bc4428 is sti… |
| `0040c380` | NameKeyedGlobalLookup | likely | Second emission of the lazily cached name-keyed lookup: the one-time guard _DAT_01bc443c zeroes the cached dword DAT_01bc4438, then while DAT_01bc4434 is zero … |
| `0040c3c6` | NameKeyedGlobalLookup | likely | Third emission of the lazily cached name-keyed lookup, using the guard _DAT_01bc4448, cache DAT_01bc4444 and flag DAT_01bc4440 around the resolver call FUN_005… |
| `0040c40c` | NameKeyedGlobalLookup | likely | Fourth emission of the lazily cached name-keyed lookup, using the guard _DAT_01bc4454, cache DAT_01bc4450 and flag DAT_01bc444c around the resolver call FUN_00… |
| `0040d001` | CopyRangeRefPtr | likely | Forward range copy of refcounted interface pointers: for every element it add-refs the source pointer (vtable slot +0xc) and releases the previous destination … |
| `0040d065` | CopyRangeRefPtr | likely | Second emission of the refcounted pointer range copy: identical body with add-ref of the source, release of the destination and the store; returns the advanced… |
| `0040d1f0` | RefPtrAssignChecked | likely | Null-checked refcounted pointer assignment: when the destination pointer is non-null it add-refs *param_2 through vtable slot +0xc and stores the source word i… |
| `0040d210` | RefPtrAssignChecked | likely | Second emission of the null-checked refcounted pointer assignment (add-ref of *param_2 via vtable slot +0xc, then store into *param_1 when the destination is n… |
| `0040d230` | RefPtrAssignChecked | likely | Third emission of the null-checked refcounted pointer assignment (add-ref of *param_2 via vtable slot +0xc, then store into *param_1 when the destination is no… |
| `0040d2b6` | ScalarDeletingDtor | likely | Scalar deleting destructor: releases the interface pointer held at +0 through vtable slot +0x10 with a zero argument, then frees the object with the CRT _free … |
| `0040d2da` | ScalarDeletingDtor | likely | Second emission of the interface-holder deleting destructor: release of the pointer at +0 via vtable slot +0x10, then _free when the flag's low bit is set. |
| `0040d2fe` | ScalarDeletingDtor | likely | Scalar deleting destructor: runs the two-interface plus tagged-value teardown FUN_0040b6e7, then frees the object with the CRT _free when the flag's low bit is… |
| `0040d31a` | ScalarDeletingDtor | likely | Third emission of the interface-holder deleting destructor: release of the pointer at +0 via vtable slot +0x10, then _free when the flag's low bit is set. |
| `0040d421` | CopyRangeBackwardRefPtr | likely | Backward range copy of refcounted interface pointers: walks the source range down from param_2 and writes each element into the destination range ending at par… |
| `0040d46d` | CopyRangeBackwardRefPtr | likely | Second emission of the backward refcounted pointer copy: walks the source down, add-refs each element and releases the destination occupant before storing; ret… |
| `0040e04e` | VariantInitZero | likely | Initialises a 0x1e-byte tagged value record: tag word 0 at +0, three caller dwords at +4/+8/+0xc, a fourth at +0x10, zeroed slots at +0x14/+0x18 and the 2-byte… |
| `0040e081` | VariantInitReference | likely | Initialises the tagged value record as a reference (tag 4 at +0): stores the caller dwords at +8/+0xc/+0x10, the 2-byte trailer at +0x1c, zeroes +0x14/+0x18 an… |
| `0040e110` | VariantInitTagged | likely | Initialises the tagged value record with tag 0x22, first clearing the previous contents through FUN_004d93a3 when the tag mask 0x24924 is non-zero; stores the … |
| `0040e1f3` | WriteFiveFieldRecord | likely | Packs its four trailing arguments into a 16-byte stack record and passes that record plus the first argument to the record writer FUN_0040516b; returns nothing. |
| `0040e9e6` | ScalarDeletingDtor | likely | Scalar deleting destructor: runs the tiny teardown wrapper FUN_00406208 (which calls FUN_004c9f6e then the GInterface-derived FUN_00526562) and frees the objec… |
| `00410405` | CtrlHitTestCellList | likely | Reads the enabled flag (this+0x1c & 4). When enabled and param_2 is non-null it scans param_3 entries of two ints at param_2 and, provided param_4 is non-zero,… |
| `0041068c` | CtrlDrawItemList | likely | Walks param_3 32-byte items at param_2: for each item with positive item+0x14/item+0x18 it sets this+0xf0 to item+0x1c when item[0]==1 (otherwise keeps the sav… |
| `00410bb6` | CtrlSegmentProximityTest | likely | Computes the squared perpendicular distance from the control point this+0x10/this+0x14 to the segment (param_2,param_3)-(param_4,param_5) as cross^2/length^2, … |
| `00410e38` | GetMember184Pointer | likely | Returns the dword at +0x184 of the object referenced by this+0x44, or 0 when this+0x44 is null (the null branch keeps a compiled-in virtual release of a null l… |
| `00410e76` | GetMember180Pointer | likely | Returns the dword at +0x180 of the object referenced by this+0x44, or 0 when this+0x44 is null (same shape as 00410e38 but reading +0x180). |
| `00410eb4` | ApplyObjectOp | likely | Six-way operation dispatcher on param_1: 0 constructs param_2 in place with FUN_00406c89, 1 constructs in place through FUN_00406cd9(param_3), 2 and 3 allocate… |
| `00410f3f` | InterfaceScalarDtor | likely | Scalar deleting destructor (bytes: call FUN_00526562, test byte [esp+8]&1, then a normal call to the free thunk at 004c6a61 with this): runs the destructor bod… |
| `00410f69` | ApplyObjectOp2 | likely | Same six-way lifecycle dispatcher as 00410eb4 but over the second helper set: 0 FUN_00406d9a, 1 FUN_00406dea(param_3), 2/3 allocate 0x14 with FUN_00406e34 then… |
| `00410ff4` | InterfaceScalarDtor2 | likely | Scalar deleting destructor: runs FUN_00526562 and frees the object through the shared free thunk when the low bit of param_2 is set; returns this otherwise. |
| `0041101e` | ApplyObjectOp3 | likely | Third copy of the six-way lifecycle dispatcher: 0 FUN_00406eab, 1 FUN_00406efb(param_3), 2/3 allocate 0x14 with FUN_00406f45 then construct, 4 FUN_00526562, 5 … |
| `004110a9` | InterfaceScalarDtor3 | likely | Scalar deleting destructor: runs FUN_00526562 and frees the object through the shared free thunk when the low bit of param_2 is set. |
| `004110c5` | FreeMemberBuffer | likely | Frees the heap buffer at this+0x10 when it is neither 0 nor 1 (the 0/1 small-buffer sentinels), calling the memory-manager hook FUN_004c9f6e first and then _fr… |
| `004114b2` | BufferScalarDtor | likely | Scalar deleting destructor: runs FUN_004110c5 (which frees the member buffer at +0x10 unless it is 0 or 1) and calls CRT _free on this when the low bit of para… |
| `004114ce` | GetListener | likely | Returns the interface pointer stored at this+0xd0. |
| `004114d5` | SetListener | likely | AddRefs param_2 (its virtual slot +0xc), releases the previous pointer at this+0xd0 (virtual slot +0x10 with argument 0) and stores param_2; returns 1. |
| `00411506` | PostStateChange | likely | Builds the state-change record (flags 0 at +0, param_2 at +4, five zero dwords and the trailing static short DAT_01bc3fe8 at +0x1c) and passes it to the object… |
| `004115b9` | QuerySizeConstraintRet | likely | Wrapper: pushes param_2 and then param_1, calls FUN_0041154d with ECX unchanged (so the callee also sees the wrapper's this in ECX), and returns param_1 in EAX… |
| `00411601` | RelayoutFromAncestors | likely | Walks the enclosing-object chain (FUN_00406665/FUN_004063d8) to the nearest ancestor that owns layout, reads its state record via virtual +0x110, compares the … |
| `0041177a` | PostStateChangeBool | likely | Boolean twin of the state-change notify: builds the record (flags 0 at +0, param_2 at +4, five zero dwords and the trailing static short DAT_01bc3fac at +0x1c)… |
| `00411972` | SetListenerAndInvoke1c | likely | When param_6 is null it fails only if this+0xd0 is empty; otherwise it AddRefs param_6, releases the old this+0xd0 and stores param_6, then invokes this+0xd0's… |
| `004119c9` | SetListenerAndInvoke18 | likely | Variant of 00411972 for slot +0x18 with six arguments: sets/AddRefs the listener at this+0xd0 and then calls its virtual slot +0x18(this,param_2..param_6), ret… |
| `00411cba` | PostStateChangeWithObject | likely | When param_2 is non-null it allocates 0x30 bytes with FUN_004d9fee and constructs an object with FUN_004061b1(param_2), AddRefs it, passes it in a state record… |
| `00411d4b` | FreeMemberBufferB8 | likely | Frees the pointer at this+0xb8 (after calling FUN_0040745a on it) and clears the field to 0 (bytes: and dword [esi],0 after the _free call); returns 1 through … |
| `00411dca` | ScalarDtor_0052F673 | likely | Scalar deleting destructor: runs the destructor body FUN_0052f673 and calls the free thunk 00407776 (->004c6a61) when the low bit of param_2 is set; returns th… |
| `00412125` | FreeMemberBuffer8 | likely | Frees the heap pointer at this+8 after calling FUN_00407b11. |
| `004121d4` | CtorDefaultRecord | likely | Constructor: writes the vtable PTR_FUN_0175d8d0 at +0, stores the static record DAT_01bc7e28 at +0x10, and zeroes the dwords at +4, +8, +0xc, +0x14, +0x18, +0x… |
| `0041221c` | ScalarDtorA | likely | Scalar deleting destructor: runs the destructor body FUN_0040eaee and frees the object with _free when the low bit of param_2 is set. |
| `00412377` | ActivateTargetObject | likely | Selects param_2 as the current target (this[3]): resolves the container via FUN_00406320/FUN_004cd10a and its virtual +8, reads the object state via +0x30, rej… |
| `004124eb` | DispatchEventToSources | likely | Gated by FUN_00504db0(DAT_01bc4350); it tries the interface at this+0x18 and then the one at this+0x14, calling each one's virtual slot +0x1c(param_2,param_3,p… |
| `004125b0` | DispatchEventByName | likely | Dispatch through chained property sources: the this+0x18 source's slot +0x18 is tested once with (param_3,param_4,param_5) and returns 1 when handled; otherwis… |
| `004128ee` | InitEventCursor | likely | Builds a cursor record for the object param_2: stores param_2 at +0, zeroes +8, sets the byte at +0x10 to 1 and the byte at +0x12 to param_7. When param_5 is n… |
| `004129e5` | FindAndTestObject | likely | For the interface at this+0x18 and then the one at this+0x14: resolves the name param_2 through the interface's virtual +8 (via FUN_004c8921/FUN_004cd10a) and,… |
| `00412a69` | DtorReleaseMember28 | likely | Destructor body: writes PTR_FUN_0175d9e8/PTR_LAB_0175d9e4 into +0/+4, releases the interface at +0x28 (virtual +0x10 with argument 0) and then tail-jumps to th… |
| `00412ace` | DtorReleaseMember28Scalar | likely | Scalar deleting destructor for FUN_00412a69: runs it and frees the object when the low bit of param_2 is set. |
| `00412aea` | ScalarDtorB | likely | Scalar deleting destructor: runs FUN_00412125 (free the buffer at +8) and frees the object with _free when the low bit of param_2 is set. |
| `00412bd8` | FindInterfaceForObject | likely | Asks the active interface (FUN_00412bab) for param_1 via virtual +0x68(param_1,0); when that returns 0 it scans the container at this+0x140 (count FUN_0040a3bf… |
| `00412f95` | GetClientRect | likely | Fills param_2 with the client rect: when this+0x120 is null it uses the interface at this+0x138 (virtual +0x1c twice) to build {0,0,w-1,h-1}; otherwise, when t… |
| `00413042` | ReleaseInterfaces | likely | Clears this+0x11c and those +0xf0, then releases and clears the interface pointers at this+0x128, +0x12c, +0x130, +0x134, +0x138, +0x13c, +0x120 and +0x124 (ea… |
| `004130f9` | PostStateChangeSub | likely | State-change notify on the sub-object at this-0xbc: builds the state record (flags 0 at +0, param_2 at +4, five zero dwords and the trailing static short DAT_0… |
| `0041314f` | MapPointScaled | likely | When this+0x11c==2 and both interfaces are present it reads the target rect via this+0x134's slot +0x44 and the other rect via this+0x12c's slot +0xd4, compute… |
| `0041341d` | AdjustActiveCount | likely | Adjusts the counter this+0x34: param_2==0 increments it, first asking the interface at this+0x68 via virtual slot 0x64(0) while the counter is 0 (returning 0 w… |
| `00413b1b` | LoadScaleInfo | likely | Reads the GCtrlScaleInfo group (looked up through this+0x68) and reads xMove, yMove, xSize and ySize through the group's virtual +0x20 into the caller's four-b… |
| `00413bed` | StoreScaleInfo | likely | Writes xMove, yMove, xSize and ySize from the caller's four-byte record into the GCtrlScaleInfo group (found/created through this+0x6c) via the group's virtual… |
| `00413cc9` | ReleaseMembersAndSetBaseVtable | likely | Destructor: releases the interfaces at +0x44, +0x3c and +0x24 (each via virtual +0x10 with argument 0), then restores the second vtable PTR_LAB_0175c44c at +4 … |
| `00413d0f` | MultiVtableCtor | likely | Constructor: writes the vtable pair PTR_LAB_0175cec8/PTR_LAB_0175d39c, calls FUN_004c9e10, rewrites the pair to PTR_FUN_0175dbb0/PTR_LAB_0175dbac, then zeroes … |
| `00413da3` | ScalarDtorC | likely | Scalar deleting destructor: runs FUN_00413cc9 and frees the object through the free thunk when the low bit of param_2 is set. |
| `00413dbf` | SetRectsAndId | likely | Stores param_2 at +0x10, copies the four bytes of param_3 into +0x30..+0x33 and, when param_4 is non-null, also copies param_3 into +0x34..+0x37 and param_4 in… |
| `00413e1e` | SetRectFields | likely | Copies the four dwords of param_3 into this+0x14..+0x20 and param_4[0]/[1] into this+0x28/+0x2c, then calls FUN_00413dbf with (param_2,param_5,param_6) to stor… |
| `00413e5f` | SetMember3C | likely | AddRefs param_2 (virtual +0xc), releases the interface at this+0x3c (virtual +0x10 with argument 0) and stores param_2; returns void. |
| `00413ec9` | ComputeAlignedPos | likely | Builds two coordinates from the alignment record param_5 (flag bits select defaults 0x7fffffff/0x80000000 and take the fields +0/+4/+8/+0xc/+0x10/+0x14/+0x18/+… |
| `00414348` | RectToPlacement | likely | Converts a rectangle (param_2) plus a parent rectangle (param_3 = left/top/right/bottom) into a placement record param_1 = {flags, near-x +4, near-y +8, far-x … |
| `0041452e` | ScalarDtorD | likely | Scalar deleting destructor: runs FUN_00528f6a and calls the free thunk 004086c7 (->004c6a61) when the low bit of param_2 is set; returns this on both paths (mo… |
| `00414601` | ForwardToMember18 | likely | Plain function (no this): reads the pointer at param_1+4; returns 0 when it is null, otherwise loads its vtable, calls slot +0x18 with param_1 pushed as the ar… |
| `00414616` | ApplyObjectOp4 | likely | Dispatcher with only the 0/2/4/5 cases: 0 runs the constructor target FUN_0040541a, 2 allocates 0x14 bytes with FUN_00408816 and constructs via FUN_0040541a, 4… |
| `00414674` | RegisterCallbackTarget | likely | Initialises a callback target: stores param_5 at +0x14, param_6 at +0x48, param_7 at +0x4c, zeroes +0x70/+0x74/+0x78/+0x7c/+0x58/+0x5c, sets +0x54=-1 and +0x50… |
| `004147cf` | QueryToolTip | likely | When this+0x40 is null it looks the name ToolTip up in the property interface at param_2+0x24 (virtual +0x60) and, if found, returns (*(found+0x24))->+0xa8(); … |
| `0041483c` | SetTooltipMember | likely | Stores param_2 at this+0x40 (AddRef of param_2, release of the previous pointer via slot +0x10(0)) but only when (this+0x14 & 0x30000000) == 0x20000000; return… |
| `0041491e` | InterfaceScalarDtor4 | likely | Scalar deleting destructor: runs the FUN_00526562 destructor body and calls the free thunk 004091eb (->004c6a61) when the low bit of param_2 is set; returns th… |
| `00414fa0` | ValueClassMessageDispatch | likely | Mode dispatcher over the 0x18-byte two-vtable value class (vtable pair 0x175d240/0x175d23c): mode 0 re-initializes the caller's object in place (base construct… |
| `004150b3` | DataAccessFetchValue | likely | When the pointer at +0x14 of the object base is set, derives the held object through its vtable slot +0x1C, builds a scoped access context (FUN_004c88d5/FUN_00… |
| `0041512f` | DataAccessStoreValue | likely | Mirror of 004150b3 for writes: with the pointer at +0x14 set it resolves the data node through the same context helpers, calls the node's slot +0x1C(value, nod… |
| `004151df` | CtorVtables175DCC0 | likely | Copy constructor of the two-vtable GFC object built around the 0x175DCC0/0x175DCBC vtable pair: writes the GInterface base vtable, runs the interface copy help… |
| `0041836e` | RectIntersect | likely | Intersects the rectangle held at param_1 (left/top/right/bottom) with the rectangle given by param_2..param_5: when the stored rectangle does not overlap it re… |
| `004183d6` | GetStringPointerChecked | likely | Reads a value through the object's vtable slot +0x18 and type-checks it against the string pointer type 'p<c GString>': when the stored type identity matches i… |
| `00418581` | ReleaseTwoHoldersAndBases | likely | Destructor-shaped body (bytes 56 8B F1 8B 4E 44 .. 8D 4E 18 E8 .. 8B CE E9 ..): invokes vtable slot +0x10(0) on the object held at +0x44 and then on the object… |
| `0041862f` | IsCurrentOwnerInterface164 | likely | Identity predicate on an owner interface: calls FUN_0040faf8 with the incoming this and returns true exactly when its result equals that same this. The getter … |
| `00418640` | IsCurrentOwnerInterface178 | likely | Identity predicate: calls FUN_0040f941 with the incoming this and returns true exactly when its result equals that same this; the getter returns the pointer at… |
| `00418651` | IsCurrentOwnerInterface160 | likely | Identity predicate: calls FUN_0040f8f0 with the incoming this and returns true exactly when its result equals that same this; the getter returns the pointer at… |
| `00418c05` | ThunkThisPlus1A8To412C48 | likely | Adjustor thunk, 11 bytes 81 C1 A8 01 00 00 E9 38 A0 FF FF: adds 0x1A8 to the receiver in ecx and tail-jumps to FUN_00412C48, so the callee runs on the this+0x1… |
| `0041a5f0` | PropertyInterfaceClassOps | likely | Class-ops dispatch for the GPropertyInterface family: param_1 selects the operation - 0 constructs in place via FUN_00411DE6, 1 copy-constructs via FUN_00411F2… |
| `0041a6bf` | AppendCacheEntry | likely | Builds a small local buffer or allocates a larger one, then invokes FUN_004125B0 with the keyed request and output buffer. On an unhandled or disabled request … |
| `0041ac8d` | SetChildInterfaceWithRollback | likely | Replaces the interface pointer at +0x15C: returns 1 immediately when it already equals param_2; if param_2 is null it Releases the stored pointer (slot +0x10 w… |
| `0041adc2` | RecordOpsDispatch24 | likely | Class-ops dispatch for a 0x24-byte (9-dword) record type: 0 zeroes a caller-provided destination, 1 copies 9 dwords from param_3 into param_2, 2 allocates a ze… |
| `0041af1a` | CtorVtables175DBB0 | likely | Constructor: sets the initial vtable pair 0x175CEC8/0x175D39C, runs the base initialiser FUN_004C9E10, then rewrites both slots to 0x175DBB0/0x175DBAC, zeroes … |
| `0041af7b` | CtorVtables175DBB0B | likely | Second constructor emission for the vtable pair 0x175DBB0/0x175DBAC: same vtable and zeroing sequence as 0041AF1A (+0x24, +0x3C, short +0x40, +0x44) but finish… |
| `0041afd1` | CopyStateFieldsWithInterface | likely | Copies the 0x3C-byte state block field by field from param_2 to param_1 (dwords +0x10..+0x2C, bytes +0x30..+0x3B), then transfers the interface pointer at +0x3… |
| `0041b6cf` | ComputeAnchorOffsets | likely | Anchor/alignment resolver: for mode 4 it maps flag bits in +0x10 (0x80000/0x800000/0x10000/0x100000) to the sentinel 0x7FFFFFFF or the stored offsets at +0x4C/… |
| `0041b7e1` | MakeCursorAtMember0C | likely | Builds a bounds-checked cursor/iterator over the buffer at +0xC via FUN_0040B244 (position *(this+0xC), container this) and returns param_2 unchanged, the MSVC… |
| `0041b7f5` | MakeCursorAtMembers10C | likely | Cursor helper variant whose position is the sum of the members at +0x10 and +0xC, validated by FUN_0040B244 against the buffer this; returns param_2 unchanged. |
| `0041b829` | MakeCursorAtMember08 | likely | Cursor helper variant using member +8 as the position: calls FUN_0041691D(*(this+8), this) to validate/construct the checked cursor and returns param_2 unchang… |
| `0041b90e` | ReadValuePairA | likely | Read-and-advance helper: takes the two dwords at *param_1/param_1[1] as the value, advances/validates the cursor through FUN_0040B36E and writes the pair to pa… |
| `0041b95f` | ReadValuePairB | likely | Variant of ReadValuePairA that advances through FUN_0040B41C, writing the value pair to param_2. |
| `0041b98c` | ReadValuePairC | likely | Variant of ReadValuePairA that advances through FUN_0040B493, writing the value pair to param_2. |
| `0041b9c8` | ReadValuePairD | likely | Variant of ReadValuePairA that advances through FUN_0040B509, writing the value pair to param_2. |
| `0041b9fa` | ReadValuePairE | likely | Variant of ReadValuePairA that advances through FUN_0040B574, writing the value pair to param_2. |
| `0041ba27` | ReadValuePairF | likely | Variant of ReadValuePairA that advances through FUN_0040B5A9, writing the value pair to param_2. |
| `0041ba54` | ReadValuePairG | likely | Variant of ReadValuePairA that advances through FUN_0040B615, writing the value pair to param_2. |
| `0041bacc` | MakeCursorVia16FAD | likely | Cursor helper using member +4: calls FUN_00416FAD(*(this+4), this) to validate/construct the checked cursor and returns param_2 unchanged. |
| `0041bae0` | MakeCursorVia16FADAt08 | likely | Cursor helper variant using member +8: calls FUN_00416FAD(*(this+8), this) to validate/construct the checked cursor and returns param_2 unchanged. |
| `0041baf4` | MakeCursorVia1691DAt04 | likely | Cursor helper variant using member +4: calls FUN_0041691D(*(this+4), this) to validate/construct the checked cursor and returns param_2 unchanged. |
| `0041bb44` | MakeCursorVia16F0BAt08 | likely | Cursor helper variant using member +8: calls FUN_00416F0B(*(this+8), this) to validate/construct the checked cursor and returns param_2 unchanged. |
| `0041bb89` | FreeBufferAt04 | likely | Frees the heap buffer at +4 (when present) through the _free thunk and clears +4, +8 and +0xC; reached from five callers including EH unwind funclets, so it is… |
| `0041bbbe` | ReadValuePairReverse | likely | Read helper with a negative advance: takes the pair at *param_1/param_1[1], calls FUN_0040B36E with the negated param_3 (rewinding the cursor) and writes the p… |
| `0041bc75` | FreeBufferAt04B | likely | Second emission of the buffer cleanup: frees the heap buffer at +4 through _free when present and clears +4, +8 and +0xC; called from seven EH unwind funclets … |
| `0041be05` | NewObjectSEHB | likely | SEH-protected allocation/construction helper: allocates through FUN_0040C965(1,0) and, when the allocation succeeds, runs the five-argument constructor FUN_004… |
| `0041be74` | NewObjectSEHC | likely | SEH-protected allocation/construction helper: allocates through FUN_0040C9CD(1,0) and runs the five-argument constructor FUN_0040BB87 on the block, returning i… |
| `0041bf97` | ReadValuePairH | likely | Variant of ReadValuePairA that advances through FUN_0040BF3C, writing the value pair to param_2. |
| `0041bfda` | ReadValuePairI | likely | Variant of ReadValuePairA that advances through FUN_0040BF71, writing the value pair to param_2. |
| `0041c032` | ReadValuePairJ | likely | Variant of ReadValuePairA that advances through FUN_0040BFA6, writing the value pair to param_2. |
| `0041c05f` | ReadValuePairK | likely | Variant of ReadValuePairA that advances through FUN_0040BFDB, writing the value pair to param_2. |
| `0041c18c` | NewObjectSEHD0x1C | likely | SEH-protected new helper: allocates 0x1C bytes with FUN_004D9207 and runs the five-argument constructor FUN_0041760B on the block, returning it. |
| `0041c1f8` | NewObjectSEHE0x20 | likely | SEH-protected new helper: allocates 0x20 bytes with FUN_004D9207 and runs the five-argument constructor FUN_0041763E, returning the block. |
| `0041c264` | NewObjectSEHF | likely | SEH-protected new helper: allocates through FUN_0040CAC2(1,0), zeroes the first three dwords and sets the state bytes at +0x30 to 1 and +0x31 to 0. |
| `0041c33c` | NewObjectSEHH | likely | SEH-protected new helper: allocates through FUN_0040CB25(1,0), zeroes the first three dwords and sets the bytes at +0xE to 1 and +0xF to 0. |
| `0041c810` | ReadValuePairL | likely | Variant of ReadValuePairA that advances through FUN_0040CC38 (stride 0x10), writing the value pair to param_2. |
| `0041d255` | IntersectRect | likely | Rectangle intersection: given two rects (param_1 and param_2 as min-x/min-y/max-x/max-y quads) it returns the intersection in param_1, or resets param_1 to (0,… |
| `0041d317` | ScalarDeletingDtor3 | likely | Third emission of the deleting destructor for the FUN_004149C8 destructor body: frees the object through _free when the low bit of param_2 is set; its callers … |
| `0041d36e` | CopyInitComposite30 | likely | Copy-initializes a 0x30-byte composite by clearing fields at +0,+4,+0x0C,+0x10..+0x24,+0x28,+0x2C, then calling FUN_00418662(this,param_2) to copy-assign its t… |
| `0041e4ba` | RecordOpsDispatch4 | likely | Class-ops dispatch for a 4-byte record type: 0 returns param_2 unchanged, 1 copies 4 bytes from param_3 to param_2, 2 allocates a 4-byte record, 3 allocates th… |
| `0041e544` | CtorVtables175DBB0C | likely | Copy/initialise constructor for the vtable pair 0x175DBB0/0x175DBAC: installs the base pair 0x175CEC8/0x175D39C via FUN_004C9E10, switches both slots to the fi… |
| `0042519e` | ScalarDeletingDtor | likely | Scalar deleting destructor: runs the destructor body FUN_004221f2 (destroys the element range at +4..+8 and frees the allocation at +4), then frees the object … |
| `0042798a` | ManagerScanByTag | likely | Scans the object list reached through 0053845d(0xffffffff): initializes a list iterator with 0040b244, walks entries with 0041b7f5/0040ad28/0040b27b/0040b2b5, … |
| `00428d16` | DeletingDtorVia0052f4ea | likely | MSVC scalar deleting destructor: runs the destructor chain at 0052f4ea (a teardown whose own analysis references the GMessageTarget vtable) and, when the low b… |
| `00428f29` | CtorVtables175E498 | likely | Constructor of the large GFC control class whose primary table is the data at 0x175e498: runs the GObjectBase base constructor 004d99d1, zeroes +0x18/+0x1C/+0x… |
| `0042904c` | HasFlag0x1000000 | likely | Flag query: returns 1 when the dword at +0x40 has bit 0x1000000 set, otherwise 0. |
| `00429061` | HasFlag0x2000000 | likely | Flag query: returns 1 when the dword at +0x40 has bit 0x2000000 set, otherwise 0. |
| `00429076` | CtorVtables175E498B | likely | Second constructor overload of the 0x175e498 control class: same sequence as 00428f29 (GObjectBase base, zeroed link fields, 004121d4, vtable installs includin… |
| `0042912f` | CtorVtables175E498C | likely | Third constructor overload of the 0x175e498 control class: identical initialization sequence, forwarding three caller arguments into the tail initializer 0040e… |
| `004291d2` | CtorVtables175E498D | likely | Fourth constructor overload in the 0x175e498 table family: performs the usual base/vtable/field initialization, builds a local five-dword block (0x8800,0,0,0,0… |
| `00429287` | CtorVtables175E498E | likely | Fifth constructor overload of the 0x175e498 control class: the same base/vtable/zeroing prologue, then a direct four-argument call to 0040ea02 (no local argume… |
| `00429325` | CtorVtables175E498F | likely | Sixth constructor overload of the 0x175e498 control class: same prologue, then the shared initializer 0040ea02 with a zero argument block, then copies the sour… |
| `00429a3e` | CtorVtables175D4C8 | likely | Constructor of a second control class (primary table 0x175d4c8): runs the GObjectBase base constructor, zeroes +0x18/+0x1C/+0x20, installs 0x175d4c8/0x175d4c4 … |
| `00429ae0` | WidgetSetTarget | likely | Sets the control's message-target pointer at +0x44: when the old target differs it clears option bit 0x100, releases the old target through 00427e6c and marks … |
| `0042a60c` | WidgetNotifySubtree | likely | Recursive notification walk over a control subtree: when the pointer at +0x44 is non-zero and its field +0xE4 is set it re-registers through 00427db2, then if … |
| `0042a73c` | WidgetClearOption200 | likely | Clears bit 0x200 of the dword at +0x90 when it is set and the pointer at +0x44 is non-zero, then runs the tag scan 0042798a for the message tag at 01bc3fac wit… |
| `0042a77f` | WidgetClearOption800 | likely | Clears option bit 0x800 on the control's +0x90 word when set and a target exists, then runs the tag scan 0042798a for the message tag at 01bc2a9c; returns whet… |
| `0042a7c2` | WidgetUpdateOrRefresh | likely | Refresh routine of the control class: resolves a display manager through 0040ee03(0) and invokes its virtual slots 0xD0 and 0xF8 for the control's field +0xB8 … |
| `0042a8b7` | CtorVtables175E718 | likely | Copy-style constructor of a third control class (primary table 0x175e718): runs the base initializer 004e68fb on the source, installs 0x175e718/0x175e714 at +0… |
| `0042a93e` | DeletingDtorVia0052f673 | likely | MSVC scalar deleting destructor: runs the destructor chain at 0052f673 and, when the flag argument's low bit is set, frees the object through the free thunk 00… |
| `0042a962` | CtorVtables175D9E8 | likely | Constructor of a fourth control class (primary table 0x175d9e8): runs the GObjectBase base constructor, zeroes +0x18/+0x1C/+0x20, installs 0x175d9e8/0x175d9e4 … |
| `0042aa23` | WidgetPropagateTarget | likely | Stores the argument at the control's field +0xDC and then enumerates children through the container's virtual slot +0x50 with a stack GObjectIterator of kind 3… |
| `0042abfd` | DeletingDtorVia0042abc3 | likely | MSVC scalar deleting destructor: runs the destructor chain 0042abc3 and frees the object through the free thunk 0047da09 (a jump to the CRT free 004c6a61) when… |
| `0042ad22` | DeletingDtorVia00425522 | likely | MSVC scalar deleting destructor: runs the destructor chain 00425522 and frees the object through the free thunk 004092b9 (a jump to the CRT free 004c6a61) when… |
| `0042ae47` | DeletingDtorVia0042554c | likely | MSVC scalar deleting destructor: runs the destructor chain 0042554c and frees the object through the free thunk 0040936e (a jump to the CRT free 004c6a61) when… |
| `0042b061` | CtorVtables175ECF8B | likely | Second constructor emission of the 0x175ecf8 data-access object: runs the base initializer 00409180 on the source, installs the intermediate 0x175d1a0 table, c… |
| `0042b0af` | WidgetSetDirtyUp | likely | Invalidation walk: starting at the given control it sets the 0xF00 dirty bits on the 16-bit flag word at +0x14 of every ancestor (following the parent pointer … |
| `0042b1ea` | WidgetUpdateVisible | likely | Visibility/state update: sets flag 0x400, compares the control's own virtual slot +0x18 result against a bit derived from the flag word, and on a change sets f… |
| `0042b27f` | WidgetSetAlign | likely | Alignment setter: when either of the two bytes at +0x16/+0x17 differs from the two bytes supplied it stores both bytes and calls the invalidation walk 0042b0af. |
| `0042b2c9` | WidgetGetOrCreateRenderObj | likely | Lazy getter for the pointer at +0x44 (the slot that 00429ae0 installs as the message target and that sibling methods query as the drawing object): when the slo… |
| `0042b352` | WidgetIsVisible | likely | Visibility query: when the control's 0x400 flag is set it resolves the object at +0x44 through 0042b2c9 and updates the low flag bits to 4 when the object exis… |
| `0042b3a6` | WidgetGetPosition | likely | Position query: resolves the object at +0x44 through 0042b2c9 (returning a zero pair when absent) and otherwise asks the object's virtual slot +0x194 for the p… |
| `0042b4d0` | WidgetMoveBy | likely | Moves the control by (dx, dy): after resolving the render object through 0042b2c9 it adds param_2 to the x fields at +0x1C/+0x24 and param_3 to the y fields at… |
| `0042b538` | WidgetQueryScalar214 | likely | Scalar query that resolves the object at +0x44 through 0042b2c9 and tail-jumps through its virtual slot +0x214; returns 0 when the pointer is empty. |
| `0042b68d` | WidgetSetParent | likely | Sets or clears the control's parent pointer at +0x18: self-parenting returns success early, a non-root child detach is refused, otherwise it updates the low fl… |
| `0042b71a` | WidgetSetFlags | likely | Applies a set/clear mask to the control's 32-bit flag word at +0x10 (new = (old and not clear) or set): when the value changes it marks every entry of the chil… |
| `0042b7b8` | WidgetRemoveChild | likely | Removes a child from the child vector: proceeds only when the argument parent slot (+0x18) equals this control, AddRefs the child, locates it through 00421495,… |
| `0042b883` | WidgetDetachChildren | likely | Detaches every child of the control: walks the child vector at +0x44 through 00427f67/0040a6b3, clears each child's parent slot at +0x18 and notifies it throug… |
| `0042b8f1` | WidgetAddChild | likely | Adds a child control: refuses (returning the flag word with the low byte cleared) when the argument is null or already has a parent, otherwise applies alignmen… |
| `0042babb` | SetStateMaskAndPair | likely | Merges bits 16..23 of param_3 into the packed state word +0x10 (uVar2 = (uVar1 ^ param_3) & 0xff0000 ^ uVar1) and, when bit 0x10000 or 0x100000 of the merged s… |
| `0042bb25` | SetWord5C | likely | When the 16-bit field +0x5C differs from param_2 or the dirty byte +0x75 is set, it stores param_2 at +0x5C, zeroes the cached dwords +0x60/+0x64/+0x68/+0x6C, … |
| `0042c5be` | ResolveHostInterface | likely | Queries the receiver's own vtable slot +0x64; when it returns 0 it clears the cached reference through slot +0x1c(0xffffffff), dispatches a default record buil… |
| `0042cf9f` | SetScrollPosition | likely | Setter for the object's scroll/origin pair +0xA4/+0xA8: returns 0 when both fields already match; otherwise it stores them, invalidates through vtable slot +0x… |
| `0042d572` | AddChildEntry | likely | Keyed child-record add: lazily allocates the 16-byte carrier vector at this+0xD8 (FUN_00d833a6(0x10), begin/end/cap zeroed) and, when the receiver has a +0x44 … |
| `0042e4c9` | DispatchEventB | likely | Uses the same activity, busy, listener and receiver +0xEC gates as 0042e3c8, including the optional FUN_0042a60c refresh. When dispatch is allowed it allocates… |
| `0042e5c3` | DispatchEventC | likely | Uses the same activity, busy, listener and receiver +0xEC gates as 0042e3c8, with FUN_0042a60c refresh argument 1. A null payload builds an empty event and see… |
| `0042e7a3` | DispatchEventD | likely | Uses the same activity, busy, listener and receiver +0xEC gates as 0042e3c8; unless param_3 bit 0x100 is set it calls FUN_0042a60c with refresh argument 1. It … |
| `0042ef3c` | SyncRendererChildren | likely | Gets the display context and returns 0 when it is missing or the render-handle refresh check fails. When this+0x4C is set, it removes the object render handle … |
| `0042f122` | MeasureContentBounds | likely | Zeroes the four-int output rectangle and, when the parent-chain predicate and context slot +0xD8 succeed, adds that context rectangle. If +0x90 bit 0x200 is se… |
| `004307f5` | SetAttachedObject | likely | Reference-counted assignment for the attached-object pointer +0xB4: when the pointer differs it add-refs the old value through slot +0xC, add-refs the new one,… |
| `0043086f` | GetParentObject | likely | Parent lookup: takes the enclosing object (FUN_00406665), reads the object attached at its +0xB4, resolves it through the element search FUN_0042b9bc(param_1) … |
| `004309b5` | CtorVtables175CDA0 | likely | Calls FUN_0042a8b7 with this in ECX and the caller argument on the stack, writes vtable pointers 0x175cda0 and 0x175cd98 at +0 and +4, then returns this. |
| `0043190a` | CommitSubObjectAcquire2 | likely | Second emission of the sub-object commit path: when the valid byte at +0x18 is set and the state byte at +4 is clear it checks record[5] non-zero and the recei… |
| `00432060` | ToggleEntryForTarget | likely | Scans the receiver's 0x30-byte entry list (count/at via FUN_0040a3bf/FUN_0040a3d2) for the entry whose target field at +0x28 equals param_2; when param_3 is se… |
| `00432369` | ToggleEntryForTargetId | likely | When the object at +0x150 exists and bit 0x400 of the state word at +0xec is set, scans the receiver's 0x30-byte entry list for the entry whose target id (targ… |
| `00434398` | SetStateBit10AndRefresh | likely | Reads the state word at +0xec and, when bit 10 (0x400) differs from param_2, rewrites that bit and, if state bit 4 is set, calls FUN_0043214f when enabling or … |
| `00435577` | LayoutItemCopyCtor | likely | Copy-constructs a layout item from param_2: base copy FUN_004151df(param_2), writes the class vtables (PTR_FUN_0175ee28 at +0, PTR_LAB_0175ee20 at +4), sets +0… |
| `004355e1` | LayoutItemDtor | likely | Destructor body: calls FUN_004d83ba to release the interned string, releases the interface pointer at +0x44 through its slot +0x10 when non-null, then runs the… |
| `00435602` | LayoutItemScalarDeletingDtor | likely | Runs the layout-item destructor FUN_004355e1 and, when the flag argument's low bit is set, frees the object through the CRT free thunk FUN_004c6a61; returns th… |
| `0043561e` | LayoutItemCtor | likely | Constructor: runs FUN_0041b670, writes the class vtables, zeroes +0x44 and the default string at +0x48, swaps param_2 into +0x44 with acquire (slot +0xc) and r… |
| `004357c1` | LayoutItemCopyCtor2 | likely | Copy-constructs from param_2: base copy FUN_004151df(param_2), writes the class vtables (PTR_FUN_0175ee80/PTR_LAB_0175ee78), copies +0x44 to +0x44, the 16-bit … |
| `0043583a` | LayoutItemCtor2 | likely | Constructor variant: runs FUN_0041b670, writes the class vtables (PTR_FUN_0175ee80/PTR_LAB_0175ee78), zeroes the 16-bit field at +0x48, stores param_4 at +0x10… |
| `00435882` | LayoutItemCtor3 | likely | Constructor variant: runs FUN_0041b670, stores param_4 at +0x10 and param_3 at +0x44, writes the class vtables (PTR_FUN_0175ee80/PTR_LAB_0175ee78), stores the … |
| `004358c7` | CopyCtorWithChildVector | likely | Copy-constructs from param_2: base copy FUN_004151df(param_2), writes the class vtables (PTR_FUN_0175eed8/PTR_LAB_0175eed0), copy-constructs the source's child… |
| `0043591d` | LayoutItemDtor2 | likely | Destructor body: writes the class vtables, and when the owner pointer at +0x18 is non-null either calls the owner's slot +0x5c(this) (flag byte bit 0 clear) or… |
| `00435a5a` | CloneItemChildren | likely | Creates a copy of the item structure through FUN_004155a8, copies the dword pair +0x54/+0x58 into the copy, then for each element of the source item list (FUN_… |
| `00435b07` | InsertItemIntoOwnerList | likely | Rejects items that already have an owner (+0x18 non-null), sets the owner to param_1, notifies the global container obtained through the accessor thunk (slot +… |
| `00435ba2` | AddLayoutItemFromControl | likely | Rejects a null control or one whose field +0x48 is non-null and whose global-context check fails; otherwise allocates 0x54 bytes (FUN_00409455), constructs the… |
| `00435c11` | SetChildStateFlag | likely | Resolves an object through FUN_0042b9bc(param_1), acquires it (slot +0xc), and when its flag byte at +0x14 has the low bit clear and its owner (+0x18) is non-n… |
| `00436350` | RecalculateLayout | likely | Recomputes a box layout: reads the inner margin rect (FUN_0041eea4/FUN_00416037/FUN_004155fe), derives the available width/height, computes each item's target … |
| `0043682c` | CreateAndAddItemA | likely | Allocates 0x54 bytes (FUN_00409455) and, when the allocation succeeded, constructs a layout item through FUN_0043583a with the caller's arguments; then asks th… |
| `00436886` | CreateAndAddItemB | likely | Same create-and-add pattern as 0043682c but constructs the item through FUN_0043583a(param_3,param_2,param_4,&stack16) - the callee's argument order differs an… |
| `004368df` | CreateAndAddItemC | likely | Same create-and-add pattern but constructs the item through FUN_00435882(param_3,param_2,param_4,&stack16) - the explicit-16-bit-id constructor variant; adds v… |
| `00436938` | ReadBoxLayoutProperties | likely | Delegates to 0043598a (FixedSize/Items), zeroes the box-layout block +0x5c..+0x78, then reads UseSeparator (bool to +0x74), UseOuterMargin (bool to +0x75), Out… |
| `00436ab3` | CopyControlToItem | likely | Clones the receiver into a new item through FUN_00435a5a(param_2) and, when that succeeded, copies the control fields +0x78, the 16-bit values +0x5c/+0x5e, the… |
| `00436b0b` | LayoutItemCopyCtor4 | likely | Copy constructor for the fourth layout-item variant: runs FUN_004358c7(param_2), writes the class vtables (PTR_FUN_0175efe8/PTR_LAB_0175efe0) and copies the so… |
| `00436d05` | AppendRectToList | likely | 18-byte forwarding call (push [esp+8]; mov ecx,[esp+8]; call FUN_0042c4f7; mov eax,[esp+4]; ret): appends one value to the receiver's array through FUN_0042c4f… |
| `00436edc` | CtorVtables175EED8 | likely | Default constructor of the 0x175eed8-vtable class: runs FUN_0041b670, writes PTR_FUN_0175eed8/PTR_LAB_0175eed0 and zeroes the five dwords at +0x48..+0x58; retu… |
| `00436f22` | Thunk_0043591d | likely | Five-byte jump thunk: the bytes are e9 f6 e9 ff ff, an unconditional jump to 0043591d, the third-emission destructor body (whose content Ghidra also renders at… |
| `004376b3` | FindFocusCandidate | likely | Walks child iterators, filters objects with state masks 0x133 and 0x1000000, recurses for codes 8/9, and for codes 10..13 builds directional bounds and calls 0… |
| `00437eb2` | SetFocus | likely | Requires the owner at +0x44, searches through 004376b3, swaps the owner field +0x170 to this object, manages reference counts, and emits records through virtua… |
| `004380de` | HandleFocusNavigationEvent | likely | Normalizes the receiver to object+0x24, compares six event inputs through 004c0ec3, maps matches to codes 8, 9, 12, 13, 10, or 11, invokes 004376b3, and return… |
| `004381ea` | HandleFocusNavigationSubcode | likely | Checks a 16-bit value against DAT_01bc75a4, maps param_2 subcodes 0, 1, 2, 3, 8, and 9 to search codes 12, 13, 10, 11, 8, and 9, then calls 004376b3 and condit… |
| `0043824a` | ClearOwnedFields | likely | Calls 00407b9d on a local copy, resolves an object through slots +0x64/+8 and calls its +0x44 slot; releases only the reference at +0xd0 through slot +0x10, th… |
| `0043833b` | SetState | likely | Rejects requests with the high flag bit or unsatisfied masks, folds request bit 0x300 into bit 3, calls 0042d287 for a new 64-bit value and pending flags, stor… |
| `00438734` | UpdateObjectRegion | likely | Returns 0 when slot +0xa8 is false and param_3 bit 1 is clear, or when 0040edc6 returns null. Otherwise it may dispatch DAT_01bc3ac8 to owner-list entries, der… |
| `00438ab0` | RefreshRegion | likely | When +0x44 is null returns 1; otherwise a flag gate either calls 0042a60c and returns 1, or uses slot +0x220 and slot +0x250 for local bounds, or captures a su… |
| `00438b62` | RefreshFromConvertedInput | likely | With +0x44 set and either +0xec bit 0x100 or param_3 bit 5 set, converts param_2 through 004c9ed9, calls 00438734 and 004c9f6e; otherwise it may call 0042a60c.… |
| `00438bc3` | RefreshRegionTranslated | likely | Uses the manager/capability gate; a null param_2 calls slots +0x220 and +0x24c, while a non-null param_2 is captured, translated by adding fields +0x94/+0x98 t… |
| `00438cb8` | ConstrainSizePairByMetric | likely | Calls slot +0x1fc(param_2,param_3). Only when this+0xb4 is nonzero and metric param_3 is not 2 does it walk the owner list and max-merge cached dimensions from… |
| `0043978f` | ObjectLifecycleModeDispatcher | likely | Dispatches modes 0 and 2 to 004076dd (mode 2 first allocates 0x34 bytes), modes 1 and 3 to 004309b5 (mode 3 first allocates 0x34 bytes), mode 4 to 0052f673, an… |
| `0043981a` | InsertChildAtDepth | likely | Rejects null or already-flagged children, walks siblings comparing depth at +0x8c under mode param_3, marks the child and inserts through interface slot +0x94.… |
| `00439e6e` | DetachChildAndNotify | likely | Checks that the child is non-null, not marked by bits 1 or 2 at +0x90, and reports this same container; AddRefs it, sets bit 2, dispatches lifecycle records an… |
| `0043aa15` | ProbeSlotA4AndCleanup | likely | Runs state-dependent local notifications and child iteration, then probes the interface at +0xbc through slot +0xa4(3). If the probe returns zero, it dispatche… |
| `0043acef` | BuildViewportRectList | likely | Selects a viewport source based on fields +0x120 and +0x11c: delegates to the object at +0x124 slot +0x54 when mode equals 1, otherwise reads bounds from the v… |
| `0043ad86` | FindNearestRectForPoint | likely | Obtains the current list of viewport rectangles, returns the first rectangle containing the supplied point, or scans all rectangles and selects the one with mi… |
| `0043aefa` | TranslateRectAgainstSelectedRegion | likely | Computes the input rectangle centre and selects a region with 0043ad86. It translates all four coordinates to correct bottom/right overflow, then translates ag… |
| `0043b27d` | InitializeTwoTreeHeadersAndVtables | likely | Runs a base constructor, writes intermediate vtable pointers, initializes two tree headers, applies the supplied base argument through 004d9a26, then installs … |
| `0043b2d7` | ComputePlacementOffset | likely | Gets accumulated ancestor coordinates in local_c/local_8 through 0040f800. With a non-null owner at +0x44, forms a reference rectangle and calls 0043afa4 with … |
| `0043b466` | GInterfaceDerivedLifecycleDispatcher | likely | Dispatches modes 0 and 1 to separate initialization helpers, modes 2 and 3 to allocation of 0x44 bytes followed by those helpers, mode 4 to destructor helper 0… |
| `0043b4f1` | ScalarDeletingDestructor | likely | Runs destructor body 004354b2; when flag bit 0 is set, frees the object through 004c6a61; returns the original object pointer otherwise. |
| `0043b50d` | LayoutItemLifecycleDispatch | likely | Dispatches modes 0..5 for the LayoutItem family: mode 0 initializes a supplied object through 0043561e, mode 2 allocates 0x54 bytes then initializes it, modes … |
| `0043b649` | LayoutItemVariantLifecycleDispatch | likely | Dispatches lifecycle modes for a second LayoutItem construction family: modes 0 and 2 call 0043583a with initialization arguments (mode 2 first allocates 0x54 … |
| `0043b718` | ScalarDeletingDestructor2 | likely | Runs destructor body 0043591d; when flag bit 0 is set, frees the object through 004c6a61; returns the original object pointer otherwise. |
| `0043b734` | InitializeFieldsAndVtables | likely | Runs base initializer 00436edc, installs two vtable pointers, zeros the 16-bit fields at +0x5c/+0x5e, dwords at +0x60..+0x70 and +0x78..+0x88, and bytes +0x74/… |
| `0043b78d` | InitializeFieldsAndVtablesVariant | likely | Runs base initializer 00436edc, installs the two vtable pointers, zeros the 16-bit fields at +0x5c/+0x5e, dwords at +0x60..+0x70 and +0x7c..+0x88, and bytes +0… |
| `0043b7ea` | CallWithWordSelectedByField78 | likely | Tests whether the dword at +0x78 is nonzero, selects DAT_01bc2d0c when true or DAT_01bc2e10 when false, then calls 004368df with the boolean, selected 16-bit v… |
| `0043b928` | InitializeFieldsAndVtablesAt5C | likely | Runs base initializer 00436edc, stores the supplied argument at +0x10, installs two vtable pointers, clears dwords at +0x5c, +0x60, and +0x74..+0x80, then retu… |
| `0043b960` | InitializeFieldsAndVtablesAt5CVariant | likely | Runs base initializer 00436edc, stores param_2 at +0x5c and param_3 at +0x10, installs the same vtable pointers as 0043b928, clears +0x60 and +0x74..+0x80, the… |
| `0043b99c` | AppendToGrowableList | likely | Compares the used count and capacity of the sequence at +8; with spare capacity it appends one 0x30-byte entry and updates the sequence pointer, otherwise it i… |
| `0043bb52` | ReleaseFieldsAndBaseObjects | likely | Rewrites three vtable fields, calls 0043824a to clear the object, releases eight non-null interface fields through virtual slot +0x10(0), then runs base/interf… |
| `0043d515` | ApplyPendingStateAndNotify | likely | Applies state words +0x58/+0x5c through 0043833b with mode 1, optionally flushes the supplied object's pending update through 004396f7, invokes this object's v… |
| `0043d552` | FlushPendingUpdateWith8 | likely | Pushes constant 8, calls 004396f7, then returns while popping 8 bytes from its caller's stack; x86 bytes show a normal call wrapper rather than a tail jump. |
| `0043d9cd` | AttachContextFromRecord | likely | Returns entry EAX masked with 0xffffff00 when param_3 or param_4 is null. Otherwise records whether state bit 0x10 was set, may compute a rect and notify slot … |
| `0043dc9a` | AttachContextVariant | likely | Proceeds only when param_2 is non-null and slot +0x30(0) returns zero. It releases old context interfaces, stores/AddRefs param_2 and optional param_3, sets mo… |
| `0043df84` | ForwardCall43B99C | likely | Adds 0x1E0 to ECX, pushes param_1, calls 0043B99C, returns 1, and uses RET 4. |
| `0043e5b0` | GfcCreateInstance5C | likely | Mode-selector object factory for a 0x5C-byte class: mode 0 constructs in place (FUN_00436EDC); mode 1 copy-constructs in place from param_3 (FUN_004358C7); mod… |
| `0043e63b` | GfcCreateInstance8C_B | likely | Mode-selector factory for a second 0x8C-byte class: mode 0 constructs in place (FUN_0043B734(0)); mode 1 copy-constructs in place (FUN_00435C70); mode 2 alloca… |
| `0043e6d0` | GfcCreateInstance84 | likely | Mode-selector factory for a 0x84-byte class: mode 0 constructs in place (FUN_0043B928(0)); mode 1 copy-constructs in place (FUN_00436B0B); mode 2 allocates 0x8… |
| `0043ed0e` | GfcCreateInstanceE8 | likely | Mode-selector factory for a 0xE8-byte class: mode 0 constructs in place (FUN_00429076(0,0)); mode 1 copy-constructs in place from param_3 (FUN_00429325); mode … |
| `0043eda5` | ScalarDeletingDtorE8 | likely | Calls destructor body 0043BB52; when param_2 bit 0 is set, calls the free helper through thunk 00406363; otherwise returns param_1. The function uses RET 4. |
| `0043ff08` | InitControlMessage | likely | Message initialiser: when any of the bits 0x24924 are set in the existing flags dword it releases the previous payload through FUN_004D93A3, then clears the fl… |
| `0043ffd1` | InitLinkedListHead2 | likely | Two-field list-head initialiser: stores 0 in field [0] and the value returned by FUN_004C04EA in field [1]; returns param_1. |
| `00440026` | InitLinkedListHead3 | likely | Two-field list-head initialiser: stores 0 in field [0] and the value returned by FUN_004C050A in field [1]; returns param_1. |
| `0044004b` | InitLinkedListHead4 | likely | Two-field list-head initialiser, byte-for-byte identical to 00440026: stores 0 in field [0] and FUN_004C050A() in field [1]; returns param_1. |
| `00440161` | CtorVtable01762888 | likely | Adapter constructor: runs the base initialiser FUN_004D99D1 (GObjectBase vtable), then installs PTR_FUN_01762888 into [0] and PTR_LAB_01762880 into [1]; return… |
| `004401b1` | ScalarDeletingDtor526562 | likely | Scalar deleting destructor: calls the destructor body FUN_00526562 (which reinstalls the GInterface vtable) and, when the low bit of param_2 is set, frees the … |
| `004403a2` | CtorVtable01762A70 | likely | Adapter constructor: runs FUN_004C6C1C, then writes PTR_FUN_01762A70 into [0]; returns param_1. |
| `004403b4` | ScalarDeletingDtorE6771 | likely | Scalar deleting destructor: calls the destructor body FUN_004E6771 (which reinstalls the GInterface vtable) and, when the low bit of param_2 is set, frees the … |
| `0044046d` | IteratorToElementAt | likely | Initializes *param_2 to index 0 and the sentinel from 004C050A. If param_3 is below the count at this+0x40, writes param_3 and the base pointer this+0xA0. Retu… |
| `004404b1` | CtorVtable01762AD0 | likely | Adapter constructor with payload: runs FUN_004D99D1, installs PTR_FUN_01762AD0 into [0] and PTR_LAB_01762ACC into [1], then copies the four source dwords param… |
| `00440a3a` | CtorVtable01762C7C | likely | Adapter constructor: writes the shared template vtable pointer 0175C44C into [0], copies the dwords at param_2+4 and param_2+8 into [1] and [2], then overwrite… |
| `00440b4a` | CtorVtable01762C98 | likely | Adapter constructor: runs FUN_004C6C1C, then writes PTR_FUN_01762C98 into [0]; returns param_1. |
| `00440b5c` | ScalarDeletingDtorE6771B | likely | Scalar deleting destructor: calls the destructor body FUN_004E6771 and, when the low bit of param_2 is set, frees the object with _free; returns the pointer ot… |
| `00440e3d` | CtorVtable01762D1C | likely | Adapter constructor: writes the shared template vtable 0175C44C into [0], copies param_2+4 and param_2+8 into [1]/[2], then overwrites [0] with the specialised… |
| `00441064` | CtorVtable01762D84 | likely | Adapter constructor: writes the shared template vtable 0175C44C into [0], copies param_2+4 and param_2+8 into [1]/[2], then overwrites [0] with the specialised… |
| `00441174` | CtorVtable01762D98 | likely | Adapter constructor: runs FUN_004C6C1C, then writes PTR_FUN_01762D98 into [0]; returns param_1. |
| `0044118c` | ScalarDeletingDtorE6771C | likely | Scalar deleting destructor: calls the destructor body FUN_004E6771 and, when the low bit of param_2 is set, frees the object with _free; returns the pointer ot… |
| `00441358` | ClampScrollDeltaA | likely | Scroll clamping helper: from the delta param_1 and the record param_4 (signed bytes at [0] and [2] acting as per-unit steps) plus limits param_3[5]/[7]/[9]/[0x… |
| `004414a7` | ClampScrollDeltaB | likely | Scroll clamping helper, twin of 00441358 with a shifted parameter layout: uses the signed bytes at param_4[1]/param_4[3] as per-unit steps and the limits param… |
| `004420d3` | CtorVtable01762EBC | likely | Adapter constructor: writes the shared template vtable 0175C44C into [0], copies param_2+4 and param_2+8 into [1]/[2], then overwrites [0] with the specialised… |
| `004421c6` | CtorVtable01762EC0 | likely | Adapter constructor: runs FUN_004C6C1C, then writes PTR_FUN_01762EC0 into [0]; returns param_1. |
| `004421d8` | ScalarDeletingDtorE6771D | likely | Scalar deleting destructor: calls the destructor body FUN_004E6771 and, when the low bit of param_2 is set, frees the object with _free; returns the pointer ot… |
| `004421f4` | IteratorKind1At140 | likely | Iterator factory: fills *param_2 with {0, sentinel from FUN_004C04CA} and, when the byte at this+0xE4 is non-zero, replaces the pair with {1, this+0x140}; retu… |
| `00442231` | IteratorKind1At140From | likely | Iterator factory variant: fills *param_2 with {0, sentinel from FUN_004C04CA} and, when the byte at *(this+4)+0xE4 is non-zero, replaces the pair with {param_3… |
| `0044253b` | CtorVtable01762F48 | likely | Adapter constructor: writes the shared template vtable 0175C44C into [0], copies param_2+4 and param_2+8 into [1]/[2], then overwrites [0] with the specialised… |
| `004433ab` | CtorVtable01762F60 | likely | Adapter constructor: runs FUN_0052F5B0, then writes PTR_FUN_01762F68 into [0] and PTR_LAB_01762F60 into [1]; returns param_1. |
| `0044350d` | DtorReleaseThreeMembers | likely | Releases members at this+0x20, +0x1C and +0x18 through three calls to 004DC61B, then tail-jumps to destructor body 004434F7. |
| `00443530` | ScalarDeletingDtor526562B | likely | Calls destructor body 004434F7; when param_2 bit 0 is set, calls the shared free helper through thunk 004C6DF1; otherwise returns param_1. The function uses RE… |
| `00443559` | LoadPairFromGlobal | likely | If param_3 is nonnull, writes param_1 to *param_3; always writes DAT_01AF5A7C to *param_2. Returns nothing and uses RET 8. |
| `004437da` | CtorVtables1763298 | likely | Initialiser/constructor: runs the base initialiser FUN_00443499, installs PTR_FUN_01763298 in this[0], PTR_LAB_01763290 in this[1] and PTR_LAB_01763260 in this… |
| `00443894` | DtorVtables1763298 | likely | Destructor body: calls FUN_004dc61b three times (each frees a wide-string member unless it is the empty-string sentinel) and then calls the base destruction FU… |
| `00443a28` | InitSecondaryVtables1762EBC | likely | Secondary-base initialiser: calls FUN_004c9e10 (which writes the shared interface vtable PTR_LAB_0175c44c and records whether the object carries the 0x56471e89… |
| `00443a8f` | InitSecondaryVtables17633CC | likely | Secondary-base initialiser: calls FUN_004c9e10 (shared interface vtable PTR_LAB_0175c44c plus the 0x56471e89 stamp test) and then installs PTR_LAB_017633cc in … |
| `00443abe` | InitSecondaryVtables1762F48 | likely | Secondary-base initialiser: calls FUN_004c9e10 and then installs PTR_LAB_01762f48 in this[0]; returns param_1. |
| `00443ad0` | InitSecondaryVtables1762C7C | likely | Secondary-base initialiser: calls FUN_004c9e10 and then installs PTR_LAB_01762c7c in this[0]; returns param_1. |
| `00443c31` | InitSecondaryVtables1762D1C | likely | Secondary-base initialiser: calls FUN_004c9e10 and then installs PTR_LAB_01762d1c in this[0]; returns param_1. |
| `00443c43` | InitSecondaryVtables1762D84 | likely | Secondary-base initialiser: calls FUN_004c9e10 and then installs PTR_LAB_01762d84 in this[0]; returns param_1. |
| `00444f75` | CallSlot18ThenCheckGString | likely | Calls virtual slot +0x18 of param_1 with (param_2, param_1, &param_3) and, when that returns non-zero, compares the cached type id for the type name "p<c GStri… |
| `00444fcf` | InitEventRecordA | likely | Initialises a 0x1E-byte record: clears the flags dword at +0, writes the 16-bit event/type value param_2 at +0x1c and stores the six caller-supplied dwords in … |
| `00445008` | InitEventRecordB | likely | Initialises a 0x1E-byte record: first releases any interface payload already present through FUN_004d93a3 when the flags dword at +0 has any of the 0x24924 bit… |
| `00445041` | InitEventRecordC | likely | Second emission of the event-record reuse constructor: releases the interface payloads through FUN_004d93a3 when flags at +0 has any 0x24924 bits set, clears +… |
| `0044507a` | InitEventRecordD | likely | Initialises a 0x1E-byte record with four payload dwords: releases the existing interface payloads through FUN_004d93a3 when flags at +0 has 0x24924 bits, write… |
| `00445125` | CtorVtables1763408 | likely | Constructor: runs the base constructor FUN_00440161, stores param_2 in this[5], installs PTR_FUN_01763408 in this[0] and PTR_LAB_01763400 in this[1]; returns t… |
| `00445154` | ScalarDeletingDtor526562_d | likely | Scalar deleting destructor: calls the destructor body FUN_00526562 (which reinstalls the GInterface vtable) and, when the low bit of param_2 is set, deletes th… |
| `004451d2` | CtorVtables1763570 | likely | Constructor: installs PTR_LAB_01762a38 in this[0], runs the secondary-base initialiser FUN_004c9e10, overwrites this[1] with PTR_LAB_01762d1c and then PTR_LAB_… |
| `00445276` | CopyCtorVtables17633CC | likely | Copy-style constructor: writes the shared base vtable PTR_LAB_0175c44c into [0], copies the dwords at param_2+4 and param_2+8 into [1] and [2], then installs P… |
| `004452b1` | CopyCtor4Dwords | likely | Copies four dwords element by element from param_2 into param_1 and returns nothing; used as the lead-in of several window-record copy constructors. |
| `004452d0` | CtorVtable017635A8 | likely | Base initialiser: writes PTR_FUN_017635a8 into [0] and copies the dwords at param_2+4, +8, +0xc and +0x10 into [1], [2], [3] and [4]; returns nothing. |
| `004452f7` | ScalarDeletingDtorE6771_44 | likely | Scalar deleting destructor: calls FUN_004e6771 (destructor body that reinstalls the GInterface vtable) and, when the low bit of param_2 is set, releases the ob… |
| `00445313` | CtorVtable017635C8 | likely | Constructor: runs FUN_004c6c1c (installs the 017635a8 base vtable and two self-linked list heads) and then installs PTR_FUN_017635c8 in [0]; returns the object. |
| `00445325` | ScalarDeletingDtorE6771_45 | likely | Scalar deleting destructor: calls FUN_004e6771 and, when the low bit of param_2 is set, releases the object storage with _free; returns param_1. |
| `004457a9` | ScalarDeletingDtor526562C | likely | Scalar deleting destructor: calls the destructor body FUN_00526562 and, when the low bit of param_2 is set, deletes the object through thunk_FUN_004c6a61; retu… |
| `004458a1` | CtorVtables1763570B | likely | Constructor variant: writes PTR_LAB_01762a38 into [0], calls the 01762d1c adapter constructor FUN_00440e3d with (param_2 ? param_2+4 : 0), then installs PTR_FU… |
| `00446124` | CtorVtable01763790 | likely | Constructor: runs FUN_004c6c1c (017635a8 base vtable plus two self-linked list heads) and then installs PTR_FUN_01763790 in [0]; returns the object. |
| `00446136` | ScalarDeletingDtorE6771_46 | likely | Scalar deleting destructor: calls FUN_004e6771 and, when the low bit of param_2 is set, releases the object storage with _free; returns param_1. |
| `0044655a` | ScalarDeletingDtor52F673 | likely | Scalar deleting destructor: calls the destructor body FUN_0052f673 and, when the low bit of param_2 is set, deletes the object through thunk_FUN_004c6a61; retu… |
| `004466ce` | CtorVtables1763928 | likely | Initialiser/constructor: runs the base initialiser FUN_00443499, installs PTR_FUN_01763928 in [0], PTR_LAB_01763924 in [1] and PTR_LAB_017638f4 in [5], points … |
| `004467c0` | ScalarDeletingDtor4350D | likely | Scalar deleting destructor: runs the destructor body FUN_0044350d (which releases three members and tears down the embedded interface) and, when the low bit of… |
| `00446809` | ScalarDeletingDtor43894 | likely | Scalar deleting destructor: runs the destructor body FUN_00443894 (three string releases plus the base destruction) and, when the low bit of param_2 is set, de… |
| `00446825` | CtorVtables1763298B | likely | Copy-style constructor: runs the property-interface constructor FUN_004467dc(param_2), installs PTR_FUN_01763298 in [0], PTR_LAB_01763290 in [1] and PTR_LAB_01… |
| `00447224` | CallSlot18ThenCheckGString_b | likely | Second emission of the generated GString glue: calls virtual slot +0x18 of param_1 with (param_2, param_1, &param_2) and, when it succeeds, compares the cached… |
| `004472bd` | CtorVtable01762A70_b | likely | Copy-style constructor: runs the record initialiser FUN_004452d0(param_2) and then installs PTR_FUN_01762a70 in [0]; returns the object. |
| `004472d5` | CtorVtable01763A40 | likely | Constructor: runs FUN_004c6c1c (installs the 017635a8 base vtable and two self-linked list heads) and then installs PTR_FUN_01763a40 in [0]; returns the object. |
| `004472e7` | ScalarDeletingDtorE6771_47 | likely | Scalar deleting destructor: calls FUN_004e6771 and, when the low bit of param_2 is set, releases the object storage with _free; returns param_1. |
| `004474fc` | CtorVtable01762C98_b | likely | Copy-style constructor: runs the record initialiser FUN_004472bd(param_2) and then installs PTR_FUN_01762c98 in [0]; returns the object. |
| `00447514` | ClassOpsVtables1763570 | likely | Class-ops dispatcher: action 0 constructs the 01763570-class object through FUN_004451d2, action 1 runs the copy-constructor form FUN_004458a1(param_3), action… |
| `004479e2` | CtorVtable01762D98_b | likely | Copy-style constructor: runs the record initialiser FUN_004472bd(param_2) and then installs PTR_FUN_01762d98 in [0]; returns the object. |
| `004479fa` | CtorVtable01762EC0_b | likely | Copy-style constructor: runs the record initialiser FUN_004472bd(param_2) and then installs PTR_FUN_01762ec0 in [0]; returns the object. |
| `00447b47` | CtorVtables1763928B | likely | Copy-style constructor: runs the property-interface constructor FUN_004467dc(param_2), installs PTR_FUN_01763928 in [0], PTR_LAB_01763924 in [1] and PTR_LAB_01… |
| `00447b9a` | ClassOpsVtables1763298 | likely | Class-ops dispatcher for the 01763298 clipboard-access class: action 0 constructs through FUN_004437da, action 1 runs the copy-constructor form FUN_00446825(pa… |
| `004575b0` | GInterfaceDerivedScalarDeletingDtor | likely | Calls FUN_0045728f to destroy two tree members and restore GInterface::vftable; tests bit 0 of the flag, calls _free(this) when set, then returns this. |
| `00457aaf` | CopyCtorVtables17641C8 | likely | Copies an interface wrapper from the source through FUN_0042a8b7 and then installs the two tables 0x17641c8 and 0x17641c0 at +0 and +4; returns this. |
| `00457bb7` | FactoryVtables17641C8 | likely | Lifetime dispatcher of one GFC reference class: case 0 default-constructs through FUN_004510ba, case 1 copy-constructs through FUN_00457aaf, cases 2/3 allocate… |
| `004592c8` | InterfaceRelease | likely | When the interface pointer at +4 is non-null, calls its virtual slot +8 (release) with the receiver as argument and then zeroes +0 and +4. |
| `00459906` | BufferDescInit | likely | Initialises a packed-record descriptor over (base=param_2, size=param_3): data pointer = base+4, payload size = param_3 and end pointer = base + 4 + align4(par… |
| `004599e9` | BlockChainAdvance | likely | Advances a block cursor: increments the used-count at +4; when it reaches the block limit at block+0x1004 it follows the chain pointer at block+0x1008 and rese… |
| `00459af8` | RefCountedRelease | likely | Reference-count release: decrements the dword at +8 when non-zero and, when it reaches zero, tail-jumps to the object's slot 0 (mov eax,[ecx]; jmp [eax]). |
| `0045a226` | SetGlobal01bc45f0 | likely | Stores the argument into the global dword at 0x1bc45f0 and returns. |
| `0045a887` | BaseInitThunk | likely | Runs the GFC object base initialiser FUN_004c9ebe on the receiver and returns it. |
| `0045b419` | ReleaseObjectBySlot10 | likely | When the pointer at the receiver is non-null it calls the object's virtual slot +0x10 with argument 1 and clears the field. |
| `0045b42e` | RecordCtor176469C | likely | Constructs the record whose vtable is 0x176469c: stores the table at +0, param_2 at +4, 1 at +8, param_3 at +0xC and param_4 at +0x10, then zeroes +0x14..+0x28… |
| `0045b47e` | Record14DeletingDtor | likely | Deleting destructor for arrays of the 0x14-byte record: with bit 2 of the flag clear it runs the element destructor FUN_0045b419 on the receiver and frees it w… |
| `0045b611` | CtorVtables17646A8 | likely | Constructs a GFC object: runs FUN_0052f5b0 and installs the two tables 0x17646a8 and 0x17646a4 at +0 and +4. |
| `0045b94e` | QueryStateViaSlot14 | likely | When the receiver and its virtual slot +8 both succeed it queries the interface at +5 through slot +0x14 and returns 1 only when that returns 0. |
| `0045bfd3` | NotifyThenDeleteOnFlag | likely | Returns 0 for a null candidate; otherwise calls receiver slot +0x64 with candidate fields param_2[6] and param_2, tests the following stack byte, conditionally… |
| `0045c1eb` | StoreInterfacePtrAddRef | likely | When the source pointer is non-null it calls its virtual slot +0xC (add-reference) and stores the pointer into the receiver. |
| `0045c281` | CtorVtbl17648b8 | likely | Constructs a small GFC object: runs the base initialiser FUN_004c9e10 and installs the table 0x17648b8 at +0. |
| `0045c2fb` | CtorVtbl17648bc | likely | Constructs a small GFC object: base initialiser FUN_004c9e10 plus the table 0x17648bc. |
| `0045c375` | CtorVtbl17648c0 | likely | Constructs a small GFC object: base initialiser FUN_004c9e10 plus the table 0x17648c0. |
| `0045c444` | CtorVtbl176487c | likely | Constructs a small GFC object: base initialiser FUN_004c9e10 plus the table 0x176487c. |
| `0045c488` | ReleaseAndStorePtr | likely | When the current pointer at the receiver is non-null it calls the object's virtual slot +0x10 with argument 0 (release) and then stores param_2. |
| `0045c4aa` | CtorVtbl17648b0 | likely | Constructs a small GFC object: base initialiser FUN_004c9e10 plus the table 0x17648b0. |
| `0045c4c0` | CtorVtbl17648b4 | likely | Calls the shared base initialiser FUN_004c9e10 then installs the table pointer 0x17648b4 at +0 and returns the receiver. |
| `0045c52a` | RefCountStorePtr | likely | When the pointer at the receiver is non-null it releases it through FUN_00459af8 and then stores param_2. |
| `0045c5b4` | ReleaseAndStorePtr_2 | likely | Second emission of the pointer store with release of the previous value through slot +0x10(0). |
| `0045c6ad` | RefCountedPtrCopy | likely | Copies a raw pointer from a source slot and increments the reference counter stored at the first dword of the pointed object. |
| `0045c6d1` | RefCountedPtrCopy_2 | likely | Second emission of the reference-counting pointer copy. |
| `0045c6f5` | RefCountedPtrCopy_3 | likely | Third emission of the reference-counting pointer copy. |
| `0045c719` | RefCountedPtrCopy_4 | likely | Fourth emission of the reference-counting pointer copy. |
| `0045c735` | RefCountedPtrCopy_5 | likely | Fifth emission of the reference-counting pointer copy. |
| `0045c74b` | CtorVtbl17648c4 | likely | Constructs a small GFC object: base initialiser FUN_004c9e10 plus the table 0x17648c4. |
| `0045c801` | CtorVtbl17647c4 | likely | Constructs a small GFC object: base initialiser FUN_004c9e10 plus the table 0x17647c4. |
| `0045c8e1` | ReleaseAndStorePtr_3 | likely | Third emission of the pointer store with release of the previous value through slot +0x10(0). |
| `0045c95e` | ReleaseAndStorePtr_4 | likely | Fourth emission of the pointer store with release of the previous value through slot +0x10(0). |
| `0045c9d2` | ReleaseAndStorePtr_5 | likely | Fifth emission of the pointer store with release of the previous value through slot +0x10(0). |
| `0045d10b` | CopyTaggedRefPtr | likely | Copies a 2-byte tag (param_2 -> param_1) and stores the refcounted pointer *param_3 into param_1+4: when the source pointer is non-null it first calls the obje… |
| `0045e48f` | CopyTaggedRefPtr_2 | likely | Copy constructor for a tagged reference-counted pointer: copies the tag word from *param_2, retains the pointer at param_2[2] through virtual slot +0xc when no… |
| `0045e578` | RefPtrCopyCtor | likely | Copy-constructs a reference-counted pointer: when the source pointer is non-null it dispatches the object's virtual slot +0xc (retain/AddRef-style), then store… |
| `0045f1ff` | RefPtrCopyIfNonNull | likely | When the destination is non-null: retains the source pointer *param_2 through the object's virtual slot +0xc (AddRef-style) if non-null, then stores it into th… |
| `0045f220` | RefPtrCopyIfNonNull_2 | likely | Second emission of the null-tolerant refcounted pointer copy: retain the non-null source via virtual slot +0xc, store it into the non-null destination. |
| `0045f26e` | RefPtrScalarDeletingDtor | likely | Destructor of a refcounted pointer holder: when the held object is non-null it calls the object's virtual slot +0x10 with argument 0 (release/destroy entry), t… |
| `0045f292` | RefPtrScalarDeletingDtor_2 | likely | Second emission of the refcounted pointer holder destructor: virtual slot +0x10(0) release of the held object when non-null, conditional _free of the holder, r… |
| `0045f501` | DeleteHelper_526562 | likely | Tail-call wrapper with no body of its own: forwards its two arguments (object, deleting flag) to FUN_0045fa13, the deleting-destructor helper of the 00526562 c… |
| `0045fa13` | WrapperScalarDeletingDtor_526562 | likely | Deleting-destructor body: runs the 00526562 cleanup chain (the destructor body of the class that 00406208 also tail-calls) and, when the flag's low bit is set,… |
| `0045fd53` | CtorVtable017648C0 | likely | Constructor fragment of a two-field object: first stores PTR_LAB_0175c44c into +0 (the same base-table pointer the GInterface destructor sequence at 00462140 w… |
| `00463e7f` | HitTestRegisteredEntries | likely | If this object field +0x44 is nonzero, calls 00463dc9 on the object. Then obtains the global registry count through 0045c668, iterates indices through 0045c686… |
| `0046c9ac` | RectArrayCompact | likely | Compacts the object rectangle array: when the used count at +0x50 has reached the capacity at +0x48 it allocates 0x3fc bytes (FUN_004d9207), copies the live ta… |
| `0046d14c` | Vector12BackPointer | likely | Returns the address of the last element of the 12-byte-element store at +4/+8: end() is built with FUN_00464ec7 (container word plus the pointer at +8), a copy… |
| `0046d175` | VectorBackPointerB | likely | Same end()-minus-one accessor as Vector12BackPointer for another container instantiation: end() from FUN_00464a6c, a backwards step FUN_0046b2b1 and dereferenc… |
| `0046d26d` | VectorBackPointerD | likely | Same end()-minus-one accessor: FUN_00464943 builds the iterator, FUN_0046b214 steps it backwards, FUN_0045d186 dereferences it. |
| `0046da0e` | EraseSlot16MoveTail | likely | Removes one 0x10-byte element from the array at +4/+8: calls FUN_00469ebd (which forwards to the 0x10-byte element copy helper FUN_00465772) with (removed+0x10… |
| `0046de29` | SlotTableScalarDeletingDtor | likely | Deleting destructor: runs SlotTableShutdown and frees the object when bit 0 of param_2 is set, returning the object pointer. |
| `0046deb7` | PopBack4Release | likely | When the 4-byte element store at +4/+8 is non-empty (FUN_0045c293) it releases the last element with FUN_0046d7a3 (ReleaseRefPtr4Thunk) and moves the end point… |
| `0046dedf` | PopBack4Release_2 | likely | Second pop-back with release, using FUN_0046d84c for the element release. |
| `0046f7f2` | RemoveIndexedSlot | likely | Clears only slot[index]+0xC for the index stored at record+0x18. For a nonzero index, constructs iterators and compacts the 0x10-byte slot array through 0046da… |
| `0046f9a6` | DtorVtable176469C | likely | Installs vtable 0176469c and releases the table at +0x14 through 0046f7f2 when present. If +0x1c is zero, clears +0x24 on the object at +0x20 when present; oth… |
| `0047b61a` | StorePointerThenMode2 | likely | Copies param_1 into a local pointer through FUN_004cd09c, then passes that local to FUN_004cd239; the latter conditionally invokes the stored object's vtable s… |
| `0047ca5e` | CreateDibSurface | likely | Reinitializes this drawing surface: derives bytes-per-pixel from (param_2>>21 & 3)+1, aligns the row pitch to 4 bytes, allocates pixel storage (a CreateDIBSect… |
| `0047d2a9` | SurfaceDrawFrame | likely | Draws a rectangle border with four calls to virtual slot 0x1dc (the two horizontal edges and the two vertical edges between param_3+1 and param_5-1) and return… |
| `0047d328` | SurfaceDrawFullBorder | likely | Draws the border of the entire surface by calling virtual slot 0x1dc once with the rectangle (0,0,width-1,height-1) read from the surface description at this[0… |
| `0047d40b` | GetSurfaceHandle | likely | Returns the dword stored at this+0x54+4*index for index 0 or 1 (the surface's two GDI handle fields) and 0 for any other index. |
| `0047d41f` | ClearColorTable | likely | Releases the color-table buffer at this+0x1c through the engine free routine and clears the table state at +0x14, +0x18, +0x1a and +0x1c. |
| `0047d500` | SetColorTableEntry | likely | Writes color value param_3 into entry param_2 of the color table, using the element size from this+0x18 (2, 3 or 4 bytes; the 3-byte case stores a high byte at… |
| `0047d61c` | SetPixelWithFormatConvert | likely | Converts packed color param_3 according to the surface pixel format in this[5]&0xfff (0x444 -> 12-bit 444, 0x555 -> 15-bit 555, 0x565 -> 16-bit 565) and stores… |
| `0047d6b2` | FindNearestColor24 | likely | Scans the color table (entries fetched through virtual slot 0x140) and returns the index of the entry whose low three packed bytes are closest to param_2 in sq… |
| `0047d9ec` | AllocStampedBlock | likely | Allocates param_1 bytes with the engine small-block allocator and, on success, stamps the 8-byte object-header magic 0x56471e89 / 0x9fe1234a at offsets +8 and … |
| `0047da84` | AllocStampedBlock2 | likely | Second copy of the stamped-allocation helper: allocates param_1 bytes with the engine allocator and stamps magic 0x56471e89 / 0x9fe1234a at +8/+12. |
| `0047deae` | DrawRecordEquals | likely | Equality predicate for two drawing-state records: compares the two counted name buffers (length dword then byte-wise via FUN_004ca923), then the flags word mas… |
| `0047e058` | DrawRecordCompare | likely | Full comparison of two drawing-state records: returns 1 when DrawRecordEquals is false and DrawRecordLess says the first is smaller, or when the records are eq… |
| `0047e289` | AllocStampedBlock3 | likely | Third copy of the stamped-allocation helper: allocates param_1 bytes with the engine allocator and stamps magic 0x56471e89 / 0x9fe1234a at +8/+12. |
| `0047e383` | ScalarDeletingDtor | likely | Calls FUN_00526562, then if bit 0 of param_2 is set frees param_1 through thunk 004c6df1; otherwise returns param_1. |
| `0047e482` | ReadMetricAndOptionalVirtualAdvance | likely | Reads the value at param_3+0x34, or +0x40 when param_6 is nonzero; if bit 3 of *(param_1+7) is set, adds the result of virtual slot 0x144(param_2,param_4,param… |
| `0047e683` | AllocStampedBlock5 | likely | Fifth copy of the stamped-allocation helper (engine allocator plus magic at +8/+12). |
| `0047e728` | ExpandByteMaskPlanes | likely | Only when the byte at param_1+3 equals 1, computes a row length from header words at +6 and +10, clears param_2, then consumes bytes from param_3 and expands e… |
| `0047e8a4` | ExpandMarkerRunBytes | likely | Expands literal bytes and run markers into param_1 up to param_3 output bytes: a marker with its top two bits set repeats the following byte (marker & 0x3f) ti… |
| `0047ead7` | AllocStampedBlock6 | likely | Sixth copy of the stamped-allocation helper (engine allocator plus magic at +8/+12). |
| `0047ebe7` | AllocStampedBlock7 | likely | Seventh copy of the stamped-allocation helper (engine allocator plus magic at +8/+12). |
| `0047ee08` | FormatColorHexRgba | likely | Same as FormatColorHexRgb but formats four bytes (with alpha) using the "#%0Nx%0Nx%0Nx%0Nx" literals selected by param_3 or by the automatic digit-width heuris… |
| `0047ef15` | HsvToRgb | likely | Converts hue/saturation/value (param_2 degrees wrapped by 0x168=360, param_3 saturation, param_4 value) into three bytes at param_1 using the standard sector f… |
| `0047f14e` | RgbToHsv | likely | Treats param_1[0..2] as three channels, writes the maximum channel to *param_4, computes saturation as (max-min)*510+max divided by 2*max (integer result 0..25… |
| `0047f26e` | HueAngleToRgbTrig | likely | Converts an angle (param_1, compared against 0, 2*pi/3 and 4*pi/3) plus saturation param_2 and value param_3 into three channel doubles, using a CRT trig routi… |
| `0047f3e6` | ColorInvertSubtractAlpha | likely | Writes three bytes derived per channel as 255-(channel+param_5) when channel+param_5 is below 0xff and 0 otherwise, in B,G,R store order. |
| `0047f446` | ComplementChannelsAndExtractMin | likely | Writes 255 minus each of param_1[2], param_1[1], and param_1[0] through param_2, param_3 and param_4, writes the minimum of those three complements through par… |
| `0047f4f4` | ColorInvert | likely | Writes 255-c for each of the three channel arguments into param_1 in B,G,R store order (plain inversion). |
| `0047f5d2` | RgbToYiq | likely | Converts the three RGB bytes at param_1 (normalised by the constant 255.0 at 0x01768b08) to NTSC YIQ using the coefficient table at 0x01769058-0x01769098 (0.31… |
| `0047f724` | RgbToYuv | likely | Converts the three RGB bytes at param_1 (normalised by 255.0) to EBU YUV using the coefficient table at 0x017690c0-0x017690e8 (Y = 0.299/0.587/0.114; U/V terms… |
| `0047f89c` | RgbToXyz | likely | Converts the three RGB bytes at param_1 (normalised by 255.0) to CIE XYZ using the sRGB D65 matrix at 0x01769140-0x01769188 (X: 0.412453/0.357580/0.180423, Y: … |
| `0047f96c` | CubePositive | likely | Returns x*x*x when x is greater than or equal to 0 and 0 when x is negative (a clamped cube used by the colour-space conversions). |
| `0047f988` | LabToXyz | likely | Converts CIE Lab (param_1=L, param_2=a, param_3=b) to XYZ: fY=(L+16)/116, X = cube(fY+a/500)*0.95, Y = cube(fY), Z = cube(fY-b/200)*1.09 using the constants 11… |
| `0047f9f2` | LabTransfer | likely | Transfer function f(t) of the Lab/Luv conversions: for t below the threshold 0.008856 it returns t*7.787037+0.137931, otherwise it calls CRL routine FUN_00d846… |
| `0047fb3b` | XyzToLuv | likely | Converts XYZ to CIE Luv: denominator X+15Y+3Z, u' = 4X/den, v' = 9Y/den, then L from the Lab transfer of Y/100 and u = 13L(u'-un), v = 13L(v'-vn) using the sam… |
| `0047fe82` | AllocStampedBlock8 | likely | Eighth copy of the stamped-allocation helper (engine allocator plus magic 0x56471e89/0x9fe1234a at +8/+12). |
| `0047ff27` | LoadCursorHotspotAndRect | likely | When FUN_004c6e64 accepts the property object param_2, reads the "HotSpot" and "CursorRect" properties into this+0x18 and this+0x20 through FUN_004eb3c2 and FU… |
| `0047ff8f` | LoadCursorHotspotAndRect2 | likely | Second variant of GCursorLoadProperties using the accessor pair FUN_004eb454/FUN_004eb7f3 to read "HotSpot" and "CursorRect" into this+0x18 and this+0x20. |
| `004801fe` | AllocStampedBlock | likely | Stamped-allocation helper: allocates the requested size with the engine small-block allocator and writes the object-header magic 0x56471e89 / 0x9fe1234a at +8/… |
| `0048031b` | AllocStampedBlock | likely | Stamped-allocation helper: allocates the requested size with the engine small-block allocator and writes the object-header magic 0x56471e89 / 0x9fe1234a at +8/… |
| `0048044a` | AllocStampedBlock | likely | Stamped-allocation helper: allocates the requested size with the engine small-block allocator and writes the object-header magic 0x56471e89 / 0x9fe1234a at +8/… |
| `0048054f` | AllocStampedBlock | likely | Stamped-allocation helper: allocates the requested size with the engine small-block allocator and writes the object-header magic 0x56471e89 / 0x9fe1234a at +8/… |
| `004806a2` | AllocStampedBlock | likely | Stamped-allocation helper: allocates the requested size with the engine small-block allocator and writes the object-header magic 0x56471e89 / 0x9fe1234a at +8/… |
| `004807dd` | AllocStampedBlock | likely | Stamped-allocation helper: allocates the requested size with the engine small-block allocator and writes the object-header magic 0x56471e89 / 0x9fe1234a at +8/… |
| `00480911` | AllocStampedBlock | likely | Stamped-allocation helper: allocates the requested size with the engine small-block allocator and writes the object-header magic 0x56471e89 / 0x9fe1234a at +8/… |
| `004809a9` | AllocStampedBlock | likely | Stamped-allocation helper: allocates the requested size with the engine small-block allocator and writes the object-header magic 0x56471e89 / 0x9fe1234a at +8/… |
| `00480a41` | AllocStampedBlock | likely | Stamped-allocation helper: allocates the requested size with the engine small-block allocator and writes the object-header magic 0x56471e89 / 0x9fe1234a at +8/… |
| `00480ab3` | ReplaceGlobalInterfacePointer | likely | If DAT_01bc673c is non-null, calls its virtual slot 0x10 with argument 0, then stores param_1 in DAT_01bc673c. |
| `00480b91` | ReadSlot1COutputOnZero | likely | Initializes a local output pointer to param_1, calls virtual slot 0x1c with its address, returns the possibly updated pointer only when the slot returns zero, … |
| `00480cce` | GetGlyphIndexByCode | likely | Returns the 16-bit value for character code param_2 from the table at this+8 (offset this+0xc-this+0x1c + param_2, bounds-checked against the table count), fal… |
| `00480d2f` | GetGlyphIndexByCode2 | likely | Variant of the glyph lookup: index = this+0xc + param_2 and the provider call uses this+0x1c + param_2, with the same 0xffff failure return. |
| `00480f0b` | ReleaseFourBuffers | likely | With ECX=this, invokes the two-temporary cleanup helper on this+0x68 and then this+8; each helper releases two adjacent counted-buffer fields, covering offsets… |
| `00481097` | PrepareStateBlockAndSetFlags | likely | Saves the current text state (seven dwords) through param_7, then re-configures the object: sets flag 0x20000, replaces field +0xc with 0xfff4d (or masks it wi… |
| `0048110f` | CopyStateBlockAndClearFlag | likely | Copies seven state values from param_2 into this object (+8,+0xc,+0x18,+0x1c,+0x20,+0x24,+0x28) and clears the dirty byte at +0x2c. |
| `004811e3` | MaskStateAndCallVirtualSlot | likely | Uses the state object at this+0x50: when its flags contain 0x20000 the requested mask param_4 is ANDed with its field +0xc, then virtual slot 0xc(this+0x74,par… |
| `00481224` | CheckCachedStateAndApply | likely | Ensures the text-draw object is set up for the requested state mask param_2: uses the state object at param_1[0x14], masks with its flags when 0x20000 is set, … |
| `0048130f` | AllocStampedBlock18 | likely | Stamped-allocation helper: allocates the requested size with the engine allocator and writes the object-header magic 0x56471e89 / 0x9fe1234a at +8/+12. |
| `0048185c` | CopyRecordWithTwoEmbeddedLists | likely | Prepares two embedded list members with FUN_004e8d36, copies the observed scalar fields from offsets +8 through +0x5e, and returns param_1. |
| `00481930` | ScalarDeletingDtorB | likely | Scalar deleting destructor: runs the teardown helper FUN_004508da and frees the object through the CRT free when bit 0 of the flag is set. |
| `00481b1a` | CopyWideString16 | likely | Copies 16-bit code units from param_2 to param_1 until (and including) the terminating zero word. |
| `00481b66` | FixedPointPairToFloat | likely | Converts a 16-bit pair (fraction at *param_1, signed integer at param_1[1]) to float as integer + fraction*scale, where the scale constant lives at 0x01769588. |
| `00481cd8` | AllocStampedBlockOffset2C | likely | Allocates the requested size with the engine allocator and stamps the object-header magic 0x56471e89 / 0x9fe1234a at the larger offsets +0x2c and +0x30 (a big-… |
| `00481de7` | AllocStampedBlock | likely | Stamped-allocation helper: allocates the requested size with the engine small-block allocator and writes the object-header magic 0x56471e89 / 0x9fe1234a at +8/… |
| `00481eb2` | AllocStampedBlock | likely | Stamped-allocation helper: allocates the requested size with the engine small-block allocator and writes the object-header magic 0x56471e89 / 0x9fe1234a at +8/… |
| `00481f57` | CompareEmbeddedBlocksAndKeys | likely | Orders records by the leading two dwords, compares the two embedded blocks through FUN_0047e058 in both directions, then compares trailing dwords at +0xc8 and … |
| `0048205e` | AllocStampedBlock | likely | Stamped-allocation helper: allocates the requested size with the engine small-block allocator and writes the object-header magic 0x56471e89 / 0x9fe1234a at +8/… |
| `004820d0` | AssignStateBlockAndRebindInterface | likely | Copies a text-device state block from param_2 (fields +0xc..+0x24 relative to param_2+0x10, the u16 at +0x38 and the embedded list at +0x18) and rebinds the de… |
| `00482256` | MeasureSpanAndCallDeviceSlotD8 | likely | Measures the run through FUN_004dcb1a(param_2) and, when this+0x1c lies beyond the extent at this+0x30 and a device is bound at this+0x38, calls the device's v… |
| `004822a9` | MeasureAlternateSpanAndCallDeviceSlotD8 | likely | Variant of DrawTextRunA using FUN_004dcbda for the measurement and comparing this+0x1c against this+0x2c before calling the device's virtual slot 0xd8. |
| `00482347` | AllocStampedBlock | likely | Stamped-allocation helper: allocates the requested size with the engine small-block allocator and writes the object-header magic 0x56471e89 / 0x9fe1234a at +8/… |
| `004823b9` | LoadStringProperties | likely | After validating param_2 and the property object at this+0x18, obtains that object's slot-0x24 receiver via slot 0x30, reads property "String" into this+0x34 t… |
| `00482a8b` | ReleaseAssignInterface | likely | If *param_1 holds an interface, calls its vtable slot +0x10 with 0 (Release), then stores param_2 into *param_1. |
| `00482b99` | ReleaseAssignInterface2 | likely | Identical to 00482A8B: releases the interface currently stored at *param_1 through vtable slot +0x10 and then stores param_2 there. |
| `00482c03` | CtorVtable1768EA0 | likely | Calls FUN_004c9e10 (base constructor) and installs &PTR_LAB_01768EA0 as the object's first dword; the pointed-to .data record holds the factory 0047E640 follow… |
| `00482c15` | CtorVtable1768EB4 | likely | Calls FUN_004c9e10 and installs &PTR_LAB_01768EB4; the record behind that pointer holds the factory 0047EA94 and the class name "GBitmapIEppm". |
| `00482c27` | CtorVtable1768EC8 | likely | Calls FUN_004c9e10 and installs &PTR_LAB_01768EC8; the record holds the factory 0047EBA4 and the class name "GBitmapIEtga". |
| `00482c39` | CtorVtable1769590 | likely | Calls FUN_004c9e10 and installs &PTR_LAB_01769590; the record holds the factory 00481C95 and the class name "GFontDeviceWin32". |
| `00482d39` | CtorVtable1769600 | likely | Calls the base constructor FUN_004c9e10 and then stores &PTR_LAB_01769600 as the object's first dword; that .rdata table starts with the deleting-destructor co… |
| `00483537` | LessThanPredicate | likely | Calls FUN_00481f57 with both arguments and returns whether its result is nonzero. |
| `00484297` | AssignDwordA | likely | Copies the dword pointed to by param_2 into the location pointed to by param_1 and returns nothing else. |
| `004842a4` | AssignDwordB | likely | Second emission of the single-dword copy stub (*param_1 = *param_2). |
| `004842b1` | AssignDwordC | likely | Third emission of the single-dword copy stub (*param_1 = *param_2). |
| `004844ab` | AbsCross2D | likely | Computes the 2D cross product of (p2-p1) and (p3-p2) from three point records and returns its absolute value. |
| `00484f9e` | CopyRecord4104 | likely | Element copy constructor for the 4104-byte record: copies 0x80 sub-records of 0x20 bytes with FUN_0047de1f (their own copy routine) and then the two trailing d… |
| `0048547d` | CopyRecords16Backward | likely | Backward copy of 16-byte records (walking from the end): copies the dwords at -8/-4 and, when the source record's +4 field points to itself, makes the copy sel… |
| `004856d5` | BuildRectExtentFromPointPairs | likely | Computes width as param_2[2]-param_2[0]+1 and height as param_2[3]-param_2[1]+1, then passes those extents and the remaining arguments to FUN_004dd7a0. It does… |
| `00485bc4` | CtorWithInterfaceAt18_2 | likely | Second emission of the GObjectBase-derived constructor that installs the vtable block &PTR_FUN_017697A0 / &PTR_LAB_0176979C / &PTR_LAB_01769774 and stores a re… |
| `00485ccc` | CtorVtables17698D8 | likely | Runs the base constructor 00485AB4 and then overwrites the vtable pointers with &PTR_FUN_017698E0 / &PTR_LAB_017698D8 / &PTR_LAB_017698AC. |
| `00485d66` | CtorVtables1769A14 | likely | Runs the base constructor 00485BC4 and then installs &PTR_FUN_01769A18 / &PTR_LAB_01769A14 / &PTR_LAB_017699EC. |
| `00485e00` | CopyNineDwords | likely | Copies nine consecutive dwords (offset 0 through +0x20) from the source record to the destination record. |
| `00489b9f` | HelperObjectDtor | likely | Installs two vtable pointers, conditionally releases the member at +0x1c through 0047d41f, then tail-jumps to 00526562. It has callers that use it during clean… |
| `00489d9f` | SetMemberAndInvokeVtable64 | likely | If param_2 differs from the member at this+0x1c, AddRefs the new pointer through slot +0x0c, Releases the old pointer through slot +0x10 with argument 0, store… |
| `00489e38` | StoreEightFields | likely | Copies eight dwords into this+0x14..+0x34: param_2 to +0x14; param_3[0..1] to +0x1c/+0x20; param_5 to +0x24; param_4[0..1] to +0x28/+0x2c; and param_6/param_7 … |
| `00489ffb` | SharedMemberDeletingDtor | likely | When this+0x10 is non-null, Releases it through virtual slot +0x10 with argument 0. It frees this through the _free thunk only when param_2 bit 0 is set; other… |
| `0048a020` | CopyTwelveDwords | likely | Copies a 12-dword structure verbatim: param_1[k] = param_2[k] for k = 0..11, pairing the copy of each dword in the same order (format/width at 0..2, bytes-per-… |
| `0048a071` | CopyWithSharedMember | likely | Copies four dwords from param_2, calls 00481a01 with ECX=this+0x14 and source param_2+0x14, then handles the shared member at +0x10. If the source member is nu… |
| `0048a191` | SetRefCountedPointerAt80 | likely | Refcounted setter for the member at this+0x80: AddRefs the incoming pointer param_2 when non-null (virtual slot +0xc), Releases the current member through virt… |
| `0048a33e` | AllocateOrResetIndexedRecordBlock | likely | Selects the pointer-table entry at this+0x88+4*param_2. If it is null, allocates 0x2200 bytes, stores the result and zeroes the block. If the entry is non-null… |
| `0048a3aa` | ReleaseIndexedRecordBlock | likely | For a non-null table entry at this+0x88+4*param_2, walks the 0x2200-byte block in 0x44-byte steps and Releases each non-null member at record offset +0x10 thro… |
| `0048c619` | EncodeBitmapRowsToWriter | likely | Emits packed bitmap rows to a writer: reads dimensions and format flags from param_1, fetches source rows through param_2 slot +0x198, packs rows with FUN_0048… |
| `0048cb35` | ScaleColorValueByPercent | likely | Reads the receiver's 3-byte colour as hue/saturation/value through FUN_0047f14e (hue in degrees 0-360, saturation and value as bytes), scales the value compone… |
| `0048cba0` | ScaleColorValueByInversePercent | likely | Mirror of FUN_0048cb35: for param_1 below 100 it first calls FUN_0048cb35(10000/param_1), then reads the receiver's hue/saturation/value bytes with FUN_0047f14… |
| `0048cbf0` | PackClampedBytesFromVarargs | likely | Variadic packer: pulls three numeric arguments through the FUN_00d84680/FUN_00d83c90 pair, clamps each into 0..0xFF and stores them as the three bytes *(param_… |
| `0048ce5b` | StoreConvertedTripleAsBytes | likely | Converts three float components by calling FUN_0047f26e with param_2..4, then converts each of the three returned doubles through FUN_00d83cc6 and stores them … |
| `0048cfab` | SetColorFromLab | likely | Applies a CIELab triple as the receiver's byte colour: converts param_1..3 through FUN_0047f988 (CIELab to XYZ, three doubles by pointer) and passes them to FU… |
| `0048d00b` | GetColorAsLabFloats | likely | Reads the receiver's three colour bytes through FUN_0047f89c (sRGB-to-XYZ matrix, three floats by pointer), converts the XYZ result with FUN_0047fa1f (XYZ to C… |
| `0048d069` | SetColorFromLuv | likely | Applies a CIELuv triple as the receiver's byte colour: converts param_1..3 through FUN_0047fa88 (CIELuv to XYZ) and passes them to FUN_0047f7ee, which converts… |
| `0048d0cd` | GetColorAsLuvFloats | likely | Reads the receiver's three colour bytes through FUN_0047f89c (sRGB-to-XYZ), converts the result with FUN_0047fb3b and stores the three floats through param_1, … |
| `0048d2db` | LookupColorTableEntry | likely | Binary-searches the caller's table (param_2, param_3 entries of 8 bytes) with the CRT _bsearch and the comparison callback LAB_0047fbe5, building the 8-byte se… |
| `0048d437` | ClampThreeSelectedResultsIntoStride4 | likely | For each requested record, calls FUN_00d83c90 six times to convert selected x87 values, uses the second, fourth and sixth converted results, clamps them to 0..… |
| `0048d90d` | InitStateBlock4C | likely | Resets the fixed 0x4c-byte state block: +0x3C and +0x40 to 1.0f, +0x34 to 0x7FFFFFFF, +0x38 to 0x80000001, and +8, +0x1C, +0x30, +0x44 and +0x48 to zero; makes… |
| `0048d937` | ComputePointBounds | likely | Computes the bounding box of a 2D point list: count at param_1+8 and points addressed through the page-pointer table at param_1+0x14 (256 points per 4-byte pag… |
| `0048db62` | AnyOfFields30_38_3C | likely | Returns 1 when any of the receiver's fields +0x30, +0x38 or +0x3C is non-zero, otherwise 0. |
| `0048dba8` | QuerySlot10COrCopyBlock | likely | Reads dword field +0x30 (param_1[0xc]) and +0x14 (param_1[5]). If +0x30 is zero and +0x14 nonnegative, a set 0x40000000 bit constructs 92-byte scratch state, c… |
| `0048dc69` | InvokeField30Slot240 | likely | Guards on the object at +0x30 supporting the interface id from FUN_004cd10a (bound through FUN_0047c09e) and then forwards param_2/param_3 to that object's vir… |
| `0048dcb2` | InvokeField30Slot248 | likely | Same guarded forwarding pattern as FUN_0048dc69 but through the +0x30 field object's virtual slot +0x248 with a single argument param_2, returning the result o… |
| `0048ddd7` | QueryThreeOutputsBySlot10COr110 | likely | Reads dword field +0x30 (param_1[0xc]) and +0x14 (param_1[5]). If +0x30 is zero and +0x14 nonnegative, a clear 0x40000000 bit returns 0 without writing outputs… |
| `0048f347` | JumpTo004c97db | likely | The 9-byte body tears down its frame and jumps to 004c97db with incoming arguments/registers unchanged; it does not make a local call and return. |
| `0048f38a` | CallHelper4c97dbWhenField48Set | likely | Returns zero when the dword at +0x48 is zero. Otherwise calls FUN_004c97db with fields +0x2c and +0x44 and the three incoming arguments, then returns that help… |
| `004916ac` | ValidateAndAttachRangeBaseMinus4 | likely | Uses the object at param_1-4 to validate name param_2, start param_4 and length param_5 through slot +0x6c. On validation success it attaches {start=param_4,en… |
| `00491cd7` | RefreshSlotRun | likely | Reads the 16-bit id run at *param_3 (bounded by param_4 unless param_4 == -1), restricts ids to param_1[0x2f..0x30], and gets each glyph-slot page through FUN_… |
| `00494731` | AssignBitmapFromSource | likely | Calls FUN_004c6e7c, whose body returns 1, so this call does not reject the source. Releases the current view through this vtable slot +0x128 when present, stor… |
| `004948e6` | DecodePixelValueToArgb | likely | Pixel-format decode into 32-bit ARGB: switches on the low 24 bits of the format field *param_2 and converts the packed source value param_3: 4-4-4-4 formats ex… |
| `00494ad7` | DecodePixelFromBuffer | likely | Reads and converts one packed pixel from a byte buffer: switches on the low 24 bits of the format field *param_2, extracts the pixel bits from the byte(s) at p… |
| `00494de3` | DecodePixelFromBufferFast | likely | Same packed-pixel decode as 00494ad7 with an additional fast path: when the surface's second size word param_2[5] is greater than 1 only the 16/32-bit codes 0x… |
| `004960da` | GetPixelConverted | likely | Thiscall form of the packed-pixel decode: the surface descriptor is this (format = *this & 0xffffff, palette object at +0x2c), the packed value is read from pa… |
| `004c41f6` | GObjectIteratorSlot88Call | likely | Thiscall (this in ECX) with three stack arguments: builds a temporary GObjectIterator (vftable GObjectIterator::vftable, two zeroed dwords), calls virtual slot… |
| `004c4d05` | ThreadPauseAcquire | likely | Fastcall (this in ECX): when bit 0 of the dword at this+0x148 is set, calls SuspendThread on the handle at this+0x14c; if that succeeds it increments the count… |
| `004c4d3f` | ThreadPauseRelease | likely | Fastcall (this in ECX): when bit 0 of this+0x148 is set, InterlockedExchangeAdd(-1) on the count at this+0x144; when the old count was 1 it calls ResumeThread … |
| `004c4d82` | ThreadHandleClose | likely | Fastcall (this in ECX): when the handle at this+0x14c is nonzero, calls CloseHandle on it and zeroes the field. |
| `004c6281` | NameRegistryPush | likely | Thiscall (node in ECX) with name pointer param_2, value param_3 and a flag byte param_4: stores the name at node+4, the value at node+8 and the flag byte at no… |
| `004c6c1c` | IntrusiveListBaseCtor_017635a8 | likely | Fastcall ctor (this in ECX): stores PTR_FUN_017635a8 at +0 and makes two self-linked list sentinels, one at +4 (its first dword points to itself) and one at +0… |
| `004e6771` | DtorBodyVtbl017635a8 | likely | Destructor body: sets the vtable to PTR_FUN_017635a8, calls 004d9806, unlinks two list nodes (each node's links at +4 and +8 are rewritten to point at each oth… |
| `004e6941` | CtrlVirtualSlot14Shared | likely | Virtual slot +0x38 target shared by twelve control classes: when this+0x10 holds an object it tail-calls that object's vtable +0x38; otherwise it calls 004d83c… |
| `004e699d` | CtrlVirtualSlot16Shared | likely | Virtual slot +0x40 target shared by twelve control classes: calls the receiver's vtable slot +4 with (&local, 0) to fill a local, looks an object up through 00… |
| `004e702d` | ResolveObjectPathSegments | likely | Thiscall (param_1 = root object, param_2 = path IString holder, param_3 = flags): walks the path characters, starting at the root (climbing the parent link at … |
| `004ea48c` | PackedTimeSetMilliseconds | likely | Writes a millisecond value into the packed time word *param_1 at bits 17 and up. Below 1000 it stores param_2 << 17 over the whole word. Otherwise it keeps mil… |
| `004ea936` | DayCountToCivilDate | likely | Converts a day count (param_2, valid below 0x231519) into a date record at param_1: day at +0 (byte), month at +4, year at +8. It uses 1461-day four-year cycle… |
| `004ea9d6` | TwoDwordLifecycle | likely | Mode switch for an 8-byte two-dword record: mode 1 copies two dwords from param_3 into param_2 when it is nonnull, mode 2 allocates 8 bytes through operator ne… |
| `004eb1b8` | TwoDwordLifecycle | likely | Mode switch for an 8-byte two-dword record, same body as 004ea9d6: mode 1 copies two dwords from param_3 into param_2 when it is nonnull, mode 2 allocates 8 by… |
| `004eb354` | TwoDwordLifecycle | likely | Mode switch for an 8-byte two-dword record, same body as 004ea9d6 and 004eb1b8: mode 1 copies two dwords from param_3 into param_2 when nonnull, mode 2 allocat… |
| `004eb4f0` | TwoDwordLifecycle | likely | Mode switch for an 8-byte two-dword record, same body as 004ea9d6, 004eb1b8, 004eb354 and 004ebaa4: mode 1 copies two dwords from param_3 into param_2 when non… |
| `004eb68c` | FourDwordLifecycle | likely | Mode switch for a 16-byte four-dword record: mode 1 copies four dwords from param_3 into param_2 when nonnull, mode 2 allocates 0x10 bytes through operator new… |
| `004ebaa4` | TwoDwordLifecycle | likely | Mode switch for an 8-byte two-dword record, same body as 004ea9d6, 004eb1b8, 004eb354 and 004eb4f0: mode 1 copies two dwords from param_3 into param_2 when non… |
| `004ece4e` | Wrapper_004d09a2 | likely | Forwarding stub (22 bytes): calls 004d09a2 with ECX = the stack argument param_2 and two pushed arguments, the dword at *(this+4) and this; returns param_2. |
| `004ecedc` | Wrapper_004cf21b | likely | Forwarding stub (22 bytes) with the same shape as Wrapper_004d09a2: calls 004cf21b with the dword at *(this+4) and this, returns param_2. |
| `004ecf06` | Wrapper_004d01b1 | likely | Forwarding stub (22 bytes) with the same shape: calls 004d01b1 with the dword at *(this+4) and this, returns param_2. |
| `004ed020` | Wrapper_004d0559 | likely | Forwarding stub (22 bytes): calls 004d0559 with the dword at *(this+4) and this as arguments and returns param_2. |
| `004ed1c3` | Wrapper_004d0be3 | likely | Forwarding stub (22 bytes): calls 004d0be3 with the dword at *(this+4) and this as arguments and returns param_2. |
| `004ed1ed` | Wrapper_004d0a25 | likely | Forwarding stub (22 bytes): calls 004d0a25 with the dword at *(this+4) and this as arguments and returns param_2. |
| `004ed217` | Wrapper_004d0b0e | likely | Forwarding stub (22 bytes): calls 004d0b0e with the dword at *(this+4) and this as arguments and returns param_2. |
| `004ed456` | Wrapper_004de79c | likely | Forwarding stub (20 bytes, ret 4): calls 004de79c with ECX = the stack argument param_2, and two stack arguments *(this+4) and this; returns param_2. |
| `004ed46a` | Wrapper_004de79c | likely | Forwarding stub (20 bytes, ret 4): calls 004de79c with ECX = the stack argument param_2, and two stack arguments *(this+8) and this; returns param_2. |
| `004ed47e` | Wrapper_004922b1 | likely | Forwarding stub (20 bytes, ret 4): calls 004922b1 with ECX = the stack argument param_2, and two stack arguments *(this+8) and this; returns param_2. |
| `004ed4b4` | Wrapper_004cfcaa | likely | Forwarding stub (24 bytes, ret 4): calls 004cfcaa with ECX = the stack argument param_2 and the stack argument this+0xc plus this+0x10; returns param_2. |
| `004ed4e3` | Wrapper_004e025d | likely | Forwarding stub (20 bytes, ret 4): calls 004e025d with ECX = the stack argument param_2, and two stack arguments *(this+4) and this; returns param_2. |
| `004ed4f7` | Wrapper_004e025d | likely | Forwarding stub (20 bytes, ret 4): calls 004e025d with ECX = the stack argument param_2, and two stack arguments *(this+8) and this; returns param_2. |
| `004ed50b` | Wrapper_0040b244 | likely | Forwarding stub (20 bytes, ret 4): calls 0040b244 with ECX = the stack argument param_2 and two stack arguments *(this+0xc) and this; returns param_2. |
| `004ed51f` | Wrapper_0040b244 | likely | Forwarding stub (24 bytes, ret 4): calls 0040b244 with ECX = the stack argument param_2 and two stack arguments (this+0x10 plus this+0xc) and this; returns par… |
| `004ed571` | ClearBlockRecord | likely | Clears a three-dword record at param_1 (fastcall): frees the block at +4 through 004c6a61 when nonzero, then zeroes +4, +8 and +0xc. Returns nothing. |
| `004ed593` | ClearBlockRecord | likely | Clears a three-dword record at param_1 (fastcall): frees the block at +4 through 004c6a61 when nonzero, then zeroes +4, +8 and +0xc. Returns nothing. |
| `004ed5fe` | Wrapper_004d10f4 | likely | Forwarding stub (12 bytes): saves ECX in esi, calls 004d10f4 with ECX = this, and returns this in EAX. |
| `004ed60a` | Wrapper_004d1093 | likely | Forwarding stub (12 bytes): saves ECX in esi, calls 004d1093 with ECX = this, and returns this in EAX. |
| `004ed636` | Wrapper_004d11a4 | likely | Forwarding stub (12 bytes): saves ECX in esi, calls 004d11a4 with ECX = this, and returns this in EAX. |
| `004ed642` | Wrapper_004d1143 | likely | Forwarding stub (12 bytes): saves ECX in esi, calls 004d1143 with ECX = this, and returns this in EAX. |
| `004ed664` | Wrapper_004cff01 | likely | Forwarding stub (22 bytes, ret 4): calls 004cff01 with ECX = the stack argument param_2, and two stack arguments *(this+4) and this; returns param_2. |
| `004ed758` | Wrapper_004df762 | likely | Forwarding stub (12 bytes): saves ECX in esi, calls 004df762 with ECX = this, and returns this in EAX. |
| `004ed7a4` | Wrapper_004e03d9 | likely | Forwarding stub (20 bytes, ret 4): calls 004e03d9 with ECX = the stack argument param_2, and two stack arguments *(this+4) and this; returns param_2. |
| `004ed7cc` | Wrapper_004e03d9 | likely | Forwarding stub (20 bytes, ret 4): calls 004e03d9 with ECX = the stack argument param_2, and two stack arguments *(this+8) and this; returns param_2. |
| `004ed7e0` | Wrapper_004e0430 | likely | Forwarding stub (20 bytes, ret 4): calls 004e0430 with ECX = the stack argument param_2, and two stack arguments *(this+4) and this; returns param_2. |
| `004ed7f4` | Wrapper_004e0430 | likely | Forwarding stub (20 bytes, ret 4): calls 004e0430 with ECX = the stack argument param_2, and two stack arguments *(this+8) and this; returns param_2. |
| `004ed80b` | ClearBlockRecordCrtFree | likely | Clears a three-dword record at param_1 (fastcall): frees the block at +4 with _free when nonzero, then zeroes +4, +8 and +0xc. Called from a CATCH funclet, not… |
| `004ed845` | Wrapper_004dfccc | likely | Forwarding stub (20 bytes, ret 4): calls 004dfccc with ECX = the stack argument param_2, and two stack arguments *(this+4) and this; returns param_2. |
| `004ed859` | Wrapper_004dfccc | likely | Forwarding stub (20 bytes, ret 4): calls 004dfccc with ECX = the stack argument param_2, and two stack arguments *(this+8) and this; returns param_2. |
| `004ed86d` | Wrapper_004d03fd | likely | Forwarding stub (22 bytes, ret 4): calls 004d03fd with ECX = the stack argument param_2, and two stack arguments (the dword pointed to by this+4) and this; ret… |
| `004ed961` | Wrapper_004d157f | likely | Forwarding stub (12 bytes): saves ECX in esi, calls 004d157f with ECX = this, and returns this in EAX. |
| `004ed96d` | Wrapper_004d151e | likely | Forwarding stub (12 bytes): saves ECX in esi, calls 004d151e with ECX = this, and returns this in EAX. |
| `004eda0a` | Wrapper_004d17a8 | likely | Forwarding stub (12 bytes): saves ECX in esi, calls 004d17a8 with ECX = this, and returns this in EAX. |
| `004eda16` | Wrapper_004d09f0 | likely | Forwarding stub (22 bytes, ret 4): calls 004d09f0 with ECX = the stack argument param_2, and two stack arguments *(this+4) dereferenced once and this; returns … |
| `004eda7d` | Wrapper_004d1832 | likely | Forwarding stub (12 bytes): saves ECX in esi, calls 004d1832 with ECX = this, and returns this in EAX. |
| `004eda9f` | Wrapper_004d1893 | likely | Forwarding stub (12 bytes): saves ECX in esi, calls 004d1893 with ECX = this, and returns this in EAX. |
| `004edb2c` | Wrapper_004d19c3 | likely | Forwarding stub (12 bytes): calls 004d19c3 with ECX = this and returns this in EAX. |
| `004edb4e` | Wrapper_004e0194 | likely | Forwarding stub (20 bytes, ret 4): calls 004e0194 with ECX = the stack argument param_2, and two stack arguments *(this+4) and this; returns param_2. |
| `004edb62` | Wrapper_004e0194 | likely | Forwarding stub (20 bytes, ret 4): calls 004e0194 with ECX = the stack argument param_2, and two stack arguments *(this+8) and this; returns param_2. |
| `004edbbb` | Wrapper_004e0203 | likely | Forwarding stub (12 bytes): calls 004e0203 with ECX = this, and returns this in EAX. |
| `004edf68` | Wrapper_004d21f9 | likely | Forwarding stub (12 bytes): calls 004d21f9 with ECX = this, and returns this in EAX. |
| `004edf74` | Wrapper_004d2198 | likely | Forwarding stub (12 bytes): calls 004d2198 with ECX = this, and returns this in EAX. |
| `004edf80` | Wrapper_004e06d1 | likely | Forwarding stub (12 bytes): calls 004e06d1 with ECX = this, and returns this in EAX. |
| `004edf8c` | Wrapper_004d2248 | likely | Forwarding stub (12 bytes): calls 004d2248 with ECX = this, and returns this in EAX. |
| `004edf98` | Wrapper_004e073e | likely | Forwarding stub (12 bytes): calls 004e073e with ECX = this, and returns this in EAX. |
| `005454b4` | QueryRegistryEntry | likely | Lazily creates the global registry DAT_01cc47b4 if needed, looks param_1 up in it through registry vtable +0x18 into a local, and when the entry exists returns… |
| `00545c26` | VisitMatchingClassMembers | likely | Resolves the key DAT_01ccb47c in the global registry and iterates the two maps at the entry's +0x18/+0x1c. For each member it builds a wide string (FUN_004f4b5… |
| `00545f04` | UnregisterClassIds | likely | Thiscall on a class-desc object: does nothing (returns 0) when its 16-bit id at +0x18 is 0. Otherwise it resolves DAT_01ccb47c in the global registry, iterates… |
| `00546012` | CreateClassInstanceByName | likely | Looks param_1 up in the global registry DAT_01cc47b4, converts the entry into a GString key (FUN_004c6d34/FUN_004cd10a) and asks param_2's vtable +0x10c to cre… |
| `005460ad` | DestroyClassInstance | likely | Looks param_1 up in the global registry DAT_01cc47b4 and resolves the instance through the entry's vtable +8. If an instance exists it calls +0x110 on the obje… |
| `005462ca` | FlushPropertyIdRegistrations | likely | Takes the global list head DAT_01ccbe58 (its address obtained with FUN_004c886d), stores 0 into it to detach the whole list, then under the lock helpers FUN_00… |
| `00547890` | ApplyClassInterfaceByClassId | likely | Apply-direction sibling of 00547650. When param_5 is non-null and passes its slot +0x38 predicate, it takes a class id from *param_2 (replaced by DAT_01ccb358 … |
| `0054809d` | FindNamedInterfaceValue | likely | First tries FUN_004c6da3(param_1) and returns that result when it is non-zero. Otherwise it resolves the global DAT_01ccb47c object in the class registry DAT_0… |
| `005482a7` | PropertyMapEntry_GetValueAsDword | likely | Thiscall (ECX=this) returning bool. It requires the low 16 bits of param_2 to equal the property id at this+4, builds a value record from param_2 (FUN_004c8873… |
| `00606a21` | CopyCtorIface017a2e1c | likely | Copy-shaped initialiser of a small interface object: writes PTR_LAB_0175c4cc to +0, copies the byte at source+4 to +4, then overwrites +0 with PTR_LAB_017a2e1c… |
| `00606a9f` | CopyCtorIface017a2e34 | likely | Copy-shaped initialiser of a small interface object: writes PTR_LAB_0175c4cc to +0, copies the byte at source+4 to +4, then installs PTR_LAB_017a2e34 at +0 and… |
| `00a13190` | GuiRegistryReleaseAndClear_00a13190 | likely | If *this is non-null and the singleton at DAT_01be1090 is live, calls virtual slot 0x28 on the held object with argument 0, passes the returned key to FUN_0081… |
| `00a131e0` | GuiRegistryReleaseAndClear_00a131e0 | likely | Identical 46-byte cleanup to 00a13190: if *this and the singleton at DAT_01be1090 are non-null, calls virtual slot 0x28 on the held object, passes the result t… |
| `00a13230` | GuiRegistryReleaseAndClear_00a13230 | likely | Identical 46-byte cleanup to 00a13190 (bytes 578bf98b0f85c97423568b359010be0185f674178b018b50286a00ffd2508bcee8...): vtable slot 0x28 fetch of the key, FUN_008… |
| `00a13280` | GuiRegistryReleaseAndClear_00a13280 | likely | Identical 46-byte cleanup to 00a13190: guarded on *this and DAT_01be1090, key from virtual slot 0x28, FUN_0081ad80 removal, field cleared. |
| `00a132d0` | GuiRegistryReleaseAndClear_00a132d0 | likely | Identical 46-byte cleanup to 00a13190: guarded on *this and DAT_01be1090, key from virtual slot 0x28, FUN_0081ad80 removal, field cleared. |
| `00a13320` | GuiRegistryReleaseAndClear_00a13320 | likely | Identical 46-byte cleanup to 00a13190: guarded on *this and DAT_01be1090, key from virtual slot 0x28, FUN_0081ad80 removal, field cleared. |
| `00a13370` | GuiRegistryReleaseAndClear_00a13370 | likely | Identical 46-byte cleanup to 00a13190: guarded on *this and DAT_01be1090, key from virtual slot 0x28, FUN_0081ad80 removal, field cleared. |
| `00a133c0` | GuiRegistryReleaseAndClear_00a133c0 | likely | Identical 46-byte cleanup to 00a13190: guarded on *this and DAT_01be1090, key from virtual slot 0x28, FUN_0081ad80 removal, field cleared. |
| `00a13410` | GuiRegistryReleaseAndClear_00a13410 | likely | Identical 46-byte cleanup to 00a13190: guarded on *this and DAT_01be1090, key from virtual slot 0x28, FUN_0081ad80 removal, field cleared. |
| `00a13460` | GuiRegistryReleaseAndClear_00a13460 | likely | Identical 46-byte cleanup to 00a13190 (bytes end 8bcee8fb78e0ffc707000000005e5fc3): guarded on *this and DAT_01be1090, key from virtual slot 0x28, FUN_0081ad80… |
| `00a13e60` | GuiRegistryReleaseAndClearAt14 | likely | Exception-protected cleanup: when the object pointer at this+0x14 is non-null and DAT_01be1090 is live, calls virtual slot 0x28 on it with argument 0, passes t… |

### `engine.gfc.draw`

**GDisplayD3D9Impl** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00476d95` | GDisplayD3D9ImplFactory | likely | Factory selector for the 0x34-byte GDisplayD3D9Impl family: mode 0 constructs in place through FUN_00476b89, mode 1 copy-constructs through FUN_00474792, modes… |

**GDrawSurface** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00475ac1` | DrawSurfaceCopyCtor | likely | Copy constructor of the 0x1E4-byte render surface (accepted family: DrawSurfaceFactory 0047afe1 / DrawSurfaceDtor 00479eb9, which use the same 01765140/0176513… |

**(no class)** (98)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00476180` | DrawStateCtor | confirmed | Constructor of a 0x68-byte GInterface-derived draw-state object: installs the GInterface base vtable then the class vtables (01765298/01765294), runs FUN_004c9… |
| `00476505` | InterfaceSubCtor | confirmed | Constructor of a 0x1C-byte GInterface-derived sub-object: installs the GInterface base vtable, runs the base initialiser FUN_0045bcb3 on the adjusted pointer, … |
| `0047658c` | InterfaceSubDtor | confirmed | Destructor body of the 0x1C-byte GInterface-derived sub-object: destroys the contained member through FUN_00473d03, then rewrites the secondary vtable slot (PT… |
| `004765cc` | WindowCursorHelperCtor | confirmed | Constructor of the 0x1C-byte GInterface-derived window/cursor helper: installs the GInterface base then the class vtables (017652D0/017652CC), runs FUN_004c9e1… |
| `00476b89` | D3D9ModuleCtor | confirmed | Constructor of the 0x34-byte Direct3D9 module object: installs the initial vtable 01764854 and then the final class pair (01765040/0176503C) over the GInterfac… |
| `00477877` | DrawStateDtor | confirmed | Destructor body of the 0x68-byte GInterface-derived draw-state object: rewrites the class vtables (01765298/01765294), destroys the record-array holder through… |
| `00824dc0` | CreateDependentDisplay | confirmed | No arguments, EH frame. Calls 004e4e6d with the string GFC_Drivers_Draw_DependentDisplayD3D9, builds a local via 004f61fe, calls 004801cd(local, 1) to create t… |
| `00459393` | SetStaticTag01af5e4c | likely | Stores the static dword from data address 0x1af5e4c into the receiver's first field (a static table/type tag). |
| `0045942b` | GlobalListInsertHead | likely | Pushes the object onto a global doubly-linked list: writes the current head into +0x80, back-links the old head's +0x7c, clears the new node's own +0x7c and ma… |
| `00459448` | GlobalListRemove | likely | Unlinks the object from the global list: patches the neighbours at +0x80/+0x7c (updating the global head DAT_01bc45c4 when it is the first element) and zeroes … |
| `0045a02c` | PackedRecordArrayDtor | likely | Destroys an array of tagged records: builds the descriptor for (param_2, count=*(this+2)) through FUN_00459906 and calls the reference release FUN_00459af8 for… |
| `0045a0d4` | RectStoreA | likely | Copies the five dwords of a rect record from param_3 to param_1, clears the byte pointed to by param_2 and returns 0x14 (the record size). |
| `0045a107` | RectStoreB | likely | Copies the five dwords of a rect record from param_2 to param_1 and returns 0x14 (the record size). |
| `0045a147` | RectFormatConvert | likely | Converts a rect record between storage formats selected by param_3 (0..5): copies the bounding and extra fields, converting the packed 16-bit coordinate varian… |
| `0045a2ad` | RecordQueueFull | likely | Returns 1 when the flag byte at +0xA8 is set or the count at +0xA4 has reached the current index at +0; else 0. |
| `0045a349` | CommandRecordCtor | likely | Initialises a 0x98-byte command record: clears the byte at +0x10, stores param_2 at +0x98, param_5 at +0x9C, zero at +0xA0 and the count from param_3+2 at +0xA… |
| `0045a3c2` | CommandRecordFlush | likely | Flushes the 0x98-byte record: pushes the pair {1, record} onto the block queue at (*(this+0x98)+0x1C) and increments its count; then, when no snapshot exists, … |
| `0045a478` | QueuePushZero | likely | Checks for 4/6/3 units of space through FUN_00459ced and, when they fit, appends a two-dword {0,0} entry to the block queue at (*(this+0x98)+0x1C), incrementin… |
| `0045a666` | EnsureBlockReady | likely | When the count at +8 is zero returns 0; otherwise, when the marker at +0x2C differs from +8 it resynchronises through FUN_00524670(this+0x1C, 0) and stores the… |
| `0045a72e` | InitNode24Buffers | likely | Initialises a 0x24-byte draw-device node: allocates a 0xFC-byte and a 0xFFC-byte buffer through the engine heap helper FUN_004d9207, stores them at +0/+4, reco… |
| `0045a78d` | FreeNode24Buffers | likely | Frees the two node buffers at +0 and +4 through FUN_004c6a61 when non-null. |
| `0045a7ad` | BufferAppendByte | likely | Appends one byte into the buffer at *param_1 at offset +0x10, advancing the position (+0x10) by one and the accumulated cost (+0x14) by param_3. |
| `0045a804` | Node24ScalarDtor | likely | Deleting destructor of the 0x24-byte node: frees the two buffers through FUN_0045a78d and calls _free(param_1) when the flag's low bit is set. |
| `0045a932` | ReleaseDeviceHandles | likely | Releases the two interface pointers at +0x24 and +0x38 (each through its virtual slot +8 with the pointer as argument), then zeroes +0x24/+0x38/+0x44/+0x30 and… |
| `0045a964` | ZeroDeviceFields90 | likely | Zeroes eight consecutive dwords starting at +0x90 of the draw device. |
| `0045a976` | FitPoolSlot | likely | Budgeted pool fitting: walks the slot list at +0x60 (links at node+0xC, sentinel at +0x54) trying to satisfy param_2 units using priority levels from param_4 d… |
| `0045aa50` | StateListCleanupA | likely | While the link at +0x60 differs from the sentinel at +0x54 it calls virtual slot 0 of the node at +0x60 (drain loop; the callee is expected to unlink). |
| `0045aa68` | StateListCleanupB | likely | Walks the ring list from +0x54 to the sentinel at +0x4C calling each node's virtual slot +8 and, when it returns true, the node's slot 0. |
| `0045aa8d` | InitStateRecordList | likely | Walks the ring list at +0x54 calling each node's virtual slot +4; on the first failure it runs the cleanup pass FUN_0045aa68 and returns 0, otherwise 1. |
| `0045ad06` | MapFormatId | likely | Switch mapping of a device format id to the internal index: 1->2, 2->3, 8->4, 9->5, 10->6, 0xB->7, 0xC->0xB, 0xD->8, 0xE->9, 0x10/0x13->10, 0x16->0xD, 0x17->0x… |
| `0045adaa` | ApplyDeviceStateTables | likely | Applies device state from the static tables: for the 14 render states at 0x1af61e4 it calls slot +0xE4(state, value); then for each of two stages applies the s… |
| `0045ae5f` | ApplyStateRecord | likely | Applies a condensed state record: for the count at +2 it calls slot +0xE4 with the (state, value) pairs starting at +8, then for each of two stages applies the… |
| `0045af01` | InitStageRecord | likely | Initialises a draw-state stage record: for formats 0/1 it writes 0 at +0xC and 1, 2, 1 at +0x10..+0x18; otherwise it pulls the three-dword row for param_2 thro… |
| `0045af64` | SetStageMode | likely | Stores param_2 at +0x2C and the constants 7 at +0x30 and 1 at +0x34 of the stage record. |
| `0045afa4` | SetBoundTextureRect | likely | Builds a quad record into this+8 from param_2 through FUN_00459c69 and, when the object returned by the record's virtual slot 0 has this record bound at +0x20,… |
| `0045afcc` | IsTextureCurrent | likely | Returns 1 when the owning record's byte at +0x77 is set and this object's +0x18 field is zero; else 0. |
| `0045b04d` | BaseTeardown1764A18 | likely | Base teardown of the small record class whose vtable is 0x1764a18: when the entry at +0x14 is present it releases it through FUN_0045afdd, then, when the objec… |
| `0045b132` | ResetVtableToPurecallBase | likely | Stores the purecall base table pointer 0x17645fc at +0 (the vtable reset tail of a destructor body) and returns. |
| `0045b21a` | BaseTeardown1764A48 | likely | Second emission of the base teardown: releases the +0x14 entry through FUN_0045afdd and calls the slot-0 cleanup of the object at +0x24 when its +8 word is zer… |
| `0045b23e` | CopyOrKeyPixels | likely | When flag bit 2 is clear it memcpys param_6 rows of param_5 dwords from param_3 (stride param_4) to param_1 (stride param_2); when set, per pixel it copies the… |
| `0045b2cf` | DtorBody1764670 | likely | Destructor body of the record class whose vtable is 0x1764670: installs that table at +0, runs the base teardown FUN_0045afdd and then stores the purecall base… |
| `0045b2fa` | BaseTeardown1764A68 | likely | Releases the interface at +0x2C through its virtual slot +8, clears that field and then runs the base teardown FUN_0045afdd. |
| `0045b316` | BaseTeardown1764A88 | likely | Releases the interface pointer at +0x18 through its virtual slot +8 and clears the field. |
| `0045b32c` | EnsureField18 | likely | When +0x18 is null it creates the object through the device's slot +0x98(0, &+0x18), reads its info through slot +0x30 and sets the byte +0x34 to 0 when the in… |
| `0045b395` | DtorBody1764688 | likely | Destructor body of the 0x1764688 list record: installs that table at +0, releases the interface at +0x14 through its slot +8, unlinks the entry at +8/+0xC from… |
| `0045b3d3` | ReleaseField14 | likely | Releases the interface pointer at +0x14 through its virtual slot +8 and clears the field. |
| `0045b564` | ValidateChildren | likely | When +0x1C is zero it validates the single object at +0x20 through slot +4; otherwise it walks the child array at +0x40 (count at +0x3C, stride 0x14) and valid… |
| `0045b5c4` | ReleaseChildren | likely | When +0x1C is zero it calls the single object at +0x20 through slot 0; otherwise it increments +8, calls slot 0 on every child at +0x40 (count +0x3C, stride 0x… |
| `0046b77b` | AddDrawRecords28Clip | likely | Appends a batch of 28-byte encoded draw records from the source cursor param_2 (element count at +8) through FUN_004655ae and, when the cursor advanced, update… |
| `0046bd72` | AddDrawRecords36Clip | likely | Same control flow as AddDrawRecords28Clip but the per-record encoder is FUN_00465699 (the 0x24-byte record flavour); the tracked rect is expanded from param_3 … |
| `0046be93` | AddDrawRecord36ClipCached | likely | 36-byte draw-record append with the cached-clip logic: validates the tracked rect through FUN_004f7cd4, takes a clip node with FUN_00466ce6 and fills it from p… |
| `0046c86d` | DrawSurfaceScalarDeletingDtor | likely | Runs FUN_0046a6d7 (which reinstalls vtables 0x1764d40/0x1764d3c and destroys the objects members) and, when bit 0 of param_2 is set, frees the object through t… |
| `0046c8cc` | AddDrawRecords28ClipEntry | likely | Stdcall entry (ret 0x18): pushes a literal 0 as the sixth argument plus its five arguments and calls FUN_0046b77b; forwards everything unchanged. |
| `0046c8ec` | AddDrawRecord28ClipCachedEntry | likely | Stdcall entry (ret 0x18) that pushes a literal 0 and its five arguments and calls FUN_0046b89c. |
| `0046c90c` | AddDrawRecord28ClipParamEntry | likely | Stdcall entry (ret 0x18) that pushes a literal 0 and its five arguments and calls FUN_0046bad7. |
| `0046c92c` | AddDrawRecord36ClipEntry | likely | Stdcall entry (ret 0x18) that pushes a literal 0 and its five arguments and calls FUN_0046bc5c. |
| `0046c94c` | AddDrawRecords36ClipEntry | likely | Stdcall entry (ret 0x18) that pushes a literal 0 and its five arguments and calls FUN_0046bd72. |
| `0046c96c` | AddDrawRecord36ClipCachedEntry | likely | Stdcall entry (ret 0x18) that pushes a literal 0 and its five arguments and calls FUN_0046be93. |
| `0046c98c` | AddDrawRecord36ClipParamEntry | likely | Stdcall entry (ret 0x18) that pushes a literal 0 and its five arguments and calls FUN_0046c0ce. |
| `0046fa3f` | DrainQueuedObjectLists | likely | While +0x48 is non-null, reads tail entries through accessors 0046d26d and 0046d175, calls each incoming pointer's virtual slot +0xC, calls each outgoing point… |
| `00472fd2` | DeviceMemberArrayCopy | likely | SEH-framed member copy-construct helper: measures the source object's 4-byte-stride element array (FUN_0045c410 reads begin/end at +4/+8), checks the count thr… |
| `0047305c` | MemberObjectCopyCtor | likely | SEH-framed copy-construct helper for one member sub-object: calls the member base initialiser FUN_0046b141, then copies the source sub-object supplied as the s… |
| `00473112` | DeviceMemberArrayCopy2 | likely | Second emission of the SEH-framed member copy-construct helper: measures the source array with FUN_0045c668, sizes/allocates the destination through FUN_00470e… |
| `004731b8` | DeviceMemberArrayCopy3 | likely | Third emission of the SEH-framed member copy-construct helper: measures the source container with FUN_0045c980, checks/sizes the destination through FUN_00470f… |
| `004740cf` | DrawQueueNodeInit | likely | Initialises a 0x28-byte queue node: zeroes three dword fields, stores the owner context at +0x14 and the argument object at +0, asks that object through virtua… |
| `004743fe` | ScalarDeletingDtorA | likely | Standard MSVC scalar deleting destructor wrapper: runs the class destructor body FUN_00472a34 and, when the low bit of the flag argument is set, frees the obje… |
| `0047441a` | DrawDeviceCopyCtor | likely | Copy constructor of the 0xCC-byte draw device (accepted family: DrawDeviceFactory 00479d94, DrawDeviceDtor 004783f1, vtables 01765078/01765070): installs the c… |
| `0047489a` | MemberObjectCopyCtor2 | likely | Second emission of the SEH-framed member copy-construct helper: initialises the member through FUN_0046af46 and copies the source sub-object supplied on the st… |
| `00474b7c` | MemberObjectCopyCtor3 | likely | Third emission of the SEH-framed member copy-construct helper: initialises the member through FUN_0046b065 and copies the source sub-object from the stack argu… |
| `00474c9e` | MemberObjectCopyCtor4 | likely | Fourth emission of the SEH-framed member copy-construct helper: initialises the member through FUN_0046b09d and copies the source sub-object from the stack thr… |
| `00474dd7` | MemberObjectCopyCtor5 | likely | Fifth emission of the SEH-framed member copy-construct helper: initialises the member through FUN_0046b0e2 and copies the source sub-object through FUN_0046e22… |
| `004752ce` | AdvanceSegmentCursor | likely | Advances a 12-byte segment cursor: computes the new end as the stored length (+0x4) plus param_3, and when growing (param_4 non-null or a current block at +8) … |
| `00475352` | DrawQueueWriteChunk | likely | Appends one length-prefixed variable-length blob to the draw queue's 0xFFFC-byte chunk pages: computes the payload length (FUN_0045a7c8), and when the current … |
| `00475902` | CreateDrawDeviceForFormat | likely | Creates the 0xCC-byte draw-device object for a format index: asks the referenced sub-interface (this+0x24, virtual slot +0x14) to produce/validate the format d… |
| `00475aa2` | TwoVtableObjectCtor34 | likely | Constructor of a 0x34-byte two-vtable object: runs the base initialiser FUN_0042a8b7 with the caller's argument and installs the class vtables (017646A8/017646… |
| `00476081` | AdjustorThunk_004752ce | likely | The x86 body adjusts ECX by +0x90, makes a normal call to FUN_004752ce with the three stack arguments, then returns that call's EAX value with ret 0xC. It is a… |
| `004760a4` | MemberObjectCopyCtor6 | likely | Member copy-construction helper: copy-constructs the destination member from the source member through the MemberObjectCopyCtor2 helper 0047489a (header initia… |
| `004760bf` | ObjectHeaderInit | likely | Initialises a composite object: runs three container-header initialisers (FUN_0046af46, FUN_0046b065, FUN_0046b09d), zeroes the pointer triples at +0x44..+0x4C… |
| `0047624a` | DrawQueueCtor | likely | Initializes a 0xC4-byte object, installs the GInterface base and its vtable pair 017652B4/017652B0, initializes two tree headers at +0x3C/+0x58, and zeroes the… |
| `00476348` | PageObjectScalarDeletingDtor | likely | Standard MSVC scalar deleting destructor wrapper: runs the page-object destructor body 0047543e and, when the low bit of the flag argument is set, frees the ob… |
| `004765b0` | InterfaceSubScalarDeletingDtor | likely | Deleting-destructor wrapper: runs FUN_0047658c (the sub-object destructor) and, when the low bit of the flag argument is set, frees the object through the free… |
| `00476606` | TwoVtableObjectFactory | likely | Class-factory selector (param_1 0..5) for the 0x34-byte two-vtable object built at 00475AA2: selector 0 constructs in place through FUN_0045b611, 1 constructs … |
| `00476691` | CreateDrawState | likely | Creates the 0x68-byte draw-state object from the render object at +0x40: allocates 0x68 bytes, constructs it through FUN_00476180 with the render object's +0x1… |
| `0047673e` | CreateDrawStateChecked | likely | Null-guarded wrapper around the draw-state creation 00476691: returns 0 for a null receiver, otherwise forwards all five arguments and returns the created obje… |
| `00476877` | QueryAllElements | likely | Aggregates a virtual query over the receiver's objects: builds a temporary container (FUN_0046b558); when the single-object field +0x18 is zero it queries the … |
| `00476934` | PageObjectCopyCtor | likely | Copy constructor for the 0xA4-byte page/state object: runs the member assignment FUN_004760a4, copies the 9-dword block from src+0x10 to dst+0x10, copy-constru… |
| `00476982` | CreatePageObject | likely | Unconditionally creates the 0xA4-byte page object and stores it at +4, overwriting any existing pointer (the idempotence check is in the caller 00476d2d, not h… |
| `004769ef` | PageObjectRelease | likely | Release path for a page object: when the page's +4 block exists it runs FUN_0047543e on it and frees it with _free, then clears the owner's back-pointer at +0x… |
| `00476a23` | GetOrLoadCursor | likely | Cursor accessor: ensures the cursor-manager sub-object at +0x44 exists (creating the 0x1C-byte helper through FUN_004765cc), looks the requested cursor key up … |
| `00476c26` | InterfaceSubFactory | likely | Class-factory selector (param_1 0..5) for the 0x1C-byte interface sub-object family: 0 constructs through FUN_004765CC, 1 constructs from the source through FU… |
| `00476cc1` | SharedPtrRelease | likely | Reference-count release for shared page objects: decrements the count at the object's +0, and when it reaches zero removes the object from the owner registry (… |
| `00476e20` | SharedPtrAssign | likely | Shared-pointer assignment with AddRef: when the incoming object is non-null increments its reference count at +0, releases the previously held object through F… |
| `00476e55` | SharedPtrSet | likely | Shared-pointer set without AddRef: releases the previously held object through FUN_00476cc1 when non-null and stores the new pointer. |
| `00476e70` | SharedPtrClear | likely | Shared-pointer clear: when the field is non-null releases it through FUN_00476cc1 and zeroes the field. |
| `00476e88` | SharedPtrCreate | likely | Create-and-store helper for a shared object: obtains an object through FUN_00474178 from the argument, stores it into the target field through the plain setter… |
| `00476eaf` | RecordPayloadClear | likely | Clears one vertex/cache record's payload: releases the held buffer at +8 (FUN_00476e70); when a second argument is supplied and the record's type id is not 8, … |
| `004771b2` | DispatchModeSlotOperations | likely | Returns 0 if the object at +0x44 is absent. With it present, mode 1 calls FUN_004803f7, mirrors +0x120 into +0x1E0, then clears +0x1CC; mode 2 calls FUN_004806… |
| `004772fc` | DrawSurfaceEnsureModeResource | likely | Lazily creates the +0x1D4 resource when flag bits 2/3 of +0x10 are set: builds a request record (type 1 plus fields, through FUN_004597E9 over the +0x124 sub-o… |

### `engine.gfc.font`

**Deep::FontBitmap9GridPrimitive** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d02c30` | FontBitmap9GridPrimitiveCtor | confirmed | Stores the Deep::FontBitmap9GridPrimitive vtable at offset 0, owner pointer at +0x4, copies nine dwords from param_3 into +0x8..+0x28, stores further arguments… |
| `00d03360` | FontBitmap9GridFillQuads | confirmed | Virtual slot [0] of Deep::FontBitmap9GridPrimitive (not a destructor): looks up up to 9 records via 00d00420 (this+0x8 onward), asks param_2 through its slot [… |

**Deep::FontBitmapPrimitive** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d02bb0` | FontBitmapPrimitiveCtor | confirmed | Stores 10 constructor arguments into the object (+0x8, +0xc, +0x10 byte, +0x14, +0x18, +0x1c, +0x20, +0x24, +0x28, +0x2c), stores the Deep::FontBitmapPrimitive… |
| `00d02de0` | FontBitmapPrimitiveFillQuad | confirmed | Virtual slot [0] of Deep::FontBitmapPrimitive (not a destructor): when 00d00420(this+0x8) returns a record, calls slot [0] of param_2 with id 0xc69b89d8 for 6 … |

**Deep::FontPrimitive** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d03a90` | FontPrimitiveBuildGlyphQuads | confirmed | Virtual slot [0] of Deep::FontPrimitive (not a destructor): reads a UTF-16 string from the object at param_1+0xc via 00701290 (length 1..0x1ff), maps each code… |

**(no class)** (80)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00450832` | RegisterGFontDeviceFT2 | confirmed | Passes the literal key ".GFontDeviceFT2" and size 0x18 to FUN_0053da53 to obtain a class record, passes the result through FUN_004ccf27, then publishes the res… |
| `00450a32` | RegisterGFontDescFT2 | confirmed | Builds the literal class name GFontDescFT2, calls FUN_0053da53 with size 0x18 to obtain or create a class descriptor, passes the result through FUN_004ccf27 to… |
| `00452b79` | RegisterGFCFontDeviceInterface | confirmed | Registers the FreeType2 font device as an interface of the GFC draw layer: copies the class name "GFontDeviceFT2" with FUN_004e4e6d, builds the wide strings L"… |
| `00458ab5` | GfcSystemFontScanFontDirectory | confirmed | Builds the 'Fonts/' asset prefix (FUN_004e889c + FUN_0044dcd9) and enumerates the font directory: for each found file (FUN_004442e5/FUN_0052b0f4) it filters na… |
| `00490072` | LoadFontInfo | confirmed | Loads a font description from the property bag param_1 through the 'GFontInfo' sub-object fetched with slot +0x68: FamilyName and StyleName through slot +0x4C … |
| `00490322` | StoreFontInfo | confirmed | Store twin of FUN_00490072: fetches the 'GFontInfo' sub-object through slot +0x6C and writes the caller's font block back - FamilyName/StyleName through slot +… |
| `004905d8` | LoadFontInfoEx | confirmed | Loads the extended font description from the 'GFontInfoEx' sub-object (read slot +0x68): FullName through slot +0x4C into param_3+0, EMSquare through slot +0x2… |
| `0049087f` | StoreFontInfoEx | confirmed | Store twin of FUN_004905d8: fetches the 'GFontInfoEx' sub-object through slot +0x6C and writes the extended metrics back - FullName through slot +0x50, EMSquar… |
| `00490b29` | LoadGlyphInfo | confirmed | Loads glyph metrics from the property sub-object named by param_2 (looked up as type 'GGlyphInfo' through slot +0x68): RenderSize through FUN_004eb55e into par… |
| `00cfff60` | FindOrCreateGlyphTextureEntry | confirmed | Looks up a glyph in a 512-entry page table, falls back to the font object when absent, creates a cache entry, and stores it in the page slot; logs and returns … |
| `00d04320` | GlyphTextureRowAlloc | confirmed | Allocates a width x height glyph rectangle in a row-packed texture: when the current row cannot fit the width it logs a hole with 00704db0, moves the row curso… |
| `00d04650` | GlyphBitmapRowAlloc | confirmed | Allocates a width x height rectangle in a 512 x 1024 bitmap: when cursor x + width + 2 is below 0x200 and cursor y + height + 2 is below 0x400 it calls 00d0460… |
| `00450923` | CtorVtables1764070 | likely | Constructor for the class with vtables 01764070/01764068: chains the base constructor FUN_004e68fb(param_2) and installs both vtable pointers. |
| `0045097e` | ScalarDeletingDtorFontObj | likely | Deleting destructor: calls FUN_00526562 on the object and, when the flag's low bit is set, frees it through thunk_FUN_004c6a61; returns the pointer. |
| `00450a65` | SetFontInfoProperty | likely | Calls FUN_004c6e70(param_2); when it returns nonzero, builds the temporary wide string L"FontInfo", calls FUN_00490322(param_2,name,this+0x24), and destroys th… |
| `00450ab7` | CtorVtables1764070_b | likely | Plain constructor for the 01764070/01764068 class: calls the base initialiser FUN_004d99d1 and installs both vtable pointers. |
| `00450ad0` | ReleaseFontFace | likely | Releases the face handle at +0x1C through FUN_006be300 and clears both +0x1C and the +0x14 field; returns 1. |
| `00450af3` | LoadGlyphMetrics | likely | Loads one glyph: resolves the char code with FUN_006ba1d0(face,code&0xFFFF), loads it with FUN_006bd770(face,index,0x808), obtains the glyph object through FUN… |
| `00450bdf` | BlitMask1bppOr | likely | ORs a packed 1-bit glyph mask into a bit-addressed destination: walks param_1[1] rows, ORing param_1[0] bits from the source base param_1[9] (bit offset param_… |
| `00450cec` | BlitMask4bpp | likely | Writes a coloured glyph nibble pattern into a 4-bit destination: for every set bit of the packed mask (read bit-by-bit from param_1[9] with the bit offset para… |
| `00450dae` | BlitMask8bpp | likely | Writes the byte value param_2 into the destination surface for every mask bit that is set: the packed 1-bit mask is read from param_1[9] (+0x24) with the bit o… |
| `00450e2a` | BlitMask16bpp | likely | Writes the 16-bit value param_2 into every pixel of the destination buffer (base +0x28 plus the 2-byte-per-pixel offset +0x10, strides +0x1C/+0x20) whose corre… |
| `00450eac` | BlitMask24bpp | likely | Writes the 24-bit value param_2 & 0xFFFFFF into every pixel of the destination buffer (base +0x28 addressed with three bytes per pixel through +0x10, destinati… |
| `00450f3c` | BlitMask32bpp | likely | Writes the 32-bit value param_2 into every pixel of the destination buffer (base +0x28 offset by four bytes per pixel through +0x10, strides +0x1C/+0x20) whose… |
| `00450fc8` | ScalePointAndAppend | likely | Calls FUN_004a74da (which re-appends the 8-byte pair currently held at container+0x30) and then FUN_004a74ab(0,-1,-1) with the point param_1[0]/param_1[1] scal… |
| `0045100c` | ScaleTwoPointsAndAppendA | likely | Scales the two points param_1 and param_2 by the global float _DAT_017641b8 and passes the four coordinates to FUN_004aaece, which compares the new pair agains… |
| `00451063` | ScaleTwoPointsAndAppendB | likely | Same shape as 0045100c but with the second point taken from param_3 (param_2 is unused): scales *param_1/param_1[1] and *param_3/param_3[1] by the global float… |
| `004510ba` | CtorVtables17641C8 | likely | Constructor for the class with vtables 017641C8/017641C0: calls FUN_0052f5b0 and installs both vtable pointers. |
| `0045149b` | FontKeyCompareName | likely | Compares only the first wide string of two records (two characters per iteration from the buffer pointed to by *param_1 and *param_2, stopping on the second ze… |
| `00451f28` | CtorCopyFontOwner | likely | Initialises the object's two string slots to the global empty wide string &DAT_01bc7e20, runs the base initialiser FUN_0048185c(param_2), then AddRefs the inte… |
| `00451f59` | ReleaseFontOwnerRef | likely | Releases the interface at +0x60 through slot +0x10 when non-null, then tail-jumps to FUN_004508ba for the two string subobjects. |
| `004524c9` | OffsetGridPoints | likely | For each of the count at +8 records it indexes a two-level array (+0x14 holds the page table, high byte selects the page, low byte selects an 8-byte record) an… |
| `0045255a` | CopySecondaryBase17642E4 | likely | Initialises a secondary base sub-object: writes PTR_LAB_0175c44c as the vtable, copies the two dwords at param_2+4 and param_2+8 into the sub-object, then over… |
| `0045257b` | FontRecordCopyCtor | likely | Copy-constructs the two wide-string fields through FUN_004e884e(param_2,-1) and FUN_004e884e(param_2+4,-1), then copies every dword field from source offsets +… |
| `00452626` | LoadFontInfoFromProperty | likely | When FUN_004c6e64(param_2) succeeds (plain non-null test) and FUN_00450ad0() has released this object's font face, it loads the FontInfo property of param_2: b… |
| `00452676` | FontDeviceCtor | likely | Constructor of the font-device object (vtables 017642f0/017642e8): chains the base constructor FUN_00450ab7, zeroes the word at +0x18, sets the three string sl… |
| `00452724` | FontDeviceDtor | likely | Destructor of the font-device object: rewrites both vtable pointers, releases the font face through FUN_00450ad0, increments the global counter DAT_01bc4578, c… |
| `00452ccd` | ReleaseField60 | likely | Releases the interface at +0x60 through vtable slot +0x10 (with 0) when non-null, then destroys two string temporaries through FUN_004508ba. |
| `00452d41` | CtorCopyFontOwnerFromPtr | likely | Initialises the two string slots to the global empty wide string &DAT_01bc7e20, runs the base initialiser FUN_0048185c(param_2), then AddRefs the interface poi… |
| `00452dad` | PointerArrayGrowAndAllocSlab | likely | Block-table grow and reserve: when the requested block index param_2 is not below the capacity at +8 it allocates (capacity + step at +0x10)*4 bytes through FU… |
| `00453571` | GfcFontCopyCtor | likely | Copy constructor body of the 0xe4-byte font object: runs the base copy 00450923, rewrites the two vtables PTR_FUN_017642f0 / PTR_LAB_017642e8, copies the words… |
| `004535dd` | GfcFontScalarDeletingDtor | likely | Scalar deleting destructor for the 0xe4-byte font object: calls destructor body 00452724, then passes this to allocator routine 004509dd only when the low flag… |
| `004535f9` | GfcFontGlyphDraw | likely | Looks up the renderer/resource object for id param_2 (006ba1d0/006bd770) and requires the tag at +0x48 to be 0x62697473 (bits) or 006bcb30 to succeed; then off… |
| `0045471b` | GfcFontCreate | likely | Passes L"GFC System Font" to 004e889c, allocates 0xe4 bytes through 004509c0, and constructs with 00452676. If +0xC is zero and either explicit dimension at +0… |
| `00454b11` | GfcFontSizeResolveStub | likely | Preserves the incoming ECX receiver for 004549aa, zeroes a two-word local, and passes its address as the fifth outgoing stack argument after the first four inc… |
| `00454bb7` | ScalePairAndAppend | likely | Uses x87 integer operations to multiply each 32-bit pair component by the qword constant at _DAT_017641b8 (1/64), stores each result as a float, calls 004544f8… |
| `00457564` | CtorVtable17644A0 | likely | Stores the table at 0x17644a0 into +0, the byte parameter at +4, param_3 at +8 and param_4 at +0xC, then initialises two container sub-objects through FUN_0045… |
| `004575cc` | CopyCtorVtable17644A0 | likely | Installs table 0x17644a0 at +0, copies the byte at +4 and the dwords at +8/+0xC from the source and copies the two container sub-objects through FUN_00456da1 a… |
| `0045760b` | GfcSystemFontLookup | likely | Computes a lookup key with FUN_00454cee/FUN_00452ee0 and tests it with FUN_004514ee; on a miss it canonicalises the descriptor through FUN_00454523 (the '%s %d… |
| `004577b6` | GfcSystemFontLookupDefaultSize | likely | Cached-font lookup that resolves the default point size first: 0x48 (72) when either size slot is zero, or GetDC(NULL)+GetDeviceCaps LOGPIXELSX (0x58)/LOGPIXEL… |
| `004581ae` | GfcSystemFontResizeRecords | likely | Calls FUN_00456f01 and then resizes the 0x24-byte record vector to param_1 entries through FUN_0045809b. |
| `004588e0` | GfcSystemFontAddEntry | likely | Inserts a font-cache entry: derives the key through FUN_00453789, probes with FUN_00451519/FUN_00451274 and FUN_0045149b, builds a default value through FUN_00… |
| `00458f8b` | GfcSystemFontManagerCopyMap | likely | Initialises the manager's map (FUN_004541d2) under an SEH frame and copy-assigns it from the source through FUN_004589ab; returns the receiver. |
| `00458ffa` | GfcSystemFontManagerDtor | likely | Destructor of the font manager: installs the two manager tables (0x176451c/0x1764518), drains the font map releasing each entry (FUN_006be300 on the glyph data… |
| `0045910d` | GfcSystemFontManagerCopyCtor | likely | Copy constructor of the font manager: runs the base copy constructor FUN_004575cc, copies the secondary GInterface-derived base at +0x28 through FUN_0045255a (… |
| `0045917c` | GfcSystemFontManagerScalarDtor | likely | MSVC scalar deleting destructor: runs the manager destructor FUN_00458ffa and releases the object through thunk_FUN_004c6a61 when the flag's low bit is set. |
| `00459198` | GfcSystemFontManagerCtor | likely | Default constructor of the 0x6c-byte font manager: runs the base initialiser FUN_00457564(0,0x200,8000) and FUN_004c9e10, installs the two manager tables, runs… |
| `00459221` | GfcSystemFontManagerFactory | likely | Lifetime dispatcher of the 0x6c-byte font manager: case 0 default-constructs (00459198), case 1 copy-constructs (0045910d), cases 2/3 allocate 0x6c bytes throu… |
| `0048a620` | GlyphLookupMetric | likely | Looks up character param_2 in the range at this+0x68..+0x6c, using page index code>>7, record index (code&0x7f)*0x44 and the table at this+0x88. A requested re… |
| `0048a6e8` | GlyphInfoGet | likely | Resolves the requested character record using the range at this+0x68..+0x6c, page table at +0x88 and 0x44-byte record stride. A valid requested record is used … |
| `00490cc0` | StoreGlyphInfo | likely | Store twin of FUN_00490b29 under the store slot +0x6C: writes RenderSize through FUN_004eb5f0, RenderOffset through FUN_004eb454, Size through FUN_004eb5f0 and… |
| `00490e77` | LookupSortedPairValue | likely | Binary-searches a sorted table of 8-byte entries for the key formed by the two 16-bit values param_2 and param_3: count at +0x34, entries at +0x3C (key in the … |
| `00490f3b` | CachePointDistanceAndTestAboveEpsilon | likely | Calls FUN_004846a1, which is a 2D point-distance helper (it loads both pointer arguments, subtracts and squares the x and y pairs and calls __CIsqrt), stores t… |
| `00490f7c` | WrapPagedIndexLookup | likely | Computes the wrapping index ((count-1)+param_2) % count (count at *param_1), then returns the 12-byte record addressed through the page table at +0xC: page = i… |
| `00491824` | InitGlyphAtlasPages | likely | Conditionally AddRefs param_9 through slot +0x0c and releases old +0x44, then stores param_9 at +0x44; it unconditionally calls param_9 slot +0x10c and stores … |
| `004919ad` | GetGlyphSlot | likely | Checks the low 16 bits of param_2 against +0xbc..+0xc0; in range, uses page=(param_2>>7)&0x1ff, slot=(param_2&0x7f) and stride 0x4c. It gets/allocates the page… |
| `00491ae2` | GetGlyphSlotSizeA | likely | Checks only the low 16 bits of param_2 against +0xbc..+0xc0; computes page=(param_2>>7)&0x1ff and slot=(param_2&0x7f) with stride 0x4c. If the slot is empty it… |
| `00794870` | LoadFontAndRegisterCache | likely | Thiscall (ECX = this, param_2 = font descriptor whose +4 is the name, param_3 = size, param_4 = owner argument). Allocates 0xa54 bytes with 00d833a6 and builds… |
| `00cffc80` | Initialize128SlotCache | likely | Clamps the requested limit to 6 through 0x40, clears 0x80 dwords in the supplied state, initializes backing storage through 006c02a0, and clears the root point… |
| `00cffd70` | PagedCacheSetValue | likely | Allocates and clears a 0x800-byte page when the key's 512-entry page is absent, then stores the supplied value at the key's low-nine-bit slot. |
| `00cffdd0` | ComputeFixedPointSpan | likely | Converts two supplied values through the shared fixed-point helpers, computes their difference, and returns the resulting signed distance shifted right by six … |
| `00cffe70` | CreateGlyphTextureEntry | likely | Allocates a 32-byte entry, stores the source rectangle dimensions shifted right by six, asks the glyph texture cache to place it, and frees the entry when plac… |
| `00d00000` | CreateGlyphTextureEntry | likely | Reads and clamps the source texture width and height to 0x40, allocates a 32-byte entry, then asks the fixed-size glyph texture cache to place it; frees the en… |
| `00d00470` | MeasureWideTextWidth | likely | Looks up each UTF-16 glyph, substitutes the space glyph when missing, accumulates glyph advances and pair spacing, and returns the resulting text width. |
| `00d00eb0` | RegisterFontTextureCache | likely | Allocates a cache object, initializes it with the requested dimensions, stores it in the owner's cache array and increments the count; on initialization failur… |
| `00d012c0` | PrepareWideBufferThenMeasure | likely | Creates a 2068-byte local buffer, initializes helper state, formats data from the first argument into that buffer, passes the buffer and second argument to 00d… |
| `00d01380` | TruncateWideStringToPixelWidth | likely | Walks the UTF-16 string, obtains each glyph record through 00cfff60, substitutes the space glyph when a lookup fails, accumulates glyph widths and pair spacing… |
| `00d04230` | ReleaseTwoRefcounted | likely | Releases two ref-counted pointers: for this+0x4 and this+0x8, if non-null, LOCK-decrements the count at its +0x4, calls its vtable slot [1] when it reaches zer… |
| `00d04280` | LockFillAlphaRows | likely | Locks a surface through vtable slot [0xc] of the object at param_1+4 (pitch returned via &local_8), writes 4-byte texels with 0xff in three channels and a sour… |
| `00d04540` | GlyphTexturePairCreate | likely | Stores the device pointer param_2 at offset 0, zeroes fields +0xc..+0x20, sets size fields 0x200 x 0x200 (or 0x400 x 0x400 when param_3 is non-zero) and 6 (or … |

### `engine.gfc.fx`

**GBlitFx** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00405b36` | GBlitFx::GBlitFx | confirmed | Constructor: runs the GDrawFx base constructor (00405a32), overwrites the vtable at +0 and +4 with GBlitFx::vftable, zeroes +0x28 and +0x2C, then calls FUN_004… |
| `0041873b` | GBlitFx::GBlitFx | confirmed | Constructor of the blit draw-effect object: runs the GDrawFx base constructor FUN_00405a32, stores the GBlitFx vtable pair at +0/+4, sets the kind field at +0x… |
| `00480689` | GBlitFx::WriteGlobalAndOptionalArgument | confirmed | Writes DAT_01af861c into *param_2 and, when param_3 is non-null, writes param_1 into *param_3. |
| `00480714` | RegisterGBlitFx | confirmed | Registers the GFC class "GBlitFx" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |
| `0048074d` | GBlitFx::CopyFieldsChecked | confirmed | Calls the guarded base copy FxCopyBaseFields and, when it succeeds, copies fields +0x24,+0x28,+0x2c,+0x30,+0x34 from param_2 to this, returning 1. |

**GDrawFx** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00405a32` | GDrawFx::GDrawFx | confirmed | Constructor: runs the GObjectBase base constructor FUN_004d99d1, installs the GDrawFx vtable pointer into both +0 and +4, zeroes +0x1C and +0x20, and returns t… |
| `004801e5` | GDrawFx::WriteGlobalAndOptionalArgument | confirmed | Writes DAT_01af81c4 into *param_2 and, when param_3 is non-null, writes param_1 into *param_3. |
| `00480270` | RegisterGDrawFx | confirmed | Registers the GFC class "GDrawFx" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |
| `0048e0b4` | GDrawFx::CopyCtor | confirmed | Constructs a GDrawFx in param_1 from the source effect object param_2: runs FUN_004e68fb(param_2), installs the GDrawFx vtable into both +0 and +4, and copies … |

**GFillFx** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00405ab0` | GFillFx::GFillFx | confirmed | Constructor: runs the GDrawFx base constructor (00405a32), overwrites the vtable at +0 and +4 with GFillFx::vftable, then calls FUN_004d9a26(param_2, 0); retur… |
| `00418700` | GFillFx::GFillFx | confirmed | Constructor of the fill draw-effect object: runs the GDrawFx base constructor FUN_00405a32, stores the GFillFx vtable pair at +0/+4, sets the kind field at +0x… |
| `00480536` | GFillFx::WriteGlobalAndOptionalArgument | confirmed | Writes DAT_01af84c4 into *param_2 and, when param_3 is non-null, writes param_1 into *param_3. |
| `004805c1` | RegisterGFillFx | confirmed | Registers the GFC class "GFillFx" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |
| `004805fa` | GFillFx::CopyFieldsAndBlockChecked | confirmed | Calls the guarded base copy FxCopyBaseFields, copies fields +0x24,+0x28,+0x2c,+0x30 from param_2 and memcpy's the 0x20 bytes at +0x34, returning the copy statu… |

**GFlashFx** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004595b0` | GFlashFx::GFlashFx | confirmed | Runs the GDrawFx base constructor (00405a32), zeroes the field at +0x24, installs GFlashFx::vftable at +0 and +4 and stores the constant 6 at +0x18; returns th… |
| `004595f2` | GFlashFx::Slot2Thunk5017F8 | confirmed | Three-argument forwarder to FUN_005017f8: raw bytes ff 74 24 08 / ff 74 24 08 / 51 / e8 f8 81 0a 00 / 83 c4 0c / c2 08 00 push the two stack arguments and ECX … |
| `00480431` | GFlashFx::WriteGlobalAndOptionalArgument | confirmed | Writes DAT_01af8414 into *param_2 and, when param_3 is non-null, writes param_1 into *param_3. |
| `004804bc` | RegisterGFlashFx | confirmed | Registers the GFC class "GFlashFx" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |
| `0045960b` | GFlashFx::GFlashFx_2 | likely | Runs GDrawFx constructor 00405a32 with ECX=this, installs the GFlashFx vtable at +0 and +4, pushes 0 then param_2 and calls 004d9a26 with ECX=this, then return… |

**GLineFx** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0040e68c` | GLineFx::GLineFx | confirmed | Constructs a GLineFx effect: runs the base effect constructor FUN_00405a32 (GDrawFx), writes the two GLineFx vtable pointers at +0 and +4, then attaches the pa… |
| `004186c9` | GLineFx::GLineFx | confirmed | Constructor of the line draw-effect object: runs the GDrawFx base constructor FUN_00405a32, stores the GLineFx vtable pair at +0/+4, sets the kind field at +0x… |
| `00480302` | GLineFx::WriteGlobalAndOptionalArgument | confirmed | Writes DAT_01af82a4 into *param_2 and, when param_3 is non-null, writes param_1 into *param_3. |
| `0048038d` | RegisterGLineFx | confirmed | Registers the GFC class "GLineFx" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |
| `004803c6` | GLineFx::CopyFieldsCheckedPlus | confirmed | Calls the guarded base copy FxCopyBaseFields and, when it succeeds, copies fields +0x24,+0x28,+0x2c,+0x30 from param_2 to this, returning 1. |

**GTextFx** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0048084f` | RegisterGTextFx | confirmed | Registers the GFC class "GTextFx" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**(no class)** (27)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00419c67` | CtorVtables175D5E8 | confirmed | Constructor of the composite draw object: installs the two-vtable pair 0x175D5E8/0x175D5E4 after running the base initialiser FUN_00410161, copies two dwords, … |
| `0043d5a3` | ConfigureBlitEffect | confirmed | Forces state bit 0x1000000 and returns 0 if the bit does not take effect; adjusts state bit 0x80000 by param_3; if no effect exists, stack-constructs a GBlitFx… |
| `0048c9f4` | LoadColorCombineOps | confirmed | Fetches the 'GColorCombine' sub-object from the property bag param_1 through the bag's vtable slot +0x68 (read accessor), then reads its 'ColorOp' and 'AlphaOp… |
| `0048ca8f` | StoreColorCombineXY | confirmed | Looks up the GColorCombine sub-object through bag write slot +0x6c. It interns data at 01769f28 and writes param_3[0] through sub-object slot +0x4c, then inter… |
| `0048de8a` | LoadBlitDrawState | confirmed | Loads the bitmap-blit draw state from the property bag param_1: fetches the sub-object named 'GFontInfo' through slot +0x68, reads 'Flags' with default string … |
| `0048df9c` | StoreBlitDrawState | confirmed | Store twin of FUN_0048de8a: fetches the 'GFontInfo' sub-object through slot +0x6c and writes the caller's block back - Flags through slot +0x48 with default GF… |
| `0048e0ee` | LoadFxBase | confirmed | Validates the property bag param_2 with FUN_004c6e64, then loads the shared effect base fields into param_1: 'FxType' through the bag's slot +0x48 into +0x18, … |
| `0048e15b` | StoreFxBase | confirmed | Store twin of FUN_0048e0ee: validates with FUN_004c6e70 and writes the shared effect base fields back - 'FxType' through slot +0x4c from +0x18 and 'DestCombine… |
| `0048e1c7` | LoadLineFxProps | confirmed | Loads the line-effect properties into the caller's block: chains FUN_0048e0ee for the base fields, reads 'Flags' with default GFC_LFX_ANTIALIAS through slot +0… |
| `0048e2af` | StoreLineFxProps | confirmed | Store twin of FUN_0048e1c7: chains FUN_0048e15b, writes 'Flags' through slot +0x48 with default GFC_LFX_ANTIALIAS from +0x14, 'Color1'/'Color2' through FUN_004… |
| `0048e441` | LoadFlashFxProps | confirmed | Chains the base loader FUN_0048e0ee for the shared effect fields and then reads the property 'FlashMovieName' through the bag's slot +0x30 into param_1+0x24, r… |
| `0048e489` | StoreFlashFxProps | confirmed | Store twin of FUN_0048e441: chains FUN_0048e15b and writes the pointer held at +0x24 back into the 'FlashMovieName' property through the bag's slot +0x34. |
| `0048e4cf` | LoadFillFxProps | confirmed | Loads the fill-effect properties: chains FUN_0048e0ee, reads 'Flags' with default GFC_FFX_ANTIALIAS into +0x14, 'ForegroundColor'/'BackgroundColor' through FUN… |
| `0048e5d8` | StoreFillFxProps | confirmed | Store twin of FUN_0048e4cf: chains FUN_0048e15b, writes 'Flags' through slot +0x48 with default GFC_FFX_ANTIALIAS from +0x14, 'ForegroundColor'/'BackgroundColo… |
| `0048e6d8` | LoadColorKeyFxProps | confirmed | Loads the colour-key blending effect properties: chains FUN_0048e0ee, reads 'Flags' with default GFC_BFX_ANTIALIAS into +0x14, 'ColorKey' and 'ColorKeyMask' th… |
| `0048e7c2` | StoreColorKeyFxProps | confirmed | Store twin of FUN_0048e6d8: chains FUN_0048e15b, writes 'Flags' with default GFC_BFX_ANTIALIAS through slot +0x48, 'ColorKey'/'ColorKeyMask' through FUN_0048d1… |
| `0048e8a8` | LoadTextFxProps | confirmed | Loads the text-effect properties: chains FUN_0048e0ee, reads 'Flags' with default GFC_TXTFX_ANTIALIAS into +0x14, 'BackgroundColor' through FUN_0048d12b into +… |
| `0048e992` | StoreTextFxProps | confirmed | Store twin of FUN_0048e8a8: chains FUN_0048e15b, writes 'Flags' with default GFC_TXTFX_ANTIALIAS through slot +0x48, 'BackgroundColor' through FUN_0048d1fe fro… |
| `00419d87` | ScalarDeletingDtor | likely | MSVC scalar deleting destructor: runs the destructor body FUN_00410182 (which unwinds this object's vtable chain towards GInterface and releases held pieces) a… |
| `004595d4` | GFlashFxCallTable0 | likely | Calls the first entry of the function table stored at +0xC with (this + 4) as argument (draw-effect virtual invocation pattern of the GDrawFx family). |
| `004595df` | GFlashFxCallTable1 | likely | Calls the second entry of the function table at +0xC with (this + 4, param_2) as arguments. |
| `004802d4` | FxCopyBaseFieldsUnchecked | likely | Copies the four base fields +0x14,+0x18,+0x1c,+0x20 from param_2 to this without any compatibility check. |
| `004803f7` | FxCopyFieldsPlusUnchecked | likely | Calls the unchecked base copy FxCopyBaseFieldsUnchecked, copies fields +0x24,+0x28,+0x2c,+0x30 from param_2 to this and returns this. |
| `0048050e` | FxCopyField24Unchecked | likely | Calls the unchecked base copy FxCopyBaseFieldsUnchecked, copies field +0x24 from param_2 to this and returns this. |
| `0048063d` | FxCopyFieldsAndBlockUnchecked | likely | Unchecked counterpart of FxCopyFieldsAndBlock: base copy plus fields +0x24..+0x30 and the 0x20-byte memcpy at +0x34. |
| `00480784` | FxCopyFields2Unchecked | likely | Unchecked counterpart of FxCopyFields2: base copy plus fields +0x24..+0x34, returning this. |
| `00480888` | CtorVtables1769298 | likely | Constructor: runs the base initializer FUN_0052f5b0 and installs primary vtable 01769298 at offset 0 and secondary vtable 01769290 at +4, returning its this po… |

### `engine.gfc.layout`

**GCtrl** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0043d778` | GCtrl::GetOrCreateAttachedVBox | confirmed | Thiscall with one stack byte argument (ret 4). Returns this+0xb4 when it is non-null. When it is null and the argument is nonzero: allocates 0x8c bytes through… |

**(no class)** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004159a4` | LoadControlNameAndFixedSize | confirmed | Loads the object state through 004154e7 (flags/stretch) and then, if that succeeds, releases the interface held at +0x44, resets it, and reads the node attribu… |
| `00415a37` | StoreControlNameAndFixedSize | confirmed | Write half of the pair: after the shared layout-theme pre-save FUN_004094c7, it takes the name string source (the interface at +0x44 or the inline buffer at +0… |
| `00415c22` | StoreSpacerState | confirmed | Write half of the spacer state: after the shared layout-theme pre-save FUN_004094c7 it writes the non-zero 16-bit 'SpacerSizeId', the non-zero 'SpacerSize' pai… |
| `00416099` | StoreTableSeparatorState | confirmed | After the shared item store 00415e55 it writes the table separator and margin state into the node: non-zero 'OuterMarginId' and 'SeparatorId' 16-bit attributes… |
| `004152c2` | InstantiateItemClassFromNode | likely | Looks an item node up through param_1's vtable slot +0x64, reads its 'ItemClass' attribute into a local block (default DAT_01bc7e28), and when present reads th… |
| `004153b9` | StoreItemClassAndData | likely | Write-back counterpart of 004152c2: resolves the item node through param_2's vtable slot +0x68 when param_3 is set, obtains the current object block through it… |
| `004154e7` | LoadFlagsAndStretch | likely | Reads the node attributes into the object's layout state: 'Flags' (dword into +0x10, default block PTR_s_GFC_LAYOUT_HDEFAULT_01af47d0), then 'HStretch' into by… |
| `004155a8` | CopyStateBlockToCreatedObject | likely | Asks its own vtable slot +4 for a target through a local descriptor, takes the object produced by FUN_004cd239 and, when present, copies +0x10, the 16-bit +0x1… |
| `004155fe` | GetCachedPairByIndex | likely | When the 0x200 flag bit of the 16-bit field at +0x14 is set it refreshes the cached pair table: reads its own vtable slot +0x2C for flag bits, fetches three pa… |
| `004156b2` | ArrangeChildRectInParent | likely | Positions a child rectangle (param_2 = left/top/right/bottom) inside the object's cached rectangle: the fast path applies only when bit 0x100 of the 16-bit +0x… |
| `0041588c` | GetSizeValueById | likely | Resolves a two-dword size value for the 16-bit id in param_2: for id 0 it copies the source pair (param_3[0], param_3[1]) straight out; otherwise it finds the … |
| `00415f75` | OffsetRectAndChildren | likely | Offsets the object's rectangle (+0x1C/+0x20/+0x24/+0x28) by (param_2, param_3) and then walks the child list (FUN_0040a67f/FUN_0040a6b3), calling each visible/… |
| `00416025` | GetWords54_58 | likely | Copies the two dwords at +0x54/+0x58 into param_2[0] and param_2[1]; no other reads or writes. |
| `004165ef` | StoreStackState | likely | After the shared item store 00415e55 it stores the stack state into the node: when the interface at +0x60 is held it writes 'ItemIndex' through the node slot +… |

### `engine.gfc.parse`

**(no class)** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00604b9a` | BuildRegExpGrammarRules | confirmed | Builds the regular-expression grammar: interns 24 (0x18) symbol names (RegExp, RegExp', B, C, D, E, CharSet, GroupSet, character, +, ?, ., \|, *, (, ), [, ], {… |
| `00605243` | GNFA_ParseRegExp | confirmed | Parses one regular expression into an NFA node: lazily builds the regex grammar once (00604728 ctor, 00604b9a grammar rules, table build 0060469d(arg,3)) under… |
| `006054c9` | GDFA_ParseRegExp | confirmed | Parses one regular expression into a DFA syntax tree: under the DAT_01bcfca4 one-time guard it builds the regex grammar (00604728, 00604b9a, table build 006046… |
| `006057d1` | BuildNFAsAndCombineDFAs | confirmed | Walks the expression container at this+4 (FUN_00553a83/FUN_00552aa8/FUN_00554a03) and builds an NFA for each expression with FUN_00605243, storing the node poi… |
| `00602a53` | BuildParseTables | likely | Builds the LR parse tables for a grammar object: counts the grammar's symbols/states, allocates the state, terminal and nonterminal arrays (FUN_005f40ae/005f40… |
| `0060469d` | ArgShuffleToBuildParseTables | likely | 17-byte calling-convention shim (push [esp+8]; push ecx; mov ecx,[esp+0xc]; call 00602a53; ret 8): it passes its own ECX/this as the second argument and its fi… |
| `006046ba` | Array0x4c_Destroy | likely | Destroys a 0x4c-stride element array through this: when the buffer pointer at this+4 is non-null it calls FUN_00600877(this+4, this+8, this) to destroy the ele… |
| `006046e7` | Array0x4c_ConstructN | likely | Forwards to FUN_00602412(dest, count, value, extra, 0, 0): constructs `count` 0x4c-byte elements at `dest`, copy-constructing each from the source element `val… |
| `00604728` | CtorInitListMembers | likely | Constructor-shaped initialiser: runs six member constructors - FUN_005e77ef, FUN_005e7827, FUN_005e7851, FUN_005e7910 (intrusive list/map heads) and FUN_005fd1… |
| `00604775` | CtorInitListMembersWithArgs | likely | Same member-initialisation body as 00604728 (FUN_005e77ef/005e7827/005e7851/005e7910/005fd138/005db243 plus zeroed fields at +0x1c..+0x24, +0x48..+0x50, +0x34)… |
| `0060483f` | CtorZeroFieldsThenBuildTables | likely | Constructor-shaped: zeroes three embedded arrays (this+4/+8/+0xc for 0x4c-byte elements, this+0x14..+0x1c, this+0x24..+0x2c), constructs the member at this+0x3… |
| `00604895` | Array0x4c_ConstructAt | likely | Constructs `count` (arg2) 0x4c-byte elements starting at the element position arg1, copy-constructing each from the source element arg3 (FUN_006046e7 into FUN_… |
| `006048bc` | Array0x4c_CopyRange | likely | Thin wrapper (push ecx; push arg2; push arg1; push arg0; call 0060247c; ret 0xc) that uninitialised-copies the 0x4c-stride element range [arg0, arg1) to the de… |
| `006048d4` | Array0x4c_InsertFill | likely | Vector-style insert of `count` 0x4c-byte elements at the element position arg1, copying them from the source element arg2 (a stack temp is built from it with F… |
| `00604ae2` | Array0x4c_AppendSlow | likely | Slow-path append for the 0x4c-element array: when the array is not empty it saves the current state through FUN_005e2779/FUN_005dd136, then inserts one element… |
| `00604b45` | Array0x4c_PushBack | likely | Appends one 0x4c-byte element to the array: if the element count (FUN_005db7d5) is below the capacity (FUN_005dbd20) it constructs the element in place with FU… |
| `00605c94` | RegExpSampleSelfTest | likely | Walks the expression container at this+4 and parses every element into a DFA node with FUN_006054c9 (0x18-byte nodes from operator new, discarded with FUN_005e… |

### `engine.gfc.serialization`

**GSimpleClassIO** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00403ddf` | GSimpleClassIO::GSimpleClassIO | likely | Constructs a GSimpleClassIO: stores the property-output callback at +8, the property-input callback at +4 and the class-name string at +0xC, then runs the base… |

**(no class)** (47)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004354b2` | DtorUnlinkFromOwner | confirmed | Destructor body: rewrites the derived vtables at +0 and +4, and when the owner pointer at +0x18 is non-null either calls the owner's slot +0x5c(this) (when the… |
| `00482437` | ReadStringStringFormatMembers | confirmed | Gates on a non-null argument (FUN_004c6e70), fetches this+0x18 vtable slot +0x30, lets that object approve the argument through slot +0x28, then reads two name… |
| `004824b5` | CopyStringStringFormatMembers | confirmed | After the interface approval check, calls FUN_004e8d36 with param_2+0x34 and FUN_0052a1c0 with param_2+0x38; returns 1 on approval and 0 otherwise. |
| `00486369` | ReadGBitmap | confirmed | Deserializes a GFC bitmap: frees any previous image (slot +0x128), reads the nested "Header" block through 00485E3F, copies the "ColorKey" dword into +0x48, an… |
| `00486586` | WriteGBitmap | confirmed | Serializes a GFC bitmap: writes the "Header" block through 00485F9D, writes the "ColorKey" dword through the output's slot +0x30, then writes the "Data" (and "… |
| `00486ce2` | ReadColorDataPlane | confirmed | Reads a color plane: frees any previous buffer (slot +0x110), reads "Flags" through the input's slot +0x44 with the default GFC_CLR_ORDER_ARGB, reads "Size" th… |
| `013c23f0` | SerialAppendInteger | confirmed | Writes one 32-bit integer into the serialization buffer in state 2 (writing; the state is set to 2 when it is 0 and a different state reports through FUN_012f5… |
| `013c2640` | SerialStartAppendContainer | confirmed | Sets the writing state (2) when it is 0, reports a state error otherwise when the state is not 2, then writes param_2 (the element count) with SerialAppendInte… |
| `013c2690` | SerialStartReadContainer | confirmed | Sets the reading state (1) when it is 0, reports a state error when the state is not 1, then reads the element count into param_2 with SerialReadInteger (013c2… |
| `013c27e0` | SerialReadTimeValue | confirmed | Sets the reading state (1) when it is 0 and reports a state error otherwise; if at least four bytes remain (read position + 4 <= limit) it reads two integers w… |
| `013c2860` | SerialAppendString | confirmed | Sets the writing state (2) when it is 0 and reports a state error otherwise. It writes the length (the int at +0x14 of the argument string) with SerialAppendIn… |
| `013c28d0` | SerialReadString | confirmed | Sets the reading state (1) when it is 0 and reports a state error otherwise. Reads the length with SerialReadInteger; when the length is 0x1400000 or less and … |
| `004352c1` | StringAccessWrapperResolve | likely | When the cached accessor at +8 is null, allocates 0x34 bytes (FUN_004d9dbd), constructs through FUN_00428da5(&emptyString,param_1[4]) and tries to attach it wi… |
| `00435394` | GStringAccessInit | likely | Initializes a holder object around param_2: runs the base constructor FUN_00409180(param_2), writes the GStringAccess vtable at +0x18, constructs the reference… |
| `00436c51` | InitTreeHeader19Thunk | likely | 12-byte wrapper (push esi; mov esi,ecx; call FUN_0041bd1c; mov eax,esi; pop esi; ret): runs the tree-header initializer FUN_0041bd1c on its receiver and return… |
| `00436c5d` | InitTreeHeader1DThunk | likely | 12-byte wrapper (call FUN_0041bd41 on ECX, then return the receiver): runs the second tree-header initializer emission FUN_0041bd41 on its receiver; no callers… |
| `00436c75` | InitContainerHeaderThunk1 | likely | 12-byte wrapper (call FUN_0041f75d on ECX, then return the receiver): runs the container-header initializer FUN_0041f75d (map-accepted as InitContainerHeader_1… |
| `00436c81` | InitContainerHeaderThunk2 | likely | 12-byte wrapper (call FUN_0041f787 on ECX, then return the receiver): runs the second container-header initializer FUN_0041f787 (map-accepted as InitContainerH… |
| `00436e5d` | AccessObjectScalarDeletingDtor | likely | Runs the destructor body FUN_00528f6a and, when the flag argument's low bit is set, frees the object through the CRT free thunk FUN_004c6a61; returns the point… |
| `00482589` | ReadPropertyAsNumberOrObject | likely | Clears a 4-byte tagged value (FUN_004c90b5), calls the property interface at this+0x14 slot +0x4C with a default at 01BC689C to fill it, then converts: tag 8 s… |
| `00482889` | ComposeStringFromMembers | likely | Builds temporary GStrings from the wide text at 01768E64 and from the member strings at +0x68 and +0x6C, concatenates them through FUN_004e8f3d, releases every… |
| `004c5f08` | LengthPrefixedBufferRead | likely | Thiscall (reader in ECX, destination param_2, count param_3): when the source block pointer at reader+8 is null it sets reader+0x10 to 9 and returns 0; otherwi… |
| `004c5fe7` | StreamCopyChunked4K | likely | Thiscall (this in ECX; out-stream at this+4, error field this+0x10): returns 0 when the out-stream pointer at this+8 is null; otherwise copies param_3 bytes fr… |
| `004e74da` | BitBufferReadFromStream | likely | Thiscall (this = bit buffer, param_2 = stream): reads a variable-length count into this+4 through 004c84c8, computes the byte count (bits+7)>>3, resizes the bu… |
| `0064f265` | ReadMenuItemFields | likely | Thiscall with a data-node pointer: when the node's first check (vtable +0x24 on the object at this+0x14 slot +0x30) passes, reads int 'MenuItemFlags' into this… |
| `0064f332` | WriteMenuItemFields | likely | Thiscall with a data-node pointer: if the same first check passes, writes this+0x30 as int 'MenuItemFlags' via the node's slot +0x48, writes the DataValue at t… |
| `00651325` | ReadSizeGripFlags | likely | Thiscall with a data-node pointer: when the base reader 00436f31 succeeds, reads the int 'SizeGripFlags' from the node (slot +0x44, name GFC_CF_SIZEGRIP_HNORES… |
| `0065139a` | WriteSizeGripFlags | likely | Thiscall with a data-node pointer: when the base writer 00418ce1 succeeds, writes this+0xe8 as the int 'SizeGripFlags' through the node's slot +0x48 (name GFC_… |
| `00652329` | ReadSliderItemFields | likely | Thiscall with a data-node pointer: when the node check 004c6e64 passes, reads int 'SliderItemFlags' into this+0x14 via the node's slot +0x44 (name GFC_SLIDER_I… |
| `006523c1` | WriteSliderItemFields | likely | Thiscall with a data-node pointer: when the node check 004c6e70 passes, writes this+0x14 as int 'SliderItemFlags' via the node's slot +0x48 (GFC_SLIDER_ITEM_RE… |
| `0065255a` | ReadSliderTickFields | likely | Thiscall with a data-node pointer: when ReadSliderItemFields (00652329) succeeds, reads 'TickLength' into this+0x2c and 'TickThickness' into this+0x30 through … |
| `0065260c` | WriteSliderTickFields | likely | Thiscall with a data-node pointer: when WriteSliderItemFields (006523c1) succeeds, writes 'TickLength' from this+0x2c and 'TickThickness' from this+0x30 throug… |
| `006527d7` | ReadSliderFormatFields | likely | Thiscall with a data-node pointer: when ReadSliderItemFields (00652329) succeeds, sets the defaults FormatFactor = 1.0 at this+0x38 and FormatOffset = 0.0 at t… |
| `00652877` | WriteSliderFormatFields | likely | Thiscall with a data-node pointer: when WriteSliderItemFields (006523c1) succeeds, writes 'FormatFactor' from this+0x38 only when it is not 1.0, 'FormatOffset'… |
| `00653c22` | ReadTabItemFields | likely | Thiscall with a data-node pointer: when the node check 004c6e64 passes and the child check (slot +0x24 of the object at this+0x14 slot +0x30) passes, reads int… |
| `00653cea` | WriteTabItemFields | likely | Thiscall with a data-node pointer: when the node check 004c6e70 and the child check (slot +0x28) pass, writes this+0x30 as int 'TabItemFlags' through the node'… |
| `00654e60` | ReadTitleBarFields | likely | Thiscall with a data-node pointer: when the base reader 00436f31 succeeds, reads int 'TitleBarFlags' (GFC_CF_TITLEBAR_CLOSEBOX default) into this+0xe8 via the … |
| `00654f3a` | WriteTitleBarFields | likely | Thiscall with a data-node pointer: when the base writer 00418ce1 succeeds, writes this+0xe8 as int 'TitleBarFlags' through the node's slot +0x48, writes 'Title… |
| `009357a0` | ReadBoundedU32 | likely | If fewer than four bytes remain, writes zero through the output pointer, marks the reader failed, moves its cursor to the end, and transfers control to the rea… |
| `00935830` | SerializeRecordFieldsAndNestedList | likely | Serializes eight fields at +0x20 through +0x90 and a count/list at +0xa0; writes three clamped 32-bit fields from +0x700d8/+0x700d4/+0x700dc, a length-prefixed… |
| `00a24750` | ReadBoundedScalarRecord 🌐 | likely | Reads two 32-bit fields and multiple one-byte flags from a bounded cursor, calls nested-value and length-bounded read helpers, then stores trailing scalar fiel… |
| `013004c0` | SerialReadVector4Local | likely | Thiscall reader: count via 013c2690, reserves storage through 012fe3e0, then for each element reads a 4-byte value through 014000c0 and appends it with 013003e… |
| `01300540` | SerialReadVector4Array | likely | Thiscall reader: count via 013c2690, reserves storage through 012f6b00, reads each element through 013ffda0 and appends it with 0042c155 (ArrayPushBackI); abor… |
| `01301640` | SerialReadStringList | likely | Thiscall reader for a string list: reads the element count with SerialStartReadContainer (013c2690), reserves it through 01300ad0 (Vector28_Reserve), then for … |
| `01302e20` | SerialReadOptionalPersistent | likely | Reads an optional polymorphic object: reads a presence integer with 013c24d0 (returns 0 on a failed read); releases the existing object in the destination slot… |
| `013a0b50` | SerialWriteTreeIntPairs | likely | Thiscall (ECX = serializer object pointer; arg = tree container). Calls 013c2640 (SerialStartAppendContainer) with the container size at +8. Walks the tree in … |
| `013c2790` | SerialAppendTimeValue | likely | Sets the writing state (2) when it is 0 and reports a state error otherwise; then writes the integer *param_2 with SerialAppendInteger and, when that succeeds,… |

### `engine.gfc.string`

**BaseClient** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c5b90` | BaseClient::StringCodeLookup | confirmed | Hashes param_2 with FUN_00705b00 and searches the bucket at this+0x1c+0x290 (index hash and 0xff) for an entry whose key matches. On a hit it confirms the stri… |
| `008c5c50` | BaseClient::StringCodeLookupSlot3 | confirmed | Same lookup and miss-log path as BaseClient::StringCodeLookup (008c5b90), but on a hit it calls vslot 1 (+0x4) of the owner with (id, param_3) instead of vslot… |
| `008c5d10` | BaseClient::StringCodeFind | confirmed | Hashes param_2 with FUN_00705b00, searches the bucket at this+0x1c+0x290 for a key match, then confirms the string with FUN_006fc5b0 along the chain. Returns t… |

**GCString** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007054c0` | GCStringCopyConstructorClosure | confirmed | Forwards the copy-constructor closure arguments to the GCString copy-construction routine. |

**GString** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007054d0` | GStringCopyConstructorClosure | confirmed | Forwards the GString copy-constructor closure arguments to its shared copy-construction routine. |

**GStringAccess** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004c53c6` | GStringAccess::StoreGlobalPair_01cc4434 | confirmed | Virtual method at GStringAccess::vftable slot 1: same shape as 004c537a (stores ECX to *arg2 when nonnull, then the dword at global 0x01cc4434 to *arg1; stdcal… |

**GStringAccessWrapper** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0041d2e3` | GStringAccessWrapper::ScalarDeletingDtor | confirmed | Calls FUN_004149C8 on this, frees this through _free when param_2 bit 0 is set, and returns this otherwise. |
| `0041b41f` | GStringAccessWrapper::ApplySlotValue | likely | When holder count at +0x10 is zero, clears the short at +0x0C, Releases the current interface at +8, then stores an AddRef'd incoming interface when the source… |

**(no class)** (42)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0093c3a0` | InitIStringFields | confirmed | Installs the IString vtable and shared empty-buffer pointer, then zeros length, capacity, and trailing dword fields. |
| `0044a33e` | GStringObjectActionDispatcher | likely | Branches on action param_1: 0 calls initializer 004466ce when param_2 is non-null; 1 calls copy initializer 00447b47 when param_2 is non-null; 2 and 3 request … |
| `004c4e66` | PooledStringCopy | likely | Fastcall (this in ECX): reads the length dword at *(*param_1), computes a 4-byte-aligned size plus 8 header bytes, and bump-allocates that block from the pool … |
| `004c60da` | WideBackslashToSlash | likely | Cdecl-style (holder pointer in param_1, wide-character buffer after a length dword): walks the wide characters and replaces each single backslash (0x5c) with a… |
| `004c611a` | WideLastPathSeparator | likely | Scans a wide-character holder backwards from min(param_2, length-1): returns the index of the first character that is a slash (0x2f), backslash (0x5c) or colon… |
| `004c6871` | HolderStringLess | likely | Thiscall (this in ECX, other object param_2): compares the length-prefixed string at this+0x14 (bytes from +4, length plus one, including the terminator) with … |
| `004e4f90` | GetStaticEmptyIStringOnce | likely | On first call sets bit 0 of DAT_01bc7fb4, stores the empty sentinel DAT_01bc7e28 in DAT_01bc7fb0 and registers LAB_017467c3 with _atexit; returns the address o… |
| `004e4fbb` | IStringResizeFill | likely | Thiscall (ECX=this, param_2 = new length, param_3 = fill byte): for length 0 releases the block (004c6a61) and resets to the empty sentinel; for a length above… |
| `004e57b6` | IStringFromInt | likely | Thiscall: formats a signed int as decimal with __itoa_s into a 64-byte buffer, then assigns the text to the IString (004e517a with count -1) on success, or cal… |
| `004e57f1` | IStringFromULong | likely | Thiscall: formats an unsigned long as decimal with __ultoa_s into a 64-byte buffer, then assigns the text through 004e517a(count -1) on success or calls 004d83… |
| `004e582c` | IStringFromInt64 | likely | Thiscall: formats the 64-bit value (param_3:param_2) as decimal with __i64toa_s into a 64-byte buffer, then assigns the text through 004e517a(count -1) on succ… |
| `004e586a` | IStringFromUInt64 | likely | Thiscall: formats the unsigned 64-bit value (param_3:param_2) as decimal with __ui64toa_s into a 64-byte buffer, then assigns the text through 004e517a(count -… |
| `004e58a8` | IStringFormatV | likely | Thiscall (this, param_2 = printf format, param_3 = va_list): first walks the format counting output characters for each conversion (flags, width, precision, I6… |
| `004e87fe` | WideStringFromMbs | likely | Cdecl (param_1 = out holder, param_2 = multibyte text, param_3 = byte count): converts the multibyte text to a wide string with _mbstowcs (size query then fill… |
| `004e89c3` | GetStaticEmptyWStringOnce | likely | On first call sets bit 0 of DAT_01bc7fbc, stores the empty wide sentinel DAT_01bc7e20 in DAT_01bc7fb8, registers LAB_017467cd with _atexit and returns the addr… |
| `004e8cbc` | WStringAssignFill | likely | Fills a wide-string holder with count copies of ch. If count is below 1 or equals the current length, memsets the existing characters (len*2 bytes; _memset wri… |
| `004e8d36` | WStringAssignCopy | likely | Copy-assigns this wide holder from the holder at *param_2 unless both are the same pointer: releases the old buffer unless it is the shared empty marker, point… |
| `004e8d8a` | WStringAssignFromNarrowHolder | likely | Converts a narrow count-prefixed holder (the holder pointer at *param_2 points to a count dword, chars at +4) into this wide holder: releases the old buffer un… |
| `004e8e04` | WStringAssignFromWideCStr | likely | Assigns this holder from a NUL-terminated wide string: the length is found by scanning for 0 (NULL gives 0), the old buffer is released unless it is the empty … |
| `004e8e78` | WStringAssignFromNarrowCStr | likely | Assigns this holder from a NUL-terminated char string: length by a strlen-style scan (NULL gives 0), releases the old buffer unless it is the empty marker, and… |
| `004e8ef9` | WStringAssignWChar | likely | Releases the old buffer unless it is the empty marker, then allocates a fixed 0x10-byte block via 004d9207 and stores length 1, the wide character at +4 and a … |
| `004e8f3d` | WStringAppendWString | likely | Appends the wide holder at *param_2 to this holder. If this holder is the empty marker it copy-assigns through 004e8d36. Otherwise, when the source length is n… |
| `004e8fb0` | WStringAppendWideCStr | likely | Appends a NUL-terminated wide string to this holder. If this holder is the empty marker it delegates to 004e8e04 (assign). Otherwise it scans the source for it… |
| `004e9030` | WStringAppendNarrowCStr | likely | Appends a NUL-terminated char string to this holder. If this holder is the empty marker it assigns through 004e8e78. Otherwise it builds a temporary wide holde… |
| `004e906e` | WStringAppendWChar | likely | Appends one wide character to this holder. If this holder is the empty marker it assigns through 004e8ef9. Otherwise it builds a one-character temporary holder… |
| `004e90dd` | WStringAssignSubstring | likely | Assigns a substring of the wide holder at *param_1 into the holder at *param_2. Returns 0 (after calling 004dc667) when start (param_3) is past the source leng… |
| `004e9262` | WStringAssignDecimalInt | likely | Formats a signed 32-bit value as base-10 text with __itow_s into a 64-wchar stack buffer, then assigns that buffer to this holder through 004e8c40 (length by s… |
| `004e92a0` | WStringAssignDecimalUInt | likely | Formats an unsigned 32-bit value as base-10 text with __ultow_s into a 64-wchar stack buffer, then assigns it to this holder through 004e8c40. On a conversion … |
| `004e92de` | WStringAssignDecimalInt64 | likely | Formats a signed 64-bit value (high dword param_3, low dword param_2) as base-10 text with __i64tow_s into a 64-wchar stack buffer, then assigns it to this hol… |
| `004e931f` | WStringAssignDecimalUInt64 | likely | Formats an unsigned 64-bit value (high dword param_3, low dword param_2) as base-10 text with __ui64tow_s into a 64-wchar stack buffer, then assigns it to this… |
| `004e9360` | WStringFormatV | likely | Formats a wide printf-style format (param_2) with a va_list (param_3) into this holder. A first pass over the format totals the output characters (it skips fla… |
| `004e9f24` | WStringScalarDeletingDtor | likely | Scalar deleting destructor shape: calls 004dc61b (releases the wide holder at this+0 unless it is the empty marker), then when bit 0 of the flag byte is set fr… |
| `0093c410` | CopyThenInitIString | likely | Copies the first dword from the source, initializes remaining destination IString fields, and returns the destination pointer. |
| `00a18f90` | DestroyMemberAtOffset8AndMaybeFree | likely | Calls 00703350 with ECX set to this+8; frees the outer this pointer only when deleting-flag bit 0 is set, otherwise returns this. |
| `00a19000` | LuaParamArrayAndIString_Dtor | likely | Calls the Lua-parameter array destructor 00a188c0 on this+0x18, writes the IString vftable at this+8, and when length at +0x14 is positive atomically decrement… |
| `00a1ad40` | DestroyLuaParamArrayAndIStringAndMaybeFree | likely | Calls 00a19000 on the incoming this pointer; frees that outer pointer only when deleting-flag bit 0 is set, otherwise returns this. |
| `00b70a00` | ReleaseEmbeddedIStringBuffer | likely | Resets the IString vtable at this+0x20; if the stored length at this+0x2c is positive, decrements the buffer refcount immediately before the pointer at this+0x… |
| `00bd5520` | HashList256_Insert | likely | Thiscall (ECX = list object): calls vtable slot [1] of the list (no real argument) and, when it returns nonzero, creates a node through 00bd54b0; appends the n… |
| `00c2f690` | CollapseAndResizeWideString | likely | Ensures room for the current wide text plus NUL, collapses whitespace in place through FUN_00c2dbb0, scans its terminator, and stores its character count. |
| `00c2f6f0` | LowercaseWideStringWithCapacity | likely | Ensures capacity for text plus NUL, then lowercases the wide string in place through FUN_00c2dc80. |
| `00c90910` | FormatValueAndReturnField8 | likely | Formats the dword at param_2[0] into the caller's string object and returns the dword stored at param_1+8. |
| `00cc56b0` | FormatIntegerIntoBuffer | likely | Passes param_1, the format global DAT_017703e0, and the integer at *param_2 to 0082ecd0, then returns the dword at param_1+8. |

### `engine.gfc.text`

**GFormattedString** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0049162f` | GFormattedString::AttachRangeFromBuffer | confirmed | Calls FUN_004e90dd(param_1,param_3,param_4), sets range start to param_3 and end to param_3+param_4 for nonnegative length or 0x7ffffffe for negative length, a… |

**GFormattedStringAccess** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0048f5ea` | GFormattedStringAccess::GFormattedStringAccess | confirmed | Tiny constructor that installs the string-access vtables: +4 gets GStringAccess::vftable first, then both +0 and +4 are set to GFormattedStringAccess::vftable,… |
| `0048f601` | GFormattedStringAccess::InvokeSlotEC | confirmed | Forwards to the receiver's virtual slot +0xEC with param_2 and the constant 1. |
| `0048f612` | GFormattedStringAccess::Wrapper6 | confirmed | Normal 24-byte call wrapper: calls FUN_004c97db with literal 0, three incoming arguments and trailing 0, then returns; x86 epilogue uses ret 0x0c. This is not … |
| `0048f62a` | GFormattedStringAccess::Wrapper7 | confirmed | Normal 24-byte call wrapper: calls FUN_004c980a with literal 0, three incoming arguments and trailing 0, then returns; x86 epilogue uses ret 0x0c. This is not … |
| `0048f642` | GFormattedStringAccess::Wrapper8 | confirmed | Normal 40-byte call wrapper: calls FUN_004c9839 with literal 0, four incoming arguments and trailing 0, then returns; x86 epilogue uses ret 0x10. This is not a… |
| `0048f67f` | GFormattedStringAccess::AdjustorThunkMinus4To0048f642 | confirmed | The 8-byte body subtracts 4 from ECX and jumps to 0048f642; it is an adjustor thunk, not a body call with unchanged arguments. |
| `0048f687` | GFormattedStringAccess::AdjustorThunkMinus4To0048f612 | confirmed | The 8-byte body subtracts 4 from ECX and jumps to 0048f612. |
| `0048f68f` | GFormattedStringAccess::AdjustorThunkMinus4To0048f717 | confirmed | The 8-byte body subtracts 4 from ECX and jumps to 0048f717. |
| `0048f697` | GFormattedStringAccess::AdjustorThunkMinus4To00482308 | confirmed | The 8-byte body subtracts 4 from ECX and jumps to 00482308. |
| `0048f69f` | GFormattedStringAccess::AdjustorThunkMinus4To0048f62a | confirmed | The 8-byte body subtracts 4 from ECX and jumps to 0048f62a. |
| `0048f6a7` | GFormattedStringAccess::AttachRangeChecked | confirmed | Validates the named range through slot +0x6c using param_2, start param_4 and length param_5. On success it attaches {start=param_4,end=param_4+param_5} for no… |
| `0048f717` | GFormattedStringAccess::ScalarDeletingDtor | confirmed | Scalar deleting destructor: writes GInterface::vftable to the object's two vtable positions and frees param_1 only when param_2 bit 0 is set; the non-free path… |

**(no class)** (35)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0048f307` | TextAccessClassDtor | confirmed | Destructor of the text-access class: installs the class vtables at +0/+4/+0x10, calls the held text interface's slot +0xD0 with the value at +0x44, runs FUN_00… |
| `0048f479` | TextAccessCopyCtor | confirmed | Copy constructor of the text-access class: installs the GPropertyInterface vtable, runs FUN_004c9e10, copies the storage header fields from param_2 (using para… |
| `0048f554` | TextAccessCtorFrom | confirmed | Third constructor emission of the text-access class taking a source object plus two extra arguments: installs the GPropertyInterface vtable, runs FUN_004c9e10,… |
| `0048ef78` | TextStorageAssignA | likely | Assigns a new text range: copies two dwords from param_2 into +0/+4, clears the 16-bit field at +0xE, runs FUN_004e78a7, stores param_3 at +8, releases the two… |
| `0048efc7` | TextStorageAssignB | likely | Ranged assignment variant: copies two dwords from param_4 into +0/+4, stores the 16-bit param_2 at +0xE, calls FUN_004f6424(param_3), clears +8 and releases an… |
| `0048f01f` | TextStorageSetPrimaryAccessor | likely | Copies two dwords from param_3 to +0/+4, clears the word at +0x0e, calls FUN_004e78a7 and clears +8. If param_2 is non-null it first calls its slot +0x0c to ac… |
| `0048f07f` | TextStorageSetSecondaryAccessor | likely | Copies two dwords from param_3 to +0/+4, clears the word at +0x0e, calls FUN_004e78a7 and clears +8. It releases and clears the old interface at +0x1c, acquire… |
| `0048f0df` | TextStorageReset | likely | Resets the text-storage object: +0=0, +4=0xFFFFFFFF, +8=0, 16-bit +0xE=0, runs FUN_004e78a7, releases and clears the two accessor interfaces at +0x1C and +0x20… |
| `0048f120` | TextStorageGetRange | likely | Calls receiver slot +0x10 with a local two-dword output. If that call returns zero, copies the local pair to param_2; if it returns nonzero, writes {0, 0xfffff… |
| `0048f1a1` | TextStorageCopyCtor | likely | Copy-constructs a text-storage object from param_2: installs the class vtables plus the global scratch-buffer pointer at +4, copies the source fields +0xC/+0x1… |
| `0048f20f` | TextStorageAssignFrom | likely | Assigns from another text-storage object: copies +8/+0xC, shares or duplicates the storage reference (FUN_004e8d36 when the source uses inline storage, otherwi… |
| `0048f2df` | ReleaseTwoInterfacesAndTailJump | likely | Releases non-null interface pointers at +0x20 and +0x1c through each pointer's slot +0x10(0), then tail-jumps to 004e78f5 with ECX set to this+0x10; raw bytes … |
| `0048f3e2` | TextAccessScalarDeletingDtor | likely | Scalar deleting destructor of the text-access class: runs the destructor FUN_0048f307 and, when the low bit of param_2 is set, frees the object through the thu… |
| `0048f65e` | CheckedIndexReturn | likely | Bounds-checked accessor: asserts (via the CRT helper FUN_00d83c2d) when the pointer at *param_1 is null, asserts again when the index param_1[1] is not below t… |
| `0048f768` | TextStorageDtor | likely | Calls FUN_0048f2df, then tail-jumps to 004dc61b. The 19-byte body does not call the second helper and return. |
| `0048f888` | TextAccessScalarDeletingDtor_2 | likely | Second emission of the text-access scalar deleting destructor: runs FUN_0048f307 and frees the object through the thunk FUN_0048207b when the low bit of param_… |
| `0048f945` | AddReturnedVectorToStoredOffsets | likely | If the pointer at +0x54 is null, writes {0,0} to param_2. Otherwise calls FUN_00482e29 with the value at +0x58 and writes the returned object's +4 field plus o… |
| `0048f9d1` | HeldObjectCtor | likely | Constructs a two-vtable holder from param_2: runs the base initialiser FUN_00450ab7, installs the class vtables PTR_FUN_0176a4f8/PTR_LAB_0176a4f4, acquires par… |
| `0048fa6d` | HeldObjectDtor | likely | Releases the non-null interface at +0x14 through slot +0x10(0), then tail-jumps to 00526562 with the base sub-object in ECX. |
| `0048fa86` | HeldObjectCtor_2 | likely | Second emission of the holder constructor: runs the base initialiser FUN_0047e32e, installs the class vtables PTR_FUN_0176a620/PTR_LAB_0176a618, acquires param… |
| `0048fbf6` | HeldObjectDtor_2 | likely | Second emission with the same body: releases the non-null interface at +0x14 through slot +0x10(0), then tail-jumps to 00526562. |
| `0048fc0f` | TextAccessPairCtor | likely | Constructs an object that embeds two accessor blocks: stores param_2/param_3 at +0/+4, binds the first block at +8 (fields +8/+0xC) and the second at +0x68 (fi… |
| `0048fc7d` | CopySelectedRecordFields0x7C | likely | Copies selected fields from param_2 to param_1: dwords at offsets 0..0x18, 0x20..0x30 and 0x38..0x54; low bytes at +0x1c and +0x34; bytes +0x58..+0x5b individu… |
| `0048fd5a` | TextStorageScalarDeletingDtor | likely | Scalar deleting destructor of the text-storage class: runs the destructor FUN_0048f768 and frees the object with _free when the low bit of param_2 is set; EH c… |
| `0048fdfa` | IndexInRangeWithPageCount | likely | Range predicate: computes the index at +0x18 plus one, or plus the count returned by FUN_00482da0 (called with ECX = param_1+0x1C) minus one when that helper r… |
| `0048feb9` | StateObjectCopyCtor | likely | Copy constructor of a two-vtable state object: installs the leading data pointer PTR_LAB_0175c908, runs the base initialiser FUN_004812a4(param_2 ? param_2+4 :… |
| `0048ff1c` | StateObjectDeletingDtor | likely | Scalar deleting destructor of the two-vtable state object: runs the destructor FUN_004813b4 and frees the object through the thunk FUN_0048132c when the low bi… |
| `0048ff38` | StateObjectCtor | likely | Default constructor of the two-vtable state object: installs the leading data pointer PTR_LAB_0175c908, runs FUN_004c9e10, installs the class vtables PTR_FUN_0… |
| `0048ffba` | ReplaceElement5Interface | likely | Reads the supplied interface pointer from the stack argument (Ghidra renders it as unaff_retaddr), looks up element 5 through receiver slot +0x60 and returns 0… |
| `00490012` | InitStateBlock60 | likely | Calls FUN_004dc667 twice, clears dwords at +0x08,+0x0c,+0x10..+0x28 and +0x30..+0x54, clears words at +0x5c/+0x5e, sets +0x2c to 1.0f and +0x58 to 0xffffffff. |
| `00491393` | HeldObjectCtorFrom | likely | Third emission of the two-vtable holder constructor: runs the base initialiser FUN_00481da2(param_2), installs the class vtables PTR_FUN_0176a620/PTR_LAB_0176a… |
| `004913c7` | TextAccessPairCopyCtor | likely | Copy constructor of the two-accessor object: copies the two leading dwords, binds the two embedded blocks at +8 and +0x68 to the global empty-string buffer DAT… |
| `00491554` | TextStorageScalarDeletingDtor_2 | likely | Second emission of the text-storage scalar deleting destructor: runs FUN_0048f768 and frees the object with _free when the low bit of param_2 is set. |
| `004915cb` | AttachBuiltTextStorage | likely | Builds the incoming interface's embedded part when non-null, releases the old interface through slot +0xd0 with handle +0x44, stores param_2 at +0x48, then eit… |
| `004e9c04` | TextWindowInitFromSource | likely | Initialises a text-window object from a source text-window (param_2). If the source's +8 points at its inline holder (+4) it sets this+8 to this+4 and copies t… |

### `engine.gfc.win32`

**(no class)** (49)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00446576` | RegisterInputAndWindowManagers | confirmed | Registers the two Win32 GFC managers: it interns the class name GInputManagerWin32, resolves L"1.0.0" and L"Win32 Input Manager" and calls FUN_00544cc0 with th… |
| `00825320` | CreateWindowManagerAndSystemWindow | confirmed | thiscall with eight stack arguments. Sets this+0x3ac to PTR_DAT_01b36310, calls 00823d10, clears the list at this+0x3f0 through 008249c0, stores this in the re… |
| `004450cf` | SetAttachedWindowAndAddRef | likely | Attaches a window/interface pointer: runs the record initialiser FUN_004400a7, clears this+0x10 and this+0x14, AddRefs param_2 through its virtual slot +0xC wh… |
| `0044581a` | BuildLimitsRecordCallVirtual70 | likely | Builds a 16-dword record on the stack (the caller's eight arguments in the leading slots, then the sentinel limits 0x7fffffff, 0x7fffffff, 0x80000000, 0x800000… |
| `004458e0` | CopyWindowRecord15Dwords | likely | Copies a window record: the dwords param_2[0..0xE] are written into param_1 (with the pairs at +4/+8 and +0x14/+0x18 staged through temporaries in the listing)… |
| `00445d5b` | AllocAndInitObject1763570 | likely | When the window is alive (byte +0x3c == 0) it allocates a 0x14-byte stamped block through FUN_00440ea1 and, when that succeeds, constructs the 01763570-class h… |
| `00445d80` | AllocAndInitObject1763408 | likely | Lazily creates and caches the 01763408 helper at this+0xEC: when the cache is empty it allocates 0x18 bytes through FUN_004d999e, constructs FUN_00445125 on th… |
| `00445e99` | CreateViaVirtual6C | likely | Creation helper guarded by the byte at this+0x39: it asks for a sub-object through virtual slot +0x68(0,2) and, when that fails, through virtual slot +0x64(0,2… |
| `0044728a` | WindowBaseCopyCtor | likely | Base copy constructor of the window objects: copies the leading four dwords with FUN_004452b1, copies the dword at param_2+0x10, AddRefs the interface at param… |
| `0044e1d3` | DetachAndReleaseField30 | likely | If the partner pointer at +0x94 is non-null it calls FUN_004453c1(0), clears that partner's +0xEC back-pointer when it still points at this object, and calls F… |
| `0044e221` | WindowManagerCopyCtor | likely | Copy constructor for the 0x80-byte window-manager class with vtables 01763d48/01763d44: runs the base constructor chain FUN_004474fc (which reaches the GWindow… |
| `0044e336` | InvokeSlot44ThenSlot20 | likely | Calls virtual slot +0x44 on the receiver as a null test, calls slot +0x44 again to fetch the subobject, then tail-jumps to slot +0x20 of that object so its res… |
| `0044e3e6` | CollectRectWithReducedEdges | likely | Collects the four dwords at param_3 into a local record, subtracts 1 from its third and fourth values, passes that record to FUN_0042c4f7, and returns 1. |
| `0044e41b` | EnumerateMonitorRectsForWindow | likely | Reads a rectangle through virtual slot +0x50; if SM_CMONITORS (0x4e) is nonzero, the monitor API probe succeeds, and EnumDisplayMonitors returns nonzero, retur… |
| `0044e661` | WindowWin32CopyCtor | likely | Copy constructor for the 0x110-byte window class with vtables 01763dd0/01763dcc: chains the base constructor FUN_004479e2 (vtable 01762d98 = GWindowWin32), ins… |
| `0044e86d` | InvokeSlot44ThenSlot20_2 | likely | Second emission of the forwarding helper: null test through slot +0x44, second call to fetch the subobject, then a tail jump to its slot +0x20; returns 0 when … |
| `0044eaa3` | Win32WindowProc | likely | On WM_NCCREATE (0x81), stores the object from CREATESTRUCT.lpCreateParams in GWL_USERDATA and records the HWND at +0x34. Otherwise it fetches that object, call… |
| `0044ee01` | DtorVtables1763F00 | likely | Destructor for the window-object class with vtables 01763f00/01763efc: rewrites both vtable pointers, calls FUN_0044b255 (destroy child windows), FUN_00447d92,… |
| `0044eec4` | AllocateAndRegisterIdEntry | likely | Returns null unless the +0xE4 flag is set; otherwise allocates a 0x14-byte record, stores param_2 at +0, param_3 at +4 and param_4 at +0x10, initializes +0x8/+… |
| `0044ef32` | WindowInitLateHandleEntries | likely | Creates four handle-table entries through FUN_0044eec4 with kind ids 1, 2, 5 and 4: the kind-1 entry gets this pointer at +0xC; the kind-2 entry's id (+8) is s… |
| `0044efb1` | CollectMatchingElementFieldPointers | likely | Returns 0 unless byte +0xE4 is set and param_2 (the output vector) is non-null. Walks the element range at +0x108/+0x10C; for kind 2, kind 6, or kind 4/5 with … |
| `0044f06c` | ClassOpsVtables1763BA0 | likely | Class-ops dispatcher for the class family with vtables 01763ba0: action 0 calls FUN_0044d1de on the instance, 1 constructs a copy through FUN_0044c949(param_3)… |
| `0044f0fd` | WindowHandleCtorVtables1763F70 | likely | Copy-style constructor for the 0x158-byte window class with vtables 01763f70/01763fdc: chains the base ctor FUN_0044a326 (which installs the PTR_FUN_01763790 v… |
| `0044f203` | InvokeSlot44ThenSlot20_3 | likely | Third emission of the forwarding helper: null test through slot +0x44, second call to fetch the subobject, then a tail jump to its slot +0x20; returns 0 when t… |
| `0044f222` | InvokeSlot44ThenSlot54 | likely | Calls slot +0x44 to fetch a subobject, then calls that object's slot +0x54 with param_2 and this+0x140; returns param_2. |
| `0044f252` | DtorVtables1763F70 | likely | Destructor body for the window class with vtables 01763f70/01763fdc: rewrites both vtable pointers, calls FUN_0044dc06, calls FUN_0044dfd0 on the sub-object at… |
| `0044f281` | CopyCtorVtables1762F68 | likely | Small constructor for the class with vtables 01762f68/01762f60 (0x34 bytes): chains the base constructor FUN_0042a8b7 with the prototype argument and installs … |
| `0044f325` | DtorVtables1763CD0 | likely | Destructor body for the window class with vtables 01763cd0/01763d40: rewrites both vtable pointers, calls FUN_0044e1d3, calls FUN_0044dfd0 on the sub-object at… |
| `0044f354` | DtorVtables1763D48 | likely | Destructor body for the window-manager class with vtables 01763d48/01763d44 (0x80 bytes): clears the global message-hook function pointer DAT_01bc44ac, rewrite… |
| `0044f59c` | WindowWin32CopyCtor_1763F00 | likely | Copy constructor for the 0x150-byte top-window class with vtables 01763f00/01763efc: chains FUN_004479fa/FUN_004420d3, installs both derived vtables, construct… |
| `0044f6c5` | ScalarDeletingDtor1763F00 | likely | Deleting destructor for the 01763f00 window class: calls the destructor body FUN_0044ee01 on the object and, when the low bit of the flag argument is set, free… |
| `0044f6e1` | WindowCtorVtables1763F00 | likely | Constructs a 0x150-byte object: calls base initializers FUN_004c6c1c and FUN_004c9e10, installs vtables 01762ec0/01762ebc then 01763f00/01763efc, initializes e… |
| `0044f7bc` | ClassOpsVtables1763F70 | likely | Class-ops dispatcher for the 01763f70 window class: action 1 runs the copy constructor FUN_0044f0fd(param_3) into param_2, action 3 allocates 0x158 bytes via F… |
| `0044f820` | ScalarDeletingDtor1763F70 | likely | Deleting destructor for the 01763f70 window class: calls the destructor body FUN_0044f252 and, when the flag's low bit is set, frees the object through the sha… |
| `0044f83c` | WindowCtorChild_1763F70 | likely | Constructor of the 0x158-byte child window object (vtables 01763f70/01763fdc) created by the child factory: sets up the base (FUN_004c6c1c plus vtables 0176379… |
| `0044f9b8` | ClassOpsVtables1762F68 | likely | Class-ops dispatcher for the 01762f68 small helper class: action 0 calls FUN_004433ab on the instance, 1 constructs through FUN_0044f281(param_3), 2 allocates … |
| `0044fa68` | ClassOpsVtables1763CD0 | likely | Class-ops dispatcher for the 01763cd0 window class: action 1 runs the copy constructor FUN_0044dfe8(param_3), action 3 allocates 0xA8 bytes via FUN_00440330 an… |
| `0044facc` | ScalarDeletingDtor1763CD0 | likely | Deleting destructor for the 01763cd0 window class: calls the destructor body FUN_0044f325 and frees the object through thunk_FUN_004c6a61 when the flag's low b… |
| `0044fae8` | WindowCtorChild_1763CD0_Kind5 | likely | Constructor of the 0xA8-byte window object for the child factory kind 5: base/vtable sequence (01763a40 then derived 01763cd0/01763d40), FUN_004450cf(0), insta… |
| `0044fb65` | WindowCtorChild_1763CD0_NativeB | likely | Variant constructor of the 01763cd0 window class used while creating the native window: base/vtable sequence, FUN_004450cf(0), PTR_LAB_01763530 pair, FUN_0044f… |
| `0044fbe6` | WindowCtorChild_1763CD0_NativeA | likely | Variant constructor of the 01763cd0 window class attached to the parent's +0x74 owner object: base/vtable sequence, FUN_004450cf(0), PTR_LAB_01763530 pair, FUN… |
| `0044fc64` | ScalarDeletingDtor1763D48 | likely | Deleting destructor for the 01763d48 window-manager class: calls the destructor body FUN_0044f354 and, when the flag's low bit is set, frees the object through… |
| `0045018a` | ClassOpsVtables1763DD0 | likely | Class-ops dispatcher for the 01763dd0 window class: action 0 constructs through FUN_0044e915, 1 runs the copy constructor FUN_0044e661(param_3), 2 allocates 0x… |
| `0045021b` | ScalarDeletingDtor1763DD0 | likely | Deleting destructor for the 01763dd0 window class: calls the destructor body FUN_0044f4ad and frees the object through thunk_FUN_004c6a61 when the flag's low b… |
| `00450237` | ClassOpsVtables1763F00 | likely | Class-ops dispatcher for the top-window class 01763f00: action 0 constructs through FUN_0044f6e1, 1 runs the copy constructor FUN_0044f59c(param_3), 2 allocate… |
| `004502c8` | WindowManagerCtor | likely | Constructor of the window-manager class (vtables 01763d48/01763d44): base chain FUN_004c6c1c, intermediate GWindowManagerWin32 vtable 01762c98, then the derive… |
| `0045035d` | WindowManagerCtorWithParent | likely | Second constructor of the 01763d48 window-manager class taking an existing window object: same base/intermediate/derived vtable sequence and member zeroing, th… |
| `004503b4` | WindowCreateChildByKind | likely | When byte +0xE4 is set, uses param_3 as the kind by default; only when param_2 is nonzero does it look up a record through FUN_0044aa94(param_2), reject null o… |
| `004504e9` | ClassOpsVtables1763D48 | likely | Class-ops dispatcher for the window-manager class 01763d48: action 0 constructs through FUN_004502c8, 1 runs the copy constructor FUN_0044e221(param_3), 2 allo… |

### `ui`

**BaseClient** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a77a0` | BaseClient::ShowZoneServerHostingStatus | confirmed | Writes a status line through the callback at (*(this+8))+0x14: param_3 selects the hosted or NOT hosted string. The negative branch passes literal 0 and a seco… |
| `008a7af0` | BaseClient::DispatchOnUpdateAction | confirmed | Frame-step trigger: calls the action dispatcher FUN_009797a0 with the name OnUpdate and two zero arguments, then calls FUN_0081f4f0 and FUN_0081e450 in that or… |

**GuiResetter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089aac0` | GuiResetter::~GuiResetter | confirmed | Trivial destructor body: writes the GuiResetter vtable pointer at +0 and returns; reached only from EH unwind funclets. |
| `0089aad0` | GuiResetterScalarDtor | confirmed | Scalar deleting destructor of GuiResetter: stores the class vtable at +0 and calls _free when bit 0 of param_2 is set. |

**GuiUserDataTable** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089cfd0` | GuiUserDataTable::~GuiUserDataTable | confirmed | Destructor body: writes the GuiUserDataTable vtable pointer at +0 and runs the cleanup FUN_010c8e90; reached only from EH unwind funclets. |

**HousingProcessor** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ab91b0` | HousingProcessor::NotifyNewHouse 🌐 | confirmed | Thiscall (this = HousingProcessor, body string pointer, length). Reads one string through FUN_00ab2f50 with bounds checks. If the script table at this+0x10 pro… |
| `00ab94b0` | HousingProcessor::SetIsInInstance | confirmed | Stores the first flag at this+0x14 and their conjunction at this+0x15. Builds a two-value inline Lua parameter array with FUN_0070ebe0, invokes FUN_009797a0 fo… |
| `00ab9860` | HousingProcessor::HandleMaxFixtures 🌐 | confirmed | Thiscall (this = HousingProcessor, body string pointer, length). Constructs a HousingPacketFixturePlacementDenied through FUN_00aad660, reads one string body t… |
| `00abaca0` | HousingProcessor::HandleInvitationDeclined 🌐 | confirmed | Reads two 8-byte pairs through FUN_00ab6bf0 (16 bytes total), with bounds checks; it does not parse a string body. After full consumption it looks up the Lua c… |

**(no class)** (23)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a6e90` | InitTooltipComponents | confirmed | One-shot component initialiser, guarded by this+0x3e0 being non-null: allocates and constructs 25 sub-objects (sizes 0x25c..0x670) through FUN_00d833a6 plus pe… |
| `008a7b30` | OptionsShowOrContextQuit | confirmed | Reads the ContextDepth value through FUN_00976630; when it is 0 it dispatches the action Options:Show and returns. Otherwise, when the object at this+0x3e0 exi… |
| `008a8bf0` | CallLua_RespawnWindow_Hide | confirmed | Loads the global UI/Lua context pointer DAT_01be1090 and calls the generic Lua-call bridge FUN_009797a0("RespawnWindow:Hide", 0, 0), i.e. invokes the UI method… |
| `00985200` | BossesDataSource_RegisterGuid | confirmed | Registers a GUID pair in the manager's 24-bucket boss table at [this+0x350] (buckets at +0x364, node link at +0x14, stored key at +0x10): computes the key as p… |
| `009852a0` | BossesDataSource_UnregisterGuid | confirmed | Removes a GUID pair from the manager's 24-bucket boss table at [this+0x350]: computes the key param_2 ^ param_3, walks the bucket chain (link +0x14, key +0x10)… |
| `00ab96e0` | CuteModeSetSizeScriptCall | confirmed | Chooses CuteMode:SetSizeInstant when param_2 is nonzero, otherwise CuteMode:SetSize, and assigns that name to a temporary IString. Constructs a one-inline Lua … |
| `00abc0d0` | ApplyHousingSearchReply | confirmed | Logs the reply entry count and current table size, then clears the existing result rows at this+0x1b4c using the count at this+0x1b54 through FUN_00ab28e0. For… |
| `008a7500` | InitComponentSubset | likely | Partial component initialiser: allocates and constructs nine sub-objects (sizes 0x268, 0x278, 0x278, 0x2b0, 0x3c, 0x254, 0x2ec, 0x670, 0x26c) via FUN_00d833a6 … |
| `008ba020` | ReleaseUiModuleReferences | likely | For each non-null UI module field in the listed offsets, calls virtual +0x28 with 0, notifies FUN_0081ad80, decrements its reference count and clears the field… |
| `00984f00` | GuiDataSourceCastCheck | likely | Returns 0 for a null pointer. Otherwise calls 00d8d382 to cast from GuiDataSource RTTI to BossesDataSource RTTI. If param_2 is nonzero and the cast fails, it o… |
| `00985190` | DataSourceManager_FindBossesByName | likely | Looks a data source up by name in the global data-source manager: hashes the name with 00705bc0, indexes the 1000-bucket table at [this+0xaea4] (chain link at … |
| `00b73d80` | MeasureMaxLineWidth | likely | Walks a NUL-terminated string, then accumulates 10 units per character, resetting the accumulator at each newline, and returns the maximum accumulated value (0… |
| `00c14e70` | GetLastClickedScreenLocation | likely | Lua C function bound as GetLastClickedScreenLocation: builds a stack Target object (Target::vftable), calls FUN_008a9100 with it and two integer out-parameters… |
| `00c14f20` | BindStringValue | likely | Lua C function bound as BindStringValue: requires exactly two arguments, reads both as strings (FUN_00e3a820) and, when the client field at +0x62c is non-zero,… |
| `00c14f70` | UnbindStringValue | likely | Lua C function bound as UnbindStringValue: requires exactly two arguments, reads both as strings and, when the client field at +0x62c is non-zero, calls FUN_00… |
| `00c15290` | GetCameraHeading | likely | Lua C function bound as GetCameraHeading: fetches the client singleton, calls virtual slot +0x84 of the object at client+0xec with a stack buffer to obtain a t… |
| `00c15380` | OnMiniGameFeedback | likely | Lua C function bound as OnMiniGameFeedback: when more than one argument is present it checks argument 1 with FUN_00e3a5f0 (numeric test) and returns 0; no othe… |
| `00c153b0` | OnBugReportUiSelectCategory | likely | Lua C function bound as OnBugReportUiSelectCategory: when at least one argument is present and argument 1 is numeric it reads it (FUN_00e3a770 + FUN_00d83c90) … |
| `00c15470` | SetCollectionFilterByCategory | likely | Lua C function bound as SetCollectionFilterByCategory: when at least one argument is present and argument 1 is numeric it reads it and calls FUN_00992e60(value… |
| `00c15c90` | SetHeadSize | likely | Lua C function bound as SetHeadSize: when exactly one argument is present it reads it as a number (FUN_00e3a770) and stores it as a float in the global DAT_01b… |
| `00ce43e0` | IconDataDerivedCtor | likely | Thiscall constructor with one optional pointer argument: installs two embedded SoeUtil::IString members at +0 and +0x10 set to the empty string, runs the IconD… |
| `00ce45e0` | IconDataDerivedCopyCtor | likely | Thiscall copy constructor with one pointer argument: installs the IString members at +0 and +0x10 (empty string, zero counts), copies the source string at +0 a… |
| `00ce4b60` | IconDataElemCreate | likely | Calls FUN_00ce49b0 with its argument to obtain the element at that index; when non-null it installs two IString members at +0 and +0x10 with the empty string, … |

### `ui.achievement`

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c09de0` | RegisterAchievementModule | confirmed | Registration wrapper: pushes nup=0, the 5-entry table 0x0182a5e0 and the 'Achievement' module-name pointer, calls FUN_00e39e40 and returns 0. |
| `00c09e00` | Achievement_SelectAchievement | likely | Lua 'SelectAchievement' handler: with one numeric argument, when the local player exists (client+0x3e0), it calls FUN_00a10ad0(id) on the achievement screen ob… |
| `00c09e50` | Achievement_SelectCategory | likely | Lua 'SelectCategory' handler: with one numeric argument, when the local player exists, it calls FUN_00a10ae0(category) on the achievement screen object; return… |
| `00c09ee0` | Achievement_SelectInspectAchievement | likely | Lua 'SelectInspectAchievement' handler: with one numeric argument it calls FUN_008a8d80(id) on the inspection screen object; returns 0. |
| `00c09f20` | Achievement_SelectInspectCategory | likely | Lua 'SelectInspectCategory' handler: with one numeric argument it calls FUN_008a8d60(category) on the inspection screen object; returns 0. |

### `ui.bugreport`

**UiBugManager::BugCategoryDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009f8950` | UiBugManager::BugCategoryDataSource::ColumnName | confirmed | Returns the constant label string Bug Category. |
| `009f93f0` | UiBugManager::BugCategoryDataSource::GetValue | confirmed | Returns the dword at offset +0x44 of the object referenced by this+0x224. |
| `009f9400` | UiBugManager::BugCategoryDataSource::SetValue | confirmed | Gets the selected row through a virtual call, resolves its category key with FUN_009f92c0, passes the result and supplied value to a manager virtual setter, an… |
| `009fc390` | UiBugManager::BugCategoryDataSource::Dtor | confirmed | Restores the data-source vtable at the primary and two embedded offsets, then destroys the embedded user-data table. |
| `009fc4c0` | UiBugManager::BugCategoryDataSource::Ctor | confirmed | Registers DesignTools.BugCategory, initializes the primary and two embedded data-source vtables, and stores the manager pointer at +0x254. |
| `009fc540` | UiBugManager::BugCategoryDataSource::ScalarDeletingDtor | confirmed | Calls the category data-source destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |

**UiBugManager::BugSeverityDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009f89b0` | UiBugManager::BugSeverityDataSource::ColumnName | confirmed | Returns the constant label string Bug Severity. |
| `009f89c0` | AdjustThisPlus4AndJump010C8F30 | confirmed | Adds 4 to ECX, then jumps to 010c8f30; the body contains no further call or field logic. |
| `009f9530` | UiBugManager::BugSeverityDataSource::SetValue | confirmed | Gets the selected row, searches the severity table at nested offset +0x80 for its key, passes the matching +4 value and supplied value to a manager virtual set… |
| `009fc450` | UiBugManager::BugSeverityDataSource::Dtor | confirmed | Restores the data-source vtable at the primary and two embedded offsets, then destroys the embedded user-data table. |
| `009fc600` | UiBugManager::BugSeverityDataSource::Ctor | confirmed | Registers DesignTools.BugSeverity, initializes the primary and two embedded data-source vtables, and stores the manager pointer at +0x254. |
| `009fc680` | UiBugManager::BugSeverityDataSource::ScalarDeletingDtor | confirmed | Calls the severity data-source destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |

**UiBugManager::BugSubCategoryDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009f8980` | UiBugManager::BugSubCategoryDataSource::ColumnName | confirmed | Returns the constant label string Bug Sub Category. |
| `009f9490` | UiBugManager::BugSubCategoryDataSource::GetValue | confirmed | Reads the selected key at nested offset +0xb4, searches the ten-bucket table at nested +0x48, and returns the matching record's +0x10 field or zero. |
| `009f94e0` | UiBugManager::BugSubCategoryDataSource::SetValue | confirmed | Gets the selected row, resolves its key through FUN_009f9330, passes the resolved value and supplied value to a manager virtual setter, and returns 1. |
| `009fc3f0` | UiBugManager::BugSubCategoryDataSource::Dtor | confirmed | Restores the data-source vtable at the primary and two embedded offsets, then destroys the embedded user-data table. |
| `009fc560` | UiBugManager::BugSubCategoryDataSource::Ctor | confirmed | Registers DesignTools.BugSubCategory, initializes the primary and two embedded data-source vtables, and stores the manager pointer at +0x254. |
| `009fc5e0` | UiBugManager::BugSubCategoryDataSource::ScalarDeletingDtor | confirmed | Calls the subcategory data-source destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009fa720` | AppendRelatedIdsForKey | confirmed | Finds the row for the supplied key, walks its integer list, and appends one formatted id=%d line per value to an IString. |
| `009fbb60` | LoadBugReportTables | confirmed | Loads category, category-mapping, and severity text files; parses caret-delimited rows into linked/hash tables, populates three data sources, and refreshes the… |
| `00c06140` | RegisterBugSubmissionModule | confirmed | Registration wrapper (same 29-byte pattern): pushes nup=0, the entry table 0x01829cd8 and the 'BugSubmission' module-name pointer, calls the Lua module registr… |
| `009fc150` | ReleaseThreeDataSourcesAndMaps | likely | Passes each DesignTools name to FUN_008b1f30 with zero; atomically decrements the referenced object counts at *(this+0xa8)+4, *(this+0xac)+4, and *(this+0xb0)+… |
| `009fc9c0` | SubmitBugReport | likely | Builds a BugSubmission data object, adds nonempty category/subcategory/severity fields, normalizes description bytes, adds PlayerData, and invokes gateway pack… |
| `009fcc30` | NotifyBugReportSubmitted | likely | Builds PlayerData and bug form values, invokes packet construction when enabled, displays Bug Form Submitted!, and closes Main.wndBugSubmission through its GUI… |

### `ui.claim`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a77680` | ClaimItemTooltip_BuildRows | confirmed | Method with this in ECX and an item-record argument: gates on the global client state (00769710 field +4 and +0x3e0), sets the row count to 8 through 00a76870 … |

### `ui.colors`

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb5410` | LoadUiColors | confirmed | Loader for UiColors.txt: builds the path from the argument (IString assign/append of the literal UiColors.txt), opens it with 00fbf4a0 and for every line not s… |
| `00bb5340` | HashListLinkTailMod10 | likely | Links a prepared node into a list+hash table: sets node+0xc (prev) to the current tail, node+8 (next) to 0, points the head at owner+8 or the old tail's +8 at … |
| `00bb5390` | HashListAppendAllocMod10 | likely | Allocates a node through the owner's virtual allocate slot (*(owner)+4), initialises it to {+0 = 0xffffffff, +4 = 0xffffff, key = *param_2 at +0x10} and links … |
| `00c207c0` | TintGroups_GetValue | likely | Reads an integer, obtains a global setting, validates the BaseClient.TintGroups data source, calls 00a734d0 on success, and pushes its result or -1. |

### `ui.commandhistory`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a9c420` | WriteCommandHistoryFile | confirmed | Reads the count at this+0x3c; when positive opens the file named by the string at PTR_s_CommandHistory_txt through 00703e10 with mode 3 and 1, then for each st… |

### `ui.controls`

**CursorManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a466c0` | CursorManager::~CursorManager | confirmed | Installs CursorManager::vftable at +0, the list vtable at +0x1c and drains that embedded list through 00a465f0; then installs the array vtable at +4, zeroes th… |
| `00a46ad0` | CursorManager::SetCurrentCursor | confirmed | Virtual slot [0] of CursorManager (ThisCall with an index): grows the array at this+4 to index+1 when the count is at or below the index (through 00a46110 when… |
| `00a46b30` | CursorManager::CursorManager | confirmed | Thiscall with the cursor-table constructor arguments: installs CursorManager::vftable at +0, the array vtable at +4 and the list vtable at +0x1c, zeroes the ar… |

**(no class)** (24)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ee600` | SetClickToMoveFlag | confirmed | Thiscall with a byte argument. When this+0x6d8 is nonzero it formats the byte with "%d" (00402fd0, format string bytes 25 64 = "%d") and logs it through 006fe5… |
| `00950e10` | KeybindingNameToId | confirmed | Stdcall with three args (ret 0xc): (context, name pointer, out pointer): clears the out dword and, when context and name are non-null and the name check FUN_00… |
| `00951270` | KeysymNameTableInitAndResolve | confirmed | Thiscall (ECX/arg1 = controller owner, arg2 = action index, arg3 = fallback, arg4 = out): on the first call registers 38 named keysyms through 009511f0 (TABKEY… |
| `00952120` | KeybindingRebindAction | confirmed | Cdecl-style (arg1 = group key, arg2 = value, arg3 = action name): maps the name to its index through 0094c760 and, when the row differs from (arg1, arg2) and t… |
| `009523b0` | KeybindingResetToDefault | confirmed | Cdecl-style (arg1 = action name): maps the name to its row index through 0094c760; when the cache at +0xc differs from the row name at +0, copies the row name … |
| `00952490` | KeybindingRefreshTwoActions | confirmed | Cdecl-style (two action names): maps each name to its row index through 0094c760; when both are known and the controls service exists, reads the cached key and… |
| `00950f40` | KeyCodeToDisplayName | likely | Thiscall (ECX = out IString, arg = key code): initialises the output IString to the empty buffer, looks up the key code in the 64-bucket table at 0x01b42284 (b… |
| `009519e0` | BindingEntryCachedLookup | likely | Cdecl-style (arg1 = action name, arg2 = out pointer): maps the name to a row index through 0094c760, returns the row value (+8) through the out pointer, and wh… |
| `00951a40` | KeybindingBindActionToGroup | likely | Cdecl-style with four args (group key, action name, kind, value): looks up the group node (0094d0f0) and the action index (0094c760, 0x51 rejects); through 007… |
| `00951b40` | KeyBindingEntryReapply | likely | Stdcall with one arg (ret 4; arg1 = action index): calls 00951560 with the stored name object, kind, index and value of row arg1 and both flags set to 1. |
| `00951b80` | KeyBindingTreeInsertEntry | likely | Stdcall with four args (ret 0x10; arg1 = key, arg2 = kind, arg3 = action index, arg4 = value): when kind is 0..2 builds a 10-dword payload (kind, value, action… |
| `00951bf0` | DefaultArrowKeyBindingsInstall | likely | Cdecl with no args: registers action rows 7, 8, 1 and 2 through 00951560 and inserts four tree records through 0094f5a0: keys 0xff51, 0xff53, 0xff52 and 0xff54… |
| `00951dc0` | RegisterBindingActions5And6 | likely | Cdecl with no args: calls 00951560 for action index 5 and for action index 6 (both flags set). |
| `00951e10` | RegisterBindingAction0x0e | likely | Cdecl with no args: calls 00951560 for action index 0x0e with both flags set. |
| `00951e50` | RegisterBindingAction0x26 | likely | Cdecl with no args: calls 00951560 for action index 0x26 with both flags set. |
| `00951e70` | RegisterBindingAction0x09 | likely | Cdecl with no args: calls 00951560 for action index 9 with both flags set. |
| `00951e90` | RegisterBindingAction0x0a | likely | Cdecl with no args: calls 00951560 for action index 10 with both flags set. |
| `00951eb0` | RegisterBindingAction0x27 | likely | Cdecl with no args: calls 00951560 for action index 0x27 with both flags set. |
| `00951ed0` | RegisterBindingAction0x25 | likely | Cdecl with no args: calls 00951560 for action index 0x25 with both flags set. |
| `00951ef0` | RegisterBindingActions1Cto21 | likely | Cdecl with no args: calls 00951560 six times for action indices 0x1c, 0x1d, 0x1e, 0x1f, 0x20 and 0x21, each with both flags set. |
| `00951fc0` | RegisterBindingActions22to24 | likely | Cdecl with no args: calls 00951560 for action indices 0x22, 0x23 and 0x24, each with both flags set. |
| `00952030` | RegisterSlotBindingsUpTo9 | likely | Stdcall with one count arg (ret 4): clamps the count to 9, then for each row from action index 0x13 upward resolves the row key through 00951270 into the row, … |
| `00952100` | RegisterBindingAction0x12 | likely | Cdecl with no args: calls 00951560 for action index 0x12 with both flags set. |
| `009522f0` | KeybindingEraseTwoAndReregister | likely | Cdecl-style with a guard arg, two action indices and two key-pair values (each a pair of args): when the guard is non-zero it evaluates two predicates (0094e0a… |

### `ui.core`

**NotificationImage** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bf6310` | NotificationImage::Ctor | confirmed | Constructor body. Stores AssetManagement::AssetRequestInterface::AssetRequester::vftable, then sets fields +0x4 = param5+0x1c4, +0x8 = arg2, +0xc = param5+0x26… |
| `00bf63d0` | NotificationImage::DtorBody | confirmed | Destructor body. Stores NotificationImage::vftable. When the byte at +0x29 is set, calls 00769710 and 00984960 on the pair at +0x18/+0x1c and, when the returne… |
| `00bf6680` | NotificationImage::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls 00bf63d0 (dtor body) on ECX, frees the object when bit 0 of the stack argument is set, and returns this (ret 4). |

**UI::FRTWGameScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013fed50` | UI::FRTWGameScreen::Ctor | confirmed | Fastcall constructor: calls base constructor 01511980, then stores UI::FRTWGameScreen::vftable at this. |
| `013fedb0` | UI::FRTWGameScreen::Dtor | confirmed | Fastcall destructor body: stores UI::FRTWGameScreen::vftable, then calls base destructor body 015119e0. |
| `013fee10` | UI::FRTWGameScreen::ScalarDeletingDtor | confirmed | Virtual slot 0 of UI::FRTWGameScreen::vftable: stores the vtable, calls base destructor body 015119e0, frees the object when the flag argument has bit 0 set, a… |

**UI::FRTWMenuScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013fe0a0` | UI::FRTWMenuScreen::Ctor | confirmed | Fastcall constructor: calls base constructor 01511870, then stores UI::FRTWMenuScreen::vftable at this. |
| `013fe100` | UI::FRTWMenuScreen::Dtor | confirmed | Fastcall destructor body: stores UI::FRTWMenuScreen::vftable, then calls base destructor body 015118d0. |
| `013fe150` | UI::FRTWMenuScreen::ScalarDeletingDtor | confirmed | Virtual slot 0 of UI::FRTWMenuScreen::vftable: stores the vtable, calls base destructor body 015118d0, frees the object when the flag argument has bit 0 set, a… |

**UI::FRTWUIService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01234d50` | UI::FRTWUIService::Ctor | confirmed | Constructor: calls 0123a270 (UIServiceBase constructor) on this, stores the UI::FRTWUIService vtable into the object, and returns this. |
| `01234db0` | UI::FRTWUIService::~FRTWUIService | confirmed | Destructor body: stores the UI::FRTWUIService vtable into the object, then calls 0123a140 (UIServiceBase destructor body). |
| `01234e10` | UI::FRTWUIService::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): stores the FRTWUIService vtable, calls 0123a140 (UIServiceBase destructor body), calls _free on this when bit 0 of … |

**UI::GameScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f7e80` | UI::GameScreen::GameScreen | confirmed | Constructor (fastcall, ECX = this): calls UI::FRTWGameScreen::Ctor 013fed50 (which stores its own vtable), then stores UI::GameScreen::vftable at *this and ret… |
| `012f7ee0` | UI::GameScreen::DtorBody | confirmed | Non-deleting destructor body: stores UI::GameScreen::vftable at *this, then calls 013fedb0 (UI::FRTWGameScreen::Dtor). |
| `012f7f30` | UI::GameScreen::~GameScreen | confirmed | Scalar deleting destructor (vtable slot [0]): stores UI::GameScreen::vftable, calls the destructor body 013fedb0, then calls _free on this when bit 0 of the fl… |

**UI::GameScreenBase** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015119e0` | UI::GameScreenBase::~GameScreenBase | confirmed | Stores UI::GameScreenBase::vftable into *this, then calls FUN_012f8480 (base destructor) and returns. |
| `01511a30` | UI::GameScreenBase::ScalarDeletingDtor | confirmed | Virtual slot 0 of UI::GameScreenBase. Stores the vtable, calls FUN_012f8480, then calls _free on this if flag bit 0 of the second argument is set. Returns this. |
| `01511a90` | UI::GameScreenBase::ExitMatch | confirmed | Virtual slot 2 of UI::GameScreenBase. Allocates 0x40 bytes with FUN_00d833a6; if the allocation is nonnull it assigns the string ExitMatch and calls FUN_012ec1… |
| `01511980` | UI::GameScreenBase::GameScreenBase | likely | Calls FUN_012f8420 on this, then stores UI::GameScreenBase::vftable into *this and returns this. |

**UI::MenuScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f7930` | UI::MenuScreen::MenuScreen | confirmed | Constructor (fastcall, ECX = this): calls 013fe0a0 (UI::FRTWMenuScreen::Ctor, which stores its own vtable first), then stores UI::MenuScreen::vftable at *this … |
| `012f7990` | UI::MenuScreen::DtorBody | confirmed | Non-deleting destructor body: stores UI::MenuScreen::vftable at *this, then calls 013fe100 (UI::FRTWMenuScreen::Dtor). |
| `012f79e0` | UI::MenuScreen::~MenuScreen | confirmed | Scalar deleting destructor (vtable slot [0]): stores UI::MenuScreen::vftable, calls the destructor body 013fe100, then calls _free on this when bit 0 of the fl… |

**UI::MenuScreenBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015118d0` | UI::MenuScreenBase::~MenuScreenBase | confirmed | Stores UI::MenuScreenBase::vftable into *this, then calls FUN_012f8480 (base destructor) and returns. |
| `01511920` | UI::MenuScreenBase::ScalarDeletingDtor | confirmed | Virtual slot 0 of UI::MenuScreenBase. Stores the vtable, calls FUN_012f8480 (base dtor), then if flag bit 0 of the second argument is set, calls _free on this.… |
| `01511870` | UI::MenuScreenBase::MenuScreenBase | likely | Calls FUN_012f8420 on this, then stores UI::MenuScreenBase::vftable into *this and returns this. |

**UI::ScreenBase** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f8420` | UI::ScreenBase::ScreenBase | confirmed | Base constructor (fastcall, ECX = this): stores UI::ScreenBase::vftable at *this, clears the byte at +4, sets the string capacity dword at +0x20 to 0xf and the… |
| `012f8480` | UI::ScreenBase::DtorBody | confirmed | Non-deleting destructor body: stores UI::ScreenBase::vftable at *this, frees the string buffer at +0xc when its capacity dword at +0x20 exceeds 15, then resets… |
| `012f84f0` | UI::ScreenBase::~ScreenBase | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 012f8480, then calls _free on this when bit 0 of the flag argument is set; returns this. |
| `012f8510` | UI::ScreenBase::ActivateNotify | likely | Virtual at vftable[1] of UI::ScreenBase (shared by the screen vtables of LoginScreen, MenuScreen, TradeLobbyScreen, TradeScreen and MatchmakingScreen): sets th… |

**UI::UIService** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f9ada0` | UIService::ScalarDeletingDtor | confirmed | Virtual slot 0 of UI::UIService: stores the vtable at +0, calls 01234db0 (FRTWUIService destructor body), then frees this through _free when bit 0 of the byte … |

**UI::UIServiceBase** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123a140` | UI::UIServiceBase::DtorBody | confirmed | Stores UI::UIServiceBase::vftable into the object, tears down the tree through FUN_0123a0d0, frees the buffer at +0x40 when non-null and zeroes +0x40..+0x48, c… |
| `0123a270` | UI::UIServiceBase::Ctor | confirmed | Calls FUN_01230d10 (base constructor stub), stores UI::UIServiceBase::vftable, initialises the string at +0x10..+0x24 to empty with capacity 15, zeroes the vec… |
| `0123a320` | UI::UIServiceBase::ScalarDeletingDtor | confirmed | Calls the destructor body FUN_0123a140 and, when bit 0 of the flag argument is set, frees the object with _free; returns the object pointer. |
| `01237f80` | UI::UIServiceBase::ForwardToChildren3 | likely | Virtual slot 3 shared by UIServiceBase, UIService and FRTWUIService: walks the pointer vector at this+0x40..+0x44 and, for each non-null element, calls its vir… |

**UIProcessor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a86fa0` | UIProcessorCreateObject0x34 | likely | UIProcessor vtable slot [3] factory: allocates 0x34 bytes through 00d833a6 (the call pushes only the size); on success sets ECX to the allocated object and cal… |

**Utility::XmlBuilder** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0130db10` | Utility::XmlBuilder::ScalarDeletingDtor | confirmed | Scalar deleting destructor (slot 0 of Utility::XmlBuilder::vftable): calls the destructor body 0130d9b0, frees the object through _free when flag bit 0 is set,… |
| `0130e6c0` | Utility::XmlBuilder::Ctor | confirmed | Fastcall constructor. Stores Utility::XmlBuilder::vftable at *this, constructs the basic_ostringstream member with 01271d00 (arguments 2 and 1), sets the resul… |

**(no class)** (36)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0071bcd0` | LookupLocalizationKey | confirmed | Constructs a temporary IString, passes the literal key Char_Unable_Log_Server_PS3 and its address to an interface method at vtable byte offset 0x10, then relea… |
| `0071bd80` | LookupLocalizationKey | confirmed | Constructs a temporary IString, passes the literal key Char_Unable_Log_Server_PS3 and its address to an interface method at vtable byte offset 0x10, then relea… |
| `008ff060` | SetLoadingWorldAssetsFlag | confirmed | thiscall with one byte argument (the new flag). If the stored byte at this+0x93 is zero and the new flag is nonzero: when this+0xec is set, calls 0094c8a0(0) a… |
| `00902320` | NotifyErrorMessage | confirmed | Builds a one-entry Lua argument array (FUN_0070ec40) holding param_1 through FUN_0070c580, calls the Lua function 'NotificationHandler:ErrorMessage' via FUN_00… |
| `009023f0` | NotificationErrorOrAlert | confirmed | Thiscall with stack args param_2 string, param_3 flag, param_4 flag, param_5 flag, param_6, param_7 (masked with 0xffffff). If param_5 is nonzero: builds a fou… |
| `009025a0` | ShowDisconnectedPopup | confirmed | Builds a Lua argument array with the FUN_0070ebe0 two-entry constructor and slot requests 0 to 3: slot 0 = param_1 and slot 1 = param_2 (FUN_0070c580), a tagge… |
| `009046e0` | HudSetIsInStartingZone | confirmed | Assigns the literal 'HUD:SetIsInStartingZone' to an IString (FUN_00402ce0), builds a one-entry Lua argument array (FUN_0070ec40, FUN_0070b470) holding the byte… |
| `00904ab0` | ShowTooltipFromCode | confirmed | Thiscall with one stack arg param_2. Gated on FUN_00a78e80(param_2). Builds a three-entry Lua argument array (FUN_0071cbc0) holding the constant 0xe, param_2 a… |
| `00904bf0` | HideTooltipFromCodeGated | confirmed | Fastcall on this. When the tag dword at (this+0x5ec)+0x280 equals 0xe, builds a one-entry Lua argument array (FUN_0070ec40) holding the dword at (this+0x5ec)+0… |
| `00904ce0` | ShowTooltipRowsFromCode | confirmed | Thiscall with three stack args (param_2, param_3 strings, param_4 value). Looks up an object with FUN_00984960 using the two strings; when found, calls FUN_00a… |
| `00904e70` | HideTooltipFromCode | confirmed | Fastcall on this. Builds a one-entry Lua argument array (FUN_0070ec40) holding the dword at (this+0x5ec)+0x288, calls 'TooltipMethods:HideFromCode' through FUN… |
| `00a19fc0` | ClientNameChangeStatusHandler | confirmed | Maps name-change status values to localized strings and Lua callback arguments; for success/reject/retry it builds a four-argument callback and invokes PaidNam… |
| `00a8eff0` | FlashTimerSetTimer | confirmed | Stores the current time plus param_2 seconds; if the timer is not already shown, sends param_2 times ten to FlashTimer:SetTimerDisplay, calls FlashTimer:Show, … |
| `00a95cd0` | UpdateFlashTimerAndClientUi | confirmed | When flag bit 8 is set, updates the timer display or emits ResetTimer and clears the flag at expiry; then runs three notification updates and other UI helpers. |
| `00c1e7f0` | UiLogValueWithCategoryLevel5 | confirmed | When the argument count is positive and argument 1 is present, uses Ui.log as the default category; if count exceeds one and argument 2 is present, uses argume… |
| `00c1e910` | UiLogValueWithCategoryLevel4 | confirmed | When the argument count is positive and argument 1 is present, uses Ui.log as the default category; if count exceeds one and argument 2 is present, uses argume… |
| `00c1ea30` | UiLogValueWithCategoryLevel3 | confirmed | When the argument count is positive and argument 1 is present, uses Ui.log as the default category; if count exceeds one and argument 2 is present, uses argume… |
| `00c1eb50` | UiLogValueWithCategoryLevel2 | confirmed | When the argument count is positive and argument 1 is present, uses Ui.log as the default category; if count exceeds one and argument 2 is present, uses argume… |
| `00c54f00` | ResolveSceneTextFlashControl | confirmed | Resolves Main.wndSceneText.swfSceneText, validates the result, casts it from GObject to GuiFlashCtrl, and stores the cast at this+0xf44; stores zero there when… |
| `004250ce` | ResetEntriesAndPruneByFlag | likely | Repeatedly scans the four-byte entry vector at +0x140 with end pointer +0x144. For every visited entry it first clears byte +0x2c. Only when byte +0x2d is zero… |
| `0042520f` | RemoveByKeyPair | likely | Scans the 0x30-stride element vector (count 0040a545, index 0040a57c) for an element whose first 16-bit word equals param_2 and whose +4 dword equals param_3; … |
| `0042528e` | RemoveAllByKey | likely | Scans the 0x30-stride element vector (0040a545/0040a57c) and erases every element whose +4 dword equals param_2, stepping the index back after each erase so th… |
| `004252f9` | DispatchInternalEvent | likely | Event dispatch on two internal event ids: for DAT_01bc2c8c it calls the node's virtual slot +0x114(param_2) and then removes that id's registration through 004… |
| `00904300` | DisplayRespawnWindow | likely | Thiscall with two stack args (param_2 = a millisecond value, param_3 = a value stored at +0x148). If param_2 < 1 it uses this+0x918. Formats the value divided … |
| `009d0fa0` | PositionGuiLabelAndReturnTextWidth | likely | Finds a named GUI object, casts it to GuiLabelCtrl, conditionally configures its text and visibility, measures a string, positions the label, and returns the m… |
| `009f8dc0` | FindGuiEditCtrlByName | likely | Resolves a named object, checks its runtime type, and returns it cast to GuiEditCtrl; returns zero when lookup or type check fails. |
| `009f8e10` | FindGuiWindowCtrlByName | likely | Resolves a named object, checks its runtime type, and returns it cast to GuiWindowCtrl; returns zero when lookup or type check fails. |
| `00f9cf10` | GetOrCreateUIService | likely | Fastcall registry accessor identical in structure to 00f9ce10 but keyed by DAT_01cbd728: creates UI::UIService through 00f9acf0 on first use, registers it, and… |
| `01238ae0` | RingStrQueueClearAndFree | likely | Clears a ring of string slots: while the count at +0x10 is non-zero it releases the last slot (index begin at +0xc plus count minus one, wrapped at capacity +8… |
| `0130dab0` | DequePushBackRecord | likely | Thiscall: grows the deque by one element through 0130d800(1) when the block map is full, computes the back slot (start plus count, wrapped), allocates a 0x1c-b… |
| `0130df10` | XmlWriteStartTag | likely | Thiscall: calls 0130db40 with the stream at this+4 and the arguments '<' (0x3c), the name param_2 and '>' (0x3e); then 0130dd10 on the result, then 0130db40 ag… |
| `0130e120` | XmlWriteEndTag | likely | Thiscall: copies the name of the innermost element (0130d560, deque back element) into a local string, pops that element with 0130d670, writes the name with 01… |
| `0130e200` | XmlBuilderFinish | likely | Fastcall: closes every open element by calling 0130e120 while the element count at this+0xb0 is non-zero, takes the built text from 0126f720, assigns it to the… |
| `0130e320` | XmlWriteElementText | likely | Thiscall: writes the start delimiter '<', the element name param_2 and '>' through 0130db40 on the stream at this+4, flushes through 0130dd10, appends the name… |
| `0130e750` | XmlWriteElementValueInt | likely | Thiscall: writes the start tag '<' name '>' through 0130db40 on the stream at this+4, flushes through 0130dd10, appends the name to the element deque with 0130… |
| `0130e7b0` | XmlWriteElementNumeric | likely | Thiscall: writes the start tag '<' name '>' as 0130e750 does, appends the name, writes param_3 through 01271b90 (numeric insert into the stream), then closes t… |

### `ui.data`

**GuiDataModuleContents** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010cc780` | GuiDataModuleContents::~GuiDataModuleContents | confirmed | Destructor body (ECX = object): restores the GuiDataModuleContents vtables at +0, +0x30 and +0x34, releases the watch-and-list object at +0x254 (010cc510 then … |
| `010cc920` | GuiDataModuleContents::GuiDataModuleContents | confirmed | Constructor (thiscall, one stack argument N): calls the base constructor 00922890 (GuiDsTable_Ctor, confirmed label) with N, installs the GuiDataModuleContents… |
| `010cca40` | GuiDataModuleContents::~GuiDataModuleContentsScalar | confirmed | Scalar deleting destructor (slot [0] of the primary GuiDataModuleContents vtable, flag on the stack): calls the destructor body 010cc780, then calls _free on t… |
| `010cbb70` | GetObjectNameLabel | likely | Virtual slot 1 of the secondary GuiDataModuleContents vtable (+0x30), no arguments: returns a pointer to the constant string Object name. |

**GuiDataModules** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010cd2c0` | GuiDataModules::GuiDataModules | confirmed | Constructor (thiscall, one stack argument N): calls the base constructor 00922890 (GuiDsTable_Ctor, confirmed label) with N, installs the GuiDataModules vtable… |
| `010cd3f0` | GuiDataModules::~GuiDataModules | confirmed | Destructor body (ECX = object): restores the GuiDataModules vtables at +0, +0x30 and +0x34, releases the watch-and-list object at +0x254 (010cd0a0, then _free)… |
| `010cd470` | GuiDataModules::~GuiDataModulesScalar | confirmed | Scalar deleting destructor (slot [0] of the primary GuiDataModules vtable, flag on the stack): calls the destructor body 010cd3f0, then calls _free on this whe… |
| `010cca80` | GetUiModuleNameLabel | likely | Virtual slot 1 of the secondary GuiDataModules vtable (+0x30), no arguments: returns a pointer to the constant string UI Module Name. |

**InGamePurchaseDataSource::CreditCardPayment** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0099f4d0` | CreditCardPayment::AdjustorThunk | confirmed | Adjusts ECX by -0x254 and jumps to 0099D680, a callback that appends payment entries. |

**InGamePurchaseDataSource::GiftOrderHistory** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0099f4c0` | GiftOrderHistory::AdjustorThunk | confirmed | Adjusts ECX by -0x254 and jumps to 0099D600, the gift-history row traversal routine. |

**InGamePurchaseDataSource::StoreBillboardPanels** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0099f4b0` | StoreBillboardPanels::AdjustorThunk | confirmed | Adjusts ECX by -0x254 then jumps directly to 0099D560; no panel processing body exists in this thunk. |

**InGamePurchaseDataSource::StoreBillboards** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0099f4a0` | StoreBillboards::AdjustorThunk | confirmed | Adjusts ECX by -0x254 and jumps directly to 0099D510, which performs the linked-list traversal. |

**InGamePurchaseDataSource::StoreBundleGroups** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0099f480` | StoreBundleGroups::AdjustorThunk | confirmed | Adjusts ECX by -0x254 and jumps directly to 0099D440; it contains no separate row-population body. |

**InGamePurchaseDataSource::StorePortalCategories** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0099f490` | StorePortalCategories::AdjustorThunk | confirmed | Adjusts ECX by -0x254 and jumps directly to 0099D4C0, a shared row-population body. |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a0690` | ValidateMerchantSellDataSource | confirmed | Checks RTTI compatibility against GuiDataSource and MerchantSellBundles, then formats a diagnostic when a non-null source has the wrong type. |
| `0099d440` | PopulateRowsFromNestedChain | likely | Clears rows through the callback at this+0x34 byte offset +0x0C, zeros the row count at +0x25C, follows the instance chain +0x258 -> +0x1C8 -> +0x3413C and nex… |
| `009b4440` | FindActionBarDataSourceByHash | likely | Hashes the argument, selects bucket hash modulo 1000, and follows next links until stored hash equals the computed hash. It returns null for a null value; othe… |
| `00a7e490` | SetField270AndRefresh | likely | Stores param_2 at byte offset +0x270 (the dword index param_1[0x9c]) and immediately calls the object's virtual slot at vtable offset +0x10 with no explicit ar… |

### `ui.datasource`

**AbilitiesDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c917a0` | AbilitiesDataSource::ColumnName | confirmed | Returns Ability ID for index zero, Tooltip Description for index one, and an error label for other indices. |
| `00c917d0` | AbilitiesDataSource::AdjustedVirtual | confirmed | Adds four to ECX and jumps to 010c8f30, forwarding this call through the adjusted base-subobject pointer. |
| `00c91e60` | AbilitiesDataSource::Dtor | confirmed | Restores AbilitiesDataSource vtables, clears the embedded integer hash map, and destroys the GUI user-data table. |
| `00c91ef0` | AbilitiesDataSource::Ctor | confirmed | Initializes the BaseClient.Abilities GUI binding, installs AbilitiesDataSource vtables, and initializes its embedded 40-bucket integer hash map. |
| `00c91f70` | AbilitiesDataSource::ScalarDeletingDtor | confirmed | Calls AbilitiesDataSource cleanup and frees the object only when the deleting flag's low bit is set. |

**AbilityLineDataSource** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8ee10` | AbilityLineDataSource::ColumnName | confirmed | Maps indices 0 through 10 to ability line ID, activation, name, description, icon, profile, experience, rank and level labels. |
| `00c8eeb0` | AbilityLineDataSource::OnRowsChanged | confirmed | If the source is not already dirty, calls a virtual callback through its adjusted owner pointer, then calls FUN_010c8f30. |
| `00c8eed0` | AbilityLineDataSource::VirtualDispatchAdjustor | confirmed | Loads the vtable function at offset 0x10, clears byte this+0x258, and jumps through the loaded function pointer. |
| `00c8eee0` | AbilityLineDataSource::ClearDirtyAndJump | confirmed | Clears byte this+0x258 then jumps to 0x010caf20. |
| `00c8eef0` | AbilityLineDataSource::RefreshRows | confirmed | Refreshes the UI list and traverses profile ability lines and their entries, emitting each available entry identifier. |
| `00c8efa0` | AbilityLineDataSource::GetColumnValue | confirmed | Searches ability rows by ID and returns row ID, activation state, localized name and description, icon, profile, experience, rank and required level fields. |
| `00c8f1a0` | AbilityLineDataSource::Dtor | confirmed | Restores AbilityLineDataSource vtables and destroys embedded GUI data source state. |
| `00c8f210` | AbilityLineDataSource::AbilityLineDataSource | confirmed | Registers BaseClient.AbilityLines, installs source vtables, stores its dependency and clears the initial dirty flag. |
| `00c8f290` | AbilityLineDataSource::ScalarDeletingDtor | confirmed | Calls the source destructor and frees the object when deleting flag bit 0 is set. |

**AcquaintanceDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c90830` | AcquaintanceDataSource::ColumnName | confirmed | Maps indices zero through four to Guid, Name, Acquaintance Type, Elapsed Time, and Online; other indices return an error label. |
| `00c90890` | AcquaintanceDataSource::OnUpdateAquaintancesList | confirmed | Calls FUN_009797a0 with the exact callback name SocialHandler:OnUpdateAquaintancesList and two zero arguments. |
| `00c908e0` | AcquaintanceDataSource::GetField | confirmed | Returns the dword at offset 0x0c from the object referenced by this+0x228. |
| `00c90930` | AcquaintanceDataSource::GetColumnValue | confirmed | Walks the acquaintance linked rows to the requested index and selects one of five fields, formatting or copying its value to the output. |
| `00c90a10` | AcquaintanceDataSource::Dtor | confirmed | Restores AcquaintanceDataSource vtables and destroys the embedded GUI user-data table. |
| `00c90a80` | AcquaintanceDataSource::Ctor | confirmed | Initializes the BaseClient.Acquaintances binding and vtables, then stores two supplied source values. |
| `00c90b00` | AcquaintanceDataSource::ScalarDeletingDtor | confirmed | Calls AcquaintanceDataSource cleanup, conditionally frees the object when the deleting flag is set, and otherwise returns it. |

**AdventurersJournalHubDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c977a0` | AdventurersJournalHubDataSource::ColumnName | confirmed | Maps indices zero through ten to hub fields including quest id, title, description, active and complete state, and map coordinates. |
| `00c97ea0` | AdventurersJournalHubDataSource::Refresh | confirmed | Rebuilds hub rows when the dirty byte is clear, then calls the shared data-source refresh helper. |
| `00c98170` | AdventurersJournalHubDataSource::GetFieldValue | confirmed | Looks up a hub row and returns its fields by column, including hub text, active quest title and description, completion description, and map coordinates. |
| `00c98980` | AdventurersJournalHubDataSource::Dtor | confirmed | Restores the hub data-source vtable pointers and invokes the shared embedded GUI data-table destructor. |
| `00c98b80` | AdventurersJournalHubDataSource::Ctor | confirmed | Initializes the named journal hub table, installs the hub source vtables, stores two source pointers, and clears its dirty flag. |
| `00c98c10` | AdventurersJournalHubDataSource::ScalarDeletingDtor | confirmed | Calls the hub data-source destructor, frees the instance when deleting flag bit one is set, and otherwise returns the original pointer. |
| `00c97c10` | AdventurersJournalHubDataSource::GetCount | likely | Walks the linked row entries and counts those whose key matches the active hub id, then returns the count. |

**AdventurersJournalMiscQuestDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c978c0` | AdventurersJournalMiscQuestDataSource::ColumnName | confirmed | Maps indices zero through twelve to miscellaneous quest fields, including quest identity and text, timestamps, progress, job data, objective count, membership,… |
| `00c97d70` | AdventurersJournalMiscQuestDataSource::GetCount | confirmed | Walks active quest records, counts records whose ids are absent from the hub quest map, and returns the count. |
| `00c97de0` | AdventurersJournalMiscQuestDataSource::Rebuild | confirmed | Refreshes the source, walks active native quest records, emits quest ids absent from the hub-quest map, marks the source built, and returns a pointer-like valu… |
| `00c97f00` | AdventurersJournalMiscQuestDataSource::Refresh | confirmed | Rebuilds miscellaneous quest rows when the dirty byte is clear, then calls the shared refresh helper. |
| `00c98a60` | AdventurersJournalMiscQuestDataSource::Dtor | confirmed | Restores miscellaneous quest data-source vtable pointers and invokes the shared embedded GUI data-table destructor. |
| `00c98ce0` | AdventurersJournalMiscQuestDataSource::Ctor | confirmed | Initializes the named journal miscellaneous quest table, installs its vtables, stores one source pointer, and clears the dirty flag. |
| `00c98d60` | AdventurersJournalMiscQuestDataSource::ScalarDeletingDtor | confirmed | Calls the miscellaneous quest source destructor, frees the object when deleting flag bit one is set, and otherwise returns the original pointer. |

**AdventurersJournalRegionDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c97710` | AdventurersJournalRegionDataSource::ColumnName | confirmed | Returns one of eight region fields by index: Region Id, Region Name, Region Description, Tab Image Id, Chapter Map Image Id, Is Selected, CompletedStringId, or… |
| `00c97e70` | AdventurersJournalRegionDataSource::Refresh | confirmed | Rebuilds the region rows when the dirty byte is clear, then calls the shared data-source refresh helper. |
| `00c98910` | AdventurersJournalRegionDataSource::Dtor | confirmed | Restores the region data-source vtable pointers and invokes the shared embedded GUI data-table destructor. |
| `00c98ad0` | AdventurersJournalRegionDataSource::Ctor | confirmed | Initializes the named journal region table, installs the region data-source vtables, stores two source pointers, and clears the dirty flag. |
| `00c98b60` | AdventurersJournalRegionDataSource::ScalarDeletingDtor | confirmed | Calls the region data-source destructor, frees the object when deleting flag bit one is set, and otherwise returns the pointer. |

**AdventurersJournalStickerDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c97850` | AdventurersJournalStickerDataSource::ColumnName | confirmed | Returns Sticker Id, Sticker Quest Id, Sticker Name, Sticker Description, Image Set Id, or Is Complete for indices zero through five. |
| `00c97cc0` | AdventurersJournalStickerDataSource::GetCount | confirmed | Walks sticker records and counts entries whose hub key matches the current hub, then returns the count. |
| `00c97ed0` | AdventurersJournalStickerDataSource::Refresh | confirmed | Rebuilds sticker rows when the dirty byte is clear, then calls the shared data-source refresh helper. |
| `00c989f0` | AdventurersJournalStickerDataSource::Dtor | confirmed | Restores sticker data-source vtable pointers and invokes the shared embedded GUI data-table destructor. |
| `00c98c30` | AdventurersJournalStickerDataSource::Ctor | confirmed | Initializes the named journal sticker table, installs sticker source vtables, stores two source pointers, and clears the dirty flag. |
| `00c98cc0` | AdventurersJournalStickerDataSource::ScalarDeletingDtor | confirmed | Calls the sticker source destructor, frees the instance when deleting flag bit one is set, and otherwise returns its pointer. |

**AllItemDefinitionsDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c974c0` | AllItemDefinitionsDataSource::Ctor | confirmed | Constructs the base item data source with BaseClient.AllItemDefinitions, installs derived vtables, and initializes the remaining state. |
| `00c97540` | AllItemDefinitionsDataSource::Dtor | confirmed | Restores derived vtable pointers, then invokes the shared base item data-source destructor. |
| `00c975a0` | AllItemDefinitionsDataSource::ScalarDeletingDtor | confirmed | Calls the all-items data-source destructor, frees the instance when deleting flag bit one is set, and otherwise returns the pointer. |

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00923050` | BaseClient_CreateAppUiDataSources | confirmed | Virtual slot [22] (shared by PlayClient and BaseClient): allocates a 0x48c object and builds a GuiPropertyDataSource named App.UI.SelectedControl.PropertyList … |

**BaseFilterItemDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cb1b10` | BaseFilterItemDataSource::Dtor | confirmed | Restores the class vtables at four subobjects, destroys the embedded ItemGuid array, then releases the embedded GUI user-data holder. |
| `00cb1bb0` | BaseFilterItemDataSource::Ctor | confirmed | Initializes the named GUI user-data holder and shared members, stores the source pointer, initializes filter state and vtables, and binds the Inventory table n… |
| `00cb1c80` | BaseFilterItemDataSource::ScalarDeletingDtor | confirmed | Calls the class destructor, frees the object when the deleting flag's low bit is set, and otherwise returns the original pointer. |

**BaseInventoryDataSource** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cb1340` | BaseInventoryDataSource::RebuildRows | confirmed | Collects qualifying records, selects a comparator based on the mode field, sorts a temporary list, copies record identifiers into the output array, updates cap… |
| `00cb1500` | BaseInventoryDataSource::GetCellText | confirmed | Switches on the requested column, reads item or client state, formats numbers, booleans and strings into the output value, and returns whether the column was h… |

**BaseItemDefinitionDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c956c0` | BaseItemDefinitionDataSource::GetFieldValue | confirmed | Switches on a column index, resolves item-related fields, formats values into the output string, and returns through the string provider. |
| `00c963b0` | BaseItemDefinitionDataSource::GetRow | confirmed | Refreshes if flagged, obtains the row key through a virtual call, searches the cached definition chain, requests a missing definition, then dispatches the row … |
| `00c96460` | BaseItemDefinitionDataSource::GetCount | confirmed | Calls the refresh helper when the dirty byte at this+0x5284 is set, then returns the 32-bit field at this+0x527c. |
| `00c968b0` | BaseItemDefinitionDataSource::Dtor | confirmed | Restores base and embedded data-source vtables, conditionally unregisters from shared state, destroys the filter map and embedded hashes, then releases holder … |
| `00c96a30` | BaseItemDefinitionDataSource::Ctor | confirmed | Initializes the base data-source object, hash-list and integer hash members, state flags, filter map, and registration in shared data-source state. |
| `00c96b40` | BaseItemDefinitionDataSource::ScalarDeletingDtor | confirmed | Calls the base data-source destructor, frees the object when the deleting flag bit is set, and otherwise returns its pointer. |

**BossesDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a79c80` | BossesDataSource::Slot1_ColumnName | confirmed | Virtual slot 1 of BossesDataSource (vtable 01816a3c): switch on the index argument returning a column name string: 0 Guid, 1 Name, 2 CurHealth, 3 MaxHealth, 4 … |
| `00a7a1d0` | BossesDataSource::DtorBody | confirmed | Destructor body: stores the BossesDataSource vtable at [this], +0x30 and +0x34 with the EH frame set up, then calls the base destructor 0091bf60 (Dtor_GuiUserD… |
| `00a7a250` | BossesDataSource::Ctor | confirmed | Constructor: calls the base constructor 00922890 with the BaseClient.Bosses name, stores the BossesDataSource vtable at [this], +0x30 and +0x34, stores its arg… |
| `00a7a2d0` | BossesDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (27 bytes, vtable slot 0): calls the destructor body 00a7a1d0 on this, then if bit 0 of the flag is set calls _free (thunk 00d82463)… |
| `00a79de0` | BossesDataSource::Slot2_CountMatching | likely | Virtual slot 2 of BossesDataSource (vtable 01816a3c): walks the list from 00984350 with 009849e0, looks up each node's key pair with 00984960, and counts the r… |

**BreadcrumbDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4fcd0` | BreadcrumbDataSource::FieldNameForIndex | confirmed | Maps index 0 to XCoordinate, 1 to YCoordinate, and 2 to ZCoordinate; other indices return "<error>". |
| `00c4fed0` | BreadcrumbDataSource::Dtor | confirmed | Restores the breadcrumb datasource vtables and destroys its GuiUserDataTable base subobject. |
| `00c4ff40` | BreadcrumbDataSource::BreadcrumbDataSource | confirmed | Initializes the base table, stores its two constructor arguments at this+0x254 and this+0x258, clears byte this+0x25c, and returns this. |
| `00c4ffd0` | BreadcrumbDataSource::ScalarDeletingDtor | confirmed | Calls the breadcrumb datasource destructor and frees this object only when flag bit 0 is set. |
| `00c4fd30` | BreadcrumbDataSource::RefreshRows | likely | Calls vtable slot 3 on the embedded row source, resets the row counter, requests each row by index, then marks the datasource populated. |

**CharacterCreateAccountInfoDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c2a0d0` | CharacterCreateAccountInfoDataSource::GetColumnName | confirmed | Virtual column-name accessor (vtable slot [1]): a 14-entry switch returns IsMember (0), MaxCharacters (1), IsAdminAccount (2), HeadshotUrl (3), CharacterGuid (… |
| `00c2a1d0` | CharacterCreateAccountInfoDataSource::GetStatusText | confirmed | Writes the current status dword (this+0x258) through the out pointer and switches on it: state 0 returns Invalid session token, state 2 returns Account has no … |
| `00c2a760` | CharacterCreateAccountInfoDataSource::CharacterCreateAccoun… | confirmed | Constructor with two arguments: registers the data-source name 'BaseClient.AccountInfo' through 00922890, installs the three vtable pointers, zeroes +0x254 and… |
| `00c2a7f0` | CharacterCreateAccountInfoDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c2a6f0 and frees the object with _free when the low bit of the flag argument is set. |
| `00c2a190` | CharacterCreateAccountInfoDataSource::Refresh | likely | Virtual override (framework vtable slot [4], default 010caf20): reads the session state field *(*(this+0x25c)+0x53d9c) into this+0x258, sets the state at this+… |
| `00c2a260` | CharacterCreateAccountInfoDataSource::GetRowCount | likely | Virtual method (vtable slot [2]): reads the client's account character count at *(this+0x22c)+0x22298 and walks the character array from +0x22290 (next entry a… |
| `00c2a2c0` | CharacterCreateAccountInfoDataSource::GetCellText | likely | Virtual method (vtable slot [4]) with signature (row, column[, out]): takes the account character count from vtable slot [2], walks the client's character list… |

**CharacterCreationCustomizationCategoryDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c2ad60` | CharacterCreationCustomizationCategoryDataSource::GetColumn… | confirmed | Virtual column-name accessor (vtable slot [1]): returns CategoryId (0), CategoryTitleId (1), DataSourceNameSuffix (2), DataSourceName (3, the DataSourceName li… |
| `00c2b130` | CustomizationCategoryDataSourceCtor | confirmed | Constructor of CharacterCreationCustomizationCategoryDataSource: registers the data-source name 'BaseClient.CharacterCustomizationCategory' through 00922890, i… |
| `00c2b1d0` | CharacterCreationCustomizationCategoryDataSource::ScalarDel… | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c2b090 and frees the object with _free when the low bit of the flag argument is set. |
| `00c2ae40` | CharacterCreationCustomizationCategoryDataSource::GetRowCou… | likely | Virtual method (vtable slot [2]): returns the dword at +0x21c of the object referenced by this+0x224 - the number of customization categories. Decoded bytes ar… |
| `00c2b090` | CustomizationCategoryDataSourceDtor | likely | Destructor body of CharacterCreationCustomizationCategoryDataSource: resets the three vtable pointers, releases the embedded SoeUtil::IString member at +0x258 … |

**CharacterCreationCustomizationDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c29800` | CharacterCreationCustomizationDataSource::GetColumnName | confirmed | Virtual column-name accessor (vtable slot [1]): a switch returns Guid (0), IconId (1), IconTint (2), ItemId (3), Race (4), Gender (5), Name (6) and Description… |
| `00c29c50` | CharacterCreationCustomizationDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c29b10 and frees the object with _free when the low bit of the flag argument is set. |
| `00c29880` | CharacterCreationCustomizationDataSource::GetRowCount | likely | Virtual method (vtable slot [2]): loads the row-list pointer from this+0x224 and the key from this+0x228 and tail-calls the hash-map lookup 0097d710, returning… |
| `00c29b10` | CharacterCreationCustomizationDataSource::~CharacterCreatio… | likely | Destructor body: resets the three vtable pointers (this+0, +0x30, +0x34), releases the embedded SoeUtil::IString at this+0x25c (checks the reference count at t… |
| `00c29bb0` | CharacterCreationCustomizationDataSource::CharacterCreation… | likely | Constructor with three arguments: registers a data-source name through 00922890(arg4), installs the three vtable pointers, stores arg2 at +0x254 and arg3 at +0… |

**CharacterCreationPresetDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c2b6c0` | CharacterCreationPresetDataSource::GetColumnName | confirmed | Virtual column-name accessor (vtable slot [1]): returns Id (0), Description (1), ModelId (2), ItemGroupId (3), Gender (4), HeadId (5), HairId (6), HairTintId (… |
| `00c2ba70` | CharacterCreationPresetDataSource::CharacterCreationPresetD… | confirmed | Constructor: registers the data-source name 'BaseClient.CharacterPresets' through 00922890, installs the three vtable pointers and stores the argument at +0x25… |
| `00c2baf0` | CharacterCreationPresetDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c2ba10 and frees the object with _free when the low bit of the flag argument is set. … |
| `00c2b790` | CharacterCreationPresetDataSource::GetCellText | likely | Virtual method (vtable slot [4]) with signature (row, column[, out]): resolves the preset record with 0097d990(row) and switches on the column - 0 formats the … |
| `00c2ba10` | CharacterCreationPresetDataSource::~CharacterCreationPreset… | likely | Destructor body: resets the three vtable pointers and calls the shared base teardown 0091bf60, then returns. |

**CharacterCreationStartingSetsDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c2a840` | CharacterCreationStartingSetsDataSource::GetColumnName | confirmed | Virtual column-name accessor (vtable slot [1]): a 14-entry switch returns Id (0), Description (1), RaceId (2), GenderId (3), ModelId (4), SkinToneId (5), HairS… |
| `00c2acb0` | CharacterCreationStartingSetsDataSource::CharacterCreationS… | confirmed | Constructor: registers the data-source name 'BaseClient.StartingSets' through 00922890, installs the three vtable pointers and stores the argument at +0x254, r… |
| `00c2ad30` | CharacterCreationStartingSetsDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c2ac40 and frees the object with _free when the low bit of the flag argument is set. |
| `00c2a820` | CharacterCreationStartingSetsDataSource::GetRowCount | likely | Virtual method (vtable slot [2]): loads the starting-sets table object from *(this+0x224)+0x2226c, adjusts this to that object's embedded sub-object at +0x10 a… |
| `00c2a900` | CharacterCreationStartingSetsDataSource::GetCellText | likely | Virtual method (vtable slot [4]) with signature (row, column[, out]): fetches the row record by calling vtable slot +0x44 of the embedded starting-sets table (… |

**CharacterDataSource** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bfca90` | CharacterDataSource::FieldNameForIndex | confirmed | Maps integer indices 0 through 0x1b to fixed field-name strings for identity, wallet, progression, membership, demographics, and social-profile entries; return… |

**CharacterNameCategoriesDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c2bb20` | CharacterNameCategoriesDataSource::GetColumnName | confirmed | Virtual column-name accessor (vtable slot [1]): returns CategoryId for column 0, NameCategory for column 1 (a pointer into the literal at 0x0182fed4), NameCate… |
| `00c2bec0` | CharacterNameCategoriesDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c2bd80 and frees the object with _free when the low bit of the flag argument is set. … |
| `00c2bb70` | CharacterNameCategoriesDataSource::GetCellText | likely | Virtual method (vtable slot [4]) with signature (row, column[, out]): stores a scratch record, resolves the category through 0097f8a0, then switches on the col… |
| `00c2bd80` | CharacterNameCategoriesDataSource::~CharacterNameCategories… | likely | Destructor body: resets the three vtable pointers, releases the embedded IString member at +0x258 (count at +0x264) and calls the shared base teardown 0091bf60… |

**CharacterNamesDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c2b200` | CharacterNamesDataSource::GetColumnName | confirmed | Virtual column-name accessor (vtable slot [1]): returns NameId for column 0 (a pointer into the literal at 0x0182fd14), Name for column 1, LocalizedName for co… |
| `00c2b690` | CharacterNamesDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c2b550 and frees the object with _free when the low bit of the flag argument is set. |
| `00c2b240` | CharacterNamesDataSource::GetRowCount | likely | Virtual method (vtable slot [2]): prepares a 52-byte scratch record through 0070bcc0(*(this+0x22c)), queries the count with 0097e590(scratch), releases with 00… |
| `00c2b2c0` | CharacterNamesDataSource::GetCellText | likely | Virtual method (vtable slot [4]) with signature (row, column, out[, extra]): builds a scratch record (00703540/00703350) and resolves the row through the name-… |
| `00c2b550` | CharacterNamesDataSource::~CharacterNamesDataSource | likely | Destructor body: resets the three vtable pointers, releases the embedded IString member at +0x258 (count at +0x264) and calls the shared base teardown 0091bf60… |

**CharacterStatsDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8e970` | CharacterStatsDataSource::ColumnName | confirmed | Returns Stat ID for column 0, Stat Value for column 1, and an error label for other indices. |
| `00c8eb30` | CharacterStatsDataSource::GetStatValue | confirmed | Resolves a character stat and returns its value field at +0x10, or zero if no matching entry exists. |
| `00c8ecf0` | CharacterStatsDataSource::Dtor | confirmed | Restores CharacterStatsDataSource vtables and destroys embedded GUI data source state. |
| `00c8ed60` | CharacterStatsDataSource::CharacterStatsDataSource | confirmed | Registers BaseClient.PlayerStats, installs source vtables, and stores the supplied source dependency. |
| `00c8ede0` | CharacterStatsDataSource::ScalarDeletingDtor | confirmed | Calls the source destructor and frees the object when deleting flag bit 0 is set. |

**CharacterTypesDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c294a0` | CharacterTypesDataSource::GetColumnName | confirmed | Virtual column-name accessor (primary vtable slot [1] of CharacterTypesDataSource): a switch over the column index returns the literals CharacterTypeId (0), Ch… |
| `00c29750` | CharacterTypesDataSource::CharacterTypesDataSource | confirmed | Constructor: registers the data-source name 'BaseClient.CharacterTypes' through 00922890, installs the three vtable pointers at this+0, this+0x30 and this+0x34… |
| `00c297d0` | CharacterTypesDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0] of the class): calls the destructor body 00c296f0 and, when the low bit of the flag argument is set, frees the obje… |
| `00c29560` | CharacterTypesDataSource::GetRowCount | likely | Virtual method (primary vtable slot [2]): returns the dword at offset +0xc of the row-list object held at this+0x224, i.e. the row count of the data source. |

**ClientActivityLaunchMembersDatasource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4e790` | ClientActivityLaunchMembersDatasource::FieldNameForIndex | confirmed | Returns one of six member-field names for indices 0 through 5; other indices return "<error>". |
| `00c4f950` | ClientActivityLaunchMembersDatasource::Dtor | confirmed | Restores the members datasource vtables, clears its embedded map, and destroys the GuiUserDataTable base subobject. |
| `00c4fa90` | ClientActivityLaunchMembersDatasource::ClientActivityLaunch… | confirmed | Initializes the base table and member HashList, stores its supplied datasource argument at this+0x368, and sets byte this+0x36c to 1. |
| `00c4fb20` | ClientActivityLaunchMembersDatasource::ScalarDeletingDtor | confirmed | Calls the members datasource destructor and frees this object only when flag bit 0 is set. |
| `00c4f3d0` | ClientActivityLaunchMembersDatasource::RefreshRows | likely | When the dirty byte is 1, clears the member map, walks request and member chains, rebuilds keyed rows, copies member fields, then clears the dirty byte. |

**ClientActivityLaunchRequestDatasource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4e6b0` | ClientActivityLaunchRequestDatasource::FieldNameForIndex | confirmed | Returns a field-name string for indices 0 through 14 and returns "<error>" for other indices. |
| `00c4f190` | ClientActivityLaunchRequestDatasource::GetCellValue | confirmed | Looks up the request row by its ID, then returns or formats the field selected by indices 0 through 14; unsupported fields use the error path. |
| `00c4f880` | ClientActivityLaunchRequestDatasource::Dtor | confirmed | Restores request-datasource vtables, clears the embedded member map and integer map, and destroys the GuiUserDataTable base subobject. |
| `00c4f9e0` | ClientActivityLaunchRequestDatasource::ClientActivityLaunch… | confirmed | Initializes the base table and embedded 64-bucket integer map, stores its supplied datasource argument at this+0x368, and returns this. |
| `00c4fa70` | ClientActivityLaunchRequestDatasource::ScalarDeletingDtor | confirmed | Calls the request datasource destructor and frees this object only when flag bit 0 is set. |

**ClientItemRootDataSource** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0098e920` | ClientItemRootDataSource::Constructor | confirmed | Calls the base data-source constructor with param_2, installs ClientItemRootDataSource's vtable, and returns this. |

**CollectionDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c31f00` | CollectionDataSourceFieldCount | confirmed | Calls FUN_00992cd0, adds nine, and returns the maximum field index used by the datasource. |
| `00c31f10` | CollectionDataSourceGetFieldName | confirmed | Maps field indices 0-8 to collection ID, name, description, image ID, two unused columns, entry count, image tint and category ID; later valid indices return E… |
| `00c326c0` | CollectionDataSourceGetFieldValue | confirmed | Refreshes rows if flagged, gets the requested collection row, then dispatches by field index to obtain localized name/description, image, counts, tint, categor… |
| `00c32dd0` | CollectionDataSourceDtor | confirmed | Restores CollectionDataSource vtables in its main and adjusted subobjects, then tears down the embedded GUI datasource base at +0x34. |
| `00c32ea0` | CollectionDataSourceCtor | confirmed | Registers BaseClient.CollectionData, installs the class vtables, stores two supplied datasource pointers, sets an enabled flag, and clears a state field. |
| `00c32f30` | CollectionDataSourceScalarDeletingDtor | confirmed | Calls CollectionDataSource cleanup and frees the object when flag bit 0 is set. |

**CollectionEntriesDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c31fd0` | CollectionEntriesDataSourceGetFieldName | confirmed | Returns one of seven field names: collection ID, entry ID, name, image-set ID, index, collected state, or tint value; otherwise returns <error>. |
| `00c322f0` | CollectionEntriesDataSourceGetCount | confirmed | If refresh flag +0x22c is set, calls the row-population routine; returns field +0xc from the object at +0x228 or zero. |
| `00c328d0` | CollectionEntriesDataSourceGetFieldValue | confirmed | Refreshes rows when flagged, resolves the requested entry row, and returns its name/localization, image set, index, collected state, tint, or other configured … |
| `00c32e30` | CollectionEntriesDataSourceDtor | confirmed | Restores CollectionEntriesDataSource vtables in its subobjects, then tears down the embedded GUI datasource base. |
| `00c32f50` | CollectionEntriesDataSourceCtor | confirmed | Registers BaseClient.CollectionEntriesData, installs class vtables, stores the configuration pointer, clears state, and sets an enabled flag. |
| `00c32fe0` | CollectionEntriesDataSourceScalarDeletingDtor | confirmed | Calls CollectionEntriesDataSource teardown and frees the object when flag bit 0 is set. |

**ConsumablesDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c31d10` | ConsumablesDataSourceCtor | confirmed | Registers BaseClient.Inventory.Consumables with the shared datasource constructor, then installs four ConsumablesDataSource vtables. |
| `00c31e40` | ConsumablesDataSourceDtorAdjustor | confirmed | Subtracts 0x254 from ECX then jumps to 00c31e50, adjusting this for the shared scalar-deleting destructor. |
| `00c31e50` | ConsumablesDataSourceScalarDeletingDtor | confirmed | Runs common datasource cleanup, then frees this object when flag bit 0 is set. |
| `00c31eb0` | ConsumablesDataSourceGetFieldValue | confirmed | For field 4 formats the datasource paging offset minus its page-size/count fields; other field IDs delegate to FUN_00cb1500. |
| `00c31de0` | ConsumablesDataSourceHasRows | likely | Checks global player and inventory state, a helper predicate, job value and positive inventory count, then compares this datasource's paging fields to decide w… |

**CurrentMountDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cc4190` | CurrentMountDataSource::~CurrentMountDataSource | confirmed | Installs the CurrentMountDataSource vtable at the object and two embedded subobjects, then destroys the GUI user-data member through 0091bf60. |
| `00cc42c0` | CurrentMountDataSource::CurrentMountDataSource | confirmed | Registers BaseClient.CurrentMount, installs the class vtable in three subobjects, stores param_2 at +0x254, and clears the byte at +0x258. |
| `00cc4340` | CurrentMountDataSource::ScalarDeletingDestructor | confirmed | Calls the CurrentMountDataSource destructor, frees this only when flag bit 0 is set, and returns this when it is retained and consumes the 4-byte delete-flag a… |

**CurrentPOIDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8e350` | CurrentPOIDataSource::ColumnName | confirmed | Maps indices 0, 1, and 2 to POIName, POIId, and AreaId; other indices return an error label. |
| `00c8e3c0` | CurrentPOIDataSource::GetColumnValue | confirmed | For column 0 reads the current POI name; columns 1 and 2 format the POI ID and area ID into the output value. |
| `00c8e490` | CurrentPOIDataSource::Dtor | confirmed | Restores source vtables, releases its narrow string buffer, and destroys embedded GUI data source state. |
| `00c8e550` | CurrentPOIDataSource::CurrentPOIDataSource | confirmed | Registers BaseClient.CurrentPOI, installs source and string vtables, initializes string storage, and clears current POI and area IDs. |
| `00c8e440` | CurrentPOIDataSource::SetCurrentPOI | likely | If either POI or area ID changed, updates the name string and two IDs, then notifies virtual slot 0x10. |
| `00c8e5f0` | CurrentPOIDataSource::ScalarDeletingDtor | likely | Calls CurrentPOIDataSource cleanup and frees the object when deleting flag bit 0 is set. |

**EquipScreenCurrentlyEquippedDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c2c750` | EquipScreenCurrentlyEquippedDataSource::EquipScreenCurrentl… | confirmed | Constructor with two arguments: calls the shared inventory-backed base constructor 00cb1bb0('BaseClient.Inventory.CurrentlyEquipped',arg3), installs the four v… |
| `00c2c890` | EquipScreenCurrentlyEquippedDataSource::AdjustorDtorThunk | confirmed | Adjustor thunk for the secondary vtable: subtracts 0x254 from ECX and jumps to the deleting destructor 00c2c8d0 (decoded bytes: sub ecx,0x254; jmp +0x35); the … |
| `00c2c8d0` | EquipScreenCurrentlyEquippedDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable 0183026c slot [0]): calls the destructor body 00c2c800 and frees the object with _free when the low bit of the flag argument… |
| `00c2c800` | EquipScreenCurrentlyEquippedDataSource::~EquipScreenCurrent… | likely | Destructor body: resets the four vtable pointers to EquipScreenCurrentlyEquippedDataSource::vftable and calls the shared base teardown 00cb1b10, then returns. |
| `00c2c860` | EquipScreenCurrentlyEquippedDataSource::SetRowId | likely | Setter: stores the argument at this+0x2e0 and calls virtual slot [6] (offset +0x18) of the object at this+0x254 to notify the data source. Its only caller is 0… |
| `00c2c9c0` | EquipScreenCurrentlyEquippedDataSource::GetCellText | likely | Virtual method (vtable slot [4]) with signature (row, column, out): looks the row up in the client item map (00769fe0 with the key at entry+4), requires the cu… |

**EquipScreenDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c2c620` | EquipScreenDataSource::EquipScreenDataSource | confirmed | Constructor with five arguments: calls the shared inventory-backed base constructor 00cb1bb0(arg4,arg3), installs the four vtable pointers (this+0, +0x30, +0x3… |
| `00c2c8a0` | EquipScreenDataSource::AdjustorDtorThunk | confirmed | Adjustor thunk for the secondary vtable: subtracts 0x254 from ECX and jumps to the deleting destructor 00c2c8b0 (decoded bytes: sub ecx,0x254; jmp +0x5); the d… |
| `00c2c8b0` | EquipScreenDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable 018301c4 slot [0]): calls the destructor body 00c2c6d0 and frees the object with _free when the low bit of the flag argument… |
| `00c2c6d0` | EquipScreenDataSource::~EquipScreenDataSource | likely | Destructor body: resets the four vtable pointers (this+0, +0x30, +0x34, +0x254) to EquipScreenDataSource::vftable and calls the shared base teardown 00cb1b10, … |
| `00c2c980` | EquipScreenDataSource::GetSpecialColumnText | likely | Virtual cell getter (vtable slot [16]) with signature (row record, column, out): for column 4 it formats the integer (record+0x14 - record+0x1c - record+0x18) … |

**ExpiredRentalsDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c30e10` | ExpiredRentalsDataSourceCtor | confirmed | Calls the RentalsDataSource constructor, then installs ExpiredRentalsDataSource vtables in four subobjects. |
| `00c30ea0` | ExpiredRentalsDataSourceDtor | confirmed | Restores ExpiredRentalsDataSource vtables at four subobject offsets, then calls RentalsDataSource teardown. |
| `00c30f20` | ExpiredRentalsDataSourceDtorAdjustor | confirmed | Subtracts 0x254 from ECX, then jumps to 00c30fa0; it is an adjustor thunk, not a separate destructor body. |
| `00c30fa0` | ExpiredRentalsDataSourceScalarDeletingDtor | confirmed | Runs expired-rentals teardown, then frees the object when flag bit 0 is set. |

**FactoryBlueprintDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c65230` | FactoryBlueprintDataSource::DtorBody | confirmed | Destructor body (no free). Resets vtables to 0183305c (primary), +0x30 to 01833038 and +0x34 to 01833020, then destroys the IntHashMap at this+0x2f0 through 00… |
| `00c652d0` | FactoryBlueprintDataSource::Ctor | confirmed | Constructor with one stack arg stored at this+0x254. Pushes the global name pointer at 01b717e8 (BaseClient.FactoryBlueprint) and calls the base constructor 00… |
| `00c65390` | FactoryBlueprintDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor in vftable slot [0] of 0183305c FactoryBlueprintDataSource::vftable: calls the body 00c65230, tests bit 0 of the flag argument and w… |
| `00c64d80` | FactoryBlueprintDataSource::FormatCellText | likely | Vftable slot [4] of 01833038 FactoryBlueprintDataSource::vftable: thiscall with a stack argument (row index). Calls 00c64ad0 first when byte +0x228 is clear, t… |

**FactoryInfoDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c65580` | FactoryInfoDataSource::DtorBody | confirmed | Destructor body (no free). Sets vtables 0183312c (primary), +0x30 to 01833108 and +0x34 to 018330f0, then calls the base destructor 0091bf60 on this. Returns w… |
| `00c65600` | FactoryInfoDataSource::Ctor | confirmed | Constructor with one stack arg stored at this+0x254. Pushes the global name pointer at 01b71970 (BaseClient.FactoryInfo) and calls the base constructor 0092289… |
| `00c65690` | FactoryInfoDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor in vftable slot [0] of 0183312c FactoryInfoDataSource::vftable: calls the body 00c65580, tests bit 0 of the flag argument and frees … |
| `00c653b0` | FactoryInfoDataSource::ColumnName | likely | Vftable slot [1] of 01833108 FactoryInfoDataSource::vftable: takes a column index on the stack and returns a static column-name pointer in EAX (ret 4): 0 Facto… |
| `00c65480` | FactoryInfoDataSource::FormatCellText | likely | Vftable slot [4] of 01833108 FactoryInfoDataSource::vftable: thiscall with a column index on the stack. First, if byte this+0x228 is clear, it runs the same re… |

**FriendsListDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cb2db0` | FriendsListDataSource::ColumnName | confirmed | Maps column numbers to friend name, coordinates, GUID, online state, profile values, encounter state, status, score and current minigame fields. |
| `00cb5140` | FriendsListDataSource::Dtor | confirmed | Restores data-source vtables, clears the row count, frees the pointer array with alignment 4, destroys the embedded hash list and GUI holder. |
| `00cb5210` | FriendsListDataSource::GetRowCount | confirmed | Returns the 32-bit value stored at this+0x238. |
| `00cb52c0` | FriendsListDataSource::Ctor | confirmed | Initializes the BaseClient.FriendsList holder, stores its source, installs class vtables, initializes the embedded hash list and row array, and clears count/di… |
| `00cb5370` | FriendsListDataSource::ScalarDeletingDtor | confirmed | Calls the friends data-source destructor, then frees the object when the low bit of the deleting flag is set. |

**GuiDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00822810` | GuiDataSource::DtorBody | likely | Sets GuiDataSource::vftable at this+0; calls 00822010 (drains the tree at this+0xc); when this+0x2c is positive, decrements the count at [this+0x24]-4 with a l… |
| `00822990` | GuiDataSource::Ctor | likely | thiscall with one argument (a C string). Calls 0071b950 (RefCounted base constructor), stores GuiDataSource::vftable at this+0, SoeUtil::Map<int,GuiDataSourceO… |
| `00822a30` | GuiDataSource::ScalarDeletingDtor | likely | Scalar deleting destructor shape: calls the base teardown 00822810 on this, then calls _free (thunk 00d82463) on this only when bit 0 of the flag argument is s… |

**GuiDataSourceObserver** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0081b470` | GuiDataSourceObserver::Ctor | confirmed | Fastcall ctor. Stores GuiDataSourceObserver::vftable at *this, zeroes this+4, stores the SoeUtil::IString<char> vtable at this+8 with the empty-string buffer D… |
| `0081b4f0` | GuiDataSourceObserver::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor shape at vtable slot 1 of GuiDataSourceObserver. Calls 00835610 on this (dtor body, not labelled here), then if the low bit… |
| `00835610` | GuiDataSourceObserver::DtorBody | confirmed | Destructor body. Stores the GuiDataSourceObserver vtable at this, clears the observed object at +0x18 and calls its vtable slot 3 (+0xc) with this, removes thi… |
| `00835560` | GuiDataSourceObserver::GetField4 | likely | Thiscall with no arguments. Returns the dword at this+4. |
| `00835570` | GuiDataSourceObserver::TestField18NonNull | likely | Thiscall with no arguments. Returns true when the pointer at this+0x18 is nonzero (setne on the compare of [ecx+0x18] with 0). |
| `00835580` | GuiDataSourceObserver::SetField18Ref | likely | Thiscall with one stack argument (new observed object). When the argument differs from the pointer at this+0x18, it calls this's vtable slot 5 (+0x14) with the… |
| `008355b0` | GuiDataSourceObserver::ClearField18AndForward | likely | Fastcall (ECX = this). When the observed object at this+0x18 is nonzero it clears that field and tail-jumps to the object's vtable slot 3 (+0xc) with this as t… |
| `00837550` | GuiDataSourceObserver::CopyCtor | likely | Thiscall with one stack argument (source observer). Stores the GuiDataSourceObserver vtable at +0, copies the source dword at +4, stores the IString<char> vtab… |

**GuiDsTable** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00922890` | GuiDsTable_Ctor | confirmed | Base constructor: calls 00822990 with its name argument, stores the IDataAccessTable and GuiUserDataTable vtables at this+0x30/+0x34, zeroes +0x38, stores the … |
| `00922940` | GuiDsTable_ScalarDtor | likely | Scalar deleting destructor: calls the body 0091bf60 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
| `010cb300` | PopulateStringKeyIndex | likely | Virtual slot 13 of GuiDsTable (vtable records list the same body in its sibling classes), ECX = this. For index i = 0, 1, ... it calls slot 1 (+4) of the objec… |

**GuiDsTableView** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010d4df0` | GuiDsTableView::~GuiDsTableView | confirmed | Destructor body. Stores GuiDsTableView::vftable at this, +0x30, +0x34 and +0x254, releases the IString<unsigned short> at +0x2c8 when its count is positive, fr… |
| `010d4f60` | GuiDsTableView::GuiDsTableView | confirmed | Thiscall constructor. Calls the base constructor 00922890 (GuiDsTable_Ctor) with the name argument, stores GuiDsTableView::vftable at this, +0x30, +0x34 and +0… |
| `010d4c10` | GuiDsTableView::CachedCount | likely | Thiscall virtual on the GuiDsTableView secondary subobject (+0x30). Calls the refresh routine 010d4a70 on this-0x30 when the byte at +0x294 is set, then return… |
| `010d50c0` | GuiDsTableView::ScalarDeletingDtor | likely | Scalar deleting destructor. Calls the destructor body 010d4df0 on this, then when bit 0 of the argument is set calls the _free thunk 00d82463 on the object, an… |

**GuiDsTree** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0099a630` | GuiDsTreeScalarDeletingDtor | confirmed | Thiscall (ECX=this) with one stack arg (byte flags). Calls 00822810 (base teardown, ECX=this per the pseudocode; cookie argument artifact) and then, if bit 0 o… |
| `009974a0` | GuiDsTree::Ctor | likely | Thiscall (ECX=this) with one stack arg param_2, returns this (ret 4). Calls 00822990 with param_2 pushed, stores the GuiDsTree vtable (0180d2b8) at [this], zer… |

**GuiDsValue** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0081b510` | GuiDsValue::Ctor | confirmed | Thiscall ctor with one stack argument (a C string name). Calls 00822990 (GuiDataSource base ctor) with the argument, then 0081b470 (observer base ctor) on the … |
| `0081b5e0` | GuiDsValue::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor, vtable slot 0 of GuiDsValue. Calls 008219b0 on this (the GuiDsValue dtor body, not labelled here), then if the low bit of … |
| `0081b600` | GuiDsValue::AdjustorThunk_ScalarDtor | confirmed | Thunk for the secondary vtable at +0x30: subtracts 0x30 from ECX and jumps to 0081b5e0 (the GuiDsValue scalar deleting destructor), so the flag argument passes… |
| `008219b0` | GuiDsValue::~GuiDsValue | likely | Destructor body: stores GuiDsValue::vftable at this+0 and this+0x30, releases the owned object at +0x50 with the same logic as 00821940 (vtable slot 0 with arg… |
| `00821b30` | GuiDsValue::SetStringValue | likely | For a non-NULL string argument, when the flag argument is 0 or this+0x48 is 0: if (this+0x4c)&6 == 6 it calls 00738ae0 with the held string data pointer and th… |
| `00821ba0` | GuiDsValue::RefreshFromSource | likely | Reads the source at this+0x18; if it dynamic-casts to GuiDsValue (00d8d382 with GuiDataSource and GuiDsValue descriptors) it copies the source value by its fla… |

**GuiPropertyDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00922a10` | GuiPropertyDataSource_Ctor | confirmed | Constructor: calls the GuiDsTable base constructor 00922890 on this with its name argument, stores the three GuiPropertyDataSource vtables at this, +0x30 and +… |
| `00922b00` | GuiPropertyDataSource_ScalarDtor | confirmed | Scalar deleting destructor: calls the body 00922aa0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
| `00922aa0` | GuiPropertyDataSource_DtorBody | likely | Destructor body: calls 008ebca0 on the sub-object at this+0x254, then calls 0091bf60 on this. |

**GuiUserDataTable** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010c8e90` | ClearOwnedObject | confirmed | Virtual slot 3: if this+4 holds an object, calls that object's vtable slot 0 with argument 1 (scalar deleting destructor, which frees it), then sets this+4 to … |
| `010c8f80` | RemoveFirstIntValue | confirmed | Virtual slot 2: when this+4 holds an array, scans its int elements for the value passed in the stack argument; on the first match it shifts the later elements … |
| `010c8fe0` | SetIntAt | confirmed | Virtual slot 0 (an ordinary virtual, not a deleting destructor: it never frees this). Creates the int array at this+4 through 010c8ec0 when it is null, grows i… |
| `010c9050` | AppendIntValue | confirmed | Virtual slot 1: creates the int array at this+4 through 010c8ec0 when it is null, then appends the stack-argument value at index count (count is raised to coun… |

**GuildDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cb53f0` | GuildDataSource::ColumnName | confirmed | Returns column labels for guild GUID, name, message of the day, and guild creation/renaming permissions, with an error label for other indices. |
| `00cb6120` | GuildDataSource::Ctor | confirmed | Initializes the BaseClient.Guilds holder, installs GuildDataSource vtables, and stores the two supplied source arguments. |
| `00cb6200` | GuildDataSource::ScalarDeletingDtor | confirmed | Destroys the embedded GUI data holder and frees the object when the deleting flag's low bit is set. |
| `00cb5710` | GuildDataSource::GetRowCount | likely | Returns the field at offset +0x9c from the object referenced by this+0x228. |
| `00cb5e30` | GuildDataSource::GetCellValue | likely | Formats guild GUID, name, message of the day, and one permission field into the requested output; returns false if the referenced row is absent or the column i… |

**GuildMemberDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cb5450` | GuildMemberDataSource::ColumnName | confirmed | Maps column numbers to guild member identity, rank, online state, profile values, encounter state, location and friend status labels. |
| `00cb5720` | GuildMemberDataSource::GetRowCount | confirmed | Returns zero if this+0x228 is null; otherwise returns the pointed-to field at offset +0xa0. |
| `00cb6260` | GuildMemberDataSource::Ctor | confirmed | Initializes the BaseClient.GuildMembers holder, installs class vtables, stores the source, and initializes the embedded character-GUID array and dirty state. |
| `00cb6320` | GuildMemberDataSource::Dtor | confirmed | Clears the GUID array count, restores its array vtable, frees aligned storage, and destroys the embedded GUI user-data holder. |
| `00cb63b0` | GuildMemberDataSource::ScalarDeletingDtor | confirmed | Calls the guild-member data-source destructor and frees the object when the deleting flag's low bit is set. |
| `00cb65f0` | GuildMemberDataSource::GetCellValue | likely | Validates and resolves a row's GUID, then maps column indices to identity, profile, member state, rank and permission values, including comparisons with curren… |

**InteractionDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c91f90` | InteractionDataSource::ColumnName | confirmed | Maps indices zero through eight to Id, Title, IconId, DefinitionId, Type, param1, param2, TooltipId, and Tooltip; invalid indices return an error label. |
| `00c92700` | InteractionDataSource::Ctor | confirmed | Initializes the base GUI data-source object, installs InteractionDataSource vtables and its entry-array vtable, and stores supplied source values. |
| `00c927c0` | InteractionDataSource::Dtor | confirmed | Restores the embedded array vtable, destroys all interaction entries, frees the backing buffer, and destroys the embedded GUI user-data table. |
| `00c92850` | InteractionDataSource::ScalarDeletingDtor | confirmed | Calls InteractionDataSource destruction and frees the object only when the deleting flag's low bit is set. |

**ItemGroupItemsDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c96b70` | ItemGroupItemsDataSource::ColumnName | confirmed | Maps column indices zero through four to GroupId, ItemId, ItemCount, IsEquipped, and IsMarketPlaceItem; other indices return <error>. |
| `00c96dc0` | ItemGroupItemsDataSource::Refresh | confirmed | Rebuilds the selected group's rows when the dirty byte is clear, then calls a second cleanup or refresh helper. |
| `00c97230` | ItemGroupItemsDataSource::Ctor | confirmed | Initializes the named source table, clears state, installs ItemGroupItemsDataSource vtables, stores the source argument, and copies its field at +0x20c. |
| `00c97320` | ItemGroupItemsDataSource::ScalarDeletingDtor | confirmed | Calls the embedded data-source destructor, frees the instance when deleting flag bit one is set, and otherwise returns the original pointer. |
| `00c96de0` | ItemGroupItemsDataSource::SetGroupId | likely | Stores the supplied value at this+0x254 and rebuilds the data source. |

**ItemGroupsDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c96be0` | ItemGroupsDataSource::ColumnName | confirmed | Maps indices zero through nine to item group fields including ID, Name, Description, SortOrder, Items, IconId, CategoryId, PageId, NumberOfItemsPlayerHas, and … |
| `00c96d80` | ItemGroupsDataSource::Refresh | confirmed | Rebuilds rows when the dirty byte is clear, then calls a second cleanup or refresh helper and returns. |
| `00c97380` | ItemGroupsDataSource::Ctor | confirmed | Initializes the inherited data-source base, installs ItemGroupsDataSource vtables, clears source state, and stores two constructor arguments. |
| `00c97460` | ItemGroupsDataSource::ScalarDeletingDtor | confirmed | Calls a destructor helper, frees the instance when deleting flag bit one is set, and otherwise returns the original pointer. |

**ItemSearchEngine** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c2cfa0` | ItemSearchEngine::ItemSearchEngine | confirmed | Constructor: calls the base constructor 00c30040, then installs the ItemSearchEngine vftable at this+0 and returns this. |
| `00c2d060` | ItemSearchEngine::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0] of 0x01830478): resets the vtable, calls the member teardown 00c2fa70 and frees the object with _free when the low … |
| `00c2d000` | ItemSearchEngine::~ItemSearchEngine | likely | Destructor body: resets the ItemSearchEngine vftable and calls 00c2fa70 (the engine's member teardown), then returns. The export additionally lists it as a cal… |
| `00c2d100` | ItemSearchEngine::GetEntryValue | likely | Virtual method (vtable slot [2]) with signature (id, out): performs the same item-table hash lookup as 00c2d0c0 and, when the entry exists, writes *(entry+0x28… |
| `00c2d150` | ItemSearchEngine::NotifyEntry | likely | Virtual method (vtable slot [3]) with signature (id, value): performs the item-table hash lookup and, when the entry exists, calls virtual slot [2] (offset +8)… |

**ItemStatsDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c31180` | ItemStatsDataSourceGetCount | confirmed | Returns the dword at this+0x224c. |
| `00c31820` | ItemStatsDataSourceDtor | confirmed | Restores ItemStatsDataSource vtables, destroys its integer array and hash map, releases a shared IString buffer with reference counting, then tears down the GU… |
| `00c31a00` | ItemStatsDataSourceScalarDeletingDtor | confirmed | Calls ItemStatsDataSource teardown, then frees the object when flag bit 0 is set. |

**MerchantBuyInventoryDataSource** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cfd8c0` | MerchantBuyInventoryDataSource::MerchantBuyInventoryDataSou… | confirmed | Initializes the shared inventory data-source base with BaseClient.MerchantBuyInventoryData, installs the derived vtables, and stores the supplied owner pointer. |
| `00cfddc0` | MerchantBuyInventoryDataSource::ScalarDeletingDtor | confirmed | Runs the shared BaseFilterItemDataSource destructor and frees this object only when deleting-flag bit 0 is set; otherwise returns the original pointer. |

**MerchantSellInventoryDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cfec80` | MerchantSellInventoryDataSource::RebuildRowOrder | confirmed | Builds a temporary list from the current row array, optionally reorders it with the configured callback, copies ordered row pointers back to the output array, … |
| `00cfeea0` | MerchantSellInventoryDataSource::MerchantSellInventoryDataS… | confirmed | Initializes the shared inventory data-source base with BaseClient.MerchantSellInventoryData, installs derived vtables, initializes row arrays, and stores the m… |
| `00cff090` | MerchantSellInventoryDataSource::ScalarDeletingDtor | confirmed | Calls the merchant sell datasource cleanup body and frees the object only when deleting-flag bit 0 is set; otherwise returns the original pointer. |
| `00cfefd0` | MerchantSellInventoryDataSource::Cleanup | likely | Clears and frees the two embedded pointer arrays, then invokes the shared BaseFilterItemDataSource destructor body. |

**MiniGameTutorialDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8d510` | MiniGameTutorialDataSource::ColumnName | confirmed | Maps indices 0 through 4 to UI labels including Icon ID and returns a default label for other indices. |
| `00c8d730` | MiniGameTutorialDataSource::GetCount | confirmed | Returns the dword at object offset 0x23c. |
| `00c8e080` | MiniGameTutorialDataSource::GetColumnValue | confirmed | Resolves a tutorial row and returns its ID, localized title, icon, placeholder TODO column, or boolean field through UI value helpers. |
| `00c8e1b0` | MiniGameTutorialDataSource::Dtor | confirmed | Restores tutorial data source vtables, destroys its entry table, calls an additional cleanup routine, and destroys embedded GUI state. |
| `00c8e240` | MiniGameTutorialDataSource::ScalarDeletingAdjustor | confirmed | Subtracts 0x254 from ECX and jumps to 0x00c8e330; this is an adjustor thunk, not the destructor body. |
| `00c8e250` | MiniGameTutorialDataSource::MiniGameTutorialDataSource | confirmed | Registers BaseClient.MiniGameTutorials, installs source vtables, initializes data-source dependencies and table fields, and invokes the tutorial file loader. |
| `00c8e330` | MiniGameTutorialDataSource::ScalarDeletingDtor | confirmed | Calls the class destructor and frees the object when deleting flag bit 0 is set. |
| `00c8df70` | SetKeyedByteOrInsert | likely | Constructs a tutorial entry string, links the new node into the list and hash buckets, and increments stored row count. |

**MountListDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cc4110` | MountListDataSource::~MountListDataSource | confirmed | Installs the MountListDataSource vtable at the object and two embedded subobjects, destroys the map member through 00b058a0, then destroys the GUI user-data me… |
| `00cc4200` | MountListDataSource::MountListDataSource | confirmed | Registers BaseClient.MountList, stores the two constructor arguments at +0x254 and +0x258, installs the class vtable in three subobjects, constructs the mount-… |
| `00cc42a0` | MountListDataSource::ScalarDeletingDestructor | confirmed | Calls the MountListDataSource destructor, frees this only when flag bit 0 is set, and returns this when it is retained and consumes the 4-byte delete-flag argu… |

**MysteryChestDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c6df0` | MysteryChestDataSourceUpdateField | confirmed | When the datasource is not marked closed, invokes a virtual update on its owner, then calls a shared cleanup helper. |
| `009c6e10` | MysteryChestDataSourceFieldName | confirmed | Maps field indices 0 through 10 to chest identifier, display strings and images, remaining seconds, membership, random activity, image set, and allowed-state n… |
| `009c8160` | RefreshMysteryChestDataSource | confirmed | Clears the temporary pointer list, collects current chest records, sorts that list with a local comparator, emits each record to the datasource, and sets the r… |
| `009c8f70` | MysteryChestDataSource::Dtor | confirmed | Restores the data-source vtables at the object base and two subobjects, then destroys its GUI user-data table member. |
| `009c9250` | MysteryChestDataSource::Ctor | confirmed | Initializes the embedded base, installs data-source vtables, stores supplied values at +0x254/+0x258, and clears byte +0x25c. |
| `009c92e0` | MysteryChestDataSource::ScalarDeletingDtor | confirmed | Calls the data-source destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |

**NameValueDataSource** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0098dcf0` | NameValueDataSource::Destructor | confirmed | Stores the NameValueDataSource vtable at object offset zero and returns. |

**NameValueObserver** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0098dda0` | NameValueObserver::Destructor | confirmed | Installs the NameValueObserver vtable, releases two embedded IString values at +0x1c and +0x2c when owned, then calls base cleanup helper 00835610. |
| `0098de90` | NameValueObserver::ScalarDeletingDestructor | confirmed | Calls the NameValueObserver destructor body; frees this only when deleting flag bit 0 is set, otherwise returns this. |
| `0098df30` | NameValueObserver::Constructor | confirmed | Calls the base observer constructor, installs the NameValueObserver vtable, initializes two embedded IStrings, copies each nonnull input string, and registers … |
| `0098e440` | NameValueObserver::NotifyChanged | confirmed | Builds a ClientLuaParam array containing the observer's key and, when available, a string value from GuiDsValue; invokes the registered callback through 009797… |

**NewItemsDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c31a20` | NewItemsDataSourceCtor | confirmed | Registers BaseClient.NewItemsData through the common datasource constructor, then installs four NewItemsDataSource vtables. |
| `00c31aa0` | NewItemsDataSourceDtor | confirmed | Restores NewItemsDataSource vtables in four subobjects and calls the shared datasource destructor. |
| `00c31b00` | NewItemsDataSourceHasRows | confirmed | Checks client/global state and item data, then returns true only when this object's enabled flag is set, the row range is nonempty, and the global eligibility … |
| `00c31b80` | NewItemsDataSourceGetFieldName | confirmed | Returns IsRental for field 0x1b, RentalSecondsRemaining for 0x1c, and delegates all other fields to FUN_00cb09d0. |
| `00c31bb0` | NewItemsDataSourceDtorAdjustor | confirmed | Subtracts 0x254 from ECX and jumps to 00c31bc0, adjusting this for an inherited-base destructor entry. |
| `00c31bc0` | NewItemsDataSourceScalarDeletingDtor | confirmed | Calls the derived datasource destructor, then frees this when flag bit 0 is set. |
| `00c31be0` | NewItemsDataSourceGetFieldValue | confirmed | For field 0x1b, resolves the row object and formats whether its item instance differs from the empty sentinel; for 0x1c, formats remaining rental seconds or ze… |

**NpcsDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a79b50` | NpcsDataSource::Ctor | confirmed | Constructor: calls 00cb2920 with param_2 and the BaseClient.Npcs name, then stores the NpcsDataSource vtable at [this], +0x30 and +0x34 and returns this. |
| `00a79bf0` | NpcsDataSource::DtorBody | confirmed | Destructor body: stores the NpcsDataSource vtable at [this], +0x30 and +0x34 with the EH frame set up, then calls base destructor 00cb2810 on this and returns. |
| `00a79c50` | NpcsDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (27 bytes, vtable slot 0): calls the destructor body 00a79bf0 on this, then if bit 0 of the flag is set calls _free (thunk 00d82463)… |
| `00cb1ce0` | DataSourceProfileColumnName | confirmed | Maps numeric columns to the literal names Guid, Name, Description, profile fields, current and maximum resources, boss state, disposition, and friend state; re… |

**OptionsDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c98db0` | OptionsDataSource::ColumnName | confirmed | Maps indices zero through twelve to option table fields such as Section, DisplayName, Tooltip, Value, OptionName, OptionType, Callback, Disabled, and DefaultVa… |
| `00c98e90` | OptionsDataSource::GetRowId | confirmed | Loads the row object at this+0x228, reads its field at +8, and returns that value. |
| `00c99250` | OptionsDataSource::GetFieldValue | confirmed | Selects an option row and column, reads typed string, numeric, boolean, and callback values, formats output, and reports missing options. |
| `00c998a0` | OptionsDataSource::Dtor | confirmed | Restores OptionsDataSource vtable pointers and calls the shared embedded GUI data table destructor. |
| `00c99920` | OptionsDataSource::Ctor | confirmed | Initializes the options table and its vtables, then stores the row data and related source pointers. |
| `00c999a0` | OptionsDataSource::ScalarDeletingDtor | confirmed | Calls the options data source destructor, frees the instance when deleting flag bit one is set, and otherwise returns the pointer. |

**PlayerEffectTagDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8e610` | PlayerEffectTagDataSource::ColumnName | confirmed | Maps columns 0 through 4 to Instance ID, Icon ID, Duration, Time Left, and Name; other values return an error label. |
| `00c8e700` | PlayerEffectTagDataSource::OnRowsChanged | confirmed | Refreshes rows when dirty, then invokes FUN_010c8f30. |
| `00c8e840` | PlayerEffectTagDataSource::Dtor | confirmed | Restores PlayerEffectTagDataSource vtables and destroys embedded GUI data source state. |
| `00c8e8b0` | PlayerEffectTagDataSource::PlayerEffectTagDataSource | confirmed | Registers BaseClient.PlayerEffectTags, installs source vtables, stores its dependency, and sets the initial dirty state. |
| `00c8e930` | PlayerEffectTagDataSource::ScalarDeletingDtor | confirmed | Calls the source destructor and frees the object when deleting flag bit 0 is set. |
| `00c8e6b0` | PlayerEffectTagDataSource::RefreshRows | likely | Refreshes the UI list, walks active effect tags from client state, emits each tag ID, and clears the dirty flag. |
| `00c8e720` | PlayerEffectTagDataSource::GetColumnValue | likely | Refreshes rows when dirty, resolves the selected tag, and returns instance ID, icon, duration, remaining time, or localized name. |

**PlayerTitleDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cc58c0` | PlayerTitleDataSource::~PlayerTitleDataSource | confirmed | Restores the PlayerTitleDataSource vtable in the object and two embedded subobjects, then destroys the GUI user-data member through 0091bf60. |
| `00cc5930` | PlayerTitleDataSource::PlayerTitleDataSource | confirmed | Registers BaseClient.PlayerTitles, installs the class vtable in three subobjects, stores constructor values at +0x254 and +0x258, and clears byte +0x25c. |
| `00cc59c0` | PlayerTitleDataSource::ScalarDeletingDestructor | confirmed | Calls the PlayerTitleDataSource destructor, frees this only when flag bit 0 is set, otherwise returns this, and consumes the 4-byte delete-flag argument. |

**ProfileDataSource** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8f2b0` | ProfileDataSource::ColumnName | confirmed | Maps fields 0 through 15 to profile, rank, stars, icon set, membership and trial labels. |
| `00c8f390` | ProfileDataSource::OnRowsChanged | confirmed | If the source is not dirty, invokes a virtual callback through its adjusted owner, then calls FUN_010c8f30. |
| `00c8f3b0` | ProfileDataSource::VirtualDispatchAdjustor | confirmed | Loads the function pointer at vtable offset 0x10, clears byte this+0x258, and jumps through it. |
| `00c8f3c0` | ProfileDataSource::ClearDirtyAndJump | confirmed | Clears byte this+0x258 then jumps to 0x010caf20. |
| `00c8f830` | ProfileDataSource::ReadColumn | confirmed | Selects a profile row by key and formats one of its indexed fields into the caller's string output; missing string IDs are formatted as an error. |
| `00c90290` | ProfileDataSource::RefreshRows | confirmed | Clears cached rows, resolves profile strings and keys, rebuilds rows from the profile tables, and marks the source refreshed. |
| `00c90520` | ProfileDataSource::Dtor | confirmed | Restores ProfileDataSource vtables, clears the embedded ProfileSortingData list, and destroys the embedded GUI user-data table. |
| `00c90630` | ProfileDataSource::Ctor | confirmed | Initializes the GUI table for BaseClient.Profiles, installs ProfileDataSource vtables, stores the constructor argument, and zeros row/list state. |
| `00c906d0` | ProfileDataSource::ScalarDeletingDtor | confirmed | Calls ProfileDataSource cleanup and frees the object when the deleting flag's low bit is set; otherwise returns the object pointer. |

**ProfileFilteredDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8ff30` | ProfileFilteredDataSource::RefreshRows | confirmed | Clears its cached linked rows, resolves profile strings and record keys, adds qualifying records, then marks the data source refreshed. |
| `00c905d0` | ProfileFilteredDataSource::Dtor | confirmed | Restores ProfileFilteredDataSource vtables and delegates base cleanup to FUN_00c90520. |
| `00c90790` | ProfileFilteredDataSource::Ctor | confirmed | Constructs the BaseClient.Profiles.Filtered base profile source, then installs ProfileFilteredDataSource vtables. |
| `00c90810` | ProfileFilteredDataSource::ScalarDeletingDtor | confirmed | Calls derived and base cleanup, then frees the object when the deleting flag's low bit is set. |

**ProxiedCharactersDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cb2810` | ProxiedCharactersDataSource::Dtor | confirmed | Restores vtables, destroys the embedded entry array, frees its aligned storage, and releases its GUI user-data holder. |
| `00cb2920` | ProxiedCharactersDataSource::Ctor | confirmed | Initializes the GUI data-source base, stores the source argument, installs proxied-character vtables, clears the dirty state, and initializes its entry array a… |
| `00cb29c0` | ProxiedCharactersDataSource::ScalarDeletingDtor | confirmed | Calls the proxied-character data-source destructor and frees the object when the deleting flag's low bit is set. |
| `00cb29e0` | ProxiedCharactersDataSource::GetCellValue | confirmed | Ensures rows are built, resolves the requested row and source record, then maps requested columns to identifiers, strings, status flags, and other row values. |
| `00cb2740` | FilteredRecordRowsRebuild | likely | Refreshes the backing source, clears the temporary entry array, walks source entries, filters them through a virtual predicate, appends selected records, and m… |

**RecipeComponentDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8a500` | RecipeComponentDataSource::ColumnName | confirmed | Maps indices 0 through 6 to Name, Icon ID, Description, Required Count, Count in Inventory, Tint Value, and Item Definition Id; other indices return an error l… |
| `00c8a570` | RecipeComponentDataSource::VirtualSlot6Adjustor | confirmed | Adjusts ECX by -0x254, loads a function pointer from vtable offset 0x10, and jumps through it. |
| `00c8a9f0` | RecipeComponentDataSource::GetItemDefinitionId | confirmed | Returns zero when field +0x248 is null; otherwise returns the dword at row pointer +0x3c. |
| `00c8bad0` | RecipeComponentDataSource::Dtor | confirmed | Restores four RecipeComponentDataSource vtables, destroys embedded source state and the base data-source object. |
| `00c8bb70` | RecipeComponentDataSource::ScalarDeletingDtorAdjustor | confirmed | Subtracts 0x254 from ECX and jumps to 0x00c8bdd0; this is an adjustor thunk, not the destructor body. |
| `00c8bd10` | RecipeComponentDataSource::RecipeComponentDataSource | confirmed | Registers BaseClient.RecipeComponents, initializes inherited data-source state and stores the supplied dependencies and inventory key. |
| `00c8bdd0` | RecipeComponentDataSource::ScalarDeletingDtor | confirmed | Calls the component source destructor, then frees the object when deleting flag bit 0 is set. |

**RecipeDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8a5b0` | RecipeDataSource::ColumnName | confirmed | Returns labels for columns 0 through 9 and Has Component for valid extended columns; invalid indices return an error label. |
| `00c8a670` | RecipeDataSource::VirtualSlot6Adjustor | confirmed | Adjusts ECX by -0x254, loads a function pointer from vtable offset 0x10, and jumps through it. |
| `00c8aa10` | RecipeDataSource::GetCount | confirmed | Returns the dword stored at object offset 0x884. |
| `00c8b360` | RecipeDataSource::GetColumnValue | confirmed | Resolves a recipe row and provides ID, name, icon, description, member flag, component count, filter, tint, item-definition, requirement, and component status … |
| `00c8cc60` | RecipeDataSource::Dtor | confirmed | Restores RecipeDataSource vtables, releases a retained reference, destroys two hash maps and embedded GUI data table state. |
| `00c8cd60` | RecipeDataSource::ScalarDeletingAdjustor | confirmed | Subtracts 0x254 from ECX and jumps to 0x00c8d0d0; this is an adjustor thunk, not the destructor body. |
| `00c8cfc0` | RecipeDataSource::RecipeDataSource | confirmed | Registers BaseClient.Recipes, initializes its data source and two hash lists, creates a component source, stores references, and registers inventory data. |
| `00c8d0d0` | RecipeDataSource::ScalarDeletingDtor | confirmed | Calls RecipeDataSource cleanup and frees the object when deleting flag bit 0 is set. |

**RentalsDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c30cf0` | RentalsDataSourceCtor | confirmed | Calls the common datasource base constructor, installs RentalsDataSource vtables at four subobject offsets, and stores the supplied configuration pointer at +0… |
| `00c30d90` | RentalsDataSourceDtor | confirmed | Restores the RentalsDataSource vtable in four subobjects and calls the common datasource destructor. |
| `00c30df0` | RentalsDataSourceGetFieldName | confirmed | For field 0x1b returns MarketplaceBundleId; all other fields delegate to FUN_00cb09d0. |
| `00c30f30` | RentalsDataSourceDtorAdjustor | confirmed | Subtracts 0x254 from ECX and jumps to 00c30f40, adjusting this before the shared destructor. |
| `00c30f40` | RentalsDataSourceScalarDeletingDtor | confirmed | Calls the RentalsDataSource destructor and frees the object when flag bit 0 is set. |
| `00c30fc0` | RentalsDataSourceGetValue | confirmed | For field 0x1b returns item+0x28 child field +0xc or zero through the output formatter; other fields delegate to FUN_00cb1500. |

**RewardDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c33020` | RewardDataSourceGetFieldName | confirmed | Maps field indices 0 through 5 to name, icon ID, experience, coin count, entry count, and icon tint; otherwise returns <error>. |
| `00c33700` | RewardDataSourceDtor | confirmed | Restores RewardDataSource vtables, releases the shared reward object when the global gate allows, then destroys the embedded GUI datasource base. |
| `00c337b0` | RewardDataSourceHasItem | confirmed | Returns whether the pointer at this+0x22c is nonzero. |
| `00c338f0` | RewardDataSourceScalarDeletingDtor | confirmed | Runs RewardDataSource teardown and frees the object when flag bit 0 is set. |
| `00c33b30` | RewardDataSourceCtor | confirmed | Registers a datasource, initializes RewardDataSource vtables and fields, constructs the table name with .Entries suffix, allocates a RewardEntryDataSource, sto… |

**RewardEntryDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c33080` | RewardEntryDataSourceGetFieldName | confirmed | Maps indices 0-12 to type, name, description, icon, quantity, tint, item ID, parameters, text color, members-only, GUID, hidden and separated columns; other in… |
| `00c331e0` | RewardEntryDataSourceGetField40 | confirmed | Reads the nested reward item at this+0x22c then +0x25c; returns its +0x40 field or zero if absent. |
| `00c33430` | RewardEntryDataSourceGetFieldValue | confirmed | Resolves the requested reward row, then returns entry type, localized name, description placeholder, icon, quantity, tint, item ID, parameters, text color, mem… |
| `00c337c0` | RewardEntryDataSourceDtor | confirmed | Restores RewardEntryDataSource vtables in its main and adjusted subobjects, then destroys the embedded GUI datasource base. |
| `00c33a80` | RewardEntryDataSourceCtor | confirmed | Registers a base datasource, installs RewardEntryDataSource vtables in three subobjects, and stores three supplied pointers in fields +0x254/+0x258/+0x25c. |
| `00c33b10` | RewardEntryDataSourceScalarDeletingDtor | confirmed | Calls RewardEntryDataSource teardown, then frees the object when flag bit 0 is set. |

**ScreenDirectionIndicatorDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00995ce0` | ScreenDirectionIndicatorDataSource::Slot1FieldNameForIndex | confirmed | Virtual (slot [1]) taking one stack argument (undefined4 index). Switches on the index and returns a pointer to a static name string: 0 Text, 1 Subtext, 2 Icon… |
| `00996df0` | ScreenDirectionIndicatorDataSource::DtorBody | confirmed | Fastcall (ECX=this, primary object). Stores the ScreenDirectionIndicatorDataSource vtable (0180ccb0) at [this], the 0180cc8c vtable at +0x30 and the 0180cc74 v… |
| `00997030` | ScreenDirectionIndicatorDataSource::Ctor | confirmed | Thiscall (ECX=this, primary object) with one stack arg param_2, returns this in EAX (ret 4). Calls 00922890 with ECX=this and the string pointer to "BaseClient… |
| `009970f0` | ScreenDirectionIndicatorDataSource::ScalarDeletingDtor | confirmed | Thiscall (ECX=this) with one stack arg (byte flags). Calls 00996df0 (dtor body, ECX=this); if bit 0 of the flags arg is set, pushes this and calls the free thu… |
| `00996a80` | ScreenDirectionIndicatorDataSource::Refresh | likely | Thiscall; ECX is the primary ScreenDirectionIndicatorDataSource object (00996ea0 and 00996ec0 call it with ECX-0x30, per bytes), so the offsets below are prima… |

**ServerListDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c29c80` | ServerListDataSource::GetColumnName | confirmed | Virtual column-name accessor (vtable slot [1]): returns Name for column 0, Online for column 1 (a pointer three bytes into the literal 0x0182f830), Locked for … |
| `00c29d00` | ServerListDataSource::GetStatusText | confirmed | Writes the current status dword (this+0x258) through the out pointer and switches on it: state 0 returns Invalid session token, state 2 returns No servers avai… |
| `00c2a020` | ServerListDataSource::ServerListDataSource | confirmed | Constructor: registers the data-source name 'BaseClient.ServerList' through 00922890, installs the three vtable pointers (this+0, +0x30, +0x34), zeroes the sta… |
| `00c2a0a0` | ServerListDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c29fb0 and frees the object with _free when the low bit of the flag argument is set. |
| `00c29cc0` | ServerListDataSource::Refresh | likely | Virtual override (framework vtable slot [4], whose default implementation is 010caf20): reads the session state field at *(*(this+0x25c)+0x53d9c) into this+0x2… |
| `00c29e00` | ServerListDataSource::GetRowCount | likely | Virtual method (vtable slot [2]): returns the dword at offset +0x22044 of the object referenced by this+0x22c - the row count of the server list. |

**SingleItemDefinitionDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c975c0` | SingleItemDefinitionDataSource::Ctor | confirmed | Constructs the base item source using BaseClient.ItemDefinitionSelect, installs the single-item data source vtables, and returns the object. |
| `00c97640` | SingleItemDefinitionDataSource::Dtor | confirmed | Restores the single-item derived vtable pointers and invokes the shared base data-source destructor. |
| `00c976a0` | SingleItemDefinitionDataSource::ScalarDeletingDtor | confirmed | Calls the single-item source destructor, frees the instance when deleting flag bit one is set, and otherwise returns the pointer. |

**SoeUtil::HashListMap<int,NotificationImage,32,0>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00968270` | HashListMapNotificationImage32ScalarDeletingDtor | confirmed | Thiscall (ECX = object). Calls the map dtor body 009677a0 on the object, then when the stack flag byte has bit 0 set it frees the object with _free (through th… |
| `00968290` | HashListMapNotificationImage32Init | confirmed | Fastcall (ECX = map object). Installs the HashListMap<int,NotificationImage,32,-1> vtable, zeroes +0x10, +0x8, +0xc and the 0x80-byte bucket area at +0x14 with… |
| `009677a0` | HashListMapNotificationImage32DtorBody | likely | Fastcall (ECX = map object). Sets the HashListMap<int,NotificationImage,32,0> vtable and loops removing the head node at +8 through 0095e740 until none remain.… |
| `00968c10` | HashListMapNotificationImage32DtorEh | likely | Unwind funclet body: saves ECX (the map object) in a local, then calls the map dtor body 009677a0 with ECX = that object. It does not free the object (no flag … |

**TargetDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7a2f0` | TargetDataSource::Ctor | confirmed | Constructor: calls the base constructor 00cb2920 with the BaseClient.Target name, then stores the TargetDataSource vtable at [this], +0x30 and +0x34 and return… |
| `00a7a370` | TargetDataSource::DtorBody | confirmed | Destructor body: stores the TargetDataSource vtable at [this], +0x30 and +0x34, then calls base destructor 00cb2810 on this and returns. |
| `00a7a3d0` | TargetDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (27 bytes, vtable slot 0): calls the destructor body 00a7a370 on this, then if bit 0 of the flag is set calls _free (thunk 00d82463)… |
| `00a7a3f0` | TargetDataSource::Slot14_IsNotCurrentTarget | likely | Virtual slot 14 of TargetDataSource (vtable 01816b04), 57 bytes: loads the owner pointer at this+0x254, calls the table method 00984960 on the table at owner+0… |

**VehicleDemolitionDerbyResultsDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c664e0` | VehicleDemolitionDerbyResultsDataSource::DtorBody | confirmed | Destructor body (no free of this). Sets vtables 018332fc (primary), +0x30 to 018332d8 and +0x34 to 018332c0, sets the dsDerbyResults array header at this+0x258… |
| `00c665c0` | VehicleDemolitionDerbyResultsDataSource::Ctor | confirmed | Constructor with one stack arg stored at this+0x254. Pushes the global name pointer at 01b71da4 (BaseClient.VehicleDemolitionDerbyProcessor.DerbyResults) and c… |
| `00c66650` | VehicleDemolitionDerbyResultsDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor in vftable slot [0] of 018332fc VehicleDemolitionDerbyResultsDataSource::vftable: calls the body 00c664e0, tests bit 0 of the flag a… |

**VehicleEquipScreenCurrentlyEquippedDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c2cc00` | VehicleEquipScreenCurrentlyEquippedDataSource::VehicleEquip… | confirmed | Constructor with two arguments: calls the shared inventory-backed base constructor 00cb1bb0('BaseClient.Vehicle.CurrentlyEquipped',arg3), installs the four vta… |
| `00c2cd50` | VehicleEquipScreenCurrentlyEquippedDataSource::AdjustorDtor… | confirmed | Adjustor thunk for the secondary vtable: subtracts 0x254 from ECX and jumps to the deleting destructor 00c2cd80 (decoded bytes: sub ecx,0x254; jmp +0x25); the … |
| `00c2cd80` | VehicleEquipScreenCurrentlyEquippedDataSource::ScalarDeleti… | confirmed | Scalar deleting destructor (vtable 018303f4 slot [0]): calls the destructor body 00c2ccb0 and frees the object with _free when the low bit of the flag argument… |
| `00c2ccb0` | VehicleEquipScreenCurrentlyEquippedDataSource::~VehicleEqui… | likely | Destructor body: resets the four vtable pointers to VehicleEquipScreenCurrentlyEquippedDataSource::vftable and calls the shared base teardown 00cb1b10, then re… |

**VehicleEquipScreenDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c2ca80` | VehicleEquipScreenDataSource::VehicleEquipScreenDataSource | confirmed | Constructor with five arguments: calls the shared inventory-backed base constructor 00cb1bb0(arg4,arg3), installs the four vtable pointers, stores arg2 at +0x2… |
| `00c2cd40` | VehicleEquipScreenDataSource::AdjustorDtorThunk | confirmed | Adjustor thunk for the secondary vtable: subtracts 0x254 from ECX and jumps to the deleting destructor 00c2cd60 (decoded bytes: sub ecx,0x254; jmp +0x15); the … |
| `00c2cd60` | VehicleEquipScreenDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable 0183034c slot [0]): calls the destructor body 00c2cb20 and frees the object with _free when the low bit of the flag argument… |
| `00c2cb20` | VehicleEquipScreenDataSource::~VehicleEquipScreenDataSource | likely | Destructor body: resets the four vtable pointers to VehicleEquipScreenDataSource::vftable and calls the shared base teardown 00cb1b10, then returns. |
| `00c2cd10` | VehicleEquipScreenDataSource::SetSlotFilter | likely | Setter: stores the argument at this+0x2e0 and calls virtual slot [6] (offset +0x18) of the object at this+0x254 to notify the data source. Its only caller is 0… |
| `00c2cda0` | VehicleEquipScreenDataSource::GetSpecialColumnText | likely | Virtual cell getter (vtable slot [16]) with signature (row record, column, out): for column 0xc it resolves the vehicle record through 0101d0f0(this+0x2e4), sc… |

**VehicleListDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c2bf00` | VehicleListDataSource::VehicleListDataSource | confirmed | Constructor: calls 00cb1bb0('BaseClient.VehicleList', arg2) - the shared inventory-backed data-source base constructor - then installs the four vtable pointers… |
| `00c2c000` | VehicleListDataSource::GetColumnName | confirmed | Virtual column-name accessor (vtable slot [1]): for column 6 it returns the literal 'Active Vehicle' and for every other column index it forwards to the shared… |
| `00c2c110` | VehicleListDataSource::AdjustorDtorThunk | confirmed | Adjustor thunk for the secondary vtable: subtracts 0x254 from ECX and jumps to the deleting destructor 00c2c130 (decoded bytes: sub ecx,0x254; jmp +0x15); the … |
| `00c2c130` | VehicleListDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable 01830024 slot [0]): calls the destructor body 00c2bfa0, tests the low bit of the flag argument and frees the object with _fr… |
| `00c2bfa0` | VehicleListDataSource::~VehicleListDataSource | likely | Destructor body: resets the four vtable pointers (this+0, +0x30, +0x34, +0x254) to VehicleListDataSource::vftable and calls the shared base teardown 00cb1b10, … |
| `00c2c020` | VehicleListDataSource::SetActiveVehicleId | likely | Setter: when the argument differs from the dword at this+0x2dc it stores it and then calls virtual slot [6] (offset +0x18) of the object at this+0x254, which n… |
| `00c2c050` | VehicleListDataSource::IsActiveVehicleRow | likely | Virtual predicate (vtable slot [15]): resolves an object with 00b0d810 and returns 1 when it exists and its dword at +0x70 equals the active-vehicle id stored … |
| `00c2c160` | VehicleListDataSource::GetSpecialCellText | likely | Virtual cell getter (vtable slot [16]) with signature (row, column, out): for column 6 (Active Vehicle) it formats 1 or 0 through 00703620 with the true/false … |

**VehicleLoadoutListDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c2c080` | VehicleLoadoutListDataSource::GetColumnName | confirmed | Virtual column-name accessor (vtable slot [1]): returns LoadoutId (0), Name (1), Description (2), ImageSetId (3), ImageTintValue (4), Quantity (5), MembersOnly… |
| `00c2c560` | VehicleLoadoutListDataSource::VehicleLoadoutListDataSource | confirmed | Constructor: registers the data-source name 'BaseClient.Vehicle.LoadoutList' through 00922890, installs the three vtable pointers, stores the argument at +0x25… |
| `00c2c5e0` | VehicleLoadoutListDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c2c4e0 and frees the object with _free when the low bit of the flag argument is set. |
| `00c2c1d0` | VehicleLoadoutListDataSource::RebuildRows | likely | Rebuilds the data source row list: notifies through vtable slot [3] (+0xc) of this+0x34, takes the client object at *(this+0x254)+0x3e0, walks its vehicle/inve… |
| `00c2c4e0` | VehicleLoadoutListDataSource::~VehicleLoadoutListDataSource | likely | Destructor body: resets the three vtable pointers and calls the shared base teardown 0091bf60, then returns. |

**(no class)** (78)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0081c570` | GuiDsTree_CastFromDataSource | confirmed | Cdecl (two stack args: pointer p, context pointer c). Returns 0 for a null p. Otherwise calls 00d8d382 as an RTTI cast of p from GuiDataSource to GuiDsTree (ty… |
| `0081c7d0` | GuiDsValue_CastFromDataSource | confirmed | Cdecl (two stack args: pointer p, context pointer c). Returns 0 for a null p. Otherwise calls 00d8d382 as an RTTI cast of p from GuiDataSource to GuiDsValue (d… |
| `0081f4f0` | GuiDataModules_UpdateAll | confirmed | Fastcall (ecx=registry). Builds a local List<GuiDataSource*> from the list at registry+0xdde4 (0081b990, 0081a300 and 0081bce0 operations on the lists) and, fo… |
| `00964680` | CastCharacterDataSource | confirmed | Returns zero for a null source; otherwise performs an RTTI cast from GuiDataSource to CharacterDataSource and, when requested and the cast fails, formats and r… |
| `00964760` | CastEquipScreenDataSource | confirmed | Returns zero for a null source; otherwise performs an RTTI cast from GuiDataSource to EquipScreenDataSource and, when requested and the cast fails, formats and… |
| `00b5b470` | RegisterPetDataSources | confirmed | Removes existing linked records from the supplied collection, stores three handler fields, then creates and assigns pet inventory, equipped-item, and pet-trick… |
| `00c298a0` | FormatInt64ToIString | confirmed | Formats the 64-bit value pointed to by the second argument into the IString-style object passed as the first argument using the wide format literal %lld throug… |
| `00c32490` | RefreshCollectionListRows | confirmed | Walks collection rows, queries two string columns, inserts case-ordered records into a temporary ClientCollection list, clears the datasource rows, appends eli… |
| `00c8a220` | BaseClientMembershipIsActiveSource | confirmed | Allocates an object, creates a source for BaseClient.Membership.IsActive, initializes its pointer field, and stores it through the reference-count helper. |
| `00c8db00` | LoadTutorialDataFile | confirmed | Builds the GameTutorialData.txt asset path, invokes the asset reader for the supplied destination, and returns the inverted reader result. |
| `00cb0c60` | DataSourceGetValue | confirmed | If the flag at this+0x2a0 is set, calls a virtual notification through the adjusted base pointer, then returns the value at this+0x24c. |
| `00cb1160` | DataSourceGetCellValue | confirmed | Optionally notifies through a virtual method when this+0x2a0 is set; for an in-range row, resolves its array entry and dispatches another virtual getter, other… |
| `00837390` | CastGuiDsImageListOrWarn | likely | Cdecl with two stack arguments (object, name). Dynamic-casts the object to GuiDsImageList (00d8d382, from GuiDataSource). When the cast fails and the name is n… |
| `00968840` | FindCharacterDataSourceByHash | likely | Thiscall with ECX = owner object and one stack argument, a C string. Hashes the string with 00705bc0, walks the bucket at this+0xaea4 + (hash % 1000)*4 along e… |
| `009688b0` | FindEquipScreenDataSourceByHash | likely | Thiscall with ECX = owner object and one stack argument, a C string. Same lookup as 00968840 (hash through 00705bc0, bucket at this+0xaea4 + (hash % 1000)*4, c… |
| `0098e030` | CreateNameValueObserver | likely | Returns zero if either input is null; otherwise allocates observer storage through 00d833a6, calls the NameValueObserver constructor, and returns the construct… |
| `0098e620` | CopyStringFromGuiDsValue | likely | Resolves and casts the supplied object to GuiDsValue; if its tag flags equal the string case, assigns the string at value+0x50 to the IString output, otherwise… |
| `0098e690` | UpdateGuiDsValueFromFormattedText | likely | Formats a temporary IString using the static format at 01770978 and param_2; if param_1 resolves and casts to GuiDsValue, calls 00821b30 with the formatted str… |
| `009a2f90` | VisitStringFields | likely | Passes three supplied field pointers to the string helper and passes a fourth field pointer only when the flag argument is zero. |
| `009a2fd0` | VisitStringFieldsWithFallback | likely | Passes two supplied fields to the shared string helper, one to the fallback helper, and conditionally processes a fourth field when the flag is zero. |
| `009a35a0` | InsertCurrentKeyAndMarkDirty | likely | Loads the value at this plus 0x294, invokes the keyed hash-list insertion helper, and sets the dirty byte at this plus 0x1298. |
| `009a35e0` | SetKeyedValueAndMarkDirty | likely | Searches a 1024-bucket chain for the key, inserts an entry when none exists, stores the supplied value at entry plus 8, and sets the dirty byte at this plus 0x… |
| `009a4be0` | AppendLinkedKeys | likely | Walks the singly linked chain from param_2 plus 8, appends each node first word with the current source value, and sets the dirty byte. |
| `009a4c50` | VisitLocalizedFieldsWithWideString | likely | Passes two fields to the shared localized-string helper, builds a temporary wide string from the third, and handles a fourth field only when the flag is zero. |
| `009a4c90` | VisitLocalizedFieldsWithWideString | likely | Passes two fields to the shared localized-string helper, builds a temporary wide string from the third, and handles a fourth field only when the flag is zero. |
| `00a75e80` | CastToItemAbilitiesDataSource | likely | Returns param_1 after a runtime dynamic_cast (00d8d382) from GuiDataSource to ItemAbilitiesDataSource (the RTTI descriptors are passed in the call), or 0 for a… |
| `00a75f60` | CastToRecipeComponentDataSource | likely | Returns param_1 after a runtime dynamic_cast (00d8d382) from GuiDataSource to RecipeComponentDataSource (the RTTI descriptors are passed in the call), or 0 for… |
| `00a76280` | LookupItemAbilitiesDataSource | likely | Hashes its argument with 00705bc0, indexes the chained hash table of 1000 buckets at this+0xaea4 (node: value at +0, next at +4, hash at +8), and if a matching… |
| `00a762f0` | LookupRecipeComponentDataSource | likely | Hashes its argument with 00705bc0, indexes the chained hash table of 1000 buckets at this+0xaea4 (node: value at +0, next at +4, hash at +8), and if a matching… |
| `00a76470` | DualIStringArray_SetAtNotify | likely | Method on an owner with a 32-byte element array (vtable at this+0x270, data at +0x274, count at +0x278). If index (arg 1) is below the count, builds a temporar… |
| `00a765b0` | DualIStringArray_SetRowFromChar | likely | Method with the object in ECX and three stack arguments (index, narrow value, text): converts the text argument to a narrow IString temporary (00a34e00), conve… |
| `00a76a20` | DualIStringArray_SetCountNotify | likely | Method with this in ECX, an element count (arg 1) and a notify flag (arg 2): if the count at this+0x278 is below the argument it calls 00a76870 with the argume… |
| `00b342a0` | NotifyTooltipDataChanged | likely | Resolves an object from the arguments; if found, looks up TooltipData, invokes an object notification helper, then calls virtual slot 4 on the datasource. |
| `00b5a9f0` | DestroyEmbeddedGuiUserDataTable | likely | Calls the GuiUserDataTable cleanup routine on an embedded object at the caller-provided stack location and restores the exception chain. |
| `00bfcc70` | NotifyDataChangedKind12 | likely | Initializes a local StringFixed<32>, resolves a value, invokes a nested object's virtual callback with the temporary string, then calls 010cb770 on this with s… |
| `00bfcd30` | NotifyDataChangedKind0 | likely | Reads a value through this+0x254, +0x3e0, and +0x41c, calls 010cb770 with selectors 0 and 0 plus that value, invokes this vtable+0x10, and destroys its tempora… |
| `00bfcdd0` | NotifyDataChangedKind2 | likely | Formats param_2 into a temporary StringFixed<8> using DAT_01770978, calls 010cb770 with selectors 0 and 2 plus that string, invokes this vtable+0x10, then rele… |
| `00bfce80` | NotifyDataChangedKind3 | likely | Formats param_2 into a temporary StringFixed<8> using DAT_01770978, calls 010cb770 with selectors 0 and 3 plus that string, invokes this vtable+0x10, then rele… |
| `00bfcf30` | NotifyDataChangedKind10 | likely | Reads a value through 008a6990, converts it to a boolean, formats it into a temporary StringFixed<8>, calls 010cb770 with selectors 0 and 10, invokes this vtab… |
| `00c1d710` | CheckItemStatsDataSource | likely | Returns 0 for a null pointer; otherwise RTTI-casts GuiDataSource to ItemStatsDataSource and returns the cast result. If the optional diagnostic flag is set and… |
| `00c1d7f0` | CheckSingleItemDefinitionDataSource | likely | Returns 0 for a null pointer; otherwise RTTI-casts GuiDataSource to SingleItemDefinitionDataSource and returns the cast result. If the optional diagnostic flag… |
| `00c1d8d0` | CheckTintGroupDataSource | likely | Returns 0 for a null pointer; otherwise RTTI-casts GuiDataSource to TintGroupDataSource and returns the cast result. If the optional diagnostic flag is set and… |
| `00c2ce40` | SetActiveVehicleId | likely | Setter for the vehicle equip screen: stores the argument at this+0x2e4, resolves the vehicle record with 0101d0f0 and, when it exists, updates this+0x2e8 with … |
| `00c2d0c0` | LookupItemSearchEntry | likely | Hash lookup over the client item table: resolves the client singleton with 00769710, indexes bucket (id & 0x3ff) of the table at *(client+4)+0x20c, and follows… |
| `00c2d3a0` | SearchEntry::SearchEntry | likely | Constructor of a search-index entry from a wide string: constructs the WideStringFixed<256> base through 00c2d290 and sets the extra dword at this+0x214 to 0, … |
| `00c2d410` | SearchEntry::SearchEntry_2 | likely | Constructor of a search-index entry from a string and a payload value: constructs the WideStringFixed<256> base through 00c2d290 and stores the second argument… |
| `00c2d480` | SearchEntry::ScalarDeletingDtor | likely | Scalar deleting destructor of the 0x218-byte search-index entry: calls the shared wide-string teardown 00a759d0 and frees the object with _free when the low bi… |
| `00c2d4e0` | SearchEntry::SearchEntry_3 | likely | Copy constructor of the search-index entry: constructs the WideStringFixed<256> base from the source entry through 00c2d340 and copies the payload dword at sou… |
| `00c2d550` | SearchEntryVectorResize | likely | Grows/rebuilds a vector of 0x218-byte search entries: calls virtual slot [1] (+4) of the container's allocator vtable to allocate the new storage, copies the e… |
| `00c2d650` | AppendSearchEntryWithPayload | likely | Appends one search entry to the vector at param_1: increments the count at param_1+8, grows the storage through 00c2d550 when needed, and constructs the new el… |
| `00c2ddd0` | HugeRecord_ScalarDeletingDtor | likely | Scalar deleting destructor of the large (0x410d4-byte) record type used by the search engine vector: calls the record teardown 00cf2de0 and frees the object wi… |
| `00c31140` | LookupItemStatsRecord | likely | Selects a bucket at this+0x114 indexed by key modulo 0x47, follows +0x20 collision links, and returns the node whose key at +0x1c matches or null. |
| `00c31700` | AppendKeyAndOrdinal | likely | Finds the client stat record by ID, grows an ID array if needed, appends the ID, inserts its array index into a hash map, and returns that index or -1. |
| `00c317b0` | FindOrAppendKey | likely | Searches a hash chain selected by key low 11 bits and returns the mapped index if found; otherwise calls FUN_00c31700 to append and index the ID. |
| `00c31d90` | DatasourceCleanupFrame | likely | Sets up an EH teardown frame, calls the shared datasource destructor, then restores the old exception-list pointer. |
| `00c32180` | RefreshLinkedRows | likely | Calls a datasource clear callback, iterates the linked rows at this+600, feeds each row key to a fill callback, then clears the pending flag at +0x25c. |
| `00c33270` | FormatOutputIntegerField | likely | Formats the dword at param_2 into param_1 through FUN_0082ecd0 using the shared numeric format string, then returns output field +8. |
| `00c4edf0` | InitMemberRecordAndField48 | likely | Calls the two-string initializer, copies the first dword from its second argument to this+0x48, and returns this; the stack-cookie value is not passed. |
| `00c4f0e0` | FormatKeyAndReturnField8 | likely | Formats the value pointed to by its second argument through 0082ecd0 and returns the dword stored at this+8. |
| `00c4fd10` | SetFlagAndDispatchSlot4 | likely | Sets byte this+0x25c to 0, then tail-jumps through this object's vtable slot 4 with ECX unchanged. |
| `00c8a690` | RecipeComponentAtIndex | likely | Starts at the linked-list head at +4, follows each node next pointer at +0x24 for the requested number of steps, and returns the node or zero. |
| `00c8e670` | PlayerEffectTagFindById | likely | Selects bucket key % 0x14, follows links at +0x84, and returns the node whose key at +0x80 matches or zero. |
| `00c8eaa0` | FindCurrentCharacterStat | likely | Finds the current stat entry by ID in a 5-bucket table, then finds the corresponding stat definition in a 10-bucket table. |
| `00cb4c90` | DataSourceAppendRecordAndNotify | likely | Inserts a friend record into the hash/list container, appends its pointer to the row array, marks the source dirty, invokes a virtual refresh method, and retur… |
| `00cc5620` | PopulateListAndMarkPrepared | likely | Calls two virtual operations on the nested object at this+0x34, resets its selection with argument zero, walks the list at this+600+0xc through each node's +0x… |
| `00cca680` | ClearDirtyThenDispatchVirtual_00cca680 | likely | Clears byte +0x25c, loads object's vtable entry at +0x10 and jumps to it. Raw bytes decode as mov/mov/store/indirect-jump. |
| `00cca760` | ClearDirtyThenDispatchVirtual_00cca760 | likely | Clears byte +0x25c, loads object's vtable entry at +0x10 and jumps to it. Raw bytes decode as mov/mov/store/indirect-jump. |
| `00cca810` | ClearDirtyThenDispatchVirtual_00cca810 | likely | Clears byte +0x25c, loads object's vtable entry at +0x10 and jumps to it. Raw bytes decode as mov/mov/store/indirect-jump. |
| `00cca870` | ClearDirtyThenDispatchVirtual_00cca870 | likely | Clears byte +0x25c, loads object's vtable entry at +0x10 and jumps to it. Raw bytes decode as mov/mov/store/indirect-jump. |
| `00ccae50` | ClearDirtyThenDispatchVirtual_00ccae50 | likely | Clears byte +0x25c, loads object's vtable entry at +0x10 and jumps to it; raw bytes decode as mov/mov/store/indirect-jump. |
| `00cf17a0` | ValidateDatasourceType | likely | Returns zero for a null object; otherwise RTTI-casts it from GuiDataSource to BaseItemDefinitionDataSource. If a nonnull name is supplied and the cast fails, f… |
| `00cf1940` | FindBaseItemDefinitionDataSource | likely | Hashes the key with 00705bc0, searches a 1000-bucket chain table at this+0xaea4, and, for a matching entry with a nonnull value, performs an RTTI cast from Gui… |
| `00cf19d0` | ReleaseIfBaseItemDefinitionDataSource | likely | Calls 00cf1940 with the supplied lookup key and, when it returns nonzero, releases the second argument through 00c94030. |
| `00cf1ac0` | RebuildFilteredNodeList | likely | If the nested object at this+0x52f0+0x3e0 exists, unlinks all nodes from the list at this+0x264, scans source nodes at nested+0x2c2c0+0xc, inserts nodes whose … |
| `00cfce00` | InsertMissingPositiveIdAndNotify | likely | Searches a 512-bucket chain for the key; for a missing positive key, inserts a node and calls the data-source callback with the decremented row count and key. |
| `00cff0b0` | BuildLinkedRowIndexAndRefresh | likely | Reserves an index array for the linked rows, copies each row pointer into it, sets the datasource dirty byte, and invokes virtual slot 4. |
| `010c8f30` | ArrayCountAt4 | likely | Returns the element count (dword at +8) of the SoeUtil::Array<int,0,1> object held at this+4, or 0 when this+4 is null (ECX = this, no stack arguments, 12 byte… |
| `010cb050` | LookupStringKeyValue | likely | Thiscall with one stack argument K (a C-string key object, string at K+4). If the byte at this+0x3c is zero it calls the object's vtable slot 13 (+0x34) first.… |

### `ui.dialog`

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00833880` | AppConfirmDisplay | confirmed | Thiscall with five stack arguments (ECX = owner; message value plus four integers). Stores the four integers into the object at owner+0x3bc (+4 = argument 3, +… |
| `009f5ae0` | ShowDialogFromPacket | confirmed | Thiscall with param_2 = parsed dialogue payload. Reads the manager as 00769710()+4 and passes its +0x854 object to 0097c1e0 for Lua parameter arrays. Sets this… |
| `009f6890` | EndDialogAndHide 🌐 | confirmed | Fastcall on a 0x30-byte dialog record. When this[0] is nonzero, releases its +0x28 and +0x2c handles. If the manager's +0x658 byte is clear, constructs Command… |
| `010c0c80` | KeyedEntryRelease | likely | Thiscall with the object in ECX and a key (stack argument). Searches the 32-bucket map at this+0x20 (bucket key & 0x1f) for the key and erases the match with 0… |

### `ui.dstree`

**GuiDsTreeNode** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0099ef20` | GuiDsTreeNode::GuiDsTreeNode | confirmed | Installs GuiDsTreeNode and child-array vtables, clears the parent pointer, and initializes child-array storage and counters to zero. |
| `0099efb0` | GuiDsTreeNode::ScalarDeletingDtor | confirmed | Calls FUN_010CDB00 for node cleanup, conditionally frees the object when the deleting flag is set, and returns its pointer. |
| `010cdb00` | GuiDsTreeNode::DtorBody | confirmed | Restores the GuiDsTreeNode vftable at this, then for each element of the embedded SoeUtil::Array at this+8 (data at this+0xc, count at this+0x10) that is non-n… |
| `0099ef90` | GetStaticPointer175B400 | likely | Returns the static pointer at 0x0175B400. |

**InGamePurchaseDataSource::StoreBundleCategories::CategoriesTreeNode** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a0020` | CategoriesTreeNodeGetMember34 | confirmed | Returns the address this+0x34. |
| `009a0050` | CategoriesTreeNodeGetField28 | confirmed | Returns the 32-bit value stored at this+0x28. |

**(no class)** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c11da0` | PushDsTreeNodeUserdata | confirmed | Creates a Lua userdata of 4 bytes (FUN_00e3b570 = lua_newuserdata), stores the node pointer param_2 in it, attaches the metatable named "DsTreeNode" (FUN_00e3a… |
| `00c11e20` | CheckDsTreeNodeArg | confirmed | Type-checks the userdata at stack index param_2 (FUN_00e39bb0 with type 7 = LUA_TUSERDATA) and fetches it with metatable "DsTreeNode" (FUN_00e39b20); if the us… |
| `00c12010` | RegisterDsTreeNodeLuaModule | confirmed | Registers the DsTreeNode scripting class: FUN_00e39e40(param_1, "DsTreeNode", &table_0182b588, 0) with the method table whose entries name GetText/GetBitmapInd… |
| `00c12120` | CheckDsTreeArg | confirmed | Type-checks the userdata at index param_2 (FUN_00e39bb0 with type 7) and fetches it with metatable "DsTree" (FUN_00e39b20); raises a Lua argument error when th… |
| `00c121c0` | DsTreeEnableScriptEvents | confirmed | Lua C function bound as DsTree.EnableScriptEvents: fetches the userdata at index 1 with FUN_00e3a930 (raising the "DsTree" argument error when it is not the ri… |
| `00c12200` | DsTreeDisableScriptEvents | confirmed | Lua C function bound as DsTree.DisableScriptEvents: fetches the userdata at index 1 with FUN_00e3a930 (raising the "DsTree" argument error on mismatch) and, wh… |
| `00c12300` | RegisterDsTreeLuaModule | confirmed | Registers the DsTree scripting class: first calls FUN_00c12010 (DsTreeNode class), then registers "DsTree" with the method table at 0182b5e8 (Find, GetRootNode… |
| `00c11f20` | DsTreeNodeGetDataName | likely | Lua C function bound as DsTreeNode.GetDataName: fetches the wrapped node with FUN_00c11e20(arg1), requires exactly 2 arguments with the second numeric (FUN_00e… |
| `00c12180` | DsTreeGetRootNode | likely | Lua C function bound as DsTree.GetRootNode: requires exactly one argument, fetches the wrapped tree with FUN_00c12120, and when the tree's node field at +0x30 … |
| `00c123e0` | DsTreeNodeGetChildCount | likely | Lua C function bound as DsTreeNode.GetChildCount: fetches the wrapped node with FUN_00c11e20(arg1), requires exactly one argument (FUN_00e3a390 == 1), pushes t… |

### `ui.equipscreen`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bdaf80` | EquipScreen_FinishedLoading | likely | Builds an SoeUtil::IString holding HandlerEquipScreen and, when the lookup 00977740 succeeds, appends the literal :FinishedLoading with SoeUtil::IStringAppend,… |

### `ui.factory`

**FactoryHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a724d0` | FactoryHandler::FireUpdateCharges | likely | Reads the sub-object pointer at this+0x254 and then its +0xc14 field; when set, calls 009d9c10 as a guard, then 010343e0 on the field, and formats the values a… |

**FactoryPacketCreateBlueprintResponse** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009e6a00` | FactoryPacketCreateBlueprintResponse_Handler 🌐 | confirmed | Constructs the FactoryPacketCreateBlueprintResponse parser object and reads one flag byte. A clean read returns 1; only a zero flag builds FactoryHandler:OnBlu… |

**FactoryToolsDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a72210` | FactoryToolsDataSource::ColumnName | confirmed | Virtual slot [1]. Switch on the int argument: 0 returns ToolId, 1 Name, 2 IconId, 3 IsCurrentlyUseable, 4 ChargesRemaining, 5 UsesCharges, 6 TooltipStringId, a… |
| `00a722e0` | FactoryToolsDataSource::GetCell | confirmed | Virtual slot [4] (row, column, out). Sets the byte at this+0x228 to 1 when it is zero. If row < count at +0x23c it walks the tool list from +0x234 (next pointe… |
| `00a72620` | FactoryToolsDataSource::DtorBody | confirmed | Stores the FactoryToolsDataSource vtables at this+0, +0x30 and +0x34, calls 009e4140 (accepted label ToolDefinitionMapDtor) on the tool-definition map, then ca… |
| `00a726b0` | FactoryToolsDataSource::Ctor | confirmed | Calls the base constructor 00922890 with the name string BaseClient.Factory.Tools, stores its stack argument at this+0x254, installs the FactoryToolsDataSource… |
| `00a72740` | FactoryToolsDataSource::ScalarDeletingDtor | confirmed | Virtual slot [0]. Calls the dtor body 00a72620 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |

**UI::IScreenFactory** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01234d20` | UI::IScreenFactory::~IScreenFactory | confirmed | Destructor body: stores the UI::IScreenFactory vtable into the object and returns; it releases nothing else. |
| `01234d30` | UI::IScreenFactory::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): stores the IScreenFactory vtable, calls _free on this when bit 0 of the flag argument is set, and returns this. |

**UI::ScreenFactory<UI::CampaignScreen>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01235800` | UI::ScreenFactory_CampaignScreen::Ctor | confirmed | Constructor body: stores the UI::IScreenFactory vtable and then the UI::ScreenFactory<UI::CampaignScreen> vtable into the object. |
| `01235d80` | UI::ScreenFactory_CampaignScreen::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of ScreenFactory<CampaignScreen>): stores the UI::IScreenFactory vtable into the object, calls _free on this when bit… |
| `01235f80` | UI::ScreenFactory_CampaignScreen::Create | confirmed | Adjustor thunk for vtable slot 1 of ScreenFactory<CampaignScreen>: adds 4 to ECX and jumps to 012358b0 (the CampaignScreen creator); no other work. |

**UI::ScreenFactory<UI::CollectionManagerScreen>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012356f0` | UI::ScreenFactory_CollectionManagerScreen::Ctor | confirmed | Constructor body: stores the UI::IScreenFactory vtable and then the UI::ScreenFactory<UI::CollectionManagerScreen> vtable into the object. |
| `01235d20` | UI::ScreenFactory_CollectionManagerScreen::ScalarDeletingDt… | confirmed | Scalar deleting destructor (vtable slot 0 of ScreenFactory<CollectionManagerScreen>): stores the UI::IScreenFactory vtable into the object, calls _free on this… |
| `01235f70` | UI::ScreenFactory_CollectionManagerScreen::Create | confirmed | Adjustor thunk for vtable slot 1 of ScreenFactory<CollectionManagerScreen>: adds 4 to ECX and jumps to 012357a0 (the CollectionManagerScreen creator); no other… |

**UI::ScreenFactory<UI::DeckBuilderScreen>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012355e0` | UI::ScreenFactory_DeckBuilderScreen::Ctor | confirmed | Constructor body: stores the UI::IScreenFactory vtable and then the UI::ScreenFactory<UI::DeckBuilderScreen> vtable into the object. |
| `01235cc0` | UI::ScreenFactory_DeckBuilderScreen::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of ScreenFactory<DeckBuilderScreen>): stores the UI::IScreenFactory vtable into the object, calls _free on this when … |
| `01235f60` | UI::ScreenFactory_DeckBuilderScreen::Create | confirmed | Adjustor thunk for vtable slot 1 of ScreenFactory<DeckBuilderScreen>: adds 4 to ECX and jumps to 01235690 (the DeckBuilderScreen creator); no other work. |

**UI::ScreenFactory<UI::GameScreen>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012354d0` | UI::ScreenFactory_GameScreen::Ctor | confirmed | Constructor body: stores the UI::IScreenFactory vtable and then the UI::ScreenFactory<UI::GameScreen> vtable into the object. |
| `01235c60` | UI::ScreenFactory_GameScreen::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of ScreenFactory<GameScreen>): stores the UI::IScreenFactory vtable into the object, calls _free on this when bit 0 o… |
| `01235f50` | UI::ScreenFactory_GameScreen::Create | confirmed | Adjustor thunk for vtable slot 1 of ScreenFactory<GameScreen>: adds 4 to ECX and jumps to 01235580 (the GameScreen creator); no other work. |

**UI::ScreenFactory<UI::LobbyScreen>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01235090` | UI::ScreenFactory_LobbyScreen::Ctor | confirmed | Constructor body: stores the UI::IScreenFactory vtable and then the UI::ScreenFactory<UI::LobbyScreen> vtable into the object. |
| `01235ae0` | UI::ScreenFactory_LobbyScreen::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of ScreenFactory<LobbyScreen>): stores the UI::IScreenFactory vtable into the object, calls _free on this when bit 0 … |
| `01235f10` | UI::ScreenFactory_LobbyScreen::Create | confirmed | Adjustor thunk for vtable slot 1 of ScreenFactory<LobbyScreen>: adds 4 to ECX and jumps to 01235140 (the LobbyScreen creator); no other work. |

**UI::ScreenFactory<UI::LoginScreen>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01234e70` | UI::ScreenFactory_LoginScreen::Ctor | confirmed | Constructor body: stores the UI::IScreenFactory vtable and then the UI::ScreenFactory<UI::LoginScreen> vtable into the object (the second store wins). |
| `01235a20` | UI::ScreenFactory_LoginScreen::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of ScreenFactory<LoginScreen>): stores the UI::IScreenFactory vtable into the object, calls _free on this when bit 0 … |
| `01235ef0` | UI::ScreenFactory_LoginScreen::Create | confirmed | Adjustor thunk for vtable slot 1 of ScreenFactory<LoginScreen>: adds 4 to ECX and jumps to 01234f20 (the LoginScreen creator); no other work. |

**UI::ScreenFactory<UI::MatchmakingScreen>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01235910` | UI::ScreenFactory_MatchmakingScreen::Ctor | confirmed | Constructor body: stores the UI::IScreenFactory vtable and then the UI::ScreenFactory<UI::MatchmakingScreen> vtable into the object. |
| `01235de0` | UI::ScreenFactory_MatchmakingScreen::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of ScreenFactory<MatchmakingScreen>): stores the UI::IScreenFactory vtable into the object, calls _free on this when … |
| `01235f90` | UI::ScreenFactory_MatchmakingScreen::Create | confirmed | Adjustor thunk for vtable slot 1 of ScreenFactory<MatchmakingScreen>: adds 4 to ECX and jumps to 012359c0 (the MatchmakingScreen creator); no other work. |

**UI::ScreenFactory<UI::MenuScreen>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01234f80` | UI::ScreenFactory_MenuScreen::Ctor | confirmed | Constructor body: stores the UI::IScreenFactory vtable and then the UI::ScreenFactory<UI::MenuScreen> vtable into the object. |
| `01235a80` | UI::ScreenFactory_MenuScreen::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of ScreenFactory<MenuScreen>): stores the UI::IScreenFactory vtable into the object, calls _free on this when bit 0 o… |
| `01235f00` | UI::ScreenFactory_MenuScreen::Create | confirmed | Adjustor thunk for vtable slot 1 of ScreenFactory<MenuScreen>: adds 4 to ECX and jumps to 01235030 (the MenuScreen creator); no other work. |

**UI::ScreenFactory<UI::TournamentScreen>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012353c0` | UI::ScreenFactory_TournamentScreen::Ctor | confirmed | Constructor body: stores the UI::IScreenFactory vtable and then the UI::ScreenFactory<UI::TournamentScreen> vtable into the object. |
| `01235c00` | UI::ScreenFactory_TournamentScreen::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of ScreenFactory<TournamentScreen>): stores the UI::IScreenFactory vtable into the object, calls _free on this when b… |
| `01235f40` | UI::ScreenFactory_TournamentScreen::Create | confirmed | Adjustor thunk for vtable slot 1 of ScreenFactory<TournamentScreen>: adds 4 to ECX and jumps to 01235470 (the TournamentScreen creator); no other work. |

**UI::ScreenFactory<UI::TradeLobbyScreen>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012351a0` | UI::ScreenFactory_TradeLobbyScreen::Ctor | confirmed | Constructor body: stores the UI::IScreenFactory vtable and then the UI::ScreenFactory<UI::TradeLobbyScreen> vtable into the object. |
| `01235b40` | UI::ScreenFactory_TradeLobbyScreen::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of ScreenFactory<TradeLobbyScreen>): stores the UI::IScreenFactory vtable into the object, calls _free on this when b… |
| `01235f20` | UI::ScreenFactory_TradeLobbyScreen::Create | confirmed | Adjustor thunk for vtable slot 1 of ScreenFactory<TradeLobbyScreen>: adds 4 to ECX and jumps to 01235250 (the TradeLobbyScreen creator); no other work. |

**UI::ScreenFactory<UI::TradeScreen>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012352b0` | UI::ScreenFactory_TradeScreen::Ctor | confirmed | Constructor body: stores the UI::IScreenFactory vtable and then the UI::ScreenFactory<UI::TradeScreen> vtable into the object. |
| `01235ba0` | UI::ScreenFactory_TradeScreen::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of ScreenFactory<TradeScreen>): stores the UI::IScreenFactory vtable into the object, calls _free on this when bit 0 … |
| `01235f30` | UI::ScreenFactory_TradeScreen::Create | confirmed | Adjustor thunk for vtable slot 1 of ScreenFactory<TradeScreen>: adds 4 to ECX and jumps to 01235360 (the TradeScreen creator); no other work. |

**UI::UIService** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012362a0` | UI::UIService::RegisterScreenFactories | confirmed | Virtual slot 4 of UI::UIService (also FRTWUIService): for each of 11 screen classes it allocates an 8-byte block (00d833a6), constructs a ScreenFactory<T> in i… |

**(no class)** (18)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009e5600` | FactoryHandler_OnFactoryRankUp 🌐 | confirmed | Reads the rank-up payload with FUN_009e03a0 and then, when the parse succeeded and the client dword at +0x23a8 is set, packs an 8-byte argument block and fires… |
| `009e5800` | FactoryHandler_SetEditButtonToExitToolMode 🌐 | confirmed | Reads the payload with FUN_009e04a0 after constructing the FactoryPacketEquipToolResponse object (009d97f0), writes the parsed id into client+0x2270+0xc14, loo… |
| `009e5cc0` | FactoryHandler_BlueprintGenerator 🌐 | confirmed | Parses the payload with FUN_009e0550. On a clean parse it initializes a UI value from the separate 16-byte local block with FUN_0070c620, fires FactoryHandler:… |
| `009e5e00` | FactoryHandler_ShowTools 🌐 | confirmed | Parses the payload with FUN_009e05b0 and, when the parse is clean, enters the UI parameter frame, pushes the parsed 16-byte and 8-byte argument blocks with FUN… |
| `009e5f40` | FactoryHandler_OnFarmTrialExpired 🌐 | confirmed | Constructs the FactoryPacketDoUpsell object (009d9960), reads the payload with FUN_009e0610, loads the data-source value BaseClient.MemberUpgrade.ExpiredJob wi… |
| `009e8760` | FactoryHandler_HaveNpcTooltip 🌐 | confirmed | For the null pair, stores that pair at client+0x22b8/+0x22bc and fires FactoryHandler:HaveNpcTooltip only if the previous +0x23a1 flag was set; it clears that … |
| `009e5980` | FactoryHandler_AddBlueprintFailure 🌐 | likely | Constructs the FactoryPacketAddBlueprintToFoundationFailure parser object, reads a string from the payload, and on a clean parse clears client+0x227c. For none… |
| `01234f20` | AllocNew28Ctor01 | likely | Allocates a 0x28-byte block with operator new (00d833a6); if the allocation is non-null it calls 012f7820 with the block as this and returns that call's result… |
| `01235030` | AllocNew28Ctor02 | likely | Allocates a 0x28-byte block with operator new (00d833a6); if the allocation is non-null it calls 012f7930 with the block as this and returns that call's result… |
| `01235140` | AllocNew28Ctor03 | likely | Allocates a 0x28-byte block with operator new (00d833a6); if the allocation is non-null it calls 012f7a40 with the block as this and returns that call's result… |
| `01235250` | AllocNew28Ctor04 | likely | Allocates a 0x28-byte block with operator new (00d833a6); if the allocation is non-null it calls 012f7b50 with the block as this and returns that call's result… |
| `01235360` | AllocNew28Ctor05 | likely | Allocates a 0x28-byte block with operator new (00d833a6); if the allocation is non-null it calls 012f7c60 with the block as this and returns that call's result… |
| `01235470` | AllocNew28Ctor06 | likely | Allocates a 0x28-byte block with operator new (00d833a6); if the allocation is non-null it calls 012f7d70 with the block as this and returns that call's result… |
| `01235580` | AllocNew28Ctor07 | likely | Allocates a 0x2c-byte block with operator new (00d833a6); if the allocation is non-null it calls 012f7e80 with the block as this and returns that call's result… |
| `01235690` | AllocNew28Ctor08 | likely | Allocates a 0x28-byte block with operator new (00d833a6); if the allocation is non-null it calls 012f7f90 with the block as this and returns that call's result… |
| `012357a0` | AllocNew28Ctor09 | likely | Allocates a 0x28-byte block with operator new (00d833a6); if the allocation is non-null it calls 012f80a0 with the block as this and returns that call's result… |
| `012358b0` | AllocNew28Ctor10 | likely | Allocates a 0x28-byte block with operator new (00d833a6); if the allocation is non-null it calls 012f81b0 with the block as this and returns that call's result… |
| `012359c0` | AllocNew28Ctor11 | likely | Allocates a 0x28-byte block with operator new (00d833a6); if the allocation is non-null it calls 012f82c0 with the block as this and returns that call's result… |

### `ui.gui`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009029a0` | BaseClient::InitGuiAndWebBrowser | confirmed | Virtual, thiscall with stack arg param_2 (bool-valued byte). If FUN_009769b0("GuiOnInit") is nonzero, builds a one-entry Lua argument array holding (param_2 !=… |

**GuiConfirmMessageBox** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008236c0` | GuiConfirmMessageBox::Ctor | confirmed | Stores GuiEventHandler::vftable then GuiConfirmMessageBox::vftable at this, zeroes this+4 and this+8, calls 0081eb80 with the string Main.wndConfirm.OnClose an… |
| `00823730` | GuiConfirmMessageBox::DtorBody | likely | Destructor body: stores GuiConfirmMessageBox::vftable at this, sets a local cleanup flag, calls base teardown 0081bf00 on this, then stores GuiEventHandler::vf… |
| `008237f0` | GuiConfirmMessageBox::ScalarDeletingDtor | likely | Scalar deleting destructor shape: calls 00823730 (dtor body) on this, then calls _free (thunk 00d82463) on this only when bit 0 of the flag argument is set; re… |

**GuiCustomCtrl** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00845200` | GuiCustomCtrl::SetByte150 | confirmed | Thiscall with one byte argument, ret 4. Stores the argument byte at [this+0x150]. Nothing else. |

**GuiEventHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00822c10` | GuiEventHandler::DtorBody | likely | Stores GuiEventHandler::vftable at [ecx] and returns (bytes c7 01 a8 2e 7f 01 c3); no members released. |
| `00822c30` | GuiEventHandler::ScalarDeletingDtor | likely | Stores GuiEventHandler::vftable at this, then calls _free (thunk 00d82463) on this only when bit 0 of the stack flag byte is set; returns this (bytes: test byt… |

**GuiImageCtrl** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0084ee80` | GuiImageCtrl::TailForwardSlot107 | likely | Calls 00406665 with ECX=this, loads the vtable of the returned object, and tail-jumps to its vtable slot +0x1ac (index 107) with ECX set to that object. No arg… |

**GuiModel** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010c9600` | GuiModel::GuiModel | confirmed | Constructor: SEH frame, calls 010c9540 (GuiModelInterface base constructor), stores ActorHandler and BaseAttachmentGroupManagerHandler vtables at +8 and +0xc, … |
| `010c9710` | GuiModel::~GuiModel | confirmed | Destructor body (not a deleting variant; ECX = object): restores the GuiModel vtables at +0/+8/+0xc/+0x10; if the byte at +0x24 is set calls slot 2 of the obje… |
| `010c9e50` | GuiModel::~GuiModelScalar | confirmed | Scalar deleting destructor (slot [0] of the primary GuiModel vtable, flag argument on the stack): calls the GuiModel destructor body 010c9710, then calls _free… |
| `010c9fa0` | GuiModelInit | likely | Thiscall initialiser of a GuiModel node (receiver from CreateInitAppendNode 010cabc0), with eight stack arguments: it stores argument 1 at +0x20, zeroes flag +… |

**GuiModelAttachmentGroup** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010c9ed0` | GuiModelAttachmentGroup::GuiModelAttachmentGroup | confirmed | Constructor (thiscall, stack arguments p2 and p3, ret 8): SEH frame; calls 00769710 and takes its result +0xf0; calls ProxiedCharacterAttachmentGroup::Ctor 00b… |
| `010caa00` | GuiModelAttachmentGroup::~GuiModelAttachmentGroupScalar | confirmed | Scalar deleting destructor (slot [0] of GuiModelAttachmentGroup, flag on the stack): calls 00964b00 with ECX = this (this is kept in a frame slot), then calls … |

**GuiModelInterface** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010c9540` | GuiModelInterface::GuiModelInterface | confirmed | Constructor: sets up an SEH frame, calls 0071b950 (RefCounted-implicit base constructor) with the object, stores the GuiModelInterface vtable into the first dw… |
| `010c95a0` | GuiModelInterface::~GuiModelInterface | confirmed | Scalar deleting destructor (slot [0] of GuiModelInterface, flag argument at stack): stores the SoeUtil::RefCounted vtable into the object, then frees the block… |

**GuiModelManager** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090c270` | GuiModelManager::GuiModelManager | confirmed | Installs GuiModelManagerInterface then GuiModelManager vtables, clears the first three dwords and fields +0xb0/+0xb4, and calls 008f9fb0. |
| `010caaf0` | GuiModelManager::~GuiModelManager | confirmed | Destructor body (ECX = manager): restores the GuiModelManager vtable and then, while the list head at +4 is non-null, unlinks the head node (prev link +0x18, n… |

**GuiProperties<GuiButtonCtrl>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00826910` | GuiProperties::Ctor | likely | Stores the value DAT_017efbd4 (dword) at this+4 and +8, zeroes +0xc and +0x10, stores GuiProperties<GuiButtonCtrl>::vftable at this+0, stores SoeUtil::Map<int,… |
| `00826cb0` | GuiProperties::DtorBody | likely | Destructor body for GuiProperties<GuiButtonCtrl>: stores its vtable at this+0, sets the embedded Map vtable at +0x14, tears the tree at +0x18 down with the pos… |

**GuiProperties<GuiCheckBoxCtrl>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00832450` | GuiProperties::GetPropertyStringNarrowAlt | likely | Virtual slot 30 of GuiProperties<GuiCheckBoxCtrl>. Same narrowing body as its slot-29 sibling but calls vtable slot 8 (+0x20) on this for the wide value. Empty… |
| `00832620` | GuiProperties::GetPropertyWideFormat | likely | Virtual slot 9 of GuiProperties<GuiCheckBoxCtrl>. Looks up the key's entry in the table at DAT_01b34f8c through 008260d0. When an entry is found it formats L"S… |

**GuiProperties<GuiEditCtrl>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00825bf0` | GuiProperties::Ctor | likely | Stores GuiProperties<GuiEditCtrl>::vftable at this+0, sets this+4 and +8 to the dword DAT_017efbd4, zeroes +0xc and +0x10, stores SoeUtil::Map<int,int,-1,0>::v… |
| `00826710` | GuiProperties::DtorBody | likely | Stores GuiProperties<GuiEditCtrl>::vftable at this+0, sets the embedded Map vtable SoeUtil::Map<int,int,-1,0> at +0x14, tears down the tree at this+0x18 throug… |

**GuiProperties<GuiImageCtrl>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00827410` | GuiProperties::Ctor | likely | Stores DAT_017efbd4 at this+4 and +8, zeroes +0xc and +0x10, stores GuiProperties<GuiImageCtrl>::vftable at this+0, the embedded Map<int,int,-1,0> vtable at +0… |

**GuiProperties<GuiLabelCtrl>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00826e90` | GuiProperties::Ctor | likely | Stores DAT_017efbd4 at this+4 and +8, zeroes +0xc and +0x10, stores GuiProperties<GuiLabelCtrl>::vftable at this+0, the embedded SoeUtil::Map<int,int,-1,0> vta… |
| `00827230` | GuiProperties::DtorBody | likely | Destructor body: stores GuiProperties<GuiLabelCtrl>::vftable at this+0, sets the embedded Map vtable at +0x14, tears the tree at +0x18 down with 008234e0 and t… |

**GuiProperties<GuiRadioButtonCtrl>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00832a60` | GuiProperties::GetPropertyStringNarrow | likely | Virtual slot 29 of GuiProperties<GuiRadioButtonCtrl>. Calls vtable slot 9 (+0x24) for the wide value and narrows it: empty length gives the empty narrow string… |
| `00832c30` | GuiProperties::GetPropertyStringNarrowAlt | likely | Virtual slot 30 of GuiProperties<GuiRadioButtonCtrl>. Calls vtable slot 8 (+0x20) for the wide value and narrows it the same way as its siblings (empty length … |
| `00832e00` | GuiProperties::GetPropertyWideFormat | likely | Virtual slot 9 of GuiProperties<GuiRadioButtonCtrl>. Keyed lookup through 008260d0 in the table at DAT_01b34f8c; on a hit formats L"S##%d" with the stored valu… |

**GuiProperties<GuiTreeCtrl>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00832670` | GuiProperties::GetPropertyStringNarrow | likely | Virtual slot 29 of GuiProperties<GuiTreeCtrl>. Calls vtable slot 9 (+0x24) on this with the key and a stack IString<unsigned short>, then narrows the wide resu… |
| `00832840` | GuiProperties::GetPropertyStringNarrowAlt | likely | Virtual slot 30 of GuiProperties<GuiTreeCtrl>. Calls vtable slot 8 (+0x20) on this for the wide value and narrows it the same way (empty length gives the empty… |
| `00832a10` | GuiProperties::GetPropertyWideFormat | likely | Virtual slot 9 of GuiProperties<GuiTreeCtrl>. Same body as the GuiCheckBoxCtrl instantiation: keyed lookup through 008260d0 in the table at DAT_01b34f8c; on a … |

**GuiPropertySet** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00820200` | GuiPropertySet_DispatchCtrlPropertyId | confirmed | Virtual slot [27] of GuiPropertySet (thiscall: this, property id, value). RTTI-casts this to GCtrl (00d8d382); when it succeeds it maps id 0xe, 0xf, 0x14, 0x15… |
| `008207e0` | GuiPropertySet_SetByteFieldFromString | confirmed | Virtual slot [17] of GuiPropertySet: parses the string through 0072d5d0 into an int, clamps it to 0..0xff (0 when negative, 0xff when above), and stores it at … |
| `00820830` | GuiPropertySet_SetHorizAlign | confirmed | Virtual slot [18] of GuiPropertySet: when the parent lookup (0043086f) is nonzero it finds the interface through 0042b9bc and matches the string argument with … |
| `00820970` | GuiPropertySet_SetVertAlign | confirmed | Virtual slot [19] of GuiPropertySet: like 00820830 for the vertical alignment. Matches Default, a 4-character literal at PTR_LAB_0176dbc0, Center, Bottom, Stre… |
| `00820dc0` | GuiPropertySet_SetSizePolicyMask0F0000 | confirmed | Virtual slot [22] of GuiPropertySet: when the parent lookup is nonzero it finds the interface (0042b9bc) and matches the string with 00401940 against Default, … |
| `00820f30` | GuiPropertySet_SetSizePolicyMask0F00000 | confirmed | Virtual slot [23] of GuiPropertySet: the same name table as 00820dc0 (Default, Fixed, Minimum, Preferred, Expanding and the two combined names) matched with 00… |
| `00821190` | GuiPropertySet_SetCtrlPropertyById | confirmed | Virtual slot [12] of GuiPropertySet (thiscall: this, property id, value). RTTI-casts this to GCtrl (00d8d382); on id 0 it assigns the string argument through 0… |
| `00820550` | GuiPropertySet_SetFieldC_FromString | likely | Virtual slot [13] of GuiPropertySet (thiscall: this, object, string). When GetParentObject (0043086f) on the object is zero and the string is non-empty, it par… |
| `008205a0` | GuiPropertySet_SetFieldX10_FromString | likely | Virtual slot [14] of GuiPropertySet: same structure as 00820550 but applies the value with the object's +0x68 field and stores the parsed value as float at thi… |
| `008205f0` | GuiPropertySet_SetScaledValueFromString | likely | Virtual slot [15] of GuiPropertySet: parses the string through 0072d5d0; when the parent lookup (0043086f) is nonzero it finds the interface (0042b9bc) and, if… |
| `008206e0` | GuiPropertySet_SetScaledValueVariant | likely | Virtual slot [16] of GuiPropertySet: like 008205f0 but takes the mask from the interface flags (0x100000 default or flags & 0xf00000), calls the interface's vt… |
| `00820ac0` | GuiPropertySet_SetHorizAlignViaAttached | likely | Virtual slot [20] of GuiPropertySet: when the object attached at this+0xb4 is nonzero it matches the string argument with 00401940 against Default, a 5-charact… |
| `00820c40` | GuiPropertySet_SetVertAlignViaAttached | likely | Virtual slot [21] of GuiPropertySet: the vertical counterpart of 00820ac0. When the object attached at this+0xb4 is nonzero it matches Default, a 4-character l… |

**GuiRegistry** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0081faa0` | GuiRegistry_Ctor | confirmed | Fastcall constructor (ecx=this). Stores ClientLuaOwner and then GuiRegistry vtables, zeroes the 1000-entry (4000-byte) and 1024-entry (0x1000-byte) tables and … |
| `0081ff10` | GuiRegistry_DtorBody | confirmed | Destructor body (does not free the object): restores GuiRegistry::vftable, drains the 64-bucket map at +0xdff0 through 0081b940, unlinks the 1024-bucket lists … |
| `008200a0` | GuiRegistry_ScalarDeletingDtor | confirmed | Scalar deleting destructor (thiscall, one stack byte flag): calls 0081ff10 (the GuiRegistry destructor body) on this and frees this through the _free thunk whe… |

**GuiSurfaceCtrl** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0085ea40` | GuiSurfaceCtrl::PointerEventRoute | likely | Thiscall with five stack arguments and a byte return. For DAT_01bc7524 (move) it updates the object held at +0x10c through 0085e880 and vtable calls, and may c… |

**GuiTabCtrl** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008345a0` | GuiTabCtrl::GuiTabCtrl | confirmed | Thiscall with two stack arguments. Calls 004291d2(0x8040, 0, DAT_01bd792c) for base initialisation, stores the GTabCtrl vtables at +0, +4 and +0x24, zeroes the… |
| `00834700` | GuiTabCtrl::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor shape (ECX = this, one stack flag argument): calls the dtor body 0084a190 on this, and when bit 0 of the flag is set calls … |
| `00834720` | AdjustorThunkMinus24To0084a010 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 0084a010 (unlabelled). |
| `00834730` | AdjustorThunkMinus4To00834700 | likely | Adjustor thunk: subtracts 4 from ECX and jumps to 00834700 (GuiTabCtrl scalar deleting destructor). |
| `00834740` | AdjustorThunkMinus24To00834700 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 00834700 (GuiTabCtrl scalar deleting destructor). |
| `00834750` | AdjustorThunkMinus24To008346a0 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 008346a0 (GuiTabCtrl::ForwardToSubobjectSlot0). |
| `00834760` | AdjustorThunkMinus24To008346d0 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 008346d0 (GuiTabCtrl::ForwardThreeArgsTo005017f8). |
| `00834770` | AdjustorThunkMinus24To008346b0 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 008346b0 (GuiTabCtrl::ForwardToSubobjectSlot1). |
| `00834780` | AdjustorThunkMinus148To00834700 | likely | Adjustor thunk: subtracts 0x148 from ECX and jumps to 00834700 (GuiTabCtrl scalar deleting destructor). |
| `00834790` | AdjustorThunkMinus24To008346f0 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 008346f0, a jump thunk to 004c6dfd. |

**GuiWebBrowserCtrl** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008454c0` | GuiWebBrowserCtrl::SetByte150AndForward | likely | Thiscall with one byte argument, ret 4. Stores the byte at [this+0x150]. If [this+0x264] is non-null it tail-jumps through that object's vtable slot +8 with th… |

**GuiWindowCtrl** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00833b70` | GuiWindowCtrl::GuiWindowCtrl | confirmed | Thiscall with three stack arguments (style word, two more values). Calls 004291d2(0x10440, 0x20, param_4) for the base initialisation, installs the GWindowCtrl… |
| `00833dd0` | GuiWindowCtrl::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor shape (ECX = this, one stack flag argument): calls the dtor body 0085a0a0 on this, and when bit 0 of the flag is set calls … |
| `00833df0` | AdjustorThunkMinus1A0To00833dd0 | likely | Adjustor thunk: subtracts 0x1a0 from ECX and jumps to 00833dd0 (the GuiWindowCtrl scalar deleting destructor). |
| `00833e00` | AdjustorThunkMinus24To00859920 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 00859920 (unlabelled). |
| `00833e10` | AdjustorThunkMinus24To00833d70 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 00833d70 (GuiWindowCtrl::ForwardToSubobjectSlot0). |
| `00833e20` | AdjustorThunkMinus24To00833da0 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 00833da0 (GuiWindowCtrl::ForwardThreeArgsTo005017f8). |
| `00833e30` | AdjustorThunkMinus24To00833d80 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 00833d80 (GuiWindowCtrl::ForwardToSubobjectSlot1). |
| `00833e40` | AdjustorThunkMinus4To00833dd0 | likely | Adjustor thunk: subtracts 4 from ECX and jumps to 00833dd0 (the GuiWindowCtrl scalar deleting destructor). |
| `00833e50` | AdjustorThunkMinus24To00833dd0 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 00833dd0 (the GuiWindowCtrl scalar deleting destructor). |
| `00833e60` | AdjustorThunkMinus24To00833dc0 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 00833dc0, a jump thunk to 004c6dfd (unlabelled). |

**SoeUtil::HashListMap<int,GuiEventHandler*,512,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00818530` | HashListMapGuiEventHandler512::Ctor | confirmed | Fastcall ctor. Stores SoeUtil::HashListMap<int,GuiEventHandler*,512,-1>::vftable at *this, zeroes this+4, this+8 and this+0x10, memsets the 0x800 bytes at this… |

**SoeUtil::HashListMap<int,GuiPropertyDataSource::PropertySetFilter::SharedProperty,128,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ebc30` | GuiSharedPropertyHashListMapDtor | confirmed | Installs the exact SharedProperty map vtable, then while the head at +8 is nonnull calls 0085c530 with the map in ECX and the current head pushed as its argume… |

**SoeUtil::Set<GuiPropertySet*,-1,0>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ebca0` | GuiPropertySetCollectionDtor | confirmed | Calls 008ebc30 with the embedded map pointer at this +0x24 in ECX, releases the two values in the current set node, calls the set-node cleanup virtual at vtabl… |

**(no class)** (36)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00818360` | SetUiScale | confirmed | Thiscall with ECX = this and one float argument. Stores the float at this+0xddb4, calls 00976490 (ClientLuaSetClientFloatField) with the name UiScale and the v… |
| `0081d1e0` | UiModules_CreateDirectoryForPath | confirmed | Callee-cleans one stack arg (ret 4): the suffix string. Reads the PathUiModules setting into a local IString via 00977740, appends the argument through 0040277… |
| `0081d610` | GuiUiModules_ScanXmlFiles | confirmed | Callee-cleans one stack arg (ret 4): the suffix. Reads PathUiModules through 00977740, appends the arg and the globals DAT_017e4400 and '*.xml' with 00402770, … |
| `0081e200` | GuiRegistry_RegisterEventNames | confirmed | Fastcall (ecx=this). Calls 005462ca, then registers a fixed sequence of GUI event name strings (OnKeyDown, OnKeyUp, OnKeyChar, OnChange, OnUserChange, OnMouseW… |
| `008ee0f0` | EnumerateUiModuleXmlFiles | confirmed | Thiscall with a stack argument (param_2). Reads the "PathUiModules" value through 00977740 into a local string, appends param_2 (00402770) and DAT_01770234, co… |
| `00917da0` | SetHoverOverDisplay | confirmed | Thiscall with one stack argument (byte). Stores the low byte at this+0x665, then calls FUN_00910930('Display', 'HoverOver', arg, -1, -1). Returns with ret 4. D… |
| `009b2020` | ValidateActionBarDataSourceType | confirmed | Uses RTTI to test whether input is ActionBarDataSource and formats a GuiDsTable type error when required validation fails. |
| `00a02980` | TryCastGuiDataSourceToTooltipDataSource | confirmed | Returns 0 for a null input; otherwise calls the runtime cast with GuiDataSource as the source type and TooltipDataSource as the target. If the cast fails and p… |
| `00a02ef0` | FireClaimUiEvent | confirmed | Builds a StringFixed<256> from the literal 'HandlerClaimSystem' through 00977740, appends the caller's event name with 00402ff0, and when the lookup succeeds c… |
| `00a03270` | LookupTooltipDataSourceByName | confirmed | Name lookup in the client's data-source hash: hashes param_2 with 00705bc0, walks the chain at (this+0xaea4)+(hash % 1000)*4 comparing stored hashes at node+8,… |
| `00a03540` | PushItemToTooltipDataSource | confirmed | Looks up the 'TooltipData' data source with 00a03270, and when found calls 00a77680 with the caller's argument and then invokes virtual slot 4 of the data sour… |
| `00a03dd0` | FireOnClaimItemContentsEvent | confirmed | Builds a one-argument event block (constructor 0070ec40, slot-2 teardown switch on the first field, then the caller's argument stored as the block's argument),… |
| `00c5c2f0` | ShowCharRootInvoke | confirmed | Thiscall, ECX = owner; four stack dwords popped by ret 0x10 (param_2 = IString*, param_3, param_4; the fourth is never read). Does nothing unless owner+0xf44 i… |
| `010cc170` | EnumerateUiModuleXml | confirmed | Thiscall (ECX = object, no stack arguments): frees and clears the index array at +0xb8 and drains the list at +0xa8 when its count is non-zero; builds the path… |
| `010ccc60` | EnumerateUiModuleDirs | confirmed | Thiscall-style (ECX = object): inside the scope 00fc45f0/00fc4610 it frees the array at +0xb0, drains the list at +0xac when non-empty and sets the count at +0… |
| `00819d20` | GuiFindGCtrlByName | likely | One stack argument, a C string name. Hashes the name through 00705bc0 and looks it up with 00819650 (uint-keyed 1024-bucket table); when found, calls 00d8d382 … |
| `0081e4b0` | GuiRegistry_DispatchUiEvent | likely | Thiscall (ecx=this) with stack args (object p2, event name p3, record p4, secondary name p5). Returns unless p3 is nonzero and either the flag at this+0xddc0 i… |
| `0081eb20` | OwnerHashListMap512_InsertForward | likely | Thiscall (ecx=owner) with two stack args; the addresses of the two arguments are passed to 0081dd10 with ecx = owner+0x3254 (a 512-bucket insert). Returns 8 by… |
| `0081eb80` | OwnerNamedPairInsert_HashAndTree | likely | Thiscall (ecx=owner) with two stack args (name string, value). Computes key = 00705bc0(name) (string hash), inserts (key, value) into the 512-bucket map at own… |
| `0081ebd0` | GuiCtrl_ResolveLayoutValue | likely | Thiscall (ecx=registry) with one stack arg (control p). With p null it returns the value at node+0x2c of the entry for key 0 in the 1024-bucket table at regist… |
| `0081f2e0` | GuiGlobal_QualifyNameAndRegister | likely | Two stack args (uint key, object p2). Looks up key in the global 1024-bucket table at DAT_01be1090+0x2254 and returns 0 when absent. On a match it calls 006f9e… |
| `008200c0` | GuiRegistry_CreateGlobalInstance | likely | No stack args. Allocates 0xe104 bytes through 00d833a6; when the allocation fails it sets DAT_01be1090 to 0, otherwise DAT_01be1090 = the result of 0081faa0 (G… |
| `00820460` | GuiPropertyNameIndex_Lookup | likely | Cdecl with one stack arg (name). Hashes the name with 00705bc0 and scans the static table from 0x01b34f8c in 16-byte steps (dword 0 is the hash) up to 0x01b353… |
| `00823a30` | FlashMiniGameEndScene | likely | No arguments. Calls 0083d550 and 008413d0, calls 00976630 with the name InFlashMiniGame (local 0) and then with ShowUI (local set to 1). When either flag is no… |
| `008347a0` | GuiCreateControlByTypeName | likely | Thiscall with one stack argument (ECX = factory owner; the requested control type name is read from owner+0x3ac). Compares the name by StrnCmpNoCase (00401940)… |
| `009759a0` | ClientLuaInitUiModuleFromArg | likely | Lua C function (one lua_State* argument, returns 0). Under the internals lock (00fc45f0 / 00fc4610): when the GUI sink at ClientLuaInternals+0x90 is non-null a… |
| `00975a50` | ClientLuaToggleGuiRegistryFlagNoArgs | likely | Lua C function (returns 0). Under the internals lock: when the GUI sink at ClientLuaInternals+0x90 is non-null and the argument count (00e3a390) is 0, sets the… |
| `00975bb0` | ClientLuaPushIsModalUp | likely | Lua C function. Under the internals lock: returns 0 when the GUI sink at ClientLuaInternals+0x90 is null; otherwise calls 008182a0, pushes its result as a Lua … |
| `00975c60` | ClientLuaPushModalResult | likely | Lua C function. Under the internals lock: returns 0 when the GUI sink at ClientLuaInternals+0x90 is null; otherwise calls 008182e0, pushes its result as a Lua … |
| `00975d10` | ClientLuaSetModalResultFromBool | likely | Lua C function (returns 0). Under the internals lock: when the GUI sink is non-null and the argument count (00e3a390) is 1, reads argument 1 as a boolean with … |
| `00975dc0` | ClientLuaSetIsModalUpFromBool | likely | Lua C function (returns 0). Under the internals lock: when the GUI sink is non-null and the argument count (00e3a390) is 1, reads argument 1 as a boolean with … |
| `00975f80` | ClientLuaUiModuleCreateDirectory | likely | Lua C function (returns 0) under the internals lock: when the argument count (00e3a390) is 1 and argument 1 read as a string with 00e39c30 (index -1, no length… |
| `00a02d30` | IterateArrayEntriesNotify | likely | For every element of the manager's claim item array (stride 0x168) it builds a formatted string from the element (00a126e0 then 00402ce0) and then calls 00a0eb… |
| `010caa60` | AllocNodeForOwner | likely | ECX = owner value. Calls 007d2da0 to obtain a block; if it is non-null it pushes the owner value and calls 010c9d60 (pool node constructor) with ECX = the bloc… |
| `010cbc30` | BuildWindowDefinition | likely | Thiscall with one stack argument N (a name string): builds a UI window definition from PathUiModules and N. It creates IString temporaries, calls 00977740 (Cli… |
| `01701650` | GuiPropertyNameIndex_StaticInit | likely | Void static initializer: no arguments, no return value. Writes dword 0 of 69 consecutive 16-byte records at 0x01b34f8c + 16*i (i=0..68, last record 0x01b353cc)… |

### `ui.guild`

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b1cb70` | HandleGuildDeleteNotification 🌐 | confirmed | Constructs and parses a GuildDeleteNotificationPacket; on a successful bounded parse, looks up the "GuildDeleted" localization key, formats the fallback "Your … |
| `00b1cd30` | HandleGuildErrorPacket 🌐 | confirmed | Constructs and parses a GuildErrorPacket; after a successful bounded parse, passes the parsed value and string to a virtual interface method and formats "Guild… |
| `00b1d5b0` | HandleGuildInviteAcceptedNotification 🌐 | confirmed | Constructs GuildInviteAcceptNotificationPacket, reads two short fields and a bounded skipped field, then emits the localized action key "GuildInviteAccepted" o… |
| `00b1b6f0` | EmitPlayerGuildMessage | likely | Resolves a message id through virtual slot 0x14 of the interface at *(this+0x8c); when it resolves, resets an argument builder (FUN_0097aa80), assigns an IStri… |

### `ui.headshot`

**HeadshotProxy** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cc5d00` | HeadshotProxy::~HeadshotProxy | confirmed | Restores HeadshotProxy vtables, calls 00cc5c20 to release and clear field +0x20, frees the buffer at +0x10 after 0072d130, then calls the VisibleProxy destruct… |
| `00cc5e10` | HeadshotProxy::ScalarDeletingDestructor | confirmed | Calls the HeadshotProxy destructor, frees this only when flag bit 0 is set, otherwise returns this, and consumes the 4-byte delete-flag argument. |
| `00cc5e30` | HeadshotProxy::ReleaseAndClearField20 | confirmed | Calls 00cc5c20 with this, clears this+0x20, and returns while consuming one 4-byte stack argument. |
| `00cc5e50` | HeadshotProxy::ClearField20 | confirmed | Stores zero at this+0x20 and returns while consuming one 4-byte stack argument. |
| `00cc5e60` | HeadshotProxy::HeadshotProxy | confirmed | Constructs the VisibleProxy base, installs HeadshotProxy vtables, stores supplied context fields, allocates and initializes two render-related objects, process… |
| `00cc5d80` | HeadshotProxy::DispatchWhenDelegateReady | likely | Calls the delegate at this+0xc vtable slot +0x10 with this+0x20; when nonzero and 0076c590 also succeeds, calls 00705200 twice, 00764850 once, then invokes vta… |

**VisibleProxy** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cc5a00` | VisibleProxy::ForwardField10ToDelegate | confirmed | Loads the pointer at this+0x10 into ECX and jumps directly to 0072cfe0. |
| `00cc5a70` | VisibleProxy::VisibleProxy | confirmed | Installs ActorHandler and BaseAttachmentGroupManagerHandler vtables, replaces both with VisibleProxy vtables, stores param_2 and param_3 at +8 and +0xc, and ze… |
| `00cc5cd0` | VisibleProxy::ScalarDeletingDestructor | confirmed | Calls the VisibleProxy destructor, frees this only when flag bit 0 is set, otherwise returns this, and consumes the 4-byte delete-flag argument. |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cc5a30` | ForwardField20ThroughDelegate | likely | Loads this+0x20, obtains the delegate at this+0xc, and calls the delegate's vtable slot +0x10 with the loaded value. |
| `00cc5df0` | ForwardFieldB8ThroughDelegate | likely | Loads this+0xb8, obtains the delegate at this+0xc, and calls delegate vtable slot +0x10 with the loaded value. |

### `ui.help`

**OnlineHelpWindow** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009959f0` | OnlineHelpWindow::Slot2FloatStateTick | likely | Virtual (slot [2]) with ECX=this and a float stack arg. If the float equals DAT_017ec67c: when [this+0x10]==3 it calls 00995660 and 009940f0 with ECX=this-4 an… |

### `ui.hud`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad11d0` | HideHudAndChat | confirmed | When this+0x5ef48 is nonzero, dispatches HUD:hideMain, GameDock:Hide, and ChatLog:Hide with zero arguments. |

### `ui.imagelist`

**GuiDsImageList** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c137a0` | GuiDsImageList::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of GuiDsImageList): calls the destructor body FUN_00c13570 (which empties and frees the embedded array and runs the G… |
| `00c13570` | GuiDsImageList::~GuiDsImageList | likely | Destructor body: resets the embedded array vtable at +0x30, releases its elements with FUN_00c13400, frees the element buffer with FUN_00401c00(ptr,4), clears … |

**SoeUtil::Array<GuiDsImageList::Image,0,1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c132d0` | ImageArrayResize | confirmed | Virtual slot 1 (reallocate) of the embedded image array: computes a new capacity from the requested count param_2 and the current capacity at +0xc (grow by 5/4… |
| `00c13370` | ImageArrayFreeBuffer | confirmed | Virtual slot 2 (release) of the embedded image array: calls FUN_00401c00(param_1, 4), i.e. frees the element buffer with alignment 4 through the memory manager. |
| `00c13390` | ImageArrayMoveAssign | confirmed | Virtual slot 3 (move/swap) of the embedded image array: when param_3 is non-zero it forwards to the source object's virtual slot +0xc; otherwise it swaps the t… |
| `00c134c0` | ImageArrayScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of the image array): stores the array vtable, releases all elements with FUN_00c13400, frees the element buffer with … |
| `00c13400` | ImageArrayPopBack | likely | Removes param_2 elements from the end of the image array: computes the new size = size - param_2, walks the elements from the new size to the old size and call… |
| `00c13490` | ImageListArrayDtor | likely | Destructor body of the embedded image array: stores the array vtable, releases every element by calling the pop-back routine FUN_00c13400 with the current size… |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c13060` | CheckImageListArg | confirmed | Type-checks the userdata at index param_2 (FUN_00e39bb0 type 7) and fetches it with metatable "ImageList" (FUN_00e39b20); raises a Lua argument error on mismat… |
| `00c131d0` | RegisterImageListLuaModule | confirmed | Registers the ImageList scripting class: FUN_00e39e40(param_1, "ImageList", &table_0182b790, 0) with the entries Create/Destroy/Find/LoadImage, re-selects the … |
| `00c130c0` | ImageListLoadImage | likely | Lua C function bound as ImageList.LoadImage: requires exactly 3 arguments, fetches the list with FUN_00c13060(arg1), reads arg2 as an integer (FUN_00e3a7b0) an… |
| `00c135f0` | ImageListCreate | likely | Lua C function bound as ImageList.Create: requires arg1 to be a string (FUN_00e39c30), allocates 0x40 bytes with FUN_00d833a6, constructs a GuiDsImageList on i… |

### `ui.job`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ff660` | BaseClient::UpdateJobSkillProgress | confirmed | Finds a job node whose key at +0 equals param_2 in the list at this+0x3e0 -> +0x49c; if param_3 indexes the node's 0x28-byte element array (count at node+0xd0 … |

**PlayClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090c500` | PlayClient::HandleJobBrowser | confirmed | Finds the HandlerJobBrowser GUI element, builds SetJobCount data from the player's job profiles, filters profiles when a selected key exists, and dispatches th… |

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00900370` | PushSetJobInfo | confirmed | Assigns param_2 to an IString (FUN_00402ce0) and appends ':SetJobInfo' to it (the call name). Builds a ten-slot Lua argument array (FUN_0070b470 slot requests)… |
| `00900a20` | PushJobSkillPoints | confirmed | Gated on FUN_00977740("HandlerJobBrowser"), appends ':SetJobSkillPoints' to an IString, builds a two-entry Lua argument array (FUN_0070ebe0) holding param_1 an… |
| `00a78860` | JobTooltip_BuildRows | confirmed | Method with this in ECX and a job record argument: sets the row count to 6 through 00a76870 or 00a75d50, resets the state at +0x280 to +0x294, calls 008356b0 w… |
| `00ad4b80` | ShowJobHandler | confirmed | When manager offset +0x14 is set, forwards param_3 to an object virtual slot, calls 008c6b60, pushes Lua parameters including constants 3 and 1, invokes JobHan… |
| `00c0a760` | RegisterJobScreenModule | confirmed | Registration wrapper: pushes nup=0, the 5-entry table 0x0182a810 and the 'JobScreen' module-name pointer, calls FUN_00e39e40 and returns 0. |
| `00c0a630` | Job_SelectJob | likely | Lua 'SelectJob' handler: with one numeric argument, when it is positive it resolves the local player (FUN_00769710) and calls the player object's virtual metho… |
| `00c0a680` | Job_PopulateJobInfo | likely | Lua 'PopulateJobInfo' handler: with one numeric argument it resolves the local player (FUN_00769710) and calls the player object's virtual method at vtable slo… |

### `ui.jobcustomization`

**JobCustomizationDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a84630` | JobCustomizationDataSource::GetField | confirmed | Lua/UI data-source field getter (vtable slot [4]): resolves the value for (row param_1, field param_2) with 00a84510, formats it through the SoeUtil IString fo… |
| `00a84750` | JobCustomizationDataSource::GetColumnName | confirmed | Column-title getter (vtable slot [1]): on first use it initialises the file-scope IString at DAT_01be785c and registers its destructor with _atexit, formats "E… |
| `00a84cb0` | JobCustomizationDataSourceDtor | confirmed | Destructor body of the job-customization data source: stores the three class vtables, destroys the row object at param_1[0x98] through 00a84840 and FreeMemory,… |
| `00a84d50` | JobCustomizationDataSource::GetRowCount | confirmed | Row-count getter (vtable slot [2]): returns 0 when the row-source object at this+0x230 is null, otherwise returns that object's field +0x14 (the row count). |
| `00a84e80` | JobCustomizationDataSourceScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]) of JobCustomizationDataSource: runs the destructor body 00a84cb0 and frees the object with _free when the delete f… |
| `00a848d0` | ResetJobCustomizationSlots | likely | Empties the equipment-slot table: allocates a small local helper object (00a035f0/00965be0/00a03690 begin-end pair), sets the value 0, and appends 18 (0x12) em… |
| `00a84be0` | JobCustomizationClearSlot | likely | Writes 0 into one equipment slot of the table (this+0x264+param_2*4) and then refreshes the table with 00a848d0. |
| `00a84d70` | JobCustomizationDataSourceCtor | likely | Constructor of the job-customization data source: registers the name through 00922890(PTR_s_BaseClient.JobCustomizationData), stores the three class vtables, s… |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a84510` | LookupJobCustomizationSlotValue | likely | Looks up one value of the job-customization row param_2: reads the client's data store at client+0xc8 -> +0x6e8, walks the row list at this+0x260 (next link at… |
| `00a845a0` | JobCustomizationSetSlotValue | likely | Stores one resolved slot value: computes the value with 00a84510(param_2,param_3) and writes it into the per-slot array at this+0x264+param_3*4, then builds a … |
| `00a84840` | RowObjectDtorBody | likely | Destructor body of the row objects kept by the job-customization data source: drains the singly-linked list at this+0xc by calling 008cb1c0 on each removed hea… |
| `00a848b0` | RowObjectScalarDeletingDtor | likely | Deleting destructor wrapper for the job-customization row object: runs the destructor body 00a84840 and then frees the object with FreeMemory (00401c00) using … |

### `ui.loading_screen`

**LoadingScreenThread** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cabcb0` | LoadingScreenThread::Run | confirmed | Pumps queued Windows messages, calls the loading-screen frame routine, sleeps 25 ms, and repeats until the stop flag is set. |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cab620` | SetLoadingBarProgress | confirmed | Clamps progress to 0..100, tracks a byte count, formats large sizes, and invokes the loading movie's setLoadingBarPct method when the value changes. |
| `00cabab0` | RenderLoadingScreenFrame | confirmed | Measures elapsed time, updates progress, advances and renders the loading GFx movie, and runs associated loading callbacks. |
| `00cab950` | UpdateLoadingProgress | likely | Reads loading counters, adds optional work totals, computes a percentage, and advances progress through the helper at 00cab620. |

### `ui.login`

**Engine::AccountUsernameRegistration** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01294c60` | Engine::AccountUsernameRegistration::BuildRegisterUsernameC… | likely | Returns 1. Reads UserName through FUN_012eb250 with the empty-string default and copies it to a local string. Constructs a local LoginCommand_RegisterUsername … |

**Engine::AccountUsernameRegistrationResultsDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01294de0` | Engine::AccountUsernameRegistrationResultsDisplayAction::Ap… | likely | Returns 1. Reads arg 0 as a boolean and arg 1 as an integer through FUN_012f8f60, defaulting mismatched types to false and zero. Maps 0x11 to registrationFailu… |

**Engine::GatewayServerSuccessDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124fda0` | Engine::GatewayServerSuccessDisplayAction::ApplyConnectionS… | confirmed | Slot [2] of Engine::GatewayServerSuccessDisplayAction::vftable, 242 bytes. If FUN_00f9c010 returns a UI object it sets the value 'ConnectionSucceeded' to 1 thr… |

**Engine::NetworkQueryUserNameRegistration** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01250040` | Engine::NetworkQueryUserNameRegistration::OpenRegisterUserN… | confirmed | Slot [2] of Engine::NetworkQueryUserNameRegistration::vftable, 245 bytes. If FUN_00f9c010 returns a UI object it sets 'ScreenName' to 'RegisterUserNameDialog' … |

**UI::LoginScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f7820` | UI::LoginScreen::LoginScreen | confirmed | Constructor (fastcall, ECX = this): calls UI::LoginScreenBase::Ctor 013fdf90, then stores UI::LoginScreen::vftable at *this. |
| `012f7880` | UI::LoginScreen::DtorBody | confirmed | Non-deleting destructor body: stores UI::LoginScreen::vftable at *this, then calls UI::LoginScreenBase::Dtor 013fdff0. |
| `012f78d0` | UI::LoginScreen::~LoginScreen | confirmed | Scalar deleting destructor (vtable slot [0]): stores UI::LoginScreen::vftable, calls the destructor body 013fdff0 (base), then calls _free on this when bit 0 o… |

**UI::LoginScreenBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013fdf90` | UI::LoginScreenBase::Ctor | confirmed | Fastcall constructor: calls base constructor 012f8420, then stores UI::LoginScreenBase::vftable at this. |
| `013fdff0` | UI::LoginScreenBase::Dtor | confirmed | Fastcall destructor body: stores UI::LoginScreenBase::vftable, then calls base destructor body 012f8480. |
| `013fe040` | UI::LoginScreenBase::ScalarDeletingDtor | confirmed | Virtual slot 0 of UI::LoginScreenBase::vftable: stores the vtable, calls base destructor body 012f8480, frees the object with _free when the flag argument has … |

### `ui.marketplace`

**ClientTCGInterface** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad1740` | ShowMarketplace | confirmed | Dispatches MarketplaceHandler:Show with two zero arguments. |
| `00ad2920` | ShowMarketplaceWithThreeArgs | confirmed | Builds three Lua arguments from the supplied values, invokes MarketplaceHandler:Show, and destroys the temporary argument array. |
| `00ad2ac0` | ShowMarketplaceWithTwoArgs | confirmed | Builds two Lua arguments from the supplied values, invokes MarketplaceHandler:Show, and destroys the temporary argument array. |
| `00ad2c30` | ShowMarketplaceWithOneArg | confirmed | Builds one Lua argument from the supplied value, invokes MarketplaceHandler:Show, and destroys the temporary argument array. |

### `ui.merchant`

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0a890` | RegisterMerchantSellModule | confirmed | Registration wrapper: pushes nup=0, the 5-entry table 0x0182a8ac and the 'MerchantSell' module-name pointer, calls FUN_00e39e40 and returns 0. |
| `00c0a9a0` | RegisterMerchantBuyModule | confirmed | Registration wrapper: pushes nup=0, the 3-entry table 0x0182a8dc and the 'MerchantBuy' module-name pointer, calls FUN_00e39e40 and returns 0. |
| `00c0a950` | MerchantBuy_SelectInventoryCategory | likely | Lua 'SelectInventoryCategory' handler: with one numeric argument, when the local player exists, it calls FUN_00c6f250(category) on the merchant window object; … |
| `00c0ac10` | MerchantSell_BuyFromUi | likely | Lua 'BuyFromUi' handler: requires 5 arguments; when the count argument is positive it resolves the merchant record (FUN_00c388b0), applies the +0xc5 price-clea… |

### `ui.minimap`

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008dd710` | MiniMapResetHandler | confirmed | Gated on FUN_006fc5b0(param_2, *(this+0x94c)) being non-zero: assigns the string argument through SoeUtil::IStringAssign, calls the map helpers 0098ae00/00b0ca… |
| `008fe620` | MiniMapSyncTileImages | confirmed | thiscall with four stack arguments (two floats and two dwords, param_2..param_5), ret 0x10. Builds a 0x8c-stride tile list size with 00b0d1e0 from the array at… |
| `00a8e540` | MiniMapKillIndicator | confirmed | Finds an indicator by its two-part key; on a hit, calls MiniMap:KillIndicator with the stored id, removes the entry, and refreshes the related manager when pre… |
| `00a8e660` | MiniMapSelectIndicator | confirmed | Looks up the keyed indicator; clears selection on a miss or sends its id and coordinates to MiniMap:SelectIndicator, updates cached and view-state keys, and re… |
| `00a8e860` | MiniMapUpdateIndicatorLocations | confirmed | When the 0x14d elapsed-time guard trips, refreshes its timestamp and walks tracked entries; re-resolves each actor key, removes absent entries only when flagge… |
| `00a8fc80` | MiniMapPlaceIndicator | confirmed | Creates a keyed indicator entry, stores its id, display strings, coordinates and flags, builds nine Lua parameters, calls MiniMap:PlaceIndicator, and refreshes… |
| `00a90070` | MiniMapPlaceFriendIndicator | confirmed | Finds two keyed entries, resolves the Friend label, and calls the shared indicator-placement routine with the resolved values and current record field. |
| `00a901b0` | MiniMapPlaceGroupMemberIndicator | confirmed | Finds two keyed entries, resolves the MiniMapLabelGroupMember label, and calls the shared indicator-placement routine with the resolved values and current reco… |
| `00a902f0` | MiniMapPlaceGuildMemberIndicator | confirmed | Finds two keyed entries, resolves the MiniMapLabelGuildMember label, and calls the shared indicator-placement routine with the resolved values and current reco… |
| `00a88280` | MiniMapClearSelectedIndicator | likely | Returns early if this+0x11a8/+0x11ac equal DAT_01be7a60/DAT_01be7a64; otherwise calls 009797a0 with the string MiniMap:ClearSelectedIndicator, stores the defau… |
| `00a8e840` | ForwardToMiniMapKillIndicator | likely | Forwards both supplied arguments to 00a8e540 and returns without additional processing. |

### `ui.namechange`

**NameChangeDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0098dbd0` | NameChangeDataSource::GetColumnName | confirmed | Maps indices 0 through 6 to Id, Type, IsCheckPending, IsRenamePending, Name, Status, and StatusString; returns <error> for other indices. |
| `0098dc40` | NameChangeDataSource::Method2 | confirmed | If byte +0x22c is clear, calls the virtual method at object this-0x30 slot +0x38; then tail-jumps to 010c8f30 with ECX adjusted to this+4. |
| `0098dc70` | NameChangeDataSource::RebuildFilteredRows | confirmed | Resets the row-list helper, walks the records, appends each record whose +8 key matches filter +0x260 or when that filter is -1, then sets byte +0x25c. |
| `0098e240` | NameChangeDataSource::GetFieldValue | confirmed | Selects a row and maps requested fields to its id, type, pending flags, name, status, or status string; writes fallback text for missing rows, missing data, an… |
| `0098e790` | NameChangeDataSource::Destructor | confirmed | Installs the NameChangeDataSource vtables at the three object subobjects and destroys the GuiUserDataTable member at +0x34. |
| `0098e810` | NameChangeDataSource::Constructor | confirmed | Registers BaseClient.NameChange, initializes its base state, installs three NameChangeDataSource vtables, stores two supplied fields, and sets its state bytes … |
| `0098e8a0` | NameChangeDataSource::ScalarDeletingDestructor | confirmed | Calls the NameChangeDataSource destructor body; frees this only when deleting flag bit 0 is set, otherwise returns this. |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a15730` | RefreshNameChangePermissions | confirmed | Refreshes the client barrier, selected quest display and control; finds the fixed-hash value for NameChange.CanChangeName/CanChangeCharacterName and selector v… |
| `00c1f5a0` | StartTwoPartNameCheck | likely | Reads the first two indexed strings, reads a third value as a boolean, passes both strings and that boolean to 00a21c00, then pushes the returned result. |
| `00c1f720` | ConfirmNameChangeForLocalHandle | likely | Reads two indexed strings, passes their temporary strings to 00a20b50, then pushes the returned result. |

### `ui.notification`

**NotificationQueue** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cb7640` | NotificationQueue::GiveTime | confirmed | Fastcall on receiver p: calls 00769710 and returns when the object it yields has byte +4 +0x959 nonzero; computes elapsed = now (007027f0 result) minus the sta… |

**SoeUtil::List<NotificationQueue::Entry,8>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cb7190` | NotificationQueueListCtor | confirmed | Installs SoeUtil::List<NotificationQueue::Entry,-1>::vftable then SoeUtil::List<NotificationQueue::Entry,8>::vftable at receiver+0, zeroes receiver+4, +8, +0xc… |

**(no class)** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a94640` | DisplayErrorMessage | confirmed | Builds a two-value Lua parameter array from its arguments and submits it as ErrorMessage with arguments 6000,0. |
| `00a94780` | DisplayAlertMessage | confirmed | Builds three Lua values, defaults a null color to the 0xFFFF00 string, and submits AlertMessage with arguments 6000,0. |
| `00a94900` | DisplaySystemMessage | confirmed | Builds three Lua values, defaults a null color to the 0xFFFF00 string, submits SystemMessage, and when param_5 is set calls a virtual method with param_2. |
| `00a95940` | NotifyAchievementCompleted | confirmed | When param_7 is zero, formats AchievementCompleted text, builds five Lua values and posts CompleteAchievement with trailing arguments 10,10. |
| `00adba80` | WheelMessage_Notify | confirmed | Builds the localized wheel text: it asks the client's text interface (vtable slot 0x14) for the wheel_bonus_spin and TimeUnitDay strings with the message's num… |
| `00a92570` | PopulateAndSubmitNotificationArgs | likely | When enabled, passes two values from the supplied record to 007121e0 and calls 00cb74f0; when param_1 is below 3 it also calls 00cb7070. |
| `00a94ab0` | FormatPlayerMessage | likely | Looks up param_3, builds a player-name format value, and if formatting succeeds calls 00a94900 with the supplied message fields. |
| `01088440` | ParseNotificationImageRow | likely | Thiscall with a text cursor (param_2) and a length (param_3), returns 1. Parses seven consecutive fields from the text: three through 0072d900 into this+0x0, +… |

### `ui.options`

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a4d650` | OptionsManager_Init | confirmed | Initializes the options UI object, stores the supplied argument at +0x30, then creates three 0x25c-byte data-source objects named BaseClient.Options, BaseClien… |
| `00a4e1f0` | OptionsManager_LoadOptionElements | confirmed | Formats "%sOptionElements.txt" from the prefix argument through SoeUtil::IStringFormat and parses the resulting path into the element list at the object base t… |
| `00a4e2b0` | OptionsManager_LoadControllerElements | confirmed | Formats "%sControllerElements.txt" from the prefix argument and parses it into the element list at this+0x10 through FUN_00a4dd70, then releases the temporary … |
| `00a4e380` | OptionsManager_LoadHelpCreditsElements | confirmed | Formats "%sHelpCreditsElements.txt" from the prefix argument and parses it into the element list at this+0x20 through FUN_00a4dd70, then releases the temporary… |
| `00a4da10` | OptionsElementRecord_ScalarDeletingDtor | likely | Scalar deleting destructor of the options-element record: runs the record destructor FUN_00a4d7a0 (releasing the eleven IString members) and frees the object t… |
| `00c20c80` | SetSoundChannels | likely | Reads a numeric argument, caps it at two, maps nonpositive values to zero, and writes Sound/Channels when the settings manager is available. |
| `00c20d50` | SetSoundQuality | likely | Reads a numeric argument, caps it at two, maps nonpositive values to zero, and writes Sound/Quality when the settings manager is available. |
| `00c20ee0` | SetGraphicsQuality | likely | Reads one numeric argument and writes Rendering/GraphicsQuality when the global settings manager is available. |
| `00c20f30` | GetGraphicsQuality | likely | Reads Rendering/GraphicsQuality when settings are available, otherwise obtains a fallback value through 007941d0, then pushes the selected integer as a number. |
| `00c20fc0` | SetTextureQuality | likely | Reads one numeric argument and writes Rendering/TextureQuality when the global settings manager is available. |
| `00c21010` | GetTextureQuality | likely | Reads Rendering/TextureQuality when settings are available, otherwise obtains a fallback value through 007941d0, then pushes the selected integer as a number. |

### `ui.photographer`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c06440` | RegisterPhotographerModule | confirmed | Registration wrapper: pushes nup=0, the entry table 0x01829de0 and the 'Photographer' module name, calls FUN_00e39e40 and returns 0. |
| `00c06200` | Photographer_GetPhotoResolution | likely | Reads the client singleton (FUN_00769710) and, when its +4 client pointer is set, pushes the two 32-bit values at client+0xc78 and client+0xc7c as Lua numbers … |

### `ui.pirates`

**PiratesShipSelection** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a60c10` | PiratesShipSelection::PopulateSlots | confirmed | When requested, iterates ship records and fills three visible ship slots with string fields, index, and other scalar values; when requested, fills up to six de… |

**PiratesStandings** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a600c0` | PiratesStandings::SetPlayerFlag | confirmed | Builds three Scaleform values of kind 1 from the three arguments, sets the appropriate slots when replacing old typed values, then invokes the named Pirates st… |

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a612d0` | DecrementCursor374AndRefresh | likely | If the cursor at this+0x374 exceeds 1, decrements it, then invokes 00a60c10 with arguments 1 and 0. |
| `00a612f0` | IncrementCursor374AndRefresh | likely | Increments the cursor at this+0x374 only while it is less than the ship-list count at nested offset +0x29c minus 2, then invokes 00a60c10 with arguments 1 and … |
| `00a61320` | DecrementCursor378AndRefresh | likely | If the cursor at this+0x378 exceeds 1, decrements it, then invokes 00a60c10 with arguments 0 and 1. |
| `00a61340` | IncrementCursor378AndRefresh | likely | Increments the cursor at this+0x378 only while below the decal-list count at nested offset +0x300 minus 5, then invokes 00a60c10 with arguments 0 and 1. |
| `00a63930` | PiratesIndicators_Update | likely | Updates three Pirates indicator timing fields and draws objective markers: while +0x39c is within the DAT_017ea5a4 cycle window it subtracts param_2 from +0x39… |

### `ui.processor`

**UIProcessor** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a97450` | UIProcessor::~UIProcessor | confirmed | Destructor body: stores UIProcessor::vftable, releases a refcounted list at this+0x4fc (per-node refcount decrement, node free via 00a8ce60), walks a second li… |
| `00a98020` | UIProcessor::UIProcessor | confirmed | Constructor body: stores AssetManagement::AssetRequestInterface::AssetRequester::vftable, then UIProcessor::vftable, initialises fields at this+0x8..0x2c and 0… |
| `00a98250` | UIProcessor::ScalarDeletingDtor | likely | Calls UIProcessor destructor body 00a97450 on this, then if the low bit of the flag argument is set calls _free on this; returns this. |

### `ui.profile`

**Engine::AccountGetAccountInfoForDisplayDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01294b00` | Engine::AccountGetAccountInfoForDisplayDisplayAction::OpenP… | confirmed | Returns 1. Reads arg 0 as an integer through FUN_012f8f60, using zero on a tag mismatch; sets AccountID, XML, HonorsXML, AvatarXML and MatchesXML on an OpenDia… |

### `ui.promotions`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a07c20` | DispatchHandlerPromotionsEvent | confirmed | Builds a StringFixed<256> initialised with the literal HandlerPromotions, appends the caller's event name through SoeUtil::IStringAppendFormat (00402ff0) with … |

### `ui.social`

**IgnoreListDataSource** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf9c90` | IgnoreListDataSource::IgnoreListDataSource | confirmed | Initializes the datasource base with BaseClient.IgnoreList, stores its constructor argument, installs the datasource vtables, and initializes the embedded Igno… |

### `ui.socialshare`

**SocialShareDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a714e0` | SocialShareDataSource::FieldName | confirmed | Switch on the int argument: returns the C string literal 'Provider' for 0, 'Status' for 1, 'Enabled' for 2, the address PTR_LAB_01815d30 for 3 and DAT_0180c430… |
| `00a71e00` | SocialShareDataSource::DtorBody | confirmed | Stores the SocialShareDataSource vtables at this+0, +0x30 and +0x34, then stores the SocialShareData array vtable at +0x254, truncates that array through 00a71… |
| `00a720f0` | SocialShareDataSource::Ctor | confirmed | Calls the base constructor 00922890 with the name string BBaseClient.SocialShare, installs the SocialShareDataSource vtables at +0, +0x30 and +0x34, installs t… |
| `00a721a0` | SocialShareDataSource::ScalarDeletingDtor | confirmed | Virtual slot [0]. Calls the dtor body 00a71e00 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |
| `00a71c60` | SocialShareDataSource::RebuildRows | likely | Calls 00769710 and returns 0 when the object at (result+4)+0x234 is zero. Otherwise it builds a header row from the literal 01813268 into a local 40-byte recor… |

### `ui.statdefinitions`

**StatDefinitionsDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a73af0` | StatDefinitionsDataSource::ColumnName | confirmed | Virtual slot [1]. Switch on the int argument: 0 returns Stat ID, 1 Icon ID, 2 Name, 3 Description, any other value returns <error>. Pure lookup; no state is re… |
| `00a73cf0` | StatDefinitionsDataSource::DtorBody | confirmed | Stores the StatDefinitionsDataSource vtables at this, +0x30 and +0x34, then calls 0091bf60 (accepted label Dtor_GuiUserDataTableAt34) with ECX = this. Does not… |
| `00a73d60` | StatDefinitionsDataSource::Ctor | confirmed | Calls the base constructor 00922890 with the name string BaseClient.StatDefinitions, then installs the StatDefinitionsDataSource vtables at +0, +0x30 and +0x34… |
| `00a73dd0` | StatDefinitionsDataSource::ScalarDeletingDtor | confirmed | Virtual slot [0]. Calls the dtor body 00a73cf0 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |
| `00a73c10` | StatDefinitionsDataSource::GetCell | likely | Virtual slot [4] (key, column, out). Gets the object through 00769710 and its +4 sub-object, then the table at +0x1d8 (returns 0 when that is null). Looks up t… |

### `ui.tintgroup`

**TintGroupDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a72780` | TintGroupDataSource::ColumnTitle | confirmed | Virtual slot [1]. Switch on the int argument: 0 returns the literal Tint Group ID, 1 returns the literal Data (<ID> - <Value>[,...]), any other value returns t… |
| `00a73610` | TintGroupDataSource::DtorBody | confirmed | Stores the TintGroupDataSource vtables at this, +0x30 and +0x34, sets a local cleanup flag, stores the HashMap<int,IString<unsigned_short>> vtable at +0x2a4 an… |
| `00a736c0` | TintGroupDataSource::Ctor | confirmed | Calls the base constructor 00922890 with the name literal BaseClient.TintGroups, stores its stack argument at this+0x254, installs the TintGroupDataSource vtab… |
| `00a73760` | TintGroupDataSource::ScalarDeletingDtor | confirmed | Virtual slot [0] of the secondary vtable. Calls the dtor body 00a73610 on this, then calls _free on this when bit 0 of its stack argument is set; returns this … |

### `ui.tree`

**GuiProperties<GuiTreeCtrl>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008379e0` | GuiProperties::SetPropertyValue | likely | Thiscall with two arguments (property id, value). Calls its vtable slot 12 (+0x30) and slot 27 (+0x6c) first and returns 0 when either handles the id. Otherwis… |
| `00837c40` | GuiProperties::CopyCtor | likely | Thiscall with one stack argument (source property table). Stores the GuiProperties<GuiTreeCtrl> vtable at +0, the Map<int,int> vtable at +0x14, two sentinel dw… |

**GuiTreeAccess** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00836200` | GuiTreeAccess::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor shape (ECX = this, one stack flag argument): calls the dtor body 00836110 on this, and when bit 0 of the flag is set calls … |
| `008361e0` | AdjustorThunkMinus4To00836200 | likely | Adjustor thunk: subtracts 4 from ECX and jumps to 00836200 (GuiTreeAccess scalar deleting destructor). |
| `008361f0` | AdjustorThunkMinus24To00836200 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 00836200 (GuiTreeAccess scalar deleting destructor). |

**GuiTreeCtrl** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00837e30` | GuiTreeCtrl::CopyCtor | confirmed | Thiscall with one stack argument (source GuiTreeCtrl). Calls 00429076 for the base constructor, stores the GFormCtrl vtables, copies the source's property tabl… |
| `008375e0` | GuiTreeCtrl::SetDsTreeAndRefresh | likely | Thiscall with one stack argument (data source). When the argument is null or does not dynamic_cast to GuiDsTree (00d8d382), it clears any existing link (when t… |

**GuiTreePrivateCtrl** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008357d0` | GuiTreePrivateCtrl::DtorBody | confirmed | Fastcall (ECX = this). Stores the GuiTreePrivateCtrl vtables at +0, +4 and +0x24, then calls 00616e55 on this and returns. |
| `00836220` | GuiTreePrivateCtrl::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor shape (ECX = this, one stack flag argument): calls the dtor body 008357d0 on this, and when bit 0 of the flag is set calls … |
| `00835b00` | GuiTreePrivateCtrl::HandleTreeMessage | likely | Thiscall with one stack argument (message record; the 16-bit message id is at record+0x1c). When the id equals DAT_01bc7488 it finds the enclosing object (0040… |
| `00836040` | AdjustorThunkMinus4To00836220 | likely | Adjustor thunk: subtracts 4 from ECX and jumps to 00836220 (GuiTreePrivateCtrl scalar deleting destructor). |
| `00836050` | AdjustorThunkMinus24To00836220 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 00836220 (GuiTreePrivateCtrl scalar deleting destructor). |
| `00836060` | AdjustorThunkMinus24To00835860 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 00835860 (GuiTreePrivateCtrl::ForwardThreeArgsTo005017f8). |
| `00836070` | AdjustorThunkMinus24To00835830 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 00835830 (GuiTreePrivateCtrl::ForwardToSubobjectSlot0). |
| `00836080` | AdjustorThunkMinus24To00835880 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 00835880, a jump thunk to 004c6dfd. |
| `00836090` | AdjustorThunkMinus24To00835840 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 00835840 (GuiTreePrivateCtrl::ForwardToSubobjectSlot1). |
| `008360a0` | AdjustorThunkMinus24To00835A40 | likely | Adjustor thunk: subtracts 0x24 from ECX and jumps to 00835a40 (GuiTreePrivateCtrl::StoreValueAndSentinel). |
| `008362c0` | GuiTreePrivateCtrl::OnPointerDownUpMove | likely | Thiscall with five stack arguments, ret 0x14, void. If the message equals DAT_01bc7690 and [this+0x1d8] is 0 and (param_4 & 0x10000) is set, it sets [this+0x1d… |

**(no class)** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c126a0` | PushTreeNodeUserdata | confirmed | Creates a 4-byte Lua userdata (FUN_00e3b570), stores the node pointer param_2 in it, attaches the metatable named "TreeNode" (FUN_00e3aca0 + FUN_00e3b000) and … |
| `00c12720` | CheckTreeNodeArg | confirmed | Type-checks the userdata at index param_2 (FUN_00e39bb0 type 7) and fetches it with metatable "TreeNode" (FUN_00e39b20); raises a Lua argument error on mismatc… |
| `00c128c0` | RegisterTreeNodeLuaModule | confirmed | Registers the TreeNode scripting class: FUN_00e39e40(param_1, "TreeNode", &table_0182b6d0, 0) with the methods GetText/GetBitmapIndex/GetDataCount/GetDataName/… |
| `00c129d0` | CheckTreeArg | confirmed | Type-checks the userdata at index param_2 (FUN_00e39bb0 type 7) and fetches it with the metatable whose name pointer is PTR_DAT_01b6b85c (the string "Tree", FU… |
| `00c12c60` | RegisterTreeLuaModule | confirmed | Registers the Tree scripting module: first calls FUN_00c128c0 (TreeNode class), then registers the module whose name pointer is PTR_DAT_01b6b85c ("Tree") with … |
| `00c12d40` | CastToGuiTreeCtrl | confirmed | Resolves the engine GUI object behind a userdata/argument: calls FUN_00705bc0(param_1, &local) to unwrap it, checks the result with FUN_00819650 and then runs … |
| `00c12a30` | TreeGetNodeForRow | likely | Lua C function bound as Tree.GetNodeForRow: requires exactly 2 arguments, fetches the tree with FUN_00c129d0(arg1), reads the row index from arg2 (FUN_00e3a7b0… |
| `00c12b10` | TreeGetSelectedNode | likely | Lua C function bound as Tree.GetSelectedNode: fetches the tree with FUN_00c129d0(arg1), asks the control for its selected node with FUN_00836670(); when a node… |
| `00c12b80` | TreeGetSelectedRow | likely | Lua C function bound as Tree.GetSelectedRow: fetches the tree with FUN_00c129d0(arg1), reads the selected row through FUN_008365f0 and pushes it as an integer … |
| `00c12bb0` | TreeGc | likely | Garbage-collector callback (__gc of the Tree userdata metatable): fetches the userdata at index 1 with FUN_00e3a930, raises the "Tree" argument error on mismat… |

### `ui.video`

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c05820` | VideoLibrary_GetVideoLibraryVideoInfo | likely | Lua C function: requires exactly one argument (FUN_00e3a390(L)==1), reads it as a number (FUN_00e3a7b0), resolves the video-info record through FUN_008bb040, a… |
| `00c05880` | VideoCapture_GetVideoCaptureOutputFile | likely | Builds a stacked SoeUtil::IString (installs IString vftable, points at the static empty buffer), fills it through FUN_008ef4f0, pushes the resulting string as … |
| `00c05940` | VideoLibrary_GetVideoLibraryMyVideosDirectory | likely | Same shape as the other VideoCapture string getters: builds a stack SoeUtil::IString, fills it through FUN_008a86d0 (the My-Videos directory query), pushes the… |
| `00c05ac0` | VideoLibrary_SetVideoLibraryVideoInfoParam | likely | Requires 3 Lua arguments: index (FUN_00e3a7b0), name and value strings (FUN_00e3a820); rejects a negative index, empty name or null value; fills two stack IStr… |
| `00c05bd0` | VideoLibrary_GetVideoLibraryVideoInfoParam | likely | Requires 2 Lua arguments (FUN_00e3a390(L)==2): record index (FUN_00e3a7b0) and parameter name (FUN_00e3a820); fills a stack IString through FUN_008d3580(index,… |
| `00d2e2b0` | IStringQuad_Ctor | likely | Constructor of a four-IString block: installs the IString vptr and empty buffer in each of the members at +0x00, +0x10, +0x20 and +0x30, then assigns the capti… |

### `ui.view`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c13a80` | ZoomCamera | likely | Lua C function bound as View.ZoomCamera: reads arg1 as a floating-point number (FUN_00e3a770); when it is non-zero it fetches the client singleton (FUN_0076971… |
| `00c13ad0` | RegisterViewLuaModule | likely | Registers the View scripting module: FUN_00e39e40(param_1, module_name_ptr_01b6ba04, &table_0182b900, 0) where the table holds SetCameraDistance, SetCameraPitc… |

### `ui.whatsnew`

**WhatsNewDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a3e960` | WhatsNewDataSource_RowCount | confirmed | Row-count getter: when the flag byte at +0x22c is clear it first calls the rebuild routine 00a3e670, then returns the element count field (+8) of the array obj… |
| `00a3e1b0` | WhatsNewDataSource_FieldName | likely | Field-name lookup of the what's-new data source: a switch over the field index returning "Id" (0), "Priority" (1), "IconId" (2), "TitleStringId" (3), "BodyStri… |
| `00a3e670` | WhatsNewDataSource_Rebuild | likely | Rebuild of the what's-new rows: calls virtual slot +0xc of the source object held at this+0x34, then loops the index 0..count-1 (count read from the array obje… |

**WhatsNewInfo** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a3e990` | EmbeddedStrings3_Dtor_18_34_44 | confirmed | Destructor of a record holding three SoeUtil::IString fields at +0x18, +0x34 and +0x44: for each field it installs the base IString vtable, and when the length… |
| `00a3ea70` | EmbeddedStrings3_Ctor_18_34_44 | confirmed | Constructor of the three string fields at +0x18, +0x34 and +0x44: for each it installs SoeUtil::IString<char>::vftable, points the buffer at the shared static … |
| `00a3efb0` | EmbeddedStrings3_CopyCtor_18_34_44 | confirmed | Copy-constructs a row: copies six leading dwords, initializes and copies the IString at +0x18, copies three dwords at +0x28..+0x30, then initializes and copies… |
| `00a3edb0` | ScalarDelete_a3edb0 | likely | Scalar deleting destructor: calls the three-string destructor body 00a3e990 and then calls _free(this) when bit 0 of the flag argument is set; returns this oth… |

## Source shards

[[S0001 00401040-00406b84|S0001]] (91), [[S0002 00406c2f-0040afca|S0002]] (39), [[S0003 0040affb-004103c5|S0003]] (30), [[S0004 00410405-004149c8|S0004]] (114), [[S0005 00414a48-00419846|S0005]] (74), [[S0006 00419961-0041e544|S0006]] (82), [[S0007 0041e58b-00422a66|S0007]] (16), [[S0008 00422dcc-00426a69|S0008]] (14), [[S0009 00426b13-0042b8f1|S0009]] (87), [[S0010 0042b921-004309b5|S0010]] (32), [[S0011 004309d4-00436f31|S0011]] (85), [[S0012 00437372-0043dc9a|S0012]] (57), [[S0013 0043de42-004435e4|S0013]] (62), [[S0014 00443617-00447db4|S0014]] (53), [[S0015 00447e1a-0044e19f|S0015]] (53), [[S0016 0044e1d3-0045302b|S0016]] (94), [[S0017 004530c5-00457488|S0017]] (12), [[S0018 004574f6-0045cca3|S0018]] (100), [[S0019 0045ccf9-00462140|S0019]] (11), [[S0020 004621ca-0046734c|S0020]] (1), [[S0022 0046b77b-0046efeb|S0022]] (28), [[S0023 0046f261-00472f3a|S0023]] (6), [[S0024 00472fd2-004778b3|S0024]] (71), [[S0025 00477947-0047ca0c|S0025]] (2), [[S0026 0047ca5e-004823b9|S0026]] (181), [[S0027 00482437-004870e5|S0027]] (73), [[S0028 0048741e-0048c4f5|S0028]] (39), [[S0029 0048c619-00491c2c|S0029]] (129), [[S0030 00491cd7-004963f2|S0030]] (18), [[S0040|S0040]] (138), [[S0041|S0041]] (88), [[S0048|S0048]] (100), [[S0049|S0049]] (176), [[S0069|S0069]] (31), [[S0100|S0100]] (103), [[S0113|S0113]] (103), [[S0127|S0127]] (8), [[S0141|S0141]] (11), [[S0143 00701a20-00707400|S0143]] (14), [[S0149 00719de0-0071efe0|S0149]] (2), [[S0171|S0171]] (1), [[S0195|S0195]] (11), [[S0196 0081b7d0-00821370|S0196]] (46), [[S0197 00821940-00827640|S0197]] (74), [[S0200|S0200]] (111), [[S0217|S0217]] (1), [[S0218 00896b90-0089d930|S0218]] (3), [[S0219 0089d990-008a3c80|S0219]] (1), [[S0220 008a3ce0-008a8500|S0220]] (7), [[S0221 008a86b0-008b1d80|S0221]] (3), [[S0223 008b7790-008bcf40|S0223]] (3), [[S0225 008c52a0-008ca980|S0225]] (3), [[S0227 008d0920-008d5c60|S0227]] (3), [[S0229 008d92f0-008dea46|S0229]] (1), [[S0231|S0231]] (1), [[S0232 008e82e0-008ec1d0|S0232]] (2), [[S0233 008ec2a0-008f0cc0|S0233]] (2), [[S0236|S0236]] (3), [[S0237 008ff3d0-00904e70|S0237]] (14), [[S0239 0090a890-009105b0|S0239]] (12), [[S0241 009162f0-0091aa10|S0241]] (3), [[S0242 0091af50-009200d0|S0242]] (1), [[S0243 00920130-00925ec0|S0243]] (6), [[S0247 00934740-00939200|S0247]] (2), [[S0248 009392c0-0093d740|S0248]] (2), [[S0253|S0253]] (25), [[S0256 009606e0-00965340|S0256]] (2), [[S0257|S0257]] (6), [[S0260|S0260]] (9), [[S0263 00982f10-009897a0|S0263]] (4), [[S0264 00989810-0098eae0|S0264]] (19), [[S0266 00995720-0099c9a0|S0266]] (11), [[S0267 0099ca10-009a1c60|S0267]] (23), [[S0268 009a1ca0-009a6ce0|S0268]] (12), [[S0270 009acf20-009b2140|S0270]] (1), [[S0271 009b2210-009b7170|S0271]] (1), [[S0274 009c2cc0-009c8d80|S0274]] (4), [[S0275 009c8e90-009cdee0|S0275]] (4), [[S0276 009cdf80-009d3fe0|S0276]] (7), [[S0280 009e4570-009ea840|S0280]] (10), [[S0283 009f39b0-009f8370|S0283]] (2), [[S0284 009f8470-009feec0|S0284]] (34), [[S0285 009fef30-00a059e0|S0285]] (8), [[S0286 00a05a50-00a0c390|S0286]] (1), [[S0288 00a13190-00a18a10|S0288]] (13), [[S0289 00a18a90-00a1dd30|S0289]] (5), [[S0291 00a244a0-00a29500|S0291]] (1), [[S0294 00a35150-00a39a80|S0294]] (6), [[S0295 00a39b00-00a3f350|S0295]] (7), [[S0296 00a3f560-00a45650|S0296]] (14), [[S0297|S0297]] (7), [[S0298 00a4b480-00a53380|S0298]] (10), [[S0300 00a59220-00a5da20|S0300]] (1), [[S0301 00a5da90-00a62140|S0301]] (6), [[S0302 00a62280-00a67470|S0302]] (2), [[S0305 00a707f0-00a75b20|S0305]] (24), [[S0306 00a75b70-00a7b4d0|S0306]] (23), [[S0307 00a7b930-00a81420|S0307]] (4), [[S0308 00a81470-00a87b30|S0308]] (15), [[S0309 00a87ba0-00a8cdc0|S0309]] (1), [[S0310 00a8ce60-00a91b00|S0310]] (10), [[S0311 00a91bf0-00a97400|S0311]] (7), [[S0312 00a97450-00a9d360|S0312]] (4), [[S0317 00ab2c80-00ab7380|S0317]] (1), [[S0318 00ab73e0-00abcb30|S0318]] (7), [[S0322 00acd6d0-00ad3250|S0322]] (5), [[S0323 00ad3270-00ad83b0|S0323]] (4), [[S0324 00ad8880-00adee10|S0324]] (1), [[S0334 00b0a370-00b10840|S0334]] (6), [[S0337 00b1b5a0-00b21f20|S0337]] (5), [[S0341 00b33bc0-00b39af0|S0341]] (2), [[S0347 00b56090-00b5b410|S0347]] (3), [[S0348 00b5b470-00b5ff50|S0348]] (1), [[S0352 00b6cd80-00b71320|S0352]] (1), [[S0353 00b71490-00b77eb0|S0353]] (1), [[S0358 00b8a640-00b906a0|S0358]] (3), [[S0366 00bb11b0-00bb7710|S0366]] (3), [[S0372 00bd37e0-00bd5d40|S0372]] (4), [[S0374 00bd9880-00be0090|S0374]] (4), [[S0375 00be0120-00be58f0|S0375]] (12), [[S0379|S0379]] (2), [[S0380 00bf5ff0-00bf9410|S0380]] (6), [[S0381 00bf9430-00bfcf30|S0381]] (6), [[S0383 00c05780-00c0db00|S0383]] (26), [[S0384 00c0dcd0-00c171d0|S0384]] (52), [[S0385 00c17390-00c1f0a0|S0385]] (8), [[S0386 00c1f190-00c26aa0|S0386]] (16), [[S0387 00c26bb0-00c2ddf0|S0387]] (169), [[S0388 00c2df40-00c34100|S0388]] (67), [[S0394|S0394]] (20), [[S0396|S0396]] (1), [[S0398|S0398]] (24), [[S0405 00c89e70-00c8f7c0|S0405]] (65), [[S0406 00c8f830-00c95560|S0406]] (31), [[S0407 00c955b0-00c9baf0|S0407]] (67), [[S0411 00cab620-00cb0bb0|S0411]] (4), [[S0412 00cb0bf0-00cb6b80|S0412]] (41), [[S0413 00cb6d20-00cbc7f0|S0413]] (4), [[S0414 00cbc8e0-00cc18f0|S0414]] (4), [[S0415 00cc1a00-00cc6bc0|S0415]] (29), [[S0416 00cc6c40-00ccbaa0|S0416]] (5), [[S0421 00ce0340-00ce4c30|S0421]] (3), [[S0422 00ce4e30-00ceb920|S0422]] (56), [[S0424 00cf1430-00cf68d0|S0424]] (5), [[S0425 00cf69c0-00cfc480|S0425]] (21), [[S0426 00cfc4e0-00d01c70|S0426]] (24), [[S0427 00d01e40-00d07440|S0427]] (17), [[S0437 00d297c0-00d2f870|S0437]] (4), [[S0499|S0499]] (22), [[S0542|S0542]] (6), [[S0581|S0581]] (1), [[S0592|S0592]] (1), [[S0594|S0594]] (78), [[S0595|S0595]] (20), [[S0596|S0596]] (10), [[S0653|S0653]] (65), [[S0654 012393a0-0123f010|S0654]] (6), [[S0658 0124dfe0-01253ee0|S0658]] (6), [[S0671 01291250-01295650|S0671]] (3), [[S0686|S0686]] (9), [[S0689|S0689]] (14), [[S0690|S0690]] (1), [[S0691|S0691]] (6), [[S0693|S0693]] (15), [[S0721|S0721]] (1), [[S0727 013be8e0-013c3b90|S0727]] (10), [[S0738|S0738]] (12), [[S0790|S0790]] (7), [[S0806 00406512-00621d93|S0806]] (35), [[S0807 0062745d-006373db|S0807]] (10), [[S0808 0063813b-006539f7|S0808]] (9), [[S0809 0066ca4d-0147b010|S0809]] (15), [[S0825|S0825]] (220), [[S0837|S0837]] (220), [[S0838|S0838]] (220), [[S0842|S0842]] (220), [[S0843|S0843]] (220), [[S0852|S0852]] (1)
