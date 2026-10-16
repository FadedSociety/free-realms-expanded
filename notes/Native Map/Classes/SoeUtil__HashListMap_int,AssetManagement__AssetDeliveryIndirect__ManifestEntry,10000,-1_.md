# Class: SoeUtil::HashListMap<int,AssetManagement::AssetDeliveryIndirect::ManifestEntry,10000,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01098d00` | `HashListMap10000Ctor` | ✅ | ctor | Fastcall (ECX=this), no stack args, returns this. Stores the HashListMap vtable (01884d24) at this, zeroes this+4, this+8, this+0xc and this+0x10, memsets the 10000-entry bucket array at this+0x14 with 40000 zero bytes,… |
| `01099220` | `AllocNode0x6c` | 🟢 | virtual | Thiscall (ECX=map), no stack args, returns the pointer in EAX. Calls FUN_006fb9c0 and FUN_006fb860 (thread-local manager lookup); manager vtbl[1](0x6c,4) when a manager is present, else the global manager DAT_01bdf280 v… |
| `01099260` | `FreeNodeAlign4` | 🟢 | virtual | Stdcall-shaped (one stack arg, ret 4; ECX unused): pushes 4 and the argument and calls FreeMemory (00401c00), then returns. |
| `0109c8e0` | `HashListMap::~HashListMap10000` | ✅ | dtor | Destructor body of the -1 variant ManifestEntry map: installs its vtable and drains the node list by calling 0109ad50 on each head node until the head is null. |
| `0109c950` | `HashListMap::ScalarDeletingDestructor10000` | ✅ | dtor | Scalar deleting destructor shape: installs the -1 vtable, drains the node list through 0109ad50, then calls _free on the object when bit 0 of the stack argument is set. |
| `0109c990` | `HashListMap::~HashListMap10000Framed` | ✅ | dtor | SEH-framed destructor body of the -1 variant manifest map: installs its vtable and drains the node list by calling 0109ad50 on each head node. |
| `0109d4b0` | `HashListMap::HashListMap10000` | ✅ | ctor | Constructor: installs the -1 map vtable, zeroes the words at +4, +8, +0xc and +0x10, clears the 40000-byte bucket table at +0x14 with _memset, and installs the HashList vtable 018854b8. |
