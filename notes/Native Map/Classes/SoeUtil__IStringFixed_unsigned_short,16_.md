# Class: SoeUtil::IStringFixed<unsigned_short,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0099dde0` | `IStringFixed16Ctor` | ✅ | ctor | Initializes the generic IString fields to the shared empty string, zeroes length and ownership metadata, then installs the fixed-16 vtable. |
| `0099de40` | `IStringFixed16Allocate` | ✅ | virtual | For requests below 0x25, clears the fallback flag, returns capacity 0x24 and inline buffer at this+0x10; otherwise calls FUN_006FACC0. |
| `0099de90` | `IStringFixed16Dtor` | ✅ | dtor | Atomically releases the owned buffer when length is positive, resets the fields to static empty storage, then restores the generic IString vtable. |
| `0099e6a0` | `IStringFixed16ScalarDeletingDtor` | ✅ | virtual | Calls the fixed-string destructor, frees the object when the deleting flag is set, and returns its original pointer. |
| `0099f100` | `IStringFixed16StackDtor` | 🟢 | dtor | Calls 0099DE90 with the incoming stack-object pointer in ECX; the cookie-shaped Ghidra argument is SEH bookkeeping, not a destructor argument. |
