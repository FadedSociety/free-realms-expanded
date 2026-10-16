# Class: SoeUtil::IStringFixed<unsigned_short,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0099dfe0` | `IStringFixed8Ctor` | ✅ | ctor | Initializes IString fields to the shared empty-string object, clears length and ownership metadata, and installs the fixed-8 vtable. |
| `0099e040` | `IStringFixed8Allocate` | ✅ | virtual | For requests below 0x15, clears a flag, returns capacity 0x14 and inline storage at this+0x10; otherwise calls the fallback allocator. |
| `0099e090` | `— ⚠reject` | ⚪ | dtor | Atomically releases owned string storage, resets the object to shared empty storage, and restores its generic IString base vtable. |
| `0099e6c0` | `IStringFixed8ScalarDeletingDtor` | ✅ | virtual | Calls the fixed-8 string destructor, frees the object when the deleting flag is set, and returns its pointer. |
| `0099f070` | `IStringFixed8StackDtor` | 🟢 | dtor | Calls 0099E090 with the incoming stack-object pointer in ECX; the cookie-shaped Ghidra argument is SEH bookkeeping, not a destructor argument. |
