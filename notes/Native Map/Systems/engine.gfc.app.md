# System: engine.gfc.app

83 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00405d3d` | `GApplication::InitializeControlManager` | GApplication | Wires the GUI control manager into the GFC application and its message routing during application initialization. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00411f68` | `GCtrlTheme_Infinity::Load` | GCtrlTheme_Infinity | Loads/attaches the control-theme driver (GCtrlTheme_Infinity) used by the GFC control drawing code. | [[Native Map/Shards/S0004 00410405-004149c8|S0004]] |
| `0043eb16` | `GuiApp::InitializeControlsA` | GuiApp | GuiApp virtual initializer variant that calls 0043D9CD before the shared control-manager initialization. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0043ebc0` | `GuiApp::InitializeControlsB` | GuiApp | GuiApp virtual initializer variant that calls 0043DC9A before the shared control-manager initialization. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0043ec67` | `GuiApp::InitializeControlsC` | GuiApp | GuiApp virtual initializer variant that calls 0043DE42 before the shared control-manager initialization. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0043fcfb` | `GuiApp::InitializeControlsFromStoredSize` | GuiApp | GuiApp virtual initializer that builds the control manager from the stored/default window size through 0043F98D. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |

## 🟢 likely (47)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00405ddc` | `GApplication::ShutdownControlManager` | GApplication | Counterpart of the control-manager initialization; detaches the GUI control manager on application shutdown. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405ef4` | `TrySetState3A6` |  | Guarded display-state update that is skipped while the busy flag is set. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00434a67` | `DispatchApplicationEvent` |  | Top-level window/application event handler of the GFC application object: focus/activation, resize, move, visibility and one class-resolved event are applied to the main control container and propagated to children. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00436eb1` | `GuiAppShutdownUI` | GuiApp | GuiApp shutdown override (slot 86, byte +0x158, of the GuiApp vtable) that tears down the control-manager state and clears the shutdown flag; its callee 00405ddc is the already-labelled GApplication::ShutdownControlMana… | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `0043baad` | `GApplication::CleanupControlManagerState` | GApplication | Application cleanup path; the direct GApplication shutdown helper receives the same object as this function. | [[Native Map/Shards/S0012 00437372-0043dc9a|S0012]] |
| `0043e7db` | `CtorVtable0175F520` |  | Constructor for the object using vtable family 0175F520; its object role is not named in this function. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0043ea76` | `CtorDefaultVtable0175F520` |  | Parameterless constructor for the object using vtable family 0175F520; its object role is not named in this function. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0043ead5` | `DtorVtable0175F520` |  | Destructor body paired with the object constructors 0043E7DB and 0043EA76; three EH unwind funclets also call it. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0043f98d` | `CreateControlManagerFromDisplays` |  | Computes a desktop-sized control-manager setup; its only caller is the GApplication::InitializeControlsFromStoredSize variant 0043FCFB, which supplies the stored (or default) width/height. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0043fc7a` | `CreateInstance0175F520` |  | Mode dispatcher for the object using vtable family 0175F520: mode 0 constructs in place, mode 2 allocates 0x3A8 bytes then constructs, mode 4 destroys in place, and mode 5 calls the object vtable with delete flag 1. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0043fcdf` | `ScalarDeletingDtor0175F520` |  | Scalar deleting destructor paired with the object using vtable family 0175F520. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004485c9` | `BaseCtorVtableP17635C8 → BaseCopyCtorVtableP17635C8` |  | Base constructor reused by the small GFC object classes at 0044a326 and 0044a9fe, which overwrite offset 0 with their own vtable immediately afterwards. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `00449f5b` | `WindowApplyWin32Style` |  | Win32 style update on the HWND-bearing object; synchronous versus asynchronous message delivery depends on the callback result from slot +0xc0. The related window class remains an ownership hypothesis. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044a326` | `BaseCopyCtorVtableP1763790` |  | Base-constructor variant used by the window copy-constructor path at 0044f0fd, which installs final window vtables afterward; the owning class is not established here. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044a9fe` | `SmallObjectCtorVtableP1763A40` |  | Constructor for another member of the small GFC helper-class family; subclasses at 0044fae8/0044fb65/0044fbe6 re-use the same vtable. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044aa16` | `TreeResetOrFreeFirstRecord` |  | Tree/cache housekeeping helper; its caller drains an owned object list before invoking it. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044b071` | `StlVecFindIteratorA` |  | Find-iterator builder used by the erase-by-key member 0044b1dc. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044b1dc` | `StlVectorEraseByKey` |  | Unlink helper used by the object lookup/cache in the GFC window layer (callers 0044e9b0/0044eaa3 outside this shard). | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044b255` | `DrainRegisteredObjectList` |  | Cleanup helper called by the destructor at 0044ee01; the element interface meaning is unknown. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044b296` | `NextUniqueTreeKey` |  | Likely supplies an unused tree key to the entry-registration call at 0044eec4; that caller context suggests an ID role, while this function itself only proves the counter and collision-search mechanics. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044c949` | `MultiVtableCtorCloneBC` |  | Constructor of a class derived from the 005383ea base (a UI/window-layer object with five interface vtables); the sibling constructor at 0044d1de creates the same shape without a source object. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044ca67` | `EraseTreeEntryAndFree` |  | Record removal helper called by the cached-record reset at 0044db39; record ownership semantics remain unknown. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044caed` | `VectorCopyConstructA` |  | Container clone used while constructing the window-holding objects of this translation unit. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044cb77` | `StlVectorResizeC` |  | Resize member (called from 0044e41b outside this shard). | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044cc10` | `VectorCopyConstructB` |  | Companion clone for a second vector member of the same object family. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044cca8` | `StlVectorInsertAtA` |  | Insert-one member used by the push_back member 0044dcd9 when the buffer must grow. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044ceaf` | `StlVectorInsertAtB` |  | Insert-one member used by the push_back member 0044ddfb. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044cf12` | `StlVectorInsertAtC` |  | Insert-one member used by the push_back member 0044de50. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044cf75` | `StlVectorResizeA` |  | Resize member of one instantiation (called from 0044cb77's siblings). | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044d000` | `StlVectorInsertAtD` |  | Insert-one member used by the push_back member 0044deb3. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044d063` | `StlVectorInsertAtE` |  | Insert-one member used by the push_back member 0044df08. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044d0c6` | `StlVectorResizeB` |  | Resize member of the counted-assign instantiation. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044d166` | `StlVectorInsertAtF` |  | Insert-one member used by the push_back member 0044df6b. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044d1de` | `MultiVtableCtorP1763BA0` |  | Default constructor of the five-vtable class whose clone constructor is 0044c949. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044d21d` | `ScalarDeletingDtorA` |  | Scalar deleting destructor adapter; the owning class is not established in this function. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044d2ef` | `WindowChildAt` |  | Child accessor of the Win32 window object (HWND +0x34, flag +0x3c) used by the child-update loop; the GWindowWin32 name is a neighbouring string (00441141), a hypothesis rather than a binding. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044d34d` | `WindowUpdateChildren` |  | Bulk show/hide-and-refresh of child windows (the +0xbc slot is the per-child visibility/notification entry); window-object ownership is a hypothesis from the neighbouring GWindowWin32 string, not bound here. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044d449` | `WindowDisplayStateCommand` |  | Implements the display-state half of the window object: the same 0xF0xx values the window procedure will receive back as WM_SYSCOMMAND; ownership hypothesis GWindowWin32 from the neighbouring name string, not bound here. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044dc54` | `ReadClipboardText` |  | Clipboard paste source for the GFC text controls (the string it returns is the GStringObject created from the clipboard handle). | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044dcb3` | `PasteClipboardText` |  | Entry point that turns clipboard contents into a GStringObject for the caller (multi-step cleanup around the read). | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044de50` | `StlVectorPushBackC` |  | push_back member of a third vector instantiation. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044df6b` | `StlVectorPushBack4F` |  | Push-back wrapper for 4-byte elements. Raw bytes show the full-capacity branch builds its iterator from the current end pointer; the caller 0044efb1 supplies the element context. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044dfd0` | `CachedObjectRelease` |  | Runs the reset 0044db39 and then releases the object cached at +0x14 (called from the containing-class destructors at 0044f325/0044f252). | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044e0db` | `SetCachedObjectWithTimeStamp` |  | Timestamped cache store (the helper 004f9372 is GetLocalTime-based); the hover/drag use is a hypothesis, not proven by this function. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044e180` | `InvokeVirtual44Then20` |  | Two-step virtual query used by the window-layer objects (slot 0x44 yields a helper object whose slot 0x20 answers). | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044e19f` | `InvokeVirtual44Then54` |  | Sibling of 0044e180 that forwards the caller's argument to the secondary object. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `008d21d0` | `UpdateLoadingScreenAndAbortCheck` |  | Main-loop tick that paces the client with a clamped delta, pumps the loading screen and converts gateway stalls into visible abort/error output; caller 00936d00 runs it. | [[Native Map/Shards/S0227 008d0920-008d5c60|S0227]] |

