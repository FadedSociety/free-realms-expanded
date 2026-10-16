# Class: SoeUtil::IStringFixed<char,2048>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00402370` | `SoeUtil::IStringFixed2048Ctor` | ✅ | ctor | Constructor body for IStringFixed<char,2048>: performs the IString base initialization, then overwrites the vtable with the IStringFixed<char,2048> vtable. |
| `004023d0` | `SoeUtil::IStringFixed2048GetBuffer` | 🟢 | virtual | Buffer-request entry for IStringFixed<char,2048>: requests under 0x805 return the inline buffer at +0x10 with capacity 0x804 and the non-heap flag; larger requests defer to the heap routine 00401cb0. |
| `00402400` | `SoeUtil::IStringFixed2048ReleaseBuffer` | 🟢 | virtual | Buffer-release entry for IStringFixed<char,2048>: the inline buffer at +0x10 is ignored; any other pointer is freed through the global free entry 00401c00 with alignment 0. |
| `00402420` | `SoeUtil::IStringFixed2048Dtor` | ✅ | dtor | Destructor body for IStringFixed<char,2048>: under an SEH frame rewrites the fixed vtable, releases the owned buffer when capacity is positive, resets to the static empty string, then stores the IString base vtable and … |
| `00402590` | `SoeUtil::IStringFixed2048DeletingDtor` | 🟢 | dtor | Scalar deleting destructor: runs the IStringFixed<char,2048> destructor body FUN_00402420, then frees the object with _free when the flag argument's low bit is set. |
| `00402860` | `SoeUtil::IStringFixed2048DtorThunk` | 🟢 | dtor | SEH-framed cleanup wrapper for the IStringFixed<char,2048> destructor body FUN_00402420; sets up and restores a scoped exception frame around the call. |
| `008d3c60` | `Ctor_IStringFixed2048` | 🟢 | ctor | Constructor of a fixed 2048-byte string: initializes the SoeUtil::IString<char> base (vtable, empty buffer 01be48fc, zero length/capacity), overwrites the vtable with SoeUtil::IStringFixed<char,2048>::vftable and assign… |
| `00994740` | `StringFixed2048Ctor` | ✅ | ctor | Installs the string interface and fixed-2048 vtables, initializes buffer and length fields, and invokes the fixed-string initialization helper with the supplied argument. |
