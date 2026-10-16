# Class: SoeUtil::Hash<SoeUtil::IString<char>,1024,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098c000` | `HashString1024::Method1Adjustor` | ✅ | virtual | Adds 0x100c to ECX and jumps to 007411c0; the vtable also lists HashMap<int,IString<char>,1024,0>. |
| `0098c010` | `HashString1024::Method2Adjustor` | ✅ | virtual | Adds 0x100c to ECX and jumps to 00741230; the vtable also lists HashMap<int,IString<char>,1024,0>. |
| `0098c710` | `HashString1024::Constructor` | ✅ | ctor | Constructs the pooled 1024-bucket string-map state, installs the Hash<IString,1024,0> vtable, and returns this. |
| `0098c7d0` | `HashString1024::ScalarDeletingDestructor` | ✅ | dtor | Calls the pooled 1024-bucket string-map cleanup body; frees this only when deleting flag bit 0 is set, otherwise returns this. |
