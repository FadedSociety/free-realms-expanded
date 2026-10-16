# Class: SoeUtil::StringFixed<256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `004029b0` | `SoeUtil::StringFixed256Ctor` | ✅ | ctor | Constructor for the StringFixed<256> string class: runs the IStringFixed<char,256> base constructor 00402090 under an SEH frame, then stores the StringFixed<256> vtable at +0 and returns this. |
| `00402b30` | `SoeUtil::StringFixed256DeletingDtor` | 🟢 | dtor | SEH-framed scalar deleting destructor: runs the IStringFixed<char,256> destructor body FUN_00402140, then frees the object with _free when the flag argument's low bit is set. |
| `008dff60` | `StringFixed256_Ctor` | ✅ | ctor | Calls the IString constructor on the supplied destination, then writes the StringFixed<256> vtable and returns the destination. |
| `00b2d650` | `StringFixed256::Constructor` | ✅ | ctor | Initializes fixed 256-byte IString storage from the argument and writes the StringFixed<256> vtable. |