## 🟡 speculative (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00405c54` | `InvokeVirtualFC ?` |  | Four-argument virtual dispatch helper used by the display-system callers in the 0x42xxxx region. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405d19` | `DisplayStateInit ?` |  | Startup/reset of the application's display-state block. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405e03` | `SetWord3A4AndRefresh ?` |  | Sets one display-state field and triggers the application refresh virtual. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405e2b` | `SetWord398AndRefresh ?` |  | Display-size/style field setter on the application object. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405e42` | `SetWords398AndRefresh ?` |  | Display-size setter (width/height pair) on the application object. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405f2f` | `MaybeInvokeVirtual9C ?` |  | Conditional two-argument dispatch through the held object. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405f8c` | `QueryOrForward ?` |  | Early-out query that forwards to another component only once the application is ready. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405fb7` | `DrainQueue1A4 ?` |  | Drains the queue/processor referenced by +0x1A4 before continuing. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405fde` | `InvokeVirtual38With3C ?` |  | Two-step virtual dispatch helper. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `007055d0` | `GApplicationBaseDefaultConstructorClosure ?` | GApplicationBase | Invokes the GApplicationBase default-construction routine with a zero initialization argument. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007055e0` | `GApplicationDefaultConstructorClosure ?` | GApplication | Invokes the GApplication default-construction routine with a zero initialization argument. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |

