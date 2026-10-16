# Class: SoeUtil::List<SoeUtil::StringFixed<256>>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bbfe0` | `AllocateListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x11c bytes aligned to four through the thread manager, global manager, or aligned CRT fallback. |
| `008bc020` | `FreeListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
