# Class: SoeUtil::Array<unsigned __int64,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd9b0` | `UInt64Array::FreeBuffer` | ✅ | virtual | Calls the engine free routine with the supplied pointer and 8-byte alignment. |
| `008d4bd0` | `CopyCtor_ArrayUInt64_0` | 🟢 | ctor | Copy constructor of the growable unsigned-64-bit array: installs the SoeUtil::Array<unsigned __int64,0,1> vtable, zeroes pointer/count/capacity, then when the source count at +8 is positive reserves capacity through FUN… |
