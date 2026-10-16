# Class: SoeUtil::IStringFixed<unsigned_short,512>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d00ba0` | `WideStringFixed512Initialize` | ✅ | ctor | Initializes the wide IString base with its static empty buffer, clears length and capacity fields, then installs the 512-unit fixed-string vtable. |
| `00d00d50` | `WideStringFixed512ScalarDeletingDtor` | ✅ | dtor | Runs fixed wide-string cleanup and frees the string object only when deleting-flag bit 0 is set; otherwise returns the original pointer. |
