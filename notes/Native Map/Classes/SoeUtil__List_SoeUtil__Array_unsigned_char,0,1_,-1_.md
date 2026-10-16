# Class: SoeUtil::List<SoeUtil::Array<unsigned_char,0,1>,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00897c10` | `ByteArrayListAllocateNode` | ✅ | method | Allocates a 0x18-byte list node through the current-thread memory manager, then the global manager, or FUN_006fb950(0x18,4) as fallback; the second allocator argument is 4. |
| `00897c50` | `ByteArrayListFreeNode` | ✅ | method | Thin free wrapper: releases param_1 through FreeMemory with alignment 4 and returns nothing. |
| `00898de0` | `SoeUtil::ByteArrayListDtor` | ✅ | dtor | Destructor body of the byte-array list: installs its vtable and runs the node-release routine FUN_00898910; reached only from EH unwind funclets. |
| `00898df0` | `SoeUtil::ByteArrayListScalarDtor` | ✅ | dtor | Scalar deleting destructor of the byte-array list: installs the list vtable, runs FUN_00898910 to release all nodes, and calls _free on the list object when bit 0 of param_2 is set. |
