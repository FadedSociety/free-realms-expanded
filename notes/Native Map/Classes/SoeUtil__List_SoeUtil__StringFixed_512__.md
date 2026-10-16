# Class: SoeUtil::List<SoeUtil::StringFixed<512>>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bc040` | `AllocateListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x21c bytes aligned to four through the thread manager, global manager, or aligned CRT fallback. |
| `008bc080` | `FreeListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
