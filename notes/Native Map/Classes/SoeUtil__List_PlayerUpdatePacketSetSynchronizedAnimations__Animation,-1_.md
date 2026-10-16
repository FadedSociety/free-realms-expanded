# Class: SoeUtil::List<PlayerUpdatePacketSetSynchronizedAnimations::Animation,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bcd10` | `AllocateListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x18 bytes aligned to eight through the thread manager, global manager, or aligned CRT fallback. |
| `008bcd50` | `FreeListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 8. |
| `008e09e0` | `ListPlayerUpdatePacketSetSynchronizedAnimationsAnimation_Dtor` | ✅ | dtor | Stores the Animation list vtable, then tail-jumps to the list cleanup routine. |
| `008e2120` | `ListSynchronizedAnimation_ScalarDeletingDtor` | ✅ | dtor | Installs the Animation list vtable, runs list cleanup, and optionally frees the object when flag bit 0 is set. |
