# Class: SoeUtil::Hash<SoeUtil::IString<char>,16384,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098bf30` | `HashString16384::Method1Adjustor` | ✅ | virtual | Adds 0x1000c to ECX and jumps to 007411c0; the vtable also lists HashMap<int,IString<char>,16384,0>. |
| `0098bf40` | `HashString16384::Method2Adjustor` | ✅ | virtual | Adds 0x1000c to ECX and jumps to 00741230; the vtable also lists HashMap<int,IString<char>,16384,0>. |
| `0098c6b0` | `HashString16384::Constructor` | ✅ | ctor | Constructs the pooled 16384-bucket string-map state, installs the Hash<IString,16384,0> vtable, and returns this. |
| `0098c770` | `HashString16384::ScalarDeletingDestructor` | ✅ | dtor | Calls the pooled 16384-bucket string-map cleanup body; frees this only when deleting flag bit 0 is set, otherwise returns this. |
