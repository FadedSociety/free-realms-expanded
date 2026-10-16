# Class: SoeUtil::List<AssetManagement::AssetDeliveryInterface::PendingEntryData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01099040` | `AllocNode0x28` | 🟢 | virtual | Thiscall (ECX=list), no stack args, returns the pointer in EAX. Calls FUN_006fb9c0, then FUN_006fb860 (thread-local manager lookup); when a manager is present calls its vtbl[1](0x28,4) and returns the result; else the g… |
| `01099080` | `FreeNodeAlign4` | 🟢 | virtual | Stdcall-shaped (one stack arg, ret 4; ECX unused): pushes 4 and the argument and calls FreeMemory (00401c00), then returns. |
| `0109ab20` | `List::~List → SoeUtil::List<AssetManagement::AssetDeliveryInterface::PendingEnt…` | ✅ | dtor | Destructor body of the -1 variant list: sets the list vtable, then drains the node list by calling 01093ee0 on each head node until the head is null. |
| `0109acb0` | `List::ScalarDeletingDestructor → SoeUtil::List<AssetManagement::AssetDeliveryIn…` | ✅ | dtor | Scalar deleting destructor shape: sets the list vtable, drains the node list through 01093ee0, and calls _free on the object when bit 0 of the stack argument is set. |
