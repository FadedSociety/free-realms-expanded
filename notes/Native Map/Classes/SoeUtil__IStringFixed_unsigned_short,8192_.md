# Class: SoeUtil::IStringFixed<unsigned_short,8192>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0099e990` | `IStringFixed8192Ctor` | 🟢 | ctor | Installs the generic UTF-16 IString vtable, stores the shared empty-data pointer at +4, zeros fields at +8 and +0xC, then installs the fixed-8192 vtable. |
| `0099e9f0` | `IStringFixed8192Allocate` | ✅ | virtual | For requests below 0x4005, clears a fallback flag, returns capacity 0x4004 and inline storage at this+0x10; otherwise calls the fallback allocator. |
| `0099ea40` | `— ⚠reject` | ⚪ | dtor | Atomically releases owned string storage, resets the fields to shared empty storage, then restores the generic IString vtable. |
| `0099ec00` | `IStringFixed8192ScalarDeletingDtor` | ✅ | virtual | Calls the fixed-8192 string destructor, conditionally frees the object for the deleting flag, and returns the original pointer. |
