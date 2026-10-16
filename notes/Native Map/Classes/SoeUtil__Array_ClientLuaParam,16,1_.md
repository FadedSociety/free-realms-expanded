# Class: SoeUtil::Array<ClientLuaParam,16,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0081c650` | `ArrayClientLuaParam16_DtorBody` | ✅ | dtor | Destructor body called from an EH unwind funclet (does not free the object): restores the Array<ClientLuaParam,16,1> vtable, calls 0070b4e0 on the used count at [2] (element destruction), frees the heap buffer through 0… |
| `0081c6f0` | `ArrayClientLuaParam16_StorageSelect` | ✅ | virtual | Thiscall (ecx=this) with two stack args (count, out pointer). When count is above 16 it returns the result of 00708950 (ArrayGrow). Otherwise it stores 16 to *out and returns the inline buffer address (this+0x17)&~7. |
| `0081dac0` | `ArrayClientLuaParam16_Ctor` | ✅ | ctor | Fastcall constructor (no args): stores the Array<ClientLuaParam,0,1> vtable at [0], zeroes [1], [2] and [3], then stores the Array<ClientLuaParam,16,1> vtable at [0]. Returns nothing. |
| `0081db40` | `ArrayClientLuaParam16_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, one stack byte flag): calls 0081c650 (the Array<ClientLuaParam,16,1> destructor body) on this, and frees this through the _free thunk when flag bit 0 is set. Returns this. |
| `0081dc10` | `— ⚠reject` | ⚪ | virtual | Virtual slot [3] (thiscall, one stack arg: the other array). Calls 00709410 with the other array's data pointer and count and 0070b4e0 on the other array's element count, does the same for this array, then clears with 0… |
