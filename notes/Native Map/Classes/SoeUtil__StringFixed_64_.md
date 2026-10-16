# Class: SoeUtil::StringFixed<64>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00402a10` | `SoeUtil::StringFixed64Ctor` | ✅ | ctor | Constructor for the StringFixed<64> string class: runs the IStringFixed<char,64> base constructor 00402200 under an SEH frame, then stores the StringFixed<64> vtable at +0 and returns this. |
| `00402b90` | `SoeUtil::StringFixed64DeletingDtor` | 🟢 | dtor | SEH-framed scalar deleting destructor: runs the IStringFixed<char,64> destructor body FUN_004022b0, then frees the object with _free when the flag argument's low bit is set. |
| `00964030` | `StringFixed64CopyCtor` | ✅ | ctor | Calls FUN_009627e0 to copy the string value, then installs the StringFixed<64> vtable and returns param_1. |
| `00964840` | `StringFixed64CopyCtor` | ✅ | ctor | Calls FUN_00962c80 to copy the source value into the fixed-string base, then installs the StringFixed<64> vtable and returns the destination. |
