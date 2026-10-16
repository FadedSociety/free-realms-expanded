# Class: SoeUtil::StringFixed<255>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d39c0` | `StringFixed255::StringFixed255` | ✅ | ctor | Initializes the embedded fixed-string base through FUN_008c7e00, then stores the SoeUtil::StringFixed<255> vtable at the object start and returns the object pointer. |
| `008d3b00` | `StringFixed255_ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (slot [0] of the StringFixed<255> vtable): runs the fixed-string record teardown FUN_008c7eb0 (same routine the IStringFixed<char,255> record uses) inside an SEH frame and frees the objec… |
| `008e0280` | `StringFixed255_Ctor` | ✅ | ctor | Calls the base string initializer and writes the StringFixed<255> vtable to the destination. |
