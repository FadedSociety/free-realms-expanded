# Class: SoeUtil::IStringFixed<char,64>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00402200` | `SoeUtil::IStringFixed64Ctor` | ✅ | ctor | Constructor body for IStringFixed<char,64>: performs the IString base initialization, then overwrites the vtable with the IStringFixed<char,64> vtable. |
| `00402260` | `SoeUtil::IStringFixed64GetBuffer` | 🟢 | virtual | Buffer-request entry for IStringFixed<char,64>: requests under 0x45 return the inline buffer at +0x10 with capacity 0x44 and the non-heap flag; larger requests defer to the heap routine 00401cb0. |
| `00402290` | `SoeUtil::IStringFixed64ReleaseBuffer` | 🟢 | virtual | Buffer-release entry for IStringFixed<char,64>: the inline buffer at +0x10 is ignored; any other pointer is freed through the global free entry 00401c00 with alignment 0. |
| `004022b0` | `SoeUtil::IStringFixed64Dtor` | ✅ | dtor | Destructor body for IStringFixed<char,64>: under an SEH frame rewrites the fixed vtable, releases the owned buffer when capacity is positive, resets to the static empty string, then stores the IString base vtable and re… |
| `00402570` | `SoeUtil::IStringFixed64DeletingDtor` | 🟢 | dtor | Scalar deleting destructor: runs the IStringFixed<char,64> destructor body FUN_004022b0, then frees the object with _free when the flag argument's low bit is set. |
| `00402810` | `SoeUtil::IStringFixed64DtorThunk` | 🟢 | dtor | SEH-framed cleanup wrapper: sets up a scoped exception frame, calls the IStringFixed<char,64> destructor body FUN_004022b0 (the object pointer is not recovered by the decompiler, so the call shows a stack-cookie express… |
| `009627e0` | `IStringFixed64CopyCtor` | ✅ | ctor | Installs the IString base state with the fixed empty-string pointer, switches to the IStringFixed<char,64> vtable, and calls FUN_006f9e60 with the source value. |
| `00962c80` | `IStringFixed64CopyCtor` | ✅ | ctor | Installs the IString base state and fixed empty-string pointer, switches to IStringFixed<char,64>, calls FUN_006f9e60 with the supplied value, and returns the destination. |
