# Class: SoeUtil::IStringFixed<char,128>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00401f20` | `SoeUtil::IStringFixed128Ctor` | ✅ | ctor | Constructor body for IStringFixed<char,128>: first performs the IString base initialization (vtable, static empty buffer, zero length/capacity), then overwrites the vtable with the IStringFixed<char,128> vtable. |
| `00401f80` | `SoeUtil::IStringFixed128GetBuffer` | 🟢 | virtual | Buffer-request entry for IStringFixed<char,128>: when the requested size is under 0x85 it reports capacity 0x84 and the non-heap flag and returns the inline buffer at +0x10; otherwise it defers to the heap allocation ro… |
| `00401fb0` | `SoeUtil::IStringFixed128ReleaseBuffer` | 🟢 | virtual | Buffer-release entry for IStringFixed<char,128>: a pointer equal to the object's inline buffer at +0x10 is ignored; any other pointer is freed through the global free entry 00401c00 with alignment 0. |
| `00401fd0` | `SoeUtil::IStringFixed128Dtor` | ✅ | dtor | Destructor body for IStringFixed<char,128>: under an SEH frame it rewrites the fixed vtable, releases the owned buffer when the capacity field is positive (refcount decrement, vtable-slot-8 free when last), resets the b… |
| `00402530` | `SoeUtil::IStringFixed128DeletingDtor` | 🟢 | dtor | Scalar deleting destructor: runs the IStringFixed<char,128> destructor body FUN_00401fd0, then frees the object with _free when the flag argument's low bit is set; returns the pointer. |
| `00402900` | `SoeUtil::IStringFixed128DtorThunk` | 🟢 | dtor | SEH-framed cleanup wrapper for the IStringFixed<char,128> destructor body FUN_00401fd0; sets up and restores a scoped exception frame around the call. |
