# Class: SoeUtil::StringFixed<8192>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00402f10` | `SoeUtil::StringFixed8192Ctor` | ✅ | ctor | Constructor for the StringFixed<8192> string class: runs the IStringFixed<char,8192> base constructor 004025e0 under an SEH frame, then stores the StringFixed<8192> vtable at +0 and returns this. |
| `00402f70` | `SoeUtil::StringFixed8192DeletingDtor` | 🟢 | dtor | SEH-framed scalar deleting destructor: runs the IStringFixed<char,8192> destructor body FUN_00402690, then frees the object with _free when the flag argument's low bit is set. |
| `00ae52f0` | `SoeUtil::StringFixed8192::Ctor` | ✅ | ctor | Calls the fixed-string base initializer with the supplied source, installs the StringFixed<char,8192> vtable, and returns the destination. |
