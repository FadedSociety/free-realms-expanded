# Class: SoeUtil::HashListMap<int,AssetManagement::AssetDeliveryIndirect::PendingRequest,8192,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01098c90` | `HashListMap8192Ctor` | ✅ | ctor | Fastcall (ECX=this), no stack args, returns this. Stores the HashListMap vtable at this, zeroes this+4, this+8, this+0xc and this+0x10, memsets the 8192-entry bucket array at this+0x14 with 0x8000 zero bytes, and return… |
| `01099140` | `AllocNode0x74` | 🟢 | virtual | Thiscall (ECX=map), no stack args, returns the pointer in EAX. Calls FUN_006fb9c0 and FUN_006fb860 (thread-local manager lookup); manager vtbl[1](0x74,4) when a manager is present, else the global manager DAT_01bdf280 v… |
| `01099180` | `FreeNodeAlign4` | 🟢 | virtual | Stdcall-shaped (one stack arg, ret 4; ECX unused): pushes 4 and the argument and calls FreeMemory (00401c00), then returns. |
| `0109c8b0` | `HashListMap::~HashListMap8192` | ✅ | dtor | Destructor body of the -1 variant PendingRequest map: installs the -1 vtable and drains the node list by calling 0109b9f0 on each head node. |
| `0109c910` | `HashListMap::ScalarDeletingDestructor8192` | ✅ | dtor | Scalar deleting destructor shape: installs the -1 vtable, drains the node list through 0109b9f0, then calls _free on the object when bit 0 of the stack argument is set. |
