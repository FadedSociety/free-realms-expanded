# Class: SoeUtil::StringFixed<2048>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00402a70` | `SoeUtil::StringFixed2048Ctor` | ✅ | ctor | Constructor for the StringFixed<2048> string class: runs the IStringFixed<char,2048> base constructor 00402370 under an SEH frame, then stores the StringFixed<2048> vtable at +0 and returns this. |
| `00402bf0` | `SoeUtil::StringFixed2048DeletingDtor` | 🟢 | dtor | SEH-framed scalar deleting destructor: runs the IStringFixed<char,2048> destructor body FUN_00402420, then frees the object with _free when the flag argument's low bit is set. |
| `008e0020` | `StringFixed2048_Ctor` | ✅ | ctor | Calls the base string initializer and writes the StringFixed<2048> vtable to the object. |
| `009948c0` | `StringFixed2048Ctor` | ✅ | ctor | Constructs the underlying fixed 2048-character string, then installs the StringFixed<2048> vtable and returns the object. |
