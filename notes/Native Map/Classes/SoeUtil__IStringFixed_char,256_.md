# Class: SoeUtil::IStringFixed<char,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00402090` | `SoeUtil::IStringFixed256Ctor` | ✅ | ctor | Constructor body for IStringFixed<char,256>: performs the IString base initialization (vtable, static empty buffer, zero length/capacity), then overwrites the vtable with the IStringFixed<char,256> vtable. |
| `004020f0` | `SoeUtil::IStringFixed256GetBuffer` | 🟢 | virtual | Buffer-request entry for IStringFixed<char,256>: requests under 0x105 return the inline buffer at +0x10 with capacity 0x104 and the non-heap flag; larger requests defer to the heap routine 00401cb0. |
| `00402120` | `SoeUtil::IStringFixed256ReleaseBuffer` | 🟢 | virtual | Buffer-release entry for IStringFixed<char,256>: the inline buffer at +0x10 is ignored; any other pointer is freed through the global free entry 00401c00 with alignment 0. |
| `00402140` | `SoeUtil::IStringFixed256Dtor` | ✅ | dtor | Destructor body for IStringFixed<char,256>: under an SEH frame rewrites the fixed vtable, releases the owned buffer when capacity is positive, resets to the static empty string, then stores the IString base vtable and r… |
| `00402550` | `SoeUtil::IStringFixed256DeletingDtor` | 🟢 | dtor | Scalar deleting destructor: runs the IStringFixed<char,256> destructor body FUN_00402140, then frees the object with _free when the flag argument's low bit is set. |
| `004028b0` | `SoeUtil::IStringFixed256DtorThunk` | 🟢 | dtor | SEH-framed cleanup wrapper for the IStringFixed<char,256> destructor body FUN_00402140; sets up and restores a scoped exception frame around the call. |
| `008d3b80` | `Ctor_IStringFixed256` | 🟢 | ctor | Constructor of a fixed 256-byte string: initializes the SoeUtil::IString<char> base (vtable, empty buffer 01be48fc, zero length/capacity), overwrites the vtable with SoeUtil::IStringFixed<char,256>::vftable and assigns … |
| `00b2af60` | `IStringFixed256::Constructor` | ✅ | ctor | Initializes the IString base and writes the fixed 256-byte string specialization vtable. |
