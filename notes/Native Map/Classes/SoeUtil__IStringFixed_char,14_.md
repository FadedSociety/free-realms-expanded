# Class: SoeUtil::IStringFixed<char,14>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a554a0` | `SoeUtil::IStringFixed14_Ctor` | ✅ | ctor | Initializes the base IString<char> vtable, points storage at the fixed global buffer, clears length and capacity fields, then installs the 14-character fixed-string vtable. |
| `00a55550` | `— ⚠reject` | ⚪ | dtor | Drops the fixed-string buffer reference count and frees it through vtable slot 2 when its count reaches zero, resets the inline fields, then releases the base IString storage reference. |
| `00a56370` | `SoeUtil::IStringFixed14_ScalarDeletingDtor` | ✅ | dtor | Calls the fixed-string destructor, then frees the object when scalar-delete flag bit 0 is set; otherwise returns its address. |
| `00a58770` | `SoeUtil::IStringFixed14_InitFromIString` | ✅ | function | Initializes the base string member and fixed 14-character vtable, then passes param_2 to FUN_006f9e60 for content assignment. |
