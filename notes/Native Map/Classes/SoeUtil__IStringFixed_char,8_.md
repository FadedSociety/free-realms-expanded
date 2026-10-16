# Class: SoeUtil::IStringFixed<char,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c6dd0` | `SoeUtil_IStringFixed8::Ctor` | ✅ | ctor | Stores SoeUtil::IString<char>::vftable at this+0, points this+4 at the static empty buffer DAT_01be48fc, zeroes this+8 and this+0xc, then stores SoeUtil::IStringFixed<char,8>::vftable at this+0. Returns void (fastcall). |
| `00b2a940` | `IStringFixed8::Constructor` | ✅ | ctor | Initializes base IString storage and writes the fixed 8-byte string specialization vtable. |
| `00d6b2e0` | `IStringFixed8::Construct` | ✅ | ctor | Sets the inline empty buffer, zeroes length and capacity, installs the IStringFixed<char,8> vtable, and passes the supplied argument to FUN_006f9e60. |
