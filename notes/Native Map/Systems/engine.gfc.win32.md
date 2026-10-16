# System: engine.gfc.win32

66 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00446576` | `RegisterInputAndWindowManagers` |  | Bring-up entry point that installs both Win32 GFC managers (input and window) and reports success through its return value. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `00825320` | `CreateWindowManagerAndSystemWindow` |  | Creates the GFC window manager and the client system window, with the GDraw D3D9 driver and window icon. | [[Native Map/Shards/S0197 00821940-00827640|S0197]] |

## 🟢 likely (47)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `004450cf` | `SetAttachedWindowAndAddRef` |  | Base-class helper of the GFC window objects: store an owner/window interface pointer with correct reference counting; its callers include the accepted WindowWin32Ctor 0044e915 and WindowManagerCtor 004502c8. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `0044581a` | `BuildLimitsRecordCallVirtual70` |  | Thin wrapper of an eight-argument call with sentinel limits (0x7fffffff/0x80000000/0xfffffffe/1) around virtual slot +0x70; no static caller in this build, so the object owning slot +0x70 is unidentified. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `004458e0` | `CopyWindowRecord15Dwords` |  | Memberwise copy constructor for the data record embedded in the Win32 window objects (text plus geometry fields). | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `00445d5b` | `AllocAndInitObject1763570` |  | Factory of the small 01763570 helper object; no static caller in this build. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `00445d80` | `AllocAndInitObject1763408` |  | Lazy factory of the 01763408 helper object of a window; built through the window's own method 00445d80. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `00445e99` | `CreateViaVirtual6C` |  | Object-creation path that asks for a sub-object through vtable slots +0x68/+0x64 and then calls virtual slot +0x6c on it with flag 0x1003120; no static caller in this build. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `0044728a` | `WindowBaseCopyCtor` |  | Base copy constructor shared by the window copy constructors; its callers include the accepted WindowManagerCopyCtor 0044e221, WindowWin32CopyCtor 0044e661 and WindowHandleCtorVtables1763F70 0044f0fd. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `0044e1d3` | `DetachAndReleaseField30` |  | Cleanup step called from the 01763cd0 window destructor 0044f325; what the +0x94 partner represents is not established (no writer of +0x94 is visible in this range). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044e221` | `WindowManagerCopyCtor` |  | GFC class-factory copy construction (action 1/3 of the class-ops dispatch 004504e9) for the Win32 window-manager class. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044e336` | `InvokeSlot44ThenSlot20` |  | Interface-forwarding thunk: asks the receiver for a subobject/interface through slot +0x44 and tail-jumps to its slot +0x20; no callers are recorded, so the window/interface context is inference. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044e3e6` | `CollectRectWithReducedEdges` |  | Passed as the EnumDisplayMonitors callback by 0044e41b; the callback's vector append semantics come from FUN_0042c4f7. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044e41b` | `EnumerateMonitorRectsForWindow` |  | Win32 helper that gathers monitor rectangles for a window object; API availability and fallback behavior are established by this function. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044e661` | `WindowWin32CopyCtor` |  | Class-factory copy construction (action 1/3 of dispatch 0045018a) for the Win32 window object class; the prototype is the GFC class-registry template instance. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044e86d` | `InvokeSlot44ThenSlot20_2` |  | Same forwarding helper as 0044e336, emitted separately for another translation unit of the Win32 window layer. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044eaa3` | `Win32WindowProc` |  | Window procedure registered by 0044f3ab for GFC_StdWindowClass; Win32 message handling is visible in this function. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044ee01` | `DtorVtables1763F00` |  | Class-level destructor steps of the Win32 top window object, tearing down children and releasing the container/base before the base destructor runs. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044eec4` | `AllocateAndRegisterIdEntry` |  | Allocates and registers an ID-bearing record. The callers 0044ef32 and 0044f51f associate it with window initialization and attachment; this body alone does not establish a wider handle-table schema. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044ef32` | `WindowInitLateHandleEntries` |  | Initialises the window's late-bound handle slots (ids kept at +0xF4/+0xF8/+0xFC) during window construction (called from the 01763f00 ctor 0044f6e1). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044efb1` | `CollectMatchingElementFieldPointers` |  | Collects addresses of matching element fields into the caller-provided vector; the reason for collecting them is not established, and the old teardown/release attribution was incorrect. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f06c` | `ClassOpsVtables1763BA0` |  | GFC class-registry operation table (construct/allocate/destroy/delete) for an anonymous Win32-layer class; reachable from the registry with an action code. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f0fd` | `WindowHandleCtorVtables1763F70` |  | Class-factory construction (action 1/3 of dispatch 0044f7bc) for a Win32 window object class; the prototype is the registry template instance. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f203` | `InvokeSlot44ThenSlot20_3` |  | Same interface-forwarding helper as 0044e336/0044e86d, emitted for a third translation unit of the Win32 window layer. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f222` | `InvokeSlot44ThenSlot54` |  | The short x86 body returns the stack argument after the virtual call; the pseudocode's unaff_ESI return is stale. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f252` | `DtorVtables1763F70` |  | Destructor of a Win32 window object class in the 01763f70 family. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f281` | `CopyCtorVtables1762F68` |  | From-prototype copy constructor of the 01762f68 class: class-ops dispatcher 0044f9b8 routes actions 1/3 here (FUN_0042a8b7 copies the source fields) while the source-less constructor FUN_004433ab serves actions 0/2. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f325` | `DtorVtables1763CD0` |  | Destructor of the 01763cd0 window object class family used by the Win32 window manager children. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f354` | `DtorVtables1763D48` |  | Destructor of the 01763d48 window-manager class; the global it clears is a registered Win32 message hook, not an instance pointer. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f59c` | `WindowWin32CopyCtor_1763F00` |  | Class-factory copy construction (action 1/3 of dispatch 00450237) for the top window object class. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f6c5` | `ScalarDeletingDtor1763F00` |  | Standard MSVC scalar deleting destructor emitted for this window class (vtable slot 0, invoked by the class-ops action 5). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f6e1` | `WindowCtorVtables1763F00` |  | Constructor for the object using vtable 01763f00; top-level window role is inferred from window-manager callers and neighboring class operations. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f7bc` | `ClassOpsVtables1763F70` |  | GFC class-registry operation table (construct/allocate/destroy/delete) for this window class. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f820` | `ScalarDeletingDtor1763F70` |  | Standard MSVC scalar deleting destructor for this window class (class-ops action 5). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f83c` | `WindowCtorChild_1763F70` |  | Child-window construction in the 01763f70 family, selected by kind 4 in the child factory 004503b4. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f9b8` | `ClassOpsVtables1762F68` |  | GFC class-registry operation table for the 0x34-byte helper class pinned by the vtable pair 01762f68/01762f60 (which layer registers it is not evidenced by this body). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044fa68` | `ClassOpsVtables1763CD0` |  | GFC class-registry operation table for the 0xA8-byte window class used for the window's child windows (this emission has no action-0/2 branch). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044facc` | `ScalarDeletingDtor1763CD0` |  | Standard MSVC scalar deleting destructor for this window class (class-ops action 5). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044fae8` | `WindowCtorChild_1763CD0_Kind5` |  | Creates the kind-5 child window object cached at the parent's +0xEC slot inside the child factory 004503b4. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044fb65` | `WindowCtorChild_1763CD0_NativeB` |  | Creates the first of the two child window objects of a native window (the parent's +0x104 slot); called from 0044fd43. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044fbe6` | `WindowCtorChild_1763CD0_NativeA` |  | Second creation path for the 0xA8-byte child window object, called from 0044fc80. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044fc64` | `ScalarDeletingDtor1763D48` |  | Standard MSVC scalar deleting destructor for the window-manager class. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0045018a` | `ClassOpsVtables1763DD0` |  | GFC class-registry operation table for the 0x110-byte window class (window object with cursor field at +0xE4). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0045021b` | `ScalarDeletingDtor1763DD0` |  | Standard MSVC scalar deleting destructor for this window class (class-ops action 5). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00450237` | `ClassOpsVtables1763F00` |  | GFC class-registry operation table for the 0x150-byte top window object class. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `004502c8` | `WindowManagerCtor` |  | Creates the Win32 window manager instance and the top-level window object with its child windows; the class descends from GWindowManagerWin32. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0045035d` | `WindowManagerCtorWithParent` |  | Construction path used by the child factory 004503b4 for kind-2 child objects (the manager owns the created window). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `004503b4` | `WindowCreateChildByKind` |  | Creates and caches one of three child objects. The optional lookup supplies a kind when param_2 is nonzero; param_3 is the direct kind input when param_2 is zero. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `004504e9` | `ClassOpsVtables1763D48` |  | GFC class-registry operation table for the 0x80-byte window-manager class. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |

## 🟡 speculative (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0044eb75` | `ChildListRemoveElementsOwnedBySelf ?` |  | The body proves a filtered unlink by owner identity; teardown intent comes only from its caller context, and there are no direct callers in this range. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044ec04` | `PrepareAndRemapByteArray ?` |  | Prepares a Win32-dependent conversion path and rewrites byte-array values according to their low three bits; the actual surface or mask significance is unknown. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044ed06` | `QueryRecordsAndConvert ?` |  | Converts the entry list returned by two resolved Win32 calls into callback objects and hands the first record's three dwords back through param_2; which Win32 entry points the two globals hold is unresolved in this rang… | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f2a0` | `InitSelfLinkedListThunk ?` |  | Out-of-line wrapper around the list-head initializer. The wrapper has no recorded callers, so its use by Win32 window classes is unproven. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |

## ⚪ unknown (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00447303` | `— ⚠reject` |  | Field handling (position cache, wheel accumulator, listener dispatch, recursion) is verified, but the function has no static caller and no pointer to its address anywhere in the image, so which window/device class owns … | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `00447a12` | `— ⚠reject` |  | Keyboard-side counterpart of 00447303; the +0x12C active byte and +0x140 device pointer match the accepted keyboard-device family, but the function has no static caller and no pointer to its address in the image, so the… | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `0044e369` | `— ⚠reject` |  | Teardown step that destroys all child windows of the owning window object during window destruction. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044e4e8` | `— ⚠reject` |  | GCompat clipboard reader that exposes Win32 clipboard formats as GFC GString/bitmap objects (the GClipboardAccess_Win32_TEXT / GClipboardAccess_GStringObject bridge). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044e915` | `— ⚠reject` |  | Plain constructor for the GFC Win32 window object class (sibling of the GWindowWin32 ctor 00441174 which uses only the base vtable). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044e9b0` | `— ⚠reject` |  | Destroys this object's native HWND and detaches it from its owner, running the self-cleanup path when it is the last reference. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f3ab` | `— ⚠reject` |  | Creates the native window class that every GFC Win32 window object is instantiated from (CreateWindowExA/W in 0044fd43). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f4ad` | `— ⚠reject` |  | Destructor of the 0x110-byte Win32 window object class constructed by 0044e915/0044e661. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f51f` | `— ⚠reject` |  | Attaches a window object to either an owner interface or a registered handle entry; used by the window constructors together with the 0x302 window type tag (kind codes 0/3/4/5 correspond to top window, child dialogs and… | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f8b9` | `— ⚠reject` |  | Second child-window construction path (from 0044fc80) for the same 0x158-byte class, linked back to the owner window. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044f937` | `— ⚠reject` |  | Creates the second of the two child window objects that a native GFC window owns (the +0x104 slot of the parent). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044fc80` | `— ⚠reject` |  | Completes window construction: registers the Win32 class, links the window to its owner and materialises its two child window objects. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0044fd43` | `— ⚠reject` |  | The GWindowWin32 creation routine: turns a GFC window object plus a create record into a real HWND whose messages are dispatched through GCFStdWindowProc. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |

