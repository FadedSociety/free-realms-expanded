# Class: SoeUtil::Array<bool,32,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bf69f0` | `BoolArray32FreeOutOfLine` | 🟢 | virtual | SoeUtil::Array<bool,32,1>::vftable[2]. When stack arg1 differs from this+0x10 (the inline 32-element storage), calls 00401c00 on it with alignment 1; returns with ret 8. |
| `00bf6a90` | `BoolArray32DtorBody` | ✅ | dtor | Destructor body. Stores SoeUtil::Array<bool,32,1>::vftable, sets count (+0x8) to 0, frees the data pointer (+0x4) with FUN_00401c00 when it is not the inline buffer at +0x10, then zeroes data, stores the SoeUtil::Array<… |
| `00bf6b20` | `BoolArray32CapacityOrInline` | 🟢 | virtual | SoeUtil::Array<bool,32,1>::vftable[1]. When stack arg1 > 0x20 it writes arg1 back and tail-jumps to 007e84f0 with the same arguments. Otherwise stores 0x20 through stack arg3 and returns this+0x10 (the inline storage); … |
| `00bf6b70` | `BoolArray32Ctor` | ✅ | ctor | Constructor body. Stores SoeUtil::Array<bool,0,1>::vftable, zeroes data (+0x4), count (+0x8) and capacity (+0xc), then stores SoeUtil::Array<bool,32,1>::vftable. The inline buffer pointer is not set here. |
| `00bf6bd0` | `BoolArray32ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls 00bf6a90 (dtor body) on ECX, frees the object when bit 0 of the stack argument is set, returns this (ret 4). |
| `00bf6c50` | `— ⚠reject` | ⚪ | virtual | SoeUtil::Array<bool,32,1>::vftable[3]. Calls 009966e0(arg2), resets arg2's count to 0, then copies this array's bytes (count at +0x8, data at +0x4) to arg2's buffer through _memcpy, growing arg2 via 00869290 when its ca… |
