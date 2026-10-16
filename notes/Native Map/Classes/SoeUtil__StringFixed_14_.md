# Class: SoeUtil::StringFixed<14>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a55500` | `SoeUtil::StringFixed14_Allocate` | ✅ | virtual | For requested sizes below 0x13, returns inline storage at this+0x10 with capacity 0x12; for larger sizes, tail-jumps to FUN_00401cb0. |
| `00a55530` | `SoeUtil::StringFixed14_Free` | ✅ | virtual | Skips freeing the inline buffer at this+0x10; otherwise forwards the supplied pointer to FUN_00401c00 with alignment zero. |
| `00a57ea0` | `SoeUtil::StringFixed14_Ctor` | ✅ | ctor | Constructs the embedded fixed IString<char,14> member, then installs the StringFixed<14> vtable and returns the object. |
| `00a57f00` | `SoeUtil::StringFixed14_ScalarDeletingDtor` | ✅ | dtor | Calls the embedded string destructor and, when flag bit 0 is set, frees the containing object; otherwise returns the object pointer. |
| `00a59f20` | `StringFixed14Ctor` | ✅ | ctor | Delegates initialization to the string helper, then installs the StringFixed<14> vtable. |
