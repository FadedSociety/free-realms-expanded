# Class: SoeUtil::StringFixed<8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d3840` | `StringFixed8::StringFixed8` | ✅ | ctor | Initializes the embedded base string through FUN_008c6dd0, then stores the SoeUtil::StringFixed<8> vtable at the object start and returns the object pointer. |
| `00b2c550` | `StringFixed8::Constructor` | ✅ | ctor | Calls the fixed-capacity IString constructor and writes the StringFixed<8> vtable. |
