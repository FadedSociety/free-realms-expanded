# Class: SoeUtil::StringFixed<128>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00402950` | `SoeUtil::StringFixed128Ctor` | ✅ | ctor | Constructor for the StringFixed<128> string class: runs the IStringFixed<char,128> base constructor 00401f20 under an SEH frame, then stores the StringFixed<128> vtable at +0 and returns this. |
| `00402ad0` | `SoeUtil::StringFixed128DeletingDtor` | 🟢 | dtor | SEH-framed scalar deleting destructor: runs the IStringFixed<char,128> destructor body FUN_00401fd0, then frees the object with _free when the flag argument's low bit is set; restores the exception list before returning. |