## ⚪ unknown (19)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00405ca7` | `— ⚠reject` |  | Stamps an object pair with a fixed two-dword marker; the same marker pair is written by 00406346, 00406550 and 00406803. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `0041454a` | `— ⚠reject` |  | Loads a themed bitmap for the control through the GBitmapCtrlThemeLoader driver (two control-side callers). | [[Native Map/Shards/S0004 00410405-004149c8|S0004]] |
| `004187fa` | `— ⚠reject` |  | Initializes the bitmap control theme of the GFC application (the warning text names GApplication::Initialize); it is the object-side loader for the theme DLL, related to the theme strings stored next to the layout attri… | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `0043de42` | `— ⚠reject` |  | One of the three control-manager setup paths reached from the GApplication::Initialize variants: it attaches the two interface objects and their helper, matching the sibling setups FUN_0043D9CD / FUN_0043DC9A called by … | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0044aa94` | `— ⚠reject` |  | Caches a keyed tree lookup. Callers 0044f51f and 004503b4 use the result in window owner/child paths, but this body does not identify the table or record type. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044ac0c` | `— ⚠reject` |  | Mechanical tagged-record/vector resolver; the tag meanings and the semantic meaning of the returned target pair are not established by this body. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044acd0` | `— ⚠reject` |  | Maps record tags to literal port-kind names. The owning class and relationship to the handle/id tree are not established here. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044ae53` | `— ⚠reject` |  | Mechanical indexed record resolver; the meaning of the tag and virtual result remain unknown. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044d7e1` | `— ⚠reject` |  | Region serialisation used by the invalidate/validate/set-region methods of the window object. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044d923` | `— ⚠reject` |  | Region invalidation path of the Win32 window object; window-object ownership is a hypothesis from the neighbouring GWindowWin32 string, not bound here. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044d9f7` | `— ⚠reject` |  | Region validate counterpart used when the window stops clipping an area; ownership hypothesis as for 0044d923. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044dac9` | `— ⚠reject` |  | Rounded/irregular window shaping for the Win32 window object. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044db39` | `— ⚠reject` |  | Clears a cached record id, the matching element of the holder's list and the cached object; the 'drag state' reading is a hypothesis (the owners at 0044f252/0044f325 and the flag flow are not proven here). | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044dc06` | `— ⚠reject` |  | Detaches the cached object from its owner's back-pointer at +0xf0 and releases it; same drag-state hypothesis as 0044db39. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044dcd9` | `— ⚠reject` |  | push_back-style member of one vector instantiation. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044ddfb` | `— ⚠reject` |  | push_back member of a second vector instantiation. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044deb3` | `— ⚠reject` |  | push_back member of a fourth vector instantiation. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044df08` | `— ⚠reject` |  | push_back member of the counted-assign vector instantiation. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044dfe8` | `— ⚠reject` |  | Clone path used when the same descriptor class is instantiated at 0044fae8/0044fb65/0044fbe6 (outside this shard), i.e. one of the input-device/port descriptor records of this translation unit. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |

