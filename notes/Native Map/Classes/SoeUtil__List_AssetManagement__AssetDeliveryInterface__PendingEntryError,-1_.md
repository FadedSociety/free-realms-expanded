# Class: SoeUtil::List<AssetManagement::AssetDeliveryInterface::PendingEntryError,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `010990a0` | `AllocNode0x28` | 🟢 | virtual | Same body as 01099040 (57 bytes): calls FUN_006fb9c0 and FUN_006fb860; manager->vtbl[1](0x28,4) when a thread-local manager is present, else the global manager vtbl[1](0x28,4), else FUN_006fb950(0x28,4); returns the poi… |
| `010990e0` | `FreeNodeAlign4` | 🟢 | virtual | Stdcall-shaped (one stack arg, ret 4; ECX unused): pushes 4 and the argument and calls FreeMemory (00401c00), then returns. |
| `0109b9c0` | `List::~List → SoeUtil::List<AssetManagement::AssetDeliveryInterface::PendingEnt…` | ✅ | dtor | Destructor body of the -1 variant PendingEntryError list: installs the vtable and drains the node list by calling 01093f70 on each head node. |
| `0109baa0` | `List::ScalarDeletingDestructor → SoeUtil::List<AssetManagement::AssetDeliveryIn…` | ✅ | dtor | Scalar deleting destructor shape: installs the vtable, drains the node list through 01093f70, and calls _free when bit 0 of the stack argument is set; returns the object pointer. |
